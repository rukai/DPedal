use keyberon::key_code::KeyCode;
use miette::IntoDiagnostic;
use std::fs::read_to_string;
use std::path::Path;

const CONFIG_LOCATION: usize = 0x8000;

pub fn append_config_to_firmware(path: &Path, binary: &mut Vec<u8>) -> miette::Result<()> {
    let text = &read_to_string(path).into_diagnostic()?;
    let config: Config = knuffel::parse(path.to_str().unwrap(), text)?;
    if binary.len() > CONFIG_LOCATION {
        panic!("firmware is > 32KB");
    }
    binary.resize(CONFIG_LOCATION, 0);
    binary.push(parse_mapping(&config.pad.up)?);
    binary.push(parse_mapping(&config.pad.down)?);
    binary.push(parse_mapping(&config.pad.left)?);
    binary.push(parse_mapping(&config.pad.right)?);

    binary.push(parse_mapping(&config.side_buttons.top_left)?);
    binary.push(parse_mapping(&config.side_buttons.top_right)?);
    binary.push(parse_mapping(&config.side_buttons.bottom_left)?);
    binary.push(parse_mapping(&config.side_buttons.bottom_right)?);

    Ok(())
}

fn parse_mapping(key: &str) -> miette::Result<u8> {
    // TODO: make this lookup case insensitive
    let keymap = key
        .parse::<KeyMap>()
        // TODO: include span
        .map_err(|_| miette::miette!("invalid key {:?}", key))?;
    Ok(keymap.into_keycode() as u8)
}
#[derive(knuffel::Decode)]
pub struct Config {
    #[knuffel(child)]
    pub pad: Pad,
    #[knuffel(child)]
    pub side_buttons: SideButtons,
    #[knuffel(child)]
    #[allow(unused)]
    pub back_extension: BackExtension,
}

#[derive(knuffel::Decode)]
pub struct Pad {
    #[knuffel(child, unwrap(argument))]
    pub left: String,
    #[knuffel(child, unwrap(argument))]
    pub right: String,
    #[knuffel(child, unwrap(argument))]
    pub up: String,
    #[knuffel(child, unwrap(argument))]
    pub down: String,
}

#[derive(knuffel::Decode)]
pub struct SideButtons {
    #[knuffel(child, unwrap(argument))]
    pub top_left: String,
    #[knuffel(child, unwrap(argument))]
    pub top_right: String,
    #[knuffel(child, unwrap(argument))]
    pub bottom_left: String,
    #[knuffel(child, unwrap(argument))]
    pub bottom_right: String,
}

#[allow(unused)]
#[derive(knuffel::Decode)]
pub struct BackExtension {
    #[knuffel(child, unwrap(argument))]
    pub button1: String,
    #[knuffel(child, unwrap(argument))]
    pub button2: String,
    #[knuffel(child, unwrap(argument))]
    pub button3: String,
    #[knuffel(child, unwrap(argument))]
    pub button4: String,
}

#[derive(enum_utils::FromStr)]
pub enum KeyMap {
    Nothing,
    MouseScrollUp,
    MouseScrollDown,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M, // 0x10
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// `1` and `!`.
    Keyboard1,
    /// `2` and `@`.
    Keyboard2,
    /// `3` and `#`.
    Keyboard3, // 0x20
    /// `4` and `$`.
    Keyboard4,
    /// `5` and `%`.
    Keyboard5,
    /// `6` and `^`.
    Keyboard6,
    /// `7` and `&`.
    Keyboard7,
    /// `8` and `*`.
    Keyboard8,
    /// `9` and `(`.
    Keyboard9,
    /// `0` and `)`.
    Keyboard0,
    Enter,
    Escape,
    Backspace,
    Tab,
    Space,
    /// `-` and `_`.
    Minus,
    /// `=` and `+`.
    Equal,
    /// `[` and `{`.
    LeftBracket,
    /// `]` and `}`.
    RightBracket, // 0x30
    /// `\` and `|`.
    Backslash,
    /// Non-US `#` and `~` (Typically near the Enter key).
    NonUsHash,
    /// `;` and `:`.
    Semicolon,
    /// `'` and `"`.
    Quote,
    // How to have ` as code?
    /// \` and `~`.
    Grave,
    /// `,` and `<`.
    Comma,
    /// `.` and `>`.
    Dot,
    /// `/` and `?`.
    Slash,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7, // 0x40
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PgUp,
    Delete,
    End,
    PgDown,
    Right,
    Left, // 0x50
    Down,
    Up,
    NumLock,
    /// Keypad `/`
    KeypadSlash,
    /// Keypad `*`
    KeypadAsterisk,
    /// Keypad `-`.
    KeypadMinus,
    /// Keypad `+`.
    KeypadPlus,
    /// Keypad enter.
    KeypadEnter,
    /// Keypad 1.
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad4,
    Keypad5,
    Keypad6,
    Keypad7,
    Keypad8, // 0x60
    Keypad9,
    Keypad0,
    KeypadDot,
    /// Non-US `\` and `|` (Typically near the Left-Shift key)
    NonUsBackslash,
    Application, // 0x65
    /// not a key, used for errors
    Power,
    /// Keypad `=`.
    KeypadEqual,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21, // 0x70
    F22,
    F23,
    F24,
    Execute,
    Help,
    Menu,
    Select,
    Stop,
    Again,
    Undo,
    Cut,
    Copy,
    Paste,
    Find,
    Mute,
    VolUp, // 0x80
    VolDown,
    /// Deprecated.
    LockingCapsLock,
    /// Deprecated.
    LockingNumLock,
    /// Deprecated.
    LockingScrollLock,
    /// Keypad `,`, also used for the brazilian keypad period (.) key.
    KeypadComma,
    /// Used on AS/400 keyboard
    KeypadEqualSign,
    Intl1,
    Intl2,
    Intl3,
    Intl4,
    Intl5,
    Intl6,
    Intl7,
    Intl8,
    Intl9,
    Lang1, // 0x90
    Lang2,
    Lang3,
    Lang4,
    Lang5,
    Lang6,
    Lang7,
    Lang8,
    Lang9,
    AltErase,
    SysReq,
    Cancel,
    Clear,
    Prior,
    Return,
    Separator,
    Out, // 0xA0
    Oper,
    ClearAgain,
    CrSel,
    ExSel,

    // According to QMK, 0xA5-0xDF are not usable on modern keyboards

    // Modifiers
    /// Left Control.
    LeftCtrl = 0xE0,
    /// Left Shift.
    LeftShift,
    /// Left Alt.
    LeftAlt,
    /// Left GUI (the Windows key).
    LeftWindows,
    /// Right Control.
    RightCtrl,
    /// Right Shift.
    RightShift,
    /// Right Alt (or Alt Gr).
    RightAlt,
    /// Right GUI (the Windows key).
    RightWindows, // 0xE7

    // Unofficial
    MediaPlayPause = 0xE8,
    MediaStopCD,
    MediaPreviousSong,
    MediaNextSong,
    MediaEjectCD,
    MediaVolUp,
    MediaVolDown,
    MediaMute,
    MediaWWW, // 0xF0
    MediaBack,
    MediaForward,
    MediaStop,
    MediaFind,
    MediaScrollUp,
    MediaScrollDown,
    MediaEdit,
    MediaSleep,
    MeidaCoffee,
    MediaRefresh,
    MediaCalc, // 0xFB
}

impl KeyMap {
    fn into_keycode(self) -> KeyCode {
        match self {
            KeyMap::Nothing => KeyCode::No,
            KeyMap::MouseScrollUp => KeyCode::MediaScrollUp,
            KeyMap::MouseScrollDown => KeyCode::MediaScrollDown,
            KeyMap::A => KeyCode::A,
            KeyMap::B => KeyCode::B,
            KeyMap::C => KeyCode::C,
            KeyMap::D => KeyCode::D,
            KeyMap::E => KeyCode::E,
            KeyMap::F => KeyCode::F,
            KeyMap::G => KeyCode::G,
            KeyMap::H => KeyCode::H,
            KeyMap::I => KeyCode::I,
            KeyMap::J => KeyCode::J,
            KeyMap::K => KeyCode::K,
            KeyMap::L => KeyCode::L,
            KeyMap::M => KeyCode::M,
            KeyMap::N => KeyCode::N,
            KeyMap::O => KeyCode::O,
            KeyMap::P => KeyCode::P,
            KeyMap::Q => KeyCode::Q,
            KeyMap::R => KeyCode::R,
            KeyMap::S => KeyCode::S,
            KeyMap::T => KeyCode::T,
            KeyMap::U => KeyCode::U,
            KeyMap::V => KeyCode::V,
            KeyMap::W => KeyCode::W,
            KeyMap::X => KeyCode::X,
            KeyMap::Y => KeyCode::Y,
            KeyMap::Z => KeyCode::Z,
            KeyMap::Keyboard1 => KeyCode::Kb1,
            KeyMap::Keyboard2 => KeyCode::Kb2,
            KeyMap::Keyboard3 => KeyCode::Kb3,
            KeyMap::Keyboard4 => KeyCode::Kb4,
            KeyMap::Keyboard5 => KeyCode::Kb5,
            KeyMap::Keyboard6 => KeyCode::Kb6,
            KeyMap::Keyboard7 => KeyCode::Kb7,
            KeyMap::Keyboard8 => KeyCode::Kb8,
            KeyMap::Keyboard9 => KeyCode::Kb9,
            KeyMap::Keyboard0 => KeyCode::Kb0,
            KeyMap::Enter => KeyCode::Enter,
            KeyMap::Escape => KeyCode::Escape,
            KeyMap::Backspace => KeyCode::BSpace,
            KeyMap::Tab => KeyCode::Tab,
            KeyMap::Space => KeyCode::Space,
            KeyMap::Minus => KeyCode::Minus,
            KeyMap::Equal => KeyCode::Equal,
            KeyMap::LeftBracket => KeyCode::LBracket,
            KeyMap::RightBracket => KeyCode::RBracket,
            KeyMap::Backslash => KeyCode::Bslash,
            KeyMap::NonUsHash => KeyCode::NonUsHash,
            KeyMap::Semicolon => KeyCode::SColon,
            KeyMap::Quote => KeyCode::Quote,
            KeyMap::Grave => KeyCode::Grave,
            KeyMap::Comma => KeyCode::Comma,
            KeyMap::Dot => KeyCode::Dot,
            KeyMap::Slash => KeyCode::Slash,
            KeyMap::CapsLock => KeyCode::CapsLock,
            KeyMap::F1 => KeyCode::F1,
            KeyMap::F2 => KeyCode::F2,
            KeyMap::F3 => KeyCode::F3,
            KeyMap::F4 => KeyCode::F4,
            KeyMap::F5 => KeyCode::F5,
            KeyMap::F6 => KeyCode::F6,
            KeyMap::F7 => KeyCode::F7,
            KeyMap::F8 => KeyCode::F8,
            KeyMap::F9 => KeyCode::F9,
            KeyMap::F10 => KeyCode::F10,
            KeyMap::F11 => KeyCode::F12,
            KeyMap::F12 => KeyCode::F12,
            KeyMap::PrintScreen => KeyCode::PScreen,
            KeyMap::ScrollLock => KeyCode::ScrollLock,
            KeyMap::Pause => KeyCode::Pause,
            KeyMap::Insert => KeyCode::Insert,
            KeyMap::Home => KeyCode::Home,
            KeyMap::PgUp => KeyCode::PgUp,
            KeyMap::Delete => KeyCode::Delete,
            KeyMap::End => KeyCode::End,
            KeyMap::PgDown => KeyCode::PgDown,
            KeyMap::Right => KeyCode::Right,
            KeyMap::Left => KeyCode::Left,
            KeyMap::Down => KeyCode::Down,
            KeyMap::Up => KeyCode::Up,
            KeyMap::NumLock => KeyCode::NumLock,
            KeyMap::KeypadSlash => KeyCode::KpSlash,
            KeyMap::KeypadAsterisk => KeyCode::KpAsterisk,
            KeyMap::KeypadMinus => KeyCode::KpMinus,
            KeyMap::KeypadPlus => KeyCode::KpPlus,
            KeyMap::KeypadEnter => KeyCode::KpEnter,
            KeyMap::Keypad1 => KeyCode::Kp1,
            KeyMap::Keypad2 => KeyCode::Kp2,
            KeyMap::Keypad3 => KeyCode::Kp3,
            KeyMap::Keypad4 => KeyCode::Kp4,
            KeyMap::Keypad5 => KeyCode::Kp5,
            KeyMap::Keypad6 => KeyCode::Kp6,
            KeyMap::Keypad7 => KeyCode::Kp7,
            KeyMap::Keypad8 => KeyCode::Kp8,
            KeyMap::Keypad9 => KeyCode::Kp9,
            KeyMap::Keypad0 => KeyCode::Kp0,
            KeyMap::KeypadDot => KeyCode::KpDot,
            KeyMap::NonUsBackslash => KeyCode::NonUsBslash,
            KeyMap::Application => KeyCode::Application,
            KeyMap::Power => KeyCode::Power,
            KeyMap::KeypadEqual => KeyCode::KpEqual,
            KeyMap::F13 => KeyCode::F13,
            KeyMap::F14 => KeyCode::F14,
            KeyMap::F15 => KeyCode::F15,
            KeyMap::F16 => KeyCode::F16,
            KeyMap::F17 => KeyCode::F17,
            KeyMap::F18 => KeyCode::F18,
            KeyMap::F19 => KeyCode::F19,
            KeyMap::F20 => KeyCode::F20,
            KeyMap::F21 => KeyCode::F21,
            KeyMap::F22 => KeyCode::F22,
            KeyMap::F23 => KeyCode::F23,
            KeyMap::F24 => KeyCode::F24,
            KeyMap::Execute => KeyCode::Execute,
            KeyMap::Help => KeyCode::Help,
            KeyMap::Menu => KeyCode::Menu,
            KeyMap::Select => KeyCode::Select,
            KeyMap::Stop => KeyCode::Stop,
            KeyMap::Again => KeyCode::Again,
            KeyMap::Undo => KeyCode::Undo,
            KeyMap::Cut => KeyCode::Cut,
            KeyMap::Copy => KeyCode::Copy,
            KeyMap::Paste => KeyCode::Paste,
            KeyMap::Find => KeyCode::Find,
            KeyMap::Mute => KeyCode::Mute,
            KeyMap::VolUp => KeyCode::VolUp,
            KeyMap::VolDown => KeyCode::VolDown,
            KeyMap::LockingCapsLock => KeyCode::LockingCapsLock,
            KeyMap::LockingNumLock => KeyCode::LockingNumLock,
            KeyMap::LockingScrollLock => KeyCode::LockingScrollLock,
            KeyMap::KeypadComma => KeyCode::KpComma,
            KeyMap::KeypadEqualSign => KeyCode::KpEqualSign,
            KeyMap::Intl1 => KeyCode::Intl1,
            KeyMap::Intl2 => KeyCode::Intl2,
            KeyMap::Intl3 => KeyCode::Intl3,
            KeyMap::Intl4 => KeyCode::Intl4,
            KeyMap::Intl5 => KeyCode::Intl5,
            KeyMap::Intl6 => KeyCode::Intl6,
            KeyMap::Intl7 => KeyCode::Intl7,
            KeyMap::Intl8 => KeyCode::Intl8,
            KeyMap::Intl9 => KeyCode::Intl9,
            KeyMap::Lang1 => KeyCode::Lang1,
            KeyMap::Lang2 => KeyCode::Lang2,
            KeyMap::Lang3 => KeyCode::Lang3,
            KeyMap::Lang4 => KeyCode::Lang4,
            KeyMap::Lang5 => KeyCode::Lang5,
            KeyMap::Lang6 => KeyCode::Lang6,
            KeyMap::Lang7 => KeyCode::Lang7,
            KeyMap::Lang8 => KeyCode::Lang8,
            KeyMap::Lang9 => KeyCode::Lang9,
            KeyMap::AltErase => KeyCode::AltErase,
            KeyMap::SysReq => KeyCode::SysReq,
            KeyMap::Cancel => KeyCode::Cancel,
            KeyMap::Clear => KeyCode::Clear,
            KeyMap::Prior => KeyCode::Prior,
            KeyMap::Return => KeyCode::Return,
            KeyMap::Separator => KeyCode::Separator,
            KeyMap::Out => KeyCode::Out,
            KeyMap::Oper => KeyCode::Oper,
            KeyMap::ClearAgain => KeyCode::ClearAgain,
            KeyMap::CrSel => KeyCode::CrSel,
            KeyMap::ExSel => KeyCode::ExSel,
            KeyMap::LeftCtrl => KeyCode::LCtrl,
            KeyMap::LeftShift => KeyCode::LShift,
            KeyMap::LeftAlt => KeyCode::LAlt,
            KeyMap::LeftWindows => KeyCode::LGui,
            KeyMap::RightCtrl => KeyCode::RCtrl,
            KeyMap::RightShift => KeyCode::RShift,
            KeyMap::RightAlt => KeyCode::RAlt,
            KeyMap::RightWindows => KeyCode::RGui,
            KeyMap::MediaPlayPause => KeyCode::MediaPlayPause,
            KeyMap::MediaStopCD => KeyCode::MediaStopCD,
            KeyMap::MediaPreviousSong => KeyCode::MediaPreviousSong,
            KeyMap::MediaNextSong => KeyCode::MediaNextSong,
            KeyMap::MediaEjectCD => KeyCode::MediaEjectCD,
            KeyMap::MediaVolUp => KeyCode::MediaVolUp,
            KeyMap::MediaVolDown => KeyCode::MediaVolDown,
            KeyMap::MediaMute => KeyCode::MediaMute,
            KeyMap::MediaWWW => KeyCode::MediaWWW,
            KeyMap::MediaBack => KeyCode::MediaBack,
            KeyMap::MediaForward => KeyCode::MediaForward,
            KeyMap::MediaStop => KeyCode::MediaStop,
            KeyMap::MediaFind => KeyCode::MediaFind,
            KeyMap::MediaScrollUp => KeyCode::MediaScrollUp,
            KeyMap::MediaScrollDown => KeyCode::MediaScrollDown,
            KeyMap::MediaEdit => KeyCode::MediaEdit,
            KeyMap::MediaSleep => KeyCode::MediaSleep,
            KeyMap::MeidaCoffee => KeyCode::MeidaCoffee,
            KeyMap::MediaRefresh => KeyCode::MediaRefresh,
            KeyMap::MediaCalc => KeyCode::MediaCalc,
        }
    }
}
