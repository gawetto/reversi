use reversi_core::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
use web_sys::{MessageEvent, WebSocket};

fn view(e: MessageEvent, context: &CanvasRenderingContext2d) {
    let jsdata: String = e
        .data()
        .dyn_into::<js_sys::JsString>()
        .unwrap()
        .as_string()
        .unwrap();
    let data: ReversiData = serde_json::from_str(&jsdata).unwrap();
    let size = 60.0;
    let offset = size / 2.0;
    let pi2 = std::f64::consts::PI * 2.0;
    for i in 0..8 {
        for j in 0..8 {
            let x = size * i as f64;
            let y = size * j as f64;
            let p = Position::new(i, j).unwrap();
            if data.cursor.eq(&p) {
                context.set_fill_style(&"#999".into());
            } else {
                context.set_fill_style(&"#3c3".into());
            }
            context.begin_path();
            context.set_stroke_style(&"#000".into());
            context.rect(x, y, size, size);
            context.fill();
            context.stroke();
            context.begin_path();
            match data.field.get(Position::new(i, j).unwrap()) {
                Masu::Empty => {
                    match data.turn {
                        BorW::Black => {
                            context.set_fill_style(&"#000".into());
                        }
                        BorW::White => {
                            context.set_fill_style(&"#fff".into());
                        }
                    }
                    if check_putable(&data.field, p, data.turn) {
                        context
                            .arc(x + offset, y + offset, size * 0.05, 0.0, pi2)
                            .unwrap();
                        context.fill();
                    }
                }
                Masu::Putted(BorW::Black) => {
                    context.set_fill_style(&"#000".into());
                    context
                        .arc(x + offset, y + offset, size * 0.45, 0.0, pi2)
                        .unwrap();
                    context.fill();
                }

                Masu::Putted(BorW::White) => {
                    context.set_fill_style(&"#fff".into());
                    context
                        .arc(x + offset, y + offset, size * 0.45, 0.0, pi2)
                        .unwrap();
                    context.fill();
                }
            }
        }
    }
}

fn mouseinput(x: f64, y: f64, ws: &mut WebSocket) {
    let x = (x / 60.0).floor() as i32;
    let y = (y / 60.0).floor() as i32;
    let pos;
    match Position::new(x, y) {
        Ok(x) => pos = x,
        Err(_) => return,
    };
    ws.send_with_str(&serde_json::to_string(&pos).unwrap())
        .unwrap();
}

fn main() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(600);
    canvas.set_height(600);
    body.append_child(&canvas)?;
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;
    let ws = WebSocket::new("ws://127.0.0.1:9001")?;

    let mut ws1 = ws.clone();
    let mouse_callback = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MouseEvent| {
        let x = e.page_x() as f64 - canvas.get_bounding_client_rect().x();
        let y = e.page_y() as f64 - canvas.get_bounding_client_rect().y();
        mouseinput(x, y, &mut ws1);
    });
    body.add_event_listener_with_callback("mousedown", mouse_callback.as_ref().unchecked_ref())?;
    mouse_callback.forget();

    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        view(e, &context);
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    return Ok(());
}
