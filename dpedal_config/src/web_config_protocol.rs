use crate::{CONFIG_SIZE, Config};
use arrayvec::ArrayVec;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(derive(Debug))]
pub enum Request {
    GetConfig,
    SetConfig(ArrayVec<u8, CONFIG_SIZE>),
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[rkyv(derive(Debug))]
pub enum Response {
    GetConfig(ArrayVec<u8, CONFIG_SIZE>),
    SetConfig,
}
