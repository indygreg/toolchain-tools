// Copyright 2022 Gregory Szorc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use {once_cell::sync::Lazy, std::collections::BTreeMap};

#[cfg(feature = "13-clang")]
pub const CLANG_13: &[u8] = include_bytes!("llvm-13/clang.json");
#[cfg(feature = "13-dsymutil")]
pub const DSYMUTIL_13: &[u8] = include_bytes!("llvm-13/dsymutil.json");
#[cfg(feature = "13-lld")]
pub const LLD_COFF_13: &[u8] = include_bytes!("llvm-13/lld-coff.json");
#[cfg(feature = "13-lld")]
pub const LLD_DARWIN_LD_13: &[u8] = include_bytes!("llvm-13/lld-darwin-ld.json");
#[cfg(feature = "13-lld")]
pub const LLD_ELF_13: &[u8] = include_bytes!("llvm-13/lld-elf.json");
#[cfg(feature = "13-lld")]
pub const LLD_MACHO_13: &[u8] = include_bytes!("llvm-13/lld-macho.json");
#[cfg(feature = "13-lld")]
pub const LLD_MINGW_13: &[u8] = include_bytes!("llvm-13/lld-mingw.json");
#[cfg(feature = "13-lld")]
pub const LLD_WASM_13: &[u8] = include_bytes!("llvm-13/lld-wasm.json");
#[cfg(feature = "13-cvtres")]
pub const LLVM_CVTRES_13: &[u8] = include_bytes!("llvm-13/llvm-cvtres.json");
#[cfg(feature = "13-cxxfilt")]
pub const LLVM_CXXFILT_13: &[u8] = include_bytes!("llvm-13/llvm-cxxfilt.json");
#[cfg(feature = "13-dlltool")]
pub const LLVM_DLLTOOL_13: &[u8] = include_bytes!("llvm-13/llvm-dlltool.json");
#[cfg(feature = "13-lib")]
pub const LLVM_LIB_13: &[u8] = include_bytes!("llvm-13/llvm-lib.json");
#[cfg(feature = "13-ml")]
pub const LLVM_ML_13: &[u8] = include_bytes!("llvm-13/llvm-ml.json");
#[cfg(feature = "13-mt")]
pub const LLVM_MT_13: &[u8] = include_bytes!("llvm-13/llvm-mt.json");
#[cfg(feature = "13-nm")]
pub const LLVM_NM_13: &[u8] = include_bytes!("llvm-13/llvm-nm.json");
#[cfg(feature = "13-rc")]
pub const LLVM_RC_13: &[u8] = include_bytes!("llvm-13/llvm-rc.json");
#[cfg(feature = "13-readobj")]
pub const LLVM_READOBJ_13: &[u8] = include_bytes!("llvm-13/llvm-readobj.json");
#[cfg(feature = "13-size")]
pub const LLVM_SIZE_13: &[u8] = include_bytes!("llvm-13/llvm-size.json");
#[cfg(feature = "13-strings")]
pub const LLVM_STRINGS_13: &[u8] = include_bytes!("llvm-13/llvm-strings.json");
#[cfg(feature = "13-symbolizer")]
pub const LLVM_SYMBOLIZER_13: &[u8] = include_bytes!("llvm-13/llvm-symbolizer.json");

/// Raw tablegen JSON for commands in LLVM version 13.
pub static LLVM_13: Lazy<BTreeMap<&str, &[u8]>> = Lazy::new(|| {
    BTreeMap::from_iter([
        #[cfg(feature = "13-clang")]
        ("clang", CLANG_13),
        #[cfg(feature = "13-dsymutil")]
        ("dsymutil", DSYMUTIL_13),
        #[cfg(feature = "13-lld")]
        ("lld-coff", LLD_COFF_13),
        #[cfg(feature = "13-lld")]
        ("lld-darwin-ld", LLD_DARWIN_LD_13),
        #[cfg(feature = "13-lld")]
        ("lld-elf", LLD_ELF_13),
        #[cfg(feature = "13-lld")]
        ("lld-macho", LLD_MACHO_13),
        #[cfg(feature = "13-lld")]
        ("lld-mingw", LLD_MINGW_13),
        #[cfg(feature = "13-lld")]
        ("lld-wasm", LLD_WASM_13),
        #[cfg(feature = "13-cvtres")]
        ("llvm-cvtres", LLVM_CVTRES_13),
        #[cfg(feature = "13-cxxfilt")]
        ("llvm-cxxfilt", LLVM_CXXFILT_13),
        #[cfg(feature = "13-dlltool")]
        ("llvm-dlltool", LLVM_DLLTOOL_13),
        #[cfg(feature = "13-lib")]
        ("llvm-lib", LLVM_LIB_13),
        #[cfg(feature = "13-ml")]
        ("llvm-ml", LLVM_ML_13),
        #[cfg(feature = "13-mt")]
        ("llvm-mt", LLVM_MT_13),
        #[cfg(feature = "13-nm")]
        ("llvm-nm", LLVM_NM_13),
        #[cfg(feature = "13-cxxfilt")]
        ("llvm-cxxfilt", LLVM_CXXFILT_13),
        #[cfg(feature = "13-rc")]
        ("llvm-rc", LLVM_RC_13),
        #[cfg(feature = "13-readobj")]
        ("llvm-readobj", LLVM_READOBJ_13),
        #[cfg(feature = "13-size")]
        ("llvm-size", LLVM_SIZE_13),
        #[cfg(feature = "13-strings")]
        ("llvm-strings", LLVM_STRINGS_13),
        #[cfg(feature = "13-symbolizer")]
        ("llvm-symbolizer", LLVM_SYMBOLIZER_13),
    ])
});
