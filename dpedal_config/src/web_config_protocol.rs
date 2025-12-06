use crate::CONFIG_SIZE;
use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[expect(clippy::large_enum_variant)]
pub enum Request {
    GetConfig,
    SetConfig(ArrayVec<u8, CONFIG_SIZE>),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[expect(clippy::large_enum_variant)]
pub enum Response {
    GetConfig(Result<ArrayVec<u8, CONFIG_SIZE>, ()>),
    SetConfig,
    ProtocolError,
}
