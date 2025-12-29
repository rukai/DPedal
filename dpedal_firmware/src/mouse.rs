use defmt::*;
use dpedal_config::MouseInput;
use embassy_futures::join::join;
use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_usb::{
    Builder,
    class::hid::{HidBootProtocol, HidReader, HidReaderWriter, HidSubclass, HidWriter, State},
};
use static_cell::StaticCell;
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};

use crate::usb::MyRequestHandler;

pub struct Mouse {
    reader: Option<HidReader<'static, Driver<'static, USB>, 1>>,
    writer: HidWriter<'static, Driver<'static, USB>, 8>,
}

pub static MOUSE_CHANNEL: Channel<ThreadModeRawMutex, MouseEvent, 64> = Channel::new();

impl Mouse {
    pub fn new(builder: &mut Builder<'static, Driver<'static, USB>>) -> Self {
        let config = embassy_usb::class::hid::Config {
            hid_subclass: HidSubclass::Boot,
            hid_boot_protocol: HidBootProtocol::Mouse,
            report_descriptor: MouseReport::desc(),
            request_handler: None,
            poll_ms: 1,
            max_packet_size: 64,
        };
        static STATE: StaticCell<State> = StaticCell::new();
        let hid =
            HidReaderWriter::<'static, _, 1, 8>::new(builder, STATE.init(State::new()), config);
        let (reader, writer) = hid.split();

        Self {
            reader: Some(reader),
            writer,
        }
    }

    pub async fn process(&mut self) {
        let reader = self.reader.take().unwrap();

        join(
            self.process_write(),
            reader.run(false, &mut MyRequestHandler {}),
        )
        .await;
    }

    pub async fn process_write(&mut self) {
        let mut report = MouseReport {
            buttons: 0,
            x: 0,
            y: 0,
            wheel: 0,
            pan: 0,
        };

        let mut ticks = 0u32;

        loop {
            ticks = ticks.wrapping_add(1);
            // Delay processing events until we are able to actually send the report to ensure the report contains the most up to date information.
            // TODO: Actually check behaviour of this await and write_serialize await, do they actually block until host has polled us?
            self.writer.ready().await;

            while let Ok(event) = MOUSE_CHANNEL.try_receive() {
                match event {
                    MouseEvent::Pressed(input) => match input {
                        MouseInput::ScrollUp(value) => {
                            scroll(&mut report, ticks, 0, (value / 10) as i8)
                        }
                        MouseInput::ScrollDown(value) => {
                            scroll(&mut report, ticks, 0, (value / -10) as i8)
                        }
                        MouseInput::ScrollLeft(value) => {
                            scroll(&mut report, ticks, (value / -10) as i8, 0)
                        }
                        MouseInput::ScrollRight(value) => {
                            scroll(&mut report, ticks, (value / 10) as i8, 0)
                        }
                        MouseInput::MoveUp(value) => {
                            move_cursor(&mut report, ticks, 0, (value / -10) as i8)
                        }
                        MouseInput::MoveDown(value) => {
                            move_cursor(&mut report, ticks, 0, (value / 10) as i8)
                        }
                        MouseInput::MoveLeft(value) => {
                            move_cursor(&mut report, ticks, (value / -10) as i8, 0)
                        }
                        MouseInput::MoveRight(value) => {
                            move_cursor(&mut report, ticks, (value / 10) as i8, 0)
                        }
                        MouseInput::ClickLeft => report.buttons |= 0b0000_0001,
                        MouseInput::ClickRight => report.buttons |= 0b0000_0010,
                        MouseInput::ClickMiddle => report.buttons |= 0b0000_0100,
                    },
                    MouseEvent::Released(input) => match input {
                        MouseInput::ScrollUp(_)
                        | MouseInput::ScrollDown(_)
                        | MouseInput::ScrollLeft(_)
                        | MouseInput::ScrollRight(_)
                        | MouseInput::MoveUp(_)
                        | MouseInput::MoveDown(_)
                        | MouseInput::MoveLeft(_)
                        | MouseInput::MoveRight(_) => {
                            // Releasing one of these inputs has no effect
                        }
                        MouseInput::ClickLeft => report.buttons &= 0b1111_1110,
                        MouseInput::ClickRight => report.buttons &= 0b1111_1101,
                        MouseInput::ClickMiddle => report.buttons &= 0b1111_1011,
                    },
                }
            }

            // Send the report.
            match self.writer.write_serialize(&report).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            };

            // reset non-button inputs
            report.wheel = 0;
            report.x = 0;
            report.y = 0;
            report.pan = 0;
        }
    }
}

fn scroll(report: &mut MouseReport, ticks: u32, x: i8, y: i8) {
    if ticks.is_multiple_of(80) {
        report.pan += x;
        report.wheel += y;
    }
}

fn move_cursor(report: &mut MouseReport, ticks: u32, x: i8, y: i8) {
    if ticks.is_multiple_of(80) {
        report.x += x;
        report.y += y;
    }
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum MouseEvent {
    Pressed(MouseInput),
    Released(MouseInput),
}
