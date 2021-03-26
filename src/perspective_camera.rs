use ultraviolet::projection::rh_yup::perspective_gl as perspective;
use ultraviolet::Mat4;
use ultraviolet::Vec3;

pub struct PerspectiveCamera {
    pub eye: Vec3,
    pub at: Vec3,
    pub up: Vec3,
    pub vertical_fov: f32,
    pub aspect_ratio: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl PerspectiveCamera {
    pub fn compute_matrix(&self) -> Mat4 {
        return // * 
            perspective(
                self.vertical_fov,
                self.aspect_ratio,
                self.z_near,
                self.z_far,
            ) * Mat4::look_at(self.eye, self.at, self.up);
    }
}
