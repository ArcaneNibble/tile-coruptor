use bitvec::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

fn get_canvas_ctx() -> CanvasRenderingContext2d {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    ctx
}

#[wasm_bindgen]
pub struct TileCorruptorAppInst {
    data: Vec<u8>,
}
#[wasm_bindgen]
impl TileCorruptorAppInst {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_owned(),
        }
    }

    pub fn foobar(&self) {
        let ctx = get_canvas_ctx();
        web_sys::console::log_1(&format!("fdsa {:?}", ctx).into());
        ctx.set_fill_style(&"#000".into());
        web_sys::console::log_2(&self.data.len().into(), &self.data[0].into());
        let bv = self.data.view_bits::<Msb0>();
        for y in 0..16 {
            for x in 0..16 {
                let bit_i = y * 16 + x;
                let b = bv[bit_i];
                if !b {
                    ctx.fill_rect(x as f64 * 8.0, y as f64 * 8.0, 8.0, 8.0);
                }
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main_js() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}
