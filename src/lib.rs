use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub mod tile_codec;

use crate::tile_codec::*;

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
        let codec_nes = NESGraphics::default();
        let codec: &dyn TileCodec = &codec_nes;
        codec.xxx_render(&self.data[0x8010..], 8, 8);
    }
}

#[wasm_bindgen(start)]
pub fn main_js() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}
