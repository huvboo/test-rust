pub mod app;
pub mod dcel;
pub mod layer;
pub mod m4;
pub mod message;
pub mod render;
pub mod scene;
pub mod service;
pub mod state;
pub mod wgpu_ctx;

use app::App;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();
    // 创建事件循环
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("run app error.");
}
