struct VertexShaderInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexShaderOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexShaderInput,
) -> VertexShaderOutput {
    var output: VertexShaderOutput;
    output.color = model.color;
    output.clip_position = vec4<f32>(model.position, 1.0);
    return output;
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}