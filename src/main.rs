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

use quite_ok_image::{
    qoi::QoiHeader,
    decode::Decoder,
    BytesToChunks,
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{ PixelFormatEnum::RGBA8888 },
    surface::Surface,
};

fn main() -> Result<(), Box<dyn error::Error>>{
    let args: Vec<String> = env::args().collect();

    let filepath = args.get(1)
        .expect("Should have file argument");

    let mut f = File::open(filepath)?;
       

    let mut buf: [u8; 4] = [0u8; 4];
    f.read(&mut buf)?;

    assert_eq!(buf, [113u8, 111u8, 105u8, 102u8]);

    let header = QoiHeader::from_file(&mut f)?;

    let width = header.width;
    let height = header.height;
    
    println!("{:?}", header);

    let metadata = f.metadata()?;
    let chunks_len = metadata.len() - 22;
    f.seek(SeekFrom::Start(14))?;

    let chunks = BufReader::new(f)
        .take(chunks_len)
        .bytes()
        .map(|b| b.unwrap())
        .chunks();

    let mut decode = Decoder::new(chunks);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut surface = Surface::new(
        width,
        height,
        RGBA8888,
    ).unwrap();

    let window = video_subsystem
        .window("Jasper's QOI Image Viewer", width, height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut i = 0;


    surface.with_lock_mut(|v| {
        for _j in 0..(width*height) as usize {
            if let Some(tracked) = decode.next() {
                let p = tracked.pix;
                v[i] = p.3;
                v[i+1] = p.2;
                v[i+2] = p.1;
                v[i+3] = p.0;
                i += 4;
            } else {
                break;
            }
        }
    });
    let texture = surface.as_texture(&texture_creator).unwrap();

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
