import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
struct TiltShift {
    band: vec4<f32>, entry: vec4<f32>, entry_rows: array<vec4<f32>, 36>,
    black_cells: array<vec4<u32>, 3>, trainer_cells: array<vec4<u32>, 3>,
    trainer_pixels: vec4<u32>, trainer_palette: array<vec4<f32>, 4>,
}
@group(0) @binding(2) var<uniform> settings: TiltShift;

fn blur(in: FullscreenVertexOutput, direction: vec2<f32>) -> vec4<f32> {
    let band = settings.band;
    let edge_distance = select(band.x, 1.0 - band.x, in.uv.y >= band.x);
    let transition = max(edge_distance - band.y, 0.0001);
    let t = clamp((abs(in.uv.y - band.x) - band.y) / transition, 0.0, 1.0);
    // Keep the center detailed and retain a gradient all the way to each
    // edge, even when the trainer is above or below the screen midpoint.
    let amount = pow(t, band.z);
    let diameter = band.w * amount;
    let center = textureSampleLevel(scene, scene_sampler, in.uv, 0.0);
    if diameter < 0.5 { return center; }

    // Separable Gaussian, with adjacent taps paired using linear filtering.
    // The early return also avoids underflow/NaNs in nearly sharp regions.
    let sigma = diameter * 0.25;
    let support = i32(ceil(sigma * 3.0));
    let step = direction / vec2<f32>(textureDimensions(scene));
    let exponent = -0.5 / (sigma * sigma);
    // Fade the Gaussian tail to zero at three sigma. New taps then enter
    // continuously instead of making steps in the spatial blur gradient.
    let tail = exp(-4.5);
    var sum = center.rgb * (1.0 - tail);
    var weight_sum = 1.0 - tail;
@if(!DIRECT_GAUSSIAN) {
    // Gaussian(n+1) / Gaussian(n) advances by a constant ratio. This keeps
    // the same paired kernel while replacing per-tap exponentials with
    // multiplications (two exponentials per blurred pixel in total).
    var gaussian = 1.0;
    var ratio = exp(exponent);
    let ratio_step = exp(2.0 * exponent);
}
    for (var i = 1; i <= support; i += 2) {
        var w0: f32;
        var w1: f32;
@if(DIRECT_GAUSSIAN) {
        w0 = max(exp(exponent * f32(i * i)) - tail, 0.0);
        w1 = max(exp(exponent * f32((i + 1) * (i + 1))) - tail, 0.0);
}
@if(!DIRECT_GAUSSIAN) {
        gaussian *= ratio;
        ratio *= ratio_step;
        w0 = max(gaussian - tail, 0.0);
        gaussian *= ratio;
        ratio *= ratio_step;
        w1 = max(gaussian - tail, 0.0);
}
        let weight = w0 + w1;
        if weight > 0.000001 {
            let offset = step * (f32(i) + w1 / weight);
            sum += (textureSampleLevel(scene, scene_sampler, in.uv + offset, 0.0).rgb
                + textureSampleLevel(scene, scene_sampler, in.uv - offset, 0.0).rgb) * weight;
            weight_sum += weight * 2.0;
        }
    }
    return vec4<f32>(sum / weight_sum, center.a);
}

@fragment
fn horizontal(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return blur(in, vec2<f32>(1.0, 0.0));
}

@fragment
fn vertical(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    if settings.entry.z == 0.0 { return blur(in, vec2<f32>(0.0, 1.0)); }
    var shifted = in;
    let row = min(u32(in.uv.y * 144.0), 143u);
    shifted.uv.x = fract(in.uv.x - settings.entry_rows[row / 4u][row % 4u] / 160.0);
    let cell_xy = min(vec2<u32>(shifted.uv * vec2<f32>(20.0, 18.0) + vec2<f32>(0.00001)), vec2<u32>(19u, 17u));
    let cell = cell_xy.y * 20u + cell_xy.x;
    let word = cell / 32u;
    let bit = 1u << (cell % 32u);
    if (settings.black_cells[word / 4u][word % 4u] & bit) != 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    // Trainer artwork keeps native square pixels, centered in the full view.
    let size = vec2<f32>(textureDimensions(scene));
    // Snap arithmetic noise at exact native-pixel boundaries consistently.
    let trainer_pixel = (shifted.uv - vec2<f32>(0.5)) * vec2<f32>(size.x / size.y * 144.0, 144.0)
        + vec2<f32>(80.0001, 72.0001);
    let trainer_xy = min(vec2<u32>(max(trainer_pixel, vec2<f32>(0.0)) / 8.0), vec2<u32>(19u, 17u));
    let trainer_cell = trainer_xy.y * 20u + trainer_xy.x;
    let trainer_word = trainer_cell / 32u;
    if all(trainer_pixel >= vec2<f32>(0.0)) && all(trainer_pixel < vec2<f32>(160.0, 144.0))
        && (settings.trainer_cells[trainer_word / 4u][trainer_word % 4u] & (1u << (trainer_cell % 32u))) != 0u {
        let pixel_xy = min(vec2<u32>(trainer_pixel), vec2<u32>(159u, 143u)) % vec2<u32>(8u);
        let pixel = pixel_xy.y * 8u + pixel_xy.x;
        let index = (settings.trainer_pixels[pixel / 16u] >> ((pixel % 16u) * 2u)) & 3u;
        return settings.trainer_palette[index];
    }
    let color = blur(shifted, vec2<f32>(0.0, 1.0));
    return vec4<f32>(mix(color.rgb * (1.0 - settings.entry.x), vec3<f32>(1.0), settings.entry.y), color.a);
}
