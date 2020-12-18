extern crate sdl2;

use std::fmt::Write;

use sdl2::event::{Event, EventType};
use sdl2::keyboard::Keycode;
use sdl2::pixels;

use sdl2::gfx::primitives::DrawRenderer;
use std::time::Duration;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::pixels::{Color, PixelFormat, PixelFormatEnum};
use sdl2::video::SwapInterval;
use cairo::{ImageSurface, Format, Context};
use std::fs::File;
use std::f64::consts::PI;
use rand::random;
use sdl2::rect::Rect;
use sdl2::mouse::SystemCursor::No;

const SCREEN_SIZE: (u32, u32) = (1280, 720);

mod colors {
    use sdl2::pixels::Color;

    pub const WHITE: Color = Color::RGB(255, 255, 255);
}
const CLEAR_COLOR: pixels::Color = pixels::Color::RGB(0, 64, 148);

fn main() -> Result<(), String> {
    let mut surface = ImageSurface::create(Format::ARgb32, SCREEN_SIZE.0 as i32, SCREEN_SIZE.1 as i32)
        .expect("couldn’t create a surface, yo");

    let cairo = Context::new(&surface);

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
        .window(
            "rust-sdl2_gfx: draw line & FPSManager",
            SCREEN_SIZE.0,
            SCREEN_SIZE.1,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().present_vsync().build().map_err(|e| e.to_string())?;

    let mut events = sdl_context.event_pump()?;
    events.disable_event(EventType::MouseMotion);
    events.disable_event(EventType::KeyDown);
    events.disable_event(EventType::KeyUp);
    events.disable_event(EventType::TextInput);

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, SCREEN_SIZE.0, SCREEN_SIZE.1)
        .unwrap();

    let mut time = std::time::Instant::now();
    let mut fps_str = String::with_capacity(8);

    'main: loop {
        for event in events.poll_iter() {
            println!("{:?}", event);
            match event {
                Event::Quit { .. } => break 'main,
                _ => {}
            }
        };

        let mouse = events.mouse_state();
        let m_pos = (mouse.x(), mouse.y());

        {
            let frame_time = (std::time::Instant::now() - time).as_millis();
            time = std::time::Instant::now();
            fps_str.clear();
            write!(fps_str, "{}ms", frame_time);
        }

        cairo.set_source_rgb(0.0, 0.3, 0.6);
        cairo.paint();
        cairo.set_line_width(1.0);
        cairo.set_source_rgb(1.0, 1.0, 0.0);
        cairo.arc(m_pos.0 as f64, m_pos.1 as f64, 50., 0., 2. * PI);
        cairo.stroke();
        cairo.set_font_size(20.0);
        cairo.move_to(10.0, 30.0);
        cairo.show_text(&fps_str);
        cairo.stroke();

        // Copy cairo buffer to SDL texture
        texture.with_lock(None, |tex: &mut [u8], pitch: usize| {
            surface.with_data(|surf: &[u8]| {
                tex.copy_from_slice(&surf);
            });
        })?;

        // Copy SDL texture to frame buffer
        // TODO: Possible to directly copy Cairo buffer to frame buffer?
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        std::thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

#[inline]
fn clear_canvas<T: RenderTarget>(canvas: &mut Canvas<T>, color: pixels::Color) {
    canvas.set_draw_color(color);
    canvas.clear();
}