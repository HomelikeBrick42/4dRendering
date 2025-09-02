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

struct Ray {
    origin: vec4<f32>,
    direction: vec4<f32>,
}

struct Hit {
    hit: bool,
    distance: f32,
    position: vec4<f32>,
    normal: vec4<f32>,
    color: vec3<f32>,
}

fn intersect_hyper_sphere(ray: Ray, hyper_sphere: HyperSphere) -> Hit {
    var hit: Hit;
    hit.hit = false;

    let oc = hyper_sphere.position - ray.origin;
    // TODO: can this be replaced with 1?
    let a = dot(ray.direction, ray.direction);
    let h = dot(ray.direction, oc);
    let c = dot(oc, oc) - hyper_sphere.radius * hyper_sphere.radius;
    let discriminant = h * h - a * c;

    if discriminant >= 0.0 {
        hit.distance = (h - sqrt(discriminant)) / a;
        if hit.distance > 0.0 {
            hit.hit = true;
            hit.position = ray.origin + ray.direction * hit.distance;
            hit.normal = (hit.position - hyper_sphere.position) / hyper_sphere.radius;
            hit.color = hyper_sphere.color;
        }
    }

    return hit;
}

fn intersect_scene(ray: Ray) -> Hit {
    var closest_hit: Hit;
    closest_hit.hit = false;

    for (var i = 0u; i < info.hyper_spheres_count; i++) {
        let hit = intersect_hyper_sphere(ray, hyper_spheres[i]);
        if hit.hit && (!closest_hit.hit || hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }

    return closest_hit;
}

fn sky_color(ray: Ray) -> vec3<f32> {
    let up = vec3<f32>(0.4, 0.5, 0.8);
    let down = vec3<f32>(0.2, 0.2, 0.3);
    return mix(down, up, ray.direction.y * 0.5 + 0.5);
}

fn trace_ray(ray: Ray) -> vec3<f32> {
    let hit = intersect_scene(ray);
    if hit.hit {
        return hit.color * max(0.1, dot(hit.normal, normalize(vec4<f32>(0.1, 1.0, 0.3, 0.4))));
    }
    else {
        return sky_color(ray);
    }
}

@compute @workgroup_size(16, 16, 1)
fn ray_trace(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = textureDimensions(output_texture);
    let coords = global_id.xy;

    if coords.x >= size.x || coords.y >= size.y {
        return;
    }

    let aspect = f32(size.x) / f32(size.y);
    let uv = ((vec2<f32>(coords) + 0.5) / vec2<f32>(size)) * 2.0 - 1.0;

    var ray: Ray;
    ray.origin = camera.position;
    ray.direction = normalize(camera.forward + camera.up * uv.y + camera.right * uv.x * aspect);

    let color = trace_ray(ray);
    textureStore(output_texture, coords, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
