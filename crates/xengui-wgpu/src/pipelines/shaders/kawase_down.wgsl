// Dual Kawase downsample: 5-tap box average at half resolution, offset by
// half a source texel so each tap lands between four source pixels.
struct KawaseParams {
    offset: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> params: KawaseParams;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let texel = params.offset / vec2<f32>(textureDimensions(src_tex));

    var sum = textureSample(src_tex, src_sampler, uv) * 4.0;
    sum += textureSample(src_tex, src_sampler, uv - texel);
    sum += textureSample(src_tex, src_sampler, uv + texel);
    sum += textureSample(src_tex, src_sampler, uv + vec2<f32>(texel.x, -texel.y));
    sum += textureSample(src_tex, src_sampler, uv - vec2<f32>(texel.x, -texel.y));

    return sum / 8.0;
}