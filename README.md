# Toolchain Tools

This multi project repository (monorepo) contains projects related to packaging
system and programming language toolchains.

The canonical repository is https://github.com/indygreg/toolchain-tools. Please
file issues and pull requests there.

# Projects

## `apple-sdk` Rust Crate

A library crate for discovering Apple SDKs and accessing their metadata.

This crate can be used for determining an appropriate Apple SDK to use
for building.

## LLVM Command Option Parser

The `llvm-option-parser`, `llvm-command-tablegen-json`, and
`llvm-command-parser` Rust crates combine to implement pure Rust option
parsing for LLVM commands (like `clang` and `lld`). Using the LLVM tablegen
data defining command options, LLVM program command strings/arguments can
be parsed with nominally identical semantics to how the canonical LLVM
commands would.
