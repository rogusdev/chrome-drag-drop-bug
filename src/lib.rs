use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Document, DragEvent, Element, Window,
};

fn append_to_output(document: &Document, msg: &str) -> () {
    let output_div = document.get_element_by_id("output").expect("No output");
    let text_node = document.create_text_node(msg);
    output_div.append_child(&text_node).expect("Failed append output");
    // Add a newline for readability
    output_div.append_child(&document.create_text_node("\n")).expect("Failed append newline");
}

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    let window: Window = web_sys::window().ok_or("No window")?;
    let document: Document = window.document().ok_or("No document")?;

    let dropzone: Element = document
        .get_element_by_id("dropzone")
        .ok_or("No dropzone")?;

    let dragover_cb = Closure::wrap(Box::new(move |event: DragEvent| {
        event.prevent_default();
        if let Some(data_transfer) = event.data_transfer() {
            data_transfer.set_drop_effect("copy");
        }
    }) as Box<dyn FnMut(DragEvent)>);

    let drop_cb = Closure::wrap(Box::new(move |event: DragEvent| {
        event.prevent_default();

        let document = web_sys::window()
            .and_then(|w| w.document())
            .expect("No document");

        if let Some(data_transfer) = event.data_transfer() {
            let items = data_transfer.items();
            append_to_output(&document, &format!("Items: {:?}", items.length()));

            wasm_bindgen_futures::spawn_local(async move {
                for i in 0..items.length() {
                    if let Some(item) = items.get(i) {
                        match item.get_as_file() {
                            Ok(Some(file)) => {
                                append_to_output(&document, &format!("Items [{i}] filename: {}", file.name()));
                            }
                            Ok(None) => {
                                append_to_output(&document, &format!("Items [{i}] not a file"));
                            }
                            Err(_) => {
                                append_to_output(&document, &format!("Items [{i}] failed get_as_file()"));
                            }
                        }
                    } else {
                        append_to_output(&document, &format!("Items [{i}] failed get()"));
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

    append_to_output(&document, "WASM initialized, ready for drag and drop");
    Ok(())
}
