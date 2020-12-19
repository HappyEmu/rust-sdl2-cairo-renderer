mod camera;

extern crate sdl2;

use std::fmt::Write;
use std::time::Duration;
use std::f64::consts::PI;

use sdl2::event::{Event, EventType};
use sdl2::keyboard::{Keycode, Scancode, KeyboardState};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::video::SwapInterval;
use sdl2::rect::Rect;
use sdl2::pixels::{self, Color, PixelFormat, PixelFormatEnum};

use cairo::{ImageSurface, Context};
use glam::Vec4;
use crate::camera::Camera;

const SCREEN_SIZE: (u32, u32) = (640, 640);

mod colors {
    use sdl2::pixels::Color;

    pub const WHITE: Color = Color::RGB(255, 255, 255);
}
const CLEAR_COLOR: pixels::Color = pixels::Color::RGB(0, 64, 148);

fn main() -> Result<(), String> {
    let cube = vec![
        vec3(-0.5, -0.5, -0.5), vec3(0.5, -0.5, -0.5), vec3(0.5, -0.5, 0.5), vec3(-0.5, -0.5, 0.5),
        vec3(-0.5, 0.5, -0.5), vec3(0.5, 0.5, -0.5), vec3(0.5, 0.5, 0.5), vec3(-0.5, 0.5, 0.5),
    ];

    let font = cairo::FontFace::toy_create("Menlo", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    let mut surface = ImageSurface::create(cairo::Format::ARgb32, SCREEN_SIZE.0 as i32, SCREEN_SIZE.1 as i32)
        .expect("couldn’t create a surface, yo");

    let cairo = Context::new(&surface);

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
        .window(
            "Bare Metal Software Renderer 0.0.1",
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

    let mut start = std::time::Instant::now();
    let mut time = start.clone();
    let mut fps_str = String::with_capacity(16);

    let mut camera = Camera::new(vec3i(0, 0, 8), vec3i(0, 0, 0));

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

        let kb = events.keyboard_state();

        let now = std::time::Instant::now();
        let dt = (now - time).as_secs_f32();
        let elapsed = (now - start).as_secs_f32();

        {
            let frame_time = (std::time::Instant::now() - time).as_millis();
            time = std::time::Instant::now();
            fps_str.clear();
            write!(fps_str, "{}ms ({:.1} fps)", frame_time, 1.0 / dt);
        }

        // Camera control
        update_camera(&mut camera, &kb, dt);

        let rot = 1.0 * elapsed;

        let m = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::one() * 4.0,
            glam::Quat::from_rotation_ypr(rot, 0.0, 0.0),
            glam::Vec3::new(0., 0., 0.)
        );

        let v = camera.view_matrix();
        let p = glam::Mat4::perspective_rh(45.0f32.to_radians(), SCREEN_SIZE.0 as f32 / SCREEN_SIZE.1 as f32, 0.1, 100.0);

        let pvm = p * v * m;

        cairo.set_source_rgb(0.0, 0.3, 0.6);
        cairo.paint();
        {
            let lw = cairo.get_line_width();
            cairo.set_line_width(4.0);
            cairo.set_source_rgb(1.0, 0.7, 0.0);
            cairo.arc(m_pos.0 as f64, m_pos.1 as f64, 20., 0., 2. * PI);
            cairo.stroke();
            cairo.set_line_width(lw);
        }
        cairo.set_font_size(20.0);
        cairo.set_font_face(&font);
        cairo.move_to(10.0, 30.0);
        cairo.show_text(&fps_str);
        cairo.stroke();

        // Draw cube
        let mut prev: Option<(f64, f64)> = None;
        for edge in cube.iter() {
            // Transform to NDC (with perspective division)
            let ndc = pvm * glam::Vec4::new(edge.x, edge.y, edge.z, 1.0);
            let ndc = ndc / ndc.w;
            // println!("{:?}", ndc);

            if ndc.z > 1.0 {
                continue;
            }

            // Convert to screen coordinates
            let x = ((ndc.x + 1.0) * 0.5 * SCREEN_SIZE.0 as f32) as f64;
            let y = ((1.0 - ((ndc.y + 1.0) * 0.5)) * SCREEN_SIZE.1 as f32) as f64;

            cairo.rectangle(x - 4.0, y - 4.0, 8.0, 8.0);
            cairo.fill();

            cairo.move_to(x, y);

            if let Some((px, py)) = prev {
                cairo.line_to(px, py);
                cairo.stroke();
            }

            prev = Some((x, y))
        }

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
fn update_camera(camera: &mut Camera, kb: &KeyboardState, dt: f32) {
    // Translation
    if kb.is_scancode_pressed(Scancode::A) {
        camera.translate(-glam::Vec3::unit_x() * 2.0 * dt);
    }
    if kb.is_scancode_pressed(Scancode::D) {
        camera.translate(glam::Vec3::unit_x() * 2.0 * dt);
    }
    if kb.is_scancode_pressed(Scancode::W) {
        camera.translate(-glam::Vec3::unit_z() * 2.0 * dt);
    }
    if kb.is_scancode_pressed(Scancode::S) {
        camera.translate(glam::Vec3::unit_z() * 2.0 * dt);
    }
    if kb.is_scancode_pressed(Scancode::LShift) {
        camera.translate(glam::Vec3::unit_y() * 2.0 * dt);
    }
    if kb.is_scancode_pressed(Scancode::LCtrl) {
        camera.translate(-glam::Vec3::unit_y() * 2.0 * dt);
    }

    // Rotation
    if kb.is_scancode_pressed(Scancode::Left) {
        camera.yaw(dt);
    }
    if kb.is_scancode_pressed(Scancode::Right) {
        camera.yaw(-dt);
    }
    if kb.is_scancode_pressed(Scancode::Up) {
        camera.pitch(dt);
    }
    if kb.is_scancode_pressed(Scancode::Down) {
        camera.pitch(-dt);
    }
}

fn mat4_look_at(eye: glam::Vec3, center: glam::Vec3, up: glam::Vec3) -> glam::Mat4 {
    let z = (eye - center).normalize();
    let x = up.cross(z).normalize();
    let y = z.cross(x);

    glam::Mat4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        z.extend(0.0),
        eye.extend(1.0)
    )
}

fn vec3<T: Into<f32>>(x: T, y: T, z: T) -> glam::Vec3 {
    glam::Vec3::new(x.into(), y.into(), z.into())
}

fn vec3i(x: i32, y: i32, z: i32) -> glam::Vec3 {
    glam::Vec3::new(x as f32, y as f32, z as f32)
}

#[inline]
fn clear_canvas<T: RenderTarget>(canvas: &mut Canvas<T>, color: pixels::Color) {
    canvas.set_draw_color(color);
    canvas.clear();
}