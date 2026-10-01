#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, main_pass_post_lighting_processing},
    mesh_view_bindings,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    shadows::fetch_directional_shadow,
}

// Marked indoor walls share the ordinary opaque material/depth pass. Only
// fragments between the camera and the player's projected capsule disappear.
// UV1 is a flat per-triangle eligibility mask; UV0 and model geometry are intact.
struct CutawayUniform {
    bottom_radius: vec4<f32>,
    top_feather: vec4<f32>,
}
@group(2) @binding(100) var<uniform> cutaway: CutawayUniform;

fn player_reveal(view_position: vec3<f32>) -> f32 {
    let radius = cutaway.bottom_radius.w;
    if radius <= 0.0 { return 0.0; }
    let bottom = (mesh_view_bindings::view.view_from_world * vec4<f32>(cutaway.bottom_radius.xyz, 1.0)).xyz;
    let top = (mesh_view_bindings::view.view_from_world * vec4<f32>(cutaway.top_feather.xyz, 1.0)).xyz;
    if bottom.z >= -0.001 || top.z >= -0.001 || view_position.z >= -0.001 { return 0.0; }
    let a = bottom.xy / -bottom.z;
    let b = top.xy / -top.z;
    let p = view_position.xy / -view_position.z;
    let axis = b - a;
    let t = clamp(dot(p - a, axis) / max(dot(axis, axis), 0.000001), 0.0, 1.0);
    // Perspective-correct depth at the closest screen point on the capsule.
    let depth = 1.0 / mix(1.0 / -bottom.z, 1.0 / -top.z, t);
    if -view_position.z >= depth - radius * 0.01 { return 0.0; }
    let distance = length(p - mix(a, b, t)) * depth;
    return 1.0 - smoothstep(radius, radius + cutaway.top_feather.w, distance);
}

fn cutaway_dither(pixel: vec2<f32>) -> f32 {
    // Fixed screen-space Bayer pattern: no animated noise or transparency sort.
    var bayer = array<f32, 16>(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0,
        3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    let xy = vec2<u32>(pixel) % vec2<u32>(4u);
    return (bayer[xy.y * 4u + xy.x] + 0.5) / 16.0;
}

// The palette and baked face shade are the surface color. The sun pass
// contributes visibility only, without applying another PBR light response.
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var surface = pbr_input_from_standard_material(in, is_front);
    surface.material.base_color = alpha_discard(surface.material, surface.material.base_color);
    let view_position = mesh_view_bindings::view.view_from_world * surface.world_position;
#ifdef VERTEX_UVS_B
    if in.uv_b.x > 0.5 && player_reveal(view_position.xyz) > cutaway_dither(in.position.xy) {
        discard;
    }
#endif
    var visibility = 1.0;
    for (var light_id = 0u; light_id < mesh_view_bindings::lights.n_directional_lights; light_id += 1u) {
        let light = mesh_view_bindings::lights.directional_lights[light_id];
        if light.skip == 0u && (light.flags & DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            visibility = min(visibility, fetch_directional_shadow(light_id,
                surface.world_position, surface.world_normal, view_position.z));
        }
    }
    var out: FragmentOutput;
    out.color = vec4<f32>(surface.material.base_color.rgb * mix(0.55, 1.0, visibility),
        surface.material.base_color.a);
    out.color = main_pass_post_lighting_processing(surface, out.color);
    return out;
}
