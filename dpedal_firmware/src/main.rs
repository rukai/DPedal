#![no_main]
#![no_std]

mod config;
mod keyboard;
mod mouse;
mod usb;
mod web_config;

use crate::keyboard::{KEYBOARD_CHANNEL, Keyboard, KeyboardEvent};
use crate::mouse::{MOUSE_CHANNEL, Mouse, MouseEvent};
use crate::web_config::WebConfig;
use dpedal_config::{ComputerInput, DpedalInput, InputSplit, MouseInput};
use embassy_executor::Spawner;
use embassy_futures::join::join5;
use embassy_rp::gpio::{Input, Pin, Pull};
use embassy_rp::{Peri, PeripheralType};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut builder = usb::usb_builder(p.USB);

    let mut web_config = WebConfig::new(&mut builder);
    let mut keyboard = Keyboard::new(&mut builder);
    let mut mouse = Mouse::new(&mut builder);

    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let main = async {
        let config = config::load().unwrap_or_default();

        // inputs
        let button_left = input(p.PIN_3);
        let button_right = input(p.PIN_20);
        let dpad_up = input(p.PIN_27);
        let dpad_down = input(p.PIN_7);
        let dpad_left = input(p.PIN_16);
        let dpad_right = input(p.PIN_15);

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
