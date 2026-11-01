import bevy_pbr::render::{forward_io::VertexOutput};

struct SkyMaterial {
    center: vec4<f32>,
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    cloud: vec4<f32>,
    sun: vec4<f32>,
    parameters: vec4<f32>,
}
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> sky: SkyMaterial;

fn hash(p: vec3<f32>) -> f32 {
    let q = fract(p * vec3<f32>(0.1031, 0.11369, 0.13787));
    let r = q + vec3<f32>(dot(q, q.yzx + vec3<f32>(19.19)));
    return fract((r.x + r.y) * r.z);
}
fn noise(p: vec3<f32>) -> f32 {
    let cell = floor(p);
    let f = fract(p);
    let u = f * f * (vec3<f32>(3.0) - 2.0 * f);
    return mix(mix(mix(hash(cell), hash(cell + vec3<f32>(1.0,0.0,0.0)),u.x),
        mix(hash(cell+vec3<f32>(0.0,1.0,0.0)),hash(cell+vec3<f32>(1.0,1.0,0.0)),u.x),u.y),
        mix(mix(hash(cell+vec3<f32>(0.0,0.0,1.0)),hash(cell+vec3<f32>(1.0,0.0,1.0)),u.x),
        mix(hash(cell+vec3<f32>(0.0,1.0,1.0)),hash(cell+vec3<f32>(1.0,1.0,1.0)),u.x),u.y),u.z);
}
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray = normalize(in.world_position.xyz - sky.center.xyz);
    let summit = sky.parameters.w;
    // At the summit the camera looks down into the lower atmosphere. Continue
    // the gradient and cloud layers there instead of a single clear-color fill.
    let elevation = max(ray.y + summit * 0.24, 0.0);
    let gradient = pow(smoothstep(0.0, 0.85, elevation), 0.55);
    var color = mix(sky.horizon.rgb, sky.zenith.rgb, gradient);
    // Broad haze near the horizon, fading into a deep, clean zenith.
    let alignment = max(dot(ray, sky.sun.xyz), 0.0);
    let glow = pow(alignment, 24.0) * sky.parameters.z;
    color += vec3<f32>(0.42, 0.23, 0.08) * glow * (1.0 - gradient * 0.65);
    // Direction-space clouds are seamless around the dome. Slow wind changes
    // presentation only; three octaves keep the WebGL shader inexpensive.
    var clouds = 0.0;
    // Below this elevation the authored cloud mask is exactly zero. Avoid
    // three octaves (24 corner hashes) where none can contribute.
    if elevation > 0.015 {
        let p = ray * vec3<f32>(5.0, 9.0, 5.0) + vec3<f32>(sky.parameters.x, 0.0, sky.parameters.x * 0.4);
        let field = noise(p)*0.57 + noise(p*2.03)*0.29 + noise(p*4.07)*0.14;
        clouds = smoothstep(0.51, 0.72, field) * smoothstep(0.015, 0.16, elevation);
        let cloud_shading = 0.78 + 0.22 * smoothstep(0.45, 0.8, field);
        color = mix(color, sky.cloud.rgb * cloud_shading, clouds * 0.82);
    }
    if summit > 0.5 {
        let wisps = noise(ray * vec3<f32>(9.0, 28.0, 9.0)
            + vec3<f32>(sky.parameters.x * 0.5, 7.0, 0.0));
        let veil = smoothstep(0.49, 0.73, wisps)
            * (1.0 - smoothstep(-0.45, 0.12, ray.y));
        color = mix(color, sky.cloud.rgb * 0.86, veil * 0.35);
    }
    // Sparse fixed stars, with brighter stars slightly larger. Their world
    // directions stay stable under camera movement, yaw, and midnight wrap.
    // Daylight and the lower horizon have exactly zero star contribution.
    if sky.parameters.y > 0.0 && elevation > 0.01 && clouds < 1.0 {
    let star_grid = ray * 380.0;
    let star_cell = floor(star_grid);
    let seed = hash(star_cell);
    let offset = vec3<f32>(hash(star_cell+vec3<f32>(11.0)),hash(star_cell+vec3<f32>(29.0)),hash(star_cell+vec3<f32>(47.0)));
    let distance = length(fract(star_grid)-offset);
    let star = (1.0-smoothstep(0.08,0.30,distance)) * step(0.982,seed);
    color += mix(vec3<f32>(0.58,0.72,1.0),vec3<f32>(1.0,0.91,0.72),hash(star_cell+vec3<f32>(71.0)))
        * star * sky.parameters.y * smoothstep(0.01,0.18,elevation) * (1.0-clouds);
    }
    // The same direction as the scene light grounds the sun/moon in the sky.
    let disc = smoothstep(0.99970,0.99986,alignment) * (1.0-clouds);
    let moon_cut = smoothstep(0.99970,0.99986,
        dot(ray,normalize(sky.sun.xyz + vec3<f32>(0.013,0.0,0.009))));
    let visible_disc = disc * mix(1.0-moon_cut,1.0,sky.parameters.z);
    let disc_color = mix(vec3<f32>(0.65,0.76,1.0),vec3<f32>(1.0,0.90,0.65),sky.parameters.z);
    color += disc_color * visible_disc * mix(0.65,1.8,sky.parameters.z);
    return vec4<f32>(color,1.0);
}
