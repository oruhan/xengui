// Combined single-pass color filter: applies every pointwise (non-spatial)
// filter in the chain sequentially, in list order, matching CSS filter
// composition semantics while costing exactly one texture sample.

struct FilterOp {
    kind: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    params: vec4<f32>,
};

struct FilterOps {
    count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    ops: array<FilterOp, 8>,
};

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> filters: FilterOps;

const KIND_BRIGHTNESS: u32 = 0u;
const KIND_CONTRAST: u32 = 1u;
const KIND_SATURATE: u32 = 2u;
const KIND_GRAYSCALE: u32 = 3u;
const KIND_HUE_ROTATE: u32 = 4u;
const KIND_INVERT: u32 = 5u;
const KIND_OPACITY: u32 = 6u;
const KIND_GAMMA: u32 = 7u;

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

// CSS Filter Effects Module hue-rotate matrix.
fn hue_rotate(c: vec3<f32>, degrees: f32) -> vec3<f32> {
    let rad = radians(degrees);
    let cs = cos(rad);
    let sn = sin(rad);
    let r0 = vec3<f32>(0.213 + cs * 0.787 - sn * 0.213, 0.715 - cs * 0.715 - sn * 0.715, 0.072 - cs * 0.072 + sn * 0.928);
    let r1 = vec3<f32>(0.213 - cs * 0.213 + sn * 0.143, 0.715 + cs * 0.285 + sn * 0.140, 0.072 - cs * 0.072 - sn * 0.283);
    let r2 = vec3<f32>(0.213 - cs * 0.213 - sn * 0.787, 0.715 - cs * 0.715 + sn * 0.715, 0.072 + cs * 0.928 + sn * 0.072);
    return vec3<f32>(dot(r0, c), dot(r1, c), dot(r2, c));
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    var color = textureSample(src_tex, src_sampler, uv);

    for (var i = 0u; i < filters.count; i = i + 1u) {
        let op = filters.ops[i];
        switch op.kind {
            case KIND_BRIGHTNESS: {
                color = vec4<f32>(color.rgb * op.params.x, color.a);
            }
            case KIND_CONTRAST: {
                color = vec4<f32>((color.rgb - vec3<f32>(0.5)) * op.params.x + vec3<f32>(0.5), color.a);
            }
            case KIND_SATURATE: {
                let gray = dot(color.rgb, LUMA);
                color = vec4<f32>(mix(vec3<f32>(gray), color.rgb, op.params.x), color.a);
            }
            case KIND_GRAYSCALE: {
                let gray = dot(color.rgb, LUMA);
                color = vec4<f32>(mix(color.rgb, vec3<f32>(gray), op.params.x), color.a);
            }
            case KIND_HUE_ROTATE: {
                color = vec4<f32>(hue_rotate(color.rgb, op.params.x), color.a);
            }
            case KIND_INVERT: {
                color = vec4<f32>(mix(color.rgb, vec3<f32>(1.0) - color.rgb, op.params.x), color.a);
            }
            case KIND_OPACITY: {
                color = vec4<f32>(color.rgb, color.a * op.params.x);
            }
            case KIND_GAMMA: {
                let g = max(op.params.x, 0.0001);
                color = vec4<f32>(pow(max(color.rgb, vec3<f32>(0.0)), vec3<f32>(1.0 / g)), color.a);
            }
            default: {}
        }
    }

    return vec4<f32>(color.rgb * color.a, color.a); // premultiply for correct blend on composite
}