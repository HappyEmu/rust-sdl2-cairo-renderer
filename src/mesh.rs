pub struct Mesh {
    vertices: Vec<glam::Vec3>,
    indices: Vec<u32>,
    transform: glam::Mat4
}

impl Mesh {
    pub fn new(vertices: Vec<glam::Vec3>, indices: Vec<u32>) -> Self {
        Mesh { vertices, indices, transform: glam::Mat4::identity() }
    }

    pub fn draw(&self, mvp: &glam::Mat4, cairo: &cairo::Context, screen: (u32, u32)) {
        // Draw cube
        let mut prev: Option<(f64, f64)> = None;
        for edge in (&self.vertices).iter() {
            // Transform to NDC (with perspective division)
            let ndc = *mvp * glam::Vec4::new(edge.x, edge.y, edge.z, 1.0);
            let ndc = ndc / ndc.w;
            // println!("{:?}", ndc);

            if ndc.z > 1.0 {
                continue;
            }

            // Convert to screen coordinates
            let x = ((ndc.x + 1.0) * 0.5 * screen.0 as f32) as f64;
            let y = ((1.0 - ((ndc.y + 1.0) * 0.5)) * screen.1 as f32) as f64;

            cairo.rectangle(x - 4.0, y - 4.0, 8.0, 8.0);
            cairo.fill();

            cairo.move_to(x, y);

            if let Some((px, py)) = prev {
                cairo.line_to(px, py);
                cairo.stroke();
            }

            prev = Some((x, y))
        }
    }
}