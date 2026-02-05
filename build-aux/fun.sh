#!/usr/bin/env sh

BUILD_DIR="/run/build/clapgrep"
COMMAND="cd $BUILD_DIR && env CARGO_HOME=$BUILD_DIR/cargo $@"

flatpak-builder --run \
  $(dirname $0)/../flatpak \
  $(dirname $0)/de.leopoldluley.Clapgrep.CI.json \
  bash -c "$COMMAND"
