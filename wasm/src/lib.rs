use futures_util::future::join_all;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{AsyncIterator, Function, Reflect};
use web_sys::{Document, DragEvent, Element, Window};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "{ value: any, done: boolean }")]
    pub type IteratorResult;

    #[wasm_bindgen(method, getter, structural)]
    pub fn value(this: &IteratorResult) -> JsValue;

    #[wasm_bindgen(method, getter, structural)]
    pub fn done(this: &IteratorResult) -> bool;
}

trait JsValueExt {
    fn get_prop(&self, p: &str) -> Result<JsValue, String>;
    fn get_fn(&self, f: &str, msg: &str) -> Result<web_sys::js_sys::Function, String>;
    fn get_string(&self, p: &str, msg: &str) -> Result<String, String>;
}

impl JsValueExt for JsValue {
    fn get_prop(&self, p: &str) -> Result<JsValue, String> {
        Reflect::get(self, &JsValue::from_str(p))
            .map_err(|_| format!("Failed to get property '{p}'"))
    }

    fn get_fn(&self, f: &str, msg: &str) -> Result<Function, String> {
        self.get_prop(f)?
            .dyn_into::<Function>()
            .map_err(|_| format!("{msg} has no '{f}' function"))
    }

    fn get_string(&self, p: &str, msg: &str) -> Result<String, String> {
        match self.get_prop(p) {
            Ok(v) => match v.as_string() {
                Some(s) => Ok(s),
                None => Err(format!("{msg} property '{p}' is not string")),
            },
            Err(_) => Err(format!("{msg} has no '{p}' property")),
        }
    }
}

async fn dir_handle_handles(
    handle: &wasm_bindgen::JsValue,
) -> Result<Vec<wasm_bindgen::JsValue>, String> {
    let values_fn = handle.get_fn("values", "Handle")?;
    let iterator = values_fn
        .call0(&handle)
        .map_err(|_| "Failed calling values on handle")?
        .dyn_into::<AsyncIterator>()
        .map_err(|_| "Not AsyncIterator from values on handle")?;

    let mut handles = Vec::new();
    loop {
        // BUG: Chrome in Mac will for sure not include any directory child handles that have quotes in the filename
        match iterator.next() {
            Ok(promise) => {
                let js_value = JsFuture::from(promise)
                    .await
                    .map_err(|_| "Promise for Directory handle values() iterator next() failed")?;
                let res = js_value.unchecked_into::<IteratorResult>();

                if res.done() {
                    break;
                } else {
                    handles.push(res.value());
                }
            }
            Err(e) => Err(format!(
                "Error iterating Directory handle values(): {:?}",
                e
            ))?,
        }
    }

    Ok(handles)
}

fn append_to_output(document: &Document, msg: &str) -> () {
    let output_div = document.get_element_by_id("output").expect("No output");
    let text_node = document.create_text_node(msg);
    output_div
        .append_child(&text_node)
        .expect("Failed append output");
    // Add a newline for readability
    output_div
        .append_child(&document.create_text_node("\n"))
        .expect("Failed append newline");
}

async fn process_handles(document: &Document, handles: Vec<JsValue>) {
    // need stack to avoid async recursion -- JsValue is not Send, so cannot do that
    let mut stack = Vec::new();

    for (i, handle) in handles.into_iter().enumerate() {
        stack.push((handle, format!("Handles [{i}] /")));
    }

    while let Some((handle, prefix)) = stack.pop() {
        let name = handle.get_string("name", "Handle").unwrap();
        let kind = handle.get_string("kind", "Handle").unwrap();

        match kind.as_str() {
            // web_sys::FileSystemHandleKind::File
            "file" => {
                append_to_output(document, &format!("{prefix} file: {name}"));
            }
            // web_sys::FileSystemHandleKind::Directory
            "directory" => {
                append_to_output(document, &format!("{prefix} directory: {name}"));
                let dir_prefix = format!("{prefix}{name}/");
                match dir_handle_handles(&handle).await {
                    Ok(handles) => {
                        for handle in handles {
                            stack.push((handle, dir_prefix.clone()));
                        }
                    }
                    Err(msg) => {
                        append_to_output(
                            document,
                            &format!("Failed getting Directory handle values for {name}: {msg}"),
                        );
                    }
                }
            }
            _ => {
                append_to_output(document, &format!("{prefix} unknown kind: {kind}"));
            }
        }
    }
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
            // BUG: Chromium in Linux will sometimes have zero items for dropped files (quotes in filename or not)
            append_to_output(&document, &format!("Items: {:?}", items.length()));

            let items = (0..items.length())
                .map(|idx| items.get(idx))
                .filter(|opt| opt.as_ref().is_some_and(|item| item.kind() == "file"))
                .flatten()
                .collect::<Vec<_>>();

            if items[0].get_fn("getAsFileSystemHandle", "Item").is_ok() {
                wasm_bindgen_futures::spawn_local(async move {
                    // QUIRK/BUG?: You must get all the handles at once, otherwise Chrome silently drops any after the first
                    // i.e. items will have length > 1 but will only output processing of the first one, if we try to getAsFileSystemHandle in a for loop
                    let promises: Vec<_> = items
                        .iter()
                        .map(|item| item.get_as_file_system_handle())
                        .map(JsFuture::from)
                        .collect::<Vec<_>>();

                    let handles = join_all(promises)
                        .await
                        .into_iter()
                        .map(|res| res.ok())
                        .flatten()
                        .collect::<Vec<_>>();

                    process_handles(&document, handles).await;
                });
            } else {
                for (i, item) in items.into_iter().enumerate() {
                    match item.get_as_file() {
                        Ok(Some(file)) => {
                            append_to_output(
                                &document,
                                &format!("Items [{i}] filename: {}", file.name()),
                            );
                        }
                        Ok(None) => {
                            append_to_output(&document, &format!("Items [{i}] not a file"));
                        }
                        Err(_) => {
                            append_to_output(
                                &document,
                                &format!("Items [{i}] failed get_as_file()"),
                            );
                        }
                    }
                }
            }
        }
    }) as Box<dyn FnMut(DragEvent)>);

    dropzone.add_event_listener_with_callback("dragover", dragover_cb.as_ref().unchecked_ref())?;
    dropzone.add_event_listener_with_callback("drop", drop_cb.as_ref().unchecked_ref())?;

    dragover_cb.forget();
    drop_cb.forget();

    append_to_output(&document, "WASM Demo initialized, ready for drag and drop");
    Ok(())
}
