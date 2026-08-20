// SPDX-License-Identifier: Apache-2.0
struct BlitParams {
    tint: vec4<f32>,
    tint_mix: f32,
    _pad0: f32,
    offset: vec2<f32>,
    scale: vec2<f32>,
    dest_pos: vec2<f32>,
    dest_half_size: vec2<f32>,
    radius: vec4<f32>,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BlitParams;

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
fn fs_main(@builtin(position) frag_coord: vec4<f32>, @location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let sample_uv = uv * params.scale + params.offset;
    let in_bounds = select(
        0.0,
        1.0,
        sample_uv.x >= 0.0 && sample_uv.x <= 1.0 && sample_uv.y >= 0.0 && sample_uv.y <= 1.0
    );
    var src = textureSample(src_tex, src_sampler, sample_uv) * in_bounds;

    if params.radius.x + params.radius.y + params.radius.z + params.radius.w > 0.0 {
        let local = frag_coord.xy - params.dest_pos;
        let r = corner_radius(local, params.radius);
        let d = sd_round_rect(local, params.dest_half_size, r);
        let mask = 1.0 - smoothstep(-1.0, 1.0, d);
        src = src * mask;
    }

    let silhouette = vec4<f32>(params.tint.rgb * src.a * params.tint.a, src.a * params.tint.a);
    return mix(src, silhouette, params.tint_mix);
}