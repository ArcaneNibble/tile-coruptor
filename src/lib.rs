use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let unexpected_dom_error: JsValue =
        "something unexpected/impossible happened accessing DOM".into();

    let canvas = web_sys::window()
        .ok_or(unexpected_dom_error.clone())?
        .document()
        .ok_or(unexpected_dom_error.clone())?
        .get_element_by_id("canvas")
        .ok_or(unexpected_dom_error.clone())?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| unexpected_dom_error.clone())?;

    let ctx = canvas
        .get_context("2d")?
        .ok_or(unexpected_dom_error.clone())?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| unexpected_dom_error.clone())?;

    ctx.set_fill_style(&"#000".into());
    ctx.fill_rect(5.0, 10.0, 10.0, 20.0);

    Ok(())
}
