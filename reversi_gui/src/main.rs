use reversi_core::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

fn view(data: &ReversiData, context: &CanvasRenderingContext2d) {
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

fn mouseinput(x: f64, y: f64, data: &mut ReversiData) {
    let x = (x / 60.0).floor() as i32;
    let y = (y / 60.0).floor() as i32;
    match Position::new(x, y) {
        Ok(x) => data.cursor = x,
        Err(_) => return,
    };
    if check_putable(&data.field, data.cursor, data.turn) {
        data.field.set(data.cursor, Masu::Putted(data.turn));
        auto_reverse(&mut data.field, data.cursor, data.turn);
        data.turn = get_another_color(data.turn);
        if !data.field.puttable(data.turn) {
            data.turn = get_another_color(data.turn);
        }
    }
}

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
            if check_putable(&data.field, data.cursor, data.turn) {
                data.field.set(data.cursor, Masu::Putted(data.turn));
                auto_reverse(&mut data.field, data.cursor, data.turn);
                data.turn = get_another_color(data.turn);
                if !data.field.puttable(data.turn) {
                    data.turn = get_another_color(data.turn);
                }
            }
        }
        _ => {}
    }
}

fn main() -> Result<(), JsValue> {
    let data = Rc::new(RefCell::new(ReversiData::new()));

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
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
    let context = Rc::new(RefCell::new(context));

    view(&data.borrow(), &context.borrow());

    let data1 = Rc::clone(&data);
    let context1 = Rc::clone(&context);
    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        input(e, &mut data1.borrow_mut());
        view(&data1.borrow(), &context1.borrow());
    }) as Box<dyn FnMut(_)>);
    body.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let canvas_x = canvas.get_bounding_client_rect().x();
    let canvas_y = canvas.get_bounding_client_rect().y();
    let closure2 = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let x = e.page_x() as f64 - canvas_x;
        let y = e.page_y() as f64 - canvas_y;
        mouseinput(x, y, &mut data.borrow_mut());
        view(&data.borrow(), &context.borrow());
    }) as Box<dyn FnMut(_)>);
    body.add_event_listener_with_callback("mousedown", closure2.as_ref().unchecked_ref())?;
    closure2.forget();

    return Ok(());
}
