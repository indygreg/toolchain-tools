#!/bin/bash

set -o errexit
set -o pipefail

if [ $# != 3 ]; then
  echo "Usage: $0 URL SHA-256 DEST_PATH"
  exit 1
fi

URL=$1
DIGEST=$2
DEST_PATH=$3

mkdir -p $(dirname $DEST_PATH)
curl --location --insecure ${URL} > ${DEST_PATH}.tmp
echo "${DIGEST}  ${DEST_PATH}.tmp" | sha256sum -c
mv ${DEST_PATH}.tmp ${DEST_PATH}
