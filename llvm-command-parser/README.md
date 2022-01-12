# llvm-command-parser

`llvm-command-parser` provides command option parsing for LLVM programs
like `clang` and `lld` by leveraging LLVM tablegen JSON data defining
options to those commands.

This crate combines the generic LLVM option parser from the
`llvm-option-parser` crate with JSON data from the `llvm-tablegen-data`
crate to facilitate LLVM program argument parsing.

Various crate features exist to control which LLVM command option
parsers are available. By default, current versions of all LLVM commands
are available. See the `Cargo.toml` for the full feature list.

Note that the command option data cumulatively adds up to multiple
megabytes. So consumers wishing to minimize binary size may wish to
trim unused command data via feature pruning.

The canonical home of this project is
https://github.com/indygreg/toolchain-tools.
