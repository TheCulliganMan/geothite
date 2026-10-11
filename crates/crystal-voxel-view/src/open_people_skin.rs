//! GPU resources and instances for the external people reader.
use bevy::asset::RenderAssetUsages;
use crate::{new_bark_actors::ModeledActor, open_people::PersonRig, VOXEL_RENDER_LAYER};
use bevy::camera::{primitives::Aabb, visibility::RenderLayers};
use bevy::{
    light::NotShadowReceiver,
    prelude::*,
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            VertexAttributeValues,
        },


        render_resource::{Extent3d, TextureDimension, TextureFormat},

    },
};
use std::{collections::HashMap, sync::Arc};
use crate::new_bark_actors::character_meshes::CharacterKind;

const HEAD_SCALE: f32 = 0.72;
const HAND_SCALE: f32 = 0.82;

#[derive(Default)]
pub(crate) struct PeopleGpu(pub HashMap<(&'static str, CharacterKind), Gpu>);
pub(crate) struct Gpu {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    remote: Handle<StandardMaterial>,
    binds: Handle<SkinnedMeshInverseBindposes>,
    bounds: (Vec3,Vec3),
    head: Option<AccessoryGpu>,
    costume: Option<AccessoryGpu>,
    backpack: Option<AccessoryGpu>,
}
struct AccessoryGpu {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    remote: Handle<StandardMaterial>,
}
fn accessory_gpu(mesh: crate::mesh::SurfaceMeshData, meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>) -> AccessoryGpu {
    accessory_mesh_gpu(mesh.into_mesh(), meshes, materials)
}
fn accessory_mesh_gpu(mesh: Mesh, meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>) -> AccessoryGpu {
    let material = StandardMaterial {
        perceptual_roughness: 0.96, reflectance: 0.12, cull_mode: None, ..default()
    };
    let ghost = StandardMaterial {
        base_color: Color::srgba(0.48, 0.88, 1.0, 0.62),
        alpha_mode: AlphaMode::Blend, ..material.clone()
    };
    AccessoryGpu { mesh: meshes.add(mesh),
        material: materials.add(material), remote: materials.add(ghost) }
}
pub(crate) struct Person {
    pub bicycle: bool,
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
        kind: CharacterKind,
        rig: Arc<PersonRig>,
        root: Entity,
        remote: bool,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
        binds: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> Person {
        let gpu = self.0.entry((name, kind)).or_insert_with(|| {
            let look = crate::npc_appearance::look(kind);
            let painting = crate::npc_textures::get(look.texture_key)
                .expect("required production character painting");
            let mut body = rig.mesh.clone();
            let mut body_joints = rig.ids.clone();
            let mut body_weights = rig.weights.clone();
            crate::hand_surface::smooth(&rig,&mut body,&mut body_joints,&mut body_weights);
            crate::hand_digits::sculpt(&rig,&mut body,&mut body_joints,&mut body_weights);
            crate::head_surface::smooth(&rig,&mut body,&mut body_joints,&mut body_weights);
            crate::head_surface::bald_crown(&rig,&mut body,&mut body_joints,&mut body_weights,look);
            crate::npc_appearance::child_head(&rig,&mut body,&mut body_joints,&mut body_weights,look);
            crate::npc_appearance::soften_jaw_normals(&rig,&mut body,&body_joints,&body_weights);
            if matches!(kind,
                CharacterKind::Trainer | CharacterKind::TrainerFemale | CharacterKind::Receptionist
                | CharacterKind::Clerk | CharacterKind::Officer | CharacterKind::Cal
                | CharacterKind::CooltrainerM | CharacterKind::LinkReceptionist
                | CharacterKind::OldLinkReceptionist | CharacterKind::Teacher
                | CharacterKind::RedsMom | CharacterKind::Bill | CharacterKind::GymGuide
                | CharacterKind::Biker | CharacterKind::BlackBelt | CharacterKind::PokefanF
                | CharacterKind::Rocket | CharacterKind::RocketGirl
                | CharacterKind::Bruno | CharacterKind::Chuck | CharacterKind::BugCatcher | CharacterKind::Bugsy
                | CharacterKind::GameboyKid | CharacterKind::StandingYoungster
                | CharacterKind::Outdoorsman | CharacterKind::UnusedGuy | CharacterKind::Will
                | CharacterKind::SwimmerGuy | CharacterKind::SwimmerGirl | CharacterKind::Twin) {
                crate::head_surface::adventure_profile(&rig, &mut body, &mut body_joints, &mut body_weights);
            }
            let hair_extension=crate::hair_shapes::sculpt(&rig,&mut body,&mut body_joints,&mut body_weights,look);
            crate::npc_appearance::shorts_body(&rig,&mut body,&mut body_joints,&mut body_weights,look);
            crate::kimono_sleeves::sculpt(&mut body,&mut body_joints,&mut body_weights,look);
            crate::garment_body::cover(&mut body,&mut body_joints,&mut body_weights,look);
            crate::npc_appearance::correct_body_uvs(&mut body, look, &mut body_joints, &mut body_weights);
            crate::painted_torso::map_front(&mut body, look, &mut body_joints, &mut body_weights);
            crate::painted_torso::long_sleeves(&mut body, &mut body_joints, &mut body_weights, look);
            if matches!(look.texture_key, "red" | "misty" | "brock" | "pokefan_m") {
                crate::painted_torso::short_sleeves(&mut body, &mut body_joints, &mut body_weights, look);
            }
            crate::bare_body::sculpt(&mut body, &mut body_joints, &mut body_weights, look);
            crate::footwear::adapt(&mut body,&mut body_joints,&mut body_weights,look);
            let mut mesh = body.into_mesh();
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(body_joints),
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, body_weights);
            let size = painting.size;
            let rgba = painting.rgba.clone();
            let texture = images.add(Image::new(
                Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                rgba,
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
                bounds: (rig.bounds.0-Vec3::splat(hair_extension),rig.bounds.1+Vec3::splat(hair_extension)),
                mesh: meshes.add(mesh),
                material: materials.add(material),
                remote: materials.add(ghost),
                binds: binds.add(SkinnedMeshInverseBindposes::from(rig.inverse_binds.clone())),
                head: crate::npc_appearance::head_mesh(&rig, look)
                    .map(|mesh| accessory_gpu(mesh, meshes, materials)),
                costume: crate::npc_appearance::costume_mesh(&rig, look)
                    .map(|costume| {
                        let mut mesh = costume.surface.into_mesh();
                        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX,
                            VertexAttributeValues::Uint16x4(costume.joints));
                        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, costume.weights);
                        accessory_mesh_gpu(mesh, meshes, materials)
                    }),
                backpack: crate::npc_appearance::backpack_mesh(&rig, look)
                    .map(|mesh| accessory_gpu(mesh, meshes, materials)),
            }
        });
        let mut idle = rig.nodes.clone();
        rig.sample("Idle", 0.0, &mut idle);
        crate::hand_pose::relax(&rig, &mut idle);
        let nodes: Vec<_> = idle
            .iter()
            .map(|t| {
                commands
                    .spawn((
                        (Visibility::default(),
*t,),
                        ModeledActor,
                    ))
                    .id()
            })
            .collect();
        let pivot = commands
            .spawn((
                (Visibility::default(),
Transform::from_xyz(0.0, -rig.floor, 0.0),),
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
            (Mesh3d(gpu.mesh.clone()),
MeshMaterial3d(if remote {
                    gpu.remote.clone()
                } else {
                    gpu.material.clone()
                }),
idle[rig.mesh_node],),
            // Keep painted faces readable under roofs and hats. Actors still
            // cast shadows onto the unchanged world, and retain directional
            // shading across their own paper contours.
            NotShadowReceiver,
            SkinnedMesh {
                inverse_bindposes: gpu.binds.clone(),
                joints: rig.skin_nodes.iter().map(|&i| nodes[i]).collect(),
            },
            Aabb::from_min_max(
                gpu.bounds.0 - Vec3::splat(0.02),
                gpu.bounds.1 + Vec3::splat(0.02),
            ),
            RenderLayers::layer(VOXEL_RENDER_LAYER),
        ));
        if let Some(gpu) = &gpu.head {
            let accessory = commands.spawn(((Mesh3d(gpu.mesh.clone()),
MeshMaterial3d(if remote { gpu.remote.clone() } else { gpu.material.clone() }),), ModeledActor, NotShadowReceiver, RenderLayers::layer(VOXEL_RENDER_LAYER))).id();
            commands.entity(nodes[rig.skin_nodes[rig.head_joint]]).add_child(accessory);
        }
        if let Some(pack) = &gpu.backpack {
            let accessory = commands.spawn(((Mesh3d(pack.mesh.clone()),
MeshMaterial3d(if remote { pack.remote.clone() } else { pack.material.clone() }),), ModeledActor, NotShadowReceiver, RenderLayers::layer(VOXEL_RENDER_LAYER))).id();
            commands.entity(nodes[rig.skin_nodes[rig.chest_joint]]).add_child(accessory);
        }
        if let Some(costume) = &gpu.costume {
            let accessory = commands.spawn(((Mesh3d(costume.mesh.clone()),
MeshMaterial3d(if remote { costume.remote.clone() } else { costume.material.clone() }),), SkinnedMesh {
                inverse_bindposes: gpu.binds.clone(),
                joints: rig.skin_nodes.iter().map(|&i| nodes[i]).collect(),
            }, Aabb::from_min_max(rig.bounds.0-Vec3::splat(0.6), rig.bounds.1+Vec3::splat(0.6)),
                ModeledActor, NotShadowReceiver, RenderLayers::layer(VOXEL_RENDER_LAYER))).id();
            commands.entity(nodes[rig.mesh_node]).add_child(accessory);
        }
        Person {
            bicycle: false,
            run: idle.clone(),
            idle,
            rig,
            nodes,
        }
    }
}
impl Person {
    /// Offline proportion experiment. Applying scale at the head joint keeps
    /// fitted hats and other head accessories in the same animated hierarchy.
    #[cfg(feature = "people-review")]
    pub(crate) fn review_head_scale(&self, scale: f32,
        transforms: &mut Query<&mut Transform, With<ModeledActor>>) {
        self.scale_head(scale / HEAD_SCALE, transforms);
    }
    fn scale_head(&self, scale: f32,
        transforms: &mut Query<&mut Transform, With<ModeledActor>>) {
        let entity = self.nodes[self.rig.skin_nodes[self.rig.head_joint]];
        if let Ok(mut transform) = transforms.get_mut(entity) {
            transform.scale *= scale;
        }
    }
    /// Scale each wrist once; finger descendants inherit it without scaling
    /// their own joints again. This is an offline experiment, not game state.
    #[cfg(feature = "people-review")]
    pub(crate) fn review_hand_scale(&self, scale: f32,
        transforms: &mut Query<&mut Transform, With<ModeledActor>>) {
        self.scale_hands(scale / HAND_SCALE, transforms);
    }
    fn scale_hands(&self, scale: f32,
        transforms: &mut Query<&mut Transform, With<ModeledActor>>) {
        for hand in &self.rig.hand_joints {
            for &joint in hand {
                let node = self.rig.skin_nodes[joint];
                let mut parent = self.rig.parents[node];
                let mut inherits_hand_scale = false;
                while let Some(ancestor) = parent {
                    if hand.iter().any(|&j| self.rig.skin_nodes[j] == ancestor) {
                        inherits_hand_scale = true;
                        break;
                    }
                    parent = self.rig.parents[ancestor];
                }
                if inherits_hand_scale { continue; }
                if let Ok(mut transform) = transforms.get_mut(self.nodes[node]) {
                    transform.scale *= scale;
                }
            }
        }
    }
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
        crate::hand_pose::relax(&self.rig, &mut self.idle);
        crate::hand_pose::relax(&self.rig, &mut self.run);
        if self.bicycle {
            // Mounted riders use the same updated mesh and skin, seated rather
            // than replaying a walking clip through a baked legacy figure.
            self.run.clone_from(&self.idle);
            crate::bicycle::pose(&self.rig, &mut self.idle, phase);
            crate::bicycle::pose(&self.rig, &mut self.run, phase);
        }
        for ((&entity, a), b) in self.nodes.iter().zip(&self.idle).zip(&self.run) {
            if let Ok(mut t) = transforms.get_mut(entity) {
                *t = Transform {
                    translation: a.translation.lerp(b.translation, weight),
                    rotation: a.rotation.slerp(b.rotation, weight).normalize(),
                    scale: a.scale.lerp(b.scale, weight),
                };
            }
        }
        // Apply after rebuilding every local transform from animation, so
        // these proportions cannot accumulate across frames. Hats and hair
        // inherit the head joint; fingers inherit each wrist exactly once.
        self.scale_head(HEAD_SCALE, transforms);
        self.scale_hands(HAND_SCALE, transforms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn costume_instances_share_gpu_assets_and_own_their_animated_joint_palettes() {
        use bevy::ecs::{system::SystemState, world::CommandQueue};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = Arc::new(PersonRig::parse(&std::fs::read(
            root.join("people/kenney/human-female.glb")).unwrap()).unwrap());
        let mut world = World::new();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
        let mut cache = PeopleGpu::default();
        let mut queue = CommandQueue::default();
        let mut people = Vec::new();
        let mut roots = Vec::new();
        for remote in [false,true] {
            let mut commands = Commands::new(&mut queue,&world);
            let root = commands.spawn((Transform::default(), Visibility::default())).id();
            roots.push(root);
            people.push(cache.spawn("human-female",CharacterKind::Mom,rig.clone(),root,remote,
                &mut commands,&mut meshes,&mut materials,&mut images,&mut binds));
            queue.apply(&mut world);
        }
        let gpu = &cache.0[&("human-female",CharacterKind::Mom)];
        let cloth = gpu.costume.as_ref().unwrap();
        let mut skins = world.query::<(&SkinnedMesh,&Mesh3d,&ChildOf)>();
        let costumes: Vec<_> = skins.iter(&world).filter(|(_,mesh,_)| mesh.0==cloth.mesh).collect();
        assert_eq!(costumes.len(),2);
        assert_eq!(skins.iter(&world).count(),4,"body and garment skins for both actors");
        assert_eq!(costumes[0].0.inverse_bindposes,costumes[1].0.inverse_bindposes);
        assert_ne!(costumes[0].0.joints,costumes[1].0.joints,"instances may not share live joints");
        for (skin,_,parent) in costumes {
            assert_eq!(world.get::<Mesh3d>(parent.parent()).map(|mesh| &mesh.0),Some(&gpu.mesh));
            assert_eq!(skin.inverse_bindposes,gpu.binds);
        }
        let counts = (meshes.len(),materials.len(),images.len(),binds.len());
        let untouched = *world.get::<Transform>(people[1].nodes[rig.skin_nodes[rig.leg_joints[0][0]]]).unwrap();
        let finger = rig.finger_nodes[1];
        let other_finger = *world.get::<Transform>(people[1].nodes[finger]).unwrap();
        let mut state = SystemState::<Query<&mut Transform,With<ModeledActor>>>::new(&mut world);
        people[0].update(std::f32::consts::FRAC_PI_2,1.0,0.0,&mut state.get_mut(&mut world).unwrap());
        assert_eq!(*world.get::<Transform>(people[1].nodes[rig.skin_nodes[rig.leg_joints[0][0]]]).unwrap(),untouched);
        assert_eq!(*world.get::<Transform>(people[1].nodes[finger]).unwrap(),other_finger);
        let head_node = rig.skin_nodes[rig.head_joint];
        let head = *world.get::<Transform>(people[0].nodes[head_node]).unwrap();
        let foot_nodes = rig.leg_joints.map(|leg| rig.skin_nodes[leg[1]]);
        let feet = foot_nodes.map(|node| *world.get::<Transform>(people[0].nodes[node]).unwrap());
        // A second identical frame must not shrink again, move the feet, or
        // mutate the independently animated remote instance.
        people[0].update(std::f32::consts::FRAC_PI_2,1.0,0.0,&mut state.get_mut(&mut world).unwrap());
        assert_eq!(*world.get::<Transform>(people[0].nodes[head_node]).unwrap(), head);
        assert_eq!(foot_nodes.map(|node| *world.get::<Transform>(people[0].nodes[node]).unwrap()), feet);
        assert_eq!(*world.get::<Transform>(people[1].nodes[finger]).unwrap(),other_finger);
        let mut expected = rig.nodes.clone();
        rig.sample("Run",rig.duration("Run")*0.25,&mut expected);
        crate::hand_pose::relax(&rig,&mut expected);
        assert!(head.scale.distance(expected[head_node].scale * HEAD_SCALE) < 1e-6);
        for &node in &rig.finger_nodes {
            let actual = world.get::<Transform>(people[0].nodes[node]).unwrap();
            assert!(actual.rotation.angle_between(expected[node].rotation)<0.001,
                "visible instance skipped relaxed finger pose");
            // The production Idle/Run blend introduces float rounding even
            // at weight 1; compare within floating point bind-space precision.
            assert!(actual.translation.distance(expected[node].translation)<1e-6);
        }
        assert_eq!(counts,(meshes.len(),materials.len(),images.len(),binds.len()));
        let ghost = materials.get(&cloth.remote).unwrap();
        assert_eq!(ghost.alpha_mode,AlphaMode::Blend);
        assert!((ghost.base_color.alpha()-0.62).abs()<0.001);
        let mut commands = Commands::new(&mut queue,&world);
        for root in roots { commands.entity(root).despawn(); }
        queue.apply(&mut world);
        assert_eq!(world.query::<&SkinnedMesh>().iter(&world).count(),0);
    }
}
