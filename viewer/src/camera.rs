use glam::{Vec3, Mat4};

#[derive(Default)]
pub struct Camera {
    /// camera position in world space
    position: Vec3,
    /// camera rotation in world space (roll, pitch, yaw)
    rotation: Vec3,
    /// camera field of view (in degrees)
    _fov: f32,
    /// camera near clipping plane
    _near: f32,
    /// camera far clipping plane
    _far: f32,
    /// forward vector
    _forward: Vec3,
    /// right vector
    _right: Vec3,
    /// up vector
    _up: Vec3,
}

impl Camera {
    /// create camera (right-handed system looking down -z)
    pub fn new() -> Self {
        let mut camera = Camera {
            position:   Vec3::new(0.0, 0.0, 0.0),
            rotation:   Vec3::ZERO,
            _fov:       45.0,
            _near:      0.1,
            _far:       100.0,
            _forward:   Vec3::new(0.0, 0.0, -1.0),
            _right:     Vec3::new(1.0, 0.0, 0.0),
            _up:        Vec3::new(0.0, 1.0, 0.0),
        };

        camera.calculate_relative_vectors();
        camera
    }

    pub fn get_view_projection(&self, aspect_ratio: f32) -> Mat4 {
        let projection = Mat4::perspective_rh(
            self._fov.to_radians(),
            aspect_ratio,
            self._near,
            self._far
        );
        let view = Mat4::look_at_rh(
            self.position,
            self.position + self._forward,
            self._up
        );
        projection * view
    }

    /// calculate relative forward, right, and up vectors
    fn calculate_relative_vectors(&mut self) {

        let (pitch, yaw) = (self.rotation.y.to_radians(), self.rotation.z.to_radians());

        /*  
            forward vector 
            euler's angles calculation
        */ 
        self._forward = Vec3::new(
            f32::cos(yaw) * f32::cos(pitch),
            f32::sin(pitch),
            f32::sin(yaw) * f32::cos(pitch)
        ).normalize();

        // right vector (cross product of forward and world up (0, 1, 0))
        self._right = self._forward.cross(Vec3::Y).normalize();

        // up vector (cross product of right and forward)
        self._up = self._right.cross(self._forward).normalize();
    }

}