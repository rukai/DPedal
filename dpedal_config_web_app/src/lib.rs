use dpedal_config::ComputerInput;
use dpedal_config::Config;
use dpedal_config::Profile;
use dpedal_config::web_config_protocol::Response;
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

    log::info!("config {:#?}", usb_device.configuration());

    open_usb.claim_interface(1).await.unwrap();
    open_usb.transfer_out(1, "HIII".as_bytes()).await.unwrap();
    let mut result = vec![];
    loop {
        let out = open_usb.transfer_in(1, 64).await.unwrap();
        log::info!("out.len() {:?}", out.len());
        result.extend(&out);
        if out.len() != 64 {
            // TODO: I think we'll need a message length prefix to avoid hangs when the last packet is 64 bytes long.
            break;
        }
    }
    let response: Response = postcard::from_bytes(&result).unwrap();
    log::info!("response {:?}", response);
    let config: Config = match response {
        Response::GetConfig(config_bytes) => {
            // TODO: Is Align required in some circumstances?
            rkyv::from_bytes::<Config, rkyv::rancor::Error>(&config_bytes.unwrap_or_default())
                .unwrap()
        }
        Response::SetConfig => panic!("Unexpected response"),
    };
    log::info!("config {:#?}", config);

    gen_for_profile(&document, &config.profiles[0]);

    log::info!("Setup complete");
}

fn gen_for_profile(document: &Document, profile: &Profile) {
    let table = document.get_element_by_id("input-output-table").unwrap();
    table.set_inner_html("");

    let left_button = create_row(document, "Left Button", profile.button_left);
    let right_button = create_row(document, "Right Button", profile.button_right);
    let dpad_up = create_row(document, "Dpad Up", profile.dpad_up);
    let dpad_down = create_row(document, "Dpad Down", profile.dpad_down);
    let dpad_left = create_row(document, "Dpad Left", profile.dpad_left);
    let dpad_right = create_row(document, "Dpad Right", profile.dpad_right);

    table.append_child(&left_button).unwrap();
    table.append_child(&right_button).unwrap();
    table.append_child(&dpad_up).unwrap();
    table.append_child(&dpad_down).unwrap();
    table.append_child(&dpad_left).unwrap();
    table.append_child(&dpad_right).unwrap();
}

fn set_error(document: &Document, error_message: &str) {
    let error = document.get_element_by_id("error").unwrap();
    let error = error.dyn_ref::<HtmlElement>().unwrap();
    error.set_inner_text(error_message);
}

fn create_row(document: &Document, input_value: &str, computer_input: ComputerInput) -> Element {
    let output_value = &format!("{:?}", computer_input);

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
