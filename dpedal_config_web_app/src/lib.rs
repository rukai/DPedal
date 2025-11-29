use log::Level;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use web_sys::{Document, Element, HtmlInputElement};

#[wasm_bindgen]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Info).expect("could not initialize logger");

    wasm_bindgen_futures::spawn_local(run_async());
}

async fn run_async() {
    let document = web_sys::window().unwrap().document().unwrap();
    let table = document.get_element_by_id("input-output-table").unwrap();

    table
        .append_child(&create_row(&document, "Left Button", "PageUp"))
        .unwrap();
    table
        .append_child(&create_row(&document, "Right Button", "PageDown"))
        .unwrap();

    log::info!("Setup complete");
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
