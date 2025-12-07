use dpedal_config::web_config_protocol::{Request, Response};
use postcard::accumulator::CobsAccumulator;
use web_sys::Document;
use webusb_web::{OpenUsbDevice, Usb, UsbDeviceFilter};

pub struct Device {
    usb: OpenUsbDevice,
}

impl Device {
    pub async fn new(document: &Document) -> Result<Self, ()> {
        let usb = match Usb::new() {
            Ok(x) => x,
            Err(e) => {
                crate::set_error(document, e.msg());
                return Err(());
            }
        };

        let mut filter = UsbDeviceFilter::new();
        filter.vendor_id = Some(0xc0de);
        filter.product_id = Some(0xcafe);
        let usb_device = match usb.request_device([filter]).await {
            Ok(x) => x,
            Err(e) => {
                crate::set_error(document, e.msg());
                return Err(());
            }
        };

        let open_usb = usb_device.open().await.unwrap();

        log::info!("usb config {:#?}", usb_device.configuration());

        open_usb.claim_interface(1).await.unwrap();

        Ok(Device { usb: open_usb })
    }

    pub async fn send_request(&self, request: &Request) -> Response {
        let request_bytes = postcard::to_stdvec_cobs(request).unwrap();
        self.usb.transfer_out(1, &request_bytes).await.unwrap();

        let mut cobs_buf: CobsAccumulator<1024> = CobsAccumulator::new();
        loop {
            let out = self.usb.transfer_in(1, 64).await.unwrap();
            match cobs_buf.feed::<Response>(&out) {
                postcard::accumulator::FeedResult::Consumed => {}
                postcard::accumulator::FeedResult::OverFull(_items) => {
                    todo!()
                }
                postcard::accumulator::FeedResult::DeserError(_items) => {
                    todo!()
                }
                postcard::accumulator::FeedResult::Success { data, .. } => return data,
            }
        }
    }
}
