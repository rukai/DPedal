use arrayvec::ArrayVec;
use dpedal_config::ComputerInput;
use dpedal_config::Config;
use dpedal_config::DPedalControl;
use dpedal_config::DpedalInput;
use dpedal_config::KeyboardInput;
use dpedal_config::Mapping;
use dpedal_config::MouseInput;
use dpedal_config::Profile;
use dpedal_config::web_config_protocol::Request;
use dpedal_config::web_config_protocol::Response;
use element_iterator::ElementChildIterator;
use log::Level;
use rkyv::rancor::Error;
use std::rc::Rc;
use strum::IntoEnumIterator;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;
use web_sys::HtmlSelectElement;
use web_sys::{Document, Element};

use crate::device::Device;

mod device;
mod element_iterator;

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

    let config = match request_get_config(&device).await {
        Ok(x) => x,
        Err(err) => {
            log::error!("Failed to request config from device {err}");
            web_sys::window()
                .unwrap()
                .alert_with_message("Config on the device is not present, outdated, or corrupt. Default config will be restored.")
                .unwrap();
            Default::default()
        }
    };

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

    if let Some(profile) = config.profiles.first() {
        gen_for_profile(&document, profile);
    }
    log::info!("device config {:#?}", config);

    let device = Rc::new(device);
    set_button_on_click(
        &document,
        "flash",
        Box::new(move || {
            let device = device.clone();
            let config = config.clone();
            wasm_bindgen_futures::spawn_local(write_config_task(device, config));
        }) as Box<dyn FnMut()>,
    );

    log::info!("Setup complete");
}

// TODO: wrap this in a mutex to prevent the user breaking things by clicking flash twice in quick succession.
async fn write_config_task(device: Rc<Device>, config: Config) {
    let document = web_sys::window().unwrap().document().unwrap();
    if let Err(err) = write_config(&document, device, config).await {
        set_error(&document, &err);
    }
}

async fn write_config(
    document: &Document,
    device: Rc<Device>,
    mut config: Config,
) -> Result<(), String> {
    let table = document.get_element_by_id("input-output-table").unwrap();

    let mut mappings = ArrayVec::new();

    // Iterate over rows, skipping the header
    for row in ElementChildIterator::new(&table).skip(1) {
        let mut cells = ElementChildIterator::new(&row);
        let input_cell = ElementChildIterator::new(&cells.next().unwrap())
            .next()
            .unwrap();
        let mut output_cell = ElementChildIterator::new(&cells.next().unwrap());
        let mut output = ArrayVec::new();
        while let Some(result) = parse_output(&mut output_cell) {
            output.push(result);
        }

        let input = input_cell.inner_html();
        let input = ArrayVec::from_iter([DpedalInput::from_string(&input)
            .ok_or_else(|| format!("{input} is not a valid input"))?]);
        mappings.push(Mapping { input, output });
    }

    config.profiles = ArrayVec::from_iter([Profile { mappings }]);

    let config_bytes =
        ArrayVec::from_iter(rkyv::to_bytes::<Error>(&config).unwrap().iter().cloned());
    device
        .send_request(&Request::SetConfig(config_bytes))
        .await?;
    log::info!("config written {:#?}", config);

    Ok(())
}

fn parse_output(output_cell: &mut ElementChildIterator) -> Option<ComputerInput> {
    let ty_value = output_cell
        .next()?
        .dyn_ref::<HtmlSelectElement>()
        .unwrap()
        .value();

    let sub_ty_value = output_cell
        .next()?
        .dyn_ref::<HtmlSelectElement>()
        .unwrap()
        .value();

    match ty_value.as_str() {
        "mouse" => MouseInput::from_string(&sub_ty_value).map(ComputerInput::Mouse),
        "keyboard" => KeyboardInput::from_string(&sub_ty_value).map(ComputerInput::Keyboard),
        "control" => DPedalControl::from_string(&sub_ty_value).map(ComputerInput::Control),
        _ => None,
    }
}

async fn request_get_config(device: &Device) -> Result<Config, String> {
    let response = device.send_request(&Request::GetConfig).await?;
    match response {
        Response::GetConfig(config_bytes) => {
            // TODO: Is Align required in some circumstances?
            rkyv::from_bytes::<Config, rkyv::rancor::Error>(
                &config_bytes.map_err(|e| format!("{e:?}"))?,
            )
            .map_err(|e| format!("{e:?}"))
        }
        Response::SetConfig => panic!("Unexpected dpedal response"),
        Response::ProtocolError => panic!("dpedal protocol error"),
    }
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
    let tr = document.create_element("tr").unwrap();

    tr.append_child(&create_row_input(document, &mapping.input))
        .unwrap();
    tr.append_child(&create_row_output(document, &mapping.output))
        .unwrap();

    tr
}

fn create_row_input<const CAP: usize>(
    document: &Document,
    inputs: &ArrayVec<DpedalInput, CAP>,
) -> Element {
    let input_value = inputs
        .iter()
        .map(|x| format!("{x:?}"))
        .collect::<Vec<String>>()
        .join("+");
    let td1 = document.create_element("td").unwrap();

    let input = document.create_element("p").unwrap();
    let input = input.dyn_ref::<HtmlElement>().unwrap();
    input.set_inner_text(&input_value);
    td1.append_child(input).unwrap();
    td1
}

fn create_row_output<const CAP: usize>(
    document: &Document,
    outputs: &ArrayVec<ComputerInput, CAP>,
) -> Element {
    let td2 = document.create_element("td").unwrap();
    for output in outputs {
        let select_type = document.create_element("select").unwrap();
        let select_type = select_type.dyn_ref::<HtmlSelectElement>().unwrap();
        select_type.set_inner_html(
            "
<option value=\"keyboard\">⌨️</option>
<option value=\"mouse\">🖱️</option>
<option value=\"control\">⚙️</option>
",
        );
        select_type.style().set_css_text("font-size:2em;");
        select_type.set_value(match output {
            ComputerInput::None => {
                continue;
            }
            ComputerInput::Mouse(_) => "mouse",
            ComputerInput::Keyboard(_) => "keyboard",
            ComputerInput::Control(_) => "control",
        });
        td2.append_child(select_type).unwrap();

        let select_subtype = document.create_element("select").unwrap();
        let select_subtype = select_subtype.dyn_ref::<HtmlSelectElement>().unwrap();
        select_subtype.style().set_css_text("font-size:2em;");
        td2.append_child(select_subtype).unwrap();

        let select_type_clone = select_type.clone();
        let select_subtype = select_subtype.clone();
        setup_select_subtype(&select_subtype, output);
        set_onchange(
            select_type,
            Box::new(move || {
                let output = match select_type_clone.value().as_str() {
                    "mouse" => ComputerInput::Mouse(Default::default()),
                    "keyboard" => ComputerInput::Keyboard(Default::default()),
                    "control" => ComputerInput::Control(Default::default()),
                    _ => ComputerInput::None,
                };
                setup_select_subtype(&select_subtype, &output);
            }) as Box<dyn FnMut()>,
        );
    }

    td2
}

fn setup_select_subtype(select_subtype: &HtmlSelectElement, output: &ComputerInput) {
    match output {
        ComputerInput::None => {}
        ComputerInput::Mouse(mouse_input) => {
            let mut options = String::new();
            for variant in MouseInput::iter() {
                options.push_str(&format!(
                    "<option value=\"{variant:?}\">{variant:?}</option>"
                ));
            }
            select_subtype.set_inner_html(&options);
            select_subtype.set_value(&format!("{mouse_input:?}"));
        }
        ComputerInput::Keyboard(keyboard_input) => {
            let mut options = String::new();
            for variant in KeyboardInput::iter() {
                options.push_str(&format!(
                    "<option value=\"{variant:?}\">{variant:?}</option>"
                ));
            }
            select_subtype.set_inner_html(&options);
            select_subtype.set_value(&format!("{keyboard_input:?}"));
        }
        ComputerInput::Control(control) => {
            let mut options = String::new();
            for variant in DPedalControl::iter() {
                options.push_str(&format!(
                    "<option value=\"{variant:?}\">{variant:?}</option>"
                ));
            }
            select_subtype.set_inner_html(&options);
            select_subtype.set_value(&format!("{control:?}"));
        }
    }
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

fn set_onchange(select: &HtmlElement, closure: Box<dyn FnMut()>) {
    let closure = Closure::wrap(closure);
    select.set_onchange(Some(closure.as_ref().unchecked_ref()));

    // Need to forget closure otherwise the destructor destroys it ;-;
    closure.forget();
}
