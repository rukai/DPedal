use dpedal_config::Config;
use dpedal_config::Mapping;
use dpedal_config::Profile;
use dpedal_config::web_config_protocol::Request;
use dpedal_config::web_config_protocol::Response;
use log::Level;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use web_sys::{Document, Element, HtmlInputElement};
use webusb_web::OpenUsbDevice;
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

    let mut filter = UsbDeviceFilter::new();
    filter.vendor_id = Some(0xc0de);
    filter.product_id = Some(0xcafe);
    let usb_device = match usb.request_device([filter]).await {
        Ok(x) => x,
        Err(e) => {
            set_error(&document, e.msg());
            return;
        }
    };

    let open_usb = usb_device.open().await.unwrap();

    log::info!("usb config {:#?}", usb_device.configuration());

    open_usb.claim_interface(1).await.unwrap();

    let config = request_get_config(open_usb).await;

    gen_for_profile(&document, &config.profiles[0]);
    log::info!("device config {:#?}", config);

    log::info!("Setup complete");
}

async fn request_get_config(open_usb: OpenUsbDevice) -> Config {
    let request_bytes = postcard::to_stdvec_cobs(&Request::GetConfig).unwrap();
    open_usb.transfer_out(1, &request_bytes).await.unwrap();

    let mut result = open_usb.transfer_in(1, 64).await.unwrap();
    let size = u32::from_be_bytes(result[0..4].try_into().unwrap());

    while result.len() < size as usize {
        let out = open_usb.transfer_in(1, 64).await.unwrap();
        result.extend(&out);
    }
    let response: Response = postcard::from_bytes(&result[4..]).unwrap();
    let config: Config = match response {
        Response::GetConfig(config_bytes) => {
            // TODO: Is Align required in some circumstances?
            rkyv::from_bytes::<Config, rkyv::rancor::Error>(&config_bytes.unwrap_or_default())
                .unwrap()
        }
        Response::SetConfig => panic!("Unexpected dpedal response"),
        Response::ProtocolError => panic!("dpedal protocol error"),
    };
    config
}

fn gen_for_profile(document: &Document, profile: &Profile) {
    let table = document.get_element_by_id("input-output-table").unwrap();
    table.set_inner_html("<tr><th>Input</th><th>Output</th></tr>");

    for mapping in &profile.mappings {
        let row = create_row(document, mapping);
        table.append_child(&row).unwrap();
    }
}

fn set_error(document: &Document, error_message: &str) {
    let error = document.get_element_by_id("error").unwrap();
    let error = error.dyn_ref::<HtmlElement>().unwrap();
    error.set_inner_text(error_message);
}

fn create_row(document: &Document, mapping: &Mapping) -> Element {
    let input_value = mapping
        .input
        .iter()
        .map(|x| format!("{x:?}"))
        .collect::<Vec<String>>()
        .join("+");
    let output_value = mapping
        .output
        .iter()
        .map(|x| format!("{x:?}"))
        .collect::<Vec<String>>()
        .join("+");

    let tr = document.create_element("tr").unwrap();

    let td1 = document.create_element("td").unwrap();
    let td2 = document.create_element("td").unwrap();

    tr.append_child(&td1).unwrap();
    tr.append_child(&td2).unwrap();

    let input = document.create_element("p").unwrap();
    let input = input.dyn_ref::<HtmlElement>().unwrap();
    input.set_inner_text(&input_value);
    td1.append_child(input).unwrap();

    let output = document.create_element("input").unwrap();
    let output = output.dyn_ref::<HtmlInputElement>().unwrap();
    output.set_value(&output_value);
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
