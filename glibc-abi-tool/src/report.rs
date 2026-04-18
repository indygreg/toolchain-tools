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
use std::collections::BTreeSet;
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
    row_class: &'static str,
    cells: Vec<GlibcArchTemplateSymbolCell>,
}

struct GlibcArchTemplateSymbolCell {
    value: String,
    span: usize,
    deleted: bool,
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
        target_lists.filter_prefix(&dir);

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
    let mut all_library_symbols = BTreeSet::new();

    for (_, lists) in versioned_abi_lists.iter() {
        for (path, list) in lists.iter() {
            let lib = path
                .file_stem()
                .expect("should have file stem")
                .to_str()
                .expect("should be valid path");

            for symbol in list.symbols.iter() {
                if symbol.version.starts_with("GLIBC_")
                    && symbol.symbol_type != SymbolType::Absolute
                {
                    all_library_symbols.insert((lib, symbol.name.as_str()));
                }
            }
        }
    }

    versioned_abi_lists
        .keys()
        .next()
        .ok_or(anyhow!("no glibc version"))?;

    let mut symbols = vec![];

    // This algorithm isn't efficient. It would be optimal to iterate the lists and
    // build up a mapping of symbol -> version info. We incur a lot of overhead
    // to find symbols data on all versions for every symbol.

    for (lib, symbol) in all_library_symbols {
        let mut cells = vec![];

        let mut seen_versions = BTreeSet::new();

        for version_lists in versioned_abi_lists.values() {
            let versions = version_lists
                .entries_for_library_symbol(lib, symbol)
                .filter_map(|s| s.symbol.glibc_version)
                .collect::<BTreeSet<_>>();

            seen_versions.insert(versions.clone());

            let versions_s = versions
                .iter()
                .map(|v| v.major_minor_patch())
                .collect::<Vec<_>>()
                .join(", ");

            match cells.last_mut() {
                // This is the first glibc version. Create a new entry.
                None => {
                    cells.push(GlibcArchTemplateSymbolCell {
                        value: versions_s,
                        span: 0,
                        deleted: false,
                    });
                }
                // Symbol was removed in this version. Create a placeholder.
                Some(last) if !last.value.is_empty() && versions.is_empty() => {
                    cells.push(GlibcArchTemplateSymbolCell {
                        value: "".to_string(),
                        span: 0,
                        deleted: true,
                    });
                }
                // Always insert a fresh cell following a deletion.
                Some(last) if last.deleted => {
                    cells.push(GlibcArchTemplateSymbolCell {
                        value: versions_s,
                        span: 0,
                        deleted: false,
                    });
                }
                // Same value as last time. Extend the cell to this column.
                Some(last) if last.value == versions_s => {
                    last.span += 1;
                }
                _ => {
                    cells.push(GlibcArchTemplateSymbolCell {
                        value: versions_s,
                        span: 0,
                        deleted: false,
                    });
                }
            }
        }

        symbols.push(GlibcArchTemplateSymbol {
            library: lib.to_string(),
            symbol: symbol.to_string(),
            row_class: if seen_versions.len() == 1 {
                "omnipresent"
            } else {
                ""
            },
            cells,
        })
    }

    let t = GlibcArchTemplate {
        platform: name,
        glibc_versions: versioned_abi_lists
            .keys()
            .map(|v| v.major_minor())
            .collect(),
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
