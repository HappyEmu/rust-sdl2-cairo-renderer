pub struct Viewport {
    pub dim: (u16, u16),
    pub mat: glam::Mat4
}

impl Viewport {
    pub fn new(width: u16, height: u16) -> Self {
        let m = glam::Mat4::from_cols_array_2d(&[
            [width as f32 / 2.0, 0.0, 0.0, 0.0],
            [0.0, -(height as f32)  / 2.0, 0.0, 0.0],
            [0.0, 0.0, 0.5, 0.0],
            [width as f32 / 2.0, height as f32  / 2.0, 0.5, 1.0]]
        );

        Self {
            dim: (width, height),
            mat: m
        }
    }
}