use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::mesh::{VERTICES, INDICES, Vertex};

pub struct Renderer {
    /// surface handle
    surface: wgpu::Surface<'static>,    
    /// gpu handle
    device: wgpu::Device,               
    /// gpu command queue
    queue: wgpu::Queue,
    /// surface settings
    config: wgpu::SurfaceConfiguration,
    /// render pipeline
    pipeline: wgpu::RenderPipeline,
    /// vertex data
    vertex_buffer: wgpu::Buffer,
    /// index data
    index_buffer: wgpu::Buffer,
    /// camera
    camera: Camera,
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

        // create shader pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let camera = Camera::new();

        Self { surface, device, queue, config, pipeline, vertex_buffer, index_buffer, camera }
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

            let mut _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    // clear to light blue before pass renders
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 }),
                    // store results after pass renders
                    store: wgpu::StoreOp::Store,
                },
            })],
                ..Default::default()
            });

            // start render!

            _pass.set_pipeline(&self.pipeline);
            _pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            _pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            _pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}