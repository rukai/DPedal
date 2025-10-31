use goblin::elf::program_header::PT_LOAD;
use miette::{IntoDiagnostic, Result};

pub fn elf_to_bin(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut binary = goblin::elf::Elf::parse(bytes).into_diagnostic()?;
    binary.program_headers.sort_by_key(|x| x.p_paddr);

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
