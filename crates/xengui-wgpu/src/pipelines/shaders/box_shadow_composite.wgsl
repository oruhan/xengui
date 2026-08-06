// SPDX-License-Identifier: Apache-2.0
// Composites a pre-blurred box-shadow coverage mask onto the scene.
// Outset: straight sample of the blurred mask, tinted by the shadow color.
// Inset: only visible through the box itself, growing inward from its
// edge - the box's own hard-edged rounded rect minus the blurred coverage
// of the offset/spread-shrunk inner shape.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) box_radius: f32,
    @location(3) mask_uv: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) inset: f32,
};

@group(0) @binding(0) var mask_tex: texture_2d<f32>;
@group(0) @binding(1) var mask_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) box_radius: f32,
    @location(4) mask_uv: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(6) inset: f32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.local_pos = local_pos;
    out.half_size = half_size;
    out.box_radius = box_radius;
    out.mask_uv = mask_uv;
    out.color = color;
    out.inset = inset;
    return out;
}

fn sd_round_rect(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) - r + min(max(q.x, q.y), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let in_bounds = select(
        0.0,
        1.0,
        in.mask_uv.x >= 0.0 &&
        in.mask_uv.x <= 1.0 &&
        in.mask_uv.y >= 0.0 &&
        in.mask_uv.y <= 1.0
    );

    let mask_alpha = textureSample(mask_tex, mask_sampler, in.mask_uv).a * in_bounds;

    // Compute derivatives outside of any control flow.
    let d = sd_round_rect(in.local_pos, in.half_size, in.box_radius);
    let aa = max(fwidth(d) * 0.5, 0.0001);
    let box_alpha = 1.0 - smoothstep(-aa, aa, d);

    let outset_alpha = mask_alpha;
    let inset_alpha = box_alpha * (1.0 - mask_alpha);

    let alpha = select(outset_alpha, inset_alpha, in.inset > 0.5);

    if alpha <= 0.0 {
        discard;
    }

    let out_alpha = in.color.a * alpha;
    return vec4<f32>(in.color.rgb * out_alpha, out_alpha);
}