// a simple sprite shader for 2D
struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    @location(1) mm_0: vec4<f32>,
    @location(2) mm_1: vec4<f32>,
    @location(3) mm_2: vec4<f32>,
    @location(4) mm_3: vec4<f32>,

    @location(5) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.mm_0,
        instance.mm_1,
        instance.mm_2,
        instance.mm_3,
    );

    var out: VertexOutput;

    out.clip_position = model_matrix * vec4<f32>(vertex.position, 0.0, 1.0);
    out.color = instance.color.xyz;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
