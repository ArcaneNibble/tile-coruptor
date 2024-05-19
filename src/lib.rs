use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement};

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
    fn draw_px_rgb(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8);
    fn draw_px_pal(&mut self, x: usize, y: usize, i: u8);
}

struct CanvasPixelWriter<'a> {
    app: &'a TileCorruptorAppInst,
}
impl<'a> AbstractPixelTarget for CanvasPixelWriter<'a> {
    fn draw_px_rgb(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        self.app
            .ctx
            .set_fill_style(&format!("rgb({},{},{})", r, g, b).into());
        self.app.ctx.fill_rect(x as f64, y as f64, 1.0, 1.0);
    }

    fn draw_px_pal(&mut self, x: usize, y: usize, i: u8) {
        // TODO
        let color = i << 6;
        self.draw_px_rgb(x, y, color, color, color);
    }
}

struct InMemoryPixelWriter<'a> {
    w: usize,
    px: &'a mut [u8],
}
impl<'a> AbstractPixelTarget for InMemoryPixelWriter<'a> {
    fn draw_px_rgb(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        self.px[(y * self.w + x) * 3 + 0] = r;
        self.px[(y * self.w + x) * 3 + 1] = g;
        self.px[(y * self.w + x) * 3 + 2] = b;
    }

    fn draw_px_pal(&mut self, x: usize, y: usize, i: u8) {
        self.px[y * self.w + x] = i;
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct BuiltinGraphicsCodec {
    #[wasm_bindgen(readonly)]
    pub i18n_name: &'static str,
    is_tiled: bool,
    tile_codec: Option<&'static dyn TileCodec>,
}

pub const BUILTIN_GRAPHICS_CODECS: &[BuiltinGraphicsCodec] = &[
    BuiltinGraphicsCodec {
        i18n_name: "nes",
        is_tiled: true,
        tile_codec: Some(&NESGraphics::new()),
    },
    BuiltinGraphicsCodec {
        i18n_name: "gb",
        is_tiled: true,
        tile_codec: Some(&GBGraphics::new()),
    },
];

#[wasm_bindgen]
pub fn wasm_get_builtin_graphics_codecs() -> Vec<BuiltinGraphicsCodec> {
    BUILTIN_GRAPHICS_CODECS.to_vec()
}

#[wasm_bindgen]
pub struct TileCorruptorAppInst {
    data: Vec<u8>,
    data_bit_off: usize,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    px_scale: f64,
    is_tiled_mode: bool,
    tiles_width: usize,
    tiles_height: usize,
    tile_codec: &'static dyn TileCodec,
}
#[wasm_bindgen]
impl TileCorruptorAppInst {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_owned(),
            data_bit_off: 0,
            canvas: get_canvas(),
            ctx: get_canvas_ctx(),
            px_scale: 2.5,
            is_tiled_mode: true,
            tiles_width: 32,
            tiles_height: 32,
            tile_codec: BUILTIN_GRAPHICS_CODECS[0].tile_codec.unwrap(),
        }
    }

    pub fn change_codec(&mut self, new_codec_idx: usize) {
        let codec = BUILTIN_GRAPHICS_CODECS[new_codec_idx];
        if codec.is_tiled {
            self.is_tiled_mode = true;
            self.tile_codec = codec.tile_codec.unwrap();
            self.render();
        } else {
            todo!();
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

    pub fn update_status_bar(&self) {
        let file_offs_elem = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("file_offs")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();

        let file_offs = if self.data_bit_off % 8 == 0 {
            format!("0x{:08X}", self.data_bit_off / 8)
        } else {
            format!("0x{:08X}.b{}", self.data_bit_off / 8, self.data_bit_off % 8)
        };

        file_offs_elem.set_inner_text((*file_offs).into());

        let gfx_dims_elem = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("gfx_dims")
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();

        let gfx_dims = if self.is_tiled_mode {
            format!(
                "{} x {} tiles ({} x {} px)",
                self.tiles_width,
                self.tiles_height,
                self.tiles_width * self.tile_codec.tile_width(),
                self.tiles_height * self.tile_codec.tile_height()
            )
        } else {
            todo!()
        };

        gfx_dims_elem.set_inner_text((*gfx_dims).into());
    }

    pub fn render(&self) {
        self.ctx.clear_rect(
            0.0,
            0.0,
            (self.tiles_width * self.tile_codec.tile_width()) as f64,
            (self.tiles_height * self.tile_codec.tile_height()) as f64,
        );

        self.tile_codec.render(
            &mut CanvasPixelWriter { app: self },
            &self.data[(self.data_bit_off / 8)..],
            (self.data_bit_off % 8) as u8,
            self.tiles_width,
            self.tiles_height,
        );
    }

    pub fn export_png(&self) -> Vec<u8> {
        if self.is_tiled_mode {
            let w = self.tiles_width * self.tile_codec.tile_width();
            let h = self.tiles_height * self.tile_codec.tile_height();

            let mut ret = Vec::new();
            let mut pixels;

            let mut png_encoder = png::Encoder::new(&mut ret, w as u32, h as u32);
            png_encoder.set_depth(png::BitDepth::Eight);
            if self.tile_codec.num_palette_colors() > 0 {
                png_encoder.set_color(png::ColorType::Indexed);
                // TODO custom palettes
                let mut pal = Vec::with_capacity(3 * 256);
                for i in 0..256 {
                    pal.push((i << 6) as u8);
                    pal.push((i << 6) as u8);
                    pal.push((i << 6) as u8);
                }
                png_encoder.set_palette(pal);
                pixels = vec![0u8; w * h];
            } else {
                png_encoder.set_color(png::ColorType::Rgb);
                pixels = vec![0u8; w * h * 3];
            }

            self.tile_codec.render(
                &mut InMemoryPixelWriter { w, px: &mut pixels },
                &self.data[(self.data_bit_off / 8)..],
                (self.data_bit_off % 8) as u8,
                self.tiles_width,
                self.tiles_height,
            );

            let mut png_writer = png_encoder.write_header().unwrap();
            png_writer.write_image_data(&pixels).unwrap();
            png_writer.finish().unwrap();

            ret
        } else {
            todo!()
        }
    }

    pub fn width_minus(&mut self) {
        if self.is_tiled_mode && self.tiles_width > 1 {
            self.tiles_width -= 1;
            self.resize();
            self.render();
            self.update_status_bar();
        }
    }
    pub fn width_plus(&mut self) {
        if self.is_tiled_mode {
            self.tiles_width += 1;
            self.resize();
            self.render();
            self.update_status_bar();
        }
    }
    pub fn height_minus(&mut self) {
        if self.is_tiled_mode && self.tiles_height > 1 {
            self.tiles_height -= 1;
            self.resize();
            self.render();
            self.update_status_bar();
        }
    }
    pub fn height_plus(&mut self) {
        if self.is_tiled_mode {
            self.tiles_height += 1;
            self.resize();
            self.render();
            self.update_status_bar();
        }
    }

    pub fn tile_minus(&mut self) {
        if self.is_tiled_mode {
            let bits_per_tile = self.tile_codec.bits_per_tile();
            if self.data_bit_off >= bits_per_tile {
                self.data_bit_off -= bits_per_tile;
            } else {
                self.data_bit_off = 0;
            }
            self.render();
            self.update_status_bar();
        }
    }
    pub fn tile_plus(&mut self) {
        if self.is_tiled_mode {
            let new_off = self.data_bit_off + self.tile_codec.bits_per_tile();
            if new_off < self.data.len() * 8 {
                self.data_bit_off = new_off;
                self.render();
                self.update_status_bar();
            }
        }
    }

    pub fn row_minus(&mut self) {
        if self.is_tiled_mode {
            let bits_per_row = self.tile_codec.bits_per_tile() * self.tiles_width;
            if self.data_bit_off >= bits_per_row {
                self.data_bit_off -= bits_per_row;
            } else {
                self.data_bit_off = 0;
            }
            self.render();
            self.update_status_bar();
        } else {
            todo!()
        }
    }
    pub fn row_plus(&mut self) {
        if self.is_tiled_mode {
            if self.is_tiled_mode {
                let new_off =
                    self.data_bit_off + self.tile_codec.bits_per_tile() * self.tiles_width;
                if new_off < self.data.len() * 8 {
                    self.data_bit_off = new_off;
                    self.render();
                    self.update_status_bar();
                }
            }
        } else {
            todo!()
        }
    }

    pub fn byte_minus(&mut self) {
        if self.data_bit_off >= 8 {
            self.data_bit_off -= 8;
            self.render();
            self.update_status_bar();
        }
    }
    pub fn byte_plus(&mut self) {
        let new_off = self.data_bit_off + 8;
        if new_off < self.data.len() * 8 {
            self.data_bit_off = new_off;
            self.render();
            self.update_status_bar();
        }
    }
    pub fn bit_minus(&mut self) {
        if self.data_bit_off > 0 {
            self.data_bit_off -= 1;
            self.render();
            self.update_status_bar();
        }
    }
    pub fn bit_plus(&mut self) {
        let new_off = self.data_bit_off + 1;
        if new_off < self.data.len() * 8 {
            self.data_bit_off = new_off;
            self.render();
            self.update_status_bar();
        }
    }

    pub fn go_to_offset(&mut self, offs_str: &str) {
        let (offs, bit) = if let Some((offs, bit_str)) = offs_str.split_once(".b") {
            if let Ok(bit) = bit_str.parse::<u8>() {
                (offs, bit)
            } else {
                return;
            }
        } else {
            (offs_str, 0)
        };

        let offs = if let Some(offs) = offs.strip_prefix("0x") {
            offs
        } else {
            offs
        };

        let offs = if let Ok(offs) = usize::from_str_radix(offs, 16) {
            offs
        } else {
            return;
        };

        let new_off = offs * 8 + bit as usize;
        if new_off < self.data.len() * 8 {
            self.data_bit_off = new_off;
            self.render();
            self.update_status_bar();
        }
    }
}

#[wasm_bindgen(start)]
pub fn main_js() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}
