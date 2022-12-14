pub mod dcel;
pub mod m4;
pub mod render;
pub mod state;

extern crate rand;

use dcel::MeshCoverage;
use state::State;
use std::{collections::HashMap, fs, path::PathBuf};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::state::Vertex;

fn generateVecF64FromBytes(bytes: Vec<u8>) -> Vec<f64> {
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

fn generateVecU32FromBytes(bytes: Vec<u8>) -> Vec<u32> {
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

fn readNodeFile(root_path: String, cov_id: String) -> Vec<f64> {
    let mut node_path_buf = PathBuf::new();
    node_path_buf.push(root_path);
    node_path_buf.push("Geometry");
    node_path_buf.push("Mesh");
    node_path_buf.push(cov_id);
    node_path_buf.set_extension("node");
    println!("{:#?}", node_path_buf);
    let node_buff = fs::read(node_path_buf).unwrap();
    let coordinate_buff: Vec<f64> = generateVecF64FromBytes(node_buff);
    coordinate_buff
}

fn readFaceFile(root_path: String, cov_id: String) -> Vec<u32> {
    let mut face_path_buf = PathBuf::new();
    face_path_buf.push(root_path);
    face_path_buf.push("Geometry");
    face_path_buf.push("Mesh");
    face_path_buf.push(cov_id);
    face_path_buf.set_extension("face");
    let face_buff = fs::read(face_path_buf).unwrap();
    let index_buff: Vec<u32> = generateVecU32FromBytes(face_buff);
    index_buff
}

fn load_mesh(root_path: String, id: String, coverage: &mut MeshCoverage) {
    // 读取.node文件
    println!("load node file...");
    let node_buff = readNodeFile(root_path.clone(), id.clone());
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
    let face_buff = readFaceFile(root_path.clone(), id.clone());
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

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // State::new 使用了异步代码，所以我们需等待其完成
    let mut state = pollster::block_on(State::new(&window));

    let root_path = String::from(r"C:\Users\Administrator\.modelingstudio\2022-11-3");
    let id = String::from("d12473a1-0977-463a-a050-2735df012cea");

    // let root_path = String::from(r"C:\Users\Administrator\.modelingstudio\福州");
    // let id = String::from("85c63f29-c349-48f9-952f-b8fe242ebcd3");

    load_mesh(root_path, id, &mut state.coverage);
    let bbox3 = state.coverage.get_bbox3();
    println!("bbox3:{:#?}", bbox3,);
    let c_x = (bbox3.min_x + bbox3.max_x) / 2.0;
    let c_y = (bbox3.min_y + bbox3.max_y) / 2.0;
    let rang_x = bbox3.max_x - bbox3.min_x;
    let rang_y = bbox3.max_y - bbox3.min_y;
    let rang_z = bbox3.max_z - bbox3.min_z;
    println!("c_x:{c_x},c_y:{c_y},rang_x:{rang_x},rang_y:{rang_y},rang_z:{rang_z}");

    let mut vertices: Vec<Vertex> = Vec::with_capacity(state.coverage.node_map.len());
    let mut map: HashMap<u32, u32> = HashMap::new();
    let mut i = 0;
    for (&id, &node) in &state.coverage.node_map {
        vertices.push(Vertex {
            position: [(node.x - c_x) as f32, (node.y - c_y) as f32, node.z as f32],
            id,
        });
        map.insert(id, i.clone());
        i = i + 1;
    }
    let ids: Vec<u32> = state.coverage.generate_half_edge_buffer();
    // let ids: Vec<u32> = state.coverage.generate_face_buffer();
    let len_ids = ids.len();
    let mut indices: Vec<u32> = Vec::with_capacity(len_ids);
    for i in 0..len_ids {
        indices.push(*map.get(&ids[i]).unwrap());
    }

    // let vertices: Vec<Vertex> = [
    //     Vertex {
    //         position: [-50.0, 50.0, 0.0],
    //         id: 1,
    //     },
    //     Vertex {
    //         position: [-50.0, -50.0, 0.0],
    //         id: 2,
    //     },
    //     Vertex {
    //         position: [50.0, -50.0, 0.0],
    //         id: 3,
    //     },
    //     Vertex {
    //         position: [50.0, 50.0, 0.0],
    //         id: 3,
    //     },
    // ]
    // .to_vec();
    // let indices: Vec<u32> = [0, 1, 1, 2, 2, 3, 3, 0].to_vec();
    state.setdata(vertices, indices);

    event_loop.run(move |event, _, control_flow| match event {
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
    });
}
