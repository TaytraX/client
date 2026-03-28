struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_position: vec3<f32>,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    _padding: f32,
    color: vec3<f32>,
    _padding2: f32,
}

@group(1) @binding(0)
var<uniform> light: Light;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    return out;
}

const AMBIENT_STRENGTH: f32 = 0.1;
const SPECULAR_STRENGTH: f32 = 0.5;
const SHININESS: f32 = 32.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = vec4<f32>(0.1, 0.6, 0.2, 1.0);

    let dx = dpdx(in.world_position);
    let dy = dpdy(in.world_position);
    let world_normal = normalize(cross(dx, dy));

    let ambient_color = light.color * AMBIENT_STRENGTH;

    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(camera.view_position - in.world_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(world_normal, half_dir), 0.0), SHININESS);
    let specular_color = specular_strength * light.color * SPECULAR_STRENGTH;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}