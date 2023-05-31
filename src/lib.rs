pub mod encode;
pub mod decode;
pub mod qoi;

pub type Pix = (u8, u8, u8, u8);
pub fn hash(pix: Pix) -> usize {
    let (r, g, b, a) = pix;
    let (r, g, b, a) = (r as usize, g as usize, b as usize, a as usize);
    ((r*3 + g*5 + b*7 + a*11) % 64) as usize
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
                    Some(LUMA(
                        b1 & 0b00111111,
                        b.next()? & 0b11110000 >> 4,
                        b.next()? & 0b00001111, 
                    ))
                },
                QOI_OP_RUN => Some(RUN(b1 & 0b00111111)),
                _ => None,
            }
        }
    }
}

