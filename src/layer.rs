use wgpu::{util::DeviceExt, CommandEncoder, Device, StoreOp, SurfaceConfiguration, TextureView};

use crate::{
    render::{create_render_pipeline, uniform4f},
    state::State,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub id: u32,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1=> Uint32];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }

        // wgpu::VertexBufferLayout {
        //     array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        //     step_mode: wgpu::VertexStepMode::Vertex,
        //     attributes: &[
        //         wgpu::VertexAttribute {
        //             offset: 0,
        //             shader_location: 0,
        //             format: wgpu::VertexFormat::Uint32,
        //         },
        //         wgpu::VertexAttribute {
        //             offset: mem::size_of::<u32>() as wgpu::BufferAddress,
        //             shader_location: 1,
        //             format: wgpu::VertexFormat::Float32x3,
        //         },
        //         wgpu::VertexAttribute {
        //             offset: mem::size_of::<u32>() as wgpu::BufferAddress
        //                 + mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
        //             shader_location: 2,
        //             format: wgpu::VertexFormat::Float32x3,
        //         },
        //     ],
        // }
    }
}

pub struct Layer {
    pub coverage_id: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub render_pipeline: wgpu::RenderPipeline,
    pub color_bind_group: wgpu::BindGroup,
}

impl Layer {
    pub fn new(
        coverage_id: String,
        state: &State,
        device: &Device,
        config: &SurfaceConfiguration,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &[],
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = 0;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let color_uniform: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let (color_bind_group_layout, color_bind_group) =
            uniform4f("color", color_uniform, device, wgpu::ShaderStages::FRAGMENT);

        let render_pipeline = create_render_pipeline(
            shader,
            device,
            &[&color_bind_group_layout, &state.camera_bind_group_layout],
            &[Vertex::desc()],
            config.format,
        );

        Self {
            coverage_id,
            vertex_buffer,
            index_buffer,
            num_indices,
            render_pipeline,
            color_bind_group,
        }
    }

    pub fn setdata(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>, device: &Device) {
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.num_indices = indices.len() as u32;
    }

    pub fn draw(&self, state: &State, encoder: &mut CommandEncoder, view: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // 这就是片元着色器中 [[location(0)]] 对应的目标
                Some(wgpu::RenderPassColorAttachment {
                    view,
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
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.color_bind_group, &[]);
        render_pass.set_bind_group(1, &state.camera_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        // render_pass.draw(0..4, 0..1);
    }
}
