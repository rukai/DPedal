#!/bin/sh

set -e

cargo install elf2uf2-rs@2.2.0 --no-default-features

cd dpedal_firmware
cargo build --release
cd ..

elf2uf2-rs dpedal_firmware/target/thumbv6m-none-eabi/release/dpedal_firmware dpedal_firmware.uf2