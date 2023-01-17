#!/usr/bin/env bash
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

set -ex

LLVM_COMMIT=db6961db7a0d44da3dd7d0a604f43fc7db8b21b5

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

pushd llvm-project
git checkout "${LLVM_COMMIT}"
rm -rf .git
popd

mkdir build
pushd build

# LLVM_APPEND_VC_REV is disabled because of intermittent (!!!) errors on macOS.
cmake \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER_LAUNCHER=sccache \
  -DCMAKE_CXX_COMPILER_LAUNCHER=sccache \
  -DCMAKE_INSTALL_PREFIX=/toolchain \
  -DLLVM_APPEND_VC_REV=OFF \
  -DLLVM_TARGETS_TO_BUILD="X86;AArch64" \
  -DLLVM_ENABLE_ASSERTIONS=ON \
  -DLLVM_ENABLE_PROJECTS="bolt" \
  -DLLVM_ENABLE_ZSTD=OFF \
  -DLLVM_OPTIMIZED_TABLEGEN=ON \
  -G Ninja \
  ../llvm-project/llvm

if [ -n "${CI}" ]; then
    NUM_JOBS=${NUM_JOBS_AGGRESSIVE}
else
    NUM_JOBS=${NUM_CPUS}
fi

ninja -j ${NUM_JOBS} bolt

mkdir -p ${ROOT}/out/toolchain/{bin,lib}
cp -av bin/{llvm-bolt,llvm-boltdiff,merge-fdata,perf2bolt} ${ROOT}/out/toolchain/bin/

# Runtime libraries aren't available for aarch64.
if [ -e lib/libbolt_rt_instr_osx.a ]; then
  cp -av lib/libbolt_rt*.a ${ROOT}/out/toolchain/lib/
fi

popd

sccache --stop-server
