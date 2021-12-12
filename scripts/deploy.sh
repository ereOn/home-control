#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly BINARY_NAME="home-control"
readonly TARGET_HOST=$1
readonly TARGET_PATH=/home/pi/${BINARY_NAME}
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/${BINARY_NAME}

cargo build --release --target=${TARGET_ARCH}
rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} ${TARGET_PATH}