use std::collections::HashMap;

use crate::{layer::Layer, service::Service, win_ctx::WinCtx};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App<'window> {
    is_first_resumed: bool,
    window_id_context_map: HashMap<WindowId, WinCtx<'window>>,
}

impl<'window> App<'window> {
    pub fn new() -> App<'window> {
        let window_id_context_map: HashMap<WindowId, WinCtx<'window>> = HashMap::new();
        App {
            is_first_resumed: true,
            window_id_context_map,
        }
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("1111111111 {}", self.is_first_resumed);
        if self.is_first_resumed {
            self.is_first_resumed = false;
            let win_attr = Window::default_attributes().with_title("WebGPU Example");
            let window = event_loop
                .create_window(win_attr)
                .expect("create window err.");

            let mut win_ctx = WinCtx::new(window);

            let mut test_layer = Layer::new(
                String::from(""),
                &win_ctx.state,
                &win_ctx.wgpu_ctx.device,
                &win_ctx.wgpu_ctx.surface_config,
            );
            Service::set_test_data(&win_ctx.wgpu_ctx.device, &mut test_layer);
            win_ctx.state.add_layer(test_layer);

            self.window_id_context_map
                .insert(win_ctx.window_id, win_ctx);
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
                if let Some(win_ctx) = self.window_id_context_map.get_mut(&window_id) {
                    win_ctx.drop_file(path_buf);
                }
            }

            // 鼠标移动
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(win_ctx) = self.window_id_context_map.get_mut(&window_id) {
                    win_ctx.mouse_move(position);
                }
            }

            // 鼠标滚动
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(win_ctx) = self.window_id_context_map.get_mut(&window_id) {
                    win_ctx.mouse_wheel(delta);
                }
            }

            WindowEvent::Resized(new_size) => {
                if let Some(win_ctx) = self.window_id_context_map.get_mut(&window_id) {
                    win_ctx.resize(new_size);
                }
            }

            // 上
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             physical_key: PhysicalKey::Code(KeyCode::KeyW | KeyCode::ArrowUp),
            //             ..
            //         },
            //     ..
            // } => {
            //     // let s = self.window_id_sender_map.get(&window_id);
            //     if let Some(sender) = &self.sender_main {
            //         send_message(sender.clone(), MessageId::Translate, (0.0, 30.0));
            //     }
            // }

            // // 下
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             physical_key: PhysicalKey::Code(KeyCode::KeyS | KeyCode::ArrowDown),
            //             ..
            //         },
            //     ..
            // } => {
            //     // let s = self.window_id_sender_map.get(&window_id);
            //     if let Some(sender) = &self.sender_main {
            //         send_message(sender.clone(), MessageId::Translate, (0.0, -30.0));
            //     }
            // }

            // // 左
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             physical_key: PhysicalKey::Code(KeyCode::KeyA | KeyCode::ArrowLeft),
            //             ..
            //         },
            //     ..
            // } => {
            //     // let s = self.window_id_sender_map.get(&window_id);
            //     if let Some(sender) = &self.sender_main {
            //         send_message(sender.clone(), MessageId::Translate, (-30.0, 0.0));
            //     }
            // }

            // // 右
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             physical_key: PhysicalKey::Code(KeyCode::KeyD | KeyCode::ArrowRight),
            //             ..
            //         },
            //     ..
            // } => {
            //     // let s = self.window_id_sender_map.get(&window_id);
            //     if let Some(sender) = &self.sender_main {
            //         send_message(sender.clone(), MessageId::Translate, (30.0, 0.0));
            //     }
            // }
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

            WindowEvent::RedrawRequested => {
                if let Some(win_ctx) = self.window_id_context_map.get_mut(&window_id) {
                    win_ctx.redraw();
                }
                // let r = self.window_id_receiver_map.get(&window_id);
                // if let Some(receiver) = &self.recevier_main {
                //     let result = receiver.try_recv();
                //     match result {
                //         Ok(value) => match value.message_id {
                //             MessageId::CloseRequested => {
                //                 let close_event = self.window_event(
                //                     event_loop,
                //                     window_id,
                //                     WindowEvent::CloseRequested,
                //                 );
                //                 self.user_event(event_loop, close_event);
                //             }
                //             MessageId::CreateChildWindow => {
                //                 create_window(self, event_loop);
                //             }
                //             _ => {}
                //         },
                //         Err(_) => {}
                //     }
                // }
            }
            WindowEvent::CloseRequested => {
                println!("CloseRequested");
                self.window_id_context_map.remove(&window_id);
                if self.window_id_context_map.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::Focused(bool) => {
                println!("id: {:?}, focus: {}", window_id.clone(), bool);
            }
            _ => {}
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}
