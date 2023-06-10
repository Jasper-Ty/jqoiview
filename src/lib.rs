pub type Pix = (u8, u8, u8, u8);
pub fn hash((r, g, b, a): (u8, u8, u8, u8)) -> usize {
    let (r, g, b, a) = (
        (r as usize) * 3,
        (g as usize) * 5,
        (b as usize) * 7,
        (a as usize) * 11,
    );
    (r + g + b + a) % 64
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

pub struct ChunkIter<I: Iterator<Item=u8>> (I);

impl<I> Iterator for ChunkIter<I> 
where
    I: Iterator<Item = u8>
{
    type Item = Chunk;

    fn next(&mut self) -> Option<Chunk> {
        let Self(iter) = self;
        let b1 = iter.next()?;
        match b1 {
            QOI_OP_RGB => Some(RGB(
                iter.next()?, 
                iter.next()?, 
                iter.next()?,
            )),
            QOI_OP_RGBA => Some(RGBA(
                iter.next()?, 
                iter.next()?, 
                iter.next()?, 
                iter.next()?,
            )),
            b1 => match b1 & QOI_MASK_2 {
                QOI_OP_INDEX => Some(INDEX(b1 & 0b00111111)),
                QOI_OP_DIFF => Some(DIFF(b1 >> 4 & 3, b1 >> 2 & 3, b1 & 3)),
                QOI_OP_LUMA => {
                    let b2 = iter.next()?;
                    Some(LUMA(
                        b1 & 0b00111111,
                        (b2 & 0b11110000) >> 4,
                        b2 & 0b00001111, 
                    ))
                },
                QOI_OP_RUN => Some(RUN(b1 & 0b00111111)),
                _ => None,
            }
        }
    }
}
pub trait Chunks {
    fn chunks(self) -> ChunkIter<Self> 
    where
        Self: Sized + Iterator<Item = u8>;
}
impl<I> Chunks for I 
where
    I: Iterator<Item = u8>
{
    fn chunks(self) -> ChunkIter<Self> {
        ChunkIter(self)
    }
}

use std::fs::File;
use std::io::Read;
use std::io::Result;

#[derive(Debug)]
pub struct QoiHeader {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub colorspace: u8,
}
impl QoiHeader {
    pub fn from_file(f: &mut File) -> Result<QoiHeader> {
        let mut buf: [u8; 4] = [0u8; 4];
        f.read(&mut buf)?;
        let width = u32::from_be_bytes(buf);

        let mut buf: [u8; 4] = [0u8; 4];
        f.read(&mut buf)?;
        let height = u32::from_be_bytes(buf);

        let mut buf: [u8; 2] = [0u8; 2];
        f.read(&mut buf)?;
        let channels = buf[0];
        let colorspace = buf[1];

        Ok(QoiHeader {
            width,
            height,
            channels,
            colorspace,
        })
    }
}
