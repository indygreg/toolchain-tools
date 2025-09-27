#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

if [ -e /build/secrets ]; then
  source /build/secrets
fi

export PATH=/toolchain/bin:$PATH

cd /build

mkdir binutils
pushd binutils
tar --strip-components=1 -xf ../binutils.tar.xz
popd

mkdir binutils-objdir
pushd binutils-objdir

if [ "$(uname -m)" = "x86_64" ]; then
  triple="x86_64-unknown-linux-gnu"
else
  triple="aarch64-unknown-linux-gnu"
fi

# gprofng requires a bison newer than what we have. So just disable it.
STAGE_CC_WRAPPER=sccache \
CC="sccache ${BUILD_CC}" \
CXX="sccache ${BUILD_CXX}" \
LDFLAGS="-static-libgcc -static-libstdc++" \
    ../binutils/configure \
    --build=${triple} \
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
