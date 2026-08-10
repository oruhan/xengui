// SPDX-License-Identifier: Apache-2.0
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@group(0) @binding(0) var t_glyph: texture_2d<f32>;
@group(0) @binding(1) var s_glyph: sampler;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tint: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    out.tint = tint;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coverage = textureSample(t_glyph, s_glyph, in.uv).r;
    let alpha = coverage * in.tint.a;
    return vec4<f32>(in.tint.rgb * alpha, alpha);
}