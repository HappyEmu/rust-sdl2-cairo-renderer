use crate::viewport::Viewport;

pub struct Mesh {
    vertices: Vec<glam::Vec3>,
    indices: Vec<u32>,
    transform: glam::Mat4
}

impl Mesh {
    pub fn new(vertices: Vec<glam::Vec3>, indices: Vec<u32>) -> Self {
        Mesh { vertices, indices, transform: glam::Mat4::identity() }
    }

    pub fn draw(&self, mvp: &glam::Mat4, cairo: &cairo::Context, vp: &Viewport) {
        // Draw vertices
        for vertex in (&self.vertices).iter() {
            // Transform to NDC (with perspective division)
            let ndc = vp.mat * *mvp * glam::Vec4::new(vertex.x, vertex.y, vertex.z, 1.0);
            let ndc = ndc / ndc.w;

            // Cull vertices behind camera plane
            if ndc.z > 1.0 {
                continue;
            }

            // Convert to screen coordinates
            // let x = ((ndc.x + 1.0) * 0.5 * screen.0 as f32) as f64;
            // let y = ((1.0 - ((ndc.y + 1.0) * 0.5)) * screen.1 as f32) as f64;
            let (x, y) = (ndc.x as f32, ndc.y as f32);

            // Draw vertex as square
            cairo.rectangle(x as f64 - 4.0, y as f64 - 4.0, 8.0, 8.0);
            cairo.fill();
        }

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
        for triangle_indices in self.indices.windows(3usize) {
            let t = vp.mat * *mvp;

            let v0 = t * self.vertices[triangle_indices[0] as usize].extend(1.0);
            let v1 = t * self.vertices[triangle_indices[1] as usize].extend(1.0);
            let v2 = t * self.vertices[triangle_indices[2] as usize].extend(1.0);

            let verts = glam::Mat3::from_cols(
                glam::vec3(v0.x, v1.x, v2.x),
                glam::vec3(v0.y, v1.y, v2.y),
                glam::vec3(v0.w, v1.w, v2.w)
            );

            // det(verts) = 0 => triangle has zero area => don't draw
            // det(verts) < 0 => back-facing triangle => don't draw
            if verts.determinant() <= 0.0 {
                continue;
            }

            let coeffs = verts.inverse();

            let (aa, ba, ca) = (coeffs.x_axis.x, coeffs.x_axis.y, coeffs.x_axis.z);
            let (ab, bb, cb) = (coeffs.y_axis.x, coeffs.y_axis.y, coeffs.y_axis.z);
            let (ay, by, cy) = (coeffs.z_axis.x, coeffs.z_axis.y, coeffs.z_axis.z);

            // Check whole screen
            // TODO: Implement AABB optimization (only test pixels within AABB of triangle)
            for y in 0..vp.dim.1 {
                for x in 0..vp.dim.0 {
                    let (x, y) = (x as f32, y as f32);

                    let aw = aa * x + ba * y + ca;
                    let bw = ab * x + bb * y + cb;
                    let cw = ay * x + by * y + cy;

                    if aw > 0.0 && bw > 0.0 && cw > 0.0 {
                        // Point is inside triangle, draw
                        cairo.rectangle(x as f64, y as f64, 1.5, 1.5);
                        cairo.fill();
                    }
                }
            }
        }
    }
}