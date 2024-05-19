use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub mod tile_codec;

use crate::tile_codec::*;

fn get_canvas() -> HtmlCanvasElement {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap()
}

fn get_canvas_ctx() -> CanvasRenderingContext2d {
    get_canvas()
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap()
}

pub trait AbstractPixelTarget {
    fn draw_px_rgb(&self, x: usize, y: usize, r: u8, g: u8, b: u8);
    fn draw_px_pal(&self, x: usize, y: usize, i: u8);
}

#[wasm_bindgen]
pub struct TileCorruptorAppInst {
    data: Vec<u8>,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    px_scale: f64,
    is_tiled_mode: bool,
    tiles_width: usize,
    tiles_height: usize,
    tile_codec: Box<dyn TileCodec>,
}
#[wasm_bindgen]
impl TileCorruptorAppInst {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_owned(),
            canvas: get_canvas(),
            ctx: get_canvas_ctx(),
            px_scale: 2.5,
            is_tiled_mode: true,
            tiles_width: 32,
            tiles_height: 32,
            tile_codec: Box::new(NESGraphics::default()),
        }
    }

    pub fn resize(&self) {
        if self.is_tiled_mode {
            self.canvas
                .set_width((self.tiles_width * self.tile_codec.tile_width()) as u32);
            self.canvas
                .set_height((self.tiles_height * self.tile_codec.tile_height()) as u32);
            self.canvas
                .style()
                .set_property(
                    "width",
                    &format!(
                        "{}px",
                        (self.tiles_width * self.tile_codec.tile_width()) as f64 * self.px_scale
                    ),
                )
                .unwrap();
            self.canvas
                .style()
                .set_property(
                    "height",
                    &format!(
                        "{}px",
                        (self.tiles_height * self.tile_codec.tile_height()) as f64 * self.px_scale
                    ),
                )
                .unwrap();
        } else {
            todo!()
        }
    }

    pub fn render(&self) {
        self.tile_codec.render(
            self,
            &self.data[0x8010..],
            self.tiles_width,
            self.tiles_height,
        );
    }
}

impl AbstractPixelTarget for TileCorruptorAppInst {
    fn draw_px_rgb(&self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        self.ctx
            .set_fill_style(&format!("rgb({},{},{})", r, g, b).into());
        self.ctx.fill_rect(x as f64, y as f64, 1.0, 1.0);
    }

    fn draw_px_pal(&self, x: usize, y: usize, i: u8) {
        // TODO
        let color = i << 6;
        self.draw_px_rgb(x, y, color, color, color);
    }
}

#[wasm_bindgen(start)]
pub fn main_js() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}
