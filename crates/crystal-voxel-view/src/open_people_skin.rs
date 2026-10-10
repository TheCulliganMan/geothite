//! GPU resources and instances for the external people reader.
use crate::{new_bark_actors::ModeledActor, open_people::PersonRig, VOXEL_RENDER_LAYER};
use bevy::{
    prelude::*,
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            VertexAttributeValues,
        },
        primitives::Aabb,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        view::RenderLayers,
    },
};
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub(crate) struct PeopleGpu(pub HashMap<&'static str, Gpu>);
pub(crate) struct Gpu {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    remote: Handle<StandardMaterial>,
    binds: Handle<SkinnedMeshInverseBindposes>,
}
pub(crate) struct Person {
    pub rig: Arc<PersonRig>,
    nodes: Vec<Entity>,
    idle: Vec<Transform>,
    run: Vec<Transform>,
}
impl PeopleGpu {
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        &mut self,
        name: &'static str,
        rig: Arc<PersonRig>,
        root: Entity,
        remote: bool,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
        binds: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> Person {
        let gpu = self.0.entry(name).or_insert_with(|| {
            let mut mesh = rig.mesh.clone().into_mesh();
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(rig.ids.clone()),
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, rig.weights.clone());
            let texture = images.add(Image::new(
                Extent3d {
                    width: rig.texture_size.0,
                    height: rig.texture_size.1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                rig.rgba.clone(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            ));
            let material = StandardMaterial {
                base_color: Color::srgb(0.97, 0.96, 0.94),
                base_color_texture: Some(texture),
                perceptual_roughness: 0.96,
                reflectance: 0.12,
                cull_mode: None,
                ..default()
            };
            let ghost = StandardMaterial {
                base_color: Color::srgba(0.48, 0.88, 1.0, 0.62),
                alpha_mode: AlphaMode::Blend,
                ..material.clone()
            };
            Gpu {
                mesh: meshes.add(mesh),
                material: materials.add(material),
                remote: materials.add(ghost),
                binds: binds.add(SkinnedMeshInverseBindposes::from(rig.inverse_binds.clone())),
            }
        });
        let mut idle = rig.nodes.clone();
        rig.sample("Idle", 0.0, &mut idle);
        let nodes: Vec<_> = idle
            .iter()
            .map(|t| {
                commands
                    .spawn((
                        SpatialBundle {
                            transform: *t,
                            ..default()
                        },
                        ModeledActor,
                    ))
                    .id()
            })
            .collect();
        let pivot = commands
            .spawn((
                SpatialBundle {
                    transform: Transform::from_xyz(0.0, -rig.floor, 0.0),
                    ..default()
                },
                ModeledActor,
            ))
            .id();
        commands.entity(root).add_child(pivot);
        for (i, &entity) in nodes.iter().enumerate() {
            commands
                .entity(rig.parents[i].map(|p| nodes[p]).unwrap_or(pivot))
                .add_child(entity);
        }
        commands.entity(nodes[rig.mesh_node]).insert((
            PbrBundle {
                mesh: gpu.mesh.clone(),
                material: if remote {
                    gpu.remote.clone()
                } else {
                    gpu.material.clone()
                },
                transform: idle[rig.mesh_node],
                ..default()
            },
            SkinnedMesh {
                inverse_bindposes: gpu.binds.clone(),
                joints: rig.skin_nodes.iter().map(|&i| nodes[i]).collect(),
            },
            Aabb::from_min_max(
                rig.bounds.0 - Vec3::splat(0.02),
                rig.bounds.1 + Vec3::splat(0.02),
            ),
            RenderLayers::layer(VOXEL_RENDER_LAYER),
        ));
        Person {
            run: idle.clone(),
            idle,
            rig,
            nodes,
        }
    }
}
impl Person {
    pub fn update(
        &mut self,
        phase: f32,
        weight: f32,
        seconds: f32,
        transforms: &mut Query<&mut Transform, With<ModeledActor>>,
    ) {
        self.rig.sample(
            "Idle",
            seconds.rem_euclid(self.rig.duration("Idle")),
            &mut self.idle,
        );
        self.rig.sample(
            "Run",
            phase.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU
                * self.rig.duration("Run"),
            &mut self.run,
        );
        for ((&entity, a), b) in self.nodes.iter().zip(&self.idle).zip(&self.run) {
            if let Ok(mut t) = transforms.get_mut(entity) {
                *t = Transform {
                    translation: a.translation.lerp(b.translation, weight),
                    rotation: a.rotation.slerp(b.rotation, weight).normalize(),
                    scale: a.scale.lerp(b.scale, weight),
                };
            }
        }
    }
}
