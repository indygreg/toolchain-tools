#!/usr/bin/env bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -ex

ROOT=$(pwd)

mkdir bin
export PATH=${ROOT}/bin:${PATH}

tar -xf ${ROOT}/sccache-v${SCCACHE_VERSION}-*-apple-darwin.tar.gz
mv sccache-*/sccache bin/
chmod +x bin/sccache

tar --strip-components=1 -xf ${ROOT}/cmake-${CMAKE_VERSION}-macos-universal.tar.gz

mkdir ninja
pushd ninja
unzip ${ROOT}/ninja-mac.zip
mv ninja ${ROOT}/bin
popd

export PATH=${ROOT}/CMake.app/Contents/bin:${PATH}

mkdir llvm
pushd llvm
tar --strip-components=1 -xf ${ROOT}/llvm-project-${LLVM_VERSION}.src.tar.xz
popd

mkdir llvm-objdir
pushd llvm-objdir

# Stage 1: Build with system Clang
mkdir stage1
pushd stage1
cmake \
    -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=/toolchain \
    -DCMAKE_C_COMPILER_LAUNCHER=sccache \
    -DCMAKE_CXX_COMPILER_LAUNCHER=sccache \
    -DCMAKE_C_COMPILER=/usr/bin/clang \
    -DCMAKE_CXX_COMPILER=/usr/bin/clang++ \
    -DCMAKE_ASM_COMPILER=/usr/bin/clang \
    -DLLVM_APPEND_VC_REV=OFF \
    -DLLVM_ENABLE_PROJECTS="bolt;clang" \
    -DLLVM_ENABLE_RUNTIMES="compiler-rt;libcxx;libcxxabi" \
    -DLLVM_ENABLE_LIBCXX=ON \
    -DLLVM_ENABLE_ZSTD=OFF \
    -DLLVM_OPTIMIZED_TABLEGEN=ON \
    -DLLVM_LINK_LLVM_DYLIB=ON \
    -DLLVM_TARGETS_TO_BUILD="AArch64;X86" \
    -DCOMPILER_RT_ENABLE_IOS=OFF \
    -DCOMPILER_RT_ENABLE_WATCHOS=OFF \
    -DCOMPILER_RT_ENABLE_TVOS=OFF \
    -DCOMPILER_RT_BUILD_SANITIZERS=OFF \
    -DCOMPILER_RT_BUILD_LIBFUZZER=OFF \
    -DCOMPILER_RT_BUILD_MEMPROF=OFF \
    -DCOMPILER_RT_BUILD_ORC=OFF \
    -DCOMPILER_RT_BUILD_XRAY=OFF \
    ../../llvm/llvm

if [ -n "${CI}" ]; then
    NUM_JOBS=${NUM_JOBS_AGGRESSIVE}
else
    NUM_JOBS=${NUM_CPUS}
fi

# There appears to be a missing dependency from bolt on VCSRevision.h. This may
# only materialize if building the bolt project before clang. Work around it
# by forcing the missing header to be generated up front.
DESTDIR=${ROOT}/out ninja -j ${NUM_JOBS} include/llvm/Support/llvm_vcsrevision_h

DESTDIR=${ROOT}/out ninja -j ${NUM_JOBS} install

# We should arguably do a 2nd build using Clang to build Clang.

# Move out of objdir
popd

sccache --stop-server
