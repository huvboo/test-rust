use crate::{dcel::MeshCoverage, layer::Layer, scene::Scene};
use std::{collections::HashMap, iter};

use wgpu::{util::DeviceExt, Adapter, Device, Queue, StoreOp, Surface, SurfaceConfiguration};

pub struct State {
    pub coverages: HashMap<String, MeshCoverage>,
    pub layers: Vec<Layer>,
    pub scene: Scene,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
}

impl State {
    pub fn new(
        surface_config: &SurfaceConfiguration,
        adapter: &Adapter,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let scene = Scene::new(surface_config.width as f64, surface_config.height as f64);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[scene.get_mat4()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &&camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let coverages: HashMap<String, MeshCoverage> = HashMap::new();
        let layers: Vec<Layer> = Vec::new();

        Self {
            coverages,
            layers,
            scene,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.scene.resize(width as f64, height as f64);
        }
    }

    pub fn update(&mut self, queue: &Queue) {
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.scene.get_mat4()]),
        );
    }

    pub fn render(
        &self,
        surface_config: &SurfaceConfiguration,
        surface: &Surface,
        device: &Device,
        queue: &Queue,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // 这就是片元着色器中 [[location(0)]] 对应的目标
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            // 设置剪切矩形
            render_pass.set_scissor_rect(
                240,
                120,
                surface_config.width - 480,
                surface_config.height - 160,
            );

            for layer in &self.layers {
                render_pass.set_pipeline(&layer.render_pipeline);
                render_pass.set_bind_group(0, &layer.color_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

                render_pass.set_vertex_buffer(0, layer.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(layer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                render_pass.draw_indexed(0..layer.num_indices, 0, 0..1);
                // render_pass.draw(0..4, 0..1);
            }
        }

        // submit 方法能传入任何实现了 IntoIter 的参数
        queue.submit(iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn add_coverage(&mut self, coverage: MeshCoverage) {
        self.coverages.insert(coverage.id.clone(), coverage);
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }
}
