use glam::{Vec3, Mat4};
use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct Camera {
    /// camera position in world space
    pub position: Vec3,
    /// camera rotation in world space (roll, pitch, yaw)
    rotation: Vec3,

    // members that probably shouldn't be manually updated at any point
    /// camera field of view (in degrees)
    _fov: f32,
    /// camera aspect ratio (surface size)
    _aspect_ratio: f32,
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
    pub fn new(aspect_ratio: f32) -> Self {
        let mut camera = Camera {
            position:       Vec3::new(0.0, 0.0, 2.0),
            rotation:       Vec3::ZERO,
            _fov:           45.0,
            _aspect_ratio:  aspect_ratio,
            _near:          0.1,
            _far:           100.0,
            _forward:       Vec3::new(0.0, 0.0, -1.0),
            _right:         Vec3::new(1.0, 0.0, 0.0),
            _up:            Vec3::new(0.0, 1.0, 0.0),
        };

        camera.calculate_relative_vectors();
        camera
    }

    pub fn get_view_projection(&self) -> Mat4 {
        let projection = Mat4::perspective_rh(
            self._fov.to_radians(),
            self._aspect_ratio,
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

        let (pitch, yaw) = (self.rotation.x.to_radians(), self.rotation.y.to_radians());

        /*  
            forward vector 
            euler's angles calculation
        */ 
        self._forward = Vec3::new(
            f32::cos(pitch) * f32::sin(yaw),
            f32::sin(pitch),
            -f32::cos(pitch) * f32::cos(yaw)
        ).normalize();

        // right vector (cross product of forward and world up (0, 1, 0))
        self._right = self._forward.cross(Vec3::Y).normalize();

        // up vector (cross product of right and forward)
        self._up = self._right.cross(self._forward).normalize();
    }

}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// Mat4 holding projection * view matrix
    view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection: Mat4::IDENTITY.to_cols_array_2d()
        }
    }

    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.get_view_projection().to_cols_array_2d();
    }
}

#[derive(Default)]
pub struct CameraController {
    // keyboard members
    move_speed: f32,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,

    // mouse members
    mouse_sensitivity: f32,
    pitch_change: f32,
    yaw_change: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            move_speed:         0.05,
            move_forward:       false,
            move_backward:      false,
            move_left:          false,
            move_right:         false,
            mouse_sensitivity:  0.9,
            pitch_change:       0.0,
            yaw_change:         0.0,
        }
    }

    pub fn handle_keyboard(&mut self, code: KeyCode, is_pressed: bool) {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => self.move_forward = is_pressed,
            KeyCode::KeyS | KeyCode::ArrowDown => self.move_backward = is_pressed,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.move_left = is_pressed,
            KeyCode::KeyD | KeyCode::ArrowRight => self.move_right = is_pressed,
            _ => {},
        }
    }

    pub fn handle_mouse(&mut self, dx: f32, dy: f32) {
        self.pitch_change = dy * self.mouse_sensitivity;
        self.yaw_change = dx * self.mouse_sensitivity;
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        
        let forward_normalized = camera._forward.normalize();
        let right_normalized = camera._right.normalize();

        if self.move_forward {
            camera.position += forward_normalized * self.move_speed;
        }

        if self.move_backward {
            camera.position -= forward_normalized * self.move_speed;
        }

        if self.move_left {
            camera.position -= right_normalized * self.move_speed;
        }

        if self.move_right {
            camera.position += right_normalized * self.move_speed;
        }

        camera.rotation.x -= self.pitch_change;
        camera.rotation.y += self.yaw_change;

        // reset change deltas
        self.pitch_change = 0.0;
        self.yaw_change = 0.0;

        camera.calculate_relative_vectors();
    }
}