// One axis of a separable Gaussian blur. Run once with direction=(1,0)
// then once more with direction=(0,1) on the result to get a full 2D blur
// in O(2*radius) samples instead of O(radius^2).

struct BlurParams {
    direction: vec2<f32>,
    radius: f32,
    _pad: f32,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> params: BlurParams;

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(src_tex));
    let texel = params.direction / dims;
    let sigma = max(params.radius / 3.0, 0.0001);
    let taps = i32(ceil(params.radius));

    var sum = vec4<f32>(0.0);
    var weight_sum = 0.0;

    for (var i = -taps; i <= taps; i = i + 1) {
        let fi = f32(i);
        let w = exp(-(fi * fi) / (2.0 * sigma * sigma));
        sum = sum + textureSample(src_tex, src_sampler, uv + texel * fi) * w;
        weight_sum = weight_sum + w;
    }

    return sum / weight_sum;
}