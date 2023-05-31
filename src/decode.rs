use std::fs::File;
use std::io::{ Read, Result, Error, ErrorKind };
use std::num::Wrapping as wr;

use super::hash;
use super::Pix;
use super::ChunkIterator;
use super::Chunk;
use super::Chunk::*;
use super::qoi::QoiHeader;

pub fn decode(f: &mut File, width: u32, height: u32) -> Result<Vec<u8>> {
    let num_bytes = (width*height) as usize * 4;

    let mut bytes: Vec<u8> = Vec::with_capacity(num_bytes);
        
    let mut curr = (0u8, 0u8, 0u8, 255u8);
    let mut index = [(0u8, 0u8, 0u8, 0u8); 64];

    let iter = f.bytes()
        .map(|b| b.unwrap());
    let chunks = ChunkIterator::new(iter);

    let mut i = 0;
    for chunk in chunks {
        match chunk {
            RGB(r, g, b) => curr = (r, g, b, 255),
            RGBA(r, g, b, a) => curr = (r, g, b, a), 
            INDEX(i) => curr = index[i as usize], 
            DIFF(dr, dg, db) => curr = (
                (wr(curr.0) + wr(dr) - wr(2)).0,
                (wr(curr.1) + wr(dg) - wr(2)).0,
                (wr(curr.2) + wr(db) - wr(2)).0,
                curr.3,
            ),
            LUMA(dg, drdg, dbdg) => {
                let vg = wr(dg) - wr(32);
                curr = (
                    (wr(curr.0) + vg - wr(8) + wr(drdg)).0,
                    (wr(curr.1) + vg).0,
                    (wr(curr.2) + vg - wr(8) + wr(dbdg)).0,
                    curr.3,
                );
            },
            RUN(r) => for _ in 1..=r {
                bytes.push(curr.3);
                bytes.push(curr.2);
                bytes.push(curr.1);
                bytes.push(curr.0);
            },
        };
        index[hash(curr)] = curr;
        bytes.push(curr.3);
        bytes.push(curr.2);
        bytes.push(curr.1);
        bytes.push(curr.0);
    }

    Ok(bytes)
}
