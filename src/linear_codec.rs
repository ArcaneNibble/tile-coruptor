use bitvec::prelude::*;
use std::marker::PhantomData;

use crate::AbstractPixelTarget;

pub trait LinearCodec {
    fn num_palette_colors(&self) -> usize;
    fn bits_per_pixel(&self) -> usize;
    fn bits_per_row(&self, w: usize) -> usize;
    fn render(
        &self,
        r: &mut dyn AbstractPixelTarget,
        bytes: &[u8],
        bit_offs: u8,
        w: usize,
        h: usize,
    );
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NbppPalettedGraphics<
    DataBitOrder: BitOrder,
    const BPP: usize,
    const PX_PAD: usize,
    const ROW_PAD: usize,
> {
    _pd: PhantomData<DataBitOrder>,
}
impl<DataBitOrder: BitOrder, const BPP: usize, const PX_PAD: usize, const ROW_PAD: usize>
    NbppPalettedGraphics<DataBitOrder, BPP, PX_PAD, ROW_PAD>
{
    pub const fn new() -> Self {
        Self { _pd: PhantomData }
    }
}
impl<DataBitOrder: BitOrder, const BPP: usize, const PX_PAD: usize, const ROW_PAD: usize>
    LinearCodec for NbppPalettedGraphics<DataBitOrder, BPP, PX_PAD, ROW_PAD>
{
    fn num_palette_colors(&self) -> usize {
        1 << BPP
    }

    fn bits_per_pixel(&self) -> usize {
        BPP + PX_PAD
    }

    fn bits_per_row(&self, w: usize) -> usize {
        (BPP + PX_PAD) * w + ROW_PAD
    }

    fn render(
        &self,
        r: &mut dyn AbstractPixelTarget,
        bytes: &[u8],
        bit_offs: u8,
        w: usize,
        h: usize,
    ) {
        debug_assert!(BPP <= 8);
        let bits = bytes.view_bits::<DataBitOrder>();

        let data_bits_per_row = (BPP + PX_PAD) * w + ROW_PAD;

        for y in 0..h {
            for x in 0..w {
                let mut px = [0u8; 1];
                let px_bv = px.view_bits_mut::<Lsb0>();
                for b_i in 0..BPP {
                    let bit_idx = bit_offs as usize + y * data_bits_per_row + x;
                    if bit_idx >= bits.len() {
                        return;
                    }
                    px_bv.set(b_i, bits[bit_idx]);
                }
                r.draw_px_pal(x, y, px[0]);
            }
        }
    }
}

pub type _1bppMsbFirstGraphics = NbppPalettedGraphics<Msb0, 1, 0, 0>;
pub type _1bppLsbFirstGraphics = NbppPalettedGraphics<Lsb0, 1, 0, 0>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_obj_safe() {
        let _: &dyn LinearCodec;
    }
}
