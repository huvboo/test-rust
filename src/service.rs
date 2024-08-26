use crate::{
    dcel::MeshCoverage,
    layer::{Layer, Vertex},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path, path::PathBuf};
use wgpu::Device;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageJSON {
    pub id: String,
    pub name: String,
    #[serde(rename = "module")]
    pub module_name: String,
    #[serde(rename = "type")]
    pub coverage_type: String,
}

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

pub struct Service {}

impl Service {
    pub fn load_mesh(root_path: &Path, id: String, coverage: &mut MeshCoverage) {
        // 读取.node文件
        println!("load node file...");
        let node_buff = read_node_file(root_path, id.clone());
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
        let face_buff = read_face_file(root_path, id.clone());
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

    pub fn set_test_data(device: &Device, layer: &mut Layer) {
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

        layer.setdata(vertices, indices, device);
    }

    pub fn set_mesh_data(device: &Device, coverage: &mut MeshCoverage, layer: &mut Layer) {
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

        layer.setdata(vertices, indices, device);
    }

    pub fn read_grmsp_coverage_file(path_buf: &PathBuf) -> Vec<CoverageJSON> {
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
                coverages
            }
            Err(_) => [].to_vec(),
        }
    }
}
