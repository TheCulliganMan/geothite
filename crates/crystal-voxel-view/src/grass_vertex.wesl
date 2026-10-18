// Preserve the standard Bevy vertex paths, then bend only rooted native grass.
import bevy_pbr::render::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::{morph_position, morph_normal, morph_tangent},
    view_transformations::position_world_to_clip,
};

// Camera/sky globals occupy binding 11 in forward views, but binding 1
// in depth/shadow prepass views. Wind must use the same clock in every pass.
@if(PREPASS_PIPELINE)
import bevy_render::globals::Globals;
@if(PREPASS_PIPELINE)
import bevy_pbr::prepass::io::{Vertex, UncompressedVertex, VertexOutput, decompress_vertex};
@else
import bevy_pbr::render::forward_io::{Vertex, UncompressedVertex, VertexOutput, decompress_vertex};

@if(PREPASS_PIPELINE)
@group(0) @binding(1) var<uniform> prepass_globals: Globals;

struct GrassUniform {
    contact: vec4<f32>, // world X/Z, radius, signed spring amplitude
    height_scale: vec4<f32>,
}
@group(constants::MATERIAL_BIND_GROUP) @binding(111) var<uniform> grass: GrassUniform;

fn grass_position(world: vec4<f32>, uv: vec2<f32>, world_scale: f32) -> vec4<f32> {
    var point = world;
    if abs(uv.x - 84000.0) < 0.5 {
        let scale = select(1.0, grass.height_scale.x, grass.height_scale.x > 0.0);
        let height = max(uv.y, 0.0);
        // UV.y is height above the blade root in mesh space. Floors and roots
        // retain their exact elevation; forward, depth and shadow passes agree.
        point.y -= height * world_scale * (1.0 - scale);
        if grass.contact.z > 0.0 {
            let delta = point.xz - grass.contact.xy;
            let distance = length(delta);
            let influence = 1.0 - smoothstep(0.0, grass.contact.z, distance);
            let direction = delta / max(distance, 0.25);
            let bend = height * scale * 0.22 * influence * grass.contact.w;
            point.x += direction.x * bend;
            point.z += direction.y * bend;
            point.y -= abs(bend) * 0.18;
        }
    }
    if abs(uv.x - 86000.0) < 0.5 || abs(uv.x - 86200.0) < 0.5 {
        let t = grass.height_scale.y;
        let phase = world.x * 0.045 + world.z * 0.031;
        let gust = pow(max(0.0, sin(t * 0.29)), 4.0);
        let bend = max(uv.y, 0.0) * world_scale
            * (0.055 * sin(t * 1.7 + phase) + 0.035 * gust);
        point.x += bend;
        point.z += bend * 0.45;
    }
    return point;
}

@if(MORPH_TARGETS)
/// The instance_index parameter must match vertex_in.instance_index. This is a work around for a wgpu dx12 bug.
/// See https://github.com/gfx-rs/naga/issues/2416
fn morph_vertex(vertex_in: UncompressedVertex, instance_index: u32) -> UncompressedVertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;
    let morph_descriptor_index = mesh[instance_index].morph_descriptor_index;

    let weight_count = bevy_pbr::render::morph::layer_count(morph_descriptor_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::render::morph::weight_at(i, morph_descriptor_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, morph_descriptor_index);
        @if(VERTEX_NORMALS)
        vertex.normal += weight * morph_normal(vertex_index, i, morph_descriptor_index);
        @if(VERTEX_TANGENTS)
        vertex.tangent += vec4(weight * morph_tangent(vertex_index, i, morph_descriptor_index), 0.0);
    }
    return vertex;
}

@if(MORPH_TARGETS)
/// Returns the morphed position of the given vertex from the previous frame.
///
/// This function is used for motion vector calculation, and, as such, it doesn't
/// bother morphing the normals and tangents.
///
/// The instance_index parameter must match vertex_in.instance_index. This is a work around for a wgpu dx12 bug.
/// See https://github.com/gfx-rs/naga/issues/2416
fn morph_prev_vertex(vertex_in: UncompressedVertex, instance_index: u32) -> UncompressedVertex {
    var vertex = vertex_in;
    let first_vertex = mesh[instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;
    let morph_descriptor_index  = mesh[instance_index].morph_descriptor_index;
    let weight_count = bevy_pbr::render::morph::layer_count(morph_descriptor_index);
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::render::morph::prev_weight_at(i, morph_descriptor_index);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph_position(vertex_index, i, morph_descriptor_index);
        // Don't bother morphing normals and tangents; we don't need them for
        // motion vector calculation.
    }
    return vertex;
}

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let uncompressed_vertex_no_morph = decompress_vertex(vertex_no_morph, vertex_no_morph.instance_index);
@if(MORPH_TARGETS)
    var vertex = morph_vertex(uncompressed_vertex_no_morph, vertex_no_morph.instance_index);
@else
    var vertex = uncompressed_vertex_no_morph;

    let mesh_world_from_local = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);

@if(SKINNED)
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = skinning::skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex_no_morph.instance_index
    );
@else
    var world_from_local = mesh_world_from_local;

@if(VERTEX_NORMALS && SKINNED && (!PREPASS_PIPELINE || NORMAL_PREPASS_OR_DEFERRED_PREPASS))
    out.world_normal = skinning::skin_normals(world_from_local, vertex.normal);
@if(VERTEX_NORMALS && !SKINNED && (!PREPASS_PIPELINE || NORMAL_PREPASS_OR_DEFERRED_PREPASS))
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );

@if(VERTEX_POSITIONS)
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
@if(VERTEX_POSITIONS && VERTEX_UVS_A) {
    out.world_position = grass_position(out.world_position, vertex.uv, length(world_from_local[1].xyz));
}
@if(VERTEX_POSITIONS)
    out.position = position_world_to_clip(out.world_position.xyz);
@if(PREPASS_PIPELINE && UNCLIPPED_DEPTH_ORTHO_EMULATION) {
    out.unclipped_depth = out.position.z;
    out.position.z = min(out.position.z, 1.0);
}

@if(VERTEX_UVS_A)
    out.uv = vertex.uv;
@if(VERTEX_UVS_B)
    out.uv_b = vertex.uv_b;

@if(VERTEX_TANGENTS && (!PREPASS_PIPELINE || NORMAL_PREPASS_OR_DEFERRED_PREPASS))
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );

@if(VERTEX_COLORS)
    out.color = vertex.color;

@if(PREPASS_PIPELINE && MOTION_VECTOR_PREPASS) {

    // Take morph targets into account.
@if(MORPH_TARGETS && HAS_PREVIOUS_MORPH)
    let prev_vertex = morph_prev_vertex(uncompressed_vertex_no_morph, vertex_no_morph.instance_index);
@elif(MORPH_TARGETS)
    let prev_vertex = uncompressed_vertex_no_morph;
@else   // MORPH_TARGETS
    let prev_vertex = uncompressed_vertex_no_morph;

    // Take skinning into account.
@if(SKINNED && HAS_PREVIOUS_SKIN)
    let prev_model = skinning::skin_prev_model(
        prev_vertex.joint_indices,
        prev_vertex.joint_weights,
        vertex_no_morph.instance_index
    );
@elif(SKINNED)
    // Use vertex_no_morph.instance_index instead of prev_vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    let prev_model = mesh_functions::get_previous_world_from_local(vertex_no_morph.instance_index);
@else   // SKINNED
    // Use vertex_no_morph.instance_index instead of prev_vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    let prev_model = mesh_functions::get_previous_world_from_local(vertex_no_morph.instance_index);

    out.previous_world_position = mesh_functions::mesh_position_local_to_world(
        prev_model,
        vec4<f32>(prev_vertex.position, 1.0)
    );
@if(VERTEX_UVS_A) {
    out.previous_world_position = grass_position(out.previous_world_position,
        prev_vertex.uv, length(prev_model[1].xyz));
}
}

@if(VERTEX_OUTPUT_INSTANCE_INDEX)
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;

@if(VISIBILITY_RANGE_DITHER)
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex_no_morph.instance_index, mesh_world_from_local[3]);

    return out;
}
