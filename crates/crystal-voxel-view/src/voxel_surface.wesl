import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, main_pass_post_lighting_processing},
    mesh_view_bindings,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    shadows::fetch_directional_shadow,
};

// A cached material controls the whole authored wall/roof group. The ordinary
// shadow prepass never reads this forward-only fade, so lighting stays stable.
struct CutawayUniform {
    bottom_radius: vec4<f32>,
    top_feather: vec4<f32>,
    fade: vec4<f32>,
}
@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> cutaway: CutawayUniform;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var water_mask: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var water_mask_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var<uniform> water_enabled: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var<uniform> cave_reveal: vec4<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(105) var painted_stone: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(106) var painted_stone_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(107) var painted_bedrock: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(108) var painted_bedrock_sampler: sampler;

fn water_impact(uv: vec2<f32>) -> f32 {
    let size = vec2<i32>(textureDimensions(water_mask));
    let p = uv * vec2<f32>(size) - vec2<f32>(0.5);
    let cell = vec2<i32>(floor(p));
    let f = fract(p);
    let hi = size - vec2<i32>(1);
    let a = textureLoad(water_mask, clamp(cell, vec2<i32>(0), hi), 0).a;
    let b = textureLoad(water_mask, clamp(cell + vec2<i32>(1,0), vec2<i32>(0), hi), 0).a;
    let c = textureLoad(water_mask, clamp(cell + vec2<i32>(0,1), vec2<i32>(0), hi), 0).a;
    let d = textureLoad(water_mask, clamp(cell + vec2<i32>(1,1), vec2<i32>(0), hi), 0).a;
    return 1.0 - mix(mix(a,b,f.x), mix(c,d,f.x), f.y);
}

// Forward-only aperture; floors, fragments behind the body and shadow
// geometry remain intact. The capsule follows authoritative player support.
fn reveal_cave_player(world: vec4<f32>, pixel: vec2<f32>) {
    if cave_reveal.x < 0.5 || cutaway.bottom_radius.w <= 0.0 { return; }
    // A faded inner wall must fade down to its base. Keeping its bottom at a
    // player-relative height left opaque stumps that looked like false steps.
    // Authored low surfaces and the outer rock enclosure keep their relief.
    if cave_reveal.w > 0.5
        && world.y <= cutaway.bottom_radius.y + cutaway.bottom_radius.w * 0.40 { return; }
    let bottom = mesh_view_bindings::view.view_from_world * vec4<f32>(cutaway.bottom_radius.xyz, 1.0);
    let top = mesh_view_bindings::view.view_from_world * vec4<f32>(cutaway.top_feather.xyz, 1.0);
    let fragment = mesh_view_bindings::view.view_from_world * world;
    let bottom_clip = mesh_view_bindings::view.clip_from_world * vec4<f32>(cutaway.bottom_radius.xyz, 1.0);
    let top_clip = mesh_view_bindings::view.clip_from_world * vec4<f32>(cutaway.top_feather.xyz, 1.0);
    let fragment_clip = mesh_view_bindings::view.clip_from_world * world;
    if min(min(bottom_clip.w, top_clip.w), fragment_clip.w) <= 0.0 { return; }
    // Project into foot-plane units to preserve aspect ratio under either
    // orthographic or perspective cameras, including user-controlled orbit.
    let scale = vec2<f32>(mesh_view_bindings::view.clip_from_view[0][0], mesh_view_bindings::view.clip_from_view[1][1]);
    let b = bottom_clip.xy / scale;
    let t = top_clip.xy / top_clip.w * bottom_clip.w / scale;
    let p = fragment_clip.xy / fragment_clip.w * bottom_clip.w / scale;
    let axis = t - b;
    let along = clamp(dot(p - b, axis) / max(dot(axis, axis), 0.0001), 0.0, 1.0);
    let body_along = (along / top_clip.w)
        / ((1.0 - along) / bottom_clip.w + along / top_clip.w);
    // Tall foreground rock needs to expose the player's floor as well as
    // their body. A body-width pinhole makes feet appear planted in the roof.
    // Keep a clear body-sized center, then dissolve foreground rock over a
    // broad shoulder. Screen-door coverage preserves opaque depth ordering
    // and the unchanged shadow mesh while reading as gradual transparency.
    let radius = cutaway.bottom_radius.w * 4.5;
    let feather = max(cutaway.top_feather.w, radius * 0.70);
    let distance = length(p - mix(b, t, along));
    let shoulder = 1.0 - smoothstep(radius - feather, radius, distance);
    let depth = fragment.z - mix(bottom.z, top.z, body_along);
    let aperture = shoulder * smoothstep(0.15, max(radius * 0.35, 0.16), depth);
    // Fade the full foreground wall, not only a hole around the trainer.
    // Compare on the foot plane: height must not turn a distant rear wall
    // into a foreground blocker under the pitched camera. Both stone and
    // ice use this same rule, and it follows the user's camera orbit.
    let foot_plane = mesh_view_bindings::view.view_from_world
        * vec4<f32>(world.x, cutaway.bottom_radius.y, world.z, 1.0);
    let front_depth = foot_plane.z - bottom.z;
    let wall_fade = 0.88 * smoothstep(cutaway.bottom_radius.w * 0.30,
        cutaway.bottom_radius.w * 1.20, front_depth);
    let strength = max(aperture, select(wall_fade, 0.0, cave_reveal.w > 0.5));
    // Continuous, screen-stable coverage avoids the sixteen visible opacity
    // bands of a small Bayer matrix. No frame seed: the shoulder does not flicker.
    let coverage = fract(52.9829189 * fract(dot(floor(pixel), vec2<f32>(0.06711056, 0.00583715))));
    if strength > coverage { discard; }
}

// The palette and baked face shade are the surface color. The sun pass
// contributes visibility only, without applying another PBR light response.
fn mineral_hash(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}

fn mineral_field(p: vec3<f32>) -> f32 {
    let cell = floor(p);
    let t = fract(p);
    let f = t * t * (vec3<f32>(3.0) - 2.0 * t);
    let a = mix(mineral_hash(cell), mineral_hash(cell + vec3<f32>(1.0, 0.0, 0.0)), f.x);
    let b = mix(mineral_hash(cell + vec3<f32>(0.0, 1.0, 0.0)), mineral_hash(cell + vec3<f32>(1.0, 1.0, 0.0)), f.x);
    let c = mix(mineral_hash(cell + vec3<f32>(0.0, 0.0, 1.0)), mineral_hash(cell + vec3<f32>(1.0, 0.0, 1.0)), f.x);
    let d = mix(mineral_hash(cell + vec3<f32>(0.0, 1.0, 1.0)), mineral_hash(cell + vec3<f32>(1.0, 1.0, 1.0)), f.x);
    return mix(mix(a, b, f.y), mix(c, d, f.y), f.z);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var surface = pbr_input_from_standard_material(in, is_front);
    var enclosing_wall = false;
@if(VERTEX_UVS_A) {
    if in.uv.x > 90000.0 {
        let kind = round(in.uv.y / 100000.0);
        enclosing_wall = kind == 2.0 || kind == 3.0;
    }
}
    // Only enclosing rock participates. Walking planes, complete low stones,
    // authored stairs and live water retain their full geometry at all heights.
    if enclosing_wall { reveal_cave_player(surface.world_position, in.position.xy); }
    if cutaway.fade.w > 0.0 && surface.material.base_color.a < cutaway.fade.w { discard; }
    surface.material.base_color = alpha_discard(surface.material.flags, surface.material.alpha_cutoff, surface.material.base_color);
    // The mask is derived from proven source animation slots in the owned
    // atlas. Its authenticated phase drives continuous map-space ripples;
    // waterfall detail follows those same live frames.
    // Untagged paint, geometry, alpha and encounter cues remain unchanged.
@if(VERTEX_UVS_A) {
    // Reserved source-guarded indoor finishes, separate from atlas UVs,
    // live water, fountain threads and the rock material coordinate ranges.
    let room_paint = in.uv.x >= 80000.0 && in.uv.x < 81000.0
        && in.uv.y >= 1000.0 && in.uv.y < 3000.0;
    let room_prop = in.uv.x >= 82000.0 && in.uv.x < 82100.0
        && in.uv.y >= 1000.0 && in.uv.y < 3000.0;
    if room_prop {
        let fabric = in.uv.y >= 2500.0;
        let enamel = in.uv.y >= 2000.0 && !fabric;
        let origin = select(select(1000.0,2000.0,enamel),2500.0,fabric);
        let map = in.uv - vec2<f32>(82000.0, origin);
        let height = surface.world_position.y;
        let cloud = mineral_field(vec3<f32>(map.x * 0.24,height * 0.15,map.y * 0.24));
        var pigment = 0.97 + cloud * 0.055;
        if fabric {
            let phase = map * 4.0;
            let resolved = vec2<f32>(1.0) - smoothstep(vec2<f32>(0.5),vec2<f32>(2.5),fwidth(phase));
            let warp = pow(0.5 + 0.5*sin(phase.x),6.0)*resolved.x;
            let weft = pow(0.5 + 0.5*sin(phase.y),6.0)*resolved.y;
            pigment = 1.02 - (warp + weft) * 0.085;
        } else if !enamel {
            let phase = (map.x + map.y) * 1.6 + cloud * 1.4;
            let resolved = 1.0 - smoothstep(0.4,2.5,fwidth(phase));
            pigment = 1.03 - pow(0.5 + 0.5*sin(phase),8.0)*0.20*resolved;
        }
        surface.material.base_color = vec4<f32>(surface.material.base_color.rgb*pigment,
            surface.material.base_color.a);
    }
    if room_paint {
        let timber = in.uv.y >= 2000.0;
        let map = in.uv - vec2<f32>(80000.0, select(1000.0, 2000.0, timber));
        var pigment = 1.0;
        if timber {
            // Long, gently wandering grain runs along the authored boards.
            // Derivative filtering fades fine lines before they can shimmer.
            let bend = mineral_field(vec3<f32>(map.x * 0.045, map.y * 0.08, 2.7));
            let phase = map.y * 1.4 + bend * 1.2;
            let resolved = 1.0 - smoothstep(0.4, 2.5, fwidth(phase));
            let grain = pow(0.5 + 0.5 * sin(phase), 8.0) * resolved;
            pigment = 1.03 - grain * 0.22
                + (mineral_field(vec3<f32>(map * 0.32, 4.1)) - 0.5) * 0.09;
        } else {
            // Glazed honey-colored ceramic: soft pigment pooling and small
            // kiln speckles, without dirt, moving noise or a plastic flat fill.
            let glaze = mineral_field(vec3<f32>(map * 0.22, 9.2));
            let speckle = mineral_field(vec3<f32>(map * 1.7, 5.6));
            let firing = mineral_hash(vec3<f32>(floor(map / 8.0), 17.0));
            pigment = 0.84 + glaze * 0.22 + (speckle - 0.5) * 0.08
                + (firing - 0.5) * 0.055;
            // Amber pigment collects unevenly in the fired glaze. Color
            // variation stays registered to the room rather than the camera.
            let pooling = (1.0 - glaze) * 0.10;
            surface.material.base_color = vec4<f32>(
                surface.material.base_color.rgb * vec3<f32>(1.0, 1.0 - pooling, 1.0 - pooling * 1.5),
                surface.material.base_color.a);
        }
        surface.material.base_color = vec4<f32>(surface.material.base_color.rgb * pigment,
            surface.material.base_color.a);
    }
    // Verified park water carries source coordinates, not atlas UVs. Moving
    // highlights remain continuous across source cells and camera scrolling.
    if in.uv.x > 40000.0 && in.uv.x < 60000.0 && in.uv.y < -40000.0 {
        let map = in.uv - vec2<f32>(50000.0, -50000.0);
        let time = mesh_view_bindings::globals.time;
        let drift = mineral_field(vec3<f32>(map * 0.37, time * 0.035));
        let cloud = mineral_field(vec3<f32>(map * 0.16, 7.4));
        let wave = sin(map.x * 2.1 + map.y * 3.7 - time * 0.85
            + sin(map.x * 0.43 - map.y * 0.37) * 0.6 + drift * 5.0);
        let cross = sin(map.x * 4.8 - map.y * 1.9 + time * 0.53 + cloud * 3.2);
        let glint = smoothstep(0.93, 0.999, wave)
            * smoothstep(0.15, 0.8, cross) * smoothstep(0.25, 0.65, drift) * 0.20;
        let depth = mix(vec3<f32>(0.035, 0.20, 0.24), vec3<f32>(0.085, 0.32, 0.34),
            0.45 + wave * 0.12 + cross * 0.04);
        let reflected = mix(depth, vec3<f32>(0.045, 0.16, 0.15),
            smoothstep(0.52, 0.82, cloud) * 0.20);
        surface.material.base_color = vec4<f32>(reflected
            + vec3<f32>(0.45, 0.59, 0.57) * glint, surface.material.base_color.a);
    }
    // Glints flow along the curved thread instead of pulsing the whole bowl.
    if in.uv.x >= 70000.0 && in.uv.x < 70020.0 {
        let phase = (in.uv.x - 70000.0) * 0.73;
        let flow = sin(in.uv.y * 28.0 - mesh_view_bindings::globals.time * 7.0 + phase);
        let glint = smoothstep(0.5, 0.98, flow);
        surface.material.base_color = vec4<f32>(mix(surface.material.base_color.rgb * 0.78,
            vec3<f32>(0.78, 0.90, 0.88), glint * 0.65), surface.material.base_color.a);
    }
    // Only original outdoor stone emits these negative coordinate tags.
    // Decode authored map coordinates so scrolling cannot slide the grain.
    // Atlas UVs and all other solid surfaces remain outside this branch.
    if in.uv.x < -9000.0 {
        let canyon = in.uv.y < -30000.0;
        let tag = select(20000.0, 40000.0, canyon);
        let rock = vec3<f32>(-in.uv.x - 10000.0, surface.world_position.y, -in.uv.y - tag);
        let mineral = mineral_field(rock * 1.4);
        let weather = mineral_field(rock * 0.19);
        let layer = abs(sin(rock.y * select(1.1, 0.78, canyon) + weather * 1.2));
        let fissure = smoothstep(0.035, 0.14, layer);
        let wall = 1.0 - smoothstep(0.35, 0.8, abs(surface.world_normal.y));
        let detail = (0.94 + mineral * 0.12) * mix(1.0, mix(0.88, 1.0, fissure), wall);
        // Triplanar map-space projection keeps sediment horizontal on both
        // wall directions and avoids stretched cliff caps at camera rotations.
        let weights_raw = pow(abs(surface.world_normal), vec3<f32>(2.0));
        let weights = weights_raw / max(dot(weights_raw, vec3<f32>(1.0)), 0.0001);
        let scale = 1.0 / 128.0;
        let x_layer = textureSample(painted_stone, painted_stone_sampler, rock.zy * scale).rgb;
        let z_layer = textureSample(painted_stone, painted_stone_sampler, rock.xy * scale).rgb;
        let y_layer = textureSample(painted_stone, painted_stone_sampler, rock.xz * scale).rgb;
        let x_rock = textureSample(painted_bedrock, painted_bedrock_sampler, rock.zy / 40.0).rgb;
        let z_rock = textureSample(painted_bedrock, painted_bedrock_sampler, rock.xy / 40.0).rgb;
        let y_rock = textureSample(painted_bedrock, painted_bedrock_sampler, rock.xz / 40.0).rgb;
        let x_paint = select(x_rock, x_layer, canyon);
        let z_paint = select(z_rock, z_layer, canyon);
        // Carry some sediment pigment over the shoulder so its cap and shaft
        // meet gradually instead of switching between unrelated materials.
        let canyon_cap = mix(y_rock * vec3<f32>(1.38, 1.05, 0.70), y_layer, 0.30);
        let y_paint = select(y_rock, canyon_cap, canyon);
        let paint = x_paint * weights.x + y_paint * weights.y + z_paint * weights.z;
        let reference = select(vec3<f32>(0.20, 0.205, 0.17), vec3<f32>(0.35, 0.245, 0.12), canyon);
        // Canyon forms should read before their grain. Broad ochre variation
        // remains visible, with quieter caps and restrained sediment bands.
        let paint_strength = select(0.82, mix(0.42, 0.60, wall), canyon);
        let broad_pigment = reference * select(1.0, 0.96 + weather * 0.08, canyon);
        let pigment = mix(broad_pigment, paint, paint_strength);
        surface.material.base_color = vec4<f32>(pigment * surface.material.base_color.rgb / reference * detail,
            surface.material.base_color.a);
    }
    // Cave stone and dust use distinct positive map-coordinate tags.
    // Floors use broader, softer paint; source art and live water stay in the atlas.
    if in.uv.x > 90000.0 {
        let category = round(in.uv.y / 100000.0);
        let sliding_ice = category == 8.0;
        let shelf_shoulder = category == 9.0;
        let frozen = category == 3.0 || category == 5.0 || category == 7.0 || sliding_ice || shelf_shoulder;
        let floor_paint = category == 4.0 || category == 5.0 || sliding_ice;
        let tag = category * 100000.0;
        let rock = vec3<f32>(in.uv.x - 100000.0, surface.world_position.y, in.uv.y - tag);
        let w = pow(abs(surface.world_normal), vec3<f32>(4.0));
        let weights = w / max(dot(w, vec3<f32>(1.0)), 0.0001);
        // Larger mineral plates support regular cave rock. Frozen surfaces
        // retain their established plate scale and pale crest detail.
        let size = select(select(select(144.0, 96.0, frozen), 160.0, floor_paint), 112.0, sliding_ice);
        let a = textureSample(painted_bedrock, painted_bedrock_sampler, rock.zy / size).rgb;
        let b = textureSample(painted_bedrock, painted_bedrock_sampler, rock.xy / size).rgb;
        let c = textureSample(painted_bedrock, painted_bedrock_sampler, rock.xz / size).rgb;
        let paint = a * weights.x + b * weights.z + c * weights.y;
        let ratio = paint / vec3<f32>(0.25, 0.21, 0.16);
        let grey = dot(ratio, vec3<f32>(0.2126, 0.7152, 0.0722));
        let mineral = select(ratio, vec3<f32>(grey), frozen);
        // Retain the existing floor and frozen-rock treatment. Regular rock
        // uses quieter, bounded grain so its fractures and map-space pigment
        // carry the form without bright flecks over every geometric facet.
        let rock_pigment = select(clamp(mineral, vec3<f32>(0.55), vec3<f32>(1.65)), mineral, frozen || floor_paint);
        var paint_amount = select(select(0.42, 0.38, frozen), select(0.65, 0.38, frozen) * 0.32, floor_paint);
        var blended_pigment = rock_pigment;
        if shelf_shoulder {
            // Keep the face paint continuous at the foot of the shoulder,
            // then feather broader cap plates across the rounded crest.
            let cap = smoothstep(0.10, 0.95, abs(surface.world_normal.y));
            let cap_sample = textureSample(painted_bedrock, painted_bedrock_sampler, rock.xz / 160.0).rgb;
            let cap_grey = dot(cap_sample / vec3<f32>(0.25, 0.21, 0.16), vec3<f32>(0.2126, 0.7152, 0.0722));
            blended_pigment = mix(rock_pigment, vec3<f32>(cap_grey), cap);
            paint_amount = mix(0.38, 0.38 * 0.32, cap);
        }
        surface.material.base_color = vec4<f32>(surface.material.base_color.rgb
            * mix(vec3<f32>(1.0), blended_pigment, paint_amount), surface.material.base_color.a);
        if sliding_ice {
            // Fine fractured plates sit beneath irregular frost clouds. The
            // field is world registered and has no time term: ice stays still
            // while authored water continues through its separate live mask.
            let plate = textureSample(painted_bedrock, painted_bedrock_sampler,
                rock.xz / 144.0 + vec2<f32>(0.37, 0.61)).rgb;
            let plate_luma = dot(plate / vec3<f32>(0.25, 0.21, 0.16), vec3<f32>(0.2126, 0.7152, 0.0722));
            let cloud = mineral_field(vec3<f32>(rock.x * 0.015, 7.3, rock.z * 0.015));
            let frost = smoothstep(0.54, 0.82, cloud) * 0.24;
            let pigment = surface.material.base_color.rgb
                * mix(1.0, clamp(plate_luma, 0.45, 1.65), 0.16);
            surface.material.base_color = vec4<f32>(mix(pigment,
                vec3<f32>(0.60, 0.82, 0.81), frost), surface.material.base_color.a);
        }
    }
    if water_enabled.x > 0.5 && !room_paint && !room_prop {
        let water = textureSample(water_mask, water_mask_sampler, in.uv);
        let kind = water.rg;
        if max(kind.x, kind.y) > 0.5 {
            let phase = water.b * 6.2831853;
            let map = (surface.world_position.xz + water_enabled.zw) / max(water_enabled.y, 0.001);
            let broad = sin(map.y * 2.7 + map.x * 0.22 + sin(map.x * 0.31 + map.y * 0.12) * 1.9 + sin(map.x * 1.1 + map.y * 0.22) * 0.5 - phase);
            let cross = sin(map.y * 5.6 + map.x * 0.33 + sin(map.x * 0.71) * 0.4 + phase * 2.0);
            let ripple = broad * 0.72 + cross * 0.28;
            let broken = smoothstep(-0.1, 0.7, sin(map.x * 0.67 + sin(map.y * 0.5) * 1.7) * 0.7 + sin(map.x * 0.29 - map.y * 0.77) * 0.3);
            let glint = smoothstep(0.95, 0.999, broad) * broken;
            let deep = vec3<f32>(0.018, 0.095, 0.125);
            let crest = vec3<f32>(0.16, 0.38, 0.40);
            let pool = mix(deep, vec3<f32>(0.055, 0.19, 0.22), ripple * 0.25 + 0.40);
            let rippled = mix(pool, crest, glint * 0.65);
            // Continuous ribbons follow the authenticated cascade phase,
            // rather than magnifying the atlas's eight-pixel foam checker.
            // Detail is continuous in map space, including the landing;
            // magnified atlas pixels must not become square splash patches.
            // Layer broken filaments over broad sheets. Periodic phase offsets
            // close at the authored animation loop; there is no independent clock.
            let turbulence = mineral_field(vec3<f32>(map.x * 0.72,
                map.y * 0.38 + sin(phase) * 0.30, 13.7 + cos(phase) * 0.30));
            let threads = mineral_field(vec3<f32>(map.x * 3.8,
                map.y * 1.15 + sin(phase) * 0.42, 37.2 + cos(phase) * 0.42));
            let drift = sin(map.y * 1.7 - phase + turbulence * 3.0) * 0.68;
            let ribbon = sin(map.x * 8.7 + drift + turbulence * 4.2);
            let streak = smoothstep(0.76, 0.99, ribbon);
            let breaks = smoothstep(0.27, 0.72, threads);
            let sheet = smoothstep(0.26, 0.76, turbulence);
            let froth = clamp(sheet * 0.29 + streak * breaks * 0.54, 0.0, 0.86);
            let falling_sheet = mix(vec3<f32>(0.035, 0.15, 0.19),
                vec3<f32>(0.62, 0.81, 0.80), froth);
            // The slope meets the lower pool at its real zero-height datum.
            // Feather flow into that pool, with irregular foam at the impact,
            // instead of ending the brighter waterfall in a straight color seam.
            let elevation = max(surface.world_position.y, 0.0) / max(water_enabled.y, 0.001);
            let impact = 1.0 - smoothstep(0.0, 0.75, elevation);
            // Two scales break the foam into small, soft clusters. Broad
            // patches alone read as enlarged source tiles at this camera angle.
            let mist = mineral_field(vec3<f32>(map.x * 9.3,
                map.y * 6.7 + sin(phase) * 0.24, 53.1 + cos(phase) * 0.24));
            let droplets = mineral_field(vec3<f32>(map.x * 18.6 + 7.1,
                map.y * 13.4 + sin(phase) * 0.38, 71.9 + cos(phase) * 0.38));
            let spray = smoothstep(0.32, 0.78, mist * 0.68 + droplets * 0.32);
            let landing = mix(rippled, vec3<f32>(0.43, 0.65, 0.64), spray * 0.38);
            let falling = mix(falling_sheet, landing, impact);
            // The same fine foam continues onto authenticated pool slots below
            // this cascade, with a smooth footprint in its own mask channel.
            let receiving_pool = mix(rippled, vec3<f32>(0.43, 0.65, 0.64),
                spray * 0.38 * water_impact(in.uv));
            surface.material.base_color = vec4<f32>(mix(receiving_pool, falling, kind.y), surface.material.base_color.a);
        }
    }
}
    let view_position = mesh_view_bindings::view.view_from_world * surface.world_position;
    var visibility = 1.0;
    var directional_energy = 0.0;
    for (var light_id = 0u; light_id < mesh_view_bindings::lights.n_directional_lights; light_id += 1u) {
        let light = mesh_view_bindings::lights.directional_lights[light_id];
        directional_energy = max(directional_energy,
            dot(light.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)));
        if (light.flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            visibility = min(visibility, fetch_directional_shadow(light_id,
                surface.world_position, surface.world_normal, view_position.z, surface.frag_coord.xy));
        }
    }
    // y/z are zero for every ordinary world material. Encounter-owned clones
    // carry the source black/white cue without altering the shared world atlas.
    // Retain authored face shading, but respond to the actual ambient and
    // directional light energies. The normal 80/3000 scene retains its palette;
    // dimmed cavern lights reduce every surface, including actors, uniformly.
    let ambient_gain = clamp(dot(mesh_view_bindings::lights.ambient_color.rgb,
        vec3<f32>(0.2126, 0.7152, 0.0722)) / 80.0, 0.0, 1.0);
    let sun_gain = clamp(directional_energy / 3000.0, 0.0, 1.0);
    var shaded = surface.material.base_color.rgb * (0.55 * ambient_gain + 0.45 * sun_gain * visibility);
    // Exterior grove daylight uses warm exposed leaves and cool, deeper
    // cast shade. Visibility still comes from the real shadow geometry, so
    // scrolling and orbit cannot move a painted patch of sunlight.
    if cave_reveal.z > 0.5 {
        let daylight = mix(vec3<f32>(0.32, 0.40, 0.44), vec3<f32>(1.04, 1.01, 0.90), visibility);
        shaded = surface.material.base_color.rgb * daylight;
    }
    let darkened = mix(shaded, vec3<f32>(0.0), clamp(cutaway.fade.y, 0.0, 1.0));
    let palette_color = mix(darkened, vec3<f32>(1.0), clamp(cutaway.fade.z, 0.0, 1.0));
    var out: FragmentOutput;
    out.color = vec4<f32>(palette_color,
        surface.material.base_color.a * (1.0 - cutaway.fade.x));
    out.color = main_pass_post_lighting_processing(surface, out.color);
    return out;
}
