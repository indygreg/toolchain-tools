#!/bin/bash

set -euo errexit -o pipefail

if [ $(uname -m) = "x86_64" ]; then
  arch=x86_64
  sha256=1fbb35e135660d04a2d5e42b59c7874d39b3deb17de56330b25b713ec59f849b
else
  arch=aarch64
  sha256=d6a1ce4acd02b937cd61bc675a8be029a60f7bc167594c33d75732bbc0a07400
fi

secure-download.sh \
  https://github.com/mozilla/sccache/releases/download/v0.10.0/sccache-v0.10.0-${arch}-unknown-linux-musl.tar.gz \
  ${sha256} \
  sccache.tar.gz

tar --strip-components=1 -xvf sccache.tar.gz
chmod +x sccache
mv sccache /usr/local/bin
rm -rf sccache*