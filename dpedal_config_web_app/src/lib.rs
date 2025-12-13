use arrayvec::ArrayVec;
use dpedal_config::ComputerInput;
use dpedal_config::Config;
use dpedal_config::DpedalInput;
use dpedal_config::Mapping;
use dpedal_config::Profile;
use dpedal_config::web_config_protocol::Request;
use dpedal_config::web_config_protocol::Response;
use log::Level;
use rkyv::rancor::Error;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCollection;
use web_sys::HtmlElement;
use web_sys::{Document, Element, HtmlInputElement};

use crate::device::Device;

mod device;

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

    let Ok(device) = Device::new(&document).await else {
        return;
    };

    let config = request_get_config(&device).await;

    let config_div = document
        .get_element_by_id("mapping-section")
        .unwrap_or_else(|| {
            let config_div = document.create_element("div").unwrap();
            config_div.set_id("mapping-section");
            config_div
        });
    config_div.set_inner_html(
        r#"
            <table id="input-output-table">
                <tr>
                    <th>Input</th>
                    <th>Output</th>
                </tr>
            </table>
            <button id="flash">Save</button>
            "#,
    );
    let config_div = config_div.dyn_ref::<HtmlElement>().unwrap();

    let app_div = document.get_element_by_id("config-app").unwrap();
    app_div.append_child(config_div).unwrap();

    gen_for_profile(&document, &config.profiles[0]);
    log::info!("device config {:#?}", config);

    let device = Rc::new(device);
    set_button_on_click(
        &document,
        "flash",
        Box::new(move || {
            let device = device.clone();
            wasm_bindgen_futures::spawn_local(write_config_task(device));
        }) as Box<dyn FnMut()>,
    );

    log::info!("Setup complete");
}

// TODO: wrap this in a mutex to prevent the user breaking things by clicking flash twice in quick succession.
async fn write_config_task(device: Rc<Device>) {
    let document = web_sys::window().unwrap().document().unwrap();
    if let Err(err) = write_config(&document, device).await {
        set_error(&document, &err);
    }
}

async fn write_config(document: &Document, device: Rc<Device>) -> Result<(), String> {
    let table = document.get_element_by_id("input-output-table").unwrap();

    let mut mappings = ArrayVec::new();

    // Iterate over rows, skipping the header
    for row in ElementChildIterator::new(&table).skip(1) {
        let mut cells = ElementChildIterator::new(&row);
        let input_cell = ElementChildIterator::new(&cells.next().unwrap())
            .next()
            .unwrap();
        let output_cell = ElementChildIterator::new(&cells.next().unwrap())
            .next()
            .unwrap();

        let input = input_cell.inner_html();
        let output = output_cell.dyn_ref::<HtmlInputElement>().unwrap().value();
        mappings.push(Mapping {
            input: ArrayVec::from_iter([DpedalInput::from_string(&input)
                .ok_or_else(|| format!("{input} is not a valid input"))?]),
            output: ArrayVec::from_iter([ComputerInput::from_string(&output)
                .ok_or_else(|| format!("{output} is not a valid output"))?]),
        });
    }

    let config = Config {
        version: 0,
        color: 0,
        profiles: ArrayVec::from_iter([Profile { mappings }]),
        pin_remappings: ArrayVec::new(),
    };

    let config_bytes =
        ArrayVec::from_iter(rkyv::to_bytes::<Error>(&config).unwrap().iter().cloned());
    device.send_request(&Request::SetConfig(config_bytes)).await;
    log::info!("config written {:#?}", config);

    Ok(())
}

async fn request_get_config(device: &Device) -> Config {
    let response = device.send_request(&Request::GetConfig).await;
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

    for mapping in &profile.mappings {
        let row = create_row(document, mapping);
        table.append_child(&row).unwrap();
    }
}

pub fn set_error(document: &Document, error_message: &str) {
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

pub struct ElementChildIterator {
    collection: HtmlCollection,
    index: u32,
    length: u32,
}

impl ElementChildIterator {
    /// Create a new iterator over the children of an element
    pub fn new(element: &Element) -> Self {
        let collection = element.children();
        let length = collection.length();

        Self {
            collection,
            index: 0,
            length,
        }
    }
}

impl Iterator for ElementChildIterator {
    type Item = Element;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let element = self.collection.item(self.index);
            self.index += 1;
            element
        } else {
            None
        }
    }
}
