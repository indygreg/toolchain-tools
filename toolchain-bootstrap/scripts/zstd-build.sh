#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

cd /build

mkdir zstd
pushd zstd
tar --strip-components=1 -xf ../zstd.tar.gz

make -j ${PARALLEL} install PREFIX=/toolchain DESTDIR=/build/out
popd
