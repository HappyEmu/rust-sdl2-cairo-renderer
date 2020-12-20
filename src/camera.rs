use glam::Vec3;
use std::f32::consts;
use crate::mat4_look_at;

#[derive(Default)]
pub struct Camera {
    pos: glam::Vec3,
    rot: glam::Quat,
    yaw: f32,
    pit: f32
}

impl Camera {
    pub fn new(pos: glam::Vec3, look_at: glam::Vec3) -> Self {
        // glam::look_at generates view matrix, not camera matrix -> inverse
        let m = glam::Mat4::look_at_rh(pos, look_at, glam::Vec3::unit_y()).inverse();

        let (_, rot, _) = m.to_scale_rotation_translation();
        let (yaw, pit, rol) = quat_to_euler(&rot);

        Self {
            pos,
            rot,
            yaw: -pit,
            pit: -(rol + consts::PI)
        }
    }

    pub fn from_ypr(pos: glam::Vec3, yaw: f32, pitch: f32, roll: f32) -> Self {
        let q = glam::Quat::from_rotation_ypr(yaw, pitch, roll);

        Self {
            pos,
            rot: q,
            yaw,
            pit: pitch
        }
    }
}

impl Camera {
    pub fn yaw(&mut self, d_yaw: f32) {
        self.yaw += d_yaw;

        let q = glam::Quat::from_rotation_ypr(self.yaw, self.pit, 0.0);
        self.rot = q;
    }

    const ANGLE_CLAMP: f32 = consts::FRAC_PI_2 - 0.1;
    pub fn pitch(&mut self, d_pit: f32) {
        self.pit += d_pit;

        // Prevent gimbal lock
        if self.pit >= Self::ANGLE_CLAMP { self.pit = Self::ANGLE_CLAMP; }
        if self.pit <= -Self::ANGLE_CLAMP { self.pit = -Self::ANGLE_CLAMP; }

        let q = glam::Quat::from_rotation_ypr(self.yaw, self.pit, 0.0);
        self.rot = q;
    }

    pub fn translate(&mut self, offset: glam::Vec3) {
        let offset = self.transform().transform_vector3(offset);
        self.pos += offset;
    }

    pub fn view_matrix(&self) -> glam::Mat4 {
        self.transform().inverse()
    }

    pub fn transform(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(self.rot, self.pos)
    }
}

fn quat_to_euler(quat: &glam::Quat) -> (f32, f32, f32) {
    let (x, y, z, w) = (quat.x, quat.y, quat.z, quat.w);

    let yaw = (2.0 * (x*y + z*w)).atan2(1.0 - 2.0 * (y*y + z*z));
    let pit = (2.0 * (x*z - w*y)).asin();
    let rol = (2.0 * (x*w + y*z)).atan2(1.0 - 2.0 * (z*z + w*w));

    (yaw, pit, rol)
}