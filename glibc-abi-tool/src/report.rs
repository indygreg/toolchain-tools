// Copyright 2022 Gregory Szorc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::abilist::{SymbolType, VersionedAbiLists};
use crate::repo::{GlibcVersion, Repo, Tag};
use anyhow::{Result, anyhow};
use askama::Template;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct DistroVersion {
    pub name: &'static str,
    pub version: &'static str,
    pub version_name: Option<&'static str>,
    pub glibc_version: GlibcVersion,
}

impl DistroVersion {
    pub fn name_string(&self) -> String {
        let suffix = if let Some(v) = &self.version_name {
            format!(" ({})", v)
        } else {
            "".to_string()
        };

        format!("{} {}{}", self.name, self.version, suffix)
    }
}

const DISTRO_GLIBC_VERSIONS: &[DistroVersion] = &[
    DistroVersion {
        name: "debian",
        version: "7",
        version_name: Some("wheezy"),
        glibc_version: GlibcVersion {
            major: 2,
            minor: 13,
            patch: None,
        },
    },
    DistroVersion {
        name: "debian",
        version: "8",
        version_name: Some("jessie"),
        glibc_version: GlibcVersion {
            major: 2,
            minor: 19,
            patch: None,
        },
    },
    DistroVersion {
        name: "debian",
        version: "9",
        version_name: Some("stretch"),
        glibc_version: GlibcVersion {
            major: 2,
            minor: 24,
            patch: None,
        },
    },
    DistroVersion {
        name: "debian",
        version: "10",
        version_name: Some("buster"),
        glibc_version: GlibcVersion {
            major: 2,
            minor: 28,
            patch: None,
        },
    },
    DistroVersion {
        name: "debian",
        version: "11",
        version_name: Some("bullseye"),
        glibc_version: GlibcVersion {
            major: 2,
            minor: 31,
            patch: None,
        },
    },
    DistroVersion {
        name: "debian",
        version: "12",
        version_name: Some("bookworm"),
        glibc_version: GlibcVersion {
            major: 2,
            minor: 36,
            patch: None,
        },
    },
    DistroVersion {
        name: "ubuntu",
        version: "14.04",
        version_name: None,
        glibc_version: GlibcVersion {
            major: 2,
            minor: 19,
            patch: None,
        },
    },
    DistroVersion {
        name: "ubuntu",
        version: "16.04",
        version_name: None,
        glibc_version: GlibcVersion {
            major: 2,
            minor: 23,
            patch: None,
        },
    },
    DistroVersion {
        name: "ubuntu",
        version: "18.04",
        version_name: None,
        glibc_version: GlibcVersion {
            major: 2,
            minor: 27,
            patch: None,
        },
    },
    DistroVersion {
        name: "ubuntu",
        version: "20.04",
        version_name: None,
        glibc_version: GlibcVersion {
            major: 2,
            minor: 31,
            patch: None,
        },
    },
    DistroVersion {
        name: "ubuntu",
        version: "22.04",
        version_name: None,
        glibc_version: GlibcVersion {
            major: 2,
            minor: 35,
            patch: None,
        },
    },
    DistroVersion {
        name: "ubuntu",
        version: "24.04",
        version_name: None,
        glibc_version: GlibcVersion {
            major: 2,
            minor: 39,
            patch: None,
        },
    },
    DistroVersion {
        name: "ubuntu",
        version: "26.04",
        version_name: None,
        glibc_version: GlibcVersion {
            major: 2,
            minor: 43,
            patch: None,
        },
    },
];

#[derive(Template)]
#[template(path = "glibc-arch.html", ext = "html")]
struct GlibcArchTemplate {
    platform: String,
    glibc_versions: Vec<String>,
    symbols: Vec<GlibcArchTemplateSymbol>,
}

struct GlibcArchTemplateSymbol {
    library: String,
    symbol: String,
    row_class: String,
    cells: Vec<GlibcArchTemplateSymbolCell>,
}

struct GlibcArchTemplateSymbolCell {
    value: String,
    span: usize,
    deleted: bool,
}

impl GlibcArchTemplateSymbolCell {
    pub fn class_and_content(&self) -> (&'static str, String) {
        match (self.deleted, self.value.as_str()) {
            (true, _) => ("deleted", "x".to_string()),
            (false, "") => ("empty", "".to_string()),
            (false, v) => ("", v.to_string()),
        }
    }
}

pub fn write_report(repo: &Repo, root_dir: &Path) -> Result<()> {
    let lists = repo.library_versioned_abilists()?;

    let mut platforms = vec![];

    for dir in lists.all_directories() {
        let dir_normal = dir.display().to_string().replace("/", "-");

        let dir_normal = dir_normal
            .strip_prefix("sysdeps-")
            .unwrap_or(&dir_normal)
            .replace("mach-hurd", "hurd")
            .replace("unix-sysv-linux", "linux");

        platforms.push(dir_normal.clone());

        let mut target_lists = lists.clone();
        target_lists.filter_parent(&dir);

        let dest_file = root_dir.join(format!("{}.html", dir_normal));
        println!("writing {}", dest_file.display());

        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut fh = std::io::BufWriter::new(std::fs::File::create(&dest_file)?);
        write_versioned_report(dir_normal.to_string(), &target_lists, &mut fh)?;
    }

    write_index(repo, root_dir, &platforms)?;

    Ok(())
}

pub fn write_versioned_report(
    name: String,
    versioned_abi_lists: &VersionedAbiLists,
    mut w: impl std::io::Write,
) -> Result<()> {
    versioned_abi_lists
        .keys()
        .next()
        .ok_or(anyhow!("no glibc version"))?;

    let mut symbols_by_name = BTreeMap::new();
    let mut symbols_by_lib = BTreeSet::new();
    let mut glibc_versions = BTreeSet::new();

    for (version, lists) in versioned_abi_lists.iter() {
        let mut any_symbol = false;

        for (path, list) in lists.iter() {
            let lib = path
                .file_stem()
                .ok_or(anyhow!("no library name"))?
                .to_str()
                .ok_or(anyhow!("path should be valid"))?;

            for symbol in list.symbols.iter() {
                if symbol.version.starts_with("GLIBC_")
                    && symbol.symbol_type != SymbolType::Absolute
                {
                    symbols_by_name
                        .entry(symbol.name.as_str())
                        .or_insert(vec![])
                        .push((version, lib, symbol));

                    symbols_by_lib.insert((lib, symbol.name.as_str()));
                    any_symbol = true;
                }
            }
        }

        if any_symbol {
            glibc_versions.insert(version);
        }
    }

    let mut symbols = vec![];

    // We iterate over symbols by library so libraries are grouped together.
    for (library, symbol) in symbols_by_lib {
        let mut cells = vec![];
        let mut seen_versions = BTreeSet::new();
        let mut have_moves = false;
        let mut have_deletions = false;

        let refs = symbols_by_name.get(symbol).expect("key should exist");

        // All references to this symbol name in other libraries. All versions.
        let other_lib_refs = refs
            .iter()
            .filter(|(_, lib, _)| *lib != library)
            .collect::<Vec<_>>();

        let mut previous_column_version = None;
        let mut previous_version_refs = BTreeSet::new();

        for column_version in glibc_versions.iter() {
            // All of the references to this symbol name in this library in this glibc version.
            let our_refs = refs
                .iter()
                .filter(|(v, lib, _)| *v == *column_version && *lib == library)
                .collect::<Vec<_>>();

            let our_version_refs = our_refs
                .iter()
                .filter_map(|(_, _, s)| s.glibc_version)
                .collect::<BTreeSet<_>>();

            // Now for each symbol version, construct the value to print.
            let mut parts = vec![];

            for symbol_version in our_version_refs.iter() {
                // Annotate if this version can be matched to a different library in the
                // previous column's version.
                let mut extra = "".to_string();
                if let Some(previous_version) = previous_column_version {
                    let previous_libs = other_lib_refs
                        .iter()
                        .filter_map(|(v, lib, s)| {
                            if *v == previous_version
                                && s.glibc_version
                                    .as_ref()
                                    .is_some_and(|x| *x == *symbol_version)
                            {
                                Some(*lib)
                            } else {
                                None
                            }
                        })
                        .collect::<BTreeSet<_>>();

                    if !previous_libs.is_empty() {
                        have_moves = true;
                        extra = format!(
                            " (moved from {})",
                            previous_libs.into_iter().collect::<Vec<_>>().join(", ")
                        )
                    };
                }

                parts.push(format!("{}{}", symbol_version.major_minor_patch(), extra));
            }

            let value = parts.join(", ");

            seen_versions.insert(our_version_refs.clone());

            match cells.last_mut() {
                // This is the first glibc version. Create a new entry.
                None => cells.push(GlibcArchTemplateSymbolCell {
                    value,
                    span: 0,
                    deleted: false,
                }),

                // Symbol was removed in this version. Create a placeholder.
                Some(last) if !last.value.is_empty() && our_version_refs.is_empty() => {
                    have_deletions = true;

                    cells.push(GlibcArchTemplateSymbolCell {
                        value: "".to_string(),
                        span: 0,
                        deleted: true,
                    });
                }
                // Always insert a fresh cell following a deletion.
                Some(last) if last.deleted => {
                    cells.push(GlibcArchTemplateSymbolCell {
                        value,
                        span: 0,
                        deleted: false,
                    });
                }
                // Same value as last time. Extend the cell to this column.
                Some(last) if our_version_refs == previous_version_refs => {
                    last.span += 1;
                }
                _ => {
                    cells.push(GlibcArchTemplateSymbolCell {
                        value,
                        span: 0,
                        deleted: false,
                    });
                }
            }

            _ = previous_column_version.insert(*column_version);
            previous_version_refs = our_version_refs;
        }

        let mut row_classes = vec![];
        if seen_versions.len() == 1 {
            row_classes.push("omnipresent");
        }
        if have_deletions {
            row_classes.push("deletion");
        }
        if have_moves {
            row_classes.push("moves");
        }

        symbols.push(GlibcArchTemplateSymbol {
            library: library.to_string(),
            symbol: symbol.to_string(),
            row_class: row_classes.join(" "),
            cells,
        })
    }

    let t = GlibcArchTemplate {
        platform: name,
        glibc_versions: glibc_versions.iter().map(|v| v.major_minor()).collect(),
        symbols,
    };

    t.write_into(&mut w)?;

    Ok(())
}

#[derive(Template)]
#[template(path = "index.html", ext = "html")]
struct IndexTemplate {
    targets: Vec<String>,
    versions: Vec<Tag>,
    distros: Vec<DistroVersion>,
}

pub fn write_index(repo: &Repo, root_dir: &Path, platforms: &[String]) -> Result<()> {
    let mut versions = repo.tags()?;
    versions.sort();
    versions.reverse();

    let t = IndexTemplate {
        targets: platforms.to_vec(),
        versions,
        distros: DISTRO_GLIBC_VERSIONS.to_vec(),
    };

    let mut fh = std::fs::File::create(root_dir.join("index.html"))?;
    t.write_into(&mut fh)?;

    Ok(())
}

pub fn write_json_metadata(
    glibc_clone: impl Into<PathBuf>,
    root_dir: impl Into<PathBuf>,
) -> Result<()> {
    let root_dir = root_dir.into();

    let repo = Repo::open(glibc_clone)?;

    let lists = repo.library_versioned_abilists()?;

    for (tag, abilists) in lists.iter().rev() {
        let out_dir = root_dir
            .join("glibc")
            .join(format!("{}.{}", tag.major, tag.minor));

        for (list_path, abilist) in abilists.iter() {
            let mut abilist = abilist.clone();
            abilist.filter_glibc();
            abilist.sort_common();

            let abilist_json = serde_json::to_vec_pretty(&abilist)?;

            let out_file = out_dir.join(list_path.with_extension("json"));
            if let Some(parent) = out_file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out_file, &abilist_json)?;
        }
    }

    Ok(())
}
