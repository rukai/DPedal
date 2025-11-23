#!/bin/sh

set -e

if ! type "kicad-cli" > /dev/null; then
    sudo add-apt-repository --yes ppa:kicad/kicad-9.0-releases
    sudo apt update
    sudo apt install --install-recommends kicad
fi
