// SPDX-License-Identifier: Apache-2.0
struct BlitParams {
    tint: vec4<f32>,
    tint_mix: f32,
    offset: vec2<f32>,
    _pad: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BlitParams;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let sample_uv = uv + params.offset;
    var src = vec4<f32>(0.0);
    if (sample_uv.x >= 0.0 && sample_uv.x <= 1.0 && sample_uv.y >= 0.0 && sample_uv.y <= 1.0) {
        src = textureSample(src_tex, src_sampler, sample_uv);
    }
    let silhouette = vec4<f32>(params.tint.rgb * src.a * params.tint.a, src.a * params.tint.a);
    return mix(src, silhouette, params.tint_mix);
}