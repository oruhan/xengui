// SPDX-License-Identifier: Apache-2.0
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: f32,
    @location(3) border_width: f32,
    @location(4) fill_color: vec4<f32>,
    @location(5) border_color: vec4<f32>,
    @location(6) gradient_meta: vec4<f32>,
    @location(7) gradient_positions: vec4<f32>,
    @location(8) gradient_color0: vec4<f32>,
    @location(9) gradient_color1: vec4<f32>,
    @location(10) gradient_color2: vec4<f32>,
    @location(11) gradient_color3: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) half_size: vec2<f32>,
    @location(3) radius: f32,
    @location(4) border_width: f32,
    @location(5) fill_color: vec4<f32>,
    @location(6) border_color: vec4<f32>,
    @location(7) gradient_meta: vec4<f32>,
    @location(8) gradient_positions: vec4<f32>,
    @location(9) gradient_color0: vec4<f32>,
    @location(10) gradient_color1: vec4<f32>,
    @location(11) gradient_color2: vec4<f32>,
    @location(12) gradient_color3: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    out.local_pos = local_pos;
    out.half_size = half_size;
    out.radius = radius;
    out.border_width = border_width;
    out.fill_color = fill_color;
    out.border_color = border_color;
    out.gradient_meta = gradient_meta;
    out.gradient_positions = gradient_positions;
    out.gradient_color0 = gradient_color0;
    out.gradient_color1 = gradient_color1;
    out.gradient_color2 = gradient_color2;
    out.gradient_color3 = gradient_color3;
    return out;
}

fn sd_round_rect(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) - r + min(max(q.x, q.y), 0.0);
}

fn sample_gradient(t: f32, o: VertexOutput) -> vec4<f32> {
    let count = i32(o.gradient_meta.z);
    let positions = o.gradient_positions;
    let colors = array<vec4<f32>, 4>(
        o.gradient_color0,
        o.gradient_color1,
        o.gradient_color2,
        o.gradient_color3
    );

    if (count <= 1) {
        return colors[0];
    }

    let tc = clamp(t, positions[0], positions[count - 1]);

    for (var i = 0; i < count - 1; i = i + 1) {
        if (tc >= positions[i] && tc <= positions[i + 1]) {
            let span = max(positions[i + 1] - positions[i], 0.0001);
            let local_t = (tc - positions[i]) / span;
            return mix(colors[i], colors[i + 1], local_t);
        }
    }

    return colors[count - 1];
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = sd_round_rect(in.local_pos, in.half_size, in.radius);
    let aa = max(fwidth(d) * 0.75, 0.0001);

    let outer_alpha = 1.0 - smoothstep(-aa, aa, d);
    if (outer_alpha <= 0.0) {
        discard;
    }

    var fill = in.fill_color;
    let kind = in.gradient_meta.x;

    if (kind > 0.5) {
        var t: f32;
        if (kind < 1.5) {
            // Linear: angle 0 points along +x, matching CSS gradient-angle convention.
            let dir = vec2<f32>(cos(in.gradient_meta.y), sin(in.gradient_meta.y));
            let extent = abs(dir.x) * in.half_size.x + abs(dir.y) * in.half_size.y;
            t = (dot(in.local_pos, dir) / max(extent * 2.0, 0.0001)) + 0.5;
        } else {
            let max_dist = length(in.half_size);
            t = length(in.local_pos) / max(max_dist, 0.0001);
        }
        fill = sample_gradient(t, in);
    }

    var color = fill;
    if (in.border_width > 0.0) {
        let inner_d = d + in.border_width;
        let inner_mask = 1.0 - smoothstep(-aa, aa, inner_d);
        color = mix(in.border_color, fill, inner_mask);
    }

    color.a = color.a * outer_alpha;
    return color;
}