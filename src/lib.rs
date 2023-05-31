pub mod encode;
pub mod decode;
pub mod qoi;

pub type Pix = (u8, u8, u8, u8);
pub fn hash(pix: Pix) -> usize {
    let (r, g, b, a) = pix;
    let (r, g, b, a) = (r as usize, g as usize, b as usize, a as usize);
    ((r*3 + g*5 + b*7 + a*11) % 64) as usize
} 
pub enum Chunk {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
    INDEX(u8),
    DIFF(u8, u8, u8),
    LUMA(u8, u8, u8),
    RUN(u8),
}
