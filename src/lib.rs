const QOI_OP_RGB: u8    = 0b11111110;
const QOI_OP_RGBA: u8   = 0b11111111;
const QOI_OP_INDEX: u8  = 0b00000000;
const QOI_OP_DIFF: u8   = 0b01000000;
const QOI_OP_LUMA: u8   = 0b10000000;
const QOI_OP_RUN: u8    = 0b11000000;

#[derive(Clone,Copy)]
#[repr(packed(1))]
pub struct Pixel {
    pub r: u8,
    pub g: u8, 
    pub b: u8,
    pub a: u8
}

pub struct QOIDecode {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub colorspace: u8,
    pub pixels: Vec<Pixel>
}

impl QOIDecode {
    /// Decode byte slice as a QOI image. 
    ///
    /// See [the detailed spec](https://qoiformat.org/qoi-specification.pdf). 
    pub fn from_bytes(mut bytes: &[u8]) -> Result<QOIDecode, String> {
        let (
            width,
            height,
            channels,
            colorspace
        ) = match bytes {
            [
                113u8, 111u8, 105u8, 102u8,
                w0, w1, w2, w3,
                h0, h1, h2, h3,
                channels,
                colorspace,
                rest @ ..
            ] => { 
                bytes = rest;
                (
                    u32::from_be_bytes([*w0, *w1, *w2, *w3]),
                    u32::from_be_bytes([*h0, *h1, *h2, *h3]),
                    *channels,
                    *colorspace,
                )
            },
            _ => { return Err("Invalid header".to_string()) }
        };
        
        let mut curr = Pixel { r: 0, g: 0, b: 0, a: 255 };
        let mut array = [Pixel { r:0, g:0, b:0, a: 0 } ; 64];
        let mut run: u8;

        let mut pixels: Vec<Pixel> = Vec::with_capacity((width * height) as usize);

        loop {
            run = 0;
            match bytes {
                // ┌─ END MARKER ─────┬─────────┬─────────────────┐
                // │ Byte[0] │   ..   │ Byte[6] │     Byte[7]     │
                // │ 7 .. 0  │        │ 7 .. 0  │ 7 6 5 4 3 2 1 0 │
                // ├─────────┼────────┼─────────┼─────────────────┤
                // │ 0 .. 0  │ 0 .. 0 │ 0 .. 0  │ 0 0 0 0 0 0 0 1 │
                // └─────────┴────────┴─────────┴─────────────────┘
                // All chunks have been processed.
                [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8] => break Ok(
                    QOIDecode{
                        width,
                        height,
                        channels,
                        colorspace,
                        pixels
                    }
                ),

                // ┌─ QOI_OP_RGB ────┬─────────┬─────────┬─────────┐
                // │     Byte[0]     │ Byte[1] │ Byte[2] │ Byte[3] │
                // │ 7 6 5 4 3 2 1 0 │ 7 .. 0  │ 7 .. 0  │ 7 .. 0  │
                // ├─────────────────┼─────────┼─────────┼─────────┤
                // │ 1 1 1 1 1 1 1 0 │   red   │  green  │  blue   │
                // └─────────────────┴─────────┴─────────┴─────────┘
                //                     0..255    0..255    0..255
                [QOI_OP_RGB, r, g, b, rest @ ..] => {
                    curr.r = *r;
                    curr.g = *g;
                    curr.b = *b;
                    bytes = rest;
                },

                // ┌─ QOI_OP_RGBA ───┬─────────┬─────────┬─────────┬─────────┐
                // │     Byte[0]     │ Byte[1] │ Byte[2] │ Byte[3] │ Byte[4] │
                // │ 7 6 5 4 3 2 1 0 │ 7 .. 0  │ 7 .. 0  │ 7 .. 0  │ 7 .. 0  │
                // ├─────────────────┼─────────┼─────────┼─────────┼─────────┤
                // │ 1 1 1 1 1 1 1 1 │   red   │  green  │  blue   │  alpha  │
                // └─────────────────┴─────────┴─────────┴─────────┴─────────┘
                //                     0..255    0..255    0..255    0..255
                [QOI_OP_RGBA, r, g, b, a, rest @ ..] => {
                    curr.r = *r;
                    curr.g = *g;
                    curr.b = *b;
                    curr.a = *a;
                    bytes = rest;
                },

                // ┌─ QOI_OP_INDEX ────┐
                // │      Byte[0]      │
                // │ 7 6   5 4 3 2 1 0 │
                // ├─────┬─────────────┤
                // │ 0 0 │    index    │
                // └─────┴─────────────┘
                //            0..64
                [b0, rest @ ..] if b0 & 0b11000000 == QOI_OP_INDEX => {
                    curr = array[*b0 as usize];
                    bytes = rest;
                },

                // ┌─ QOI_OP_DIFF ────────────┐
                // │        Byte[0]           │
                // │ 7 6    5 4    3 2    1 0 │
                // ├─────┬──────┬──────┬──────┤
                // │ 0 1 │  dr  │  dg  │  db  │
                // └─────┴──────┴──────┴──────┘
                //        -2..1  -2..1  -2..1
                [b0, rest @ ..] if b0 & 0b11000000  == QOI_OP_DIFF => {
                    let (dr, dg, db) = (b0 >> 4 & 0b00000011, b0 >> 2 & 0b00000011, b0 & 0b00000011);

                    curr.r = curr.r.wrapping_add(dr).wrapping_sub(2);
                    curr.g = curr.g.wrapping_add(dg).wrapping_sub(2);
                    curr.b = curr.b.wrapping_add(db).wrapping_sub(2);
                    bytes = rest;
                },

                // ┌─ QOI_OP_LUMA ─────┐
                // │      Byte[0]      │
                // │ 7 6   5 4 3 2 1 0 │
                // ├─────┬─────────────┤
                // │ 1 0 │     dg      │
                // └─────┴─────────────┘
                //          -32..31
                // Index into pixel array
                [b0, b1, rest @ ..] if b0 & 0b11000000 == QOI_OP_LUMA => {
                    let (dg, drdg, dbdg) = (b0 & 0b00111111, b1 >> 4 & 0b00001111, b1 & 0b00001111);
                    curr.r = curr.r.wrapping_add(drdg).wrapping_add(dg).wrapping_sub(40);
                    curr.g = curr.g.wrapping_add(dg).wrapping_sub(32);
                    curr.b = curr.b.wrapping_add(dbdg).wrapping_add(dg).wrapping_sub(40);
                    bytes = rest;
                },

                // ┌─ QOI_OP_RUN ──────┐
                // │      Byte[0]      │
                // │ 7 6   5 4 3 2 1 0 │
                // ├─────┬─────────────┤
                // │ 1 1 │     run     │
                // └─────┴─────────────┘
                // Index into pixel array
                [b0, rest @ ..] if b0 & 0b11000000 == QOI_OP_RUN => {
                    run = b0 & 0b00111111;
                    bytes = rest;
                },
                _ => { return Err("Invalid chunk".to_string()) }
            }
            let hash = (curr.r as usize * 3) + (curr.g as usize * 5) + (curr.b as usize * 7) + (curr.a as usize * 11);
            array[hash % 64] = curr;
            for _ in 0..=run {
                pixels.push(curr);
            }
        }
    }
}
