#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

readonly BINARY_NAME="home-control"
readonly TARGET_HOST=${1:-${DEPLOY_TARGET_HOST:-}}
readonly TARGET_PATH=/home/pi/.local/bin/${BINARY_NAME}
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/${BINARY_NAME}

if [[ -z "${TARGET_HOST}" ]]; then
  echo "No target host specified."
  echo
  echo "You may specify one with the first argument or set the 'DEPLOY_TARGET_HOST' environment variable."
  exit 1
fi

cargo build --release --target=${TARGET_ARCH} --features gpio
rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} ${TARGET_PATH}