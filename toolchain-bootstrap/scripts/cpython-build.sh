#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

if [ -e /build/secrets ]; then
  source /build/secrets
fi

export PATH=/toolchain/bin:$PATH
echo /toolchain/lib64 > /etc/ld.so.conf.d/toolchain

cd /build

mkdir python
pushd python
tar --strip-components=1 -xf ../cpython.tar.xz

CC="sccache gcc" CXX="sccache g++" ./configure \
  --prefix /toolchain \
  --without-ensurepip

make -j ${PARALLEL}
make -j ${PARALLEL} install DESTDIR=/build/out
popd

sccache -s
