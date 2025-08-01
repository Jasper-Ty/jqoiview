use std::env;
use std::error;
use std::fs;

use jqoiview::QOIDecode;

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

    let bytes = fs::read(filepath)?;
    let QOIDecode {
        width,
        height,
        pixels,
        ..
    } = QOIDecode::from_bytes(&bytes[..])?;

    let mut data: Vec<u8> = pixels.into_iter()
        .flat_map(|pixel| [pixel.a, pixel.b, pixel.g, pixel.r].into_iter())
        .collect();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let surface = Surface::from_data(
        &mut data,
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

    let mut zoom_level: u32 = 1;
    let mut irene = width;
    let mut seulgi = height;

    let mut view_x = width as i32/2;
    let mut view_y = height as i32/2;

    let mut img_rect = Rect::new(0, 0, width as u32, height as u32);

    draw(&mut canvas, &texture, img_rect)?;

    let mut width = width as i32;
    let mut height = height as i32;
    let mut event_pump = sdl_context.event_pump()?;
    for event in event_pump.wait_iter() {
        match event {
            Event::Quit { .. } => break,
            Event::KeyDown { 
                keycode: code,
                ..
            } => match code {
                Some(Keycode::Escape) 
                | Some(Keycode::Q) => {
                    break
                },
                Some(Keycode::Up) 
                | Some(Keycode::K) => {
                    view_y -= 16;
                },
                Some(Keycode::Down) 
                | Some(Keycode::J) => {
                    view_y += 16;
                },
                Some(Keycode::Left) 
                | Some(Keycode::H) => {
                    view_x -= 16;
                },
                Some(Keycode::Right) 
                | Some(Keycode::L) => {
                    view_x += 16;
                },
                Some(Keycode::Plus) 
                | Some(Keycode::I) => {
                    irene = (zoom_level+1) * (irene/zoom_level) + (irene % zoom_level);
                    seulgi = (zoom_level+1) * (seulgi/zoom_level) + (seulgi% zoom_level);
                    view_x = (zoom_level as i32+1) * (view_x/zoom_level as i32) + (view_x % zoom_level as i32);
                    view_y = (zoom_level as i32+1) * (view_y/zoom_level as i32) + (view_y % zoom_level as i32);
                    zoom_level += 1;
                },
                Some(Keycode::Minus) 
                | Some(Keycode::O) => {
                    if zoom_level > 1 {
                        irene = (zoom_level-1) * (irene/zoom_level) + (irene % zoom_level);
                        seulgi = (zoom_level-1) * (seulgi/zoom_level) + (seulgi% zoom_level);
                        view_x = (zoom_level as i32-1) * (view_x/zoom_level as i32) + (view_x % zoom_level as i32);
                        view_y = (zoom_level as i32-1) * (view_y/zoom_level as i32) + (view_y % zoom_level as i32);
                        zoom_level -= 1;
                    }
                },
                _ => {}
            },
            Event::Window { 
                win_event: WindowEvent::Resized(w, h),
                ..
            } => { 
                width = w;
                height = h;
            },
            _ => {}
        };

        let window = canvas.window_mut();
        let title = format!("jqoiview - {}x zoom", zoom_level);

        window.set_title(&title)?;
        img_rect.set_x(-((view_x - (width as i32/2)) as i32));
        img_rect.set_y(-((view_y - (height as i32/2)) as i32));
        img_rect.set_width(irene);
        img_rect.set_height(seulgi);
        draw(&mut canvas, &texture, img_rect)?;
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

use sdl2::render::Texture;

fn draw<R>(
    canvas: &mut Canvas<Window>,
    texture: &Texture,
    rect: R,
) -> Result<(), String> 
where
    R: Into<Option<Rect>>
{
    draw_checkered_background(canvas)?;

    canvas.copy(
        texture,
        None,
        rect,
    )?;
    canvas.present();
    Ok(())
}
