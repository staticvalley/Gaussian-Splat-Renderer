use std::sync::Arc;
use winit::{
    application::ApplicationHandler, event::{DeviceEvent, ElementState, MouseButton, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{CursorGrabMode, CursorIcon, Window, WindowId}
};
use crate::renderer::{self, Renderer};
use crate::camera::CameraController;
 
#[derive(Default)]
pub struct App {
    /// reference to window and renderer (both None until created)
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    camera_controller: CameraController,
    is_dragging: bool,
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
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.camera_controller = CameraController::new();
    }
 
    // something has happened to the window (resizing, drawing, closing)
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {

        // get window and renderer from state
        let Some(window) = &mut self.window else { return };
        let Some(renderer) = &mut self.renderer else { return };

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
                    self.camera_controller.handle_keyboard(code, is_pressed);
                }
            },
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                let is_pressed = state == ElementState::Pressed;
                self.is_dragging = is_pressed;
            }
            _ => {}
        }
    }

    fn device_event( &mut self, _event_loop: &ActiveEventLoop, _device_id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        
        // get window and renderer from state
        let Some(window) = &mut self.window else {return};

        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                if self.is_dragging {
                    window.set_cursor(CursorIcon::Grabbing);
                    self.camera_controller.handle_mouse_drag(dx as f32, dy as f32);
                } else {
                    window.set_cursor(CursorIcon::Grab);
                }
            },
            _ => {}
        }
    }
}