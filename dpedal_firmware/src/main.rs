#![no_main]
#![no_std]

// set the panic handler
use panic_halt as _;

use hal::gpio::{Input, Pin, PullUp};
use hal::prelude::*;
use hal::usb;
use hal::{stm32, timers};
use keyberon::action::Action;
use keyberon::debounce::Debouncer;
use keyberon::key_code::KeyCode;
use keyberon::layout::CustomEvent;
use keyberon::layout::{Event, Layers, Layout};
use keyberon::matrix::DirectPinMatrix;
use once_cell::sync::Lazy;
use rtic::app;
use stm32f0xx_hal as hal;
use usb_device::bus::UsbBusAllocator;
use usbd_human_interface_device::device::keyboard::{NKROBootKeyboard, NKROBootKeyboardConfig};
use usbd_human_interface_device::device::mouse::{WheelMouse, WheelMouseConfig, WheelMouseReport};
use usbd_human_interface_device::page::Keyboard;
use usbd_human_interface_device::prelude::*;

type UsbDevice = usb_device::device::UsbDevice<'static, usb::UsbBusType>;
type MultiDevice = frunk::HList!(
    WheelMouse<'static, hal::usb::UsbBusType>,
    NKROBootKeyboard<'static, hal::usb::UsbBusType>,
);

// TODO: We'll probably need to vendor the keyberon bits we need so we can make layers runtime configurable
static LAYERS: Lazy<Layers<8, 1, 1, MouseEvent>> = Lazy::new(|| unsafe {
    [[[
        action_from_mem(0), // up
        action_from_mem(1), // down
        action_from_mem(2), // left
        action_from_mem(3), // right
        action_from_mem(4), // top-left
        action_from_mem(5), // top-right
        action_from_mem(6), // bottom-left
        action_from_mem(7), // bottom-right
    ]]]
});

#[derive(Clone, Copy)]
enum MouseEvent {
    ScrollUp,
    ScrollDown,
}

#[derive(Clone, Copy, Default)]
struct MouseState {
    ticks_since_last_change: u32,
    event: Option<MouseEvent>,
}

unsafe fn action_from_mem(offset: usize) -> Action<MouseEvent> {
    let config = (0x0800_8000 + offset) as *mut u8;
    let config_byte1 = unsafe { core::ptr::read_volatile(config) };
    let keycode: KeyCode = unsafe { core::mem::transmute(config_byte1) };
    match keycode {
        KeyCode::MediaScrollUp => Action::Custom(MouseEvent::ScrollUp),
        KeyCode::MediaScrollDown => Action::Custom(MouseEvent::ScrollDown),
        _ => Action::KeyCode(keycode),
    }
}

#[app(device = crate::hal::pac, peripherals = true, dispatchers = [CEC_CAN])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        multi_device: UsbHidClass<'static, hal::usb::UsbBusType, MultiDevice>,
        usb_device: UsbDevice,
        #[lock_free]
        layout: Layout<8, 1, 1, MouseEvent>,
    }

    #[local]
    struct Local {
        matrix: DirectPinMatrix<Pin<Input<PullUp>>, 8, 1>,
        debouncer: Debouncer<[[bool; 8]; 1]>,
        timer: timers::Timer<stm32::TIM3>,
        mouse_state: MouseState,
    }

    #[init(local = [bus: Option<UsbBusAllocator<usb::UsbBusType>> = None])]
    fn init(mut c: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut rcc = c
            .device
            .RCC
            .configure()
            .hsi48()
            .enable_crs(c.device.CRS)
            .sysclk(48.mhz())
            .pclk(24.mhz())
            .freeze(&mut c.device.FLASH);

        let gpioa = c.device.GPIOA.split(&mut rcc);
        let gpiob = c.device.GPIOB.split(&mut rcc);
        let gpioc = c.device.GPIOC.split(&mut rcc);

        let usb = usb::Peripheral {
            usb: c.device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };
        *c.local.bus = Some(usb::UsbBusType::new(usb));
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
        .manufacturer("rukai")
        .product("dpedal")
        .serial_number("TEST")
        .build();

        let mut timer = timers::Timer::tim3(c.device.TIM3, 1.khz(), &mut rcc);
        timer.listen(timers::Event::TimeOut);

        let matrix = cortex_m::interrupt::free(move |cs| {
            DirectPinMatrix::new([[
                Some(gpioc.pc13.into_pull_up_input(cs).downgrade()),
                Some(gpiob.pb3.into_pull_up_input(cs).downgrade()),
                Some(gpioa.pa10.into_pull_up_input(cs).downgrade()),
                Some(gpiob.pb12.into_pull_up_input(cs).downgrade()),
                Some(gpioa.pa9.into_pull_up_input(cs).downgrade()),
                Some(gpiob.pb9.into_pull_up_input(cs).downgrade()),
                Some(gpioa.pa15.into_pull_up_input(cs).downgrade()),
                Some(gpiob.pb8.into_pull_up_input(cs).downgrade()),
            ]])
            .unwrap()
        });

        (
            Shared {
                multi_device,
                usb_device,
                layout: Layout::new(&LAYERS),
            },
            Local {
                timer,
                debouncer: Debouncer::new([[false; 8]; 1], [[false; 8]; 1], 5),
                matrix,
                mouse_state: MouseState::default(),
            },
            init::Monotonics(),
        )
    }

    #[task(binds = USB, priority = 3, shared = [multi_device, usb_device])]
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
        })
    }

    #[task(priority = 2, capacity = 8, shared = [layout])]
    fn handle_event(c: handle_event::Context, event: Event) {
        c.shared.layout.event(event)
    }

    #[task(priority = 2, shared = [layout, multi_device], local = [mouse_state])]
    fn tick_keyberon(mut c: tick_keyberon::Context) {
        match c.shared.layout.tick() {
            CustomEvent::Press(e) => {
                c.local.mouse_state.event = Some(*e);
                c.local.mouse_state.ticks_since_last_change = 0;
            }
            CustomEvent::Release(_) => {
                c.local.mouse_state.event = None;
                c.local.mouse_state.ticks_since_last_change = 0;
            }
            CustomEvent::NoEvent => {}
        }

        let t = c.local.mouse_state.ticks_since_last_change;
        let mut mouse_report = WheelMouseReport::default();
        match c.local.mouse_state.event {
            Some(MouseEvent::ScrollDown) => {
                mouse_report.vertical_wheel = -if t % 100 == 0 { 1 } else { 0 }
            }
            Some(MouseEvent::ScrollUp) => {
                mouse_report.vertical_wheel = if t % 100 == 0 { 1 } else { 0 }
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
            if mouse_report != WheelMouseReport::default() {
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
        binds = TIM3,
        priority = 1,
        local = [matrix, debouncer, timer],
    )]
    fn tick(c: tick::Context) {
        c.local.timer.wait().ok();

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
