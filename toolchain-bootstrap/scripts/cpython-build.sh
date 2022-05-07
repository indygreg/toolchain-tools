#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

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
