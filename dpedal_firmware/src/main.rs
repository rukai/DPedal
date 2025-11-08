#![no_main]
#![no_std]

mod config;
mod keyboard;
mod mouse;
mod usb;

use crate::keyboard::{KEYBOARD_CHANNEL, Keyboard, KeyboardEvent};
use crate::mouse::{MOUSE_CHANNEL, Mouse, MouseEvent};
use dpedal_config::{ComputerInput, InputSplit, MouseInput};
use embassy_executor::Spawner;
use embassy_futures::join::join4;
use embassy_rp::gpio::{Input, Pin, Pull};
use embassy_rp::{Peri, PeripheralType};
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut builder = usb::usb_builder(p.USB);

    let mut keyboard = Keyboard::new(&mut builder);
    let mut mouse = Mouse::new(&mut builder);

    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let main = async {
        let config = config::load().unwrap(); // TODO: handle this error

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

            handle_input(&button_left, profile.button_left).await;
            handle_input(&button_right, profile.button_right).await;
            handle_input(&dpad_up, profile.dpad_up).await;
            handle_input(&dpad_down, profile.dpad_down).await;
            handle_input(&dpad_left, profile.dpad_left).await;
            handle_input(&dpad_right, profile.dpad_right).await;
        }
    };

    join4(usb_fut, main, keyboard.process(), mouse.process()).await;
}

async fn handle_input(pin: &Input<'static>, config: ComputerInput) {
    if pin.is_low() {
        pressed(config).await;
    } else {
        released(config).await;
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
