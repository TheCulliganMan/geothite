//! Native shadow QA: use Bevy's current joint bounds, never a static skin AABB.
use bevy::{
    asset::AssetEventSystems,
    camera::{
        primitives::Aabb,
        visibility::{DynamicSkinnedMeshBounds, VisibilitySystems},
    },
    diagnostic::FrameCount,
    ecs::message::MessageReader,
    mesh::{
        VertexAttributeValues,
        morph::MeshMorphWeights,
        skinning::{
            SkinnedMesh, SkinnedMeshInverseBindposes, entity_aabb_from_skinned_mesh_bounds,
        },
    },
    prelude::*,
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Metadata {
    max_joint: usize,
    weight_error: f32,
    generated: bool,
}
#[derive(Resource, Default)]
struct Cache(HashMap<AssetId<Mesh>, Option<Metadata>>);
#[derive(Component, Clone, Copy)]
struct Prepared {
    mesh: AssetId<Mesh>,
    metadata: Metadata,
    original: Aabb,
    owns_dynamic: bool,
}
#[derive(Component, Clone, Copy)]
pub(super) struct FreshBounds {
    frame: u32,
    mesh: AssetId<Mesh>,
    metadata: Metadata,
}

pub(super) fn register<M: bevy::pbr::Material>(app: &mut App) {
    if std::env::var("CRYSTAL_SHADOW_ORTHO_CLAMP").as_deref() != Ok("on") {
        return;
    }
    app.init_resource::<Cache>().add_systems(
        PostUpdate,
        (
            prepare::<M>
                .after(AssetEventSystems)
                .before(VisibilitySystems::CalculateBounds),
            certify
                .after(VisibilitySystems::CalculateBounds)
                .before(VisibilitySystems::CheckVisibility),
        ),
    );
}

fn metadata(mesh: &Mesh) -> Option<Metadata> {
    let VertexAttributeValues::Float32x3(positions) =
        mesh.try_attribute_option(Mesh::ATTRIBUTE_POSITION).ok()??
    else {
        return None;
    };
    let VertexAttributeValues::Uint16x4(indices) = mesh
        .try_attribute_option(Mesh::ATTRIBUTE_JOINT_INDEX)
        .ok()??
    else {
        return None;
    };
    let VertexAttributeValues::Float32x4(weights) = mesh
        .try_attribute_option(Mesh::ATTRIBUTE_JOINT_WEIGHT)
        .ok()??
    else {
        return None;
    };
    if positions.is_empty() || positions.len() != indices.len() || positions.len() != weights.len()
    {
        return None;
    }
    if let Some(uvs) = mesh.try_attribute_option(Mesh::ATTRIBUTE_UV_0).ok()? {
        match uvs {
            VertexAttributeValues::Float32x2(uvs)
                if uvs
                    .iter()
                    .all(|uv| uv[0].is_finite() && (uv[0] - 84000.0).abs() >= 0.5) => {}
            _ => return None,
        }
    }
    let mut error = 0.0f64;
    for (position, weights) in positions.iter().zip(weights) {
        if !position.iter().all(|x| x.is_finite())
            || !weights.iter().all(|&x| x.is_finite() && x >= 0.)
        {
            return None;
        }
        let deviation = (weights.iter().map(|&x| f64::from(x)).sum::<f64>() - 1.0).abs();
        if deviation > 1e-5 {
            return None;
        }
        error = error.max(deviation);
    }
    Some(Metadata {
        max_joint: indices.iter().flatten().copied().max()? as usize,
        weight_error: error as f32,
        generated: false,
    })
}
fn affine_finite(matrix: Mat4) -> bool {
    matrix.is_finite()
        && matrix.x_axis.w == 0.
        && matrix.y_axis.w == 0.
        && matrix.z_axis.w == 0.
        && matrix.w_axis.w == 1.
}
fn restore(commands: &mut Commands, entity: Entity, previous: Option<&Prepared>) {
    if let Some(previous) = previous {
        let mut entity = commands.entity(entity);
        entity
            .insert(previous.original)
            .remove::<(Prepared, FreshBounds)>();
        if previous.owns_dynamic {
            entity.remove::<DynamicSkinnedMeshBounds>();
        }
    }
}
fn prepare<M: bevy::pbr::Material>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    binds: Res<Assets<SkinnedMeshInverseBindposes>>,
    mut events: MessageReader<AssetEvent<Mesh>>,
    mut cache: ResMut<Cache>,
    actors: Query<(
        Entity,
        &Mesh3d,
        &SkinnedMesh,
        Has<MeshMorphWeights>,
        Option<&Prepared>,
        Option<&Aabb>,
        Has<DynamicSkinnedMeshBounds>,
        Option<&MeshMaterial3d<M>>,
        Has<MeshMaterial3d<StandardMaterial>>,
    )>,
    mut last_report: Local<Option<(usize, usize, usize)>>,
) {
    // This reads real current-frame events after AssetEventSystems. Updating
    // CPU-only bounds metadata below is deliberately untracked: no GPU vertex
    // buffer changed, and self-generated Modified events would loop forever.
    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                cache.0.remove(id);
            }
            _ => {}
        }
    }
    for (entity, handle, skin, morph, previous, original, dynamic, voxel, standard) in &actors {
        if morph || (voxel.is_none() && !standard) || original.is_none() {
            restore(&mut commands, entity, previous);
            continue;
        }
        let Some(mesh) = meshes.get(&handle.0) else {
            restore(&mut commands, entity, previous);
            continue;
        };
        let entry = cache.0.entry(handle.id()).or_insert_with(|| metadata(mesh));
        let Some(meta) = entry else {
            restore(&mut commands, entity, previous);
            continue;
        };
        let valid = !morph
            && binds.get(&skin.inverse_bindposes).is_some_and(|b| {
                b.len() > meta.max_joint
                    && b.iter().take(meta.max_joint + 1).all(|&m| affine_finite(m))
            })
            && skin.joints.len() > meta.max_joint;
        if !valid {
            restore(&mut commands, entity, previous);
            continue;
        }
        if !meta.generated {
            let Some(mesh) = meshes.get_mut_untracked(handle.id()) else {
                continue;
            };
            if mesh.generate_skinned_mesh_bounds().is_err() {
                *entry = None;
                restore(&mut commands, entity, previous);
                continue;
            }
            meta.generated = true;
        }
        commands.entity(entity).insert((
            DynamicSkinnedMeshBounds,
            Prepared {
                mesh: handle.id(),
                metadata: *meta,
                original: previous
                    .filter(|p| p.mesh == handle.id())
                    .map_or_else(|| *original.unwrap(), |p| p.original),
                owns_dynamic: previous.map_or(!dynamic, |p| p.owns_dynamic),
            },
        ));
    }
    if std::env::var("CRYSTAL_GPU_CULLING_TRACE").as_deref() == Ok("on") {
        let stats = (
            cache.0.len(),
            cache.0.values().filter(|v| v.is_some()).count(),
            cache
                .0
                .values()
                .filter(|v| v.is_some_and(|m| m.generated))
                .count(),
        );
        if *last_report != Some(stats) {
            eprintln!(
                "shadow_hzb_skin_metadata revisions={} validated={} generated={}",
                stats.0, stats.1, stats.2
            );
            *last_report = Some(stats);
        }
    }
}
fn current_bound(
    mesh: &Mesh,
    skin: &SkinnedMesh,
    binds: &SkinnedMeshInverseBindposes,
    transform: &GlobalTransform,
    joints: &Query<&GlobalTransform>,
    meta: Metadata,
) -> Option<Aabb> {
    if !meta.generated
        || skin.joints.len() <= meta.max_joint
        || binds.len() <= meta.max_joint
        || !transform.affine().is_finite()
        || !transform.affine().inverse().is_finite()
    {
        return None;
    }
    for (joint, &bind) in skin
        .joints
        .iter()
        .zip(binds.iter())
        .take(meta.max_joint + 1)
    {
        if !affine_finite(bind) || !joints.get(*joint).ok()?.affine().is_finite() {
            return None;
        }
    }
    let mut bounds: Aabb =
        entity_aabb_from_skinned_mesh_bounds(joints, mesh, skin, binds, Some(transform))
            .ok()?
            .into();
    if !bounds.center.is_finite()
        || !bounds.half_extents.is_finite()
        || bounds.half_extents.min_element() < 0.
    {
        return None;
    }
    let mut magnitude = transform.translation().abs().max_element() + 1.;
    for bits in 0..8 {
        let corner = Vec3::from(bounds.center)
            + Vec3::from(bounds.half_extents)
                * Vec3::new(
                    if bits & 1 == 0 { -1. } else { 1. },
                    if bits & 2 == 0 { -1. } else { 1. },
                    if bits & 4 == 0 { -1. } else { 1. },
                );
        magnitude = magnitude.max(
            transform.transform_point(corner).abs().max_element()
                + transform.translation().abs().max_element()
                + 1.,
        );
    }
    // Union-of-joint bounds assumes unit-sum weights. Cover accepted source
    // rounding and GPU arithmetic error in world space, then map that padding
    // conservatively through the entity inverse linear transform.
    let padding = magnitude * (meta.weight_error + 64. * f32::EPSILON) + 0.001;
    let inverse = transform.affine().inverse();
    let local = (inverse.transform_vector3(Vec3::X).abs()
        + inverse.transform_vector3(Vec3::Y).abs()
        + inverse.transform_vector3(Vec3::Z).abs())
        * padding;
    if !local.is_finite() {
        return None;
    }
    bounds.half_extents += bevy::math::Vec3A::from(local);
    Some(bounds)
}
fn certify(
    mut commands: Commands,
    frame: Res<FrameCount>,
    meshes: Res<Assets<Mesh>>,
    binds: Res<Assets<SkinnedMeshInverseBindposes>>,
    joints: Query<&GlobalTransform>,
    mut actors: Query<
        (
            Entity,
            &Mesh3d,
            &SkinnedMesh,
            &GlobalTransform,
            &Prepared,
            &mut Aabb,
        ),
        With<DynamicSkinnedMeshBounds>,
    >,
    mut last_report: Local<Option<(usize, usize)>>,
) {
    let mut seen = 0;
    let mut certified = 0;
    for (entity, handle, skin, transform, prepared, mut bounds) in &mut actors {
        seen += 1;
        if handle.id() != prepared.mesh {
            continue;
        }
        let Some(mesh) = meshes.get(&handle.0) else {
            continue;
        };
        let Some(bind) = binds.get(&skin.inverse_bindposes) else {
            continue;
        };
        if let Some(current) =
            current_bound(mesh, skin, bind, transform, &joints, prepared.metadata)
        {
            *bounds = current;
            certified += 1;
            commands.entity(entity).insert(FreshBounds {
                // Bevy FrameCountPlugin increments in Last, after this
                // PostUpdate certification and before render extraction.
                frame: frame.0.wrapping_add(1),
                mesh: handle.id(),
                metadata: prepared.metadata,
            });
        }
    }
    if std::env::var("CRYSTAL_GPU_CULLING_TRACE").as_deref() == Ok("on")
        && *last_report != Some((seen, certified))
    {
        eprintln!("shadow_hzb_skin_current candidates={seen} certified={certified}");
        *last_report = Some((seen, certified));
    }
}
pub(super) fn verified(
    fresh: Option<&FreshBounds>,
    frame: u32,
    handle: &Mesh3d,
    mesh: &Mesh,
    skin: &SkinnedMesh,
    binds: &SkinnedMeshInverseBindposes,
    transform: &GlobalTransform,
    bounds: Option<&Aabb>,
    joints: &Query<&GlobalTransform>,
) -> bool {
    let Some(fresh) = fresh.filter(|f| f.frame == frame && f.mesh == handle.id()) else {
        return false;
    };
    let Some(expected) = current_bound(mesh, skin, binds, transform, joints, fresh.metadata) else {
        return false;
    };
    bounds.is_some_and(|actual| {
        actual.center == expected.center && actual.half_extents == expected.half_extents
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{asset::RenderAssetUsages, render::render_resource::PrimitiveTopology};
    fn fixture() -> Mesh {
        let mut m = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        m.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[1., 0., 0.], [-1., 1., 0.], [0., -1., 0.]],
        );
        m.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(vec![[0, 1, 0, 0]; 3]),
        );
        m.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![[0.1, 0.9, 0., 0.]; 3]);
        m
    }
    #[test]
    fn dynamic_skin_metadata_rejects_nonfinite_negative_and_unnormalized_inputs() {
        assert!(metadata(&fixture()).is_some());
        for weights in [
            [f32::NAN, 0., 0., 0.],
            [-1., 2., 0., 0.],
            [0., 0., 0., 0.],
            [0.6, 0.6, 0., 0.],
        ] {
            let mut mesh = fixture();
            mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![weights; 3]);
            assert!(metadata(&mesh).is_none());
        }
        let mut mesh = fixture();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[f32::INFINITY, 0., 0.]; 3]);
        assert!(metadata(&mesh).is_none());
        let mut mesh = fixture();
        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, vec![[1., 0., 0., 0.]; 2]);
        assert!(metadata(&mesh).is_none());
    }
    #[test]
    fn official_joint_bounds_enclose_weighted_vertices_and_freshness_fails_closed() {
        let mut mesh = fixture();
        let mut meta = metadata(&mesh).unwrap();
        mesh.generate_skinned_mesh_bounds().unwrap();
        meta.generated = true;
        let mut world = World::new();
        let a = world
            .spawn(GlobalTransform::from_translation(Vec3::new(100., 4., 0.)))
            .id();
        let b = world
            .spawn(GlobalTransform::from(
                Transform::from_xyz(-50., 2., 3.).with_rotation(Quat::from_rotation_z(1.2)),
            ))
            .id();
        let mut query = world.query::<&GlobalTransform>();
        let joints = query.query(&world);
        let skin = SkinnedMesh {
            inverse_bindposes: default(),
            joints: vec![a, b],
        };
        let binds = SkinnedMeshInverseBindposes::from(vec![Mat4::IDENTITY; 2]);
        let transform = GlobalTransform::from(
            Transform::from_xyz(20., 0., 4.).with_scale(Vec3::new(2., 0.5, 1.)),
        );
        let bounds = current_bound(&mesh, &skin, &binds, &transform, &joints, meta).unwrap();
        for p in [[1., 0., 0.], [-1., 1., 0.], [0., -1., 0.]] {
            let point = joints.get(a).unwrap().transform_point(Vec3::from_array(p)) * 0.1
                + joints.get(b).unwrap().transform_point(Vec3::from_array(p)) * 0.9;
            let local = transform.affine().inverse().transform_point3(point);
            assert!(
                (local - Vec3::from(bounds.center))
                    .abs()
                    .cmple(Vec3::from(bounds.half_extents))
                    .all()
            );
        }
        let handle = Mesh3d(default());
        let fresh = FreshBounds {
            frame: 7,
            mesh: handle.id(),
            metadata: meta,
        };
        assert!(verified(
            Some(&fresh),
            7,
            &handle,
            &mesh,
            &skin,
            &binds,
            &transform,
            Some(&bounds),
            &joints
        ));
        assert!(!verified(
            Some(&fresh),
            8,
            &handle,
            &mesh,
            &skin,
            &binds,
            &transform,
            Some(&bounds),
            &joints
        ));
        assert!(!verified(
            None,
            7,
            &handle,
            &mesh,
            &skin,
            &binds,
            &transform,
            Some(&bounds),
            &joints
        ));
        let missing = SkinnedMesh {
            inverse_bindposes: default(),
            joints: vec![a],
        };
        assert!(current_bound(&mesh, &missing, &binds, &transform, &joints, meta).is_none());
    }
}
