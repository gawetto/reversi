use reversi_message::*;
use reversi_wasm_common::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlButtonElement, HtmlCanvasElement, HtmlDivElement};
use web_sys::{MessageEvent, WebSocket};

fn create_button(ws: &WebSocket) {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let buttondiv = document
        .create_element("div")
        .unwrap()
        .dyn_into::<HtmlDivElement>()
        .unwrap();
    let button = document
        .create_element("button")
        .unwrap()
        .dyn_into::<HtmlButtonElement>()
        .unwrap();
    buttondiv.append_child(&button).unwrap();
    body.append_child(&buttondiv).unwrap();
    button.set_inner_html("create new game");
    let ws1 = ws.clone();
    let button_click_callback = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
        ws1.send_with_str(&serde_json::to_string(&ClientMessage::CreateGame {}).unwrap())
            .unwrap();
    });
    button
        .add_event_listener_with_callback("click", button_click_callback.as_ref().unchecked_ref())
        .unwrap();
    button_click_callback.forget();

    let reset_button = document
        .create_element("button")
        .unwrap()
        .dyn_into::<HtmlButtonElement>()
        .unwrap();
    buttondiv.append_child(&reset_button).unwrap();
    reset_button.set_inner_html("reset");
    let ws2 = ws.clone();
    let reset_button_click_callback = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
        ws2.send_with_str(&serde_json::to_string(&ClientMessage::Reset {}).unwrap())
            .unwrap();
    });
    reset_button
        .add_event_listener_with_callback(
            "click",
            reset_button_click_callback.as_ref().unchecked_ref(),
        )
        .unwrap();
    reset_button_click_callback.forget();
}

fn game_list_view(list: &Vec<GameSummary>, listener: &::js_sys::Function) {
    let document = web_sys::window().unwrap().document().unwrap();
    let listdiv = match document.get_element_by_id("gamelist") {
        None => {
            let x = document.create_element("div").unwrap();
            x.set_id("gamelist");
            document.body().unwrap().append_child(&x).unwrap();
            x
        }
        Some(x) => {
            while let Some(y) = x.first_child() {
                x.remove_child(&y).unwrap();
            }
            x
        }
    };
    list.iter().enumerate().for_each(|(_, gs)| {
        let div = document
            .create_element("div")
            .unwrap()
            .dyn_into::<HtmlDivElement>()
            .unwrap();
        listdiv.append_child(&div).unwrap();
        div.set_attribute("game_id", &format!("{}", gs.id.0))
            .unwrap();
        let mut inner = format!("id: {} , number : {}", gs.id.0, gs.members);
        if gs.your {
            inner = inner + "★";
        }
        div.set_inner_text(&inner);
        div.add_event_listener_with_callback("click", listener)
            .unwrap();
    });
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(0);
    canvas.set_height(0);
    body.append_child(&canvas)?;
    let ws = WebSocket::new("ws://127.0.0.1:9001")?;
    let ws_clone = ws.clone();
    create_button(&ws_clone);

    let ws_clone = ws.clone();
    let mouse_callback = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MouseEvent| {
        if let Ok(x) = mouseinput(e) {
            ws_clone
                .send_with_str(&serde_json::to_string(&ClientMessage::Put(x)).unwrap())
                .unwrap();
        }
    });
    canvas
        .add_event_listener_with_callback("mousedown", mouse_callback.as_ref().unchecked_ref())?;
    mouse_callback.forget();

    let ws_clone = ws.clone();
    let select_game_callback = Closure::<dyn FnMut(_)>::new(move |e: web_sys::MouseEvent| {
        let tmp = e.target();
        let target = tmp.unwrap();
        let target1 = target.dyn_into::<web_sys::HtmlElement>().unwrap();
        let html = target1.get_attribute("game_id").unwrap();
        let id: u32 = html.parse().unwrap();
        ws_clone
            .send_with_str(&serde_json::to_string(&ClientMessage::SelectGame(GameID(id))).unwrap())
            .unwrap();
    });
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        let jsdata: String = e
            .data()
            .dyn_into::<js_sys::JsString>()
            .unwrap()
            .as_string()
            .unwrap();
        let server_message: ServerMessage;
        match serde_json::from_str(&jsdata) {
            Ok(x) => server_message = x,
            Err(_) => return,
        }
        match server_message {
            ServerMessage::GameList(x) => {
                game_list_view(&x, select_game_callback.as_ref().unchecked_ref());
            }
            ServerMessage::View(x) => {
                view(&x, &canvas);
            }
        }
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();

    let ws_clone = ws.clone();
    let open_callback = Closure::<dyn FnMut(_)>::new(move |_: web_sys::Event| {
        ws_clone
            .send_with_str(&serde_json::to_string(&ClientMessage::SessionList {}).unwrap())
            .unwrap();
    });
    ws.set_onopen(Some(open_callback.as_ref().unchecked_ref()));
    open_callback.forget();

    return Ok(());
}
