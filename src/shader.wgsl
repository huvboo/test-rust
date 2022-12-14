
struct ColorUniform {
    rgba: vec4<f32>;
}
;[[group(0), binding(0)]]
var<uniform> color: ColorUniform;

struct CameraUniform {
    model_view_proj: mat4x4<f32>;
};
;[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] id: u32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

;[[stage(vertex)]]
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.model_view_proj * vec4<f32>(model.position, 1.0);
    return out;
}


;[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return color.rgba;
}
