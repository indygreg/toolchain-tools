// Copyright 2022 Gregory Szorc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Linux Standard Base primitives.

/// A library as defined by the Linux Standard Base specifications.
pub struct LsbLibraryRequirement {
    pub name: &'static str,
    pub so_name: &'static str,
}

/// Core module libraries as defined by the Linux Standard Base Common specification.
///
/// https://refspecs.linuxfoundation.org/LSB_5.0.0/LSB-Common/LSB-Common/requirements.html
pub const CORE_LIBRARIES: &[LsbLibraryRequirement] = &[
    LsbLibraryRequirement {
        name: "libcrypt",
        so_name: "libcrypt.so.1",
    },
    LsbLibraryRequirement {
        name: "libdl",
        so_name: "libdl.so.2",
    },
    LsbLibraryRequirement {
        name: "libgcc",
        so_name: "libgcc_s.so.1",
    },
    LsbLibraryRequirement {
        name: "libncurses",
        so_name: "libncurses.so.5",
    },
    LsbLibraryRequirement {
        name: "libncursesw",
        so_name: "libncursesw.so.5",
    },
    LsbLibraryRequirement {
        name: "libnspr4",
        so_name: "libnspr4.so",
    },
    LsbLibraryRequirement {
        name: "libnss3",
        so_name: "libnss3.so",
    },
    LsbLibraryRequirement {
        name: "libpam",
        so_name: "libpam.so.0",
    },
    LsbLibraryRequirement {
        name: "libpthread",
        so_name: "libpthread.so.0",
    },
    LsbLibraryRequirement {
        name: "librt",
        so_name: "librt.so.1",
    },
    LsbLibraryRequirement {
        name: "libssl3",
        so_name: "libssl3.so",
    },
    LsbLibraryRequirement {
        name: "libstdcxx",
        so_name: "libstdc++.so.6",
    },
    LsbLibraryRequirement {
        name: "libstdcxx",
        so_name: "libstdc++.so.6",
    },
    LsbLibraryRequirement {
        name: "libutil",
        so_name: "libutil.so.1",
    },
    LsbLibraryRequirement {
        name: "libz",
        so_name: "libz.so.1",
    },
];

/// Core libraries for x86-64 platform.
///
/// https://refspecs.linuxfoundation.org/LSB_5.0.0/LSB-Core-AMD64/LSB-Core-AMD64/requirements.html
pub const CORE_LIBRARIES_X86_64: &[LsbLibraryRequirement] = &[
    LsbLibraryRequirement {
        name: "libc",
        so_name: "libc.so.6",
    },
    LsbLibraryRequirement {
        name: "libcrypt",
        so_name: "libcrypt.so.1",
    },
    LsbLibraryRequirement {
        name: "libdl",
        so_name: "libdl.so.2",
    },
    LsbLibraryRequirement {
        name: "libgcc_s",
        so_name: "libgcc_s.so.1",
    },
    LsbLibraryRequirement {
        name: "libm",
        so_name: "libm.so.6",
    },
    LsbLibraryRequirement {
        name: "libncurses",
        so_name: "libncurses.so.5",
    },
    LsbLibraryRequirement {
        name: "libncursesw",
        so_name: "libncursesw.so.5",
    },
    LsbLibraryRequirement {
        name: "libpthread",
        so_name: "libpthread.so.0",
    },
    LsbLibraryRequirement {
        name: "libstdcxx",
        so_name: "libstdc++.so.6",
    },
    LsbLibraryRequirement {
        name: "libutil",
        so_name: "libutil.so.1",
    },
    LsbLibraryRequirement {
        name: "libz",
        so_name: "libz.so.1",
    },
];

/// Program interpreter path for x86-64 platform.
pub const PROGRAM_INTERPRETER_X86_64: &str = "/lib64/ld-lsb-x86-64.so.3";
