use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer {
    /// surface handle
    surface: wgpu::Surface<'static>,    
    /// gpu handle
    device: wgpu::Device,               
    /// gpu command queue
    queue: wgpu::Queue,
    /// surface settings
    config: wgpu::SurfaceConfiguration
}

impl Renderer {

    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // get wgpu instance and surface handle from window
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        // get gpu adapter
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.unwrap();

        // request gpu handle and gpu command queue from adapter (async)
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await.unwrap();

        // configure surface for window display
        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0], // using 0 for optimal display format
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0], 
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self { surface, device, queue, config }
    }

    /// reconfigure surface on window resize
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        // ignore minimized windows
        if size.width == 0 || size.height == 0 { return; }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        // get next texture "buffer" from surface to render to
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                // suboptimal: surface is still usable but should be reconfigured (maybe window size changed)
                // in this case, i reconfigure and then return output texture
                log::warn!("surface texture is suboptimal, reconfiguring surface");
                self.surface.configure(&self.device, &self.config);
                texture
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                // outdated or lost: surface cant be used, reconfigure and skip to next frame
                log::warn!("surface texture is outdated or lost, reconfiguring surface & skipping frame");
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };

        // get texture view for gpu
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    
        // create gpu command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // simple render to clear screen
        {

            let clear_color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    // clear to light blue before pass renders
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 }),
                    // store results after pass renders
                    store: wgpu::StoreOp::Store,
                },
            };

            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(clear_color_attachment)],
                ..Default::default()
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}