use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    console, Document, DragEvent, Element, HtmlElement, Window,
};

fn console_log(s: &str) {
    console::log_1(&JsValue::from_str(s));
}

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    let window: Window = web_sys::window().ok_or("No window")?;
    let document: Document = window.document().ok_or("No document")?;

    let dropzone: Element = document
        .get_element_by_id("dropzone")
        .ok_or("No dropzone")?;
    let dropzone: HtmlElement = dropzone.dyn_into()?;

    let dragover_cb = Closure::wrap(Box::new(move |event: DragEvent| {
        event.prevent_default();
        if let Some(dt) = event.data_transfer() {
            dt.set_drop_effect("copy");
        }
    }) as Box<dyn FnMut(DragEvent)>);

    let drop_cb = Closure::wrap(Box::new(move |event: DragEvent| {
        event.prevent_default();
        if let Some(data_transfer) = event.data_transfer() {
            let items = data_transfer.items();
            console_log(&format!("Items: {:?}", items.length()));

            wasm_bindgen_futures::spawn_local(async move {
                for i in 0..items.length() {
                    if let Some(item) = items.get(i) {
                        match item.get_as_file() {
                            Ok(Some(file)) => {
                                console_log(&format!("Items [{i}] filename: {}", file.name()));
                            }
                            Ok(None) => {
                                console_log(&format!("Items [{i}] not a file"));
                            }
                            Err(_) => {
                                console_log(&format!("Items [{i}] failed get_as_file()"));
                            }
                        }
                    } else {
                        console_log(&format!("Items [{i}] failed get()"));
                    }
                }
            });
        }
    }) as Box<dyn FnMut(DragEvent)>);

    dropzone
        .add_event_listener_with_callback("dragover", dragover_cb.as_ref().unchecked_ref())?;
    dropzone
        .add_event_listener_with_callback("drop", drop_cb.as_ref().unchecked_ref())?;

    dragover_cb.forget();
    drop_cb.forget();

    console_log("WASM initialized, ready for drag and drop");
    Ok(())
}
