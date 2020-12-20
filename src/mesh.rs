use crate::viewport::Viewport;

pub struct Mesh {
    vertices: Vec<glam::Vec3>,
    colors: Vec<glam::Vec3>,
    indices: Vec<u32>,
    transform: glam::Mat4
}

impl Mesh {
    pub fn new(vertices: Vec<glam::Vec3>, indices: Vec<u32>) -> Self {
        let colors = vertices.iter().map(|_|
            glam::Vec3::new(rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>())
        ).collect();

        Mesh {
            vertices,
            indices,
            colors,
            transform: glam::Mat4::identity()
        }
    }

    pub fn draw(&self, mvp: &glam::Mat4, canvas: &mut sdl2::render::WindowCanvas, vp: &Viewport) {
        let t = vp.mat * *mvp;

        for triangle_indices in self.indices.chunks(3) {
            let v0 = t * self.vertices[triangle_indices[0] as usize].extend(1.0);
            let v1 = t * self.vertices[triangle_indices[1] as usize].extend(1.0);
            let v2 = t * self.vertices[triangle_indices[2] as usize].extend(1.0);

            let c = self.colors[triangle_indices[0] as usize];
            canvas.set_draw_color(sdl2::pixels::Color::RGB(
                (c.x * 255.0) as u8,
                (c.y * 255.0) as u8,
                (c.z * 255.0) as u8)
            );

            if let Some(points) = Self::rasterize_triangle(v0, v1, v2, vp) {
                canvas.draw_points(points.as_slice());
            }
        }
    }

    fn rasterize_triangle(
        v0: glam::Vec4,
        v1: glam::Vec4,
        v2: glam::Vec4,
        vp: &Viewport
    ) -> Option<Vec<sdl2::rect::Point>> {
        // Rasterize triangles
        //
        // Setup edge functions
        // a(x, y, w) = aa*x + ba*y + ca*w = 1 for vert a, 0 on edge opposite to a
        // b(x, y, w) = ab*x + bb*y + cb*w = 1 for vert b, 0 on edge opposite to b
        // c(x, y, w) = ay*x + by*y + cy*w = 1 for vert c, 0 on edge opposite to c
        //
        // a, b, c = barycentric coordinates
        //
        // points inside triangle have 0 < a, b, c < 1
        // [ x0 y0 w0 ] [ aa ab ay ]   [ 1 0 0 ]
        // [ x1 y1 w1 ] [ ba bb by ] = [ 0 1 0 ]
        // [ x2 y2 w2 ] [ ca cb cy ]   [ 0 0 1 ]
        // solve:
        // [ aa ab ay ]   [ x0 y0 w0 ][-1]
        // [ ba bb by ] = [ x1 y1 w1 ]
        // [ ca cb cy ]   [ x2 y2 w2 ]

        if v0.w < 0.0 && v1.w < 0.0 && v2.w < 0.0 {
            // Triangle is behind camera, don't draw
            return None;
        }

        // Compute AABB of triangle
        let (x_min, x_max, y_min, y_max) = if v0.w > 0.0 && v1.w > 0.0 && v2.w > 0.0 {
            use std::cmp::{min, max};

            // Project and compute AABB
            let v0 = v0 / v0.w;
            let v1 = v1 / v1.w;
            let v2 = v2 / v2.w;

            let x_min = min(v0.x as u16, min(v1.x as u16, v2.x as u16));
            let x_max = max(v0.x as u16, max(v1.x as u16, v2.x as u16));
            let y_min = min(v0.y as u16, min(v1.y as u16, v2.y as u16));
            let y_max = max(v0.y as u16, max(v1.y as u16, v2.y as u16));

            (x_min, x_max, y_min, y_max)
        } else {
            // Use whole viewport
            (0u16, vp.dim.0, 0u16, vp.dim.1)
        };

        // Setup barycentric coefficient matrix
        let verts = glam::Mat3::from_cols(
            glam::vec3(v0.x, v1.x, v2.x),
            glam::vec3(v0.y, v1.y, v2.y),
            glam::vec3(v0.w, v1.w, v2.w)
        );

        // det(verts) = 0 => triangle has zero area => don't draw
        // det(verts) < 0 => back-facing triangle => don't draw
        if verts.determinant() <= 0.0 {
            return None;
        }

        let coeffs = verts.inverse();
        let (aa, ba, ca) = (coeffs.x_axis.x, coeffs.x_axis.y, coeffs.x_axis.z);
        let (ab, bb, cb) = (coeffs.y_axis.x, coeffs.y_axis.y, coeffs.y_axis.z);
        let (ay, by, cy) = (coeffs.z_axis.x, coeffs.z_axis.y, coeffs.z_axis.z);

        // Store points instead of directly drawing here, should be easier to parallelize
        // since canvas is not thread safe.
        let mut points: Vec<sdl2::rect::Point> = Vec::with_capacity(2048usize);

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let (x, y) = (x as f32, y as f32);

                let aw = aa * x + ba * y + ca;
                let bw = ab * x + bb * y + cb;
                let cw = ay * x + by * y + cy;

                if aw > 0.0 && bw > 0.0 && cw > 0.0 {
                    // Point is inside triangle, draw
                    points.push(sdl2::rect::Point::new(x as i32, y as i32));
                }
            }
        }

        Some(points)
    }
}