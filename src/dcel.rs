use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

static GLOBAL_NODE_ID_COUNTER: Mutex<u32> = Mutex::new(0);
static GLOBAL_FACE_ID_COUNTER: Mutex<u32> = Mutex::new(0);
static GLOBAL_HALF_EDGE_ID_COUNTER: Mutex<u32> = Mutex::new(0);

const MAX_ID: u32 = u32::MAX;

pub fn generate_id(counter: &Mutex<u32>) -> u32 {
    let current_val = counter.lock().unwrap().clone();
    if current_val > MAX_ID {
        panic!("Factory ids overflowed");
    }
    let next_id = current_val + 1;
    *counter.lock().unwrap() = next_id;
    next_id
}

fn generate_node_id() -> u32 {
    generate_id(&GLOBAL_NODE_ID_COUNTER)
}

fn generate_face_id() -> u32 {
    generate_id(&GLOBAL_FACE_ID_COUNTER)
}

fn generate_half_edge_id() -> u32 {
    generate_id(&GLOBAL_HALF_EDGE_ID_COUNTER)
}

#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Node {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub n0: u32,
    pub n1: u32,
    pub n2: u32,
    pub n3: u32,
}

impl Face {
    fn new(n0: u32, n1: u32, n2: u32, n3: u32) -> Self {
        Self { n0, n1, n2, n3 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HalfEdge {
    pub start_id: u32,
    pub end_id: u32,
    pub face_id: u32,
    pub prev_id: u32,
    pub next_id: u32,
    pub twin_id: u32,
}

impl HalfEdge {
    fn new(
        start_id: u32,
        end_id: u32,
        face_id: u32,
        prev_id: u32,
        next_id: u32,
        twin_id: u32,
    ) -> Self {
        Self {
            start_id,
            end_id,
            face_id,
            prev_id,
            next_id,
            twin_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeFaceAdj {
    map: HashMap<u32, HashSet<u32>>,
}

impl NodeFaceAdj {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn bind(&mut self, node_id: u32, face_id: u32) {
        if let Some(set) = self.map.get_mut(&node_id) {
            set.insert(face_id);
        } else {
            let mut set = HashSet::new();
            set.insert(face_id);
            self.map.insert(node_id, set);
        }
    }

    pub fn unbind(&mut self, node_id: u32, face_id: u32) {
        if let Some(set) = self.map.get_mut(&node_id) {
            set.remove(&face_id);
        }
    }

    pub fn remove_node(&mut self, node_id: u32) {
        self.map.remove(&node_id);
    }

    pub fn get_node_adj_faces(&self, node_id: u32) -> Option<&HashSet<u32>> {
        self.map.get(&node_id)
    }
}

#[derive(Debug, Clone)]
pub struct FaceHalfEdgeAdj {
    map: HashMap<u32, HashSet<u32>>,
}

impl FaceHalfEdgeAdj {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn bind(&mut self, face_id: u32, half_edge_id: u32) {
        if let Some(set) = self.map.get_mut(&face_id) {
            set.insert(half_edge_id);
        } else {
            let mut set = HashSet::new();
            set.insert(half_edge_id);
            self.map.insert(face_id, set);
        }
    }

    pub fn unbind(&mut self, face_id: u32, half_edge_id: u32) {
        if let Some(set) = self.map.get_mut(&face_id) {
            set.remove(&half_edge_id);
        }
    }

    pub fn remove_face(&mut self, face_id: u32) {
        self.map.remove(&face_id);
    }

    pub fn get_face_adj_half_edges(&self, face_id: u32) -> Option<&HashSet<u32>> {
        self.map.get(&face_id)
    }
}

#[derive(Debug, Clone)]
pub struct BBox3 {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl BBox3 {
    pub fn new() -> Self {
        Self {
            min_x: f64::MAX,
            max_x: f64::MIN,
            min_y: f64::MAX,
            max_y: f64::MIN,
            min_z: f64::MAX,
            max_z: f64::MIN,
        }
    }

    pub fn eat(&mut self, node: Node) {
        if node.x < self.min_x {
            self.min_x = node.x;
        }
        if node.x > self.max_x {
            self.max_x = node.x;
        }
        if node.y < self.min_y {
            self.min_y = node.y;
        }
        if node.y > self.max_y {
            self.max_y = node.y;
        }
        if node.z < self.min_z {
            self.min_z = node.z;
        }
        if node.z > self.max_z {
            self.max_z = node.z;
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshCoverage {
    pub id: String,
    pub node_map: HashMap<u32, Node>,
    pub face_map: HashMap<u32, Face>,
    pub half_edge_map: HashMap<u32, HalfEdge>,
    pub node_face_adj: NodeFaceAdj,
    pub face_half_edge_adj: FaceHalfEdgeAdj,
}

impl MeshCoverage {
    pub fn new(id: String) -> Self {
        Self {
            id,
            node_map: HashMap::new(),
            face_map: HashMap::new(),
            half_edge_map: HashMap::new(),
            node_face_adj: NodeFaceAdj::new(),
            face_half_edge_adj: FaceHalfEdgeAdj::new(),
        }
    }

    pub fn query_node_by_id(&mut self, id: u32) -> Option<&mut Node> {
        self.node_map.get_mut(&id)
    }

    pub fn query_face_by_id(&mut self, id: u32) -> Option<&mut Face> {
        self.face_map.get_mut(&id)
    }

    pub fn query_half_edge_by_id(&mut self, id: u32) -> Option<&mut HalfEdge> {
        self.half_edge_map.get_mut(&id)
    }

    pub fn create_node(&mut self, x: f64, y: f64, z: f64) -> u32 {
        let new_id = generate_node_id();
        let new_node = Node::new(x, y, z);
        self.node_map.insert(new_id, new_node);
        new_id
    }

    pub fn create_face(&mut self, n0: u32, n1: u32, n2: u32, n3: u32) -> u32 {
        let new_id = generate_face_id();

        // 绑定点
        self.node_face_adj.bind(n0, new_id);
        self.node_face_adj.bind(n1, new_id);
        self.node_face_adj.bind(n2, new_id);
        if n3 > 0 {
            self.node_face_adj.bind(n3, new_id);
        }

        let new_face = Face::new(n0, n1, n2, n3);
        self.face_map.insert(new_id, new_face);
        new_id
    }

    pub fn generate_half_edges(&mut self) {
        for (id, face) in &mut self.face_map {
            let n0 = face.n0.clone();
            let n1 = face.n1.clone();
            let n2 = face.n2.clone();
            let n3 = face.n3.clone();
            let id = id.clone();
            // 创建半边
            // self.create_face_half_edges(n0, n1, n2, n3, id);
        }
    }

    pub fn create_face_half_edges(&mut self, n0: u32, n1: u32, n2: u32, n3: u32, face_id: u32) {
        let half_edge_0_id = generate_half_edge_id();
        let half_edge_1_id = generate_half_edge_id();
        let half_edge_2_id = generate_half_edge_id();
        let mut last_half_edge_id = half_edge_2_id;
        if n3 > 0 {
            let half_edge_3_id = generate_half_edge_id();
            last_half_edge_id = half_edge_3_id;
            self.create_half_edge(
                half_edge_3_id,
                n3,
                n0,
                face_id,
                half_edge_2_id,
                half_edge_0_id,
            );
        }
        self.create_half_edge(
            half_edge_0_id,
            n0,
            n1,
            face_id,
            last_half_edge_id,
            half_edge_1_id,
        );
        self.create_half_edge(
            half_edge_1_id,
            n1,
            n2,
            face_id,
            half_edge_0_id,
            half_edge_2_id,
        );
        if n3 > 0 {
            self.create_half_edge(
                half_edge_2_id,
                n2,
                n3,
                face_id,
                half_edge_1_id,
                last_half_edge_id,
            );
        } else {
            self.create_half_edge(
                half_edge_2_id,
                n2,
                n0,
                face_id,
                half_edge_1_id,
                half_edge_0_id,
            );
        }
    }

    pub fn create_half_edge(
        &mut self,
        id: u32,
        start_id: u32,
        end_id: u32,
        face_id: u32,
        prev_id: u32,
        next_id: u32,
    ) {
        self.face_half_edge_adj.bind(face_id, id);
        let half_edge = HalfEdge::new(
            start_id,
            end_id,
            face_id,
            prev_id,
            next_id,
            self.find_twin(start_id, end_id, face_id),
        );
        self.half_edge_map.insert(id, half_edge);
    }

    pub fn find_twin(&mut self, start_id: u32, end_id: u32, face_id: u32) -> u32 {
        if let Some(face_set) = self.node_face_adj.get_node_adj_faces(start_id) {
            for &id in face_set {
                if id != face_id {
                    if let Some(half_edge_set) = self.face_half_edge_adj.get_face_adj_half_edges(id)
                    {
                        for &id2 in half_edge_set {
                            if let Some(half_edge) = self.half_edge_map.get(&id2) {
                                if half_edge.start_id == end_id && half_edge.end_id == start_id {
                                    return id2;
                                }
                            }
                        }
                    }
                }
            }
        }
        0
    }

    pub fn remove_node(&mut self, node_id: u32) {
        self.node_map.remove(&node_id);
        // 删除关联的三角形及半边
        if let Some(set) = self.node_face_adj.get_node_adj_faces(node_id) {
            for &id in set {
                self.face_map.remove(&id);
                self.face_half_edge_adj.remove_face(id);
            }
        }
        self.node_face_adj.remove_node(node_id);
    }

    pub fn remove_face(&mut self, face_id: u32) {
        // 解绑点
        if let Some(face) = self.face_map.get(&face_id) {
            self.node_face_adj.unbind(face.n0, face_id);
            self.node_face_adj.unbind(face.n1, face_id);
            self.node_face_adj.unbind(face.n2, face_id);
            if face.n3 > 0 {
                self.node_face_adj.unbind(face.n3, face_id)
            };
        }
        // 删除半边
        self.face_map.remove(&face_id);
        self.face_half_edge_adj.remove_face(face_id);
    }

    pub fn generate_buffer(&self) -> (Vec<f32>, Vec<u32>) {
        let mut coordinates: Vec<f32> = Vec::new();
        let mut indexes: Vec<u32> = Vec::new();
        let mut map: HashMap<u32, u32> = HashMap::new();
        let mut i = 0;
        for (&id, node) in &self.node_map {
            coordinates.push(node.x as f32);
            coordinates.push(node.y as f32);
            coordinates.push(node.z as f32);
            map.insert(id, i.clone());
            i = i + 1;
        }
        for id in &self.generate_half_edge_buffer() {
            indexes.push(*map.get(id).unwrap());
        }
        (coordinates, indexes)
    }

    pub fn generate_half_edge_buffer(&self) -> Vec<u32> {
        let mut indexes: Vec<u32> = Vec::new();
        // for (&id, _face) in &self.face_map {
        //     if let Some(half_edge_set) = self.face_half_edge_adj.get_face_adj_half_edges(id) {
        //         for half_edge_id in &*half_edge_set {
        //             if let Some(&half_edge) = self.half_edge_map.get(&half_edge_id) {
        //                 indexes.push(half_edge.start_id);
        //                 indexes.push(half_edge.end_id);
        //             }
        //         }
        //     }
        // }

        for (&_id, face) in &self.face_map {
            if face.n3 > 0 {
                indexes.push(face.n0);
                indexes.push(face.n1);
                indexes.push(face.n1);
                indexes.push(face.n2);
                indexes.push(face.n2);
                indexes.push(face.n3);
                indexes.push(face.n3);
                indexes.push(face.n0);
            } else {
                indexes.push(face.n0);
                indexes.push(face.n1);
                indexes.push(face.n1);
                indexes.push(face.n2);
                indexes.push(face.n2);
                indexes.push(face.n0);
            }
        }

        indexes
    }

    pub fn generate_face_buffer(&self) -> Vec<u32> {
        let mut indexes: Vec<u32> = Vec::new();
        for (&_id, face) in &self.face_map {
            if face.n3 > 0 {
                indexes.push(face.n0);
                indexes.push(face.n1);
                indexes.push(face.n2);
                indexes.push(face.n2);
                indexes.push(face.n3);
                indexes.push(face.n0);
            } else {
                indexes.push(face.n0);
                indexes.push(face.n1);
                indexes.push(face.n2);
            }
        }
        indexes
    }

    pub fn get_bbox3(&mut self) -> BBox3 {
        let mut bbox3 = BBox3::new();
        for (_id, &node) in &self.node_map {
            bbox3.eat(node);
        }
        bbox3
    }
}
