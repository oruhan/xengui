// SPDX-License-Identifier: Apache-2.0
// Rasterizes a single filled rounded rect as a straight-alpha coverage
// mask - the Dual Kawase pass that follows operates on this directly, so
// its output alpha already carries the CSS box-shadow blur.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) radius: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.local_pos = local_pos;
    out.half_size = half_size;
    out.radius = radius;
    return out;
}

fn corner_radius(p: vec2<f32>, radii: vec4<f32>) -> f32 {
    if p.x < 0.0 {
        if p.y < 0.0 {
            return radii.x;
        }
        return radii.w;
    }
    if p.y < 0.0 {
        return radii.y;
    }
    return radii.z;
}

fn sd_round_rect(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) - r + min(max(q.x, q.y), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = corner_radius(in.local_pos, in.radius);
    let d = sd_round_rect(in.local_pos, in.half_size, r);
    let aa = max(fwidth(d) * 0.5, 0.0001);
    let coverage = 1.0 - smoothstep(-aa, aa, d);
    return vec4<f32>(coverage, coverage, coverage, coverage);
}