use std::{
    env,
    error,
    fs::File,
    io::{
        BufReader,
        Read,
        Seek,
        SeekFrom,
    }
};

use jqoiview::{
    Chunk::*,
    Header,
    Chunks,
    Pix,
    hash,
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{ PixelFormatEnum::RGBA8888 },
    surface::Surface,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn error::Error>>{
    let args: Vec<String> = env::args().collect();

    let filepath = match args
        .get(1)
        .map(|s| s.as_str()) 
    {
        None | Some("-h") => {
            println!("Usage: jqoiview <file>");
            return Ok(())
        },
        Some("-v") => {
            println!("jqoiview v{}", VERSION);
            return Ok(())
        },
        Some(arg) => arg,
    };

    let mut f = File::open(filepath)?;
    let Header { width, height, .. } = Header::from_file(&mut f)?;
    
    let metadata = f.metadata()?;
    let chunks_len = metadata.len() - 22;
    f.seek(SeekFrom::Start(14))?;

    let chunks = BufReader::new(f)
        .take(chunks_len)
        .bytes()
        .map(|b| b.unwrap())
        .chunks();

    let mut curr: Pix = (0, 0, 0, 255);
    let mut run;
    let mut index = [(0, 0, 0, 0); 64];
    let mut pixels: Vec<u8> = Vec::with_capacity((4*width*height) as usize);

    for chunk in chunks {
        (curr, run) = match chunk {
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
        };
        index[hash(curr)] = curr;
        for _ in 0..=run {
            pixels.push(curr.3);
            pixels.push(curr.2);
            pixels.push(curr.1);
            pixels.push(curr.0);
        }
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let surface = Surface::from_data(
        &mut pixels,
        width,
        height,
        width*4,
        RGBA8888,
    )?;

    let window = video_subsystem
        .window("Jasper's QOI Image Viewer", width, height)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let texture = surface.as_texture(&texture_creator)?;

    canvas.copy(
        &texture,
        None,
        None,
    )?;
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    for event in event_pump.wait_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } 
            | Event::KeyDown {
                keycode: Some(Keycode::Q),
                ..
            } => break,
            _ => {}
        }
    }
    Ok(())
}
