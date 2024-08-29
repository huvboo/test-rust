use crate::{
    dcel::BBox3,
    m4::{multiply, projection, scale, translate, x_rotate, y_rotate, z_rotate},
};
use cgmath::num_traits::Float;

pub struct Scene {
    w: f64,                   // 画布宽，像素值
    h: f64,                   // 画布高，像素值
    mouse_position: [f64; 2], // 鼠标指针相对于可视窗口的 X 轴 Y轴的距离 单位：像素
    bbox: BBox3,              // 可视区域坐标包围盒
    center: [f64; 3],         // 包围盒的中心点
    min_zoom: f64,            // 最小缩放值，初始值 0.8 ** 50
    max_zoom: f64,            // 最大缩放值，初始值 1.25 ** 22
    _zoom: f64,               // 上一次的缩放值
    zoom: f64,                // 当前缩放值
    tx: f64,                  // X轴平移的距离（单位1）而非屏幕像素
    ty: f64,                  // Y轴平移的距离（单位1）而非屏幕像素
    tz: f64,                  // Z轴平移的距离（单位1）而非屏幕像素
    rx: f64,                  // 绕X轴旋转的角度
    ry: f64,                  // 绕Y轴旋转的角度
    rz: f64,                  // 绕Z轴旋转的角度
}

impl Scene {
    pub fn new(w: f64, h: f64) -> Self {
        Self {
            w,
            h,
            mouse_position: [0.0, 0.0],
            bbox: BBox3::new(),
            center: [0.0, 0.0, 0.0],
            min_zoom: 0.8.powi(50),
            max_zoom: 1.25.powi(22),
            _zoom: 1.0,
            zoom: 1.0,
            tx: 0.0,
            ty: 0.0,
            tz: 0.0,
            rx: 0.0,
            ry: 0.0,
            rz: 0.0,
        }
    }

    pub fn size(&self) -> [f64; 2] {
        [self.w, self.h]
    }

    pub fn resize(&mut self, w: f64, h: f64) {
        self.w = w;
        self.h = h;
    }

    pub fn mouse_position(&self) -> [f64; 2] {
        self.mouse_position
    }

    pub fn set_mouse_position(&mut self, vec2: [f64; 2]) {
        self.mouse_position = vec2;
    }

    pub fn get_mat4(&self) -> [[f32; 4]; 4] {
        let far = 1000000000.0;
        let mut matrix = projection(self.w, self.h, far * self.zoom);

        matrix = translate(
            matrix,
            self.tx * self.zoom,
            self.ty * self.zoom,
            -self.w / 2.0, // state.transform.tz * state.transform.scale
        );

        matrix = x_rotate(matrix, self.rx);
        matrix = y_rotate(matrix, self.ry);
        matrix = z_rotate(matrix, self.rz);

        matrix = scale(matrix, self.zoom, self.zoom, self.zoom);
        matrix = multiply(
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
            ],
            matrix,
        );
        return [
            [
                matrix[0] as f32,
                matrix[1] as f32,
                matrix[2] as f32,
                matrix[3] as f32,
            ],
            [
                matrix[4] as f32,
                matrix[5] as f32,
                matrix[6] as f32,
                matrix[7] as f32,
            ],
            [
                matrix[8] as f32,
                matrix[9] as f32,
                matrix[10] as f32,
                matrix[11] as f32,
            ],
            [
                matrix[12] as f32,
                matrix[13] as f32,
                matrix[14] as f32,
                matrix[15] as f32,
            ],
        ];
    }

    pub fn scale_on_mouse_wheel(&mut self, times: f64) {
        self.scale(times);

        let ds = 1.0 - self.zoom / self._zoom;
        let dx = (self.mouse_position[0] - self.w / 2.0) * ds;
        let dy = (self.h / 2.0 - self.mouse_position[1]) * ds;
        self.translate(dx, dy);
    }

    pub fn scale(&mut self, times: f64) {
        let multiplier = 1.25.powf(times);
        // println!("{:?}", multiplier);
        self._zoom = self.zoom;
        self.zoom = (self.zoom * multiplier)
            .min(self.max_zoom)
            .max(self.min_zoom);
    }

    pub fn translate(&mut self, tx: f64, ty: f64) {
        self.tx = self.tx + tx / self.zoom;
        self.ty = self.ty + ty / self.zoom;
    }
}
