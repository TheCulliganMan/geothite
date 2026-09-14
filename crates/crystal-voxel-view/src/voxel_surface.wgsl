#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, main_pass_post_lighting_processing},
    mesh_view_bindings,
    mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT,
    shadows::fetch_directional_shadow,
}

// The palette and baked face shade are the surface color. The sun pass
// contributes visibility only, without applying another PBR light response.
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var surface = pbr_input_from_standard_material(in, is_front);
    surface.material.base_color = alpha_discard(surface.material, surface.material.base_color);
    let view_position = mesh_view_bindings::view.view_from_world * surface.world_position;
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
