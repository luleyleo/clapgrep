#!/usr/bin/env sh

flatpak-builder --run $(dirname $0)/../flatpak $(dirname $0)/de.leopoldluley.Clapgrep.json "$@"
