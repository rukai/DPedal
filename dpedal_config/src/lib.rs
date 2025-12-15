#![no_std]

pub mod web_config_protocol;

// Memory layout
pub const RP2040_FLASH_OFFSET: usize = 0x10000000;
pub const RP2040_FLASH_SIZE: usize = 1024 * 1024 * 16; // 16 MiB

pub const FIRMWARE_OFFSET: usize = 0;
pub const FIRMWARE_SIZE: usize = 1024 * 1024 * 15; // 15 MiB
pub const CONFIG_OFFSET: usize = 1024 * 1024 * 15;
pub const CONFIG_SIZE: usize = 1024 * 16; // 10 KiB

use arrayvec::ArrayVec;
use defmt::Format;
use rkyv::{Archive, Deserialize, Serialize};
use usbd_hid::descriptor::KeyboardUsage;

const fn assert_config_fits_in_flash() {
    // TODO: This isnt actually accurate, since the data will be serialized into rkyv format first.
    //       Is there a way to calculate the maximum possible size in rkyv format?
    assert!(core::mem::size_of::<Config>() <= CONFIG_SIZE);
}

const fn assert_config_size_fits_into_writable_flash_blocks() {
    // Flash can only be written in blocks of 4096 bytes.
    assert!(CONFIG_SIZE.is_multiple_of(4096));
}

const _: () = assert_config_fits_in_flash();
const _: () = assert_config_size_fits_into_writable_flash_blocks();

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default)]
#[rkyv(derive(Debug))]
pub struct Config {
    pub version: u32,
    //pub name: String,
    pub color: u32,
    pub profiles: ArrayVec<Profile, 2>,
    pub pin_remappings: ArrayVec<PinRemapping, 6>,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default)]
#[rkyv(derive(Debug))]
pub struct PinRemapping {
    pub input: DpedalInput,
    // TODO: make u8
    pub pin: u32,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default)]
#[rkyv(derive(Debug))]
pub struct Profile {
    pub mappings: ArrayVec<Mapping, 20>,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[rkyv(derive(Debug))]
pub struct Mapping {
    pub input: ArrayVec<DpedalInput, 4>,
    pub output: ArrayVec<ComputerInput, 20>,
}

#[derive(Format, Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone, Copy)]
#[rkyv(derive(Debug))]
pub enum DpedalInput {
    #[default]
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    ButtonLeft,
    ButtonRight,
}

impl DpedalInput {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "DpadUp" => Some(Self::DpadUp),
            "DpadDown" => Some(Self::DpadDown),
            "DpadLeft" => Some(Self::DpadLeft),
            "DpadRight" => Some(Self::DpadRight),
            "ButtonLeft" => Some(Self::ButtonLeft),
            "ButtonRight" => Some(Self::ButtonRight),
            _ => None,
        }
    }
}

#[derive(Format, Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone, Copy)]
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
    KeyboardUpArrow,
    KeyboardDownArrow,
    KeyboardLeftArrow,
    KeyboardRightArrow,
    KeyboardPageUp,
    KeyboardPageDown,
    KeyboardBackspace,
    KeyboardDelete,
    KeyboardTab,
    KeyboardEnter,
}

impl ComputerInput {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "None" => Some(Self::None),
            "MouseScrollUp" => Some(Self::MouseScrollUp),
            "MouseScrollDown" => Some(Self::MouseScrollDown),
            "MouseScrollLeft" => Some(Self::MouseScrollLeft),
            "MouseScrollRight" => Some(Self::MouseScrollRight),
            "KeyboardA" => Some(Self::KeyboardA),
            "KeyboardB" => Some(Self::KeyboardB),
            "KeyboardUpArrow" => Some(Self::KeyboardUpArrow),
            "KeyboardDownArrow" => Some(Self::KeyboardDownArrow),
            "KeyboardLeftArrow" => Some(Self::KeyboardLeftArrow),
            "KeyboardRightArrow" => Some(Self::KeyboardRightArrow),
            "KeyboardPageUp" => Some(Self::KeyboardPageUp),
            "KeyboardPageDown" => Some(Self::KeyboardPageDown),
            "KeyboardBackspace" => Some(Self::KeyboardBackspace),
            "KeyboardDelete" => Some(Self::KeyboardDelete),
            "KeyboardTab" => Some(Self::KeyboardTab),
            "KeyboardEnter" => Some(Self::KeyboardEnter),
            _ => None,
        }
    }
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
            ComputerInput::KeyboardUpArrow => InputSplit::Keyboard(KeyboardUsage::KeyboardUpArrow),
            ComputerInput::KeyboardDownArrow => {
                InputSplit::Keyboard(KeyboardUsage::KeyboardDownArrow)
            }
            ComputerInput::KeyboardLeftArrow => {
                InputSplit::Keyboard(KeyboardUsage::KeyboardLeftArrow)
            }
            ComputerInput::KeyboardRightArrow => {
                InputSplit::Keyboard(KeyboardUsage::KeyboardRightArrow)
            }
            ComputerInput::KeyboardPageUp => InputSplit::Keyboard(KeyboardUsage::KeyboardPageUp),
            ComputerInput::KeyboardPageDown => {
                InputSplit::Keyboard(KeyboardUsage::KeyboardPageDown)
            }
            ComputerInput::KeyboardBackspace => {
                InputSplit::Keyboard(KeyboardUsage::KeyboardBackspace)
            }
            ComputerInput::KeyboardDelete => InputSplit::Keyboard(KeyboardUsage::KeyboardDelete),
            ComputerInput::KeyboardTab => InputSplit::Keyboard(KeyboardUsage::KeyboardTab),
            ComputerInput::KeyboardEnter => InputSplit::Keyboard(KeyboardUsage::KeyboardEnter),
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
