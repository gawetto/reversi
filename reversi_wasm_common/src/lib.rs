use reversi_core::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

pub fn mouseinput(e: web_sys::MouseEvent) -> std::result::Result<Position, FieldOutError> {
    let x = (e.page_x() as f64 / 60.0).floor() as i32;
    let y = (e.page_y() as f64 / 60.0).floor() as i32;
    Position::new(x, y)
}

pub fn view(data: &ReversiData, canvas: &HtmlCanvasElement) {
    canvas.set_width(60 * 8);
    canvas.set_height(60 * 8);
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
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
