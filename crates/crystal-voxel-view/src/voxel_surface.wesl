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
struct VoxelSurfaceUniform {
    cutaway: CutawayUniform,
    water_enabled: vec4<f32>,
    cave_reveal: vec4<f32>,
    water_deep: vec4<f32>,
    water_foam: vec4<f32>,
    water_bounds: vec4<f32>,
    water_style: vec4<f32>,
    water_phase: vec4<f32>,
    water_contact: vec4<f32>,
}
@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<uniform> terrain: VoxelSurfaceUniform;
@group(constants::MATERIAL_BIND_GROUP) @binding(101) var water_mask: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(102) var water_mask_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(103) var water_shore: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(104) var water_shore_sampler: sampler;
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
fn reveal_cave_player(world: vec4<f32>, pixel: vec2<f32>, aperture_only: bool) {
    if terrain.cave_reveal.x < 0.5 || terrain.cutaway.bottom_radius.w <= 0.0 { return; }
    // A faded inner wall must fade down to its base. Keeping its bottom at a
    // player-relative height left opaque stumps that looked like false steps.
    // Authored low surfaces and the outer rock enclosure keep their relief.
    if terrain.cave_reveal.w > 0.5
        && world.y <= terrain.cutaway.bottom_radius.y + terrain.cutaway.bottom_radius.w * 0.40 { return; }
    let bottom = mesh_view_bindings::view.view_from_world * vec4<f32>(terrain.cutaway.bottom_radius.xyz, 1.0);
    let top = mesh_view_bindings::view.view_from_world * vec4<f32>(terrain.cutaway.top_feather.xyz, 1.0);
    let fragment = mesh_view_bindings::view.view_from_world * world;
    let bottom_clip = mesh_view_bindings::view.clip_from_world * vec4<f32>(terrain.cutaway.bottom_radius.xyz, 1.0);
    let top_clip = mesh_view_bindings::view.clip_from_world * vec4<f32>(terrain.cutaway.top_feather.xyz, 1.0);
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
    let radius = terrain.cutaway.bottom_radius.w * 4.5;
    let feather = max(terrain.cutaway.top_feather.w, radius * 0.70);
    let distance = length(p - mix(b, t, along));
    let shoulder = 1.0 - smoothstep(radius - feather, radius, distance);
    let depth = fragment.z - mix(bottom.z, top.z, body_along);
    // Fade the full foreground wall, not only a hole around the trainer.
    // Compare on the foot plane: height must not turn a distant rear wall
    // into a foreground blocker under the pitched camera. Both stone and
    // ice use this same rule, and it follows the user's camera orbit.
    let foot_plane = mesh_view_bindings::view.view_from_world
        * vec4<f32>(world.x, terrain.cutaway.bottom_radius.y, world.z, 1.0);
    let front_depth = foot_plane.z - bottom.z;
    // Height alone can bring a distant rear wall into the body's projected
    // depth range. Its base must also be in front before opening an aperture;
    // otherwise tall enclosing rock develops floating holes behind the room.
    let foreground = smoothstep(-terrain.cutaway.bottom_radius.w * 0.35,
        0.0, front_depth);
    // Shallow frozen rims can cover the shoes while sitting only a few
    // world units in front. Their depth feather follows the body radius,
    // while the broad screen-space shoulder still dissolves gradually.
    let depth_feather = select(max(radius * 0.35, 0.16),
        max(terrain.cutaway.bottom_radius.w * 0.10, 0.16), aperture_only);
    let aperture = shoulder * smoothstep(0.15, depth_feather, depth) * foreground;
    // Feather across both sides of the foot plane. A one-sided transition
    // kept grazing side faces opaque immediately beside dissolved rock,
    // leaving thin diagonal slivers through otherwise readable passages.
    // Distant rear rock remains solid; front rock retains its faint outline.
    let wall_fade = 0.88 * smoothstep(-terrain.cutaway.bottom_radius.w * 3.0,
        terrain.cutaway.bottom_radius.w * 1.20, front_depth);
    let strength = max(aperture, select(wall_fade, 0.0, terrain.cave_reveal.w > 0.5 || aperture_only));
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

// Soft overlapping splash clusters avoid the square interpolation pattern
// of a value-noise lattice at enlarged waterfall landings. The caller moves
// this field only with the authenticated water phase. Wider unequal clusters
// overlap into foam patches instead of isolated dots. A nine-cell neighborhood
// covers their complete support; jitter stays within the middle half.
fn foam_clusters(p: vec2<f32>) -> f32 {
    let base = floor(p);
    var foam = 0.0;
    for (var z = -1; z <= 1; z = z + 1) {
        for (var x = -1; x <= 1; x = x + 1) {
            let cell = base + vec2<f32>(f32(x), f32(z));
            let jitter = vec2<f32>(mineral_hash(vec3<f32>(cell, 17.3)),
                mineral_hash(vec3<f32>(cell, 31.7)));
            let center = cell + vec2<f32>(0.25) + jitter * 0.5;
            let radius = mix(0.46, 0.95, mineral_hash(vec3<f32>(cell, 59.1)));
            let distance = length(p - center);
            foam = foam + 1.0 - smoothstep(radius * 0.20, radius, distance);
        }
    }
    return clamp(foam, 0.0, 1.0);
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

struct WaterOptics {
    color: vec3<f32>,
    normal: vec3<f32>,
    fresnel: f32,
    glint: f32,
}

fn shore_distance(map: vec2<f32>) -> f32 {
    if min(terrain.water_bounds.z, terrain.water_bounds.w) < 1.0 { return 8.0; }
    let uv = (map - terrain.water_bounds.xy) / terrain.water_bounds.zw;
    // Outside a retained grid there is no evidence of a bank. Clamping a dry
    // border texel into unknown water would create a moving foam curtain.
    if min(uv.x, uv.y) < 0.0 || max(uv.x, uv.y) > 1.0 { return 8.0; }
    return max((textureSampleLevel(water_shore, water_shore_sampler, uv, 0.0).r
        * 2.0 - 1.0) * 8.0, 0.0);
}

// Shared by live source pools and the existing modeled park basin. Analytic
// wave normals add volume without displacing swimming support or opening seams.
fn pool_optics(map: vec2<f32>, phase: f32, distance: f32, deep: vec3<f32>,
    shallow: vec3<f32>, crest: vec3<f32>, style: vec4<f32>, view: vec3<f32>) -> WaterOptics {
    let q = map * style.x;
    let cloud = mineral_field(vec3<f32>(map * 0.085, 91.3));
    let ripple_bend = mineral_field(vec3<f32>(map * 0.31
        + vec2<f32>(sin(phase), cos(phase)) * 0.06, 37.4));
    let bend = sin(q.x * 0.31 - q.y * 0.23) * 0.65 + cloud * 2.4 + ripple_bend * 4.0;
    let a = q.x * 0.91 + q.y * 1.47 + bend - phase;
    let b = q.x * 2.73 - q.y * 1.81 + bend * 0.61 + phase * 2.0;
    let c = q.x * 7.91 + q.y * 4.23 + sin(q.y * 0.67) * 0.7 - phase * 3.0;
    let wave = sin(a) * 0.62 + sin(b) * 0.28 + sin(c) * 0.10;
    let slope = (vec2<f32>(0.91, 1.47) * cos(a) * 0.62
        + vec2<f32>(2.73, -1.81) * cos(b) * 0.20
        + vec2<f32>(7.91, 4.23) * cos(c) * 0.035) * style.y;
    let normal = normalize(vec3<f32>(-slope.x, 1.0, -slope.y));
    let fresnel = 0.025 + 0.975 * pow(1.0 - clamp(dot(normal, view), 0.0, 1.0), 5.0);
    let shallows = 1.0 - smoothstep(0.15, 4.5, distance);
    let basin_tone = (cloud - 0.5) * 0.18;
    var color = mix(deep, shallow,
        clamp(0.16 + shallows * 0.68 + wave * 0.08 + basin_tone, 0.03, 0.96));
    // Fine intersecting caustic ribbons fade naturally as the bottom deepens.
    let caustic_a = q.x * 4.1 + q.y * 2.7 + sin(b) * 0.55 + sin(phase) * 0.35;
    let caustic_b = q.x * 2.6 - q.y * 4.8 + sin(a) * 0.70 - cos(phase) * 0.35;
    let caustic = pow(1.0 - abs(sin(caustic_a)), 10.0)
        * pow(0.5 + 0.5 * sin(caustic_b), 3.0);
    color = color + crest * caustic * shallows * 0.09;
    // Broken wash hugs the actual bank. A second weak band advances into the
    // shallows, with less foam in sheltered pools than on exposed coasts.
    let wash = 0.09 + (0.5 + 0.5 * sin(a + phase * 2.0)) * 0.14;
    let edge = 1.0 - smoothstep(0.02, wash, distance);
    let lap = (1.0 - smoothstep(0.035, 0.12, abs(distance - wash - 0.18))) * 0.28;
    let breaks = 0.35 + 0.65 * smoothstep(-0.3, 0.8, sin(b + cloud * 4.0));
    color = mix(color, mix(shallow, crest, 0.72), (edge + lap) * breaks * style.z);
    // Long irregular reflected crests replace repeated short bright bars.
    let glint = pow(max(sin(a), 0.0), 70.0)
        * pow(0.5 + 0.5 * sin(b), 2.5) * (0.3 + cloud * 0.7)
        * smoothstep(0.27, 0.67, ripple_bend);
    color = mix(color, crest, glint * 0.22);
    var result: WaterOptics;
    result.color = color;
    result.normal = normal;
    result.fresnel = fresnel;
    result.glint = glint;
    return result;
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var surface = pbr_input_from_standard_material(in, is_front);
    var water_normal = surface.world_normal;
    // Fresnel, source crest, exposed sky strength and active optical surface.
    var water_reflection = vec4<f32>(0.0);
    var enclosing_wall = false;
    var frozen_platform = false;
@if(VERTEX_UVS_A) {
    if in.uv.x > 90000.0 {
        let kind = round(in.uv.y / 100000.0);
        enclosing_wall = kind == 2.0 || kind == 3.0 || kind == 10.0 || kind == 18.0;
        frozen_platform = kind == 7.0 || kind == 9.0;
    }
}
    // Only enclosing rock participates. Walking planes, complete low stones,
    // authored stairs and live water retain their full geometry at all heights.
    // A walkable rock crown is the player's floor, not an occluding roof.
    // Preserve surfaces below their real support and the nearby upward cap
    // relief; otherwise the reveal removes the platform and exposes its base.
    let below_support = surface.world_position.y <= terrain.cutaway.bottom_radius.y + 0.02;
    // Only a nearly horizontal lip at the actual foot plane is support.
    // A sloping wall facet below the trainer's head is still an occluder;
    // preserving those facets leaves opaque blades inside faded rock.
    let supported_crown = surface.world_normal.y > 0.85
        && surface.world_position.y <= terrain.cutaway.bottom_radius.y
            + terrain.cutaway.bottom_radius.w * 0.10;
    if enclosing_wall && !(below_support || supported_crown) {
        reveal_cave_player(surface.world_position, in.position.xy, false);
    }
    // A complete ice platform can obstruct a trainer on its lower approach.
    // Reveal only the projected body aperture, retaining the platform outside
    // it and every surface at or below the authoritative supporting floor.
    if frozen_platform && !below_support {
        reveal_cave_player(surface.world_position, in.position.xy, true);
    }
    if terrain.cutaway.fade.w > 0.0 && surface.material.base_color.a < terrain.cutaway.fade.w { discard; }
    // A masked crown contains many overlapping leaves. Multiplying every
    // leaf's alpha makes even a faded crown opaque after enough layers. Share
    // one screen-stable coverage decision across those layers instead; retain
    // the original cutout and every leaf, without changing the shadow pass.
    let masked_cutaway = terrain.cutaway.fade.w > 0.0 && terrain.cutaway.fade.x > 0.0;
    if masked_cutaway {
        let coverage = fract(52.9829189 * fract(dot(floor(in.position.xy), vec2<f32>(0.06711056, 0.00583715))));
        if coverage < terrain.cutaway.fade.x { discard; }
    }
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
        let optics = pool_optics(map, time * 0.42, 1.8,
            vec3<f32>(0.025, 0.12, 0.16), vec3<f32>(0.085, 0.32, 0.34),
            vec3<f32>(0.40, 0.65, 0.61), vec4<f32>(1.1, 0.12, 0.0, 0.25), surface.V);
        water_normal = optics.normal;
        water_reflection = vec4<f32>(optics.fresnel, optics.glint, 0.25, 1.0);
        surface.material.base_color = vec4<f32>(optics.color, surface.material.base_color.a);
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
        // Broad cap plates use the regular cave-rock scale. Sampling this
        // same asset at 40 units packed many tiny painted lumps onto each
        // native tile and concealed the actual large mineral planes.
        let y_rock = textureSample(painted_bedrock, painted_bedrock_sampler, rock.xz / 144.0).rgb;
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
        let paint_strength = select(0.82, mix(0.50, 0.60, wall), canyon);
        // Pale weathered cap beds sit above the warmer sediment faces, as
        // in the source drawing. Blend over the actual shoulder normal so
        // the lighter top describes its shape without a painted border.
        let canyon_bed = mix(vec3<f32>(0.49, 0.37, 0.20), reference, wall);
        let broad_pigment = select(reference, canyon_bed * (0.96 + weather * 0.08), canyon);
        let pigment = mix(broad_pigment, paint, paint_strength);
        surface.material.base_color = vec4<f32>(pigment * surface.material.base_color.rgb / reference * detail,
            surface.material.base_color.a);
    }
    // Cave stone and dust use distinct positive map-coordinate tags.
    // Floors use broader, softer paint; source art and live water stay in the atlas.
    if in.uv.x > 90000.0 {
        let category = round(in.uv.y / 100000.0);
        let textured_cave = category == 18.0 || category == 20.0 || category == 22.0;
        let native_cave = category == 10.0 || category == 12.0 || category == 14.0 || textured_cave;
        let sliding_ice = category == 8.0;
        let shelf_shoulder = category == 9.0;
        let frozen = category == 3.0 || category == 5.0 || category == 7.0 || sliding_ice || shelf_shoulder;
        let floor_paint = category == 4.0 || category == 5.0 || category == 12.0 || category == 20.0 || sliding_ice;
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
        let mineral = select(ratio, vec3<f32>(grey), frozen || native_cave);
        // Retain the existing floor and frozen-rock treatment. Regular rock
        // uses quieter, bounded grain so its fractures and map-space pigment
        // carry the form without bright flecks over every geometric facet.
        let rock_pigment = select(clamp(mineral, vec3<f32>(0.55), vec3<f32>(1.65)), mineral, frozen || floor_paint);
        var paint_amount = select(select(0.42, 0.38, frozen), select(0.65, 0.38, frozen) * 0.32, floor_paint);
        if native_cave && !textured_cave { paint_amount = select(0.18, 0.06, floor_paint); }
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
    if terrain.water_enabled.x > 0.5 && !room_paint && !room_prop {
        let water = textureSample(water_mask, water_mask_sampler, in.uv);
        let kind = water.rg;
        if max(kind.x, kind.y) > 0.5 {
            // Interpolate only the last received source step, never predict a
            // future frame. The blend finishes even if source animation stops.
            let elapsed = (mesh_view_bindings::globals.time - terrain.water_phase.x + 3600.0) % 3600.0;
            let blend = clamp(elapsed / max(terrain.water_phase.y, 0.001), 0.0, 1.0);
            let step = select(terrain.water_phase.z, terrain.water_phase.w, kind.y > 0.75);
            let phase = water.b * 6.2831853 - step * (1.0 - blend);
            let map = (surface.world_position.xz + terrain.water_enabled.zw) / max(terrain.water_enabled.y, 0.001);
            let open_water = terrain.water_enabled.x == 2.0;
            let native_cave_water = terrain.water_enabled.x == 3.0;
            let source_water = surface.material.base_color.rgb;
            let palette_current = native_cave_water && terrain.water_deep.w > 0.5;
            let indoor_pool = terrain.water_style.w > 0.0 && terrain.water_style.w < 0.2;
            let deep_hint = select(select(vec3<f32>(0.018, 0.095, 0.125),
                vec3<f32>(0.023, 0.12, 0.18), open_water), vec3<f32>(0.018, 0.075, 0.24), indoor_pool);
            let crest_hint = select(select(vec3<f32>(0.24, 0.46, 0.46),
                vec3<f32>(0.48, 0.72, 0.65), open_water), vec3<f32>(0.40, 0.62, 0.90), indoor_pool);
            let shallow_hint = select(select(vec3<f32>(0.065, 0.23, 0.25),
                vec3<f32>(0.12, 0.39, 0.37), open_water), vec3<f32>(0.055, 0.26, 0.56), indoor_pool);
            let deep = select(deep_hint, terrain.water_deep.rgb, palette_current);
            let crest = select(crest_hint, terrain.water_foam.rgb, palette_current);
            let pool_tone = select(shallow_hint, mix(deep,crest,0.28),palette_current);
            let optics = pool_optics(map, phase, shore_distance(map), deep, pool_tone, crest,
                terrain.water_style, surface.V);
            var pool = optics.color;
            if terrain.water_contact.w > 0.5 && kind.x > 0.5 {
                // A swimmer disturbs only authenticated water, in map space.
                // Fine uneven rings make contact legible without moving the
                // floor, tinting the actor, or creating a separate decal mesh.
                let relative = map - terrain.water_contact.xy;
                let angle = atan2(relative.y, relative.x);
                let radius = length(relative) + sin(angle * 3.0 + phase) * 0.13
                    + sin(angle * 7.0 - phase * 2.0) * 0.05;
                let body = terrain.water_contact.z;
                let envelope = smoothstep(body * 0.65, body, radius)
                    * (1.0 - smoothstep(body * 1.05, body * 2.0, radius));
                let ring = pow(max(sin(radius * 8.5 - phase * 2.0), 0.0), 18.0);
                let arc = 0.28 + 0.72 * smoothstep(0.16, 0.55,
                    0.5 + 0.5 * sin(angle * 3.0 + radius * 4.0 + phase * 2.0));
                pool = mix(pool, crest, ring * envelope * arc * 0.18);
            }
            let rippled = pool;
            let vortex = kind.y > 0.25 && kind.y < 0.75;
            water_normal = optics.normal;
            water_reflection = vec4<f32>(optics.fresnel, optics.glint,
                terrain.water_style.w, select(1.0, 0.0, kind.y > 0.75));
            let pool_impact = water_impact(in.uv);
            var receiving_pool = rippled;
            var falling = rippled;
            // Lake fragments skip the cascade's turbulence and nine-cell foam
            // neighborhood. The richer pool optics do not add a waterfall pass
            // over every body of water.
            if kind.y > 0.75 || pool_impact > 0.001 {
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
            let streak = pow(max(ribbon, 0.0), 12.0);
            let breaks = smoothstep(0.27, 0.72, threads);
            let sheet = smoothstep(0.26, 0.76, turbulence);
            let filament = pow(max(sin(map.x * 21.3 + turbulence * 3.0
                + sin(map.y * 2.1 - phase) * 0.4), 0.0), 10.0) * breaks;
            let froth = clamp(sheet * 0.29 + streak * breaks * 0.36 + filament * 0.25, 0.0, 0.88);
            // Entrained air lightens the authenticated hue toward its own
            // brightest channel, never replacing a native cave palette with teal.
            let aerated = mix(crest, vec3<f32>(max(max(crest.r, crest.g), crest.b)), 0.28);
            let falling_sheet = mix(select(vec3<f32>(0.035, 0.15, 0.19), mix(deep,crest,0.10),palette_current),
                select(vec3<f32>(0.62, 0.81, 0.80),aerated,palette_current), froth);
            // The slope meets the lower pool at its real zero-height datum.
            // Feather flow into that pool, with irregular foam at the impact,
            // instead of ending the brighter waterfall in a straight color seam.
            // Cave receiving pools use the lower floor's zero datum. Treating
            // the native two-pixel recess as two whole source cells put the
            // foam ramp below the sheet and prevented a visible landing blend.
            let elevation = max(surface.world_position.y, 0.0) / max(terrain.water_enabled.y, 0.001);
            let impact = 1.0 - smoothstep(0.0, 0.75, elevation);
            // Rotate and bend the foam domain so the noise lattice cannot
            // appear as enlarged square pixels at the landing. Two moderate
            // scales form soft, overlapping clusters rather than bright grit.
            let foam_map = vec2<f32>(map.x * 0.82 - map.y * 0.57,
                map.x * 0.57 + map.y * 0.82);
            let foam_bend = vec2<f32>(sin(map.y * 1.3 + sin(phase)),
                cos(map.x * 1.1 + cos(phase))) * 0.19;
            let mist = foam_clusters((foam_map + foam_bend) * vec2<f32>(1.9, 1.6)
                + vec2<f32>(sin(phase), cos(phase)) * 0.24);
            let droplets = mineral_field(vec3<f32>((foam_map - foam_bend) * vec2<f32>(7.3, 6.1)
                + vec2<f32>(7.1 + sin(phase) * 0.38, cos(phase) * 0.38), 71.9));
            let spray = smoothstep(0.18, 0.94, mist * 0.62 + droplets * 0.38);
            let landing = mix(rippled, select(vec3<f32>(0.43, 0.65, 0.64),aerated,palette_current), spray * 0.26);
            falling = mix(falling_sheet, landing, impact);
            // The same fine foam continues onto authenticated pool slots below
            // this cascade, with a smooth footprint in its own mask channel.
            receiving_pool = mix(rippled, select(vec3<f32>(0.43, 0.65, 0.64),aerated,palette_current),
                spray * 0.26 * pool_impact);
            }
            // Only complete Dragon's Den source drawings use the middle
            // role value. Their current shares the adjacent live pool phase.
            var pool_surface = receiving_pool;
            if vortex {
                let pool_warp = mineral_field(vec3<f32>(map.x * 0.38 + sin(phase) * 0.12,
                    map.y * 0.43 + cos(phase) * 0.12, 91.3));
                let radial = fract(map / 4.0) * 4.0 - vec2<f32>(1.0);
                let radius = length(radial);
                let angle = atan2(radial.y, radial.x);
                let spiral = sin(angle * 3.0 + radius * 14.0 - phase * 2.0 + pool_warp * 1.1);
                let whirl_crest = smoothstep(0.70, 0.98, spiral)
                    * smoothstep(0.12, 0.40, radius);
                let hollow = 1.0 - smoothstep(0.08, 0.52, radius);
                let swirling = mix(pool, deep * 0.48, hollow * 0.70);
                let eddy = mix(swirling, crest, whirl_crest * 0.38);
                let whirl_blend = 1.0 - smoothstep(0.72, 1.0, radius);
                pool_surface = mix(receiving_pool, eddy, whirl_blend);
            }
            var flowing_water = mix(pool_surface, falling, select(kind.y, 0.0, vortex));
            if native_cave_water && !palette_current {
                // Missing authenticated palette art cannot select foreign
                // teal. Keep the actual source surface until it is available.
                flowing_water = source_water;
                water_reflection = vec4<f32>(0.0);
            }
            surface.material.base_color = vec4<f32>(flowing_water, surface.material.base_color.a);
        }
    }
}
    let view_position = mesh_view_bindings::view.view_from_world * surface.world_position;
    var visibility = 1.0;
    var directional_energy = 0.0;
    var sun_tint = vec3<f32>(1.0);
    var water_sun = 0.0;
    for (var light_id = 0u; light_id < mesh_view_bindings::lights.n_directional_lights; light_id += 1u) {
        let light = mesh_view_bindings::lights.directional_lights[light_id];
        let energy = dot(light.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        if energy > directional_energy {
            sun_tint = light.color.rgb / max(energy, 0.0001);
        }
        directional_energy = max(directional_energy,
            dot(light.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)));
        if water_reflection.w > 0.5 {
            let half_sum = surface.V + light.direction_to_light;
            let half_vector = half_sum / max(length(half_sum), 0.0001);
            water_sun = max(water_sun, pow(max(dot(water_normal, half_vector), 0.0), 96.0));
        }
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
    var indirect_occlusion = vec3<f32>(1.0);
@if(SCREEN_SPACE_AMBIENT_OCCLUSION) {
    // These palette-preserving materials bypass StandardMaterial's Lambert
    // response, so sample the native contact pass explicitly for sky fill.
    let ao_pixel = vec2<i32>(in.position.xy - mesh_view_bindings::view.viewport.xy);
    let contact = textureLoad(mesh_view_bindings::screen_space_ambient_occlusion_texture, ao_pixel, 0i).r;
    indirect_occlusion = vec3<f32>(mix(1.0, clamp(contact, 0.3, 1.0), 0.45));
}
    var shaded = surface.material.base_color.rgb * (indirect_occlusion * (0.55 * ambient_gain)
        + sun_tint * (0.45 * sun_gain * visibility));
    // Exterior grove daylight uses warm exposed leaves and cool, deeper
    // cast shade. Visibility still comes from the real shadow geometry, so
    // scrolling and orbit cannot move a painted patch of sunlight.
    if terrain.cave_reveal.z > 0.5 {
        // Broad sky fill keeps painted detail readable in shade. A modest
        // hemisphere response gives rounded leaves and raised roof courses
        // volume without applying full Lambert shading over authored paint.
        let sky = mix(vec3<f32>(0.52, 0.61, 0.69), vec3<f32>(0.64, 0.72, 0.78),
            clamp(surface.world_normal.y * 0.5 + 0.5, 0.0, 1.0));
        let daylight = mix(sky * indirect_occlusion, sun_tint * 0.96, visibility);
        let hemisphere = 0.94 + 0.06 * max(surface.world_normal.y, 0.0);
        shaded = surface.material.base_color.rgb * daylight * hemisphere;
    }
    if water_reflection.w > 0.5 {
        // Camera-dependent sky sheen and the actual directional light reflect
        // off the wave normals. Real scene shadows still suppress direct sun;
        // the normal scene energies also dim these effects at night/in caves.
        let sky = mix(vec3<f32>(0.15, 0.30, 0.36), vec3<f32>(0.46, 0.64, 0.68),
            clamp(water_normal.y * 0.5 + 0.5, 0.0, 1.0));
        let sky_weight = clamp((water_reflection.x * 0.65 + water_reflection.y * 0.045)
            * water_reflection.z, 0.0, 0.35);
        shaded = mix(shaded, sky * ambient_gain, sky_weight);
        shaded = shaded + sun_tint * water_sun * sun_gain * visibility
            * (0.08 + water_reflection.x * 0.24) * water_reflection.z;
    }
    let darkened = mix(shaded, vec3<f32>(0.0), clamp(terrain.cutaway.fade.y, 0.0, 1.0));
    let palette_color = mix(darkened, vec3<f32>(1.0), clamp(terrain.cutaway.fade.z, 0.0, 1.0));
    var out: FragmentOutput;
    let opacity = select(surface.material.base_color.a * (1.0 - terrain.cutaway.fade.x),
        1.0, masked_cutaway);
    out.color = vec4<f32>(palette_color, opacity);
    out.color = main_pass_post_lighting_processing(surface, out.color);
    return out;
}
