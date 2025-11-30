use defmt::*;
use embassy_rp::usb::{Endpoint, In, Out};
use embassy_rp::{peripherals::USB, usb::Driver};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use embassy_usb::Builder;
use embassy_usb::class::web_usb::{Config as WebUsbConfig, State, Url, WebUsb};
use embassy_usb::driver::{Endpoint as EndpointTrait, EndpointIn, EndpointOut};
use embassy_usb::msos::{self, windows_version};
use static_cell::StaticCell;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage, SerializedDescriptor};

use crate::usb::MyRequestHandler;

// This is a randomly generated GUID to allow clients on Windows to find our device
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{da327103-02a8-4d8a-8329-be81cdb97cc7}"];

pub struct WebConfig {
    write_ep: Endpoint<'static, USB, In>,
    read_ep: Endpoint<'static, USB, Out>,
}

pub static CONFIG_CHANNEL: Channel<ThreadModeRawMutex, (), 64> = Channel::new();

impl WebConfig {
    pub fn new(builder: &mut Builder<'static, Driver<'static, USB>>) -> Self {
        static WEBUSB_CONFIG: StaticCell<WebUsbConfig> = StaticCell::new();
        let webusb_config = WEBUSB_CONFIG.init(WebUsbConfig {
            max_packet_size: 64,
            vendor_code: 1,
            // If defined, shows a landing page which the device manufacturer would like the user to visit in order to control their device.
            // Suggest the user to navigate to this URL when the device is connected.
            //landing_url: Some(Url::new("http://dpedal.com/config")),
            landing_url: None,
        });

        // Add the Microsoft OS Descriptor (MSOS/MOD) descriptor.
        // We tell Windows that this entire device is compatible with the "WINUSB" feature,
        // which causes it to use the built-in WinUSB driver automatically, which in turn
        // can be used by libusb/rusb software without needing a custom driver or INF file.
        // In principle you might want to call msos_feature() just on a specific function,
        // if your device also has other functions that still use standard class drivers.
        builder.msos_descriptor(windows_version::WIN8_1, 0);
        builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
        builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
            "DeviceInterfaceGUIDs",
            msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
        ));

        static STATE: StaticCell<State> = StaticCell::new();
        WebUsb::configure(builder, STATE.init(State::new()), webusb_config);

        let mut func = builder.function(0xff, 0x00, 0x00);
        let mut iface = func.interface();
        let mut alt = iface.alt_setting(0xff, 0x00, 0x00, None);

        let write_ep = alt.endpoint_bulk_in(None, 64);
        let read_ep = alt.endpoint_bulk_out(None, 64);

        Self { write_ep, read_ep }
    }

    pub async fn process(&mut self) {
        self.wait_connected().await;
        info!("Connected");
        self.echo().await;
    }

    // Wait until the device's endpoints are enabled.
    async fn wait_connected(&mut self) {
        self.read_ep.wait_enabled().await
    }

    // Echo data back to the host.
    async fn echo(&mut self) {
        let mut buf = [0; 64];
        loop {
            let n = self.read_ep.read(&mut buf).await.unwrap();
            let data = &buf[..n];
            info!("Data read: {:x}", data);
            self.write_ep.write(data).await.unwrap();
        }
    }
}
