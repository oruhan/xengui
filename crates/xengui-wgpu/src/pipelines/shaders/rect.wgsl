// SPDX-License-Identifier: Apache-2.0
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: vec4<f32>, // top-left, top-right, bottom-right, bottom-left
    @location(3) border_width: f32,
    @location(4) fill_color: vec4<f32>,
    @location(5) border_color: vec4<f32>,
    @location(6) gradient_meta: vec4<f32>,
};

struct GradientPositions {
    values: array<vec4<f32>, 128>,
};

struct GradientColors {
    values: array<vec4<f32>, 512>,
};

@group(0) @binding(0) var<uniform> grad_positions: GradientPositions;
@group(0) @binding(1) var<uniform> grad_colors: GradientColors;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) screen_rect: vec4<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: vec4<f32>,
    @location(3) border_width: f32,
    @location(4) fill_color: vec4<f32>,
    @location(5) border_color: vec4<f32>,
    @location(6) gradient_meta: vec4<f32>,
) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let corner = corners[vertex_index];
    let outer_half_size = half_size + vec2<f32>(1.5, 1.5);
    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        mix(screen_rect.x, screen_rect.z, corner.x),
        mix(screen_rect.y, screen_rect.w, corner.y),
        0.0,
        1.0,
    );
    out.local_pos = mix(-outer_half_size, outer_half_size, corner);
    out.half_size = half_size;
    out.radius = radius;
    out.border_width = border_width;
    out.fill_color = fill_color;
    out.border_color = border_color;
    out.gradient_meta = gradient_meta;
    return out;
}

// Selects the correct corner radius for `p` (in box-local, center-origin
// space) out of the four independent corner radii, matching CSS
// border-radius's per-quadrant selection.
fn corner_radius(p: vec2<f32>, radii: vec4<f32>) -> f32 {
    if (p.x < 0.0) {
        if (p.y < 0.0) {
            return radii.x; // top-left
        }
        return radii.w; // bottom-left
    }
    if (p.y < 0.0) {
        return radii.y; // top-right
    }
    return radii.z; // bottom-right
}

fn sd_round_rect(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) - r + min(max(q.x, q.y), 0.0);
}

fn gradient_position_at(index: i32) -> f32 {
    return grad_positions.values[index / 4][index % 4];
}

fn sample_gradient(t: f32, offset: i32, count: i32) -> vec4<f32> {
    if (count <= 1) {
        return grad_colors.values[offset];
    }

    let first_pos = gradient_position_at(offset);
    let last_pos = gradient_position_at(offset + count - 1);
    let tc = clamp(t, first_pos, last_pos);

    for (var i = 0; i < count - 1; i = i + 1) {
        let p0 = gradient_position_at(offset + i);
        let p1 = gradient_position_at(offset + i + 1);
        if (tc >= p0 && tc <= p1) {
            let span = max(p1 - p0, 0.0001);
            let local_t = (tc - p0) / span;
            return mix(grad_colors.values[offset + i], grad_colors.values[offset + i + 1], local_t);
        }
    }

    return grad_colors.values[offset + count - 1];
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = corner_radius(in.local_pos, in.radius);
    let d = sd_round_rect(in.local_pos, in.half_size, r);
    let aa = max(fwidth(d) * 0.5, 0.0001);

    let outer_alpha = 1.0 - smoothstep(-aa, aa, d);
    if (outer_alpha <= 0.0) {
        discard;
    }

    var fill = in.fill_color;
    let kind = in.gradient_meta.x;

    if (kind > 0.5) {
        let count = i32(in.gradient_meta.z);
        let offset = i32(in.gradient_meta.w);
        var t: f32;
        if (kind < 1.5) {
            let dir = vec2<f32>(cos(in.gradient_meta.y), sin(in.gradient_meta.y));
            let extent = abs(dir.x) * in.half_size.x + abs(dir.y) * in.half_size.y;
            t = (dot(in.local_pos, dir) / max(extent * 2.0, 0.0001)) + 0.5;
        } else {
            let max_dist = length(in.half_size);
            t = length(in.local_pos) / max(max_dist, 0.0001);
        }
        fill = sample_gradient(t, offset, count);
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
