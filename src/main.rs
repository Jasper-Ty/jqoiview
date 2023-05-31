use std::env;
use std::fs::File;
use std::io::Read;

use quite_ok_image::qoi::QoiHeader;
use quite_ok_image::decode::Decode;
use quite_ok_image::decode::DecodeOutput;

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
    
    println!("{:?}", header);
    let mut decode = Decode::new(header, f);

    let decode_output = decode.go().unwrap();

    sdl(decode_output).unwrap();
}

fn sdl(decode_output: DecodeOutput) -> Result<(), String> {
    let DecodeOutput { width, height, bytes } = decode_output;

    let mut irene = Vec::<u8>::with_capacity(bytes.len() * 4);
    for byte in bytes {
        irene.push(byte.3);
        irene.push(byte.2);
        irene.push(byte.1);
        irene.push(byte.0);
    }
    
    let sdl_context = sdl2::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Jasper's QOI Image Viewer", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let surface = Surface::from_data(
        &mut irene,
        width, 
        height, 
        width*4,
        RGBA8888
    ).unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let texture = surface.as_texture(&texture_creator).unwrap();


    canvas.copy(
        &texture,
        None,
        None,
    )?;
    canvas.present();


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
        canvas.clear();
        canvas.copy(
            &texture,
            None,
            None,
        )?;
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
    }
    Ok(())
}

