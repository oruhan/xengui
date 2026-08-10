// SPDX-License-Identifier: Apache-2.0
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_length: f32,
    @location(2) half_thickness: f32,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_length: f32,
    @location(3) half_thickness: f32,
    @location(4) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.local_pos = local_pos;
    out.half_length = half_length;
    out.half_thickness = half_thickness;
    out.color = color;
    return out;
}

// Distance to a capsule (rounded line segment) in segment-local space,
// where the segment runs along the x axis from -half_length to +half_length.
fn sd_capsule(p: vec2<f32>, half_length: f32, radius: f32) -> f32 {
    let cx = clamp(p.x, -half_length, half_length);
    let d = vec2<f32>(p.x - cx, p.y);
    return length(d) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = sd_capsule(in.local_pos, in.half_length, in.half_thickness);
    let aa = max(fwidth(d) * 0.5, 0.0001);
    let alpha = 1.0 - smoothstep(-aa, aa, d);
    if alpha <= 0.0 {
        discard;
    }
    let out_alpha = in.color.a * alpha;
    return vec4<f32>(in.color.rgb * out_alpha, out_alpha);
}