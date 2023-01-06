pub mod dcel;
pub mod m4;
pub mod render;
pub mod state;

extern crate rand;

extern crate websocket;
use serde::{Deserialize, Serialize};
use std::{path::Path, thread};
use websocket::sync::Server;
use websocket::Message;

use dcel::MeshCoverage;
use state::{Layer, State};
use std::{collections::HashMap, fs, path::PathBuf};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::state::Vertex;

fn generate_vec_f64_from_bytes(bytes: Vec<u8>) -> Vec<f64> {
    let len = bytes.len() / 8;
    let mut vec_f64: Vec<f64> = Vec::with_capacity(bytes.len() / 8);
    for i in 0..len {
        vec_f64.push(f64::from_le_bytes([
            bytes[i * 8 + 0],
            bytes[i * 8 + 1],
            bytes[i * 8 + 2],
            bytes[i * 8 + 3],
            bytes[i * 8 + 4],
            bytes[i * 8 + 5],
            bytes[i * 8 + 6],
            bytes[i * 8 + 7],
        ]));
    }
    vec_f64
}

fn generate_vec_u32_from_bytes(bytes: Vec<u8>) -> Vec<u32> {
    let len = bytes.len() / 4;
    let mut vec_u32: Vec<u32> = Vec::with_capacity(len);
    for i in 0..len {
        vec_u32.push(u32::from_le_bytes([
            bytes[i * 4 + 0],
            bytes[i * 4 + 1],
            bytes[i * 4 + 2],
            bytes[i * 4 + 3],
        ]));
    }
    vec_u32
}

fn read_node_file(root_path: &Path, cov_id: String) -> Vec<f64> {
    let mut node_path_buf = PathBuf::new();
    node_path_buf.push(root_path);
    node_path_buf.push("Geometry");
    node_path_buf.push("Mesh");
    node_path_buf.push(cov_id);
    node_path_buf.set_extension("node");
    println!("{:#?}", node_path_buf);
    let node_buff = fs::read(node_path_buf).unwrap();
    let coordinate_buff: Vec<f64> = generate_vec_f64_from_bytes(node_buff);
    coordinate_buff
}

fn read_face_file(root_path: &Path, cov_id: String) -> Vec<u32> {
    let mut face_path_buf = PathBuf::new();
    face_path_buf.push(root_path);
    face_path_buf.push("Geometry");
    face_path_buf.push("Mesh");
    face_path_buf.push(cov_id);
    face_path_buf.set_extension("face");
    let face_buff = fs::read(face_path_buf).unwrap();
    let index_buff: Vec<u32> = generate_vec_u32_from_bytes(face_buff);
    index_buff
}

fn load_mesh(root_path: &Path, id: String, coverage: &mut MeshCoverage) {
    // 读取.node文件
    println!("load node file...");
    let node_buff = read_node_file(root_path.clone(), id.clone());
    let len_node = node_buff.len() / 3;
    for n in 0..len_node {
        coverage.create_node(
            node_buff[n * 3 + 0],
            node_buff[n * 3 + 1],
            node_buff[n * 3 + 2],
        );
    }
    // 读取.face文件
    println!("load face file...");
    let face_buff = read_face_file(root_path.clone(), id.clone());
    let len_face = face_buff.len() / 4;
    for n in 0..len_face {
        coverage.create_face(
            face_buff[n * 4 + 0],
            face_buff[n * 4 + 1],
            face_buff[n * 4 + 2],
            face_buff[n * 4 + 3],
        );
    }
    println!("mesh coverage loaded.");
}

fn set_test_data(state: &mut State, layer: &mut Layer) {
    let vertices: Vec<Vertex> = [
        Vertex {
            position: [-50.0, 50.0, 0.0],
            id: 1,
        },
        Vertex {
            position: [-50.0, -50.0, 0.0],
            id: 2,
        },
        Vertex {
            position: [50.0, -50.0, 0.0],
            id: 3,
        },
        Vertex {
            position: [50.0, 50.0, 0.0],
            id: 3,
        },
    ]
    .to_vec();
    let indices: Vec<u32> = [0, 1, 1, 2, 2, 3, 3, 0].to_vec();

    layer.setdata(vertices, indices, state);
}

fn set_mesh_data(state: &mut State, coverage: &mut MeshCoverage, layer: &mut Layer) {
    let bbox3 = coverage.get_bbox3();
    println!("bbox3:{:#?}", bbox3,);
    let c_x = (bbox3.min_x + bbox3.max_x) / 2.0;
    let c_y = (bbox3.min_y + bbox3.max_y) / 2.0;
    let rang_x = bbox3.max_x - bbox3.min_x;
    let rang_y = bbox3.max_y - bbox3.min_y;
    let rang_z = bbox3.max_z - bbox3.min_z;
    println!("c_x:{c_x},c_y:{c_y},rang_x:{rang_x},rang_y:{rang_y},rang_z:{rang_z}");

    let mut vertices: Vec<Vertex> = Vec::with_capacity(coverage.node_map.len());
    let mut map: HashMap<u32, u32> = HashMap::new();
    let mut i = 0;
    for (&id, &node) in &coverage.node_map {
        vertices.push(Vertex {
            position: [(node.x - c_x) as f32, (node.y - c_y) as f32, node.z as f32],
            id,
        });
        map.insert(id, i.clone());
        i = i + 1;
    }
    let ids: Vec<u32> = coverage.generate_half_edge_buffer();
    // let ids: Vec<u32> = state.coverage.generate_face_buffer();
    let len_ids = ids.len();
    let mut indices: Vec<u32> = Vec::with_capacity(len_ids);
    for i in 0..len_ids {
        indices.push(*map.get(&ids[i]).unwrap());
    }

    layer.setdata(vertices, indices, state);
}

fn event_handler(
    event: Event<()>,
    control_flow: &mut ControlFlow,
    window: &Window,
    state: &mut State,
) {
    match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    WindowEvent::DroppedFile(path_buf) => {
                        let ext = path_buf.extension().unwrap();
                        if ext.to_str() == Some("grmsp") {
                            let coverages = read_grmsp_coverage_file(path_buf);
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
                                    load_mesh(dir, coverage.id.clone(), &mut mesh_coverage);
                                    let mut layer = Layer::new(coverage.id.clone(), state);
                                    set_mesh_data(state, &mut mesh_coverage, &mut layer);
                                    state.add_coverage(mesh_coverage);
                                    state.add_layer(layer);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // 如果发生上下文丢失，就重新配置 surface
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // 系统内存不足，此时应该退出
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // 所有其他错误（如过时、超时等）都应在下一帧解决
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // 除非手动请求，否则 RedrawRequested 只会触发一次
            window.request_redraw();
        }
        _ => {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CoverageJSON {
    id: String,
    name: String,
    #[serde(rename = "module")]
    module_name: String,
    #[serde(rename = "type")]
    coverage_type: String,
}

fn read_grmsp_coverage_file(path_buf: &PathBuf) -> Vec<CoverageJSON> {
    let name = path_buf.file_name().unwrap();
    let stem = path_buf.file_stem().unwrap();
    let dir = path_buf.parent().unwrap();
    println!("name:{:#?},stem:{:#?},dir:{:#?}", name, stem, dir);
    let mut coverage_path_buf = dir.to_path_buf();
    coverage_path_buf.push("coverage");
    coverage_path_buf.set_extension("json");
    let result = fs::read_to_string(coverage_path_buf);
    match result {
        Ok(s) => {
            println!("content->{}", s);
            let vec: Vec<serde_json::Value> = serde_json::from_str(&s).unwrap();
            let mut coverages = Vec::new();
            for item in vec {
                println!("{:#?}", item);
                let coverage: CoverageJSON = serde_json::from_value(item).unwrap();
                println!("{:#?}", coverage);
                // let id: String = item["id"].to_string();
                // let name: String = item["name"].to_string();
                // let module_name: String = item["module"].to_string();
                // let coverage_type: String = item["type"].to_string();
                // let coverage = CoverageJSON {
                //     id,
                //     name,
                //     module_name,
                //     coverage_type,
                // };
                coverages.push(coverage)
            }
            return coverages;
        }
        Err(_) => todo!(),
    }
}

fn test_ws() {
    let server = Server::bind("127.0.0.1:1234").unwrap();

    for connection in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            let mut client = connection.accept().unwrap();

            // let message = Message::text("Hello, client!");
            let message = Message::binary::<&[u8]>(&[1, 2, 3]);
            let _ = client.send_message(&message);

            // ...
        });
    }
}

// fn main() {
//     env_logger::init();

//     // test_ws();

//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();

//     // State::new 使用了异步代码，所以我们需等待其完成
//     let mut state = pollster::block_on(State::new(&window));

//     let mut test_layer = Layer::new(String::from(""), &mut state);
//     set_test_data(&mut state, &mut test_layer);
//     state.add_layer(test_layer);

//     event_loop
//         .run(move |event, _, control_flow| event_handler(event, control_flow, &window, &mut state));
// }

use druid::shell;
use druid::widget::{Button, Flex, Label};
use druid::{
    AppLauncher, Application, DruidHandler, LocalizedString, PlatformError, Widget, WidgetExt,
    WindowDesc, WindowHandle,
};

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(ui_builder);
    let data = 0_u32;
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
}

fn ui_builder() -> impl Widget<u32> {
    // The label text will be computed dynamically based on the current locale and count
    let text =
        LocalizedString::new("hello-counter").with_arg("count", |data: &u32, _env| (*data).into());
    let label = Label::new(text).padding(5.0).center();
    let button = Button::new("加")
        .on_click(|_ctx, data, _env| *data += 1)
        .padding(5.0);

    Flex::column().with_child(label).with_child(button)
}
