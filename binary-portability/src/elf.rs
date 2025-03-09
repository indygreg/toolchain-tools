// Copyright 2022 Gregory Szorc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    crate::{ObjectFileInfo, Result, UndefinedSymbol},
    object::{
        elf::{DT_NEEDED, SHT_DYNSYM},
        read::elf::{Dyn, FileHeader, SectionHeader, Sym},
        Endianness, SectionIndex,
    },
};

pub fn analyze_elf<Elf: FileHeader<Endian = Endianness>>(data: &[u8]) -> Result<ObjectFileInfo> {
    let in_elf = Elf::parse(data)?;
    let endian = in_elf.endian()?;

    let sections = in_elf.sections(endian, data)?;

    let symbol_versions = sections.versions(endian, data)?;

    let mut dt_needed = vec![];
    let mut undefined = vec![];

    for (section_index, section) in sections.iter().enumerate() {
        if let Some((entries, strings_index)) = section.dynamic(endian, data)? {
            let strings = sections.strings(endian, data, strings_index)?;

            for entry in entries {
                // DT_NEEDED defines external libraries we require.
                if entry.tag32(endian) == Some(DT_NEEDED) {
                    let value = entry.string(endian, strings)?;
                    let value = String::from_utf8_lossy(value).to_string();

                    dt_needed.push(value);
                }
            }
        }

        if let Some(symbols) =
            section.symbols(endian, data, &sections, SectionIndex(section_index))?
        {
            let strings = symbols.strings();

            for (symbol_index, symbol) in symbols.iter().enumerate() {
                let name = symbol.name(endian, strings)?;
                let name = String::from_utf8_lossy(name).to_string();

                // If we're in the .dynsym section, there should be version info for
                // every symbol.
                let symbol_version = if section.sh_type(endian) == SHT_DYNSYM {
                    if let Some(versions) = &symbol_versions {
                        let version_index =
                            versions.version_index(endian, object::SymbolIndex(symbol_index));

                        if let Some(version) = versions.version(version_index)? {
                            let version = version.name();
                            let version = String::from_utf8_lossy(version).to_string();

                            Some(version)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if symbol.is_undefined(endian) {
                    undefined.push(UndefinedSymbol {
                        name,
                        version: symbol_version,
                        library: None,
                    });
                }
            }
        }
    }

    Ok(ObjectFileInfo {
        required_libraries: dt_needed,
        undefined_symbols: undefined,
    })
}
