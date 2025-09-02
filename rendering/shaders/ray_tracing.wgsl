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
    hyperplanes_count: u32,
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

struct Hyperplane {
    transform: Transform,
    color: vec3<f32>,
    width: f32,
    height: f32,
    depth: f32,
}

@group(2) @binding(1)
var<storage, read> hyperplanes: array<Hyperplane>;

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

fn intersect_hypersphere(ray: Ray, hypersphere: Hypersphere) -> Hit {
    var hit: Hit;
    hit.hit = false;

    let oc = hypersphere.position - ray.origin;
    // TODO: can this be replaced with 1?
    let a = dot(ray.direction, ray.direction);
    let h = dot(ray.direction, oc);
    let c = dot(oc, oc) - hypersphere.radius * hypersphere.radius;
    let discriminant = h * h - a * c;

    if discriminant >= 0.0 {
        hit.distance = (h - sqrt(discriminant)) / a;
        if hit.distance > 0.0 {
            hit.hit = true;
            hit.position = ray.origin + ray.direction * hit.distance;
            hit.normal = (hit.position - hypersphere.position) / hypersphere.radius;
            hit.color = hypersphere.color;
        }
    }

    return hit;
}

fn intersect_hyperplane(ray: Ray, hyperplane: Hyperplane) -> Hit {
    var hit: Hit;
    hit.hit = false;

    let reverse_transform = transform_reverse(hyperplane.transform);

    var transformed_ray: Ray;
    transformed_ray.origin = transform_point(reverse_transform, ray.origin);
    transformed_ray.direction = transform_direction(reverse_transform, ray.direction);

    if sign(transformed_ray.origin.y) == sign(transformed_ray.direction.y) {
        return hit;
    }

    hit.distance = abs(transformed_ray.origin.y / transformed_ray.direction.y);

    let relative_point = transformed_ray.origin + transformed_ray.direction * hit.distance;
    if abs(relative_point.x) > hyperplane.height * 0.5 {
        return hit;
    }
    if abs(relative_point.z) > hyperplane.width * 0.5 {
        return hit;
    }
    if abs(relative_point.w) > hyperplane.depth * 0.5 {
        return hit;
    }

    hit.hit = true;
    hit.position = ray.origin + ray.direction * hit.distance;
    hit.normal = transform_y(hyperplane.transform) * sign(transformed_ray.origin.y);
    hit.color = hyperplane.color;
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

    for (var i = 0u; i < info.hyperplanes_count; i++) {
        let hit = intersect_hyperplane(ray, hyperplanes[i]);
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

struct Transform {
    s: f32,
    e0e1: f32,
    e0e2: f32,
    e0e3: f32,
    e0e4: f32,
    e1e2: f32,
    e1e3: f32,
    e1e4: f32,
    e2e3: f32,
    e2e4: f32,
    e3e4: f32,
    e0e1e2e3: f32,
    e0e1e2e4: f32,
    e0e1e3e4: f32,
    e0e2e3e4: f32,
    e1e2e3e4: f32,
}

fn transform_reverse(transform: Transform) -> Transform {
    let _0 = transform.s;
    let _1 = transform.e0e1;
    let _2 = transform.e0e2;
    let _3 = transform.e0e3;
    let _4 = transform.e0e4;
    let _5 = transform.e1e2;
    let _6 = transform.e1e3;
    let _7 = transform.e1e4;
    let _8 = transform.e2e3;
    let _9 = transform.e2e4;
    let _10 = transform.e3e4;
    let _11 = transform.e0e1e2e3;
    let _12 = transform.e0e1e2e4;
    let _13 = transform.e0e1e3e4;
    let _14 = transform.e0e2e3e4;
    let _15 = transform.e1e2e3e4;
    var result: Transform;
    result.s = (_0);
    result.e0e1 = (- 1.0 * _1);
    result.e0e2 = (- 1.0 * _2);
    result.e0e3 = (- 1.0 * _3);
    result.e0e4 = (- 1.0 * _4);
    result.e1e2 = (- 1.0 * _5);
    result.e1e3 = (- 1.0 * _6);
    result.e1e4 = (- 1.0 * _7);
    result.e2e3 = (- 1.0 * _8);
    result.e2e4 = (- 1.0 * _9);
    result.e3e4 = (- 1.0 * _10);
    result.e0e1e2e3 = (_11);
    result.e0e1e2e4 = (_12);
    result.e0e1e3e4 = (_13);
    result.e0e2e3e4 = (_14);
    result.e1e2e3e4 = (_15);
    return result;
}

fn transform_point(transform: Transform, point: vec4<f32>) -> vec4<f32> {
    let _0 = transform.s;
    let _1 = transform.e0e1;
    let _2 = transform.e0e2;
    let _3 = transform.e0e3;
    let _4 = transform.e0e4;
    let _5 = transform.e1e2;
    let _6 = transform.e1e3;
    let _7 = transform.e1e4;
    let _8 = transform.e2e3;
    let _9 = transform.e2e4;
    let _10 = transform.e3e4;
    let _11 = transform.e0e1e2e3;
    let _12 = transform.e0e1e2e4;
    let _13 = transform.e0e1e3e4;
    let _14 = transform.e0e2e3e4;
    let _15 = transform.e1e2e3e4;
    let _16 = point.x;
    let _17 = point.y;
    let _18 = point.z;
    let _19 = point.w;
    var result: vec4<f32>;
    result.x = (_16) + (- 2.0 * _14 * _15) + (- 2.0 * _2 * _5) + (- 2.0 * _3 * _6) + (- 2.0 * _4 * _7) + (2.0 * _0 * _1) + (2.0 * _10 * _13) + (2.0 * _11 * _8) + (2.0 * _12 * _9) + (- 2.0 * _0 * _17 * _5) + (- 2.0 * _0 * _18 * _6) + (- 2.0 * _0 * _19 * _7) + (- 2.0 * _10 * _15 * _17) + (- 2.0 * _10 * _18 * _7) + (- 2.0 * _15 * _15 * _16) + (- 2.0 * _15 * _19 * _8) + (- 2.0 * _16 * _5 * _5) + (- 2.0 * _16 * _6 * _6) + (- 2.0 * _16 * _7 * _7) + (- 2.0 * _17 * _6 * _8) + (- 2.0 * _17 * _7 * _9) + (2.0 * _10 * _19 * _6) + (2.0 * _15 * _18 * _9) + (2.0 * _18 * _5 * _8) + (2.0 * _19 * _5 * _9);
    result.y = (_17) + (- 2.0 * _11 * _6) + (- 2.0 * _12 * _7) + (- 2.0 * _3 * _8) + (- 2.0 * _4 * _9) + (2.0 * _0 * _2) + (2.0 * _1 * _5) + (2.0 * _10 * _14) + (2.0 * _13 * _15) + (- 2.0 * _0 * _18 * _8) + (- 2.0 * _0 * _19 * _9) + (- 2.0 * _10 * _18 * _9) + (- 2.0 * _15 * _15 * _17) + (- 2.0 * _15 * _18 * _7) + (- 2.0 * _16 * _6 * _8) + (- 2.0 * _16 * _7 * _9) + (- 2.0 * _17 * _5 * _5) + (- 2.0 * _17 * _8 * _8) + (- 2.0 * _17 * _9 * _9) + (- 2.0 * _18 * _5 * _6) + (- 2.0 * _19 * _5 * _7) + (2.0 * _0 * _16 * _5) + (2.0 * _10 * _15 * _16) + (2.0 * _10 * _19 * _8) + (2.0 * _15 * _19 * _6);
    result.z = (_18) + (- 2.0 * _10 * _4) + (- 2.0 * _12 * _15) + (- 2.0 * _13 * _7) + (- 2.0 * _14 * _9) + (2.0 * _0 * _3) + (2.0 * _1 * _6) + (2.0 * _11 * _5) + (2.0 * _2 * _8) + (- 2.0 * _0 * _10 * _19) + (- 2.0 * _10 * _10 * _18) + (- 2.0 * _10 * _16 * _7) + (- 2.0 * _10 * _17 * _9) + (- 2.0 * _15 * _15 * _18) + (- 2.0 * _15 * _16 * _9) + (- 2.0 * _15 * _19 * _5) + (- 2.0 * _17 * _5 * _6) + (- 2.0 * _18 * _6 * _6) + (- 2.0 * _18 * _8 * _8) + (- 2.0 * _19 * _6 * _7) + (- 2.0 * _19 * _8 * _9) + (2.0 * _0 * _16 * _6) + (2.0 * _0 * _17 * _8) + (2.0 * _15 * _17 * _7) + (2.0 * _16 * _5 * _8);
    result.w = (_19) + (2.0 * _0 * _4) + (2.0 * _1 * _7) + (2.0 * _10 * _3) + (2.0 * _11 * _15) + (2.0 * _12 * _5) + (2.0 * _13 * _6) + (2.0 * _14 * _8) + (2.0 * _2 * _9) + (- 2.0 * _10 * _10 * _19) + (- 2.0 * _15 * _15 * _19) + (- 2.0 * _15 * _17 * _6) + (- 2.0 * _17 * _5 * _7) + (- 2.0 * _18 * _6 * _7) + (- 2.0 * _18 * _8 * _9) + (- 2.0 * _19 * _7 * _7) + (- 2.0 * _19 * _9 * _9) + (2.0 * _0 * _10 * _18) + (2.0 * _0 * _16 * _7) + (2.0 * _0 * _17 * _9) + (2.0 * _10 * _16 * _6) + (2.0 * _10 * _17 * _8) + (2.0 * _15 * _16 * _8) + (2.0 * _15 * _18 * _5) + (2.0 * _16 * _5 * _9);
    return result;
}

fn transform_direction(transform: Transform, direction: vec4<f32>) -> vec4<f32> {
    let _0 = transform.s;
    let _1 = transform.e1e2;
    let _2 = transform.e1e3;
    let _3 = transform.e1e4;
    let _4 = transform.e2e3;
    let _5 = transform.e2e4;
    let _6 = transform.e3e4;
    let _7 = transform.e1e2e3e4;
    let _8 = direction.x;
    let _9 = direction.y;
    let _10 = direction.z;
    let _11 = direction.w;
    var result: vec4<f32>;
    result.x = (_8) + (- 2.0 * _0 * _1 * _9) + (- 2.0 * _0 * _10 * _2) + (- 2.0 * _0 * _11 * _3) + (- 2.0 * _1 * _1 * _8) + (- 2.0 * _10 * _3 * _6) + (- 2.0 * _11 * _4 * _7) + (- 2.0 * _2 * _2 * _8) + (- 2.0 * _2 * _4 * _9) + (- 2.0 * _3 * _3 * _8) + (- 2.0 * _3 * _5 * _9) + (- 2.0 * _6 * _7 * _9) + (- 2.0 * _7 * _7 * _8) + (2.0 * _1 * _10 * _4) + (2.0 * _1 * _11 * _5) + (2.0 * _10 * _5 * _7) + (2.0 * _11 * _2 * _6);
    result.y = (_9) + (- 2.0 * _0 * _10 * _4) + (- 2.0 * _0 * _11 * _5) + (- 2.0 * _1 * _1 * _9) + (- 2.0 * _1 * _10 * _2) + (- 2.0 * _1 * _11 * _3) + (- 2.0 * _10 * _3 * _7) + (- 2.0 * _10 * _5 * _6) + (- 2.0 * _2 * _4 * _8) + (- 2.0 * _3 * _5 * _8) + (- 2.0 * _4 * _4 * _9) + (- 2.0 * _5 * _5 * _9) + (- 2.0 * _7 * _7 * _9) + (2.0 * _0 * _1 * _8) + (2.0 * _11 * _2 * _7) + (2.0 * _11 * _4 * _6) + (2.0 * _6 * _7 * _8);
    result.z = (_10) + (- 2.0 * _0 * _11 * _6) + (- 2.0 * _1 * _11 * _7) + (- 2.0 * _1 * _2 * _9) + (- 2.0 * _10 * _2 * _2) + (- 2.0 * _10 * _4 * _4) + (- 2.0 * _10 * _6 * _6) + (- 2.0 * _10 * _7 * _7) + (- 2.0 * _11 * _2 * _3) + (- 2.0 * _11 * _4 * _5) + (- 2.0 * _3 * _6 * _8) + (- 2.0 * _5 * _6 * _9) + (- 2.0 * _5 * _7 * _8) + (2.0 * _0 * _2 * _8) + (2.0 * _0 * _4 * _9) + (2.0 * _1 * _4 * _8) + (2.0 * _3 * _7 * _9);
    result.w = (_11) + (- 2.0 * _1 * _3 * _9) + (- 2.0 * _10 * _2 * _3) + (- 2.0 * _10 * _4 * _5) + (- 2.0 * _11 * _3 * _3) + (- 2.0 * _11 * _5 * _5) + (- 2.0 * _11 * _6 * _6) + (- 2.0 * _11 * _7 * _7) + (- 2.0 * _2 * _7 * _9) + (2.0 * _0 * _10 * _6) + (2.0 * _0 * _3 * _8) + (2.0 * _0 * _5 * _9) + (2.0 * _1 * _10 * _7) + (2.0 * _1 * _5 * _8) + (2.0 * _2 * _6 * _8) + (2.0 * _4 * _6 * _9) + (2.0 * _4 * _7 * _8);
    return result;
}

fn transform_y(transform: Transform) -> vec4<f32> {
    let _0 = transform.s;
    let _1 = transform.e1e2;
    let _2 = transform.e1e3;
    let _3 = transform.e1e4;
    let _4 = transform.e2e3;
    let _5 = transform.e2e4;
    let _6 = transform.e3e4;
    let _7 = transform.e1e2e3e4;
    var result: vec4<f32>;
    result.x = (- 2.0 * _0 * _1) + (- 2.0 * _2 * _4) + (- 2.0 * _3 * _5) + (- 2.0 * _6 * _7);
    result.y = (1.0) + (- 2.0 * _1 * _1) + (- 2.0 * _4 * _4) + (- 2.0 * _5 * _5) + (- 2.0 * _7 * _7);
    result.z = (- 2.0 * _1 * _2) + (- 2.0 * _5 * _6) + (2.0 * _0 * _4) + (2.0 * _3 * _7);
    result.w = (- 2.0 * _1 * _3) + (- 2.0 * _2 * _7) + (2.0 * _0 * _5) + (2.0 * _4 * _6);
    return result;
}
