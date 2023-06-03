use std::env;
use std::fs::File;
use std::io::{ BufReader, Read, Seek, SeekFrom };

use quite_ok_image::qoi::QoiHeader;
use quite_ok_image::decode::Decoder;
use quite_ok_image::ChunkIter;
use quite_ok_image::Chunk::*;
use quite_ok_image::hash;
use quite_ok_image::BytesToChunks;

use std::num::Wrapping as wr;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{ PixelFormatEnum::RGBA8888 },
    surface::Surface,
};
use std::time::Duration;

const SKIP: usize = 400;

fn main() {
    let args: Vec<String> = env::args().collect();

    let filepath = args.get(1)
        .expect("Should have file argument");

    let mut f = File::open(filepath)
        .expect("Should be able to open file");

    let mut buf: [u8; 4] = [0u8; 4];
    f.read(&mut buf)
        .expect("Should be able to read file.");

    assert_eq!(buf, [113u8, 111u8, 105u8, 102u8]);

    let header = QoiHeader::from_file(&mut f)
        .expect("Should be able to read header");

    let width = header.width;
    let height = header.height;
    
    println!("{:?}", header);

    let metadata = f.metadata().unwrap();
    let chunks_len = metadata.len() - 22;
    f.seek(SeekFrom::Start(14)).unwrap();

    let metadata = f.metadata().unwrap();
    let chunks_len = metadata.len() - 22;
    f.seek(SeekFrom::Start(14)).unwrap();

    let chunks = BufReader::new(f)
        .take(chunks_len)
        .bytes()
        .map(|b| b.unwrap())
        .chunks();


}


fn sdl(surface: &mut Surface, surface2: Surface) -> Result<(), String> {
    
    let sdl_context = sdl2::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Jasper's QOI Image Viewer", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let texture = surface.as_texture(&texture_creator).unwrap();

    let window_2 = video_subsystem
        .window("Jasper's QOI Image Viewer", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas_2 = window_2.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator_2 = canvas_2.texture_creator();
    let texture_2 = surface2.as_texture(&texture_creator_2).unwrap();

    canvas.copy(
        &texture,
        None,
        None,
    )?;
    canvas.present();

    canvas_2.copy(
        &texture_2,
        None,
        None,
    )?;
    canvas_2.present();

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => { 
                    surface.with_lock_mut(|v| {
                        println!("TRAPPED IN DA SURFACE: {}", v.len())
                    });
                }
                _ => {}
            }
        }
        canvas.copy(
            &texture,
            None,
            None,
        )?;
        canvas.present();
        canvas_2.copy(
            &texture_2,
            None,
            None,
        )?;
    canvas_2.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
    Ok(())
}
