#!/bin/sh

set -e

OUT=target/kicad/dpedal_gerber_files
ZIP=dpedal_gerber_files.zip

rm -f $ZIP

mkdir -p $OUT
kicad-cli pcb export drill --output $OUT pcb/dpedal_pcb.kicad_pcb
kicad-cli pcb export gerbers --output $OUT pcb/dpedal_pcb.kicad_pcb
cd target/kicad
zip $ZIP dpedal_gerber_files/*
mv $ZIP ../..

