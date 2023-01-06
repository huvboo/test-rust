extern crate rand;
use crate::{
    dcel::{BBox3, MeshCoverage},
    m4::{multiply, projection, scale, translate, xRotate, yRotate, zRotate},
    render::{create_render_pipeline, uniform4f},
};
use std::{collections::HashMap, iter};

use cgmath::{
    num_traits::{Float, ToPrimitive},
    Matrix4,
};
use wgpu::{util::DeviceExt, CommandEncoder, Device, RenderPass, SurfaceTexture, TextureView};
use winit::{dpi::PhysicalPosition, event::*, window::Window};

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

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(1.0,0.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,0.5,0.0,0.0,0.0,0.5,1.0);

pub struct Scene {
    w: f64,                  // 画布宽，像素值
    h: f64,                  // 画布高，像素值
    mousePosition: [f64; 2], // 鼠标指针相对于可视窗口的 X 轴 Y轴的距离 单位：像素
    bbox: BBox3,             // 可视区域坐标包围盒
    center: [f64; 3],        // 包围盒的中心点
    minZoom: f64,            // 最小缩放值，初始值 0.8 ** 50
    maxZoom: f64,            // 最大缩放值，初始值 1.25 ** 22
    _zoom: f64,              // 上一次的缩放值
    zoom: f64,               // 当前缩放值
    tx: f64,                 // X轴平移的距离（单位1）而非屏幕像素
    ty: f64,                 // Y轴平移的距离（单位1）而非屏幕像素
    tz: f64,                 // Z轴平移的距离（单位1）而非屏幕像素
    rx: f64,                 // 绕X轴旋转的角度
    ry: f64,                 // 绕Y轴旋转的角度
    rz: f64,                 // 绕Z轴旋转的角度
}

impl Scene {
    pub fn new(w: f64, h: f64) -> Self {
        Self {
            w,
            h,
            mousePosition: [0.0, 0.0],
            bbox: BBox3::new(),
            center: [0.0, 0.0, 0.0],
            minZoom: 0.8.powi(50),
            maxZoom: 1.25.powi(22),
            _zoom: 1.0,
            zoom: 1.0,
            tx: 0.0,
            ty: 0.0,
            tz: 0.0,
            rx: 0.0,
            ry: 0.0,
            rz: 0.0,
        }
    }

    pub fn get_mat4(&self) -> [[f32; 4]; 4] {
        let far = 1000000000.0;
        let mut matrix = projection(self.w, self.h, far * self.zoom);

        matrix = translate(
            matrix,
            self.tx * self.zoom,
            self.ty * self.zoom,
            -self.w / 2.0, // state.transform.tz * state.transform.scale
        );

        matrix = xRotate(matrix, self.rx);
        matrix = yRotate(matrix, self.ry);
        matrix = zRotate(matrix, self.rz);

        matrix = scale(matrix, self.zoom, self.zoom, self.zoom);
        matrix = multiply(
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
            ],
            matrix,
        );
        return [
            [
                matrix[0] as f32,
                matrix[1] as f32,
                matrix[2] as f32,
                matrix[3] as f32,
            ],
            [
                matrix[4] as f32,
                matrix[5] as f32,
                matrix[6] as f32,
                matrix[7] as f32,
            ],
            [
                matrix[8] as f32,
                matrix[9] as f32,
                matrix[10] as f32,
                matrix[11] as f32,
            ],
            [
                matrix[12] as f32,
                matrix[13] as f32,
                matrix[14] as f32,
                matrix[15] as f32,
            ],
        ];
    }

    fn scale(&mut self, times: f64) {
        let mut multiplier = 1.25.powf(times);
        println!("{:?}", multiplier);
        self._zoom = self.zoom;
        self.zoom = (self.zoom * multiplier).min(self.maxZoom).max(self.minZoom);
    }

    fn translate(&mut self, tx: f64, ty: f64) {
        self.tx = self.tx + tx / self.zoom;
        self.ty = self.ty + ty / self.zoom;
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mousePosition = [position.x, position.y];
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_x, y) = delta {
                    self.scale(*y as f64);

                    let ds = 1.0 - self.zoom / self._zoom;
                    let dx = (self.mousePosition[0] - self.w / 2.0) * ds;
                    let dy = (self.h / 2.0 - self.mousePosition[1]) * ds;
                    self.translate(dx, dy);
                }
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => match keycode {
                VirtualKeyCode::W | VirtualKeyCode::Up => {
                    self.translate(0.0, 30.0);
                    true
                }
                VirtualKeyCode::A | VirtualKeyCode::Left => {
                    self.translate(-30.0, 0.0);
                    true
                }
                VirtualKeyCode::S | VirtualKeyCode::Down => {
                    self.translate(0.0, -30.0);
                    true
                }
                VirtualKeyCode::D | VirtualKeyCode::Right => {
                    self.translate(30.0, 0.0);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

struct CameraStaging {
    camera: Camera,
    model_rotation: cgmath::Deg<f32>,
}

impl CameraStaging {
    fn new(camera: Camera) -> Self {
        Self {
            camera,
            model_rotation: cgmath::Deg(0.0),
        }
    }

    fn update_camera(&self, camera_uniform: &mut CameraUniform) {
        camera_uniform.model_view_proj = (OPENGL_TO_WGPU_MATRIX
            * self.camera.build_view_projection_matrix()
            * cgmath::Matrix4::from_angle_z(self.model_rotation))
        .into();
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    model_view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            model_view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_left_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_left_right_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            // WindowEvent::MouseWheel {
            //     device_id,
            //     delta,
            //     phase,
            //     ..
            // } => {
            //     if let MouseScrollDelta::LineDelta(x, y) = delta {
            //         println!("y:{:?}", y);
            //         self.is_forward_pressed = y > &0.0;
            //         self.is_backward_pressed = y < &0.0;
            //     }
            //     true
            // }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_left_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update_camera(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // 防止摄像机离场景中心太近时出现故障
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // 在按下前进或后退键时重做半径计算
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_left_right_pressed {
            // 重新调整目标与眼睛之间的距离，以使其不发生变化
            // 因此，眼睛仍位于由目标和眼睛所组成的圆上
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
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
    pub fn new(coverage_id: String, state: &mut State) -> Self {
        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: &[],
                usage: wgpu::BufferUsages::INDEX,
            });
        let num_indices = 0;

        let shader = state
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let color_uniform: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let (color_bind_group_layout, color_bind_group) = uniform4f(
            "color",
            color_uniform,
            &state.device,
            wgpu::ShaderStages::FRAGMENT,
        );

        let render_pipeline = create_render_pipeline(
            shader,
            &state.device,
            &[&color_bind_group_layout, &state.camera_bind_group_layout],
            &[Vertex::desc()],
            state.config.format,
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

    pub fn setdata(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>, state: &mut State) {
        self.vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        self.index_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
                wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                },
            ],
            depth_stencil_attachment: None,
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

pub struct State {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub coverages: HashMap<String, MeshCoverage>,
    pub layers: Vec<Layer>,
    pub scene: Scene,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // instance 变量是到 GPU 的 handle
        // Backends::all 对应 Vulkan + Metal +DX12 + 浏览器的 WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let scene = Scene::new(size.width as f64, size.height as f64);

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
            surface,
            device,
            queue,
            config,
            size,
            coverages,
            layers,
            scene,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.scene.w = new_size.width as f64;
            self.scene.h = new_size.height as f64;
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.scene.process_events(event)
    }

    pub fn update(&mut self) {
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.scene.get_mat4()]),
        );
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // 这就是片元着色器中 [[location(0)]] 对应的目标
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

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
        self.queue.submit(iter::once(encoder.finish()));

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
