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
    model_scale: Vec3,
    head_scale: f32,
    hand_scale: f32,
    nodes: Vec<Entity>,
    wrist_nodes: Vec<usize>,
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
        bicycle: bool,
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
                | CharacterKind::Biker | CharacterKind::BlackBelt | CharacterKind::PokefanF | CharacterKind::Blue | CharacterKind::Brock
                | CharacterKind::Rocket | CharacterKind::RocketGirl
                | CharacterKind::Bruno | CharacterKind::Chuck | CharacterKind::BugCatcher | CharacterKind::Bugsy
                | CharacterKind::GameboyKid | CharacterKind::StandingYoungster
                | CharacterKind::Outdoorsman | CharacterKind::UnusedGuy | CharacterKind::Will
                | CharacterKind::SwimmerGuy | CharacterKind::SwimmerGirl | CharacterKind::Twin) {
                crate::head_surface::adventure_profile(&rig, &mut body, &mut body_joints, &mut body_weights);
            }
            if matches!(kind, CharacterKind::Trainer | CharacterKind::TrainerFemale) {
                crate::head_surface::player_profile(&rig, &mut body, &body_joints, &body_weights);
            }
            if matches!(kind, CharacterKind::Kurt | CharacterKind::KurtOutside) {
                // Keep Kurt's drawn eyes, nose, mouth and forehead in place.
                // Extra refinement is restricted to the established jaw/nape
                // selection, with pinned crown and collar boundaries.
                for _ in 0..2 {
                    crate::head_surface::smooth(&rig, &mut body, &mut body_joints, &mut body_weights);
                }
                crate::head_surface::nape_normals(&rig, &mut body, &body_joints, &body_weights);
            }
            let hair_extension=crate::hair_shapes::sculpt(&rig,&mut body,&mut body_joints,&mut body_weights,look);
            crate::npc_appearance::shorts_body(&rig,&mut body,&mut body_joints,&mut body_weights,look);
            crate::kimono_sleeves::sculpt(&mut body,&mut body_joints,&mut body_weights,look);
            crate::garment_body::cover(&mut body,&mut body_joints,&mut body_weights,look);
            crate::garment_body::nurse_legs(&mut body,&mut body_joints,&mut body_weights,look);
            crate::npc_appearance::correct_body_uvs(&mut body, look, &mut body_joints, &mut body_weights);
            crate::painted_torso::avoid_shirt_gutter(&mut body, look, &mut body_joints, &mut body_weights);
            crate::painted_torso::map_front(&mut body, look, &mut body_joints, &mut body_weights);
            crate::painted_torso::long_sleeves(&mut body, &mut body_joints, &mut body_weights, look);
            if matches!(look.texture_key, "red" | "misty" | "brock" | "pokefan_m" | "nurse") {
                crate::painted_torso::short_sleeves(&mut body, &mut body_joints, &mut body_weights, look);
            }
            crate::bare_body::sculpt(&rig, &mut body, &mut body_joints, &mut body_weights, look);
            crate::footwear::adapt(&mut body,&mut body_joints,&mut body_weights,look);
            crate::handwear::adapt(&rig, &mut body, &mut body_joints, &mut body_weights, look);
            // Offline review only: inspect the final production surface rather
            // than diagnosing altered normals from the original GLB alone.
            #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
            if let Some(directory) = std::env::var_os("CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES") {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory).expect("create mesh review directory");
                let snapshot = serde_json::json!({
                    "positions": body.positions,
                    "normals": body.normals,
                    "uvs": body.uvs,
                    "indices": body.indices,
                    "joints": body_joints,
                    "weights": body_weights,
                });
                std::fs::write(directory.join(format!("{}.json", look.texture_key)),
                    serde_json::to_vec(&snapshot).expect("encode review mesh"))
                    .expect("write review mesh");
            }
            // Walking and mounted instances share GPU resources. Include both
            // envelopes even when the first instance happens to be on foot.
            let mut bounds = (rig.bounds.0-Vec3::splat(hair_extension),
                rig.bounds.1+Vec3::splat(hair_extension));
            if matches!(kind, CharacterKind::Trainer | CharacterKind::TrainerFemale) {
                let scale = Vec3::new(look.height / rig.height * look.width,
                    look.height / rig.height, look.height / rig.height);
                let mounted = crate::bicycle::body_bounds(&rig, &body.positions,
                    &body_joints, &body_weights, scale, HEAD_SCALE, HAND_SCALE);
                bounds.0 = bounds.0.min(mounted.0);
                bounds.1 = bounds.1.max(mounted.1);
            }
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
                bounds,
                mesh: meshes.add(mesh),
                material: materials.add(material),
                remote: materials.add(ghost),
                binds: binds.add(SkinnedMeshInverseBindposes::from(rig.inverse_binds.clone())),
                head: crate::npc_appearance::head_mesh(&rig, look)
                    .map(|mesh| accessory_gpu(mesh, meshes, materials)),
                costume: crate::npc_appearance::costume_mesh(&rig, look)
                    .map(|costume| {
                        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
                        if let Some(directory)=std::env::var_os("CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES") {
                            let snapshot=serde_json::json!({
                                "positions":costume.surface.positions,
                                "normals":costume.surface.normals,
                                "uvs":costume.surface.uvs,
                                "indices":costume.surface.indices,
                                "joints":costume.joints,
                                "weights":costume.weights,
                            });
                            std::fs::write(std::path::PathBuf::from(directory)
                                .join(format!("{}.costume.json",look.texture_key)),
                                serde_json::to_vec(&snapshot).expect("encode review costume"))
                                .expect("write review costume");
                        }
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
        let look = crate::npc_appearance::look(kind);
        let model_scale = Vec3::new(look.height / rig.height * look.width,
            look.height / rig.height, look.height / rig.height);
        let head_scale = match kind {
            CharacterKind::PokefanF => 0.60,
            CharacterKind::Youngster | CharacterKind::StandingYoungster | CharacterKind::BugCatcher => 0.84,
            _ => HEAD_SCALE,
        };
        let hand_scale = if kind == CharacterKind::Blue { 0.70 } else { HAND_SCALE };
        let wrist_nodes = rig.hand_joints.iter().flat_map(|hand| hand.iter().filter_map(|&joint| {
            let node = rig.skin_nodes[joint];
            let mut parent = rig.parents[node];
            while let Some(ancestor) = parent {
                if hand.iter().any(|&j| rig.skin_nodes[j] == ancestor) { return None; }
                parent = rig.parents[ancestor];
            }
            Some(node)
        })).collect::<Vec<_>>();
        let mut idle = rig.nodes.clone();
        rig.sample("Idle", 0.0, &mut idle);
        if bicycle { crate::bicycle::pose(&rig, &mut idle, 0., model_scale); }
        else { crate::hand_pose::relax(&rig, &mut idle); }
        // Deferred entities must already contain the final visible pose.
        // Keep the animation buffers unscaled so updates never accumulate it.
        let mut presented = idle.clone();
        presented[rig.skin_nodes[rig.head_joint]].scale *= head_scale;
        for &node in &wrist_nodes { presented[node].scale *= hand_scale; }
        let nodes: Vec<_> = presented
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
presented[rig.mesh_node],),
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
            bicycle,
            model_scale,
            head_scale,
            hand_scale,
            wrist_nodes,
            run: idle.clone(),
            idle,
            rig,
            nodes,
        }
    }
}
impl Person {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    pub(crate) fn review_skin_palette(&self,
        transforms: &Query<&mut Transform, With<ModeledActor>>) -> Vec<[f32; 16]> {
        fn world(index: usize, rig: &PersonRig, nodes: &[Entity],
            transforms: &Query<&mut Transform, With<ModeledActor>>) -> Mat4 {
            let local = transforms.get(nodes[index]).expect("review node transform").to_matrix();
            rig.parents[index].map(|parent| world(parent, rig, nodes, transforms))
                .unwrap_or(Mat4::IDENTITY) * local
        }
        let inverse = world(self.rig.mesh_node, &self.rig, &self.nodes, transforms).inverse();
        self.rig.skin_nodes.iter().zip(&self.rig.inverse_binds)
            .map(|(&node, bind)| (inverse * world(node, &self.rig, &self.nodes, transforms)
                * *bind).to_cols_array()).collect()
    }

    /// Offline proportion experiment. Applying scale at the head joint keeps
    /// fitted hats and other head accessories in the same animated hierarchy.
    #[cfg(feature = "people-review")]
    pub(crate) fn review_head_scale(&self, scale: f32,
        transforms: &mut Query<&mut Transform, With<ModeledActor>>) {
        self.scale_head(scale / self.head_scale, transforms);
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
        self.scale_hands(scale / self.hand_scale, transforms);
    }
    fn scale_hands(&self, scale: f32,
        transforms: &mut Query<&mut Transform, With<ModeledActor>>) {
        for &node in &self.wrist_nodes {
            if let Ok(mut transform) = transforms.get_mut(self.nodes[node]) {
                transform.scale *= scale;
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
        if !self.bicycle {
            crate::hand_pose::relax(&self.rig, &mut self.idle);
            crate::hand_pose::relax(&self.rig, &mut self.run);
        }
        if self.bicycle {
            // Mounted riders use the same updated mesh and skin, seated rather
            // than replaying a walking clip through a baked legacy figure.
            crate::bicycle::pose(&self.rig, &mut self.idle, phase, self.model_scale);
            self.run.clone_from(&self.idle);
        }
        // Offline robe-gait experiment: isolate knee flexion from the whole
        // stride blend. The default production animation is unchanged.
        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
        if !self.bicycle {
            if let Ok(value) = std::env::var("CRYSTAL_PEOPLE_REVIEW_KNEE_FLEX") {
                let flex: f32 = value.parse().expect("review knee flex must be numeric");
                assert!(flex.is_finite() && (0.0..=1.0).contains(&flex),
                    "review knee flex must be from 0 to 1");
                for leg in self.rig.leg_joints {
                    let node = self.rig.skin_nodes[leg[1]];
                    self.run[node].rotation = self.idle[node].rotation
                        .slerp(self.run[node].rotation, flex).normalize();
                }
            }
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
        self.scale_head(self.head_scale, transforms);
        self.scale_hands(self.hand_scale, transforms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn costume_instances_share_gpu_assets_and_own_their_animated_joint_palettes() {
        for kind in [CharacterKind::Mom, CharacterKind::PokefanF, CharacterKind::Blue,
            CharacterKind::Youngster, CharacterKind::StandingYoungster, CharacterKind::BugCatcher] {
            verify_costume_instance_animation(kind);
        }
    }

    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    #[test]
    #[ignore = "requires supplied rider rigs and paintings in CRYSTAL_OPEN_MODEL_ROOT"]
    fn mounted_production_hands_and_shoes_fit_grips_and_pedals_through_full_cycle() {
        use bevy::ecs::{system::SystemState, world::CommandQueue};
        fn sole_height(a: Vec3, b: Vec3, c: Vec3, x: f32, z: f32) -> Option<f32> {
            let denominator = (b.z-c.z)*(a.x-c.x)+(c.x-b.x)*(a.z-c.z);
            if denominator.abs() < 1e-10 { return None; }
            let u = ((b.z-c.z)*(x-c.x)+(c.x-b.x)*(z-c.z))/denominator;
            let v = ((c.z-a.z)*(x-c.x)+(a.x-c.x)*(z-c.z))/denominator;
            (u.min(v).min(1.-u-v) >= -1e-6).then_some(u*a.y+v*b.y+(1.-u-v)*c.y)
        }
        fn clip(poly: &[Vec3], bound: f32, sign: f32) -> Vec<Vec3> {
            let mut result = Vec::new();
            for (a,b) in poly.iter().zip(poly.iter().cycle().skip(1)).take(poly.len()) {
                let va = (a.x-bound)*sign;
                let vb = (b.x-bound)*sign;
                if va >= 0. { result.push(*a); }
                if (va >= 0.) != (vb >= 0.) { result.push(a.lerp(*b, va/(va-vb))); }
            }
            result
        }
        fn radial_distance(poly: &[Vec3]) -> f32 {
            if poly.is_empty() { return f32::INFINITY; }
            let points = poly.iter().map(|p| Vec2::new(p.y-0.720,p.z-0.25)).collect::<Vec<_>>();
            let mut minimum = f32::INFINITY;
            let mut low = f32::INFINITY;
            let mut high = f32::NEG_INFINITY;
            let mut area = 0.;
            for (&a,&b) in points.iter().zip(points.iter().cycle().skip(1)).take(points.len()) {
                let edge = b-a;
                let t = if edge.length_squared() > 1e-20 {
                    (-a.dot(edge)/edge.length_squared()).clamp(0.,1.)
                } else { 0. };
                minimum = minimum.min((a+edge*t).length());
                let cross = a.perp_dot(b);
                area += cross;
                low = low.min(cross);
                high = high.max(cross);
            }
            if area.abs() > 1e-14 && (low >= -1e-14 || high <= 1e-14) { 0. } else { minimum }
        }
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        for kind in [CharacterKind::Trainer, CharacterKind::TrainerFemale] {
            let look = crate::npc_appearance::look(kind);
            let rig = Arc::new(PersonRig::parse(&std::fs::read(
                root.join(format!("people/kenney/{}.glb", look.base))).unwrap()).unwrap());
            let mut world = World::new();
            let mut queue = CommandQueue::default();
            let mut meshes = Assets::<Mesh>::default();
            let mut materials = Assets::<StandardMaterial>::default();
            let mut images = Assets::<Image>::default();
            let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
            let mut cache = PeopleGpu::default();
            let mut commands = Commands::new(&mut queue, &world);
            let actor = commands.spawn((Transform::default(), Visibility::default())).id();
            let mut person = cache.spawn(look.base, kind, rig.clone(), actor, false, true,
                &mut commands, &mut meshes, &mut materials, &mut images, &mut binds);
            person.bicycle = true;
            queue.apply(&mut world);
            let body = meshes.get(&cache.0[&(look.base, kind)].mesh).unwrap();
            let Some(VertexAttributeValues::Float32x3(positions)) = body.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!("positions"); };
            let Some(VertexAttributeValues::Uint16x4(ids)) = body.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) else { panic!("joints"); };
            let Some(VertexAttributeValues::Float32x4(weights)) = body.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT) else { panic!("weights"); };
            let feet = rig.leg_joints.map(|leg| {
                let shin = rig.skin_nodes[leg[1]];
                rig.skin_nodes.iter().position(|&node| rig.parents[node] == Some(shin)).unwrap()
            });
            let mut state = SystemState::<Query<&mut Transform, With<ModeledActor>>>::new(&mut world);
            for step in 0..32 {
                let phase = step as f32 / 32. * std::f32::consts::TAU;
                let mut query = state.get_mut(&mut world).unwrap();
                // Include the source Idle cycle as well as the pedal phase.
                person.update(phase, 1., step as f32 / 32. * rig.duration("Idle"), &mut query);
                let palette = person.review_skin_palette(&query).iter()
                    .map(Mat4::from_cols_array).collect::<Vec<_>>();
                let posed = positions.iter().zip(ids).zip(weights).map(|((p,j),w)| {
                    let v = (0..4).map(|i| palette[j[i] as usize]
                        .transform_point3(Vec3::from_array(*p))*w[i]).sum::<Vec3>();
                    let bounds = cache.0[&(look.base,kind)].bounds;
                    assert!(v.is_finite()
                        && v.cmpge(bounds.0-Vec3::splat(0.02)).all()
                        && v.cmple(bounds.1+Vec3::splat(0.02)).all(),
                        "{} phase {step}: body vertex {v:?} escapes culling envelope {bounds:?}",
                        look.texture_key);
                    (v-Vec3::Y*rig.floor)*person.model_scale
                }).collect::<Vec<_>>();
                let bevy::mesh::Indices::U32(indices) = body.indices().unwrap() else { panic!("indices"); };
                for side in 0..2 {
                    let sign = if side == 0 { 1. } else { -1. };
                    let angle = phase + if side == 0 { 0. } else { std::f32::consts::PI };
                    let pad = Vec3::new(sign*0.11, 0.265+0.078*angle.sin(), -0.015+0.078*angle.cos());
                    let is_foot = |v: usize| (0..4).filter(|&i| ids[v][i] as usize == feet[side])
                        .map(|i| weights[v][i]).sum::<f32>() > 0.8;
                    let height = indices.chunks_exact(3).filter(|f| f.iter().all(|&v| is_foot(v as usize)))
                        .filter_map(|f| sole_height(posed[f[0] as usize], posed[f[1] as usize],
                            posed[f[2] as usize], pad.x, pad.z)).fold(f32::INFINITY, f32::min);
                    let gap = height-pad.y-0.016;
                    assert!(gap.is_finite() && gap.abs() < 0.0015,
                        "{} phase{step} side{side} actual shoe/pedal gap {gap}", look.texture_key);
                    let is_hand = |v: usize| (0..4).filter(|&i|
                        rig.hand_joints[side].contains(&(ids[v][i] as usize)))
                        .map(|i| weights[v][i]).sum::<f32>() > 0.8;
                    let mut grip_gap = f32::INFINITY;
                    let mut metal_gap = f32::INFINITY;
                    for face in indices.chunks_exact(3).filter(|f| f.iter().all(|&v| is_hand(v as usize))) {
                        let triangle = face.iter().map(|&v| {
                            let mut p = posed[v as usize]; p.x *= sign; p
                        }).collect::<Vec<_>>();
                        grip_gap = grip_gap.min(radial_distance(&clip(&clip(&triangle,0.19,1.),0.27,-1.))-0.023);
                        metal_gap = metal_gap.min(radial_distance(&clip(&clip(&triangle,0.,1.),0.19,-1.))-0.018);
                    }
                    assert!(grip_gap > -0.0015 && grip_gap < 0.003,
                        "{} phase{step} side{side} actual hand/grip gap {grip_gap}", look.texture_key);
                    assert!(metal_gap > -0.0015,
                        "{} phase{step} side{side} thumb intersects inner bar by {}", look.texture_key, -metal_gap);
                    let mut angles = posed.iter().enumerate().filter(|&(v,p)|
                        is_hand(v) && (0.19..=0.27).contains(&(p.x*sign)))
                        .map(|(_,p)| (p.y-0.72).atan2(p.z-0.25)).collect::<Vec<_>>();
                    angles.sort_by(f32::total_cmp);
                    assert!(!angles.is_empty());
                    let largest_gap = angles.windows(2).map(|p| p[1]-p[0])
                        .fold(angles[0]+std::f32::consts::TAU-angles[angles.len()-1],f32::max);
                    let grip_wrap = std::f32::consts::TAU-largest_gap;
                    assert!(grip_wrap > 2.9,
                        "{} phase{step} side{side} actual hand wrap {grip_wrap} radians; hand must curl around the grip rather than float beside it", look.texture_key);
                }
            }
        }
    }

    #[test]
    #[ignore = "requires supplied character rigs and paintings"]
    fn spawned_characters_start_with_production_pose_and_proportions() {
        use bevy::ecs::{system::SystemState, world::CommandQueue};
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
        let mut cache = PeopleGpu::default();
        let mut cases = crate::new_bark_actors::character_meshes::SOURCE_KINDS.iter()
            .map(|&(_,kind)| (kind,false)).collect::<Vec<_>>();
        cases.extend([(CharacterKind::Outdoorsman,false),
            (CharacterKind::Trainer,true),(CharacterKind::TrainerFemale,true)]);
        assert_eq!(cases.len(),77);
        for (kind,bicycle) in cases {
            let look = crate::npc_appearance::look(kind);
            let rig = crate::open_people::get(look.base).unwrap().clone();
            let mut commands = Commands::new(&mut queue,&world);
            let actor = commands.spawn((Transform::default(),Visibility::default())).id();
            let mut person = cache.spawn(look.base,kind,rig,actor,false,bicycle,
                &mut commands,&mut meshes,&mut materials,&mut images,&mut binds);
            queue.apply(&mut world);
            let initial = person.nodes.iter().map(|&node| *world.get::<Transform>(node).unwrap())
                .collect::<Vec<_>>();
            let mut state = SystemState::<Query<&mut Transform,With<ModeledActor>>>::new(&mut world);
            person.update(0.,0.,0.,&mut state.get_mut(&mut world).unwrap());
            for (&node,first) in person.nodes.iter().zip(initial) {
                let current = world.get::<Transform>(node).unwrap();
                assert!(first.translation.distance(current.translation)<1e-6
                    && first.scale.distance(current.scale)<1e-6
                    && first.rotation.abs_diff_eq(current.rotation,1e-5),
                    "{} mounted={bicycle}: first frame differs from settled production pose",look.texture_key);
            }
            world.entity_mut(actor).despawn();
        }
    }

    fn verify_costume_instance_animation(kind: CharacterKind) {
        use bevy::ecs::{system::SystemState, world::CommandQueue};
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let base = crate::npc_appearance::look(kind).base;
        let rig = Arc::new(PersonRig::parse(&std::fs::read(
            root.join(format!("people/kenney/{base}.glb"))).unwrap()).unwrap());
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
            people.push(cache.spawn(base,kind,rig.clone(),root,remote,false,
                &mut commands,&mut meshes,&mut materials,&mut images,&mut binds));
            queue.apply(&mut world);
        }
        let gpu = &cache.0[&(base,kind)];
        let cloth = gpu.costume.as_ref();
        let mut skins = world.query::<(&SkinnedMesh,&Mesh3d,&ChildOf)>();
        let bodies: Vec<_> = skins.iter(&world).filter(|(_,mesh,_)| mesh.0==gpu.mesh).collect();
        assert_eq!(bodies.len(),2);
        assert_ne!(bodies[0].0.joints,bodies[1].0.joints);
        if let Some(cloth) = cloth {
        let costumes: Vec<_> = skins.iter(&world).filter(|(_,mesh,_)| mesh.0==cloth.mesh).collect();
        assert_eq!(costumes.len(),2);
        assert_eq!(skins.iter(&world).count(),4,"body and garment skins for both actors");
        assert_eq!(costumes[0].0.inverse_bindposes,costumes[1].0.inverse_bindposes);
        assert_ne!(costumes[0].0.joints,costumes[1].0.joints,"instances may not share live joints");
        for (skin,_,parent) in costumes {
            assert_eq!(world.get::<Mesh3d>(parent.parent()).map(|mesh| &mesh.0),Some(&gpu.mesh));
            assert_eq!(skin.inverse_bindposes,gpu.binds);
        }
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
        let hands: Vec<_> = rig.hand_joints.iter().flatten().map(|&joint| {
            let node = rig.skin_nodes[joint];
            (node,*world.get::<Transform>(people[0].nodes[node]).unwrap())
        }).collect();
        // A second identical frame must not shrink again, move the feet, or
        // mutate the independently animated remote instance.
        people[0].update(std::f32::consts::FRAC_PI_2,1.0,0.0,&mut state.get_mut(&mut world).unwrap());
        assert_eq!(*world.get::<Transform>(people[0].nodes[head_node]).unwrap(), head);
        assert_eq!(foot_nodes.map(|node| *world.get::<Transform>(people[0].nodes[node]).unwrap()), feet);
        assert_eq!(*world.get::<Transform>(people[1].nodes[finger]).unwrap(),other_finger);
        for &(node,transform) in &hands {
            assert_eq!(*world.get::<Transform>(people[0].nodes[node]).unwrap(),transform,
                "hand proportions accumulated on a repeated animation frame");
        }
        let mut expected = rig.nodes.clone();
        rig.sample("Run",rig.duration("Run")*0.25,&mut expected);
        crate::hand_pose::relax(&rig,&mut expected);
        let head_scale = match kind {
            CharacterKind::PokefanF => 0.60,
            CharacterKind::Youngster | CharacterKind::StandingYoungster | CharacterKind::BugCatcher => 0.84,
            _ => HEAD_SCALE,
        };
        assert!(head.scale.distance(expected[head_node].scale * head_scale) < 1e-6);
        let hand_scale = if kind == CharacterKind::Blue { 0.70 } else { HAND_SCALE };
        for hand in &rig.hand_joints {
            let nodes: Vec<_> = hand.iter().map(|&j| rig.skin_nodes[j]).collect();
            let wrists: Vec<_> = nodes.iter().copied().filter(|&node|
                rig.parents[node].is_none_or(|parent| !nodes.contains(&parent))).collect();
            assert_eq!(wrists.len(),1,"supplied hand must have one wrist root");
            let node = wrists[0];
            let actual = world.get::<Transform>(people[0].nodes[node]).unwrap();
            assert!(actual.scale.distance(expected[node].scale * hand_scale)<1e-6);
        }
        for &node in &rig.finger_nodes {
            let actual = world.get::<Transform>(people[0].nodes[node]).unwrap();
            assert!(actual.rotation.angle_between(expected[node].rotation)<0.001,
                "visible instance skipped relaxed finger pose");
            // The production Idle/Run blend introduces float rounding even
            // at weight 1; compare within floating point bind-space precision.
            assert!(actual.translation.distance(expected[node].translation)<1e-6);
            assert!(actual.scale.distance(expected[node].scale)<1e-6,
                "finger descendant must inherit wrist scale only once");
        }
        assert_eq!(counts,(meshes.len(),materials.len(),images.len(),binds.len()));
        if let Some(cloth) = cloth {
        let ghost = materials.get(&cloth.remote).unwrap();
        assert_eq!(ghost.alpha_mode,AlphaMode::Blend);
        assert!((ghost.base_color.alpha()-0.62).abs()<0.001);
        }
        let mut commands = Commands::new(&mut queue,&world);
        for root in roots { commands.entity(root).despawn(); }
        queue.apply(&mut world);
        assert_eq!(world.query::<&SkinnedMesh>().iter(&world).count(),0);
    }
}
