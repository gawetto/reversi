use reversi_core::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

fn view(field: &Field, cursor: Position, turn: BorW, context: &CanvasRenderingContext2d) {
    let size = 60.0;
    let offset = size / 2.0;
    let pi2 = std::f64::consts::PI * 2.0;
    for i in 0..8 {
        for j in 0..8 {
            let x = size * i as f64;
            let y = size * j as f64;
            let p = Position::new(i, j).unwrap();
            if cursor.eq(&p) {
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
            match field.get(Position::new(i, j).unwrap()) {
                Masu::Empty => {
                    match turn {
                        BorW::Black => {
                            context.set_fill_style(&"#000".into());
                        }
                        BorW::White => {
                            context.set_fill_style(&"#fff".into());
                        }
                    }
                    if check_putable(field, p, turn) {
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

fn mouseinput(x: f64, y: f64, field: &mut Field, cursor: &mut Position, turn: &mut BorW) {
    let x = (x / 60.0).floor() as i32;
    let y = (y / 60.0).floor() as i32;
    match Position::new(x, y) {
        Ok(x) => *cursor = x,
        Err(_) => return,
    };
    if check_putable(&field, *cursor, *turn) {
        field.set(*cursor, Masu::Putted(*turn));
        auto_reverse(field, *cursor, *turn);
        *turn = get_another_color(*turn);
        if !field.puttable(*turn) {
            *turn = get_another_color(*turn);
        }
    }
}

fn input(event: web_sys::KeyboardEvent, field: &mut Field, cursor: &mut Position, turn: &mut BorW) {
    match &*event.key() {
        "r" => {
            (*field, *cursor, *turn) = create_initial_data();
        }
        "ArrowUp" => {
            *cursor = cursor.up().unwrap_or(*cursor);
        }
        "ArrowDown" => {
            *cursor = cursor.down().unwrap_or(*cursor);
        }
        "ArrowLeft" => {
            *cursor = cursor.left().unwrap_or(*cursor);
        }
        "ArrowRight" => {
            *cursor = cursor.right().unwrap_or(*cursor);
        }
        "Enter" => {
            if check_putable(&field, *cursor, *turn) {
                field.set(*cursor, Masu::Putted(*turn));
                auto_reverse(field, *cursor, *turn);
                *turn = get_another_color(*turn);
                if !field.puttable(*turn) {
                    *turn = get_another_color(*turn);
                }
            }
        }
        _ => {}
    }
}

fn main() -> Result<(), JsValue> {
    let (field, cursor, turn) = create_initial_data();
    let field = Rc::new(RefCell::new(field));
    let cursor = Rc::new(RefCell::new(cursor));
    let turn = Rc::new(RefCell::new(turn));

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

    view(
        &field.borrow(),
        *cursor.borrow(),
        *turn.borrow(),
        &context.borrow(),
    );

    let field1 = Rc::clone(&field);
    let cursor1 = Rc::clone(&cursor);
    let turn1 = Rc::clone(&turn);
    let context1 = Rc::clone(&context);
    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        input(
            e,
            &mut field1.borrow_mut(),
            &mut cursor1.borrow_mut(),
            &mut turn1.borrow_mut(),
        );
        view(
            &field1.borrow(),
            *cursor1.borrow(),
            *turn1.borrow(),
            &context1.borrow(),
        );
    }) as Box<dyn FnMut(_)>);
    body.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let canvas_x = canvas.get_bounding_client_rect().x();
    let canvas_y = canvas.get_bounding_client_rect().y();
    let closure2 = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let x = e.page_x() as f64 - canvas_x;
        let y = e.page_y() as f64 - canvas_y;
        mouseinput(
            x,
            y,
            &mut field.borrow_mut(),
            &mut cursor.borrow_mut(),
            &mut turn.borrow_mut(),
        );
        view(
            &field.borrow(),
            *cursor.borrow(),
            *turn.borrow(),
            &context.borrow(),
        );
    }) as Box<dyn FnMut(_)>);
    body.add_event_listener_with_callback("mousedown", closure2.as_ref().unchecked_ref())?;
    closure2.forget();

    return Ok(());
}
