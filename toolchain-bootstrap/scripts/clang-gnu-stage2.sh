#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

export PATH=/toolchain/bin:$PATH
echo /toolchain/lib64 > /etc/ld.so.conf.d/toolchain

tar -C /usr/local --strip-components=1 -xf /build/cmake.tar.gz
unzip ninja.zip
mv ninja /usr/local/bin/

mkdir llvm
pushd llvm
tar --strip-components=1 -xf ../llvm.tar.xz
popd

mkdir stage2
pushd stage2

# We build an LLVM toolchain that is dependent on GNU runtimes.
#
# We don't use lld as the default linker because we don't provide crt1.o and
# other GNU runtime object files in our distribution.

cmake \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER_LAUNCHER=sccache \
  -DCMAKE_CXX_COMPILER_LAUNCHER=sccache \
  -DCMAKE_C_COMPILER=/toolchain/bin/clang \
  -DCMAKE_CXX_COMPILER=/toolchain/bin/clang++ \
  -DCMAKE_ASM_COMPILER=/toolchain/bin/clang \
  -DCMAKE_INSTALL_PREFIX=/toolchain \
  -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
  -DLLVM_BINUTILS_INCDIR=/toolchain/include \
  -DLLVM_ENABLE_PROJECTS="clang;compiler-rt" \
  -DLLVM_ENABLE_ZSTD=OFF \
  -DLLVM_INSTALL_UTILS=ON \
  -DLLVM_LINK_LLVM_DYLIB=ON \
  -G Ninja \
  ../llvm/llvm

LD_LIBRARY_PATH=/toolchain/lib64 DESTDIR=/build/out ninja -j ${PARALLEL_NINJA} -l ${NINJA_MAX_LOAD} install
popd

# Free up a few gigabytes.
rm -rf llvm stage2

sccache -s
