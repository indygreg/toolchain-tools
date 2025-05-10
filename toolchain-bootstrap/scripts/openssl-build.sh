#!/usr/bin/env bash

set -o errexit
set -o pipefail
set -x

cd /build

mkdir openssl
pushd openssl
tar --strip-components=1 -xf ../openssl.tar.gz

if [ "$(uname -m)" = "x86_64" ]; then
  target="linux-x86_64"
else
  target="linux-aarch64"
fi

/usr/bin/perl ./Configure \
  --prefix=/toolchain \
  ${target} \
  no-shared \
  no-tests \
  --openssldir=/etc/ssl

make -j ${PARALLEL}
make -j ${PARALLEL} install_sw install_ssldirs DESTDIR=/build/out
popd
