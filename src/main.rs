mod camera;
mod mesh;
mod viewport;

use crate::camera::Camera;
use crate::mesh::Mesh;

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
use crate::viewport::Viewport;

const SCREEN_SIZE: (u32, u32) = (1280, 720);
const CLEAR_COLOR: pixels::Color = pixels::Color::RGB(0, 64, 148);

fn main() -> Result<(), String> {
    let mesh = Mesh::new(
        vec![
            vec3(-0.5, -0.5, -0.5),
            vec3(0.5, -0.5, -0.5),
            vec3(0.5, -0.5, 0.5),
            vec3(-0.5, -0.5, 0.5),
            vec3(-0.5, 0.5, -0.5),
            vec3(0.5, 0.5, -0.5),
            vec3(0.5, 0.5, 0.5),
            vec3(-0.5, 0.5, 0.5)
        ], vec![
            3, 7, 6, 6, 2, 3,
            2, 6, 5, 5, 1, 2,
            1, 5, 4, 4, 0, 1,
            0, 4, 7, 7, 3, 0,
            7, 4, 5, 5, 6, 7,
            0, 3, 2, 2, 1, 0
        ]);

    let viewport = Viewport::new(SCREEN_SIZE.0 as u16, SCREEN_SIZE.1 as u16);

    let font = cairo::FontFace::toy_create(
        "Menlo",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal
    );

    let mut surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        SCREEN_SIZE.0 as i32,
        SCREEN_SIZE.1 as i32
    ).expect("couldn’t create a surface, yo");

    let cairo = cairo::Context::new(&surface);

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
        .window("Bare Metal Software Renderer 0.0.1",
                SCREEN_SIZE.0,
                SCREEN_SIZE.1)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // Setup event pump, disable some events
    let mut events = sdl_context.event_pump()?;
    events.disable_event(EventType::MouseMotion);
    events.disable_event(EventType::KeyDown);
    events.disable_event(EventType::KeyUp);
    events.disable_event(EventType::TextInput);

    // Create texture to draw to using cairo surface
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, SCREEN_SIZE.0, SCREEN_SIZE.1)
        .unwrap();

    let mut start = std::time::Instant::now();
    let mut time = start.clone();
    let mut fps_str = String::with_capacity(16);

    let mut camera = Camera::new(vec3i(0, 5, 8), vec3i(0, 0, 0));

    'main: loop {
        // Poll event loop
        for event in events.poll_iter() {
            println!("{:?}", event);
            match event {
                Event::Quit { .. } => break 'main,
                _ => {}
            }
        };

        // Get keyboard and mouse state
        let mouse = events.mouse_state();
        let kb = events.keyboard_state();

        let mouse_pos = (mouse.x(), mouse.y());

        // Figure out frame timing
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

        // Compute transformation matrix
        let m = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::one() * 2.0,
            glam::Quat::from_rotation_ypr(rot, 0.0, 0.0),
            glam::Vec3::new(0., 0., 0.)
        );

        let v = camera.view_matrix();
        let p = glam::Mat4::perspective_rh(
            45.0f32.to_radians(),
            SCREEN_SIZE.0 as f32 / SCREEN_SIZE.1 as f32,
            0.1, 100.0
        );

        let pvm = p * v * m;

        // Clear texture
        cairo.set_source_rgb(0.0, 0.3, 0.6);
        cairo.paint();

        // Draw mouse "pointer"
        {
            let lw = cairo.get_line_width();
            cairo.set_line_width(4.0);
            cairo.set_source_rgb(1.0, 0.7, 0.0);
            cairo.arc(mouse_pos.0 as f64, mouse_pos.1 as f64, 20., 0., 2. * PI);
            cairo.stroke();
            cairo.set_line_width(lw);
        }

        // Draw fps text
        cairo.set_font_size(20.0);
        cairo.set_font_face(&font);
        cairo.move_to(10.0, 30.0);
        cairo.show_text(&fps_str);
        cairo.stroke();

        // TODO: (Perf) Possible to directly copy Cairo buffer to frame buffer, bypassing texture?
        // Copy cairo buffer to SDL texture
        texture.with_lock(None, |tex: &mut [u8], pitch: usize| {
            surface.with_data(|surf: &[u8]| {
                tex.copy_from_slice(&surf);
            });
        })?;

        // Copy SDL texture to frame buffer
        canvas.copy(&texture, None, None).unwrap();

        // Draw cube
        mesh.draw(&pvm, &mut canvas, &viewport);

        canvas.present();
        clear_canvas(&mut canvas, CLEAR_COLOR);

        // Since we are polling the event cube, yield some time to free up CPU
        std::thread::sleep(Duration::from_millis(4));
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