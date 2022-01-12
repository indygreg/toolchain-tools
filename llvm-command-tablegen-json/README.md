# llvm-command-tablegen-json

`llvm-command-tablegen-json` is a library crate defining static data
structures containing JSON data that defines the command options to LLVM
programs. The JSON data is derived from running `llvm-tblgen --dump-json`.

This crate is likely utilized alongside the `llvm-option-parser` crate
to load the JSON into an LLVM option parser so command arguments for LLVM
programs (like `clang` and `lld`) can be easily parsed. The
`llvm-command-parser` crate implements this functionality.

Various crate features exist to control which JSON files are available in
the crate. By default, all JSON files are made available. See the
`Cargo.toml` for the full feature list.

Note that the JSON data cumulatively adds up to multiple megabytes. So
callers wishing to minimize binary size may wish to trim unused JSON
blobs via feature pruning.

The canonical home of this project is
https://github.com/indygreg/toolchain-tools.
