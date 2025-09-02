struct Camera {
    position: vec4<f32>,
    forward: vec4<f32>,
    up: vec4<f32>,
    right: vec4<f32>,
    aspect: f32,
}

var<push_constant> camera: Camera;

struct SceneInfo {
    hyper_spheres_count: u32,
}

@group(0) @binding(0)
var<uniform> info: SceneInfo;

struct HyperSphere {
    position: vec4<f32>,
    color: vec3<f32>,
    radius: f32,
}

@group(1) @binding(0)
var<storage, read> hyper_spheres: array<HyperSphere>;

@compute @workgroup_size(16, 16, 1)
fn ray_trace() { }
