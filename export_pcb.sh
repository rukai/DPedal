#!/bin/sh

cd "$(dirname "$0")"

set -e

OUT=target/kicad/dpedal_gerber_files
ZIP=dpedal_gerber_files.zip
PCB=pcb/dpedal_pcb.kicad_pcb

rm -f $ZIP

mkdir -p $OUT
kicad-cli pcb export drill --output $OUT $PCB
kicad-cli pcb export gerbers --output $OUT $PCB
cd target/kicad
zip $ZIP dpedal_gerber_files/*
mv $ZIP ../../pcb
cd -

kicad-cli pcb export step --subst-models --output dpedal_pcb.step $PCB

