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
