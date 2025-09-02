struct Camera {
    position: vec4<f32>,
    forward: vec4<f32>,
    up: vec4<f32>,
    right: vec4<f32>,
}

var<push_constant> camera: Camera;

@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba32float, write>;

struct SceneInfo {
    hyper_spheres_count: u32,
}

@group(1) @binding(0)
var<uniform> info: SceneInfo;

struct HyperSphere {
    position: vec4<f32>,
    color: vec3<f32>,
    radius: f32,
}

@group(2) @binding(0)
var<storage, read> hyper_spheres: array<HyperSphere>;

@compute @workgroup_size(16, 16, 1)
fn ray_trace(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = textureDimensions(output_texture);
    let coords = global_id.xy;

    if coords.x >= size.x || coords.y >= size.y {
        return;
    }

    let aspect = f32(size.x) / f32(size.y);
    let uv = ((vec2<f32>(coords) + 0.5) / vec2<f32>(size)) * 2.0 - 1.0;

    let color = vec3<f32>(uv * 0.5 + 0.5, 0.0);
    textureStore(output_texture, coords, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
