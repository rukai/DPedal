#![no_std]

// Memory layout
pub const RP2040_FLASH_OFFSET: usize = 0x10000000;
pub const RP2040_FLASH_SIZE: usize = 1024 * 1024 * 16; // 16 MiB

pub const FIRMWARE_OFFSET: usize = 0;
pub const FIRMWARE_SIZE: usize = 1024 * 1024 * 15; // 15 MiB
pub const CONFIG_OFFSET: usize = 1024 * 1024 * 15;
pub const CONFIG_SIZE: usize = 256; // 10 KiB

use arrayvec::ArrayVec;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default)]
#[rkyv(derive(Debug))]
pub struct Config {
    pub version: u32,
    //pub name: String,
    pub color: u32,
    pub profiles: ArrayVec<Profile, 2>,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default)]
#[rkyv(derive(Debug))]
pub struct Profile {
    pub dpad_up: ComputerInput,
    pub dpad_down: ComputerInput,
    pub dpad_left: ComputerInput,
    pub dpad_right: ComputerInput,
    pub button_left: ComputerInput,
    pub button_right: ComputerInput,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone, Copy)]
#[rkyv(derive(Debug))]
pub enum ComputerInput {
    #[default]
    None,
    MouseScrollUp,
    MouseScrollDown,
    MouseScrollLeft,
    MouseScrollRight,
    KeyboardPageUp,
    KeyboardPageDown,
}
