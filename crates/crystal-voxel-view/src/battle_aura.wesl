import bevy_pbr::render::{forward_io::VertexOutput};

struct AuraMaterial {
    parameters: vec4<f32>,
    cool: vec4<f32>,
    hot: vec4<f32>,
}
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> aura: AuraMaterial;

fn hash(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(0.1031, 0.11369));
    return fract((q.x + q.y) * (q.x * 19.19 + q.y * 31.71));
}
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (vec2<f32>(3.0) - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0)), u.x), u.y);
}
fn noise3(p: vec3<f32>) -> f32 {
    let z = floor(p.z);
    let f = fract(p.z);
    return mix(noise(p.xy + vec2<f32>(z * 37.0, z * 17.0)),
        noise(p.xy + vec2<f32>((z + 1.0) * 37.0, (z + 1.0) * 17.0)), f*f*(3.0-2.0*f));
}
fn turbulence(p: vec2<f32>) -> f32 {
    return noise(p) * 0.57 + noise(p * 2.03 + vec2<f32>(7.1)) * 0.29
        + noise(p * 4.07 + vec2<f32>(13.7)) * 0.14;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time = aura.parameters.x;
    let phase = in.color.r;
    let uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    let seed = phase * 29.0;
    if aura.parameters.y > 2.5 {
        // A finite bank of curling flame tongues, eroded by advected 3D noise.
        // Integrating dense tongues (rather than a Gaussian glow) keeps the
        // silhouette crisp while their edges split, fold and reconnect.
        let x = (uv.x - 0.5) * 2.0;
        let y = uv.y;
        var light = vec3<f32>(0.0);
        var opacity = 0.0;
        for (var i = 0; i < 10; i += 1) {
            let z = (f32(i) + 0.5) * 0.16 - 0.8;
            let flow = vec3<f32>(x * 4.0 + seed, y * 6.0 - time * 3.2, z * 4.0);
            let curl = vec2<f32>(noise3(flow + vec3<f32>(7.1)),
                noise3(flow + vec3<f32>(19.7))) - vec2<f32>(0.5);
            let p = vec2<f32>(x, z) + curl * (0.12 + y * 0.42);
            let field = noise3(flow) * 0.65
                + noise3(flow * 2.07 + vec3<f32>(13.3)) * 0.35;
            var distance = -1.0;
            for (var tongue = 0; tongue < 5; tongue += 1) {
                let k = f32(tongue);
                let height = 0.76 + 0.15 * sin(k * 7.3 + time * 1.4 + seed);
                let level = clamp(y / height, 0.0, 1.0);
                let taper = pow(1.0 - level, 0.62);
                let center = (k - 2.0) * 0.27 * (0.72 + level * 0.4)
                    + sin(level * 7.0 - time * (2.2 + k * 0.15) + k * 3.1 + seed)
                        * level * 0.12;
                let radius = (0.23 + 0.035 * sin(level * 19.0 - time * 5.0 + k)) * taper;
                distance = max(distance, radius - length(p - vec2<f32>(center,0.0)));
            }
            let torn_edge = distance + (field - 0.5) * (0.05 + y * 0.20);
            let density = smoothstep(-0.016,0.055,torn_edge)
                * (1.0 - smoothstep(0.88,1.0,y))
                * smoothstep(0.0,0.18,y + (field-0.5)*0.10 - abs(x)*0.035)
                * (1.0 - smoothstep(0.92,1.0,abs(x)));
            let alpha = 1.0 - exp(-density * 0.9);
            let heat = clamp(distance * 3.2 + (1.0-y) * 0.75
                + (field-0.5) * 0.25,0.0,1.0);
            let color = mix(aura.cool.rgb,aura.hot.rgb,smoothstep(0.12,0.9,heat));
            light += (1.0-opacity) * alpha * color;
            opacity += (1.0-opacity) * alpha;
        }
        if opacity < 0.01 { discard; }
        return vec4<f32>(light / max(opacity,0.001),opacity);
    }
    let age = fract(time * in.color.g + phase);
    let base = in.color.b;
    let life = mix(smoothstep(0.0, 0.12, age) * (1.0 - smoothstep(0.62, 1.0, age)), 1.0, base);
    let field = turbulence(uv * vec2<f32>(4.0, 5.0) + vec2<f32>(seed, -time * 2.7));
    if aura.parameters.y > 1.5 {
        let p = uv * 2.0 - vec2<f32>(1.0);
        let radius = length(p + vec2<f32>(field - 0.5, noise(p * 3.0 + vec2<f32>(seed, -time * 0.4)) - 0.5) * 0.25);
        let density = (1.0 - smoothstep(0.3, 0.96, radius)) * smoothstep(0.25, 0.7, field);
        let alpha = density * life * 0.24;
        if alpha < 0.003 { discard; }
        return vec4<f32>(mix(aura.cool.rgb, aura.hot.rgb, field), alpha);
    }
    if aura.parameters.y > 0.5 {
        let radius = length((uv - vec2<f32>(0.5)) * vec2<f32>(2.0, 1.6));
        let alpha = (1.0 - smoothstep(0.0, 0.75, radius)) * life * 0.9;
        if alpha < 0.003 { discard; }
        return vec4<f32>(mix(aura.hot.rgb, aura.cool.rgb, age), alpha);
    }
    // Short-lived curled flame wisps, with a soft density falloff rather than
    // a geometric triangle taper. The persistent base lives in the volume above.
    let bend = sin(uv.y*8.0-time*2.1+seed)*uv.y*0.25 + (field-0.5)*0.4;
    let p = vec2<f32>((uv.x-0.5+bend)*2.0,(uv.y-0.35)*1.8);
    let density = exp(-dot(p,p)*3.5) * smoothstep(0.2,0.72,field)
        * smoothstep(0.0,0.08,uv.y) * (1.0-smoothstep(0.75,1.0,uv.y));
    let alpha = density * life * 0.7;
    if alpha < 0.003 { discard; }
    return vec4<f32>(mix(aura.cool.rgb,aura.hot.rgb,(1.0-uv.y)*0.7),alpha);
}
