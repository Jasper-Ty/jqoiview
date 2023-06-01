pub mod encode;
pub mod decode;
pub mod qoi;

pub type Pix = (u8, u8, u8, u8);
pub fn hash((r, g, b, a): (u8, u8, u8, u8)) -> u8 {
    let (r, g, b, a) = (
        r.wrapping_mul(3),
        g.wrapping_mul(5),
        b.wrapping_mul(7),
        a.wrapping_mul(11),
    );
    r.wrapping_add(g).wrapping_add(b).wrapping_add(a) % 64
} 

const QOI_OP_RGB: u8    = 0b11111110;
const QOI_OP_RGBA: u8   = 0b11111111;
const QOI_OP_INDEX: u8  = 0b00000000;
const QOI_OP_DIFF: u8   = 0b01000000;
const QOI_OP_LUMA: u8   = 0b10000000;
const QOI_OP_RUN: u8    = 0b11000000;

const QOI_MASK_2: u8    = 0b11000000;

pub enum Chunk {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
    INDEX(u8),
    DIFF(u8, u8, u8),
    LUMA(u8, u8, u8),
    RUN(u8),
}

use Chunk::*;
pub struct ChunkIterator<T: Iterator<Item=u8>> {
    iter: T,
}
impl<T> ChunkIterator<T> 
where
    T: Iterator<Item=u8>
{
    pub fn new(iter: T) -> Self {
        ChunkIterator { iter }
    }
}
impl<T> Iterator for ChunkIterator<T>
where
    T: Iterator<Item=u8>
{
    type Item = Chunk;

    fn next(&mut self) -> Option<Chunk> {
        let b = &mut self.iter;
        let b1 = b.next()?;
        match b1 {
            QOI_OP_RGB => Some(RGB(b.next()?, b.next()?, b.next()?)),
            QOI_OP_RGBA => Some(RGBA(b.next()?, b.next()?, b.next()?, b.next()?)),
            b1 => match b1 & QOI_MASK_2 {
                QOI_OP_INDEX => Some(INDEX(b1)),
                QOI_OP_DIFF => Some(DIFF((b1 >> 4) & 0x03, (b1 >> 2) & 0x03, b1 & 0x03)),
                QOI_OP_LUMA => {
                    let b2 = b.next()?;
                    Some(LUMA(
                        b1 & 0b00111111,
                        b2 & 0b11110000 >> 4,
                        b2 & 0b00001111, 
                    ))
                },
                QOI_OP_RUN => Some(RUN(b1 & 0b00111111)),
                _ => None,
            }
        }
    }
}

