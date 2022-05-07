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

# Build Clang with GCC.
#
# The built clang has a run-time dependency on libstdc++ and libgcc.
#
# Explanations for arguments:
#
# CMAKE_CXX_FLAGS
#   Suppress warnings to cut down on output spam.
#
# LLVM_LINK_LLVM_DYLIB=ON
#   Forces binaries to link against shared libraries. This greatly reduces
#   size of toolchain at expense of runtime speed.
#
# *_LINKER_FLAGS
#   Runtime loader search path where libstdc++ and friends are located.
#   Without this, the build environment's old libstdc++ is used and things
#   fail due to missing symbols.

mkdir stage1
pushd stage1

cmake \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER_LAUNCHER=sccache \
  -DCMAKE_CXX_COMPILER_LAUNCHER=sccache \
  -DCMAKE_CXX_FLAGS="-Wno-address -Wno-missing-template-keyword -Wno-uninitialized" \
  -DCMAKE_EXE_LINKER_FLAGS="-Wl,-R/toolchain/lib64" \
  -DCMAKE_SHARED_LINKER_FLAGS="-Wl,-R/toolchain/lib64" \
  -DCMAKE_INSTALL_PREFIX=/toolchain \
  -DLLVM_BUILD_TOOLS=OFF \
  -DLLVM_ENABLE_PROJECTS="clang" \
  -DLLVM_LINK_LLVM_DYLIB=ON \
  -DLLVM_TARGETS_TO_BUILD=Native \
  -G Ninja \
  ../llvm/llvm

LD_LIBRARY_PATH=/toolchain/lib64 DESTDIR=/build/out ninja -j ${PARALLEL_NINJA} -l ${NINJA_MAX_LOAD} install
popd

# Free up a few gigabytes.
rm -rf llvm stage1

sccache -s