#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

export PATH=/toolchain/bin:$PATH

cd /build

mkdir binutils
pushd binutils
tar --strip-components=1 -xf ../binutils.tar.xz
popd

mkdir binutils-objdir
pushd binutils-objdir

# gprofng requires a bison newer than what we have. So just disable it.

STAGE_CC_WRAPPER=sccache \
CC="sccache ${BUILD_CC}" \
CXX="sccache ${BUILD_CXX}" \
LDFLAGS="-static-libgcc -static-libstdc++" \
    ../binutils/configure \
    --build=x86_64-unknown-linux-gnu \
    --prefix=/toolchain \
    --enable-gold=default \
    --enable-gprofng=no \
    --enable-ld \
    --enable-plugins \
    --with-sysroot=/

make -j32
make install DESTDIR=/build/out
popd

sccache -s
