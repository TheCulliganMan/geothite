import bevy_pbr::render::{forward_io::VertexOutput};

struct SilhouetteMaterial {
    color: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var silhouette_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var silhouette_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var<uniform> material: SilhouetteMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let source = textureSample(silhouette_texture, silhouette_sampler, in.uv);
    if source.a < 0.5 {
        discard;
    }
    return vec4<f32>(source.rgb * material.color.rgb, source.a * material.color.a);
}
