#[cfg(test)]
mod tests {
    use reversi_core::ReversiData;
    use reversi_wasm_common::view;
    use wasm_bindgen_test::*;
    use web_sys::HtmlCanvasElement;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::{JsCast, JsValue};

    #[wasm_bindgen_test]
    fn input_test() {
        assert_eq!(1, 1);
    }
    #[wasm_bindgen_test]
    fn view_test() {
        let data = ReversiData::new();
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();
        view(&data, &canvas);
    }
}
