use reversi_core::*;
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
            let x = size * j as f64;
            let y = size * i as f64;
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
    let (mut field, mut cursor, mut turn) = create_initial_data();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(600);
    canvas.set_height(600);
    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;
    body.append_child(&canvas)?;
    view(&field, cursor, turn, &context);
    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        input(e, &mut field, &mut cursor, &mut turn);
        view(&field, cursor, turn, &context);
    }) as Box<dyn FnMut(_)>);
    body.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();

    return Ok(());
}
