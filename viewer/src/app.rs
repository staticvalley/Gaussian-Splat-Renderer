use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
use crate::renderer::Renderer;
 
#[derive(Default)]
pub struct App {
    /// reference to window and renderer (both None until created)
    state: Option<(Arc<Window>, Renderer)>,
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
                renderer.render();
                window.request_redraw();
            },
            _ => {}
        }
    }
}
