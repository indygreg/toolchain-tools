# llvm-option-parser

`llvm-option-parser` provides a command option parser that strives to be
functionally equivalent to the option parser in the LLVM project. Its
original goal is to facilitate Rust programs parsing the options of LLVM
programs (like `clang` and `lld`) using the same semantics as the canonical
programs.

The library remodels LLVM's tablegen-based definitions of command options
as Rust types. It can parse the output of `llvm-tblgen --dump-json`, which
enables Rust programs to parse command options defined used LLVM tablegen.

See also the `llvm-command-tablegen-json` crate for a data crate providing
JSON for LLVM programs and the `llvm-command-parser` crate which glues the
JSON data to the option parser to enable parsing of options for LLVM
programs like `clang` and `lld`.

The canonical home of this project is
https://github.com/indygreg/toolchain-tools.
