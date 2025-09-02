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
    hyperspheres_count: u32,
}

@group(1) @binding(0)
var<uniform> info: SceneInfo;

struct Hypersphere {
    position: vec4<f32>,
    color: vec3<f32>,
    radius: f32,
}

@group(2) @binding(0)
var<storage, read> hyperspheres: array<Hypersphere>;

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

fn intersect_hypersphere(ray: Ray, hyper_sphere: Hypersphere) -> Hit {
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

    for (var i = 0u; i < info.hyperspheres_count; i++) {
        let hit = intersect_hypersphere(ray, hyperspheres[i]);
        if hit.hit && (!closest_hit.hit || hit.distance < closest_hit.distance) {
            closest_hit = hit;
        }
    }

    return closest_hit;
}

const SUN_DIRECTION: vec4<f32> = vec4<f32>(- 0.1, 1.0, 0.3, 0.1);

fn sky_color(ray: Ray) -> vec3<f32> {
    let up = vec3<f32>(0.4, 0.5, 0.8);
    let down = vec3<f32>(0.2, 0.2, 0.3);
    if dot(ray.direction, normalize(SUN_DIRECTION)) > 0.99 {
        return vec3<f32>(1.0);
    }
    return mix(down, up, ray.direction.y * 0.5 + 0.5);
}

fn trace_ray(ray: Ray) -> vec3<f32> {
    let hit = intersect_scene(ray);
    if hit.hit {
        var sun_ray: Ray;
        sun_ray.origin = hit.position + hit.normal * 0.001;
        sun_ray.direction = normalize(SUN_DIRECTION);

        let sun_hit = intersect_scene(sun_ray);

        return hit.color * max(0.1, f32(!sun_hit.hit) * dot(hit.normal, sun_ray.direction));
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
