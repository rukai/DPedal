#![no_main]
#![no_std]

// set the panic handler
use panic_halt as _;

use hal::gpio::{Input, Pin, PullUp};
use hal::prelude::*;
use hal::usb;
use hal::{stm32, timers};
use keyberon::debounce::Debouncer;
use keyberon::key_code::KbHidReport;
use keyberon::layout::{Event, Layers, Layout};
use keyberon::matrix::DirectPinMatrix;
use rtic::app;
use stm32f0xx_hal as hal;
use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass as _;
use usb_device::device::UsbDeviceState;

type UsbClass = keyberon::Class<'static, usb::UsbBusType, ()>;
type UsbDevice = usb_device::device::UsbDevice<'static, usb::UsbBusType>;

pub static LAYERS: Layers<4, 1, 1> = keyberon::layout::layout! {
    {
        [ Up Down Left Right ]
    }
};

#[app(device = crate::hal::pac, peripherals = true, dispatchers = [CEC_CAN])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice,
        usb_class: UsbClass,
        #[lock_free]
        layout: Layout<4, 1, 1>,
    }

    #[local]
    struct Local {
        matrix: DirectPinMatrix<Pin<Input<PullUp>>, 4, 1>,
        debouncer: Debouncer<[[bool; 4]; 1]>,
        timer: timers::Timer<stm32::TIM3>,
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

        let usb = usb::Peripheral {
            usb: c.device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };
        *c.local.bus = Some(usb::UsbBusType::new(usb));
        let usb_bus = c.local.bus.as_ref().unwrap();

        let usb_class = keyberon::new_class(usb_bus, ());
        let usb_dev = keyberon::new_device(usb_bus);

        let mut timer = timers::Timer::tim3(c.device.TIM3, 1.khz(), &mut rcc);
        timer.listen(timers::Event::TimeOut);

        let matrix = cortex_m::interrupt::free(move |cs| {
            DirectPinMatrix::new([[
                Some(gpioa.pa1.into_pull_up_input(cs).downgrade()),
                Some(gpioa.pa3.into_pull_up_input(cs).downgrade()),
                Some(gpioa.pa2.into_pull_up_input(cs).downgrade()),
                Some(gpioa.pa4.into_pull_up_input(cs).downgrade()),
            ]])
            .unwrap()
        });

        (
            Shared {
                usb_dev,
                usb_class,
                layout: Layout::new(&LAYERS),
            },
            Local {
                timer,
                debouncer: Debouncer::new([[false; 4]; 1], [[false; 4]; 1], 5),
                matrix,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = USB, priority = 3, shared = [usb_dev, usb_class])]
    fn usb_rx(c: usb_rx::Context) {
        (c.shared.usb_dev, c.shared.usb_class).lock(|usb_dev, usb_class| {
            if usb_dev.poll(&mut [usb_class]) {
                usb_class.poll();
            }
        });
    }

    #[task(priority = 2, capacity = 8, shared = [layout])]
    fn handle_event(c: handle_event::Context, event: Event) {
        c.shared.layout.event(event)
    }

    #[task(priority = 2, shared = [usb_dev, usb_class, layout])]
    fn tick_keyberon(mut c: tick_keyberon::Context) {
        c.shared.layout.tick();
        if c.shared.usb_dev.lock(|d| d.state()) != UsbDeviceState::Configured {
            return;
        }
        let report: KbHidReport = c.shared.layout.keycodes().collect();
        if !c
            .shared
            .usb_class
            .lock(|k| k.device_mut().set_keyboard_report(report.clone()))
        {
            return;
        }
        while let Ok(0) = c.shared.usb_class.lock(|k| k.write(report.as_bytes())) {}
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
