struct VertexInput {

}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn pbr_vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);

    return out;
}

@fragment
fn pbr_fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
