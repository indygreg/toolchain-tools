#!/bin/bash

set -euo errexit -o pipefail

if [ $(uname -m) = "x86_64" ]; then
  arch=x86_64
  sha256=8424b38cda4ecce616a1557d81328f3d7c96503a171eab79942fad618b42af44
else
  arch=aarch64
  sha256=62a6c942c47c93333bc0174704800cef7edfa0416d08e1356c1d3e39f0b462f2
fi

secure-download.sh \
  https://github.com/mozilla/sccache/releases/download/v0.14.0/sccache-v0.14.0-${arch}-unknown-linux-musl.tar.gz \
  ${sha256} \
  sccache.tar.gz

tar --strip-components=1 -xvf sccache.tar.gz
chmod +x sccache
mv sccache /usr/local/bin
rm -rf sccache*