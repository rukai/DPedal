use log::Level;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use web_sys::{Document, Element, HtmlInputElement};
use webusb_web::Usb;
use webusb_web::UsbDeviceFilter;

#[wasm_bindgen]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Info).expect("could not initialize logger");

    let document = web_sys::window().unwrap().document().unwrap();
    set_button_on_click(
        &document,
        "open-device",
        Box::new(move || {
            wasm_bindgen_futures::spawn_local(open_device());
        }) as Box<dyn FnMut()>,
    );
}

async fn open_device() {
    let document = web_sys::window().unwrap().document().unwrap();
    set_error(&document, "");

    let usb = match Usb::new() {
        Ok(x) => x,
        Err(e) => {
            set_error(&document, e.msg());
            return;
        }
    };

    let filter = UsbDeviceFilter::new();
    // filter.vendor_id = Some(0x06);
    // filter.product_id = Some(0x11);
    let usb_device = match usb.request_device([filter]).await {
        Ok(x) => x,
        Err(e) => {
            set_error(&document, e.msg());
            return;
        }
    };

    let open_usb = usb_device.open().await.unwrap();

    log::info!("config {:#?}", usb_device.configuration());

    open_usb.claim_interface(1).await.unwrap();
    open_usb.transfer_out(1, "HIII".as_bytes()).await.unwrap();
    let result = open_usb.transfer_in(1, 4).await.unwrap();
    log::info!("response {:?}", result);
    let table = document.get_element_by_id("input-output-table").unwrap();

    table
        .append_child(&create_row(&document, "Left Button", "PageUp"))
        .unwrap();
    table
        .append_child(&create_row(&document, "Right Button", "PageDown"))
        .unwrap();

    log::info!("Setup complete");
}

fn set_error(document: &Document, error_message: &str) {
    let error = document.get_element_by_id("error").unwrap();
    let error = error.dyn_ref::<HtmlElement>().unwrap();
    error.set_inner_text(error_message);
}

fn create_row(document: &Document, input_value: &str, output_value: &str) -> Element {
    let tr = document.create_element("tr").unwrap();

    let td1 = document.create_element("td").unwrap();
    let td2 = document.create_element("td").unwrap();

    tr.append_child(&td1).unwrap();
    tr.append_child(&td2).unwrap();

    let input = document.create_element("p").unwrap();
    let input = input.dyn_ref::<HtmlElement>().unwrap();
    input.set_inner_text(input_value);
    td1.append_child(input).unwrap();

    let output = document.create_element("input").unwrap();
    let output = output.dyn_ref::<HtmlInputElement>().unwrap();
    output.set_value(output_value);
    td2.append_child(output).unwrap();

    tr
}

fn set_button_on_click(document: &Document, id: &str, closure: Box<dyn FnMut()>) {
    let closure = Closure::wrap(closure);
    document
        .get_element_by_id(id)
        .unwrap()
        .dyn_ref::<HtmlElement>()
        .unwrap()
        .set_onclick(Some(closure.as_ref().unchecked_ref()));

    // Need to forget closure otherwise the destructor destroys it ;-;
    closure.forget();
}
