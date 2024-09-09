#!/usr/bin/env sh

export CARGO_HOME=/run/build/clapgrep/cargo
flatpak-builder --run $(dirname $0)/../flatpak $(dirname $0)/de.leopoldluley.Clapgrep.json "$@"
