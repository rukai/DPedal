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
use usbd_hid::descriptor::KeyboardUsage;

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
    KeyboardA,
    KeyboardB,
    KeyboardPageUp,
    KeyboardPageDown,
}

impl ComputerInput {
    pub fn split(&self) -> InputSplit {
        match self {
            ComputerInput::None => InputSplit::None,
            ComputerInput::MouseScrollUp => InputSplit::Mouse(MouseInput::Scroll { x: 0, y: 1 }),
            ComputerInput::MouseScrollDown => InputSplit::Mouse(MouseInput::Scroll { x: 0, y: -1 }),
            ComputerInput::MouseScrollLeft => InputSplit::Mouse(MouseInput::Scroll { x: -1, y: 0 }),
            ComputerInput::MouseScrollRight => InputSplit::Mouse(MouseInput::Scroll { x: 1, y: 0 }),
            ComputerInput::KeyboardA => InputSplit::Keyboard(KeyboardUsage::KeyboardAa),
            ComputerInput::KeyboardB => InputSplit::Keyboard(KeyboardUsage::KeyboardBb),
            ComputerInput::KeyboardPageUp => InputSplit::Keyboard(KeyboardUsage::KeyboardPageUp),
            ComputerInput::KeyboardPageDown => {
                InputSplit::Keyboard(KeyboardUsage::KeyboardPageDown)
            }
        }
    }
}

pub enum InputSplit {
    None,
    Keyboard(KeyboardUsage),
    Mouse(MouseInput),
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum MouseInput {
    Scroll { x: i8, y: i8 },
    Move { x: i8, y: i8 },
    Click(MouseClick),
}

#[derive(Clone, Copy)]
pub enum MouseClick {
    Left,
    Middle,
    Right,
}
