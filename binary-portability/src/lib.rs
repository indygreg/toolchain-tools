// Copyright 2022 Gregory Szorc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Inspect binaries for portability concerns.

pub mod elf;
pub mod linux_standard_base;

use thiserror::Error;

/// Crate's error enumeration.
#[derive(Debug, Error)]
pub enum Error {
    #[error("object file error{0}")]
    Object(#[from] object::Error),
}

/// Crate's result type.
pub type Result<T> = std::result::Result<T, Error>;

/// An object file symbol that is not defined.
pub struct UndefinedSymbol {
    /// The symbol's name.
    pub name: String,

    /// Symbol version.
    pub version: Option<String>,

    /// Library the symbol is in.
    pub library: Option<String>,
}

/// Information from a parsed object file.
pub struct ObjectFileInfo {
    /// Libraries that this object file references.
    pub required_libraries: Vec<String>,

    /// Undefined symbols.
    pub undefined_symbols: Vec<UndefinedSymbol>,
}
