use std::sync::Arc;

use crate::{dcel::MeshCoverage, layer::Layer, service::Service, state::State, wgpu_ctx::WgpuCtx};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    state: Option<State>,
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("WebGPU Example");
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            let wgpu_ctx = WgpuCtx::new(window.clone());
            let mut state = State::new(
                &wgpu_ctx.surface_config,
                &wgpu_ctx.adapter,
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
            );

            let mut test_layer = Layer::new(
                String::from(""),
                &state,
                &wgpu_ctx.device,
                &wgpu_ctx.surface_config,
            );
            Service::set_test_data(&wgpu_ctx.device, &mut test_layer);
            state.add_layer(test_layer);

            self.state = Some(state);
            self.wgpu_ctx = Some(wgpu_ctx);
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(window) = self.window.as_ref() {
            if window_id == window.id() {
                match event {
                    // 拖放文件
                    WindowEvent::DroppedFile(path_buf) => {
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
                                    Service::load_mesh(
                                        dir,
                                        coverage.id.clone(),
                                        &mut mesh_coverage,
                                    );
                                    if let (Some(wgpu_ctx), Some(state)) =
                                        (self.wgpu_ctx.as_ref(), self.state.as_mut())
                                    {
                                        let mut layer = Layer::new(
                                            coverage.id.clone(),
                                            state,
                                            &wgpu_ctx.device,
                                            &wgpu_ctx.surface_config,
                                        );
                                        Service::set_mesh_data(
                                            &wgpu_ctx.device,
                                            &mut mesh_coverage,
                                            &mut layer,
                                        );
                                        state.add_coverage(mesh_coverage);
                                        state.add_layer(layer);
                                    }
                                }
                            }
                        }
                        window.request_redraw();
                    }

                    // 鼠标移动
                    WindowEvent::CursorMoved { position, .. } => {
                        if let Some(state) = self.state.as_mut() {
                            state.scene.set_mouse_position([position.x, position.y]);
                        }
                    }

                    // 鼠标滚动
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let MouseScrollDelta::LineDelta(_x, y) = delta {
                            if let Some(state) = self.state.as_mut() {
                                state.scene.scale_on_mouse_wheel(y as f64);
                            }
                        }
                        window.request_redraw();
                    }

                    // 上
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::KeyW | KeyCode::ArrowUp),
                                ..
                            },
                        ..
                    } => {
                        // self.translate(0.0, 30.0);
                        // true
                    }

                    // 下
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::KeyS | KeyCode::ArrowDown),
                                ..
                            },
                        ..
                    } => {
                        // self.translate(0.0, -30.0);
                        // true
                    }

                    // 左
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::KeyA | KeyCode::ArrowLeft),
                                ..
                            },
                        ..
                    } => {
                        // self.translate(-30.0, 0.0);
                        // true
                    }

                    // 右
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(KeyCode::KeyD | KeyCode::ArrowRight),
                                ..
                            },
                        ..
                    } => {
                        // self.translate(30.0, 0.0);
                        // true
                    }

                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => {
                        event_loop.exit();
                    }

                    WindowEvent::Resized(new_size) => {
                        if let (Some(window), Some(wgpu_ctx), Some(state)) = (
                            self.window.as_ref(),
                            self.wgpu_ctx.as_mut(),
                            self.state.as_mut(),
                        ) {
                            wgpu_ctx.resize(new_size);
                            state.resize(new_size.width, new_size.height);
                            window.request_redraw();
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if let (Some(wgpu_ctx), Some(state)) =
                            (self.wgpu_ctx.as_mut(), self.state.as_mut())
                        {
                            state.update(&wgpu_ctx.queue);
                            match state.render(
                                &wgpu_ctx.surface_config,
                                &wgpu_ctx.surface,
                                &wgpu_ctx.device,
                                &wgpu_ctx.queue,
                            ) {
                                Ok(_) => {}
                                // 如果发生上下文丢失，就重新配置 surface
                                Err(wgpu::SurfaceError::Lost) => state.resize(
                                    wgpu_ctx.surface_config.width.clone(),
                                    wgpu_ctx.surface_config.height.clone(),
                                ),
                                // 系统内存不足，此时应该退出
                                Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                                // 所有其他错误（如过时、超时等）都应在下一帧解决
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                    }
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    _ => {}
                }
            }
        }
    }
}
