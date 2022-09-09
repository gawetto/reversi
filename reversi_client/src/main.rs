use reversi_core::*;
use reversi_wasm_common::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;
use web_sys::{MessageEvent, WebSocket};

fn main() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    body.append_child(&canvas)?;
    let ws = WebSocket::new("ws://127.0.0.1:9001")?;

    let ws1 = ws.clone();
    let mouse_callback = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MouseEvent| {
        if let Ok(x) = mouseinput(e) {
            ws1.send_with_str(&serde_json::to_string(&x).unwrap())
                .unwrap();
        }
    });
    canvas
        .add_event_listener_with_callback("mousedown", mouse_callback.as_ref().unchecked_ref())?;
    mouse_callback.forget();

    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        let jsdata: String = e
            .data()
            .dyn_into::<js_sys::JsString>()
            .unwrap()
            .as_string()
            .unwrap();
        let data: ReversiData = serde_json::from_str(&jsdata).unwrap();
        view(&data, &canvas);
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    return Ok(());
}
