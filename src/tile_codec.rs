use bitvec::{
    index::{BitIdx, BitPos},
    mem::BitRegister,
    prelude::*,
};
use std::marker::PhantomData;

use crate::AbstractPixelTarget;

pub trait TileCodec {
    fn num_palette_colors(&self) -> usize;
    fn bits_per_tile(&self) -> usize;
    fn tile_width(&self) -> usize;
    fn tile_height(&self) -> usize;
    fn render(
        &self,
        r: &mut dyn AbstractPixelTarget,
        bytes: &[u8],
        bit_offs: u8,
        tiles_w: usize,
        tiles_h: usize,
    );
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
    >
    PlanarNonInterleavedTileGraphics<
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
    pub const fn new() -> Self {
        Self { _pd: PhantomData }
    }
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
        1 << PLANES
    }

    fn bits_per_tile(&self) -> usize {
        let data_bits_per_row = TILE_W + TILE_W_PAD;
        let data_bits_per_plane = data_bits_per_row * TILE_H + PLANE_PAD;
        let data_bits_per_tile = data_bits_per_plane * PLANES + FINAL_PAD;

        data_bits_per_tile
    }

    fn tile_width(&self) -> usize {
        TILE_W
    }

    fn tile_height(&self) -> usize {
        TILE_H
    }

    fn render(
        &self,
        r: &mut dyn AbstractPixelTarget,
        bytes: &[u8],
        bit_offs: u8,
        tiles_w: usize,
        tiles_h: usize,
    ) {
        debug_assert!(PLANES <= 8);
        let bits = bytes.view_bits::<DataBitOrder>();

        let data_bits_per_row = TILE_W + TILE_W_PAD;
        let data_bits_per_plane = data_bits_per_row * TILE_H + PLANE_PAD;
        let data_bits_per_tile = data_bits_per_plane * PLANES + FINAL_PAD;

        for tile_y in 0..tiles_h {
            for tile_x in 0..tiles_w {
                let tile_i = tile_y * tiles_w + tile_x;
                for px_y in 0..TILE_H {
                    for px_x in 0..TILE_W {
                        let mut px = [0u8; 1];
                        let px_bv = px.view_bits_mut::<PlaneBitOrder>();
                        for plane in 0..PLANES {
                            let bit_idx = bit_offs as usize
                                + tile_i * data_bits_per_tile
                                + plane * data_bits_per_plane
                                + px_y * data_bits_per_row
                                + px_x;
                            if bit_idx >= bits.len() {
                                return;
                            }
                            px_bv.set(plane, bits[bit_idx]);
                        }

                        r.draw_px_pal(tile_x * TILE_W + px_x, tile_y * TILE_H + px_y, px[0]);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlanarInterleavedTileGraphics<
    DataBitOrder: BitOrder,
    PlaneBitOrder: BitOrder,
    const PLANES: usize,
    const TILE_W: usize,
    const TILE_H: usize,
    const TILE_W_PAD: usize,
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
        const FINAL_PAD: usize,
    >
    PlanarInterleavedTileGraphics<
        DataBitOrder,
        PlaneBitOrder,
        PLANES,
        TILE_W,
        TILE_H,
        TILE_W_PAD,
        FINAL_PAD,
    >
{
    pub const fn new() -> Self {
        Self { _pd: PhantomData }
    }
}
impl<
        DataBitOrder: BitOrder,
        PlaneBitOrder: BitOrder,
        const PLANES: usize,
        const TILE_W: usize,
        const TILE_H: usize,
        const TILE_W_PAD: usize,
        const FINAL_PAD: usize,
    > TileCodec
    for PlanarInterleavedTileGraphics<
        DataBitOrder,
        PlaneBitOrder,
        PLANES,
        TILE_W,
        TILE_H,
        TILE_W_PAD,
        FINAL_PAD,
    >
{
    fn num_palette_colors(&self) -> usize {
        1 << PLANES
    }

    fn bits_per_tile(&self) -> usize {
        let data_bits_per_plane = TILE_W + TILE_W_PAD;
        let data_bits_per_row = data_bits_per_plane * PLANES;
        let data_bits_per_tile = data_bits_per_row * TILE_H + FINAL_PAD;

        data_bits_per_tile
    }

    fn tile_width(&self) -> usize {
        TILE_W
    }

    fn tile_height(&self) -> usize {
        TILE_H
    }

    fn render(
        &self,
        r: &mut dyn AbstractPixelTarget,
        bytes: &[u8],
        bit_offs: u8,
        tiles_w: usize,
        tiles_h: usize,
    ) {
        debug_assert!(PLANES <= 8);
        let bits = bytes.view_bits::<DataBitOrder>();

        let data_bits_per_plane = TILE_W + TILE_W_PAD;
        let data_bits_per_row = data_bits_per_plane * PLANES;
        let data_bits_per_tile = data_bits_per_row * TILE_H + FINAL_PAD;

        for tile_y in 0..tiles_h {
            for tile_x in 0..tiles_w {
                let tile_i = tile_y * tiles_w + tile_x;
                for px_y in 0..TILE_H {
                    for px_x in 0..TILE_W {
                        let mut px = [0u8; 1];
                        let px_bv = px.view_bits_mut::<PlaneBitOrder>();
                        for plane in 0..PLANES {
                            let bit_idx = bit_offs as usize
                                + tile_i * data_bits_per_tile
                                + plane * data_bits_per_plane
                                + px_y * data_bits_per_row
                                + px_x;
                            if bit_idx >= bits.len() {
                                return;
                            }
                            px_bv.set(plane, bits[bit_idx]);
                        }

                        r.draw_px_pal(tile_x * TILE_W + px_x, tile_y * TILE_H + px_y, px[0]);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NonPlanarTileGraphics<
    DataBitOrder: BitOrder,
    const BPP: usize,
    const TILE_W: usize,
    const TILE_H: usize,
    const TILE_W_PAD: usize,
    const FINAL_PAD: usize,
> {
    _pd: PhantomData<DataBitOrder>,
}
impl<
        DataBitOrder: BitOrder,
        const BPP: usize,
        const TILE_W: usize,
        const TILE_H: usize,
        const TILE_W_PAD: usize,
        const FINAL_PAD: usize,
    > NonPlanarTileGraphics<DataBitOrder, BPP, TILE_W, TILE_H, TILE_W_PAD, FINAL_PAD>
{
    pub const fn new() -> Self {
        Self { _pd: PhantomData }
    }
}
impl<
        DataBitOrder: BitOrder,
        const BPP: usize,
        const TILE_W: usize,
        const TILE_H: usize,
        const TILE_W_PAD: usize,
        const FINAL_PAD: usize,
    > TileCodec
    for NonPlanarTileGraphics<DataBitOrder, BPP, TILE_W, TILE_H, TILE_W_PAD, FINAL_PAD>
{
    fn num_palette_colors(&self) -> usize {
        1 << BPP
    }

    fn bits_per_tile(&self) -> usize {
        let data_bits_per_row = BPP * TILE_W + TILE_W_PAD;
        let data_bits_per_tile = data_bits_per_row * TILE_H + FINAL_PAD;

        data_bits_per_tile
    }

    fn tile_width(&self) -> usize {
        TILE_W
    }

    fn tile_height(&self) -> usize {
        TILE_H
    }

    fn render(
        &self,
        r: &mut dyn AbstractPixelTarget,
        bytes: &[u8],
        bit_offs: u8,
        tiles_w: usize,
        tiles_h: usize,
    ) {
        debug_assert!(BPP <= 8);
        let bits = bytes.view_bits::<DataBitOrder>();

        let data_bits_per_row = BPP * TILE_W + TILE_W_PAD;
        let data_bits_per_tile = data_bits_per_row * TILE_H + FINAL_PAD;

        for tile_y in 0..tiles_h {
            for tile_x in 0..tiles_w {
                let tile_i = tile_y * tiles_w + tile_x;
                for px_y in 0..TILE_H {
                    for px_x in 0..TILE_W {
                        let mut px = [0u8; 1];
                        let px_bv = px.view_bits_mut::<Lsb0>();
                        for b_i in 0..BPP {
                            let bit_idx = bit_offs as usize
                                + tile_i * data_bits_per_tile
                                + px_y * data_bits_per_row
                                + px_x * BPP
                                + b_i;
                            if bit_idx >= bits.len() {
                                return;
                            }
                            px_bv.set(b_i, bits[bit_idx]);
                        }

                        r.draw_px_pal(tile_x * TILE_W + px_x, tile_y * TILE_H + px_y, px[0]);
                    }
                }
            }
        }
    }
}

pub type NESGraphics = PlanarNonInterleavedTileGraphics<Msb0, Lsb0, 2, 8, 8, 0, 0, 0>;
pub type GBGraphics = PlanarInterleavedTileGraphics<Msb0, Lsb0, 2, 8, 8, 0, 0>;
pub type GBATileGraphics4bpp = NonPlanarTileGraphics<Lsb0, 4, 8, 8, 0, 0>;
pub type GenesisGraphics4bpp = NonPlanarTileGraphics<HiLo, 4, 8, 8, 0, 0>;
pub type TileGraphics8bpp = NonPlanarTileGraphics<Lsb0, 8, 8, 8, 0, 0>;

pub struct HiLo;
unsafe impl BitOrder for HiLo {
    fn at<R>(index: BitIdx<R>) -> BitPos<R>
    where
        R: BitRegister,
    {
        unsafe { BitPos::new_unchecked(index.into_inner() ^ 4) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_obj_safe() {
        let _: &dyn TileCodec;
    }

    #[test]
    fn prove_hilo() {
        bitvec::order::verify::<HiLo>(true);
    }
}
