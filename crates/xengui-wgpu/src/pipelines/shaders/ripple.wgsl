// Procedural port of Android's patterned RippleShader structure: a soft
// expanding wave, turbulent ring and high-frequency white sparkles.
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: vec4<f32>,
    @location(3) origin_radius: vec4<f32>,
    @location(4) progress_noise: vec2<f32>,
    @location(5) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) screen_rect: vec4<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) radius: vec4<f32>,
    @location(3) origin_radius: vec4<f32>,
    @location(4) progress_noise: vec2<f32>,
    @location(5) color: vec4<f32>,
) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let corner = corners[vertex_index];
    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        mix(screen_rect.x, screen_rect.z, corner.x),
        mix(screen_rect.y, screen_rect.w, corner.y), 0.0, 1.0);
    out.local_pos = (corner - vec2<f32>(0.5)) * half_size * 2.0;
    out.half_size = half_size;
    out.radius = radius;
    out.origin_radius = origin_radius;
    out.progress_noise = progress_noise;
    out.color = color;
    return out;
}

fn saturate(v: f32) -> f32 { return clamp(v, 0.0, 1.0); }
fn sub_progress(start: f32, end: f32, progress: f32) -> f32 {
    return saturate((progress - start) / (end - start));
}
fn triangle_noise(n: f32) -> f32 {
    return abs(fract(n) - 0.5) * 2.0;
}
fn sparkle_noise(uv: vec2<f32>, phase: f32) -> f32 {
    let p = uv * 2.1;
    let n0 = triangle_noise(p.x * 0.73 + triangle_noise(p.y * 1.37 + phase * 91.0));
    let n1 = triangle_noise(p.y * 1.11 + triangle_noise(p.x * 1.91 - phase * 67.0));
    let star = pow(saturate(1.0 - abs(n0 + n1 - 1.0) * 2.0), 18.0);
    let glint = pow(triangle_noise(n0 * 7.0 + n1 * 13.0 + phase * 43.0), 28.0);
    return saturate(star * 0.8 + glint);
}
fn corner_radius(p: vec2<f32>, radii: vec4<f32>) -> f32 {
    if (p.x < 0.0) {
        if (p.y < 0.0) { return radii.x; }
        return radii.w;
    }
    if (p.y < 0.0) { return radii.y; }
    return radii.z;
}
fn rounded_rect_distance(p: vec2<f32>, half_size: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - half_size + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) - r + min(max(q.x, q.y), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let mask_d = rounded_rect_distance(in.local_pos, in.half_size,
        corner_radius(in.local_pos, in.radius));
    let mask = 1.0 - smoothstep(-fwidth(mask_d), fwidth(mask_d), mask_d);
    if (mask <= 0.0) { discard; }

    let progress = in.progress_noise.x;
    let scale_in = sub_progress(0.0, 1.0, progress);
    let fade_in = sub_progress(0.0, 0.13, progress);
    // XenGui keeps the touch itself as the visual origin and expands far
    // enough to cover the most distant corner of the bounded view.
    let center = in.origin_radius.xy;
    let point = in.local_pos + in.half_size;
    let distance = length(point - center);
    let wave_radius = max(in.origin_radius.z * scale_in, 0.001);
    // A broad analytic feather avoids the hard legacy-ripple edge without
    // requiring a costly offscreen blur pass on mobile GPUs.
    let softness = clamp(in.origin_radius.z * 0.04, 6.0, 18.0);
    let wave = 1.0 - smoothstep(wave_radius - softness, wave_radius + softness, distance);

    let ring_width = max(7.0, wave_radius * 0.09);
    let ring = 1.0 - smoothstep(ring_width * 0.2, ring_width,
        abs(distance - wave_radius));
    let turbulence = 0.65 + 0.35 * triangle_noise(
        atan2(point.y - center.y, point.x - center.x) * 3.8 + in.progress_noise.y * 31.0);
    let sparkle = sparkle_noise(point, in.progress_noise.y) * ring * turbulence * fade_in;
    // Keep a lower-energy turbulent grain inside the wave as it expands;
    // the former ring-only noise could look absent on large containers.
    let grain = sparkle_noise(point * 1.37, in.progress_noise.y + 0.371)
        * wave * fade_in;

    let lifecycle = in.origin_radius.w;
    let wave_alpha = wave * fade_in * lifecycle * in.color.a;
    let sparkle_alpha = (sparkle + grain * 0.32) * lifecycle
        * min(in.color.a * 5.2, 0.68);
    let alpha = saturate((wave_alpha + sparkle_alpha) * mask);
    let rgb = mix(in.color.rgb, vec3<f32>(1.0),
        saturate(sparkle_alpha / max(alpha, 0.0001)));
    return vec4<f32>(rgb, alpha);
}
