#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

cd /build

mkdir openssl
pushd openssl
tar --strip-components=1 -xf ../openssl.tar.gz

/usr/bin/perl ./Configure \
  --prefix=/toolchain \
  linux-x86_64 \
  no-shared \
  no-tests \
  --openssldir=/etc/ssl

make -j ${PARALLEL}
make -j ${PARALLEL} install_sw install_ssldirs DESTDIR=/build/out
popd
