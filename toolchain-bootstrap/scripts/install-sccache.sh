#!/bin/bash

set -euo errexit -o pipefail

if [ $(uname -m) = "x86_64" ]; then
  arch=x86_64
  sha256=b0e89ead6899224a4ba2b90e9073bf1ce036d95bab30f3dc33c1e1468bc4ad44
else
  arch=aarch64
  sha256=111ddd28fb108cb3e17edb69ab62cefe1dcc97b02e5006ff9c1330f4f2e78368
fi

secure-download.sh \
  https://github.com/mozilla/sccache/releases/download/v0.12.0/sccache-v0.12.0-${arch}-unknown-linux-musl.tar.gz \
  ${sha256} \
  sccache.tar.gz

tar --strip-components=1 -xvf sccache.tar.gz
chmod +x sccache
mv sccache /usr/local/bin
rm -rf sccache*