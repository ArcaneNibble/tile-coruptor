use bitvec::prelude::*;
use std::marker::PhantomData;

pub trait TileCodec {
    fn num_palette_colors(&self) -> usize;
    fn tile_width(&self) -> usize;
    fn tile_height(&self) -> usize;
    fn xxx_render(&self, bytes: &[u8], tiles_w: usize, tiles_h: usize);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlanarNonInterleavedTileGraphics<
    DataBitOrder: BitOrder,
    PlaneBitOrder: BitOrder,
    const PLANES: usize,
    const TILE_W: usize,
    const TILE_H: usize,
    const TILE_W_PAD: usize,
    const PLANE_PAD: usize,
    const FINAL_PAD: usize,
> {
    _pd: PhantomData<(DataBitOrder, PlaneBitOrder)>,
}
impl<
        DataBitOrder: BitOrder,
        PlaneBitOrder: BitOrder,
        const PLANES: usize,
        const TILE_W: usize,
        const TILE_H: usize,
        const TILE_W_PAD: usize,
        const PLANE_PAD: usize,
        const FINAL_PAD: usize,
    > TileCodec
    for PlanarNonInterleavedTileGraphics<
        DataBitOrder,
        PlaneBitOrder,
        PLANES,
        TILE_W,
        TILE_H,
        TILE_W_PAD,
        PLANE_PAD,
        FINAL_PAD,
    >
{
    fn num_palette_colors(&self) -> usize {
        1 << (PLANES - 1)
    }

    fn tile_width(&self) -> usize {
        TILE_W
    }

    fn tile_height(&self) -> usize {
        TILE_H
    }

    fn xxx_render(&self, bytes: &[u8], tiles_w: usize, tiles_h: usize) {
        debug_assert!(PLANES <= 8);
        let bits = bytes.view_bits::<DataBitOrder>();

        let data_bits_per_row = TILE_W + TILE_W_PAD;
        let data_bits_per_plane = data_bits_per_row * TILE_H + PLANE_PAD;
        let data_bits_per_tile = data_bits_per_plane * PLANES + FINAL_PAD;

        let ctx = crate::get_canvas_ctx();

        for tile_y in 0..tiles_h {
            for tile_x in 0..tiles_w {
                let tile_i = tile_y * tiles_w + tile_x;
                for px_y in 0..TILE_H {
                    for px_x in 0..TILE_W {
                        let mut px = [0u8; 1];
                        let px_bv = px.view_bits_mut::<PlaneBitOrder>();
                        for plane in 0..PLANES {
                            let bit_idx = tile_i * data_bits_per_tile
                                + plane * data_bits_per_plane
                                + px_y * data_bits_per_row
                                + px_x;
                            if bit_idx >= bits.len() {
                                return;
                            }
                            px_bv.set(plane, bits[bit_idx]);
                        }

                        let color = px[0] << 6;

                        ctx.set_fill_style(&format!("rgb({},{},{})", color, color, color).into());
                        ctx.fill_rect(
                            ((tile_x * TILE_W + px_x) * 4) as f64,
                            ((tile_y * TILE_H + px_y) * 4) as f64,
                            4.0,
                            4.0,
                        );
                    }
                }
            }
        }
    }
}

pub type NESGraphics = PlanarNonInterleavedTileGraphics<Msb0, Lsb0, 2, 8, 8, 0, 0, 0>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_obj_safe() {
        let _: &dyn TileCodec;
    }
}
