#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

export PATH=/toolchain/bin:$PATH
echo /toolchain/lib64 > /etc/ld.so.conf.d/toolchain

cd /build

mkdir zstd
pushd zstd
tar --strip-components=1 -xf ../zstd.tar.gz

make -j ${PARALLEL} install PREFIX=/toolchain DESTDIR=/build/out
popd
