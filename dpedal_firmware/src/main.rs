#![no_main]
#![no_std]

mod config;

use dpedal_config::ComputerInput;
// set the panic handler
use panic_probe as _;

use defmt::*;
use defmt_rtt as _;
use hal::gpio::DynPinId;
use hal::usb;
use keyberon::action::Action;
use keyberon::debounce::Debouncer;
use keyberon::key_code::KeyCode;
use keyberon::layout::CustomEvent;
use keyberon::layout::{Event, Layers, Layout};
use keyberon::matrix::DirectPinMatrix;
use once_cell::sync::Lazy;
use rp2040_hal as hal;
use rtic::app;
use usb_device::bus::UsbBusAllocator;
use usbd_human_interface_device::device::keyboard::{NKROBootKeyboard, NKROBootKeyboardConfig};
use usbd_human_interface_device::device::mouse::{WheelMouse, WheelMouseConfig, WheelMouseReport};
use usbd_human_interface_device::page::Keyboard;
use usbd_human_interface_device::prelude::*;

use bsp::hal::{
    clocks::{Clock, init_clocks_and_plls},
    sio::Sio,
    watchdog::Watchdog,
};
use embedded_hal::digital::{InputPin, OutputPin};
use rp_pico as bsp;
use rp2040_hal::{gpio::Pin, rom_data::reset_to_usb_boot};

type UsbDevice = usb_device::device::UsbDevice<'static, usb::UsbBus>;
type MultiDevice = frunk::HList!(
    WheelMouse<'static, hal::usb::UsbBus>,
    NKROBootKeyboard<'static, hal::usb::UsbBus>,
);

// TODO: We'll probably need to vendor the keyberon bits we need so we can make layers runtime configurable
static LAYERS: Lazy<Layers<6, 1, 1, MouseEvent>> = Lazy::new(|| unsafe {
    let config = config::load().unwrap(); // TODO: handle this error
    [[[
        map_to_keyberon(config.profiles[0].button_left),
        map_to_keyberon(config.profiles[0].button_right),
        map_to_keyberon(config.profiles[0].dpad_up),
        map_to_keyberon(config.profiles[0].dpad_down),
        map_to_keyberon(config.profiles[0].dpad_left),
        map_to_keyberon(config.profiles[0].dpad_right),
    ]]]
});

#[allow(unused)]
#[derive(Clone, Copy)]
enum MouseEvent {
    ScrollUp,
    ScrollDown,
    ClickLeft,
    ClickMiddle,
    ClickRight,
}

#[derive(Clone, Copy, Default)]
struct MouseState {
    ticks_since_last_change: u32,
    event: Option<MouseEvent>,
}

unsafe fn map_to_keyberon(input: ComputerInput) -> Action<MouseEvent> {
    match input {
        ComputerInput::MouseScrollUp => Action::Custom(MouseEvent::ScrollUp),
        ComputerInput::MouseScrollDown => Action::Custom(MouseEvent::ScrollDown),
        ComputerInput::MouseScrollLeft => Action::KeyCode(KeyCode::A), // TODO
        ComputerInput::MouseScrollRight => Action::KeyCode(KeyCode::B), // TODO
        ComputerInput::KeyboardPageUp => Action::KeyCode(KeyCode::C),  // TODO
        ComputerInput::KeyboardPageDown => Action::KeyCode(KeyCode::D), // TODO
        ComputerInput::None => Action::KeyCode(KeyCode::No),
    }
}

#[app(device = crate::hal::pac, peripherals = true, dispatchers = [DMA_IRQ_0])]
mod app {
    use fugit::ExtU32;
    use rp2040_hal::{
        gpio::{FunctionSioInput, PullUp},
        timer::Alarm,
    };
    use usb_device::device::StringDescriptors;

    use super::*;

    #[shared]
    struct Shared {
        multi_device: UsbHidClass<'static, hal::usb::UsbBus, MultiDevice>,
        usb_device: UsbDevice,
        #[lock_free]
        layout: Layout<6, 1, 1, MouseEvent>,
    }

    #[local]
    struct Local {
        matrix: DirectPinMatrix<Pin<DynPinId, FunctionSioInput, PullUp>, 6, 1>,
        debouncer: Debouncer<[[bool; 6]; 1]>,
        timer: hal::timer::Timer,
        mouse_state: MouseState,
    }

    #[init(local = [bus: Option<UsbBusAllocator<usb::UsbBus>> = None])]
    fn init(mut c: init::Context) -> (Shared, Local, init::Monotonics) {
        info!("starting up");
        let mut watchdog = Watchdog::new(c.device.WATCHDOG);
        let sio = Sio::new(c.device.SIO);

        // External high-speed crystal on the pico board is 12Mhz
        let clocks = init_clocks_and_plls(
            rp_pico::XOSC_CRYSTAL_FREQ,
            c.device.XOSC,
            c.device.CLOCKS,
            c.device.PLL_SYS,
            c.device.PLL_USB,
            &mut c.device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let mut timer = hal::Timer::new(c.device.TIMER, &mut c.device.RESETS, &clocks);
        let mut delay =
            cortex_m::delay::Delay::new(c.core.SYST, clocks.system_clock.freq().to_Hz());

        let pins = bsp::Pins::new(
            c.device.IO_BANK0,
            c.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut c.device.RESETS,
        );

        let mut led_pin = pins.led.into_push_pull_output();
        let mut start = pins.gpio0.into_pull_up_input().into_dyn_pin();

        *c.local.bus = Some(UsbBusAllocator::new(hal::usb::UsbBus::new(
            c.device.USBCTRL_REGS,
            c.device.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut c.device.RESETS,
        )));
        let usb_bus = c.local.bus.as_ref().unwrap();
        let multi_device = UsbHidClassBuilder::new()
            .add_device(NKROBootKeyboardConfig::default())
            .add_device(WheelMouseConfig::default())
            .build(usb_bus);

        // https://pid.codes
        let usb_device = usb_device::device::UsbDeviceBuilder::new(
            usb_bus,
            usb_device::device::UsbVidPid(0x1209, 0x0001),
        )
        .strings(&[StringDescriptors::default()
            .manufacturer("rukai")
            .product("DPedal")
            .serial_number("TEST")])
        .unwrap()
        .build();

        // TODO: not sure why this is needed
        delay.delay_ms(10);

        if start.is_low().unwrap_or(true) {
            reset_to_usb_boot(0, 0);
        }

        for _ in 0..10 {
            led_pin.set_high().unwrap();
            delay.delay_ms(100);
            led_pin.set_low().unwrap();
            delay.delay_ms(100);
        }

        let mut alarm = timer.alarm_0().unwrap();
        let _ = alarm.schedule(1_u32.micros());
        alarm.enable_interrupt();

        let matrix = cortex_m::interrupt::free(move |_cs| {
            DirectPinMatrix::new([[
                // button left
                Some(pins.gpio3.into_pull_up_input().into_dyn_pin()),
                // button right
                Some(pins.gpio20.into_pull_up_input().into_dyn_pin()),
                // dpad up
                Some(pins.gpio27.into_pull_up_input().into_dyn_pin()),
                // dpad down
                Some(pins.gpio7.into_pull_up_input().into_dyn_pin()),
                // dpad left
                Some(pins.gpio16.into_pull_up_input().into_dyn_pin()),
                // dpad right
                Some(pins.gpio15.into_pull_up_input().into_dyn_pin()),
            ]])
            .unwrap()
        });

        info!("finished startup");

        (
            Shared {
                multi_device,
                usb_device,
                layout: Layout::new(&LAYERS),
            },
            Local {
                timer,
                debouncer: Debouncer::new([[false; 6]; 1], [[false; 6]; 1], 5),
                matrix,
                mouse_state: MouseState::default(),
            },
            init::Monotonics(),
        )
    }

    #[task(binds = USBCTRL_IRQ, priority = 3, shared = [multi_device, usb_device])]
    fn usb_rx(c: usb_rx::Context) {
        (c.shared.multi_device, c.shared.usb_device).lock(|multi_device, usb_device| {
            if usb_device.poll(&mut [multi_device]) {
                let interface = multi_device.device::<NKROBootKeyboard<'_, _>, _>();
                match interface.read_report() {
                    Err(usb_device::UsbError::WouldBlock) => {}
                    Err(e) => core::panic!("Failed to read keyboard report: {:?}", e),
                    Ok(_) => {}
                }
            }
        });
    }

    #[task(priority = 2, capacity = 8, shared = [layout])]
    fn handle_event(c: handle_event::Context, event: Event) {
        info!("handle event");
        c.shared.layout.event(event)
    }

    #[task(priority = 2, shared = [layout, multi_device], local = [mouse_state])]
    fn tick_keyberon(mut c: tick_keyberon::Context) {
        let mut mouse_report = WheelMouseReport::default();
        let mut force_report = false;
        match c.shared.layout.tick() {
            CustomEvent::Press(e) => {
                info!("press");
                c.local.mouse_state.event = Some(*e);
                c.local.mouse_state.ticks_since_last_change = 0;
                // TODO: doesnt allow pressing multiple mouse clicks at once
                match e {
                    MouseEvent::ClickLeft => mouse_report.buttons = 1,
                    MouseEvent::ClickMiddle => mouse_report.buttons = 2,
                    MouseEvent::ClickRight => mouse_report.buttons = 4,
                    _ => (),
                }
            }
            CustomEvent::Release(e) => {
                info!("release");
                c.local.mouse_state.event = None;
                c.local.mouse_state.ticks_since_last_change = 0;
                match e {
                    MouseEvent::ClickLeft => force_report = true,
                    MouseEvent::ClickMiddle => force_report = true,
                    MouseEvent::ClickRight => force_report = true,
                    _ => (),
                }
            }
            CustomEvent::NoEvent => {}
        }

        let t = c.local.mouse_state.ticks_since_last_change;
        match c.local.mouse_state.event {
            Some(MouseEvent::ScrollDown) => {
                info!("scroll down");
                mouse_report.vertical_wheel = -if t.is_multiple_of(100) { 1 } else { 0 }
            }
            Some(MouseEvent::ScrollUp) => {
                info!("scroll up");
                mouse_report.vertical_wheel = if t.is_multiple_of(100) { 1 } else { 0 }
            }
            Some(_) => {
                info!("other");
            }
            None => {}
        }

        // Run this after generating mouse_report to ensure the first tick is at 0
        c.local.mouse_state.ticks_since_last_change += 1;

        let keyboard_report = c.shared.layout.keycodes().map(|x| Keyboard::from(x as u8));

        c.shared.multi_device.lock(|multi_device| {
            match multi_device
                .device::<NKROBootKeyboard<'_, _>, _>()
                .write_report(keyboard_report)
            {
                Err(UsbHidError::WouldBlock) => {}
                Err(UsbHidError::Duplicate) => {}
                Ok(_) => {}
                Err(e) => core::panic!("Failed to write keyboard report: {:?}", e),
            }

            // Sending empty mouse reports achieves nothing and appears to cause issues by sending too many reports
            // So we check for empty reports and skip
            if mouse_report != WheelMouseReport::default() || force_report {
                let mouse = multi_device.device::<WheelMouse<'_, _>, _>();
                match mouse.write_report(&mouse_report) {
                    Err(UsbHidError::WouldBlock) => {}
                    Ok(_) => {}
                    Err(e) => core::panic!("Failed to write mouse report: {:?}", e),
                };
            }
        });

        c.shared.multi_device.lock(|k| match k.tick() {
            Err(UsbHidError::WouldBlock) => {}
            Ok(_) => {}
            Err(e) => core::panic!("Failed to process keyboard tick: {:?}", e),
        });
    }

    #[task(
        binds = TIMER_IRQ_0,
        priority = 1,
        local = [matrix, debouncer, timer],
    )]
    fn tick(c: tick::Context) {
        for event in c.local.debouncer.events(c.local.matrix.get().unwrap()) {
            handle_event::spawn(event).unwrap();
        }

        tick_keyberon::spawn().unwrap();
    }
}

// Implemented by reference to these examples:
// https://github.com/dlkj/usbd-human-interface-device/blob/main/examples/src/bin/keyboard_rtic.rs
// https://github.com/dlkj/usbd-human-interface-device/blob/main/examples/src/bin/multi_device.rs
// https://github.com/dlkj/usbd-human-interface-device/blob/main/examples/src/bin/keyboard_nkro.rs
// https://github.com/dlkj/usbd-human-interface-device/blob/main/examples/src/bin/mouse_wheel.rs
