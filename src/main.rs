use std::env;
use std::fs::File;
use std::io::Read;

use quite_ok_image::qoi::QoiHeader;
use quite_ok_image::decode::{decode, decode_debug};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{ PixelFormatEnum::RGBA8888 },
    surface::Surface,
};
use std::time::Duration;

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
    let mut bytes = decode(&mut f, width, height).unwrap();
    println!("output size: {}", bytes.len());

    let mut debug = decode_debug(&mut f, width, height).unwrap();

    let surface = Surface::from_data(
        &mut bytes,
        width, 
        height, 
        width*4,
        RGBA8888
    ).unwrap();

    let surface_debug = Surface::from_data(
        &mut debug,
        width, 
        height, 
        width*4,
        RGBA8888
    ).unwrap();
    sdl(surface, surface_debug).unwrap();
}


fn sdl(surface: Surface, surface2: Surface) -> Result<(), String> {
    
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

