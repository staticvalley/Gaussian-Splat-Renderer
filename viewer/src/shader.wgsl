struct CameraUniform {
    view_projection: mat4x4f,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexShaderInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) color: vec3f,
}

struct VertexShaderOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
}

@vertex
fn vs_main(
    model: VertexShaderInput,
) -> VertexShaderOutput {
    var output: VertexShaderOutput;
    output.color = model.color;
    output.clip_position = camera.view_projection * vec4f(model.position, 1.0);
    return output;
}

@fragment
fn fs_main(in: VertexShaderOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}