// Copyright 2022 Gregory Szorc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::repo::GlibcVersion;
use anyhow::{Context, Result, anyhow};
use gix::bstr::ByteSlice;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use strum::EnumIter;

/// The type of symbol in an .abilist file entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SymbolType {
    /// Symbol is a function.
    #[serde(rename = "F")]
    Function,
    /// Symbol is data with specified size.
    #[serde(rename = "O")]
    Object(u16),
    #[serde(rename = "A")]
    Absolute,
}

/// A parsed line in a glibc .abilist file. Represents a symbol in a library.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ABIListSymbol {
    pub version: String,
    pub name: String,
    pub symbol_type: SymbolType,
    pub glibc_version: Option<GlibcVersion>,
}

impl ABIListSymbol {
    pub fn parse(s: &str) -> Result<Self> {
        // Invalid syntax. Fixed by glibc commit 5e63c240a22c70d928e5c645f913d59074afd329 (2.29).
        let s = if s == "GLIBC_2.2.6 _end GLIBC_2.2.6 g ? D .bss 00000000" {
            "GLIBC_2.2.6 _end D 0x0"
        } else {
            s
        };

        let mut words = s.split_ascii_whitespace();

        let version = words.next().ok_or(anyhow!("version entry not found"))?;
        let name = words.next().ok_or(anyhow!("symbol name not found"))?;
        let type_raw = words
            .next()
            .ok_or_else(|| anyhow!("symbol type not found").context(s.to_string()))?;

        let symbol_type = match type_raw {
            "F" => SymbolType::Function,
            "D" => {
                let size_raw = words.next().ok_or(anyhow!("object size not found"))?;
                let size_hex = size_raw
                    .strip_prefix("0x")
                    .ok_or(anyhow!("object size not in hex format"))?;

                let size = u16::from_str_radix(size_hex, 16)?;

                SymbolType::Object(size)
            }
            // Removed in glibc commit b289cd9db8286fa6c670104dd5dfcfc68d5d00d6 (2.28).
            "A" => SymbolType::Absolute,
            _ => {
                return Err(anyhow!("symbol type parse error: {}", s));
            }
        };

        // Should be at EOS.
        if words.next().is_some() {
            return Err(anyhow!("unexpected extra syntax: {}", s));
        }

        let glibc_version = if let Some((_, v)) = version.split_once("GLIBC_") {
            let mut parts = v.split(".");

            let major = u8::from_str(parts.next().ok_or(anyhow!("no major version"))?)?;
            let minor = u8::from_str(parts.next().ok_or(anyhow!("no minor version"))?)?;
            let patch = if let Some(patch) = parts.next() {
                Some(u8::from_str(patch)?)
            } else {
                None
            };

            Some(GlibcVersion {
                major,
                minor,
                patch,
            })
        } else {
            None
        };

        Ok(Self {
            version: version.to_string(),
            name: name.to_string(),
            symbol_type,
            glibc_version,
        })
    }
}

/// The "base operating system" of a machine environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseOS {
    MachHurd,
    UnixSysV,
}

impl BaseOS {
    pub fn sysdeps_path(self) -> &'static str {
        match self {
            Self::MachHurd => "sysdeps/mach/hurd",
            Self::UnixSysV => "sysdeps/unix/sysv",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OS {
    Hurd,
    Linux,
}

impl OS {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hurd => "hurd",
            Self::Linux => "linux",
        }
    }
}

/// Represents a supported glibc platform target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ABITarget {
    pub base_os: BaseOS,
    pub os: Option<OS>,
    pub machine: &'static str,
    pub sub_machine: Option<&'static str>,
}

impl ABITarget {
    pub fn sysdeps_path(self) -> String {
        let mut parts = vec![self.base_os.sysdeps_path()];

        if let Some(os) = self.os {
            parts.push(os.as_str());
        }

        parts.push(self.machine);
        if let Some(sub) = self.sub_machine {
            parts.push(sub);
        }

        parts.join("/")
    }
}

/// Target types that have .abilist collections.
#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum ABIListTarget {
    HurdI386,
    HurdX86_64,
    LinuxAarch64,
    LinuxAlpha,
    LinuxArmBigEndian,
    LinuxArmLittleEndian,
    LinuxCSKY,
    LinuxHPPA,
    LinuxI386,
    LinuxLoongArch,
    LinuxM68kColdfire,
    LinuxM68kM680x0,
    LinuxMicroblazeBigEndian,
    LinuxMicroblazeLittleEndian,
    // TODO MIPS. Requires more data modeling.
    // TODO or1k.
    // TODO powerpc
    LinuxRiscV32,
    LinuxRistV64,
    LinuxS39032,
    LinuxS39064,
    LinuxShBigEndian,
    LinuxShLittleEndian,
    LinuxSparc32,
    LinuxSparc64,
    LinuxX86_64,
    LinuxX86_64x32,
}

impl From<ABIListTarget> for ABITarget {
    fn from(val: ABIListTarget) -> Self {
        match val {
            ABIListTarget::HurdI386 => ABITarget {
                base_os: BaseOS::MachHurd,
                os: Some(OS::Hurd),
                machine: "i386",
                sub_machine: None,
            },
            ABIListTarget::HurdX86_64 => ABITarget {
                base_os: BaseOS::MachHurd,
                os: Some(OS::Hurd),
                machine: "i386",
                sub_machine: None,
            },
            ABIListTarget::LinuxAarch64 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "aarch64",
                sub_machine: None,
            },
            ABIListTarget::LinuxAlpha => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "arc",
                sub_machine: None,
            },
            ABIListTarget::LinuxArmBigEndian => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "arm",
                sub_machine: Some("be"),
            },
            ABIListTarget::LinuxArmLittleEndian => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "arm",
                sub_machine: Some("le"),
            },
            ABIListTarget::LinuxCSKY => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "csky",
                sub_machine: None,
            },
            ABIListTarget::LinuxHPPA => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "hppa",
                sub_machine: None,
            },
            ABIListTarget::LinuxI386 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "i386",
                sub_machine: None,
            },
            ABIListTarget::LinuxLoongArch => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "loongarch",
                sub_machine: Some("lp64"),
            },
            ABIListTarget::LinuxM68kColdfire => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "m68k",
                sub_machine: Some("coldfire"),
            },
            ABIListTarget::LinuxM68kM680x0 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "m68k",
                sub_machine: Some("m680x0"),
            },
            ABIListTarget::LinuxMicroblazeBigEndian => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "microblaze",
                sub_machine: Some("be"),
            },
            ABIListTarget::LinuxMicroblazeLittleEndian => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "microblaze",
                sub_machine: Some("le"),
            },
            ABIListTarget::LinuxRiscV32 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "riscv",
                sub_machine: Some("rv32"),
            },
            ABIListTarget::LinuxRistV64 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "riscv",
                sub_machine: Some("rv64"),
            },
            ABIListTarget::LinuxS39032 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "s390",
                sub_machine: Some("s390-32"),
            },
            ABIListTarget::LinuxS39064 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "s390",
                sub_machine: Some("s390-64"),
            },
            ABIListTarget::LinuxShBigEndian => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "sh",
                sub_machine: Some("be"),
            },
            ABIListTarget::LinuxShLittleEndian => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "sh",
                sub_machine: Some("le"),
            },
            ABIListTarget::LinuxSparc32 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "sparc",
                sub_machine: Some("sparc32"),
            },
            ABIListTarget::LinuxSparc64 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "arc",
                sub_machine: Some("sparc64"),
            },
            ABIListTarget::LinuxX86_64 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "x86_64",
                sub_machine: Some("64"),
            },
            ABIListTarget::LinuxX86_64x32 => ABITarget {
                base_os: BaseOS::UnixSysV,
                os: Some(OS::Linux),
                machine: "x86_64",
                sub_machine: Some("x32"),
            },
        }
    }
}

/// AbiList represents a parsed glibc .abilist file.
///
/// The file / struct defines metadata about ELF symbols - usually symbols found
/// in libraries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ABIList {
    pub symbols: Vec<ABIListSymbol>,
}

impl ABIList {
    /// Parses a glibc .abilist file.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut symbols = vec![];

        // Modern format has independent entries on own lines.

        // Legacy format:
        //
        //  GLIBC_2.10
        //   GLIBC_2.10 A
        //   __cxa_at_quick_exit F
        //   ...
        //  GLIBC_2.11
        //   ...
        let mut legacy_format = false;
        let mut current_version = None;

        for (i, line) in data.lines().enumerate() {
            let line_s = String::from_utf8_lossy(line);

            if line_s.starts_with('#') {
                continue;
            }

            if i == 0 && !line_s.contains(' ') {
                legacy_format = true;
            }

            let parse_line = match (legacy_format, line_s.starts_with(' ')) {
                (true, false) => {
                    current_version = Some(line_s.to_string());
                    continue;
                }
                (true, true) => {
                    if let Some(version) = current_version.as_ref() {
                        Cow::from(format!("{}{}", version, line_s))
                    } else {
                        return Err(anyhow!("parse error: intended line missing version"));
                    }
                }
                (false, _) => line_s,
            };

            let symbol = ABIListSymbol::parse(&parse_line)
                .with_context(|| format!("parsing line {}:{}", i, parse_line))?;

            symbols.push(symbol);
        }

        Ok(Self { symbols })
    }

    /// Discard all symbols that don't have glibc symbol versions.
    pub fn filter_glibc(&mut self) {
        self.symbols
            .retain(|symbol| symbol.version.starts_with("GLIBC_"));
    }

    /// Sorts the symbols with a recommended heuristic (name then version).
    pub fn sort_common(&mut self) {
        self.symbols
            .sort_by(|a, b| match (a.glibc_version, b.glibc_version) {
                (Some(a_ver), Some(b_ver)) => {
                    (a.name.as_str(), a_ver).cmp(&(b.name.as_str(), b_ver))
                }
                (_, _) => a.name.cmp(&b.name),
            });
    }

    /// Resolves all glibc symbols in this list.
    pub fn all_glibc_symbols(&self) -> BTreeSet<&str> {
        self.symbols
            .iter()
            .filter_map(|s| {
                if s.version.starts_with("GLIBC_") && s.symbol_type != SymbolType::Absolute {
                    Some(s.name.as_str())
                } else {
                    None
                }
            })
            .collect::<_>()
    }

    /// Resolves all the GLIBC_* symbol version strings seen in this list.
    pub fn all_glibc_versions(&self) -> BTreeSet<GlibcVersion> {
        self.symbols
            .iter()
            .filter_map(|s| s.glibc_version)
            .collect::<_>()
    }
}

pub struct SymbolReference<'list> {
    pub path: &'list Path,
    pub library: &'list str,
    pub symbol: &'list ABIListSymbol,
}

/// Represents a collection of parsed .abilist files.
#[derive(Clone, Debug, Default)]
pub struct ABILists(HashMap<PathBuf, ABIList>);

impl Deref for ABILists {
    type Target = HashMap<PathBuf, ABIList>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ABILists {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ABILists {
    /// Split a collection of ABI lists by the directory they are in.
    pub fn split_by_directory(self, merge_identical: bool) -> BTreeMap<PathBuf, ABILists> {
        let mut res = BTreeMap::<PathBuf, ABILists>::new();
        for (path, list) in self.0.into_iter() {
            let path = if merge_identical {
                // nptl directory was pruned.
                if let Some(parent) = path.parent()
                    && parent.ends_with("nptl")
                    && let Some(grandparent) = parent.parent()
                    && let Some(file_name) = path.file_name()
                {
                    grandparent.join(file_name)
                } else {
                    path
                }
            } else {
                path
            };

            if let Some(parent) = path.parent() {
                res.entry(parent.to_path_buf())
                    .or_default()
                    .insert(path, list);
            }
        }

        res
    }

    /// Removes all entries that don't have the given path parent.
    pub fn filter_parent(&mut self, path: &Path) {
        self.retain(|key, _| {
            if let Some(parent) = key.parent() {
                parent == path
            } else {
                false
            }
        })
    }

    /// Discard all entries that aren't relevant to the specified target.
    pub fn filter_target(&mut self, target: ABITarget) {
        self.filter_parent(Path::new(&target.sysdeps_path()))
    }

    /// Discard all entries that aren't relevant to the specified target enumeration.
    pub fn filter_known_target(&mut self, target: ABIListTarget) {
        self.filter_target(target.into());
    }

    /// Obtain the counts of symbols per library in this collection.
    pub fn library_symbol_counts(&self) -> BTreeMap<String, usize> {
        let mut res = BTreeMap::<String, usize>::new();

        for (path, list) in self.iter() {
            if list.symbols.is_empty() {
                continue;
            }

            let library = path
                .file_stem()
                .expect("should have file stem")
                .to_str()
                .expect("should be valid str");

            let count = res.entry(library.to_string()).or_default();
            *count += list.symbols.len();
        }

        res
    }

    /// Obtain all symbol references from all tracked files.
    pub fn all_entries(&self) -> impl Iterator<Item = SymbolReference<'_>> {
        self.iter().flat_map(move |(path, list)| {
            let library = path
                .file_stem()
                .expect("should have file stem")
                .to_str()
                .expect("should be valid str");

            list.symbols.iter().map(move |s| SymbolReference {
                path,
                library,
                symbol: s,
            })
        })
    }

    pub fn entries_for_library_symbol(
        &self,
        library: &str,
        symbol: &str,
    ) -> impl Iterator<Item = SymbolReference<'_>> {
        self.iter()
            .filter_map(move |(path, list)| {
                let lib = path
                    .file_stem()
                    .expect("should have file stem")
                    .to_str()
                    .expect("should be valid str");

                if lib == library {
                    Some((path, lib, list))
                } else {
                    None
                }
            })
            .flat_map(move |(path, lib, list)| {
                list.symbols.iter().filter_map(move |s| {
                    if s.name == symbol {
                        Some(SymbolReference {
                            path,
                            library: lib,
                            symbol: s,
                        })
                    } else {
                        None
                    }
                })
            })
    }

    pub fn symbol_entries(&self, symbol: &str) -> impl Iterator<Item = SymbolReference<'_>> {
        self.all_entries().filter(move |e| e.symbol.name == symbol)
    }
}

/// Represents parsed ABI lists at different glibc versions.
#[derive(Clone, Debug)]
pub struct VersionedAbiLists(BTreeMap<GlibcVersion, ABILists>);

impl From<BTreeMap<GlibcVersion, ABILists>> for VersionedAbiLists {
    fn from(value: BTreeMap<GlibcVersion, ABILists>) -> Self {
        Self(value)
    }
}

impl Deref for VersionedAbiLists {
    type Target = BTreeMap<GlibcVersion, ABILists>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VersionedAbiLists {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl VersionedAbiLists {
    pub fn split_by_directory(self, merge_identical: bool) -> BTreeMap<PathBuf, VersionedAbiLists> {
        let mut res = BTreeMap::new();

        for (version, lists) in self.0.into_iter() {
            for (dir, list) in lists.split_by_directory(merge_identical) {
                res.entry(dir.clone())
                    .or_insert_with(|| VersionedAbiLists(BTreeMap::new()))
                    .insert(version, list);
            }
        }

        res
    }

    /// Discard all path entries that aren't under the given directory.
    pub fn filter_parent(&mut self, parent: &Path) {
        for (_, lists) in self.iter_mut() {
            lists.filter_parent(parent);
        }
    }

    /// Discard all entries that aren't relevant to the specified target.
    pub fn filter_target(&mut self, target: ABITarget) {
        for (_, lists) in self.0.iter_mut() {
            lists.filter_target(target);
        }
    }

    /// Discard all entries that aren't relevant to the specified target enumeration.
    pub fn filter_known_target(&mut self, target: ABIListTarget) {
        self.filter_target(target.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::Repo;
    use gix::path::env::home_dir;

    #[test]
    fn load_all() -> Result<()> {
        let home = home_dir().expect("failed to get home directory");
        let glibc = home.join("src").join("glibc");

        if !glibc.exists() {
            eprintln!("{} does not exist; skipping test", glibc.display());
            return Ok(());
        }

        let repo = Repo::open(glibc)?;

        repo.tags()?.into_iter().try_for_each(|tag| -> Result<()> {
            // .abilist files introduced in 2.16. Don't waste time on older versions.
            if tag.version.lt(&GlibcVersion {
                major: 2,
                minor: 16,
                patch: None,
            }) {
                return Ok(());
            }

            let abilists = repo.library_abilists_for_commit(tag.commit_id)?;

            for (_, list) in abilists.iter() {
                list.all_glibc_versions();
                list.all_glibc_symbols();
            }

            for (_, lists) in abilists.split_by_directory(false) {
                assert!(lists.all_entries().count() > 0);
            }

            Ok(())
        })?;

        Ok(())
    }
}
