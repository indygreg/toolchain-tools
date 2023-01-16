#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

export PATH=/toolchain/bin:$PATH
echo /toolchain/lib > /etc/ld.so.conf.d/toolchain

tar -C /usr/local --strip-components=1 -xf /build/cmake.tar.gz
unzip ninja.zip
mv ninja /usr/local/bin/

pushd llvm-project
git checkout "${LLVM_COMMIT}"
popd

mkdir build
pushd build
cmake \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_C_COMPILER_LAUNCHER=sccache \
  -DCMAKE_CXX_COMPILER_LAUNCHER=sccache \
  -DCMAKE_EXE_LINKER_FLAGS="-static-libstdc++" \
  -DCMAKE_INSTALL_PREFIX=/toolchain \
  -DLLVM_TARGETS_TO_BUILD="X86;AArch64" \
  -DLLVM_ENABLE_ASSERTIONS=ON \
  -DLLVM_ENABLE_PROJECTS="bolt" \
  -G Ninja \
  ../llvm-project/llvm

LD_LIBRARY_PATH=/toolchain/lib ninja -j ${PARALLEL_NINJA} -l ${NINJA_MAX_LOAD} bolt

mkdir -p /build/out/toolchain/{bin,lib}
cp -av bin/{llvm-bolt,llvm-boltdiff,merge-fdata,perf2bolt} /build/out/toolchain/bin/
cp -av lib/libbolt_rt*.a /build/out/toolchain/lib/

popd

# Free up a few gigabytes.
rm -rf llvm.git build

sccache -s
