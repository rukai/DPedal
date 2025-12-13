#![no_main]
#![no_std]

mod config;
mod keyboard;
mod mouse;
mod usb;
mod web_config;

use crate::config::ConfigFlash;
use crate::keyboard::{KEYBOARD_CHANNEL, Keyboard, KeyboardEvent};
use crate::mouse::{MOUSE_CHANNEL, Mouse, MouseEvent};
use crate::web_config::WebConfig;
use dpedal_config::{ComputerInput, DpedalInput, InputSplit, MouseInput};
use embassy_executor::Spawner;
use embassy_futures::join::join5;
use embassy_rp::gpio::{AnyPin, Input, Pin, Pull};
use embassy_rp::{Peri, PeripheralType};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let config_flash = ConfigFlash::new(p.FLASH);
    let config = config_flash.load().unwrap_or_default();

    let mut builder = usb::usb_builder(p.USB);

    let mut web_config = WebConfig::new(&mut builder, config_flash);
    let mut keyboard = Keyboard::new(&mut builder);
    let mut mouse = Mouse::new(&mut builder);

    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let main = async {
        let mut pins: [Option<Peri<AnyPin>>; _] = [
            Some(p.PIN_0.into()),
            Some(p.PIN_1.into()),
            Some(p.PIN_2.into()),
            Some(p.PIN_3.into()),
            Some(p.PIN_4.into()),
            Some(p.PIN_5.into()),
            Some(p.PIN_6.into()),
            Some(p.PIN_7.into()),
            Some(p.PIN_8.into()),
            Some(p.PIN_9.into()),
            Some(p.PIN_10.into()),
            Some(p.PIN_11.into()),
            Some(p.PIN_12.into()),
            Some(p.PIN_13.into()),
            Some(p.PIN_14.into()),
            Some(p.PIN_15.into()),
            Some(p.PIN_16.into()),
            Some(p.PIN_17.into()),
            Some(p.PIN_18.into()),
            Some(p.PIN_19.into()),
            Some(p.PIN_20.into()),
            Some(p.PIN_21.into()),
            Some(p.PIN_22.into()),
            Some(p.PIN_23.into()),
            Some(p.PIN_24.into()),
            Some(p.PIN_25.into()),
            Some(p.PIN_26.into()),
            Some(p.PIN_27.into()),
            Some(p.PIN_28.into()),
            Some(p.PIN_29.into()),
        ];

        let mut button_left_pin = 13;
        let mut button_right_pin = 27;
        let mut dpad_up_pin = 26;
        let mut dpad_down_pin = 16;
        let mut dpad_left_pin = 17;
        let mut dpad_right_pin = 22;
        for remapping in config.pin_remappings {
            match remapping.input {
                DpedalInput::DpadUp => dpad_up_pin = remapping.pin as usize,
                DpedalInput::DpadDown => dpad_down_pin = remapping.pin as usize,
                DpedalInput::DpadLeft => dpad_left_pin = remapping.pin as usize,
                DpedalInput::DpadRight => dpad_right_pin = remapping.pin as usize,
                DpedalInput::ButtonLeft => button_left_pin = remapping.pin as usize,
                DpedalInput::ButtonRight => button_right_pin = remapping.pin as usize,
            }
        }

        let button_left = input(pins[button_left_pin].take().unwrap());
        let button_right = input(pins[button_right_pin].take().unwrap());
        let dpad_up = input(pins[dpad_up_pin].take().unwrap());
        let dpad_down = input(pins[dpad_down_pin].take().unwrap());
        let dpad_left = input(pins[dpad_left_pin].take().unwrap());
        let dpad_right = input(pins[dpad_right_pin].take().unwrap());

        loop {
            let profile = &config.profiles[0];
            Timer::after_millis(1).await;

            let input_state = DpedalInputState {
                button_left: button_left.is_low(),
                button_right: button_right.is_low(),
                dpad_up: dpad_up.is_low(),
                dpad_down: dpad_down.is_low(),
                dpad_left: dpad_left.is_low(),
                dpad_right: dpad_right.is_low(),
            };

            for mapping in &profile.mappings {
                if input_state.is_all_pressed(&mapping.input) {
                    for output in &mapping.output {
                        pressed(*output).await;
                    }
                } else {
                    for output in &mapping.output {
                        released(*output).await;
                    }
                }
            }
        }
    };

    join5(
        usb_fut,
        main,
        keyboard.process(),
        mouse.process(),
        web_config.process(),
    )
    .await;
}

struct DpedalInputState {
    button_left: bool,
    button_right: bool,
    dpad_up: bool,
    dpad_down: bool,
    dpad_left: bool,
    dpad_right: bool,
}

impl DpedalInputState {
    fn is_all_pressed(&self, check: &[DpedalInput]) -> bool {
        // Disable the mapping when the inputs are entirely empty
        // It is an obvious configuration mistake and having it constantly trigger the input would be very annoying
        if check.is_empty() {
            return false;
        }

        for input in check {
            let pressed = match input {
                DpedalInput::DpadUp => self.dpad_up,
                DpedalInput::DpadDown => self.dpad_down,
                DpedalInput::DpadLeft => self.dpad_left,
                DpedalInput::DpadRight => self.dpad_right,
                DpedalInput::ButtonLeft => self.button_left,
                DpedalInput::ButtonRight => self.button_right,
            };

            if !pressed {
                return false;
            }
        }
        true
    }
}

async fn pressed(input: ComputerInput) {
    match input.split() {
        InputSplit::None => {}
        InputSplit::Keyboard(key) => KEYBOARD_CHANNEL.send(KeyboardEvent::Pressed(key)).await,
        InputSplit::Mouse(input) => {
            MOUSE_CHANNEL
                .send(match input {
                    MouseInput::Scroll { x, y } => MouseEvent::Scroll { x, y },
                    MouseInput::Move { x, y } => MouseEvent::Move { x, y },
                    MouseInput::Click(click) => MouseEvent::Pressed(click),
                })
                .await;
        }
    }
}

async fn released(input: ComputerInput) {
    match input.split() {
        InputSplit::None => {}
        InputSplit::Keyboard(key) => KEYBOARD_CHANNEL.send(KeyboardEvent::Released(key)).await,
        InputSplit::Mouse(MouseInput::Click(click)) => {
            MOUSE_CHANNEL.send(MouseEvent::Released(click)).await
        }
        InputSplit::Mouse(MouseInput::Move { .. } | MouseInput::Scroll { .. }) => {}
    }
}

// TODO: become Input::new
fn input<T: PeripheralType + Pin>(pin: Peri<'static, T>) -> Input<'static> {
    let mut pin = Input::new(pin, Pull::Up);
    pin.set_schmitt(true);
    pin
}
