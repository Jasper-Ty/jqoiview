use std::env;
use std::error;
use std::fs::File;
use std::io::{
    BufReader,
    Read,
    Seek,
    SeekFrom,
};

use jqoiview::Header;
use jqoiview::Chunks;
use jqoiview::Pix;
use jqoiview::hash;

use sdl2::rect::Point;
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

fn main() -> Result<(), Box<dyn error::Error>> {

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

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let Header { width, height, .. } = Header::from_file(&mut f)?;
    let mut pixels = decode_qoi_file(&mut f)?;

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

    let mut zoom_level: u32 = 0;

    let mut img_rect = Rect::new(0, 0, width as u32, height as u32);

    draw(&mut canvas, &texture, img_rect);

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
            } 
            | Event::KeyDown {
                keycode: Some(Keycode::K),
                ..
            } => { 
                img_rect.set_y(img_rect.y()+16);
            },
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } 
            | Event::KeyDown {
                keycode: Some(Keycode::J),
                ..
            } => { 
                img_rect.set_y(img_rect.y()-16);
            },
            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            }
            | Event::KeyDown {
                keycode: Some(Keycode::H),
                ..
            } => { 
                img_rect.set_x(img_rect.x()+16);
            },
            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            }
            | Event::KeyDown {
                keycode: Some(Keycode::L),
                ..
            } => { 
                img_rect.set_x(img_rect.x()-16);
            },
            Event::KeyDown {
                keycode: Some(Keycode::I),
                ..
            } => { 
                zoom_level += 1;
            },
            Event::KeyDown {
                keycode: Some(Keycode::O),
                ..
            } => { 
                zoom_level = if zoom_level > 0 { zoom_level - 1 } else { zoom_level };
            },
            Event::Window { 
                win_event: WindowEvent::Resized(w, h),
                ..
            } => { 
            }
            _ => {}
        };
        println!("zoom: {}", zoom_level);
        draw(&mut canvas, &texture, img_rect);
    }
    Ok(())
}

use sdl2::render::Canvas;
use sdl2::render::Texture;
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

fn decode_qoi_file(f: &mut File) -> std::io::Result<Vec<u8>> {
    let Header { width, height, .. } = Header::from_file(f)?;
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

    Ok(pixels)
}

fn draw<R>(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    rect: R,
) -> Result<(), String> 
where
    R: Into<Option<Rect>>
{
    draw_checkered_background(canvas)?;

    let (width, height) = canvas.window().size();
    let center = (width/2, height/2);

    canvas.copy(
        texture,
        None,
        rect,
    )?;
    canvas.present();
    Ok(())
}
