use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the dpedal .kdl config file.
    /// If not specified, loads config from `config.kdl` located in the same directory as the exe/binary.
    #[arg()]
    pub path: Option<PathBuf>,
}
