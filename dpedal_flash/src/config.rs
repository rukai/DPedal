use arrayvec::ArrayVec;
use dpedal_config::Config;
use kdl::{KdlDocument, KdlNode};
use kdl_config::{KdlConfig, KdlConfigFinalize, Parsed, error::ParseError};
use kdl_config_derive::{KdlConfig, KdlConfigFinalize};
use miette::{IntoDiagnostic, NamedSource, miette};
use rkyv::rancor::Error;
use std::path::PathBuf;

pub fn encode_config(config: &Config) -> miette::Result<Vec<u8>> {
    let bytes = rkyv::to_bytes::<Error>(config).map_err(|e| miette!(e))?;
    let mut result = vec![];
    result.extend((bytes.len() as u32).to_be_bytes());
    result.extend(bytes.iter());
    Ok(result)
}

pub fn load(path: Option<PathBuf>) -> miette::Result<Config> {
    let input = load_source(path)?;
    // TODO: upstream a way to tell KDL parser what the filename is.
    let kdl: KdlDocument = input.inner().parse()?;
    let (profile, error): (Parsed<ConfigKdl>, ParseError) = kdl_config::parse(input, kdl);

    // TODO: extra diagnostics here.

    if !error.diagnostics.is_empty() {
        return Err(error.into());
    }

    Ok(profile.value.finalize())
}

fn load_source(path: Option<PathBuf>) -> miette::Result<NamedSource<String>> {
    let path = if let Some(path) = path {
        path
    } else if let Ok(cargo_manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(cargo_manifest_dir)
            .parent()
            .unwrap()
            .join("config.kdl")
    } else {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("config.kdl")
    };
    let filename = path.file_name().unwrap().to_str().unwrap();
    let text = std::fs::read_to_string(&path)
        .into_diagnostic()
        .map_err(|e| e.context(format!("Failed to load config file at {path:?}")))?;
    Ok(NamedSource::new(filename, text))
}

#[derive(KdlConfig, KdlConfigFinalize, Default, Debug)]
#[kdl_config_finalize_into = "dpedal_config::Config"]
pub struct ConfigKdl {
    pub version: Parsed<u32>,
    //pub name: Parsed<String>,
    pub color: Parsed<u32>,
    pub profiles: Parsed<ArrayVec<Parsed<ProfileKdl>, 2>>,
    // TODO: add validation: no duplicate pins (including default values), valid pin range
    pub pin_remappings: Parsed<ArrayVec<Parsed<PinRemappingKdl>, 6>>,
}

#[derive(KdlConfig, KdlConfigFinalize, Default, Debug)]
#[kdl_config_finalize_into = "dpedal_config::PinRemapping"]
pub struct PinRemappingKdl {
    pub input: Parsed<DpedalInputKdl>,
    pub pin: Parsed<u32>,
}

// TODO: add derive side validation that Parsed is used everywhere.
#[derive(KdlConfig, KdlConfigFinalize, Default, Debug)]
#[kdl_config_finalize_into = "dpedal_config::Profile"]
pub struct ProfileKdl {
    pub mappings: Parsed<ArrayVec<Parsed<MappingKdl>, 20>>,
}

#[derive(KdlConfig, KdlConfigFinalize, Default, Debug)]
#[kdl_config_finalize_into = "dpedal_config::Mapping"]
pub struct MappingKdl {
    pub input: Parsed<ArrayVec<Parsed<DpedalInputKdl>, 4>>,
    pub output: Parsed<ArrayVec<Parsed<ComputerInputKdl>, 20>>,
}

#[derive(KdlConfig, KdlConfigFinalize, Default, Debug)]
#[kdl_config_finalize_into = "dpedal_config::DpedalInput"]
pub enum DpedalInputKdl {
    #[default]
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    ButtonLeft,
    ButtonRight,
}

#[derive(KdlConfig, KdlConfigFinalize, Default, Debug)]
#[kdl_config_finalize_into = "dpedal_config::ComputerInput"]
pub enum ComputerInputKdl {
    #[default]
    None,
    MouseScrollUp,
    MouseScrollDown,
    MouseScrollLeft,
    MouseScrollRight,
    KeyboardA,
    KeyboardB,
    KeyboardPageUp,
    KeyboardPageDown,
}
