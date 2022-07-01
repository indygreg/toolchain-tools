# text-stub-library

`text-stub-library` is a library crate for reading and writing
*text stub files*, which define metadata about dynamic libraries.

*text stub files* are commonly materialized as `.tbd` files and
are commonly seen in Apple SDKs, where they serve as placeholders
for `.dylib` files, enabling linkers to work without access to
the full `.dylib` file.
