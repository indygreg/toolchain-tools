#!/usr/bin/env bash

set -ex

# GCC main library directory is preserved. Clang looks for lib/gcc to denote
# versioned GNU toolchain files.
cp -a gcc-support/lib/gcc llvm/lib/

# Same for c++ include files.
cp -a gcc-support/include/c++ llvm/include/

# Copy GNU runtime libraries into appropriate location.
for lib in libstdc++ libgcc; do
  cp -a gcc-support/lib64/${lib}* llvm/lib/gcc/${GNU_TARGET}/${GCC_VERSION}/
  cp -a gcc-support/lib32/${lib}* llvm/lib/gcc/${GNU_TARGET}/${GCC_VERSION}/32/
done

# Symlink the native libraries where LLVM binaries can find them.
ln -s gcc/${GNU_TARGET}/${GCC_VERSION}/libstdc++.so.6 llvm/lib/libstdc++.so.6
ln -s gcc/${GNU_TARGET}/${GCC_VERSION}/libgcc_s.so.1 llvm/lib/libgcc_s.so.1

# Make sure clang works.
llvm/bin/clang -v

tar \
  --sort=name \
  --owner=root:0 \
  --group=root:0 \
  --mtime="2022-01-01 00:00:00" \
  -cvf - llvm \
  | zstd -18 - -o llvm.tar.zst
