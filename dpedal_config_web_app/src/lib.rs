use log::Level;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Warn).expect("could not initialize logger");

    wasm_bindgen_futures::spawn_local(run_async());
}

async fn run_async() {
    let _document = web_sys::window().unwrap().document().unwrap();
}
