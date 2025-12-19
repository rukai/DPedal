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
use strum::EnumIter;
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
    //pub name: String,
    pub color: u32,
    pub profiles: ArrayVec<Profile, 2>,
    pub pin_remappings: ArrayVec<PinRemapping, 6>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: Default::default(),
            color: 0x1790e3,
            profiles: ArrayVec::from_iter([Profile {
                mappings: ArrayVec::from_iter([
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::DpadLeft]),
                        output: ArrayVec::from_iter([ComputerInput::Mouse(MouseInput::ScrollLeft)]),
                    },
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::DpadRight]),
                        output: ArrayVec::from_iter([ComputerInput::Mouse(
                            MouseInput::ScrollRight,
                        )]),
                    },
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::DpadUp]),
                        output: ArrayVec::from_iter([ComputerInput::Mouse(MouseInput::ScrollUp)]),
                    },
                    Mapping {
                        input: ArrayVec::from_iter([DpedalInput::DpadDown]),
                        output: ArrayVec::from_iter([ComputerInput::Mouse(MouseInput::ScrollDown)]),
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
pub struct PinRemapping {
    pub input: DpedalInput,
    // TODO: make u8
    pub pin: u32,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone)]
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
    #[default]
    ScrollUp,
    ScrollDown,
    ScrollRight,
    ScrollLeft,
    MoveUp,
    MoveDown,
    MoveRight,
    MoveLeft,
    ClickLeft,
    ClickMiddle,
    ClickRight,
}

impl MouseInput {
    // TODO: return Result<Self, MouseInputError>
    // enum MouseInputError {
    //   /// Entirely incorrect
    //   Invalid,
    //   /// requires more data e.g. `scroll-right 5`
    //   Partial
    // }
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "ScrollUp" => Some(MouseInput::ScrollUp),
            "ScrollDown" => Some(MouseInput::ScrollDown),
            "ScrollRight" => Some(MouseInput::ScrollRight),
            "ScrollLeft" => Some(MouseInput::ScrollLeft),
            "MoveUp" => Some(MouseInput::MoveUp),
            "MoveDown" => Some(MouseInput::MoveDown),
            "MoveRight" => Some(MouseInput::MoveRight),
            "MoveLeft" => Some(MouseInput::MoveLeft),
            "ClickLeft" => Some(MouseInput::ClickLeft),
            "ClickMiddle" => Some(MouseInput::ClickMiddle),
            "ClickRight" => Some(MouseInput::ClickRight),
            _ => None,
        }
    }

    pub fn from_string_kebab(s: &str) -> Option<Self> {
        match s {
            "scroll-up" => Some(MouseInput::ScrollUp),
            "scroll-down" => Some(MouseInput::ScrollDown),
            "scroll-right" => Some(MouseInput::ScrollRight),
            "scroll-left" => Some(MouseInput::ScrollLeft),
            "move-up" => Some(MouseInput::MoveUp),
            "move-down" => Some(MouseInput::MoveDown),
            "move-right" => Some(MouseInput::MoveRight),
            "move-left" => Some(MouseInput::MoveLeft),
            "click-left" => Some(MouseInput::ClickLeft),
            "click-middle" => Some(MouseInput::ClickMiddle),
            "click-right" => Some(MouseInput::ClickRight),
            _ => None,
        }
    }
}

#[derive(
    Format, Archive, Deserialize, Serialize, Debug, PartialEq, Default, Clone, Copy, EnumIter,
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
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "A" => Some(KeyboardInput::A),
            "B" => Some(KeyboardInput::B),
            "UpArrow" => Some(KeyboardInput::UpArrow),
            "DownArrow" => Some(KeyboardInput::DownArrow),
            "LeftArrow" => Some(KeyboardInput::LeftArrow),
            "RightArrow" => Some(KeyboardInput::RightArrow),
            "PageUp" => Some(KeyboardInput::PageUp),
            "PageDown" => Some(KeyboardInput::PageDown),
            "Backspace" => Some(KeyboardInput::Backspace),
            "Delete" => Some(KeyboardInput::Delete),
            "Tab" => Some(KeyboardInput::Tab),
            "Enter" => Some(KeyboardInput::Enter),
            _ => None,
        }
    }
    pub fn from_string_kebab(s: &str) -> Option<Self> {
        match s {
            "a" => Some(KeyboardInput::A),
            "b" => Some(KeyboardInput::B),
            "up-arrow" => Some(KeyboardInput::UpArrow),
            "down-arrow" => Some(KeyboardInput::DownArrow),
            "left-arrow" => Some(KeyboardInput::LeftArrow),
            "right-arrow" => Some(KeyboardInput::RightArrow),
            "page-up" => Some(KeyboardInput::PageUp),
            "page-down" => Some(KeyboardInput::PageDown),
            "backspace" => Some(KeyboardInput::Backspace),
            "delete" => Some(KeyboardInput::Delete),
            "tab" => Some(KeyboardInput::Tab),
            "enter" => Some(KeyboardInput::Enter),
            _ => None,
        }
    }
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
}

impl DPedalControl {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "DoNothing" => Some(DPedalControl::DoNothing),
            _ => None,
        }
    }
}
