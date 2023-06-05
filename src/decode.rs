use std::num::Wrapping as wr;

use super::hash;
use super::ChunkIter;
use super::Chunk;
use super::Chunk::*;
use super::Pix;

pub struct Decoder<I>
where 
    I: Iterator<Item = Chunk>
{
    iter: I,
    curr: Pix,
    index: [Pix; 64],
    pixels: Vec<Pix>,
    top: usize,
}
impl<I> Decoder<I>
where
    I: Iterator<Item = Chunk>
{
    pub fn new(iter: I) -> Self {
        let curr = (0, 0, 0, 255);
        let index = [(0, 0, 0, 0); 64];
        let pixels = Vec::new();
        let top = 0;

        Self {
            iter,
            curr,
            index,
            pixels,
            top,
        }
    }

    fn decode_next_chunk(&mut self) -> Option<Chunk> {
        if let Some(chunk) = self.iter.next() {
            self.curr = match chunk {
                RGB(r, g, b) => (r, g, b, 255),
                RGBA(r, g, b, a) =>(r, g, b, a), 
                INDEX(i) => self.index[i as usize],
                DIFF(dr, dg, db) => (
                    self.curr.0
                        .wrapping_add(dr)
                        .wrapping_sub(2),
                    self.curr.1
                        .wrapping_add(dg)
                        .wrapping_sub(2),
                    self.curr.2
                        .wrapping_add(db)
                        .wrapping_sub(2),
                    self.curr.3,
                ),
                LUMA(dg, drdg, dbdg) => (
                    self.curr.0
                        .wrapping_add(dg)
                        .wrapping_sub(8)
                        .wrapping_add(drdg),
                    self.curr.1
                        .wrapping_add(dg),
                    self.curr.2
                        .wrapping_add(dg)
                        .wrapping_sub(8)
                        .wrapping_add(dbdg),
                    self.curr.3,
                ),
                RUN(_) => self.curr, 
            };
            let r = match chunk {
                RUN(r) => r,
                _ => 0,
            };
            self.index[hash(self.curr) as usize] = self.curr;
            for _ in 0..=r {
                self.pixels.push(self.curr);
            }
            Some(chunk)
        } else {
            None
        }
    }
}

impl<I> Iterator for Decoder<I> 
where 
    I: Iterator<Item = Chunk>
{
    type Item = Pix;
    
    fn next(&mut self) -> Option<Pix> {
        if self.top >= self.pixels.len() {
            if let None = self.decode_next_chunk() {
                return None
            }
        }
        self.top += 1;
        Some(self.pixels[self.top - 1])
    }
}

/*
pub fn decode_debug(f: &mut File, width: u32, height: u32) -> Result<Vec<u8>> {
    let num_pixels = width*height;
    let num_bytes = num_pixels as usize * 4;

    let metadata = f.metadata()?;
    let chunks_len = metadata.len() - 22;
    f.seek(SeekFrom::Start(14))?;

    let reader = BufReader::new(f).take(chunks_len);
    let chunks = ChunkIterator::new(reader.bytes().map(|b| b.unwrap()));

    let mut bytes: Vec<u8> = Vec::with_capacity(num_bytes);
    for chunk in chunks {
        let curr = match chunk {
            RGB(..) => (255, 0, 0, 255),
            RGBA(..) =>(255, 0, 0, 255), 
            INDEX(_) => (0, 0, 0, 255),
            DIFF(..) => (0, 0, 255, 255),
            LUMA(..) => (0, 255, 0, 255),
            RUN(_) => (255, 255, 255, 255), 
        };
        let r = match chunk {
            RUN(r) => r,
            _ => 0,
        };

        for _ in 0..=r {
            bytes.push(curr.3);
            bytes.push(curr.2);
            bytes.push(curr.1);
            bytes.push(curr.0);
        }
    }

    Ok(bytes)
}

*/
