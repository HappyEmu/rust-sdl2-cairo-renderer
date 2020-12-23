use std::marker::{Send, Sync};
use std::ops::{Deref, DerefMut};

// Wrap canvas so that we can send it across threads (synchronized with RwLock)
pub struct SendCanvas(pub sdl2::render::WindowCanvas);

unsafe impl Send for SendCanvas {}
unsafe impl Sync for SendCanvas {}

impl Deref for SendCanvas {
    type Target = sdl2::render::WindowCanvas;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SendCanvas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
