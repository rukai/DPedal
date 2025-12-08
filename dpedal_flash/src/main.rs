use clap::Parser;
use miette::Result;

pub mod cli;
pub mod config;
pub mod elf;
pub mod flash;

fn main() -> Result<()> {
    let cli = cli::Args::parse();
    let config = config::load(cli.path)?;
    let config_bytes = config::encode_config(&config)?;

    // TODO: use this once -Z bindeps stabilizes
    // let firmware_bytes = elf::elf_to_bin(include_bytes!(env!(
    //     "CARGO_BIN_FILE_DPEDAL_FIRMWARE_dpedal_firmware"
    // )))?;
    let firmware_bytes = elf::elf_to_bin(include_bytes!(env!("FIRMWARE_PATH")))?;

    flash::flash_device(&firmware_bytes, &config_bytes)?;

    println!("Succesfully flashed!");
    Ok(())
}
