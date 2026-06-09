mod app;
mod renderer;
mod camera;
mod mesh;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    // log crate for wgpu
    env_logger::init();

    // create event loop
    let event_loop = EventLoop::new().expect("Viewer: failed to create event loop");

    // set to polling for rendering
    event_loop.set_control_flow(ControlFlow::Poll);

    // run application
    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("Viewer: failed to run application");
}
