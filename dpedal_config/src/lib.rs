#![no_std]

pub mod web_config_protocol;

// Memory layout
pub const RP2040_FLASH_OFFSET: usize = 0x10000000;
pub const RP2040_FLASH_SIZE: usize = 1024 * 1024 * 16; // 16 MiB

pub const FIRMWARE_OFFSET: usize = 0;
pub const FIRMWARE_SIZE: usize = 1024 * 1024 * 15; // 15 MiB
pub const CONFIG_OFFSET: usize = 1024 * 1024 * 15;
pub const CONFIG_SIZE: usize = 1024 * 16; // 10 KiB

use arrayvec::{ArrayString, ArrayVec};
use defmt::Format;
use rkyv::{Archive, Deserialize, Serialize};
use strum::{EnumIter, EnumString};
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

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(derive(Debug))]
pub struct Config {
    pub version: u32,
    // TODO: get arrayvec::ArrayString working with rkyv
    pub nickname: ArrayString<50>,
    pub device: Device,
    pub color: u32,
    pub profiles: ArrayVec<Profile, 2>,
    pub pin_remappings: ArrayVec<PinRemapping, 6>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: Default::default(),
            nickname: ArrayString::from("my DPedal").unwrap(),
            device: Default::default(),
            color: 0x1790e3,
            profiles: ArrayVec::from_iter([Profile {
                mappings: ArrayVec::from_iter([
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::DpadLeft]),
                        output: ArrayVec::from_iter([ComputerInput::Mouse(
                            MouseInput::ScrollLeft(10),
                        )]),
                    },
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::DpadRight]),
                        output: ArrayVec::from_iter([ComputerInput::Mouse(
                            MouseInput::ScrollRight(10),
                        )]),
                    },
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::DpadUp]),
                        output: ArrayVec::from_iter([ComputerInput::Mouse(MouseInput::ScrollUp(
                            10,
                        ))]),
                    },
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::DpadDown]),
                        output: ArrayVec::from_iter([ComputerInput::Mouse(
                            MouseInput::ScrollDown(10),
                        )]),
                    },
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::ButtonLeft]),
                        output: ArrayVec::from_iter([ComputerInput::Keyboard(
                            KeyboardInput::PageUp,
                        )]),
                    },
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::ButtonRight]),
                        output: ArrayVec::from_iter([ComputerInput::Keyboard(
                            KeyboardInput::PageDown,
                        )]),
                    },
                ]),
            }]),
            pin_remappings: Default::default(),
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[rkyv(derive(Debug))]
pub enum Device {
    #[default]
    Dpedal,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[rkyv(derive(Debug))]
pub struct PinRemapping {
    pub input: DpedalInput,
    // TODO: make u8
    pub pin: u32,
}

pub const MAX_MAPPINGS: usize = 20;
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
#[rkyv(derive(Debug))]
pub struct Profile {
    pub mappings: ArrayVec<Mapping, MAX_MAPPINGS>,
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

    pub fn from_string_kebab(s: &str) -> Option<Self> {
        match s {
            "dpad-up" => Some(Self::DpadUp),
            "dpad-down" => Some(Self::DpadDown),
            "dpad-left" => Some(Self::DpadLeft),
            "dpad-right" => Some(Self::DpadRight),
            "button-left" => Some(Self::ButtonLeft),
            "button-right" => Some(Self::ButtonRight),
            _ => None,
        }
    }
}

#[derive(Format, Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone, Copy)]
#[rkyv(derive(Debug))]
pub enum ComputerInput {
    #[default]
    None, // TODO: remove?
    Mouse(MouseInput),
    Keyboard(KeyboardInput),
    Control(DPedalControl),
}

#[derive(
    Format, Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone, Copy, EnumIter,
)]
#[rkyv(derive(Debug))]
pub enum MouseInput {
    ScrollUp(i16),
    ScrollDown(i16),
    ScrollRight(i16),
    ScrollLeft(i16),
    MoveUp(i16),
    MoveDown(i16),
    MoveRight(i16),
    MoveLeft(i16),
    #[default]
    ClickLeft,
    ClickMiddle,
    ClickRight,
}

impl MouseInput {
    pub fn from_string(s: &str, value: &str) -> Option<Self> {
        match s {
            "ScrollUp" | "scroll-up" => Some(MouseInput::ScrollUp(value.parse().ok()?)),
            "ScrollDown" | "scroll-down" => Some(MouseInput::ScrollDown(value.parse().ok()?)),
            "ScrollRight" | "scroll-right" => Some(MouseInput::ScrollRight(value.parse().ok()?)),
            "ScrollLeft" | "scroll-left" => Some(MouseInput::ScrollLeft(value.parse().ok()?)),
            "MoveUp" | "move-up" => Some(MouseInput::MoveUp(value.parse().ok()?)),
            "MoveDown" | "move-down" => Some(MouseInput::MoveDown(value.parse().ok()?)),
            "MoveRight" | "move-right" => Some(MouseInput::MoveRight(value.parse().ok()?)),
            "MoveLeft" | "move-left" => Some(MouseInput::MoveLeft(value.parse().ok()?)),
            "ClickLeft" | "click-left" => Some(MouseInput::ClickLeft),
            "ClickMiddle" | "click-middle " => Some(MouseInput::ClickMiddle),
            "ClickRight" | "click-right" => Some(MouseInput::ClickRight),
            _ => None,
        }
    }
}

#[derive(
    Format,
    Archive,
    Deserialize,
    Serialize,
    Debug,
    PartialEq,
    Default,
    Clone,
    Copy,
    EnumIter,
    EnumString,
)]
#[rkyv(derive(Debug))]
pub enum KeyboardInput {
    #[default]
    A,
    B,
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,
    PageUp,
    PageDown,
    Backspace,
    Delete,
    Tab,
    Enter,
}

impl KeyboardInput {
    pub fn usage(&self) -> KeyboardUsage {
        match self {
            KeyboardInput::A => KeyboardUsage::KeyboardAa,
            KeyboardInput::B => KeyboardUsage::KeyboardBb,
            KeyboardInput::UpArrow => KeyboardUsage::KeyboardUpArrow,
            KeyboardInput::DownArrow => KeyboardUsage::KeyboardDownArrow,
            KeyboardInput::LeftArrow => KeyboardUsage::KeyboardLeftArrow,
            KeyboardInput::RightArrow => KeyboardUsage::KeyboardRightArrow,
            KeyboardInput::PageUp => KeyboardUsage::KeyboardPageUp,
            KeyboardInput::PageDown => KeyboardUsage::KeyboardPageDown,
            KeyboardInput::Backspace => KeyboardUsage::KeyboardBackspace,
            KeyboardInput::Delete => KeyboardUsage::KeyboardDelete,
            KeyboardInput::Tab => KeyboardUsage::KeyboardTab,
            KeyboardInput::Enter => KeyboardUsage::KeyboardEnter,
        }
    }
}

#[derive(
    Format, Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone, Copy, EnumIter,
)]
#[rkyv(derive(Debug))]
pub enum DPedalControl {
    #[default]
    DoNothing,
    // ReleaseAndSleep(u16)
    // HoldAndSleep(u16)
    // SetProfile(u8)
}

impl DPedalControl {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "DoNothing" => Some(DPedalControl::DoNothing),
            _ => None,
        }
    }
}
