use super::hash;
use super::Chunk;
use super::Chunk::*;
use super::Pix;

#[derive(Debug, Copy, Clone)]
pub struct TrackedPix {
    pub pix: Pix,
    pub from: Chunk,
}

pub struct Decoder<I>
where 
    I: Iterator<Item = Chunk>
{
    iter: I,
    curr: Pix,
    pub index: [Pix; 64],
    pixels: Vec<TrackedPix>,
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
                RGB(r, g, b) => (r, g, b, self.curr.3),
                RGBA(r, g, b, a) => (r, g, b, a), 
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
                        .wrapping_sub(40)
                        .wrapping_add(drdg),
                    self.curr.1
                        .wrapping_add(dg)
                        .wrapping_sub(32),
                    self.curr.2
                        .wrapping_add(dg)
                        .wrapping_sub(40)
                        .wrapping_add(dbdg),
                    self.curr.3,
                ),
                RUN(_) => self.curr, 
            };
            let r = match chunk {
                RUN(r) => r,
                _ => 0,
            };
            for _ in 0..=r {
                self.pixels.push(TrackedPix {
                    pix: self.curr,
                    from: chunk,
                });
            }
            self.index[hash(self.curr) as usize] = self.curr;
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
    type Item = TrackedPix;
    
    fn next(&mut self) -> Option<TrackedPix> {
        if self.top >= self.pixels.len() {
            if let None = self.decode_next_chunk() {
                return None
            }
        }
        self.top += 1;
        Some(self.pixels[self.top - 1])
    }
}

