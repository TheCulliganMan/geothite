#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
@group(2) @binding(100) var<uniform> cue: vec4<f32>;

// Apply the authoritative background palette cue after sampling the albedo.
// Whitening vertex colors alone leaves a colored texture unchanged.
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var surface = pbr_input_from_standard_material(in, is_front);
    surface.material.base_color = alpha_discard(surface.material, surface.material.base_color);
    var out: FragmentOutput;
    if (surface.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(surface);
    } else {
        out.color = surface.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(surface, out.color);
    out.color = vec4<f32>(mix(mix(out.color.rgb, vec3<f32>(0.0), cue.x), vec3<f32>(1.0), cue.y), out.color.a);
    return out;
}
