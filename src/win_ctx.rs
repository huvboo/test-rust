use core::f64;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::MouseScrollDelta,
    window::{Window, WindowId},
};

use crate::{dcel::MeshCoverage, layer::Layer, service::Service, state::State, wgpu_ctx::WgpuCtx};

struct Element {}

struct ElementTreeNode {}

pub struct WinCtx<'window> {
    pub window_id: WindowId,
    pub wgpu_ctx: WgpuCtx<'window>,
    pub state: State,
    pub element_map: HashMap<u32, Element>,
    pub element_tree: Vec<ElementTreeNode>,
}

impl<'window> WinCtx<'window> {
    pub fn new(window: Window) -> WinCtx<'window> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let window_id = window.id();

        // 创建 wgpu::Instance
        let instance = wgpu::Instance::default();
        // 从窗口创建 Surface
        let surface = instance.create_surface(Arc::new(window).clone()).unwrap();

        let wgpu_ctx = WgpuCtx::new(instance, surface, width, height);

        let state = State::new(
            &wgpu_ctx.surface_config,
            &wgpu_ctx.adapter,
            &wgpu_ctx.device,
            &wgpu_ctx.queue,
        );

        let element_map: HashMap<u32, Element> = HashMap::new();

        let element_tree: Vec<ElementTreeNode> = Vec::new();

        WinCtx {
            window_id,
            wgpu_ctx,
            state,
            element_map,
            element_tree,
        }
    }

    pub fn redraw(&mut self) {
        self.state.update(&self.wgpu_ctx.queue);
        match self.state.render(
            &self.wgpu_ctx.surface_config,
            &self.wgpu_ctx.surface,
            &self.wgpu_ctx.device,
            &self.wgpu_ctx.queue,
        ) {
            Ok(_) => {}
            // 如果发生上下文丢失，就重新配置 surface
            Err(wgpu::SurfaceError::Lost) => self.state.resize(
                self.wgpu_ctx.surface_config.width.clone(),
                self.wgpu_ctx.surface_config.height.clone(),
            ),
            // 系统内存不足，此时应该退出
            Err(wgpu::SurfaceError::OutOfMemory) => {
                // send_message(Arc::clone(&s), MessageId::CloseRequested, true)
            }
            // 所有其他错误（如过时、超时等）都应在下一帧解决
            Err(e) => eprintln!("{:?}", e),
        }
    }

    pub fn drop_file(&mut self, path_buf: PathBuf) {
        drop_file(path_buf, &mut self.state, &self.wgpu_ctx);
        self.redraw();
    }
    pub fn mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.state
            .scene
            .set_mouse_position([position.x, position.y]);
    }
    pub fn mouse_wheel(&mut self, delta: MouseScrollDelta) {
        if let MouseScrollDelta::LineDelta(_x, y) = delta {
            self.state.scene.scale_on_mouse_wheel(y as f64);
            self.redraw();
        }
    }
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.wgpu_ctx.resize(new_size);
        self.state.resize(new_size.width, new_size.height);
        self.redraw();
    }
    pub fn translate(&mut self, (tx, ty): (f64, f64)) {
        self.state.scene.translate(tx.clone(), ty.clone());
        self.redraw();
    }
}

pub fn drop_file(path_buf: PathBuf, state: &mut State, wgpu_ctx: &WgpuCtx<'_>) {
    let ext = path_buf.extension().unwrap();
    if ext.to_str() == Some("grmsp") {
        let coverages = Service::read_grmsp_coverage_file(&path_buf);
        for coverage in coverages {
            println!(
                "{:#?}, {:#?}",
                coverage.module_name.as_str(),
                coverage.module_name.as_str() == "mesh"
            );
            if coverage.module_name.as_str() == "mesh" {
                println!("coverage:{:#?}", coverage);
                let mut mesh_coverage = MeshCoverage::new(coverage.id.clone());
                let dir = path_buf.parent().unwrap();
                Service::load_mesh(dir, coverage.id.clone(), &mut mesh_coverage);
                let mut layer = Layer::new(
                    coverage.id.clone(),
                    &state,
                    &wgpu_ctx.device,
                    &wgpu_ctx.surface_config,
                );
                Service::set_mesh_data(&wgpu_ctx.device, &mut mesh_coverage, &mut layer);
                state.add_coverage(mesh_coverage);
                state.add_layer(layer);
            }
        }
    }
}
