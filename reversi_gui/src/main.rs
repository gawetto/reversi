use reversi_core::*;
use reversi_wasm_common::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlCanvasElement;

fn input(event: web_sys::KeyboardEvent, data: &mut ReversiData) {
    match &*event.key() {
        "r" => {
            *data = ReversiData::new();
        }
        "ArrowUp" => {
            data.cursor = data.cursor.up().unwrap_or(data.cursor);
        }
        "ArrowDown" => {
            data.cursor = data.cursor.down().unwrap_or(data.cursor);
        }
        "ArrowLeft" => {
            data.cursor = data.cursor.left().unwrap_or(data.cursor);
        }
        "ArrowRight" => {
            data.cursor = data.cursor.right().unwrap_or(data.cursor);
        }
        "Enter" => {
            try_put(data);
        }
        _ => {}
    }
}

fn main() -> Result<(), JsValue> {
    let data = Rc::new(RefCell::new(ReversiData::new()));
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    body.append_child(&canvas)?;

    view(&data.borrow(), &canvas);
    let data1 = Rc::clone(&data);
    let canvas1 = canvas.clone();
    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        input(e, &mut data1.borrow_mut());
        view(&data1.borrow(), &canvas1);
    }) as Box<dyn FnMut(_)>);
    body.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let canvas2 = canvas.clone();
    let closure2 = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        if let Ok(x) = mouseinput(e) {
            data.borrow_mut().cursor = x;
            try_put(&mut data.borrow_mut());
        }
        view(&data.borrow(), &canvas2);
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousedown", closure2.as_ref().unchecked_ref())?;
    closure2.forget();

    return Ok(());
}
