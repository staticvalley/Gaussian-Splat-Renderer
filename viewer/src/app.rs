use std::sync::Arc;
use winit::{
    application::ApplicationHandler, event::{DeviceEvent, ElementState, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{CursorGrabMode, Window, WindowId}
};
use crate::renderer::Renderer;
use crate::camera::CameraController;
 
#[derive(Default)]
pub struct App {
    /// reference to window and renderer (both None until created)
    state: Option<(Arc<Window>, Renderer)>,
    camera_controller: CameraController,
    cursor_locked: bool,
}
 
impl ApplicationHandler for App {
    
    // init window and renderer when application is "resumed" (ie. started)
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        
        // create window handle
        let window = Arc::new(event_loop.create_window(
            Window::default_attributes().with_title("3D Gaussian Splat Renderer"),
        ).unwrap());

        // create renderer (block async function to get instance)
        let renderer = pollster::block_on(Renderer::new(Arc::clone(&window)));

        // assign state
        self.state = Some((window, renderer));
        self.camera_controller = CameraController::new();
        
        self.cursor_locked = true;
        let (window, _) = self.state.as_ref().unwrap();
        App::update_cursor_state(window, self.cursor_locked);
    }
 
    // something has happened to the window (resizing, drawing, closing)
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {

        // get window and renderer from state
        let Some((window, renderer)) = &mut self.state else { return };

        // handle certain event
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => renderer.resize(size),
            WindowEvent::RedrawRequested => {
                // update camera position before redraw
                self.camera_controller.update_camera(renderer.get_mutable_camera());
                // do redraw
                renderer.render();
                window.request_redraw();
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let is_pressed = event.state == ElementState::Pressed;
                    if is_pressed && code == KeyCode::Tab {
                        self.cursor_locked = !self.cursor_locked;
                        App::update_cursor_state(window, self.cursor_locked);
                    }
                    self.camera_controller.handle_keyboard(code, is_pressed);
                }
            },
            _ => {}
        }
    }

    fn device_event( &mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            if self.cursor_locked {
                self.camera_controller.handle_mouse(dx as f32, dy as f32);
            }
        }
    }
}

impl App {

    fn update_cursor_state(window: &Arc<Window>, cursor_locked: bool) {
        let grab = if cursor_locked {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        window.set_cursor_grab(grab).ok();
        window.set_cursor_visible(!cursor_locked);
    }

}
