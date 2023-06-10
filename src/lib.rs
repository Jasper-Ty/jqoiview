use std::{
    fs::File,
    io::{
        Read,
        Result,
        Seek,
        SeekFrom,
    }
};

pub struct Header {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub colorspace: u8,
}
impl Header {
    pub const SIZE: u64 = 14;
    pub fn from_file(f: &mut File) -> Result<Self> {
        f.seek(SeekFrom::Start(0))?;
        let mut buf: [u8; 4] = [0u8; 4];
        f.read(&mut buf)?;
        assert_eq!(buf, [113u8, 111u8, 105u8, 102u8]);

        f.read(&mut buf)?;
        let width = u32::from_be_bytes(buf);

        f.read(&mut buf)?;
        let height = u32::from_be_bytes(buf);

        f.read(&mut buf)?;
        let channels = buf[0];
        let colorspace = buf[1];

        let mut buf: [u8; 8] = [0u8; 8];
        f.seek(SeekFrom::End(-8))?;
        f.read(&mut buf)?;
        assert_eq!(buf, [0, 0, 0, 0, 0, 0, 0, 1]);

        Ok(Self {
            width,
            height,
            channels,
            colorspace,
        })
    }
}

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

#[derive(Clone, Copy)]
pub enum Chunk {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
    INDEX(u8),
    DIFF(u8, u8, u8),
    LUMA(u8, u8, u8),
    RUN(u8),
}
impl Chunk {
    pub fn parse(&self, curr: Pix, index: &[Pix]) -> (Pix, u8) {
        match *self {
            RGB(r, g, b) => ((r, g, b, curr.3), 0),
            RGBA(r, g, b, a) => ((r, g, b, a), 0), 
            INDEX(i) => (index[i as usize], 0),
            DIFF(dr, dg, db) => ((
                curr.0
                    .wrapping_add(dr)
                    .wrapping_sub(2),
                curr.1
                    .wrapping_add(dg)
                    .wrapping_sub(2),
                curr.2
                    .wrapping_add(db)
                    .wrapping_sub(2),
                curr.3,
            ), 0),
            LUMA(dg, drdg, dbdg) => ((
                curr.0
                    .wrapping_add(dg)
                    .wrapping_sub(40)
                    .wrapping_add(drdg),
                curr.1
                    .wrapping_add(dg)
                    .wrapping_sub(32),
                curr.2
                    .wrapping_add(dg)
                    .wrapping_sub(40)
                    .wrapping_add(dbdg),
                curr.3,
            ), 0),
            RUN(len) => (curr, len), 
        }
    }
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
            b1 => match b1 & 0b11000000 {
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
