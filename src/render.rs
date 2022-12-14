use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, RenderPipeline, TextureFormat, VertexBufferLayout,
};

pub fn uniform4f(
    name: &str,
    arr: [f32; 4],
    device: &wgpu::Device,
    stage: wgpu::ShaderStages,
) -> (BindGroupLayout, BindGroup) {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&(name.to_string() + "Buffer")),
        contents: bytemuck::cast_slice(&[arr]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: stage,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(&(name.to_string() + "_bind_group_layout")),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &&bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some(&(name.to_string() + "_bind_group")),
    });
    (bind_group_layout, bind_group)
}

pub fn create_render_pipeline(
    shader: wgpu::ShaderModule,
    device: &wgpu::Device,
    bind_group_layouts: &[&BindGroupLayout],
    buffers: &[VertexBufferLayout],
    format: TextureFormat,
) -> RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // 如果将该字段设置为除了 Fill 之外的任何职值， 都
            // 需要 Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // 需要 Features::DEPTH_CLIP_ENABLE
            unclipped_depth: false,
            // 需要 Features::CONSERVATILE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    render_pipeline
}
