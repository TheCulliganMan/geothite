@if(PREPASS_PIPELINE)
import bevy_pbr::prepass::io::{VertexOutput, FragmentOutput};
@else
import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};
@if(PREPASS_PIPELINE && MOTION_VECTOR_PREPASS)
import bevy_pbr::render::pbr_prepass_functions::calculate_motion_vector;
import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT,
    pbr_functions::{alpha_discard, main_pass_post_lighting_processing},
    mesh_view_bindings,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    shadows::fetch_directional_shadow,
    pbr_lighting::ltc_integrate_quad,
};
@if(VOXEL_MATERIAL_LIGHTING && ENVIRONMENT_MAP)
import bevy_pbr::light_probe::environment_map::compute_radiances;
@if(VOXEL_MATERIAL_LIGHTING)
import bevy_pbr::render::{pbr_lighting::LayerLightingInput, clustered_forward::ClusterableObjectIndexRanges};

// Masks stay at exact LOD zero. Painted 3D detail uses gradients computed
// before any material branch/discard, keeping WGSL derivatives well-defined.
fn sample_detail(tex: texture_2d<f32>, tex_sampler: sampler,
    uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
@if(VOXEL_MATERIAL_LIGHTING)
    return textureSampleGrad(tex, tex_sampler, uv, dx, dy);
@else
    return textureSampleLevel(tex, tex_sampler, uv, 0.0);
}

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
    water_motion: vec4<f32>,
    water_contact: vec4<f32>,
    cave_exits: array<vec4<f32>, 4>,
    cave_sky: vec4<f32>,
    city_night: vec4<f32>,
    city_windows: array<vec4<f32>, 32>,
    city_window_planes: array<vec4<f32>, 32>,
    battle_clearance: array<vec4<f32>, 2>,
    battle_clearance_shape: array<vec4<f32>, 2>,
    battle_grass_contact: array<vec4<f32>, 2>,
    battle_contact_shape: array<vec4<f32>, 2>,
    battle_grass_mask_info: vec4<f32>,
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
@group(constants::MATERIAL_BIND_GROUP) @binding(109) var battle_grass_mask: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(110) var battle_grass_mask_sampler: sampler;

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
// geometry remain intact. The capsule follows source-backed body support.
fn reveal_player(world: vec4<f32>, pixel: vec2<f32>, aperture_only: bool, radius_scale: f32, mineral_dissolve: bool) {
    if terrain.cutaway.bottom_radius.w <= 0.0 { return; }
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
    let radius = terrain.cutaway.bottom_radius.w * radius_scale;
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
    // Keep the mineral dissolve screen-stable without the diagonal stripes
    // of interleaved-gradient noise on grazing faces. All rock layers share
    // this pixel sample; no frame seed means the shoulder cannot flicker.
    var coverage = fract(52.9829189 * fract(dot(floor(pixel), vec2<f32>(0.06711056, 0.00583715))));
    if mineral_dissolve {
        let pixel_id = vec2<u32>(floor(pixel));
        var seed = (pixel_id.x * 1597334677u) ^ (pixel_id.y * 3812015801u);
        seed = (seed ^ (seed >> 16u)) * 2246822519u;
        seed = (seed ^ (seed >> 13u)) * 3266489917u;
        seed = seed ^ (seed >> 16u);
        coverage = f32(seed >> 8u) * (1.0 / 16777216.0);
    }
    if strength > coverage { discard; }
}

fn reveal_cave_player(world: vec4<f32>, pixel: vec2<f32>, aperture_only: bool, radius_scale: f32) {
    if terrain.cave_reveal.x < 0.5 || terrain.cutaway.bottom_radius.w <= 0.0 { return; }
    reveal_player(world, pixel, aperture_only, radius_scale, false);
}

// The palette and baked face shade are the surface color. The sun pass
// contributes visibility only, without applying another PBR light response.
fn mineral_hash(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}

// A bounded four-corner field for town pigment; no texture or extra pass.
fn town_grain(p: vec2<f32>) -> f32 {
    let cell=floor(p);
    let t=fract(p);
    let f=t*t*(vec2<f32>(3.0)-2.0*t);
    return mix(mix(mineral_hash(vec3<f32>(cell,17.0)),
        mineral_hash(vec3<f32>(cell+vec2<f32>(1.0,0.0),17.0)),f.x),
        mix(mineral_hash(vec3<f32>(cell+vec2<f32>(0.0,1.0),17.0)),
        mineral_hash(vec3<f32>(cell+vec2<f32>(1.0),17.0)),f.x),f.y);
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

// Contact includes fitted rock feet; broad depth tint follows actual land.
fn shore_distances(map: vec2<f32>) -> vec2<f32> {
    if min(terrain.water_bounds.z, terrain.water_bounds.w) < 1.0 { return vec2<f32>(8.0); }
    let uv = (map - terrain.water_bounds.xy) / terrain.water_bounds.zw;
    // Outside a retained grid there is no evidence of a bank. Clamping a dry
    // border texel into unknown water would create a moving foam curtain.
    if min(uv.x, uv.y) < 0.0 || max(uv.x, uv.y) > 1.0 { return vec2<f32>(8.0); }
    return max((textureSampleLevel(water_shore, water_shore_sampler, uv, 0.0).rg
        * 2.0 - 1.0) * 8.0, vec2<f32>(0.0));
}

// Pools, lakes and seas share a level surface. Depth tint and a quiet bank
// edge describe the water without rolling normals, crests or moving bands.
fn pool_optics(map: vec2<f32>, distance: f32, depth_distance: f32, deep: vec3<f32>,
    shallow: vec3<f32>, crest: vec3<f32>, style: vec4<f32>, view: vec3<f32>) -> WaterOptics {
    let cloud = mineral_field(vec3<f32>(map * 0.085, 91.3));
    let normal = vec3<f32>(0.0, 1.0, 0.0);
    let fresnel = 0.025 + 0.975 * pow(1.0 - clamp(dot(normal, view), 0.0, 1.0), 5.0);
    let shallows = 1.0 - smoothstep(0.15, 3.5, depth_distance);
    let basin_tone = (cloud - 0.5) * 0.08;
    var color = mix(deep, shallow,
        clamp(0.16 + shallows * 0.68 + basin_tone, 0.03, 0.96));
    let edge = 1.0 - smoothstep(0.02, 0.09, distance);
    color = mix(color, mix(shallow, crest, 0.35), edge * style.z * 0.25);
    var result: WaterOptics;
    result.color = color;
    result.normal = normal;
    result.fresnel = fresnel;
    result.glint = 0.0;
    return result;
}

// Ripples on proven regional water. Analytic wave slopes drive the
// reflection normal; the authored plane stays level at banks and under Surf.
// Unequal directions/wavelengths avoid a scrolling tiled normal map. The
// caller supplies only motion confirmed by the source animation, so pausing
// the game holds the water too. Fine waves fade below a pixel at distant zoom.
fn coastal_wave_packet(p: vec2<f32>, direction: vec2<f32>, frequency: f32,
    cross_frequency: f32, phase: f32, cross_phase: f32) -> vec3<f32> {
    let across = vec2<f32>(-direction.y, direction.x);
    let carrier = dot(p,direction)*frequency + phase;
    let envelope_phase = dot(p,across)*cross_frequency + cross_phase;
    let envelope = 0.55 + 0.45*sin(envelope_phase);
    // Include the envelope's slope as well as the carrier's. A rotating
    // camera then sees real short wave groups instead of endless mirror bands.
    let gradient = direction*envelope*cos(carrier)
        + across*(0.45*cross_frequency/frequency)*cos(envelope_phase)*sin(carrier);
    return vec3<f32>(envelope*sin(carrier),gradient);
}

fn rippled_pool_optics(map: vec2<f32>, distance: f32, depth_distance: f32, phase: f32, pixel: f32,
    deep: vec3<f32>, shallow: vec3<f32>, crest: vec3<f32>,
    style: vec4<f32>, view: vec3<f32>) -> WaterOptics {
    let p = map * style.x * 1.75;
    let d0 = vec2<f32>(0.940, 0.342);
    let d1 = vec2<f32>(-0.447, 0.894);
    let d2 = vec2<f32>(0.781, -0.625);
    let d3 = vec2<f32>(0.242, 0.970);
    let a = dot(p, d0) * 0.82 - phase * 2.0;
    let b = dot(p, d1) * 1.37 + phase * 3.0;
    let c = dot(p, d2) * 3.10 - phase * 5.0;
    let d = dot(p, d3) * 6.20 + phase * 7.0;
    let fine = 1.0 - smoothstep(0.3, 1.2, pixel * style.x * 1.75 * 6.20);
    var long0 = vec3<f32>(sin(a),d0*cos(a));
    var long1 = vec3<f32>(sin(b),d1*cos(b));
    var crest_field = sin(c) * 0.65 + sin(b) * 0.35;
    var slope = long0.yz * 0.56 + long1.yz * 0.28
        + (d2 * cos(c) * 0.12 + d3 * cos(d) * 0.04) * fine;
    if style.w > 0.25 {
        // Outdoor ponds and seas share finite crossing wavelets. Their
        // separate strength/scale settings retain calm inland water; indoor
        // pools keep their short waves. Integer phase rates close at TAU.
        long0 = coastal_wave_packet(p,d0,0.82,0.63,-phase*2.0,phase);
        long1 = coastal_wave_packet(p,d1,1.37,0.91,phase*3.0,-phase*2.0);
        let chop0 = coastal_wave_packet(p,d2,3.10,2.30,-phase*5.0,-phase*3.0);
        let chop1 = coastal_wave_packet(p,d3,6.20,4.10,phase*7.0,phase*2.0);
        let middle = 1.0 - smoothstep(0.3,1.2,pixel*style.x*1.75*3.10);
        // Long swells carry volume, but short crossing wavelets carry most
        // of the visible normal. Otherwise sunlight and reflected clouds
        // stretch into parallel ribbons across the entire coastal surface.
        // Each wavelet has its own bounded crosswind envelope and derivative.
        slope = long0.yz * 0.18 + long1.yz * 0.13
            + chop0.yz * 0.42 * middle + chop1.yz * 0.27 * fine;
        // Bright crests need the same finite crossing groups. The former
        // two nearly parallel carriers painted long silver atlas-like marks
        // even with all reflection lighting disabled.
        crest_field = chop0.x * 0.65 + chop1.x * 0.35;
    }
    // Capillary motion damps at the actual shore rather than clipping through
    // piers and rocks. The same retained distance field gives islands shallows.
    let strength = style.y * mix(0.35, 1.4, smoothstep(0.0, 1.2, distance));
    let normal = normalize(vec3<f32>(-slope.x * strength, 1.0, -slope.y * strength));
    let facing = clamp(dot(normal, view), 0.0, 1.0);
    let fresnel = 0.02 + 0.98 * pow(1.0 - facing, 5.0);
    let swell = long0.x * 0.55 + long1.x * 0.30 + sin(c) * 0.15 * fine;
    let shallows = exp(-depth_distance * 0.72);
    var color = mix(deep, shallow, shallows * 0.85);
    color *= 0.94 + swell * 0.14;
    // Short broken silver crests leave broad quiet areas between wavelets.
    // Periodic drift stays continuous when the retained motion wraps at TAU.
    let breakup = town_grain(p * 0.72 + vec2<f32>(sin(phase), cos(phase)) * 0.22);
    let ridge = smoothstep(0.72, 0.97, crest_field);
    let glint = ridge * smoothstep(0.38, 0.75, breakup) * fine;
    color = mix(color, crest, glint * style.y * 0.60);
    // Refracted light makes a soft moving network in the shallow margin.
    // It disappears into blue absorption offshore instead of covering the
    // whole sea with a repeated bright pattern.
    let caustic_field = sin(c + sin(a)) + sin(b + sin(c));
    var caustic = pow(1.0 - min(abs(caustic_field), 1.0), 5.0) * fine;
    if style.w > 0.25 && style.w < 0.39 {
        // A narrow outdoor pond must not magnify two continuous carriers
        // into silver ribbons. Rotated, gently warped contours form a soft
        // broken light network; indoor pools retain their existing caustics.
        let q=vec2<f32>(p.x*0.82-p.y*0.57,p.x*0.57+p.y*0.82)*2.4
            +vec2<f32>(sin(phase),cos(phase))*0.24
            +vec2<f32>(breakup-0.5,long0.x)*0.24;
        let contour=abs(town_grain(q)-0.5)*5.0;
        caustic=pow(1.0-min(contour,1.0),6.0)*fine*0.35;
    }
    // Open coastal water has no authored visible bed to receive pool-floor
    // caustics. Pools and sheltered water retain their existing treatment.
    color += shallow * caustic * shallows * select(0.24, 0.0, style.w > 0.39);
    // One feathered, broken wash follows banks; no repeated contour bands or
    // foam at the unknown edge of a streamed region. Live phase drives its lap.
    // A small sea-rock foot carries a thin broken wash, not the broad bright
    // beach margin of a real bank. Dense native rows must not merge into a
    // lattice of pale square islands beneath the unchanged rocks.
    let rock_contact = smoothstep(0.05, 0.75, depth_distance - distance);
    let wash_width = mix(0.16, 0.06, rock_contact)
        + mix(0.10, 0.03, rock_contact) * sin(phase * 2.0 + breakup * 5.0);
    let wash = (1.0 - smoothstep(wash_width, wash_width + mix(0.22, 0.10, rock_contact), distance))
        * smoothstep(0.20, 0.64, breakup);
    color = mix(color, crest, wash * style.z * 0.65 * mix(1.0, 0.45, rock_contact));
    var result: WaterOptics;
    result.color = color;
    result.normal = normal;
    result.fresnel = fresnel;
    result.glint = glint;
    return result;
}

@fragment
fn fragment(raw: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var in=raw;
    var source_atlas=false;
    var source_mineral=false;
    var source_concrete=false;
@if(VERTEX_UVS_A) {
    source_concrete=in.uv.x>=-7.0 && in.uv.x<=-6.0 && in.uv.y>=0.0 && in.uv.y<=1.0;
    source_mineral=in.uv.x>=-5.0 && in.uv.x<=-4.0 && in.uv.y>=0.0 && in.uv.y<=1.0;
    source_atlas=(in.uv.x>=-3.0 && in.uv.x<=-2.0 && in.uv.y>=0.0 && in.uv.y<=1.0) || source_mineral || source_concrete;
    if source_atlas {in.uv.x=select(select(-2.0,-4.0,source_mineral),-6.0,source_concrete)-in.uv.x;}
}
    // Decode before the standard atlas sample and every mask/prepass path.
    var surface = pbr_input_from_standard_material(in, is_front);
    source_atlas=source_atlas && (surface.material.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT)!=0u;
    source_mineral=source_mineral && source_atlas;
    if source_mineral {
        let native=surface.material.base_color.rgb;
        let value=(dot(native,vec3<f32>(0.2126,0.7152,0.0722))*0.58
            +max(max(native.r,native.g),native.b)*0.42)*0.68;
        let mineral=vec3<f32>(0.90,0.96,1.0)*value;
        surface.material.base_color=vec4<f32>(mix(native,mineral,0.90),surface.material.base_color.a);
    }
    let world_dx = dpdx(surface.world_position.xyz);
    let world_dy = dpdy(surface.world_position.xyz);
    source_concrete=source_concrete && source_atlas;
    if source_concrete {
        let p=(surface.world_position.xz+terrain.water_enabled.zw)
            /max(terrain.water_enabled.y,0.001);
        let pixel=max(length(world_dx.xz),length(world_dy.xz))
            /max(terrain.water_enabled.y,0.001);
        let fine=1.0-smoothstep(0.5,1.5,pixel*42.0);
        let grain=(town_grain(p*42.0)-0.5)*0.13*fine;
        let weather=(town_grain(p*0.85)-0.5)*0.065;
        let slab=(town_grain(floor(p*0.5)+vec2<f32>(17.0,31.0))-0.5)*0.08;
        let native=surface.material.base_color.rgb;
        let value=dot(native,vec3<f32>(0.2126,0.7152,0.0722));
        let concrete=mix(native,vec3<f32>(value),0.65);
        let pigment=vec3<f32>(0.66,0.65,0.63)*(1.0+grain+weather+slab);
        surface.material.base_color=vec4<f32>(concrete*pigment,surface.material.base_color.a);
    }
@if(VERTEX_UVS_A)
    let uv_dx = dpdx(in.uv);
@if(VERTEX_UVS_A)
    let uv_dy = dpdy(in.uv);
    var water_normal = surface.world_normal;
    // Fresnel, source crest, exposed sky strength and active optical surface.
    var water_reflection = vec4<f32>(0.0);
    var wind_water = false;
    var enclosing_wall = false;
    var frozen_platform = false;
    var cave_portal = false;
    var styled_outdoor_rock = false;
    var source_painted = source_atlas;
    var source_paving = false;
    var source_road = false;
    var source_fabric = false;
    var town_source_paint = false;
    var town_ceramic = false;
    var town_glass = false;
    var source_glass = false;
    var luminous_glass = false;
    var city_source_window = false;
    var town_timber = false;
    var town_paint_kind = -1.0;
    var timber_pigment = 1.0;
    var pane = vec2<f32>(0.0);
    var source_forest_floor = false;
    var woodland_floor_light = vec3<f32>(1.0);
@if(VERTEX_UVS_A) {
    var town_kind=round((in.uv.y-1024.0)/32.0);
    var town_v=in.uv.y-(1024.0+town_kind*32.0);
    let town_slate=abs(in.uv.x)<16.0 && abs(in.uv.y-1328.0)<8.0;
    if town_slate {
        town_kind=4.0;
        town_v=in.uv.y-1328.0;
    }
    town_source_paint=town_kind>=0.0 && (town_kind<=3.0 || town_slate)
        && abs(in.uv.x)<16.0 && abs(town_v)<8.0;
    town_ceramic=town_source_paint && town_kind==0.0;
    if town_source_paint { town_paint_kind = town_kind; }
    pane=vec2<f32>(in.uv.x,in.uv.y-1152.0);
    town_glass=all(pane>=vec2<f32>(0.0)) && all(pane<=vec2<f32>(1.0));
    luminous_glass=town_glass;
    var source_pane=vec2<f32>(in.uv.x,in.uv.y-1216.0);
    let source_lamp=source_pane.y>=2.0 && source_pane.y<=3.0;
    source_pane.y-=select(0.0,2.0,source_lamp);
    source_glass=all(source_pane>=vec2<f32>(0.0)) && all(source_pane<=vec2<f32>(1.0));
    if source_glass {pane=source_pane; luminous_glass=source_lamp;}
    town_glass=town_glass || source_glass;
    let timber=vec2<f32>(in.uv.x,in.uv.y-1184.0);
    town_timber=all(timber>=vec2<f32>(0.0)) && all(timber<vec2<f32>(16.0));
    city_source_window = in.uv.y >= 1280.0 && in.uv.y <= 1280.25;
    source_road=in.uv.x>=79500.0 && in.uv.x<81500.0
        && in.uv.y>=-5000.0 && in.uv.y< -2000.0;
    source_paving=(in.uv.x>=82500.0 && in.uv.x<84000.0
        && in.uv.y>=-5000.0 && in.uv.y< -2000.0) || source_road;
    source_fabric = in.uv.x >= 82000.0 && in.uv.x < 82100.0
        && in.uv.y >= 3500.0 && in.uv.y < 4000.0;
    source_painted = source_painted || (abs(in.uv.x - 86100.0) < 0.5 && abs(in.uv.y - 85000.0) < 0.5) || source_fabric || source_paving || city_source_window || abs(in.uv.x - 85000.0) < 0.5 || abs(in.uv.x - 86200.0) < 0.5 || town_source_paint || town_glass || town_timber;
    if abs(in.uv.x - 86100.0) < 0.5 && abs(in.uv.y - 85000.0) < 0.5 {
        // The existing rooted sea-stone domain retains its native pigment.
        // Continuous world coordinates weld grain across the sculpt's paint
        // seams; fine pores fade before they can shimmer at a distance.
        let rock=vec3<f32>(surface.world_position.x+terrain.water_enabled.z,
            surface.world_position.y,surface.world_position.z+terrain.water_enabled.w)
            /max(terrain.water_enabled.y,0.001);
        let pixel=max(length(world_dx),length(world_dy))/max(terrain.water_enabled.y,0.001);
        let detail=1.0-smoothstep(0.35,1.0,pixel*28.0);
        let broad=mineral_field(rock*2.6)-0.5;
        let pores=mineral_field(rock*28.0)-0.5;
        let pigment=1.0+broad*0.22+pores*0.22*detail;
        surface.material.base_color=vec4<f32>(surface.material.base_color.rgb*pigment,
            surface.material.base_color.a);
    }
    if source_paving {
        let p=vec2<f32>(in.uv.x-select(83000.0,80000.0,source_road),-3000.0-in.uv.y);
        let footprint=max(fwidth(in.uv.x),fwidth(in.uv.y));
        let detail=1.0-smoothstep(0.6,1.5,footprint*36.0);
        let broad=town_grain(p*2.2)-0.5;
        let fine=town_grain(p*36.0)-0.5;
        var pigment=1.0+broad*0.06+fine*0.12*detail;
        if source_road {
            // Authored blank pavement stays seamless. Mineral aggregate and
            // broad wear read at the normal camera distance without a fake bond.
            let rotated=vec2<f32>(p.x*0.8-p.y*0.6,p.x*0.6+p.y*0.8);
            let wear=town_grain(rotated*0.55)-0.5;
            let aggregate=town_grain(rotated*8.0)-0.5;
            let aggregate_detail=1.0-smoothstep(0.5,1.5,footprint*8.0);
            let stone_dx=vec2<f32>(uv_dx.x*0.8+uv_dx.y*0.6,uv_dx.x*0.6-uv_dx.y*0.8)*1.25;
            let stone_dy=vec2<f32>(uv_dy.x*0.8+uv_dy.y*0.6,uv_dy.x*0.6-uv_dy.y*0.8)*1.25;
            let stone=sample_detail(painted_bedrock,painted_bedrock_sampler,
                rotated*1.25,stone_dx,stone_dy).rgb;
            let mineral=clamp(dot(stone/vec3<f32>(0.25,0.21,0.16),
                vec3<f32>(0.2126,0.7152,0.0722)),0.65,1.35);
            pigment=(1.0+wear*0.08+broad*0.04+aggregate*0.06*aggregate_detail
                +fine*0.04*detail)*mix(1.0,mineral,0.30);
        }
        surface.material.base_color=vec4<f32>(surface.material.base_color.rgb*pigment,
            surface.material.base_color.a);
    }
    let timber_footprint=max(fwidth(in.uv.x)*36.0,fwidth(in.uv.y)*0.8);
    if town_timber {
        // Grain follows the member's long axis in authored scene coordinates.
        // Fine fibers fade with pixel footprint instead of shimmering on rails.
        let detail=1.0-smoothstep(0.35,1.0,timber_footprint);
        let fiber=town_grain(timber*vec2<f32>(36.0,0.8))-0.5;
        let broad=town_grain(timber*vec2<f32>(5.0,0.25))-0.5;
        timber_pigment=1.0+fiber*0.07*detail+broad*0.025;
    }
    let town_footprint=max(fwidth(in.uv.x),fwidth(in.uv.y));
    if town_source_paint {
        let p=vec2<f32>(in.uv.x,town_v);
        let detail=1.0-smoothstep(0.35,1.0,town_footprint*32.0);
        let broad=town_grain(p*2.2)-0.5;
        let fine=town_grain(p*32.0)-0.5;
        var pigment=1.0+broad*0.06+fine*0.04*detail;
        if town_kind==1.0 {pigment=1.0+broad*0.035+fine*0.06*detail;}
        if town_kind==2.0 {pigment=1.0+broad*0.035+fine*0.08*detail;}
        if town_kind==3.0 {pigment=1.0+broad*0.02+fine*0.035*detail;}
        if town_kind==4.0 {
            let cleft=town_grain(p*vec2<f32>(5.5,1.1))-0.5;
            pigment=1.0+broad*0.10+cleft*0.10+fine*0.16*detail;
        }
        surface.material.base_color=vec4<f32>(surface.material.base_color.rgb*pigment,
            surface.material.base_color.a);
    }
    source_forest_floor = in.uv.x > 75000.0 && in.uv.x < 79000.0
        && round(-in.uv.y / 10000.0) == 7.0;
    let source_planting = in.uv.x >= 78000.0 && in.uv.x < 79000.0
        && round(-in.uv.y / 10000.0) == 8.0;
    source_painted = source_painted || source_forest_floor || source_planting;
    if in.uv.x > 90000.0 {
        let kind = round(in.uv.y / 100000.0);
        enclosing_wall = kind == 2.0 || kind == 3.0 || kind == 10.0 || kind == 18.0;
        frozen_platform = kind == 7.0 || kind == 9.0;
        cave_portal = kind == 32.0 || kind == 34.0;
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
        reveal_cave_player(surface.world_position, in.position.xy, false, 4.5);
    }
    // A complete ice platform can obstruct a trainer on its lower approach.
    // Reveal only the projected body aperture, retaining the platform outside
    // it and every surface at or below the authoritative supporting floor.
    if frozen_platform && !below_support {
        reveal_cave_player(surface.world_position, in.position.xy, true, 4.5);
    }
    // The doorway may stand between the camera and a trainer immediately
    // inside it. Open only the body aperture, retaining the arch silhouette,
    // the source floor, and the original shadow geometry.
    if cave_portal && !below_support {
        reveal_cave_player(surface.world_position, in.position.xy, true, 1.0);
    }

@if(VERTEX_UVS_A) {
    // Native outdoor canyon stone can cover the trainer in a narrow pass.
    // Open only the projected body, preserving the ridge outside that aperture
    // and every surface at or below the authoritative walking support.
    let outdoor_canyon = in.uv.x < -9000.0 && in.uv.y < -30000.0;
    if outdoor_canyon && terrain.cave_reveal.z > 0.5
        && surface.world_position.y > terrain.cutaway.bottom_radius.y + 0.02 {
        reveal_player(surface.world_position, in.position.xy, true, 1.5, true);
    }
}
    // A flat transparent shadow can disappear beneath the retained terrain.
    // Apply contact occlusion to the supporting floor itself and to grass roots.
    // World-space feet/footprint come from the presented neutral body; zero
    // radii leave the overworld and absent/fainted actors unchanged.
    var contact_grass = false;
@if(VERTEX_UVS_A) {
    contact_grass = abs(in.uv.x - 84000.0) < 0.5;
}
    if contact_grass || surface.world_normal.y > 0.65 {
        for (var actor = 0u; actor < 2u; actor = actor + 1u) {
            let contact = terrain.battle_grass_contact[actor];
            let shape = terrain.battle_contact_shape[actor];
            if contact.w > 0.0 && all(shape.zw > vec2<f32>(0.0)) {
                let delta = surface.world_position.xz - contact.xz;
                let local = vec2<f32>(dot(delta, shape.xy), dot(delta, vec2<f32>(-shape.y, shape.x)));
                let distance = length(local / max(shape.zw, vec2<f32>(0.04)));
                let radial = 1.0 - smoothstep(0.20, 1.20, distance);
                let height = surface.world_position.y - contact.y;
                let floor_band = 1.0 - smoothstep(0.015, 0.12, abs(height));
                let root_band = (1.0 - smoothstep(0.04, 0.35, max(height, 0.0)))
                    * smoothstep(-0.08, -0.01, height);
                let support = select(floor_band, root_band, contact_grass);
                surface.material.base_color = vec4<f32>(surface.material.base_color.rgb
                    * (1.0 - 0.55 * radial * support), surface.material.base_color.a);
            }
        }
    }
    // Retain the real route foliage while opening a camera aperture around
    // both complete battlers. Only explicitly tagged native grass participates;
    // floors, architecture and the overworld (zero radii) remain untouched.
@if(VERTEX_UVS_A) {
    // Material tags travel through perspective interpolation. Match the
    // neighboring painted-material tolerance rather than requiring exact bits.
    if abs(in.uv.x - 84000.0) < 0.5 {
        let grass_view = mesh_view_bindings::view.view_from_world * surface.world_position;
        for (var actor = 0u; actor < 2u; actor = actor + 1u) {
            let clearance = terrain.battle_clearance[actor];
            if clearance.w > 0.0 {
                let center_view = mesh_view_bindings::view.view_from_world * vec4<f32>(clearance.xyz, 1.0);
                let shape = terrain.battle_clearance_shape[actor];
                let depth = select(clearance.w, shape.z, shape.z > 0.0);
                // Foreground grass is invisible during battle. Small Pokémon
                // must stay completely clear; source grass behind them remains.
                if grass_view.z > center_view.z - depth {
                    discard;
                }
            }
        }
    }
}
@if(VOXEL_SOURCE_ALPHA_CUTOFF) {
    if terrain.cutaway.fade.w > 0.0 && surface.material.base_color.a < terrain.cutaway.fade.w { discard; }
}
    // A crown contains many overlapping leaves. Multiplying every
    // leaf's alpha makes even a faded crown opaque after enough layers. Share
    // one screen-stable coverage decision across those layers instead; retain
    // the original cutout and every leaf, without changing the shadow pass.
    // Positive W retains a source alpha cutoff; negative W enables coverage
    // for solid palette needles without inventing an alpha cutout.
    var masked_cutaway = terrain.cutaway.fade.w != 0.0 && terrain.cutaway.fade.x > 0.0;
@if(VOXEL_SMOOTH_DEPTH_FADE) {
    // Native alpha cutouts still reject their transparent texels above, but
    // must not turn a smooth linked caption back into whole-object stipple.
    masked_cutaway = false;
    // Authored buildings and source furniture carry their blocked body capsule.
    // Retain the smooth ghost outside it, with the same clear aperture as
    // the coverage path. Other layered alpha keeps a zero capsule radius.
    if terrain.cutaway.fade.x > 0.0 {
        reveal_player(surface.world_position, in.position.xy, true, 2.0, false);
    }
}
@if(VOXEL_COVERAGE_FADE) {
    if masked_cutaway {
        // Building reveal keeps a faint complete silhouette, but its retained
        // pixels must not stipple a character. Buildings and opted-in furniture
        // supply a body capsule; foliage and room walls retain their policy.
        reveal_player(surface.world_position, in.position.xy, true, 2.0, false);
        let coverage = fract(52.9829189 * fract(dot(floor(in.position.xy), vec2<f32>(0.06711056, 0.00583715))));
        if coverage < terrain.cutaway.fade.x { discard; }
    }
}
    surface.material.base_color = alpha_discard(surface.material.flags, surface.material.alpha_cutoff, surface.material.base_color);
@if(PREPASS_PIPELINE) {
    // The custom unlit paint still has geometry. Store its normal after the
    // exact same source mask, stochastic fade and camera-aperture decisions;
    // otherwise SSAO shades invisible cards and fills the opened cave view.
    var out: FragmentOutput;
    out.normal = vec4<f32>(surface.N * 0.5 + vec3<f32>(0.5), 1.0);
@if(UNCLIPPED_DEPTH_ORTHO_EMULATION)
    out.frag_depth = in.unclipped_depth;
@if(MOTION_VECTOR_PREPASS)
    out.motion_vector = calculate_motion_vector(in.world_position, in.previous_world_position);
    return out;
}
@else {
    // The mask is derived from proven source animation slots in the owned
    // atlas. Its authenticated phase drives continuous map-space ripples;
    // waterfall detail follows those same live frames.
    // Untagged paint, geometry, alpha and encounter cues remain unchanged.
@if(VERTEX_UVS_A) {
    // Verified outdoor floors use one existing material-texture read. The
    // coordinates are authored map cells: neither scrolling nor orbit can
    // drag the roughness across the ground. Accents never carry this tag.
    if ((in.uv.x > 65000.0 && in.uv.x < 69000.0)
        || (in.uv.x > 75000.0 && in.uv.x < 79000.0)) && in.uv.y < -55000.0 {
        let role = round(-in.uv.y / 10000.0);
        let map = vec2<f32>(in.uv.x - select(66000.0, 76000.0, in.uv.x > 75000.0),
            in.uv.y + role * 10000.0);
        let grain_scale = select(2.0, 6.0, role == 7.0);
        let grain = sample_detail(painted_bedrock, painted_bedrock_sampler, map / grain_scale,
            uv_dx / grain_scale, uv_dy / grain_scale).rgb;
        let pigment = clamp(grain / vec3<f32>(0.25, 0.21, 0.16), vec3<f32>(0.4), vec3<f32>(1.6));
        let tone = dot(pigment, vec3<f32>(0.2126, 0.7152, 0.0722));
        var craft = mix(vec3<f32>(0.80), vec3<f32>(1.16), smoothstep(0.45, 1.5, tone));
        if role == 7.0 {
            if source_forest_floor {
                // Live pigment already contains the forest's source palette;
                // retain grain relief without adding brown/green paint.
                craft = vec3<f32>(mix(0.85, 1.07, smoothstep(0.45, 1.5, tone)));
            } else {
            // Deep humus and bright moss form contiguous material patches.
            // A broad green shoulder survives the forest's cool source light.
            let moss = smoothstep(0.72, 1.32, pigment.g);
            craft = mix(vec3<f32>(0.62, 0.64, 0.67), vec3<f32>(0.65, 0.87, 0.83), moss);
            // Broad filtered warmth separates clearings from cool humus. The
            // retained grain supplies this field without another texture read.
            woodland_floor_light = mix(vec3<f32>(0.78, 0.88, 0.90),
                vec3<f32>(1.08, 1.03, 0.92), smoothstep(0.85, 1.30, tone));
            }
        } else if role == 6.0 || role == 11.0 {
            // Packed dusty ground has darker, warmer worn patches, rather
            // than the former uniformly pale slab between scattered specks.
            craft = mix(vec3<f32>(0.85, 0.81, 0.74), vec3<f32>(1.07, 1.05, 0.99),
                smoothstep(0.45, 1.45, tone));
        } else if role == 8.0 {
            craft = mix(vec3<f32>(0.85), vec3<f32>(1.10), smoothstep(0.45, 1.5, tone));
        }
        surface.material.base_color = vec4<f32>(surface.material.base_color.rgb * craft,
            surface.material.base_color.a);
        if role == 10.0 || role == 11.0 {
            // These roles belong solely to authenticated shoreline caps.
            // Terrace fitting can expose a cap as a steep face: use mineral
            // paint there, while retaining the original gentle-bank finish.
            let exposed = 1.0-smoothstep(0.60,0.84,abs(surface.world_normal.y));
            let rock = vec3<f32>(map.x*8.0,surface.world_position.y,map.y*8.0);
            let rock_dx = vec3<f32>(uv_dx.x*8.0,world_dx.y,uv_dx.y*8.0);
            let rock_dy = vec3<f32>(uv_dy.x*8.0,world_dy.y,uv_dy.y*8.0);
            let raw_weights = pow(abs(surface.world_normal),vec3<f32>(2.0));
            let weights = raw_weights/max(dot(raw_weights,vec3<f32>(1.0)),0.0001);
            let stone_x = sample_detail(painted_stone,painted_stone_sampler,
                rock.zy/128.0,rock_dx.zy/128.0,rock_dy.zy/128.0).rgb;
            let stone_y = sample_detail(painted_stone,painted_stone_sampler,
                rock.xz/128.0,rock_dx.xz/128.0,rock_dy.xz/128.0).rgb;
            let stone_z = sample_detail(painted_stone,painted_stone_sampler,
                rock.xy/128.0,rock_dx.xy/128.0,rock_dy.xy/128.0).rgb;
            let stone = stone_x*weights.x+stone_y*weights.y+stone_z*weights.z;
            // Use the stone's relief without transferring its ochre palette
            // into every coast. A restrained earth-brown base joins bare rock.
            let stone_tone = dot(stone,vec3<f32>(0.2126,0.7152,0.0722))/0.2583;
            let mineral = vec3<f32>(0.33,0.26,0.18)*mix(1.0,stone_tone,0.55);
            surface.material.base_color = vec4<f32>(mix(surface.material.base_color.rgb,
                mineral,exposed),surface.material.base_color.a);
        }
    }
    // Reserved source-guarded room finishes and live entry cloth, separate from atlas UVs,
    // live water, fountain threads and the rock material coordinate ranges.
    let room_paint = in.uv.x >= 80000.0 && in.uv.x < 81000.0
        && in.uv.y >= 1000.0 && in.uv.y < 3000.0;
    let room_prop = in.uv.x >= 82000.0 && in.uv.x < 82100.0
        && ((in.uv.y >= 1000.0 && in.uv.y < 3000.0) || source_fabric);
    if room_prop {
        let fabric = in.uv.y >= 2500.0;
        let enamel = in.uv.y >= 2000.0 && !fabric;
        let side_mat = source_fabric && in.uv.y >= 3700.0;
        let fabric_origin = select(3500.0,3700.0,side_mat);
        let origin = select(select(select(1000.0,2000.0,enamel),2500.0,fabric),fabric_origin,source_fabric);
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
            if source_fabric {
                // Broad diagonal twill remains legible when individual
                // threads fall below a pixel. All cells share mat-local
                // coordinates, so neither weave nor binding repeats at seams.
                let diagonal=vec2<f32>(map.x+map.y,map.x-map.y)*1.25;
                let resolved_twill=vec2<f32>(1.0)-smoothstep(vec2<f32>(0.5),
                    vec2<f32>(2.0),fwidth(diagonal));
                let twill=sin(diagonal.x)*sin(diagonal.y)*resolved_twill.x*resolved_twill.y;
                pigment=0.92+twill*0.16-(warp+weft)*0.065;
                let extent=select(vec2<f32>(32.0,16.0),vec2<f32>(16.0,32.0),side_mat);
                let nearest=min(map,extent-map);
                let edge=min(nearest.x,nearest.y);
                let binding=1.0-smoothstep(0.45,0.9,edge);
                pigment*=1.0-binding*0.30;
                let along=select(map.x,map.y,nearest.x<nearest.y);
                let stitch_phase=along*5.0;
                let stitch=pow(0.5+0.5*sin(stitch_phase),5.0)
                    *(1.0-smoothstep(0.5,2.0,fwidth(stitch_phase)));
                let seam=1.0-smoothstep(0.08,0.20,abs(edge-0.38));
                pigment+=stitch*seam*0.22;
            }
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
    // Verified park water carries source coordinates, not atlas UVs. Its flat
    // depth tint stays registered across source cells and camera scrolling.
    if in.uv.x > 40000.0 && in.uv.x < 60000.0 && in.uv.y < -40000.0 {
        let map = in.uv - vec2<f32>(50000.0, -50000.0);
        let optics = pool_optics(map, 1.8, 1.8,
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
    let shoreline = in.uv.x > 87000.0 && in.uv.x < 90000.0 && in.uv.y < -10000.0;
    // Native banks already carry their live atlas palette. Keep fixed-color
    // terrace lips/stairs in the original band with their existing lighting.
    let source_shoreline = shoreline && in.uv.y < -65000.0;
    source_painted = source_painted || source_shoreline;
    if in.uv.x < -9000.0 || shoreline {
        // Mountain material belongs to mountain geometry. A shoreline's
        // source paint must not promote every bank into the canyon finish.
        let canyon = in.uv.y < -30000.0 && !shoreline;
        let volcanic = in.uv.y < -50000.0 && !shoreline;
        let tag = select(select(20000.0, 40000.0, canyon), 60000.0, volcanic);
        let rock = select(vec3<f32>(-in.uv.x - 10000.0, surface.world_position.y, -in.uv.y - tag),
            vec3<f32>((in.uv.x - 88000.0) * 8.0, surface.world_position.y, (-in.uv.y - select(20000.0, 70000.0, source_shoreline)) * 8.0), shoreline);
        let rock_dx = select(vec3<f32>(-uv_dx.x, world_dx.y, -uv_dx.y), vec3<f32>(uv_dx.x * 8.0, world_dx.y, -uv_dx.y * 8.0), shoreline);
        let rock_dy = select(vec3<f32>(-uv_dy.x, world_dy.y, -uv_dy.y), vec3<f32>(uv_dy.x * 8.0, world_dy.y, -uv_dy.y * 8.0), shoreline);
        let mineral = mineral_field(rock * 1.4);
        let weather = mineral_field(rock * 0.19);
        let layer = abs(sin(rock.y * select(1.1, 0.78, canyon) + weather * 1.2));
        let fissure = smoothstep(0.035, 0.14, layer);
        let wall = 1.0 - smoothstep(0.35, 0.8, abs(surface.world_normal.y));
        let detail = (0.94 + mineral * 0.12) * mix(1.0, mix(0.88, 1.0, fissure), wall);
        // Triplanar map-space projection keeps sediment horizontal on both
        // wall directions and avoids stretched cliff caps at camera rotations.
        let weights_raw = pow(abs(surface.world_normal),
            vec3<f32>(select(2.0, 4.0, canyon && !volcanic)));
        let weights = weights_raw / max(dot(weights_raw, vec3<f32>(1.0)), 0.0001);
        let scale = 1.0 / 128.0;
        let x_layer = sample_detail(painted_stone, painted_stone_sampler, rock.zy * scale, rock_dx.zy * scale, rock_dy.zy * scale).rgb;
        let z_layer = sample_detail(painted_stone, painted_stone_sampler, rock.xy * scale, rock_dx.xy * scale, rock_dy.xy * scale).rgb;
        let y_layer = sample_detail(painted_stone, painted_stone_sampler, rock.xz * scale, rock_dx.xz * scale, rock_dy.xz * scale).rgb;
        let x_rock = sample_detail(painted_bedrock, painted_bedrock_sampler, rock.zy / 40.0, rock_dx.zy / 40.0, rock_dy.zy / 40.0).rgb;
        let z_rock = sample_detail(painted_bedrock, painted_bedrock_sampler, rock.xy / 40.0, rock_dx.xy / 40.0, rock_dy.xy / 40.0).rgb;
        // Broad cap plates use the regular cave-rock scale. Sampling this
        // same asset at 40 units packed many tiny painted lumps onto each
        // native tile and concealed the actual large mineral planes.
        let cap_scale = 144.0;
        let y_rock = sample_detail(painted_bedrock, painted_bedrock_sampler, rock.xz / cap_scale, rock_dx.xz / cap_scale, rock_dy.xz / cap_scale).rgb;
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
        let paint_strength = select(select(0.82, 0.94, shoreline), mix(0.50, 0.60, wall), canyon);
        // Pale weathered cap beds sit above the warmer sediment faces, as
        // in the source drawing. Blend over the actual shoulder normal so
        // the lighter top describes its shape without a painted border.
        let canyon_bed = mix(vec3<f32>(0.49, 0.37, 0.20), reference, wall);
        let broad_pigment = select(reference, canyon_bed * (0.96 + weather * 0.08), canyon);
        let pigment = mix(broad_pigment, paint, paint_strength);
        var finished_rock = pigment * surface.material.base_color.rgb / reference * detail;
        if canyon && !volcanic {
            styled_outdoor_rock = true;
            // Keep the original warm ochre cliffs while fractured paint and
            // cooler mineral beds describe their sculpted 3D form. Carry the
            // same material around every face and onto the weathered cap.
            let geology = mineral_field(rock * 0.018);
            let warm_stone = vec3<f32>(0.46, 0.31, 0.14);
            let cool_stone = vec3<f32>(0.28, 0.24, 0.15);
            let stone = mix(cool_stone, warm_stone, smoothstep(0.25, 0.77, geology));
            let granite = x_layer * weights.x + y_layer * weights.y + z_layer * weights.z;
            let rise = smoothstep(0.15, 0.9, surface.world_normal.y);
            let cap_light = mix(vec3<f32>(1.0), vec3<f32>(1.16, 1.13, 1.04), rise);
            let bed = mineral_field(vec3<f32>(rock.x * 0.022,
                rock.y * 0.07 + rock.x * 0.006, rock.z * 0.022));
            let bedding = 0.91 + smoothstep(0.20, 0.76, bed) * 0.14;
            var styled = mix(stone, granite * vec3<f32>(1.20, 0.91, 0.60), 0.68)
                * cap_light * bedding;
            // Broken lichen and moss settle on upward-facing shelves, rather
            // than drawing a bright border along a source tile or camera cut.
            let growth = mineral_field(rock * vec3<f32>(0.045, 0.065, 0.045));
            let moss = rise * smoothstep(0.48, 0.73, growth) * 0.42;
            let lichen = mix(vec3<f32>(0.21, 0.285, 0.105),
                vec3<f32>(0.34, 0.365, 0.18), geology);
            styled = mix(styled, lichen, moss);
            // Keep the recessed tunnel's authored darkness and the broad
            // sculpt's value shading beneath the warm mineral pigment.
            let shade = min(1.0, dot(surface.material.base_color.rgb / reference,
                vec3<f32>(0.2126, 0.7152, 0.0722)) * 1.65);
            finished_rock = styled * shade;
        }
        surface.material.base_color = vec4<f32>(finished_rock,
            surface.material.base_color.a);
    }
    // Cave stone and dust use distinct positive map-coordinate tags.
    // Floors use broader, softer paint; source art and live water stay in the atlas.
    if in.uv.x > 90000.0 {
        let category = round(in.uv.y / 100000.0);
        let textured_cave = category == 18.0 || category == 20.0 || category == 22.0;
        let native_cave = category == 10.0 || category == 12.0 || category == 14.0 || category == 26.0 || category == 28.0 || category == 30.0 || category == 32.0 || category == 34.0 || textured_cave;
        let sliding_ice = category == 8.0;
        let shelf_shoulder = category == 9.0;
        let frozen = category == 3.0 || category == 5.0 || category == 7.0 || sliding_ice || shelf_shoulder;
        let floor_paint = category == 4.0 || category == 5.0 || category == 12.0 || category == 20.0 || category == 30.0 || sliding_ice;
        let tag = category * 100000.0;
        let rock = vec3<f32>(in.uv.x - 100000.0, surface.world_position.y, in.uv.y - tag);
        let rock_dx = vec3<f32>(uv_dx.x, world_dx.y, uv_dx.y);
        let rock_dy = vec3<f32>(uv_dy.x, world_dy.y, uv_dy.y);
        let w = pow(abs(surface.world_normal), vec3<f32>(4.0));
        let weights = w / max(dot(w, vec3<f32>(1.0)), 0.0001);
        // Larger mineral plates support regular cave rock. Frozen surfaces
        // retain their established plate scale and pale crest detail.
        let size = select(select(select(144.0, 96.0, frozen), 160.0, floor_paint), 112.0, sliding_ice);
        let a = sample_detail(painted_bedrock, painted_bedrock_sampler, rock.zy / size, rock_dx.zy / size, rock_dy.zy / size).rgb;
        let b = sample_detail(painted_bedrock, painted_bedrock_sampler, rock.xy / size, rock_dx.xy / size, rock_dy.xy / size).rgb;
        let c = sample_detail(painted_bedrock, painted_bedrock_sampler, rock.xz / size, rock_dx.xz / size, rock_dy.xz / size).rgb;
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
            let cap_sample = sample_detail(painted_bedrock, painted_bedrock_sampler, rock.xz / 160.0, rock_dx.xz / 160.0, rock_dy.xz / 160.0).rgb;
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
            let plate = sample_detail(painted_bedrock, painted_bedrock_sampler,
                rock.xz / 144.0 + vec2<f32>(0.37, 0.61), rock_dx.xz / 144.0, rock_dy.xz / 144.0).rgb;
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
        let water = textureSampleLevel(water_mask, water_mask_sampler, in.uv, 0.0);
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
            let tohjo_water = terrain.water_enabled.x == 4.0;
            let native_cave_water = terrain.water_enabled.x == 3.0 || tohjo_water;
            let source_water = surface.material.base_color.rgb;
            let source_outdoor_water = open_water && abs(terrain.water_deep.w) > 1.5;
            let palette_current = (native_cave_water && terrain.water_deep.w > 0.5)
                || (source_outdoor_water && terrain.water_deep.w > 1.5);
            source_painted = source_painted || source_outdoor_water || (tohjo_water && palette_current);
            let indoor_pool = !native_cave_water && terrain.water_style.w > 0.0 && terrain.water_style.w < 0.2;
            let deep_hint = select(select(vec3<f32>(0.018, 0.095, 0.125),
                vec3<f32>(0.023, 0.12, 0.18), open_water), vec3<f32>(0.018, 0.075, 0.24), indoor_pool);
            let crest_hint = select(select(vec3<f32>(0.24, 0.46, 0.46),
                vec3<f32>(0.48, 0.72, 0.65), open_water), vec3<f32>(0.40, 0.62, 0.90), indoor_pool);
            let shallow_hint = select(select(vec3<f32>(0.065, 0.23, 0.25),
                vec3<f32>(0.12, 0.39, 0.37), open_water), vec3<f32>(0.055, 0.26, 0.56), indoor_pool);
            let woodland_pool = open_water && terrain.water_style.w == 0.20;
            // Source lighting still modulates the water, with a restrained mineral
            // blue-green depth instead of a saturated magnification of atlas blue.
            let source_deep = mix(terrain.water_deep.rgb,
                terrain.water_deep.rgb * 0.20 + shallow_hint * 0.22,
                select(0.0, 0.78, source_outdoor_water));
            let deep = select(select(deep_hint, vec3<f32>(0.018, 0.075, 0.105), woodland_pool),
                source_deep, palette_current);
            let crest = select(select(crest_hint, vec3<f32>(0.24, 0.38, 0.42), woodland_pool),
                terrain.water_foam.rgb, palette_current);
            let pool_tone = select(select(shallow_hint, vec3<f32>(0.065, 0.20, 0.22), woodland_pool),
                mix(deep,crest,select(0.28,0.035,source_outdoor_water)),palette_current);
            let pool_phase = mix(terrain.water_motion.y, terrain.water_motion.x, blend);
            let distances = shore_distances(map);
            let distance = distances.x;
            let depth_distance = distances.y;
            let water_pixel = max(length(world_dx.xz), length(world_dy.xz))
                / max(terrain.water_enabled.y, 0.001);
            var optics: WaterOptics;
            if open_water && terrain.water_deep.w > 2.5 && terrain.water_style.w > 0.25 {
                wind_water = true;
                // Regional source palettes supply pigment and night exposure;
                // their white pixels become reflection/foam, never tiled paint.
                let depth_color = terrain.water_deep.rgb * 0.72;
                let shallow_color = mix(depth_color, terrain.water_foam.rgb, 0.22);
                optics = rippled_pool_optics(map, distance, depth_distance, pool_phase, water_pixel,
                    depth_color, shallow_color, crest, terrain.water_style, surface.V);
            } else if tohjo_water && palette_current {
                optics = rippled_pool_optics(map, distance, depth_distance, pool_phase, water_pixel,
                    deep * 0.88, mix(deep, crest, 0.10), crest, terrain.water_style, surface.V);
            } else if indoor_pool {
                // The authenticated Cerulean pool uses short, low ripples and
                // shallow caustics. Keep its blue pool pigment and quiet indoor
                // reflections; it receives neither coastal wash nor cloud sheen.
                optics = rippled_pool_optics(map, distance, depth_distance, pool_phase, water_pixel,
                    deep_hint, shallow_hint, crest_hint, terrain.water_style, surface.V);
            } else {
                optics = pool_optics(map, distance, depth_distance, deep, pool_tone, crest,
                    terrain.water_style, surface.V);
            }
            var pool = optics.color;
            if terrain.water_contact.w > 0.5 && kind.x > 0.5 {
                // A swimmer disturbs only authenticated water, in map space.
                // Fine uneven rings make contact legible without moving the
                // floor, tinting the actor, or creating a separate decal mesh.
                let relative = map - terrain.water_contact.xy;
                let angle = atan2(relative.y, relative.x);
                let radius = length(relative) + sin(angle * 3.0 + pool_phase) * 0.13
                    + sin(angle * 7.0 - pool_phase * 2.0) * 0.05;
                let body = terrain.water_contact.z;
                let envelope = smoothstep(body * 0.65, body, radius)
                    * (1.0 - smoothstep(body * 1.05, body * 2.0, radius));
                let ring = pow(max(sin(radius * 8.5 - pool_phase * 2.0), 0.0), 18.0);
                let arc = 0.28 + 0.72 * smoothstep(0.16, 0.55,
                    0.5 + 0.5 * sin(angle * 3.0 + radius * 4.0 + pool_phase * 2.0));
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
            let froth = clamp(sheet * 0.29 + streak * breaks * 0.36 + filament * 0.25
                + select(0.0, 0.12, tohjo_water), 0.0, 0.88);
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
            if tohjo_water {
                // Aeration builds as the sheet turns over its rounded lip.
                // Both level ends retain the neighboring basin's exact optics.
                let slope = 1.0 - abs(surface.world_normal.y);
                falling = mix(receiving_pool, falling, smoothstep(0.015, 0.18, slope));
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
            if (native_cave_water || source_outdoor_water) && !palette_current {
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
    var sun_direction = vec3<f32>(0.0, 1.0, 0.0);
    var sunlight_facing = 1.0;
    var water_sun = 0.0;
    var ceramic_sun = 0.0;
    for (var light_id = 0u; light_id < mesh_view_bindings::lights.n_directional_lights; light_id += 1u) {
        let light = mesh_view_bindings::lights.directional_lights[light_id];
        let energy = dot(light.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        if energy > directional_energy {
            sun_tint = light.color.rgb / max(energy, 0.0001);
            sun_direction = light.direction_to_light;
            sunlight_facing = max(dot(surface.world_normal, light.direction_to_light), 0.0);
        }
        directional_energy = max(directional_energy,
            dot(light.color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)));
        if water_reflection.w > 0.5 {
            let half_sum = surface.V + light.direction_to_light;
            let half_vector = half_sum / max(length(half_sum), 0.0001);
            water_sun = max(water_sun, pow(max(dot(water_normal, half_vector), 0.0), 96.0));
        }
@if(!VOXEL_MATERIAL_LIGHTING) {
        if town_ceramic {
            let half_sum = surface.V + light.direction_to_light;
            let half_vector = half_sum / max(length(half_sum), 0.0001);
            // Broad, low-energy dielectric lobe on the authored tile normals.
            // It reveals closed courses without recoloring their source paint.
            ceramic_sun=max(ceramic_sun,
                pow(max(dot(surface.world_normal,half_vector),0.0),24.0)
                * max(dot(surface.world_normal,light.direction_to_light),0.0));
        }
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
    let ambient_luminance = dot(mesh_view_bindings::lights.ambient_color.rgb,
        vec3<f32>(0.2126, 0.7152, 0.0722));
    let ambient_tint = mesh_view_bindings::lights.ambient_color.rgb / max(ambient_luminance, 0.0001);
    let sun_gain = clamp(directional_energy / 3000.0, 0.0, 1.0);
    var indirect_occlusion = vec3<f32>(1.0);
@if(SCREEN_SPACE_AMBIENT_OCCLUSION) {
    // These palette-preserving materials bypass StandardMaterial's Lambert
    // response, so sample the native contact pass explicitly for sky fill.
    let ao_pixel = vec2<i32>(in.position.xy - mesh_view_bindings::view.viewport.xy);
    let contact = textureLoad(mesh_view_bindings::screen_space_ambient_occlusion_texture, ao_pixel, 0i).r;
    indirect_occlusion = vec3<f32>(mix(1.0, clamp(contact, 0.3, 1.0), 0.45));
}
    var shaded = surface.material.base_color.rgb * (indirect_occlusion * ambient_tint * (0.55 * ambient_gain)
        + sun_tint * (0.45 * sun_gain * visibility));
    // Exterior grove daylight preserves painted colors in cast shade.
    // Visibility still comes from the real shadow geometry, so
    // scrolling and orbit cannot move a painted patch of sunlight.
    if terrain.cave_reveal.z > 0.5 {
        // Broad sky fill keeps painted detail readable in shade. A modest
        // hemisphere response gives rounded leaves and raised roof courses
        // volume without applying full Lambert shading over authored paint.
        // Neutral fill preserves the surface pigment. Its temperature
        // follows the scene light instead of repainting warm paths cyan.
        let town_fill = select(1.0, 0.75, terrain.cave_reveal.z > 1.5);
        let sky = mix(vec3<f32>(0.62), vec3<f32>(0.72),
            clamp(surface.world_normal.y * 0.5 + 0.5, 0.0, 1.0)) * town_fill;
        // Keep cast shade on the same color axis as the lit surface. Mixing
        // the cooler ambient tint into dawn's yellow light turned its path
        // shadows gray-green. Geometry changes brightness, not the pigment.
        let fill_tint = sun_tint;
        // Weak dawn/dusk light should not stamp a hard building silhouette
        // over the path. Keep the ambient surface color prominent until the
        // direct light approaches full daytime strength.
        let shadow_contrast = smoothstep(0.55, 0.95, sun_gain);
        let sunlight_visibility = mix(1.0, visibility, shadow_contrast);
        var daylight = mix(sky * indirect_occlusion * fill_tint * ambient_gain,
            sun_tint * (0.96 * sun_gain * (0.72 + 0.28 * sunlight_facing)), sunlight_visibility);
@if(VOXEL_MATERIAL_LIGHTING)
        daylight *= mix(vec3<f32>(1.0), indirect_occlusion, 0.28 * sunlight_visibility);
        // Painted geometry retains its authored pigment, while boughs, leaf
        // folds and rock courses now face the actual sun. This mild response
        // separates their volume without full PBR or another lighting pass.
        let hemisphere = 0.88 + 0.12 * max(surface.world_normal.y, 0.0);
        shaded = surface.material.base_color.rgb * daylight * hemisphere;
    }
    if styled_outdoor_rock {
        // Outdoor mineral is sculpted volume. Let its facets face the real
        // sun instead of retaining the flat fill used for source paintings.
        let hemisphere = mix(vec3<f32>(0.33, 0.355, 0.36),
            vec3<f32>(0.48, 0.49, 0.45),
            clamp(surface.world_normal.y * 0.5 + 0.5, 0.0, 1.0));
        let sky = hemisphere * ambient_tint * ambient_gain * indirect_occlusion;
        let direct = sun_tint * sunlight_facing * sun_gain * visibility * 0.82;
        shaded = surface.material.base_color.rgb * (sky + direct);
    }
    if source_painted {
        // Source paint is independent of the older map-specific daylight
        // selection above. The atlas already supplied morning/day/night
        // paint; a second moon tint turns white source fences electric blue.
        // Retain real face, contact and cast-shadow modulation in every map
        // whose exact authored material roles select live source paint.
        // The town flag changes cast shade only, never source pigment or the
        // forest fill. Low-energy dawn/night still suppress cast contrast.
        let town_fill = select(1.0, 0.75, terrain.cave_reveal.z > 1.5);
        let source_sky = mix(vec3<f32>(0.62), vec3<f32>(0.72),
            clamp(surface.world_normal.y * 0.5 + 0.5, 0.0, 1.0)) * town_fill;
        let source_shadow_contrast = smoothstep(0.55, 0.95, sun_gain);
        let source_visibility = mix(1.0, visibility, source_shadow_contrast);
        let town_facing = mix(1.0, 0.65 + 0.35 * sunlight_facing,
            select(0.0, source_shadow_contrast, town_source_paint || town_timber));
        var source_light = mix(source_sky * indirect_occlusion,
            vec3<f32>(0.96 * town_facing), source_visibility);
@if(VOXEL_MATERIAL_LIGHTING)
        source_light *= mix(vec3<f32>(1.0), indirect_occlusion, 0.28 * source_visibility);
        let source_hemisphere = 0.94 + 0.06 * max(surface.world_normal.y, 0.0);
        shaded = surface.material.base_color.rgb * source_light * source_hemisphere;
@if(!VOXEL_MATERIAL_LIGHTING) {
        if town_ceramic {
            let facing=clamp(abs(dot(surface.world_normal,surface.V)),0.0,1.0);
            let fresnel=0.04+0.96*pow(1.0-facing,5.0);
            let reflected=reflect(-surface.V,surface.world_normal);
            let sky=smoothstep(-0.15,0.65,reflected.y);
            // Neutral sky/sun reflection is separate from the live pigment.
            // Suppress it with weak light so night's source black stays black.
            let sheen=(0.12*fresnel*sky*indirect_occlusion
                +vec3<f32>(0.045*ceramic_sun*visibility))*source_shadow_contrast;
            shaded+=sheen;
        }
}
        if town_timber {shaded*=mix(1.0,timber_pigment,source_shadow_contrast);}
        if town_glass {
            // Closed authored panes retain opaque source paint. A shallow
            // edge recess and bounded sky sheen supply glass depth without
            // repainting the live day/night hue or adding a reflection pass.
            let edge=min(min(pane.x,1.0-pane.x),min(pane.y,1.0-pane.y));
            let recess=mix(0.88,1.0,smoothstep(0.0,0.09,edge));
            let facing=abs(dot(surface.world_normal,surface.V));
            let fresnel=pow(1.0-clamp(facing,0.0,1.0),5.0);
            let reflected=reflect(-surface.V,surface.world_normal);
            let sky=clamp(reflected.y*0.5+0.5,0.0,1.0);
            let sheen=(0.04+0.16*fresnel)*sky;
            // Source clerestories need a visible recess and sky gradient at
            // normal game distance, while keeping their live palette hue.
            let pane_light=select(recess+0.10*(pane.y-0.5)+sheen,
                recess*(0.56+0.34*pane.y)+sheen*1.4,source_glass);
            shaded*=mix(1.0,pane_light,source_shadow_contrast);
            if source_glass {
                // A broad reflected sky patch distinguishes closed glass
                // from painted infill. It stays subordinate to the native
                // pigment and disappears with daylight, leaving lamp policy
                // and the authored night palette authoritative.
                let diagonal=pane.y+pane.x*0.35;
                let sky_patch=smoothstep(0.22,0.42,diagonal)
                    *(1.0-smoothstep(0.65,0.88,diagonal));
                // The raised game camera often reflects the lower hemisphere
                // in vertical panes. Keep a subdued ground reflection there
                // rather than losing the entire glass finish below the sky.
                let reflection=mix(vec3<f32>(0.16,0.20,0.21),
                    vec3<f32>(0.48,0.66,0.78),sky)*source_light;
                shaded=mix(shaded,reflection,
                    (0.14+0.32*sky_patch)*source_shadow_contrast);
            }
        }
    }
    shaded *= woodland_floor_light;
    if water_reflection.w > 0.5 {
        // Camera-dependent sky sheen and the actual directional light reflect
        // off the level water surface. Real scene shadows still suppress direct sun;
        // the normal scene energies also dim these effects at night/in caves.
        var sky = mix(vec3<f32>(0.15, 0.30, 0.36), vec3<f32>(0.46, 0.64, 0.68),
            clamp(water_normal.y * 0.5 + 0.5, 0.0, 1.0));
        var sky_response = water_reflection.x * 0.65 + water_reflection.y * 0.045;
        if wind_water {
            let reflected = reflect(-surface.V, water_normal);
            let sky_height = smoothstep(0.0, 0.85, reflected.y);
            // Broad, low-contrast cloud reflections bend with the wave normal.
            // A sharp high-frequency cloud field stamps a dark checker onto
            // the short wave groups. Keep cloud volume at a wider sky scale.
            // Scene light supplies exposure/tint; Bevy's environment
            // map below supplies the current sky's specular radiance as well.
            let cloud_field = mineral_field(reflected * vec3<f32>(1.4, 2.2, 1.4));
            let cloud = smoothstep(0.25, 0.85, cloud_field);
            sky = mix(vec3<f32>(0.42, 0.64, 0.78), vec3<f32>(0.16, 0.38, 0.68), sky_height);
            sky = mix(sky, vec3<f32>(0.80, 0.88, 0.94), cloud * 0.22);
            sky_response = 0.08 + water_reflection.x * 1.1 + water_reflection.y * 0.20;
        }
        let sky_weight = clamp(sky_response
            * water_reflection.z, 0.0, 0.35);
        shaded = mix(shaded, sky * ambient_tint * ambient_gain, sky_weight);
        shaded = shaded + sun_tint * water_sun * sun_gain * visibility
            * select(0.035 + water_reflection.x * 0.16,
                0.65 + water_reflection.x * 1.5, wind_water) * water_reflection.z;
    }
@if(VOXEL_MATERIAL_LIGHTING) {
    // Authored roles control the BRDF without recoloring source paint or
    // applying a second diffuse sun response over baked face shading.
    // x = perceptual roughness, y = normal-incidence reflectance, z = strength.
    var response = vec3<f32>(surface.material.perceptual_roughness,
        dot(surface.material.reflectance, vec3<f32>(0.333333)) * 0.08, 0.0);
    if source_paving { response = vec3<f32>(0.94, 0.025, 0.30); }
    if source_fabric { response = vec3<f32>(0.98, 0.02, 0.15); }
    if source_concrete { response = vec3<f32>(0.97, 0.02, 0.30); }
    if source_mineral { response = vec3<f32>(0.96, 0.025, 0.30); }
    if town_paint_kind == 0.0 { response = vec3<f32>(0.40, 0.04, 0.85); } // ceramic
    if town_paint_kind == 1.0 { response = vec3<f32>(0.94, 0.025, 0.35); } // plaster
    if town_paint_kind == 2.0 { response = vec3<f32>(0.82, 0.035, 0.50); } // clay
    if town_paint_kind == 3.0 { response = vec3<f32>(0.97, 0.025, 0.30); } // mortar
    if town_paint_kind == 4.0 { response = vec3<f32>(0.92, 0.025, 0.35); } // mineral slate
    if town_timber { response = vec3<f32>(0.72, 0.035, 0.55); }
    if enclosing_wall { response = vec3<f32>(0.90, 0.035, 0.35); }
    if frozen_platform { response = vec3<f32>(0.28, 0.025, 0.70); }
    if town_glass || city_source_window { response = vec3<f32>(0.16, 0.04, 0.90); }
    var N = surface.N;
    if water_reflection.w > 0.5 {
        response = vec3<f32>(0.22, 0.02, water_reflection.z);
        N = water_normal;
    }
    let metallic = clamp(surface.material.metallic, 0.0, 1.0);
    if metallic > 0.0 { response.z = max(response.z, metallic); }
    if response.z > 0.0 {
        let perceptual_roughness = clamp(response.x, 0.12, 1.0);
        let ndotv = clamp(dot(N, surface.V), 0.001, 1.0);
        let F0 = mix(vec3<f32>(response.y), surface.material.base_color.rgb, metallic);
        let fresnel = F0 + (vec3<f32>(1.0) - F0) * pow(1.0 - ndotv, 5.0);
        let half_sum = surface.V + sun_direction;
        let H = half_sum / max(length(half_sum), 0.0001);
        // Normalised bounded specular lobe; broad matte materials cannot turn
        // into shiny plastic, and small highlights have a finite energy budget.
        let exponent = min(2.0 / pow(perceptual_roughness, 4.0) - 2.0, 512.0);
        let lobe = pow(max(dot(N, H), 0.0), exponent) * (exponent + 2.0) / 8.0;
        let direct_specular = min(lobe, 8.0) * max(dot(N, sun_direction), 0.0)
            * sun_gain * visibility;
        shaded += F0 * sun_tint * direct_specular * response.z * 0.22;
@if(ENVIRONMENT_MAP) {
        var layer: LayerLightingInput;
        layer.N = N;
        layer.R = reflect(-surface.V, N);
        layer.NdotV = ndotv;
        layer.perceptual_roughness = perceptual_roughness;
        layer.roughness = perceptual_roughness * perceptual_roughness;
        var ranges: ClusterableObjectIndexRanges;
        let environment = compute_radiances(layer, &ranges, surface.world_position.xyz, false);
        // Source palettes already carry time-of-day temperature. Keep their
        // diffuse sky fill neutral while reflections carry the actual sky hue.
        var irradiance = environment.irradiance;
        if source_painted { irradiance = vec3<f32>(dot(irradiance, vec3<f32>(0.2126,0.7152,0.0722))); }
        shaded += surface.material.base_color.rgb * irradiance * indirect_occlusion
            * (1.0 - metallic) * 0.09 * response.z;
        shaded += environment.radiance * fresnel * response.z * indirect_occlusion;
}
    }
}
    // Outdoor mouths admit soft sky light along the approach. Descending
    // shafts remain occluded; neither ladders nor pits become light sources.
    var exit_fill = 0.0;
    for (var i = 0u; i < 4u; i += 1u) {
        let exit = terrain.cave_exits[i];
        if exit.w <= 0.0 { continue; }
        let delta = surface.world_position.xyz - exit.xyz;
        let approach = smoothstep(-2.0, 4.0, delta.z);
        let reach = max(delta.z - exit.w * 0.225, 0.0);
        let depth = 1.0 - smoothstep(exit.w * 0.35, exit.w, reach);
        let half_width = exit.w * 0.14 + reach * 0.60;
        let width = 1.0 - smoothstep(half_width * 0.60, half_width, abs(delta.x));
        let height = 1.0 - smoothstep(exit.w * 0.20, exit.w * 0.62, abs(delta.y));
        // Sky enters above the lip and slopes down onto lower walking ground.
        // Deep recessed shafts still fall below that illuminated envelope.
        let above_lip = smoothstep(-1.5, 0.0, delta.y + reach * 0.45);
        let facing = 0.35 + 0.65 * max(dot(surface.world_normal,
            normalize(vec3<f32>(-delta.x * 0.08, 0.65, -1.0))), 0.0);
        exit_fill = max(exit_fill, approach * depth * width * height * above_lip * facing);
    }
    shaded += surface.material.base_color.rgb * terrain.cave_sky.rgb * (1.30 * terrain.cave_sky.w * exit_fill);
    // The distant outdoor opening receives sky exposure. Its hue remains
    // the authenticated 2D mineral highlight; the shaded tunnel lining and
    // floor spill establish depth without a white lamp or bloom.
@if(VERTEX_UVS_A) {
    if in.uv.x > 90000.0 && round(in.uv.y / 100000.0) == 28.0 {
        shaded += surface.material.base_color.rgb * 0.50;
    }
}
    if terrain.city_night.x > 0.0 {
        // Integrate the actual finite luminous pane over the receiving
        // hemisphere. Area emitters have a bounded near field and a broad,
        // naturally soft falloff, rather than point or spotlight hotspots.
        let identity=mat3x3<f32>(vec3<f32>(1.0,0.0,0.0),vec3<f32>(0.0,1.0,0.0),vec3<f32>(0.0,0.0,1.0));
        var window_light=0.0;
        let window_count = min(u32(terrain.city_night.y), 32u);
        for (var i=0u; i<window_count; i+=1u) {
            let window=terrain.city_windows[i];
            if window.w<=0.0 {continue;}
            let plane=terrain.city_window_planes[i];
            let delta=surface.world_position.xyz-window.xyz;
            let reach=96.0+2.0*max(window.w,plane.w);
            let distance=length(delta);
            if dot(delta,plane.xyz)<=0.0 || distance>=reach {continue;}
            let right=cross(plane.xyz,vec3<f32>(0.0,1.0,0.0))*window.w;
            let up=vec3<f32>(0.0,plane.w,0.0);
            let corners=array<vec3<f32>,4>(window.xyz+right-up,window.xyz-right-up,
                window.xyz-right+up,window.xyz+right+up);
            let irradiance=max(ltc_integrate_quad(surface.world_normal,surface.V,
                surface.world_position.xyz,identity,corners),0.0);
            window_light+=irradiance*(1.0-smoothstep(reach*0.45,reach,distance));
        }
        shaded+=surface.material.base_color.rgb*vec3<f32>(1.0,0.72,0.36)
            *(window_light/(1.0+6.0*window_light))*(6.0*terrain.city_night.x);
@if(VERTEX_UVS_A) {
        let city_role = in.uv.y - 1248.0;
        if (city_role >= 0.0 && city_role <= 3.0) || city_source_window || luminous_glass {
            // Emission is independent of moonlight, cast shade and camera.
            // Keep a shallow edge/face response so panes retain their volume.
            var glow = vec3<f32>(1.0, 0.58, 0.19);
            if city_role > 0.5 && city_role <= 3.0 { glow = max(surface.material.base_color.rgb * 2.4,
                vec3<f32>(0.18, 0.10, 0.035)); }
            if city_role > 2.5 && city_role <= 3.0 { glow = vec3<f32>(1.0, 0.055, 0.018); }
@if(VOXEL_MATERIAL_LIGHTING && VOXEL_HDR) {
            // Actual light energy above one allows the HDR camera to bloom
            // luminous panes without scattering daylight plaster or the UI.
            glow *= 2.2;
}
            let pane_detail = clamp(dot(surface.material.base_color.rgb,
                vec3<f32>(0.25, 0.60, 0.15)), 0.0, 1.0);
            let glass_depth = 0.78 + 0.18 * pane_detail + 0.04 * abs(surface.world_normal.z);
            shaded = mix(shaded, glow * glass_depth,
                terrain.city_night.x);
        }
}
    }
    let darkened = mix(shaded, vec3<f32>(0.0), clamp(terrain.cutaway.fade.y, 0.0, 1.0));
    let palette_color = mix(darkened, vec3<f32>(1.0), clamp(terrain.cutaway.fade.z, 0.0, 1.0));
    var out: FragmentOutput;
    let opacity = select(surface.material.base_color.a * (1.0 - terrain.cutaway.fade.x),
        1.0, masked_cutaway);
    out.color = vec4<f32>(palette_color, opacity);
    out.color = main_pass_post_lighting_processing(surface, out.color);
    // The classic dark map palette conceals terrain until Flash, while
    // actor materials (reveal.x == 0) retain their visible source colors.
    // Apply after sky fill and post-processing so neither can reveal a map.
    if terrain.cave_reveal.x > 0.5 && terrain.cave_reveal.y > 0.5 {
        out.color = vec4<f32>(0.0, 0.0, 0.0, out.color.a);
    }
    return out;
}
}
