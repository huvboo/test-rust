use std::sync::Arc;

use wgpu::{
    Adapter, Device, Features, Instance, Limits, MemoryHints, Queue, Surface, SurfaceConfiguration,
};
use winit::{dpi::PhysicalSize, window::Window};

pub struct WgpuCtx<'window> {
    pub surface: Surface<'window>,
    pub surface_config: SurfaceConfiguration,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl<'window> WgpuCtx<'window> {
    pub async fn new_async(
        instance: Instance,
        surface: Surface<'window>,
        width: u32,
        height: u32,
    ) -> Self {
        // 获取适配器
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // 获取逻辑设备、命令队列
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    memory_hints: MemoryHints::Performance,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let surface_config = surface.get_default_config(&adapter, width, height).unwrap();

        surface.configure(&device, &surface_config);

        WgpuCtx {
            surface,
            surface_config,
            adapter,
            device,
            queue,
        }

        // let config = wgpu::SurfaceConfiguration {
        //     usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        //     format: surface.get_preferred_format(adapter).unwrap(),
        //     width: size.width,
        //     height: size.height,
        //     present_mode: wgpu::PresentMode::Fifo,
        //     desired_maximum_frame_latency: todo!(),
        //     alpha_mode: todo!(),
        //     view_formats: todo!(),
        // };

        // surface.configure(&device, &config);
    }

    pub fn new(instance: Instance, surface: Surface<'window>, width: u32, height: u32) -> Self {
        pollster::block_on(WgpuCtx::new_async(instance, surface, width, height))
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width.max(1);
        self.surface_config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn draw(&mut self) {}
}
