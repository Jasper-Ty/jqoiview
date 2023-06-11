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
    Header,
    Chunks,
    Pix,
    hash,
};

use sdl2::surface::Surface;
use sdl2::keyboard::Keycode;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::pixels::{
    Color,
    PixelFormatEnum,
};
use sdl2::rect::Rect;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn error::Error>>{
    let args: Vec<String> = env::args().collect();
    let filepath = match args
        .get(1)
        .map(|s| s.as_str()) 
    {
        None 
        | Some("-h") 
        | Some("--help") => {
            println!("Usage: jqoiview <file>");
            return Ok(())
        },
        Some("-v") 
        | Some("--version") => {
            println!("jqoiview v{}", VERSION);
            return Ok(())
        },
        Some(arg) => arg,
    };

    let mut f = File::open(filepath)?;
    let Header { width, height, .. } = Header::from_file(&mut f)?;
    let metadata = f.metadata()?;

    let chunks_len = metadata.len() - 22;
    f.seek(SeekFrom::Start(Header::SIZE))?;
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
        (curr, run) = chunk.parse(curr, &index);
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
        PixelFormatEnum::RGBA8888,
    )?;

    let window = video_subsystem
        .window("jqoiview", width, height)
        .resizable()
        .position_centered()
        .opengl()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();
    let texture = surface.as_texture(&texture_creator)?;

    draw_checkered_background(&mut canvas)?;

    canvas.copy(
        &texture,
        None,
        None,
    )?;
    canvas.present();

    let mut zoom_level:i32 = 0;
    let mut dx: i32 = 0;
    let mut dy: i32 = 0;
    let mut view_x: i32 = 0;
    let mut view_y: i32 = 0;
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
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => { 
                view_y += 16;
            },
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => { 
                view_y -= 16;
            },
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => { 
                view_x += 16;
            },
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => { 
                view_x -= 16;
            },
            Event::Window { 
                win_event: WindowEvent::Resized(w, h),
                ..
            } => { 
                dx = (w as i32 - width as i32) / 2;
                dy = (h as i32 - height as i32) / 2;
            }
            _ => {}
        };
        draw_checkered_background(&mut canvas)?;
        canvas.copy(
            &texture,
            None,
            Rect::new(dx+view_x, dy+view_y, width as u32, height as u32),
        )?;
        canvas.present();
    }
    Ok(())
}

use sdl2::render::Canvas;
use sdl2::video::Window;

fn draw_checkered_background(
    canvas: &mut Canvas<Window>
) -> Result<(), String> {
    let (width, height) = canvas.window().size();

    let white = Color::RGB(255, 255, 255);
    let gray = Color::RGB(223, 223, 223);

    for i in 0..=width / 24 {
        for j in 0..=height / 24 {
            let (x, y) = (i as i32 * 24, j as i32 * 24);
            if (i+j) % 2 == 0 {
                canvas.set_draw_color(white);
            } else {
                canvas.set_draw_color(gray);
            }
            canvas.fill_rect(Rect::new(x, y, 24, 24))?;
        }
    }
    Ok(())
}
