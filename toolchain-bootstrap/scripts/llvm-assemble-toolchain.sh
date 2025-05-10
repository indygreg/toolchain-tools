#!/usr/bin/env bash

set -ex

# Remove some cruft that bloats the build.
rm llvm/bin/c-index-test
rm llvm/bin/llvm-exegesis

# GCC main library directory is preserved. Clang looks for lib/gcc to denote
# versioned GNU toolchain files.
cp -a gcc-support/lib/gcc llvm/lib/

# Same for c++ include files.
cp -a gcc-support/include/c++ llvm/include/

# Copy GNU libraries into appropriate location.
cp -a gcc-support/lib64/* llvm/lib/gcc/${GNU_TARGET}/${GCC_VERSION}/
cp -a gcc-support/lib32/* llvm/lib/gcc/${GNU_TARGET}/${GCC_VERSION}/32/

# Symlink the native libraries where LLVM binaries can find them.
ln -s gcc/${GNU_TARGET}/${GCC_VERSION}/libstdc++.so.6 llvm/lib/libstdc++.so.6
ln -s gcc/${GNU_TARGET}/${GCC_VERSION}/libgcc_s.so.1 llvm/lib/libgcc_s.so.1

# Make sure clang works.
llvm/bin/clang -v

mtime=$(date -d @${EARTHLY_SOURCE_DATE_EPOCH} "+%Y-%m-%d 00:00:00")

tar \
  --sort=name \
  --owner=root:0 \
  --group=root:0 \
  --mtime="${mtime}" \
  -cvf - llvm \
  | zstd -18 - -o llvm.tar.zst
