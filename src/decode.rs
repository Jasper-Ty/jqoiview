use std::fs::File;
use std::io::{ BufReader, Read, Result, Seek, SeekFrom };
use std::num::Wrapping as wr;

use super::hash;
use super::ChunkIterator;
use super::Chunk::*;

pub fn decode(f: &mut File, width: u32, height: u32) -> Result<Vec<u8>> {
    let num_pixels = width*height;
    let num_bytes = num_pixels as usize * 4;

    let metadata = f.metadata()?;
    let chunks_len = metadata.len() - 22;
    f.seek(SeekFrom::Start(14))?;

    let reader = BufReader::new(f).take(chunks_len);
    let chunks = ChunkIterator::new(reader.bytes().map(|b| b.unwrap()));

    let mut bytes: Vec<u8> = Vec::with_capacity(num_bytes);
    let mut curr = (0u8, 0u8, 0u8, 255u8);
    let mut index = [(0u8, 0u8, 0u8, 0u8); 64];
    for chunk in chunks {
        curr = match chunk {
            RGB(r, g, b) => (r, g, b, 255),
            RGBA(r, g, b, a) => (r, g, b, a), 
            INDEX(i) => index[i as usize], 
            DIFF(dr, dg, db) => (
                curr.0.wrapping_add(dr).wrapping_sub(2),
                curr.1.wrapping_add(dg).wrapping_sub(2),
                curr.2.wrapping_add(db).wrapping_sub(2),
                curr.3,
            ),
            LUMA(dg, drdg, dbdg) => {
                let vg = wr(dg) - wr(32);
                (
                    (wr(curr.0) + vg - wr(8) + wr(drdg)).0,
                    (wr(curr.1) + vg).0,
                    (wr(curr.2) + vg - wr(8) + wr(dbdg)).0,
                    curr.3,
                )
            },
            RUN(_) => curr,
        };
        let r = match chunk {
            RUN(r) => r,
            _ => 0,
        };

        index[hash(curr) as usize] = curr;
        for _ in 0..=r {
            bytes.push(curr.3);
            bytes.push(curr.2);
            bytes.push(curr.1);
            bytes.push(curr.0);
        }
    }

    Ok(bytes)
}

pub fn print_chunks(f: &mut File) -> Result<()> {
    let metadata = f.metadata()?;
    let chunks_len = metadata.len() - 22;
    f.seek(SeekFrom::Start(14))?;

    let reader = BufReader::new(f).take(chunks_len);
    let chunks = ChunkIterator::new(reader.bytes().map(|b| b.unwrap()));
    
    for _chunk in chunks {
        println!("wooh");
    }
    Ok(())
}

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
