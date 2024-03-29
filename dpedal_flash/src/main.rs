use clap::Parser;
use dfu_libusb::DfuLibusb;
use goblin::elf::program_header::PT_LOAD;
use miette::{Context, IntoDiagnostic, Result};

mod cli;
mod config;

fn main() -> miette::Result<()> {
    let elf = include_bytes!(env!("CARGO_BIN_FILE_DPEDAL_FIRMWARE_dpedal_firmware"));

    let cli = cli::Args::parse();

    let mut binary = elf_to_bin(elf)?;
    config::append_config_to_firmware(&cli.path, &mut binary)?;
    flash(&binary)?;

    println!("Succesfully flashed");
    Ok(())
}

pub fn flash(bytes: &[u8]) -> Result<()> {
    let intf = 0;
    let alt = 0;
    let vid = 0x0483;
    let pid = 0xdf11;
    let context = rusb::Context::new().into_diagnostic()?;

    let bar = indicatif::ProgressBar::new(bytes.len() as u64);
    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:27.cyan/blue}] \
                    {bytes}/{total_bytes} ({bytes_per_sec}) ({eta}) {msg:10}",
            )
            .into_diagnostic()?
            .progress_chars("#>-"),
    );

    let mut device = DfuLibusb::open(&context, vid, pid, intf, alt).map_err(|e| match e {
        dfu_libusb::Error::CouldNotOpenDevice => miette::miette!(
            r#"Could not find a DPedal to flash.
Ensure the device is plugged in and in flash mode.
To set the DPedal to flash mode:
  1. Hold down the flash button
  2. Press and release the reset button
  3. Release the flash button"#
        ),
        e => miette::miette!("Failed to open a device to flash: {}", e),
    })?;
    device.with_progress({
        let bar = bar.clone();
        move |count| {
            bar.inc(count as u64);
        }
    });

    device
        .download_from_slice(bytes)
        .into_diagnostic()
        .wrap_err("could not write firmware to the device")?;

    bar.finish();

    Ok(())
}

pub fn elf_to_bin(bytes: &[u8]) -> Result<Vec<u8>> {
    let binary = goblin::elf::Elf::parse(bytes).into_diagnostic()?;

    let mut last_address: u64 = 0;

    let mut data = vec![];
    for (i, ph) in binary
        .program_headers
        .iter()
        .filter(|ph| {
            ph.p_type == PT_LOAD
                && ph.p_filesz > 0
                && ph.p_offset >= binary.header.e_ehsize as u64
                && ph.is_read()
        })
        .enumerate()
    {
        // on subsequent passes, if there's a gap between this section and the
        // previous one, fill it with zeros
        if i != 0 {
            let difference = (ph.p_paddr - last_address) as usize;
            data.resize(data.len() + difference, 0x0);
        }

        data.extend_from_slice(&bytes[ph.p_offset as usize..][..ph.p_filesz as usize]);

        last_address = ph.p_paddr + ph.p_filesz;
    }

    Ok(data)
}
