use std::fs::File;
use std::io::{ Read, Result, Error, ErrorKind };

use super::hash;
use super::Pix;
use super::Chunk;
use super::Chunk::*;
use super::qoi::QoiHeader;

const QOI_OP_RGB: u8    = 0b11111110;
const QOI_OP_RGBA: u8   = 0b11111111;
const QOI_OP_INDEX: u8  = 0b00000000;
const QOI_OP_DIFF: u8   = 0b01000000;
const QOI_OP_LUMA: u8   = 0b10000000;
const QOI_OP_RUN: u8    = 0b11000000;

const QOI_MASK_2: u8    = 0b11000000;

pub struct Decode {
    header: QoiHeader,
    index: [(u8, u8, u8, u8); 64],
    f: File,
}

pub struct DecodeOutput {
    pub width: u32,
    pub height: u32, 
    pub bytes: Vec<Pix>,
}

impl Decode {
    pub fn new(header: QoiHeader, f: File) -> Decode {
        let index = [(0u8, 0u8, 0u8, 0u8); 64];
        Decode {
            header,
            index,
            f,
        }
    }
    fn next_chunk(&mut self) -> Result<Chunk> {
        let mut buf: [u8; 4] = [0u8; 4];
        self.f.read(&mut buf[0..1])?;
        match buf[0] {
            QOI_OP_RGB => {
                self.f.read(&mut buf[0..3])?;
                Ok(RGB(buf[0], buf[1], buf[2]))
            },
            QOI_OP_RGBA => {
                self.f.read(&mut buf[0..4])?;
                Ok(RGBA(buf[0], buf[1], buf[2], buf[3]))
            },
            b => match b & QOI_MASK_2 {
                QOI_OP_INDEX => Ok(INDEX(b)),
                QOI_OP_DIFF => Ok(DIFF((b >> 4) & 3, (b >> 2) & 3, b & 3)),
                QOI_OP_LUMA => {
                    self.f.read(&mut buf[0..1])?;
                    Ok(LUMA(
                        b & 0b00111111,
                        buf[0] & 0b11110000 >> 4,
                        buf[0] & 0b00001111, 
                    ))
                },
                QOI_OP_RUN => Ok(RUN(b & 0b00111111)),
                _ => Err(Error::new(ErrorKind::Other, "Invalid Chunk detected"))
            }
        }
    }

    fn next_px(&mut self, chunk: Chunk, curr: Pix) -> Result<Pix> {
        Ok(match chunk {
            RGB(r, g, b) => (r, g, b, 255),
            RGBA(r, g, b, a) => (r, g, b, a),
            INDEX(i) => self.index[i as usize],
            DIFF(dr, dg, db) => {
                let (r, g, b, a) = curr;
                let r = ((r as i32) + (dr as i32) - 2) % 255;
                let g = ((g as i32) + (dg as i32) - 2) % 255;
                let b = ((b as i32) + (db as i32) - 2) % 255;
                let r = r as u8;
                let g = g as u8;
                let b = b as u8;
                (r, g, b, a)
            },
            LUMA(dg, drdg, dbdg) => {
                let (r, g, b, a) = curr;
                let vg = (dg as i32) - 32;
                let r = ((r as i32) + vg + (drdg as i32) - 8) % 255;
                let g = ((g as i32) + vg) % 255;
                let b = ((b as i32) + vg + (dbdg as i32) - 8) % 255;
                let r = r as u8;
                let g = g as u8;
                let b = b as u8;
                (r, g, b, a)
            }
            _ => curr, 
        })
    }

    pub fn go(&mut self) -> Result<DecodeOutput> {
        let width = self.header.width;
        let height = self.header.height;
        let num_chunks = width*height;

        let mut bytes: Vec<Pix> = Vec::with_capacity(num_chunks as usize);
            
        let mut run = 0;
        let mut curr = (0u8, 0u8, 0u8, 255u8);
        for _i in 0..num_chunks {


            if run > 0 {
                run -= 1;
            } else {
                let chunk = self.next_chunk()?;
                match chunk {
                    RUN(r) => { run = r; },
                    _ => curr = self.next_px(chunk, curr)?,
                };
                self.index[hash(curr)] = curr;
            }

            bytes.push(curr);

        }
        println!("Length: {}", bytes.len());
        Ok(DecodeOutput {
            width,
            height,
            bytes,
        })
    }

    pub fn debug(&mut self) {}
}
