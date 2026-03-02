struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
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

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    _padding: f32,
    color: vec3<f32>,
    _padding2: f32,
}

@group(2) @binding(0)
var<uniform> light: Light;

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var t_normal: texture_2d<f32>;
@group(0) @binding(3) var s_normal: sampler;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // Calculer la matrice normale (inverse transpose de la partie 3x3 de model_matrix)
    // Pour les transformations uniformes, on peut utiliser directement la partie 3x3
    let normal_matrix = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    // Transformer la position en world space
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    
    // Transformer la normale en world space et la normaliser
    out.world_normal = normalize(normal_matrix * model.normal);
    
    // Calculer le vecteur tangent et bitangent pour le normal mapping
    // On utilise une méthode simple basée sur la normale
    let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(out.world_normal.y) < 0.999);
    out.tangent = normalize(cross(up, out.world_normal));
    out.bitangent = normalize(cross(out.world_normal, out.tangent));
    
    // Calculer la position finale en clip space
    out.clip_position = camera.view_proj * world_position;

    return out;
}

// Paramètres d'éclairage configurables
const AMBIENT_STRENGTH: f32 = 0.1;
const SPECULAR_STRENGTH: f32 = 0.5;
const SHININESS: f32 = 32.0;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // Échantillonner la normal map et convertir de [0,1] à [-1,1]
    let normal_map = textureSample(t_normal, s_normal, in.tex_coords).xyz;
    let tangent_normal = normalize(normal_map * 2.0 - 1.0);
    
    // Construire la matrice TBN pour transformer la normale du tangent space au world space
    let T = normalize(in.tangent);
    let B = normalize(in.bitangent);
    let N = normalize(in.world_normal);
    let TBN = mat3x3<f32>(T, B, N);
    
    // Transformer la normale de tangent space à world space
    let world_normal = normalize(TBN * tangent_normal);
    
    // Calcul de l'éclairage avec la normale de la normal map
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