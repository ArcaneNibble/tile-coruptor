use std::borrow::Cow;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement};

pub mod linear_codec;
pub mod tile_codec;

pub mod palette;

use crate::linear_codec::*;
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
        let color = self.app.pal[i as usize];
        self.draw_px_rgb(x, y, color.0, color.1, color.2);
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
    #[wasm_bindgen(readonly)]
    pub is_tiled: bool,
    tile_codec: Option<&'static dyn TileCodec>,
    lin_codec: Option<&'static dyn LinearCodec>,
}

pub const BUILTIN_GRAPHICS_CODECS: &[BuiltinGraphicsCodec] = &[
    BuiltinGraphicsCodec {
        i18n_name: "nes",
        is_tiled: true,
        tile_codec: Some(&NESGraphics::new()),
        lin_codec: None,
    },
    BuiltinGraphicsCodec {
        i18n_name: "gb",
        is_tiled: true,
        tile_codec: Some(&GBGraphics::new()),
        lin_codec: None,
    },
    BuiltinGraphicsCodec {
        i18n_name: "lin-1bpp-msbfirst",
        is_tiled: false,
        tile_codec: None,
        lin_codec: Some(&_1bppMsbFirstGraphics::new()),
    },
    BuiltinGraphicsCodec {
        i18n_name: "lin-1bpp-lsbfirst",
        is_tiled: false,
        tile_codec: None,
        lin_codec: Some(&_1bppLsbFirstGraphics::new()),
    },
];

#[wasm_bindgen]
pub fn wasm_get_builtin_graphics_codecs() -> Vec<BuiltinGraphicsCodec> {
    BUILTIN_GRAPHICS_CODECS.to_vec()
}

enum TileCorruptorTiledOrLinear {
    Tiled {
        tiles_width: usize,
        tiles_height: usize,
        tile_codec: &'static dyn TileCodec,
    },
    Linear {
        width: usize,
        height: usize,
        lin_codec: &'static dyn LinearCodec,
    },
}

#[wasm_bindgen]
pub struct TileCorruptorAppInst {
    data: Vec<u8>,
    data_bit_off: usize,
    pal: Cow<'static, [(u8, u8, u8)]>,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    px_scale: f64,
    tiled_or_linear: TileCorruptorTiledOrLinear,
}
#[wasm_bindgen]
impl TileCorruptorAppInst {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Self {
        Self {
            data: data.to_owned(),
            data_bit_off: 0,
            pal: (&palette::DEFAULT_PAL).into(),
            canvas: get_canvas(),
            ctx: get_canvas_ctx(),
            px_scale: 2.5,
            tiled_or_linear: TileCorruptorTiledOrLinear::Tiled {
                tiles_width: 32,
                tiles_height: 32,
                tile_codec: BUILTIN_GRAPHICS_CODECS[0].tile_codec.unwrap(),
            },
        }
    }

    pub fn change_codec(&mut self, new_codec_idx: usize) {
        let codec = BUILTIN_GRAPHICS_CODECS[new_codec_idx];
        if codec.is_tiled {
            let (tiles_width, tiles_height) = match self.tiled_or_linear {
                TileCorruptorTiledOrLinear::Tiled {
                    tiles_width,
                    tiles_height,
                    ..
                } => (tiles_width, tiles_height),
                _ => (32, 32),
            };
            self.tiled_or_linear = TileCorruptorTiledOrLinear::Tiled {
                tiles_width,
                tiles_height,
                tile_codec: codec.tile_codec.unwrap(),
            };
            self.resize();
            self.render();
        } else {
            let (width, height) = match self.tiled_or_linear {
                TileCorruptorTiledOrLinear::Linear { width, height, .. } => (width, height),
                _ => (256, 256),
            };
            self.tiled_or_linear = TileCorruptorTiledOrLinear::Linear {
                width,
                height,
                lin_codec: codec.lin_codec.unwrap(),
            };
            self.resize();
            self.render();
        }
    }

    pub fn resize(&self) {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                tiles_width,
                tiles_height,
                tile_codec,
            } => {
                self.canvas
                    .set_width((tiles_width * tile_codec.tile_width()) as u32);
                self.canvas
                    .set_height((tiles_height * tile_codec.tile_height()) as u32);
                self.canvas
                    .style()
                    .set_property(
                        "width",
                        &format!(
                            "{}px",
                            (tiles_width * tile_codec.tile_width()) as f64 * self.px_scale
                        ),
                    )
                    .unwrap();
                self.canvas
                    .style()
                    .set_property(
                        "height",
                        &format!(
                            "{}px",
                            (tiles_height * tile_codec.tile_height()) as f64 * self.px_scale
                        ),
                    )
                    .unwrap();
            }
            TileCorruptorTiledOrLinear::Linear { width, height, .. } => {
                self.canvas.set_width(width as u32);
                self.canvas.set_height(height as u32);
                self.canvas
                    .style()
                    .set_property("width", &format!("{}px", width as f64 * self.px_scale))
                    .unwrap();
                self.canvas
                    .style()
                    .set_property("height", &format!("{}px", height as f64 * self.px_scale))
                    .unwrap();
            }
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

        let gfx_dims = match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                tiles_width,
                tiles_height,
                tile_codec,
            } => {
                format!(
                    "{} x {} tiles ({} x {} px)",
                    tiles_width,
                    tiles_height,
                    tiles_width * tile_codec.tile_width(),
                    tiles_height * tile_codec.tile_height()
                )
            }
            TileCorruptorTiledOrLinear::Linear { width, height, .. } => {
                format!("{} x {} px", width, height)
            }
        };

        gfx_dims_elem.set_inner_text((*gfx_dims).into());
    }

    pub fn render(&self) {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                tiles_width,
                tiles_height,
                tile_codec,
            } => {
                self.ctx.clear_rect(
                    0.0,
                    0.0,
                    (tiles_width * tile_codec.tile_width()) as f64,
                    (tiles_height * tile_codec.tile_height()) as f64,
                );

                tile_codec.render(
                    &mut CanvasPixelWriter { app: self },
                    &self.data[(self.data_bit_off / 8)..],
                    (self.data_bit_off % 8) as u8,
                    tiles_width,
                    tiles_height,
                );
            }
            TileCorruptorTiledOrLinear::Linear {
                width,
                height,
                lin_codec,
            } => {
                self.ctx.clear_rect(0.0, 0.0, width as f64, height as f64);

                lin_codec.render(
                    &mut CanvasPixelWriter { app: self },
                    &self.data[(self.data_bit_off / 8)..],
                    (self.data_bit_off % 8) as u8,
                    width,
                    height,
                );
            }
        }
    }

    pub fn export_png(&self) -> Vec<u8> {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                tiles_width,
                tiles_height,
                tile_codec,
            } => {
                let w = tiles_width * tile_codec.tile_width();
                let h = tiles_height * tile_codec.tile_height();

                let mut ret = Vec::new();
                let mut pixels;

                let mut png_encoder = png::Encoder::new(&mut ret, w as u32, h as u32);
                png_encoder.set_depth(png::BitDepth::Eight);
                if tile_codec.num_palette_colors() > 0 {
                    png_encoder.set_color(png::ColorType::Indexed);
                    // TODO custom palettes
                    let mut pal = Vec::with_capacity(3 * 256);
                    for i in 0..256 {
                        pal.push(self.pal[i].0);
                        pal.push(self.pal[i].1);
                        pal.push(self.pal[i].2);
                    }
                    png_encoder.set_palette(pal);
                    pixels = vec![0u8; w * h];
                } else {
                    png_encoder.set_color(png::ColorType::Rgb);
                    pixels = vec![0u8; w * h * 3];
                }

                tile_codec.render(
                    &mut InMemoryPixelWriter { w, px: &mut pixels },
                    &self.data[(self.data_bit_off / 8)..],
                    (self.data_bit_off % 8) as u8,
                    tiles_width,
                    tiles_height,
                );

                let mut png_writer = png_encoder.write_header().unwrap();
                png_writer.write_image_data(&pixels).unwrap();
                png_writer.finish().unwrap();

                ret
            }
            TileCorruptorTiledOrLinear::Linear {
                width,
                height,
                lin_codec,
            } => {
                let mut ret = Vec::new();
                let mut pixels;

                let mut png_encoder = png::Encoder::new(&mut ret, width as u32, height as u32);
                png_encoder.set_depth(png::BitDepth::Eight);
                if lin_codec.num_palette_colors() > 0 {
                    png_encoder.set_color(png::ColorType::Indexed);
                    // TODO custom palettes
                    let mut pal = Vec::with_capacity(3 * 256);
                    for i in 0..256 {
                        pal.push(self.pal[i].0);
                        pal.push(self.pal[i].1);
                        pal.push(self.pal[i].2);
                    }
                    png_encoder.set_palette(pal);
                    pixels = vec![0u8; width * height];
                } else {
                    png_encoder.set_color(png::ColorType::Rgb);
                    pixels = vec![0u8; width * height * 3];
                }

                lin_codec.render(
                    &mut InMemoryPixelWriter {
                        w: width,
                        px: &mut pixels,
                    },
                    &self.data[(self.data_bit_off / 8)..],
                    (self.data_bit_off % 8) as u8,
                    width,
                    height,
                );

                let mut png_writer = png_encoder.write_header().unwrap();
                png_writer.write_image_data(&pixels).unwrap();
                png_writer.finish().unwrap();

                ret
            }
        }
    }

    pub fn width_minus(&mut self) {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                ref mut tiles_width,
                ..
            } => {
                if *tiles_width > 1 {
                    *tiles_width -= 1;
                }
            }
            TileCorruptorTiledOrLinear::Linear { ref mut width, .. } => {
                if *width > 1 {
                    *width -= 1;
                }
            }
        }
        self.resize();
        self.render();
        self.update_status_bar();
    }
    pub fn width_plus(&mut self) {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                ref mut tiles_width,
                ..
            } => {
                *tiles_width += 1;
            }
            TileCorruptorTiledOrLinear::Linear { ref mut width, .. } => {
                *width += 1;
            }
        }
        self.resize();
        self.render();
        self.update_status_bar();
    }
    pub fn height_minus(&mut self) {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                ref mut tiles_height,
                ..
            } => {
                if *tiles_height > 1 {
                    *tiles_height -= 1;
                }
            }
            TileCorruptorTiledOrLinear::Linear { ref mut height, .. } => {
                if *height > 1 {
                    *height -= 1;
                }
            }
        }
        self.resize();
        self.render();
        self.update_status_bar();
    }
    pub fn height_plus(&mut self) {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                ref mut tiles_height,
                ..
            } => {
                *tiles_height += 1;
            }
            TileCorruptorTiledOrLinear::Linear { ref mut height, .. } => {
                *height += 1;
            }
        }
        self.resize();
        self.render();
        self.update_status_bar();
    }

    pub fn tile_minus(&mut self) {
        if let TileCorruptorTiledOrLinear::Tiled { tile_codec, .. } = self.tiled_or_linear {
            let bits_per_tile = tile_codec.bits_per_tile();
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
        if let TileCorruptorTiledOrLinear::Tiled { tile_codec, .. } = self.tiled_or_linear {
            let new_off = self.data_bit_off + tile_codec.bits_per_tile();
            if new_off < self.data.len() * 8 {
                self.data_bit_off = new_off;
                self.render();
                self.update_status_bar();
            }
        }
    }

    pub fn px_minus(&mut self) {
        if let TileCorruptorTiledOrLinear::Linear { lin_codec, .. } = self.tiled_or_linear {
            let bpp = lin_codec.bits_per_pixel();
            if self.data_bit_off >= bpp {
                self.data_bit_off -= bpp;
            } else {
                self.data_bit_off = 0;
            }
            self.render();
            self.update_status_bar();
        }
    }
    pub fn px_plus(&mut self) {
        if let TileCorruptorTiledOrLinear::Linear { lin_codec, .. } = self.tiled_or_linear {
            let new_off = self.data_bit_off + lin_codec.bits_per_pixel();
            if new_off < self.data.len() * 8 {
                self.data_bit_off = new_off;
                self.render();
                self.update_status_bar();
            }
        }
    }

    pub fn row_minus(&mut self, faster: bool) {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                tiles_width,
                tile_codec,
                ..
            } => {
                let bits_per_row =
                    tile_codec.bits_per_tile() * tiles_width * if faster { 8 } else { 1 };
                if self.data_bit_off >= bits_per_row {
                    self.data_bit_off -= bits_per_row;
                } else {
                    self.data_bit_off = 0;
                }
            }
            TileCorruptorTiledOrLinear::Linear {
                width, lin_codec, ..
            } => {
                let bits_per_row = lin_codec.bits_per_row(width) * if faster { 32 } else { 1 };
                if self.data_bit_off >= bits_per_row {
                    self.data_bit_off -= bits_per_row;
                } else {
                    self.data_bit_off = 0;
                }
            }
        }
        self.render();
        self.update_status_bar();
    }
    pub fn row_plus(&mut self, faster: bool) {
        match self.tiled_or_linear {
            TileCorruptorTiledOrLinear::Tiled {
                tiles_width,
                tile_codec,
                ..
            } => {
                let new_off = self.data_bit_off
                    + tile_codec.bits_per_tile() * tiles_width * if faster { 8 } else { 1 };
                if new_off < self.data.len() * 8 {
                    self.data_bit_off = new_off;
                }
            }
            TileCorruptorTiledOrLinear::Linear {
                width, lin_codec, ..
            } => {
                let new_off =
                    self.data_bit_off + lin_codec.bits_per_row(width) * if faster { 32 } else { 1 };
                if new_off < self.data.len() * 8 {
                    self.data_bit_off = new_off;
                }
            }
        }
        self.render();
        self.update_status_bar();
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
