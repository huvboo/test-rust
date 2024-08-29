use core::f64;
use std::{collections::HashMap, path::PathBuf, sync::Arc, thread};

use crate::{
    dcel::MeshCoverage,
    layer::Layer,
    message::{send_message, DynamicMessage, MessageId},
    service::Service,
    state::State,
    wgpu_ctx::WgpuCtx,
};
use crossbeam_channel::{Receiver, Sender};
use wgpu::{Instance, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::{Window, WindowId},
};

// 定义子线程的渲染逻辑
fn render_thread(
    r: Receiver<DynamicMessage>,
    s: Sender<DynamicMessage>,
    instance: Instance,
    surface: Surface,
    width: u32,
    height: u32,
) {
    println!("render_thread");
    let mut wgpu_ctx = WgpuCtx::new(instance, surface, width, height);

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

    fn redraw(state: &mut State, wgpu_ctx: &WgpuCtx<'_>, s: &Sender<DynamicMessage>) {
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
            Err(wgpu::SurfaceError::OutOfMemory) => {
                send_message(s.clone(), MessageId::Escape, true);
                // event_loop.exit()
            }
            // 所有其他错误（如过时、超时等）都应在下一帧解决
            Err(e) => eprintln!("{:?}", e),
        }
    }

    fn dropFile(path_buf: PathBuf, state: &mut State, wgpu_ctx: &WgpuCtx<'_>) {
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

    // 异步执行任务

    loop {
        let result = r.try_recv();
        match result {
            Ok(value) => {
                // message_queue.push(value);
                match value.message_id {
                    MessageId::DroppedFile => {
                        if let Some(path_buf) = value.data.downcast_ref::<PathBuf>() {
                            dropFile(path_buf.clone(), &mut state, &wgpu_ctx);
                            redraw(&mut state, &wgpu_ctx, &s)
                        }
                    }
                    MessageId::CursorMoved => {
                        if let Some(position) = value.data.downcast_ref::<PhysicalPosition<f64>>() {
                            state.scene.set_mouse_position([position.x, position.y]);
                            // redraw(&mut state, &wgpu_ctx, &s)
                        }
                    }
                    MessageId::MouseWheel => {
                        if let Some(delta) = value.data.downcast_ref::<MouseScrollDelta>() {
                            if let MouseScrollDelta::LineDelta(_x, y) = delta {
                                state.scene.scale_on_mouse_wheel(*y as f64);
                                redraw(&mut state, &wgpu_ctx, &s)
                            }
                        }
                    }
                    MessageId::Translate => {
                        if let Some((tx, ty)) = value.data.downcast_ref::<(f64, f64)>() {
                            state.scene.translate(tx.clone(), ty.clone());
                            redraw(&mut state, &wgpu_ctx, &s)
                        }
                    }
                    MessageId::Escape => {}
                    MessageId::Resized => {
                        if let Some(new_size) = value.data.downcast_ref::<PhysicalSize<u32>>() {
                            wgpu_ctx.resize(*new_size);
                            state.resize(new_size.width, new_size.height);
                            redraw(&mut state, &wgpu_ctx, &s)
                        }
                    }
                    MessageId::RedrawRequested => redraw(&mut state, &wgpu_ctx, &s),
                }
            }
            Err(_) => {}
        }
        // 获取窗口帧
        // let window_guard = window_mutex.lock().unwrap();
        // let size = window_guard.inner_size();
        // 发送信号给主线程
        // s.send((String::from("resize"), size));
    }
}

#[derive(Default)]
pub struct App {
    is_first_resumed: bool,
    window_id_sender_map: HashMap<WindowId, Sender<DynamicMessage>>,
    window_id_receiver_map: HashMap<WindowId, Receiver<DynamicMessage>>,
}

impl App {
    pub fn new() -> App {
        let window_id_sender_map: HashMap<WindowId, Sender<DynamicMessage>> = HashMap::new();
        let window_id_receiver_map: HashMap<WindowId, Receiver<DynamicMessage>> = HashMap::new();
        App {
            is_first_resumed: true,
            window_id_sender_map,
            window_id_receiver_map,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("1111111111 {}", self.is_first_resumed);
        if self.is_first_resumed {
            self.is_first_resumed = false;
            let win_attr = Window::default_attributes().with_title("WebGPU Example");
            let window = event_loop
                .create_window(win_attr)
                .expect("create window err.");

            let window_id = window.id();
            let (s1, r1): (Sender<DynamicMessage>, Receiver<DynamicMessage>) =
                crossbeam_channel::unbounded();
            self.window_id_sender_map.insert(window_id.clone(), s1);
            let (s2, r2): (Sender<DynamicMessage>, Receiver<DynamicMessage>) =
                crossbeam_channel::unbounded();
            self.window_id_receiver_map.insert(window_id.clone(), r2);

            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            // 创建 wgpu::Instance
            let instance = wgpu::Instance::default();
            // 从窗口创建 Surface
            let surface = instance.create_surface(Arc::new(window).clone()).unwrap();

            thread::spawn(move || {
                render_thread(r1, s2, instance, surface, width, height);
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            // 拖放文件
            WindowEvent::DroppedFile(path_buf) => {
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::DroppedFile, path_buf);
                }
            }

            // 鼠标移动
            WindowEvent::CursorMoved { position, .. } => {
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::CursorMoved, position);
                }
            }

            // 鼠标滚动
            WindowEvent::MouseWheel { delta, .. } => {
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::MouseWheel, delta);
                }
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
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::Translate, (0.0, 30.0));
                }
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
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::Translate, (0.0, -30.0));
                }
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
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::Translate, (-30.0, 0.0));
                }
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
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::Translate, (30.0, 0.0));
                }
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
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::Resized, new_size);
                }
            }
            WindowEvent::RedrawRequested => {
                println!("RedrawRequested");
                let s = self.window_id_sender_map.get(&window_id);
                if let Some(sender) = s {
                    send_message(sender.clone(), MessageId::RedrawRequested, true);
                }
            }
            WindowEvent::CloseRequested => {
                // let s = self.window_id_sender_map.get(&window_id);
                // if let Some(sender) = s {
                //     // sender.send(msg)
                // }
                self.window_id_sender_map.remove(&window_id);
                self.window_id_receiver_map.remove(&window_id);
                if self.window_id_sender_map.is_empty() {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}
