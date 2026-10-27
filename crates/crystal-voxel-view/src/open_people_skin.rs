//! GPU resources and instances for the external people reader.
#[cfg(test)]
use bevy::asset::RenderAssetUsages;
use crate::{new_bark_actors::ModeledActor, open_people::PersonRig, VOXEL_RENDER_LAYER};
use bevy::camera::{primitives::Aabb, visibility::{NoAutoAabb, RenderLayers}};
use bevy::{
    light::NotShadowReceiver,
    prelude::*,
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            VertexAttributeValues,
        },



    },
};
use std::{collections::HashMap, sync::Arc};
use crate::new_bark_actors::character_meshes::CharacterKind;

const HEAD_SCALE: f32 = 0.72;
const HAND_SCALE: f32 = 0.82;

#[derive(Default)]
pub(crate) struct PeopleGpu(
    pub HashMap<(&'static str, CharacterKind), Gpu>,
    HashMap<&'static str, Handle<Image>>,
    pub PreparedAppearances,
    HashMap<(&'static str, CharacterKind, [u8; 3]), Gpu>,
);
#[derive(Clone)]
pub(crate) struct Gpu {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    remote: Handle<StandardMaterial>,
    binds: Handle<SkinnedMeshInverseBindposes>,
    source_binds: Handle<SkinnedMeshInverseBindposes>,
    arm_deformation: Option<Arc<crate::arm_deformation::Recipe>>,
    shoe_support: Arc<crate::human_locomotion::ShoeSupport>,
    bounds: (Vec3,Vec3),
    head: Option<AccessoryGpu>,
    costume: Option<AccessoryGpu>,
    backpack: Option<AccessoryGpu>,
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    live_drape: Option<(Arc<crate::garment_drape_trial::live::Template>, Handle<SkinnedMeshInverseBindposes>)>,
}
#[derive(Clone)]
struct AccessoryGpu {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    remote: Handle<StandardMaterial>,
}
#[cfg(feature = "actor-worker")]
#[path = "actor_worker/appearance_wire.rs"]
pub(crate) mod worker_wire;

/// CPU-only final appearance. No asset handles or ECS state cross this boundary.
pub(crate) struct PreparedAppearance {
    mesh: Mesh,
    arm_deformation: Option<Arc<crate::arm_deformation::Recipe>>,
    bounds: (Vec3, Vec3),
    head: Option<Mesh>,
    costume: Option<Mesh>,
    backpack: Option<Mesh>,
}
#[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
impl PreparedAppearance {
    pub(crate) fn review_body(&self) -> &Mesh { &self.mesh }
    pub(crate) fn review_arm_deformation(&self) -> Option<&crate::arm_deformation::Recipe> {
        self.arm_deformation.as_deref()
    }
    pub(crate) fn review_drape_meshes(&mut self) -> (&Mesh, &mut Mesh) {
        (&self.mesh, self.costume.as_mut().expect("review garment mesh"))
    }
}
pub(crate) type PreparedAppearances = Arc<std::sync::Mutex<HashMap<(&'static str, CharacterKind), PreparedAppearance>>>;

pub(crate) fn prepare_appearance(rig: &PersonRig, kind: CharacterKind) -> PreparedAppearance {
    let _trace = crate::loading_trace::span("actor.prepare_appearance", || serde_json::json!({"kind":format!("{kind:?}")}));
    let look = crate::npc_appearance::look(kind);
    let crate::open_people::RefinedBody {
        mesh: mut body, ids: mut body_joints, weights: mut body_weights,
    } = rig.refined_body();
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
    if matches!(kind, CharacterKind::Kurt | CharacterKind::KurtOutside) {
        // Keep Kurt's drawn eyes, nose, mouth and forehead in place.
        // Extra refinement is restricted to the established jaw/nape
        // selection, with pinned crown and collar boundaries.
        for _ in 0..2 {
            crate::head_surface::smooth(&rig, &mut body, &mut body_joints, &mut body_weights);
        }
        crate::head_surface::nape_normals(&rig, &mut body, &body_joints, &body_weights);
    }
    let face_frame=crate::face_volume::Frame::new(&rig,&body,&body_joints,&body_weights);
    let hair_extension=crate::hair_shapes::sculpt(&rig,&mut body,&mut body_joints,&mut body_weights,look);
    crate::face_volume::sculpt(&rig,look,face_frame,&mut body,&mut body_joints,&mut body_weights);
    crate::npc_appearance::shorts_body(&rig,&mut body,&mut body_joints,&mut body_weights,look);
    crate::kimono_sleeves::sculpt(&mut body,&mut body_joints,&mut body_weights,look);
    crate::garment_body::cover(&mut body,&mut body_joints,&mut body_weights,look);
    crate::garment_body::nurse_legs(&mut body,&mut body_joints,&mut body_weights,look);
    crate::npc_appearance::correct_body_uvs(&mut body, look, &mut body_joints, &mut body_weights);
    crate::painted_torso::avoid_shirt_gutter(&mut body, look, &mut body_joints, &mut body_weights);
    crate::painted_torso::map_front(&mut body, look, &mut body_joints, &mut body_weights);
    crate::painted_torso::long_sleeves(&mut body, &mut body_joints, &mut body_weights, look);
    if crate::painted_torso::has_short_sleeves(look) {
        crate::painted_torso::short_sleeves(&mut body, &mut body_joints, &mut body_weights, look);
    }
    crate::bare_body::sculpt(&rig, &mut body, &mut body_joints, &mut body_weights, look);
    crate::footwear::adapt(&mut body,&mut body_joints,&mut body_weights,look);
    crate::handwear::adapt(&rig, &mut body, &mut body_joints, &mut body_weights, look);
    crate::arm_surface::round(&rig, &mut body, &mut body_joints, &mut body_weights);
    let mut arm_reference=body.positions.clone();
    crate::arm_surface::root_shape::finish(&rig,&mut body,&body_joints,&body_weights);
    #[cfg(all(feature="people-review",not(target_arch="wasm32")))]
    let late_normals=std::env::var_os("CRYSTAL_PEOPLE_REVIEW_LATE_ARM_NORMALS").is_some();
    #[cfg(not(all(feature="people-review",not(target_arch="wasm32"))))]
    let late_normals=false;
    if !late_normals {
        // Finish the smooth field on the curved surface, then interpolate it
        // through collinear splits. Rebuilding normals on planar subfaces
        // hardens the original cage's diagonal fans into visible facets.
        crate::arm_surface::finish_normals(&rig,&mut body,&body_joints,&body_weights);
        crate::bare_body::finish_normals(&mut body, look);
        crate::kimono_sleeves::finish_normals(&mut body,look);
    }
    crate::arm_skin::refine_reference(&rig, &mut body, &mut body_joints, &mut body_weights,&mut arm_reference);
    drop(arm_reference);
    if late_normals {
        crate::arm_surface::finish_normals(&rig,&mut body,&body_joints,&body_weights);
        crate::bare_body::finish_normals(&mut body, look);
        crate::kimono_sleeves::finish_normals(&mut body,look);
    }
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    if let Some(directory) = std::env::var_os("CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("create arm input review directory");
        std::fs::write(directory.join(format!("{}.unfitted.json", look.texture_key)),
            serde_json::to_vec(&serde_json::json!({
                "positions":body.positions,"normals":body.normals,"uvs":body.uvs,
                "colors":body.colors,"cutaway_ranges":body.cutaway_ranges,
                "indices":body.indices,"joints":body_joints,"weights":body_weights,
            })).expect("encode arm fitting inputs"))
            .expect("write arm fitting inputs");
    }
    crate::arm_skin::fit(&rig, &body, &mut body_joints, &mut body_weights);
    crate::painted_torso::cuffs::finish(&mut body, &mut body_joints, &mut body_weights, look);
    crate::people_mesh::compact(&mut body,&mut body_joints,&mut body_weights);
    let hair_bounds = crate::hair_curtain::append(
        rig,
        kind,
        HEAD_SCALE,
        &mut body,
        &mut body_joints,
        &mut body_weights,
    );
    // Walking and mounted instances share GPU resources. Include both
    // envelopes even when the first instance happens to be on foot.
    let locomotion_bounds = rig.locomotion_bounds();
    let mut bounds = (locomotion_bounds.0-Vec3::splat(hair_extension),
        locomotion_bounds.1+Vec3::splat(hair_extension));
    if let Some(hair) = hair_bounds {
        bounds.0 = bounds.0.min(hair.min);
        bounds.1 = bounds.1.max(hair.max);
    }
    if matches!(kind, CharacterKind::Trainer | CharacterKind::TrainerFemale | CharacterKind::Biker) {
        let scale = Vec3::new(look.height / rig.height * look.width,
            look.height / rig.height, look.height / rig.height);
        let mounted = crate::bicycle::body_bounds(&rig, &body.positions,
            &body_joints, &body_weights, scale, HEAD_SCALE, HAND_SCALE);
        bounds.0 = bounds.0.min(mounted.0);
        bounds.1 = bounds.1.max(mounted.1);
    }
    // Build the sparse blend palette off the interactive frame thread, with
    // the rest of the appearance. Worker transport retains the same recipe.
    #[allow(unused_mut)]
    let mut arm_blends = true;
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_ARM_MATRIX_SKIN").is_some()
        || std::env::var_os("CRYSTAL_PEOPLE_REVIEW_DQ_SKIN").is_some() {
        arm_blends = false;
    }
    let arm_deformation = arm_blends.then(|| Arc::new(crate::arm_deformation::Recipe::prepare(
        rig, &body, &mut body_joints, &mut body_weights)));
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    if let Some(directory) = std::env::var_os("CRYSTAL_PEOPLE_REVIEW_EXPORT_MESHES") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("create mesh review directory");
        let snapshot = serde_json::json!({
            "positions": body.positions, "normals": body.normals, "uvs": body.uvs,
            "colors":body.colors,"cutaway_ranges":body.cutaway_ranges,
            "indices": body.indices, "joints": body_joints, "weights": body_weights,
            "arm_deformation": arm_deformation.as_deref(),
        });
        std::fs::write(directory.join(format!("{}.json", look.texture_key)),
            serde_json::to_vec(&snapshot).expect("encode review mesh")).expect("write review mesh");
    }
    let head = crate::npc_appearance::head_mesh_on_body(&rig,look,&body).map(|mut mesh| {
        // Falkner's fringe is already fitted against this prepared surface.
        if look.texture_key!="falkner" {crate::face_volume::attachment(&rig,look,face_frame,&mut mesh);}
        mesh.into_mesh()
    });
    let mut mesh = body.into_mesh();
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(body_joints),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, body_weights);
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    if crate::garment_drape_trial::local_body()
        || std::env::var_os("CRYSTAL_PEOPLE_REVIEW_DQ_SKIN").is_some() {
        mesh.asset_usage |= bevy::asset::RenderAssetUsages::MAIN_WORLD;
    }
        let costume = crate::npc_appearance::costume_mesh(&rig, look)
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
                #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
                if std::env::var_os("CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL").is_some() {
                    // Frozen art trials edit after the actual pose is sampled.
                    // Ordinary clients keep their render-only mesh allocation.
                    mesh.asset_usage |= bevy::asset::RenderAssetUsages::MAIN_WORLD;
                }
                mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX,
                    VertexAttributeValues::Uint16x4(costume.joints));
                mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, costume.weights);
                mesh
            });
        let backpack = crate::npc_appearance::backpack_mesh(&rig, look).map(|mesh| mesh.into_mesh());
    PreparedAppearance { mesh, arm_deformation, bounds, head, costume, backpack }
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
    arm_skin: Option<ArmInstance>,
    pub run_blend: f32,
    locomotion: crate::human_locomotion::HumanLocomotion,
    idle_motion: crate::human_idle::HumanIdle,
    pub bicycle: bool,
    pub surfing: bool,
    pub surf_half_width: f32,
    /// Accepted step progress and airborne blend; root lift belongs to terrain presentation.
    pub hop_step: Option<(f32, f32)>,
    hop: Vec<Transform>,
    pub sitting: crate::seating::Sitting,
    seated: Vec<Transform>,
    pub rig: Arc<PersonRig>,
    model_scale: Vec3,
    head_scale: f32,
    hand_scale: f32,
    nodes: Vec<Entity>,
    wrist_nodes: Vec<usize>,
    idle: Vec<Transform>,
    run: Vec<Transform>,
    posed: Vec<Transform>,
}
struct ArmInstance {
    recipe: Arc<crate::arm_deformation::Recipe>,
    nodes: Vec<Entity>,
    posed: Vec<Transform>,
}
impl PeopleGpu {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    pub(crate) fn review_frozen_arm_skin(&self, kind: CharacterKind, rig: &PersonRig,
        palette: &[Mat4], meshes: &mut Assets<Mesh>) {
        let gpu = self.0.get(&(crate::npc_appearance::look(kind).base, kind)).unwrap();
        crate::arm_deformation::freeze(rig,&mut meshes.get_mut(&gpu.mesh).expect("retained DQ preview body"),palette);
    }
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    pub(crate) fn review_frozen_drape(&self, kind: CharacterKind, rig: &PersonRig,
        palette: &[Mat4], meshes: &mut Assets<Mesh>) {
        let look = crate::npc_appearance::look(kind);
        if crate::garment_body::hem(look).is_none() || look.costume == "hakama" { return; }
        let gpu = self.0.get(&(look.base, kind)).unwrap();
        let costume = gpu.costume.as_ref().unwrap();
        let body = (crate::garment_drape_trial::local_body())
            .then(|| crate::garment_drape_trial::body_collision(meshes.get(&gpu.mesh)
                .expect("local cloth trial retains the final body"), rig, palette));
        crate::garment_drape_trial::freeze(look.texture_key, rig,
            &mut meshes.get_mut(&costume.mesh).unwrap(), palette, body);
    }
    pub(super) fn retain_kinds(&mut self, kinds: &std::collections::HashSet<CharacterKind>) {
        self.0.retain(|(_, kind), _| kinds.contains(kind));
        self.3.retain(|(_, kind, _), _| kinds.contains(kind));
        let bodies = &self.0;
        self.1.retain(|texture, _| bodies.keys().any(|(_, kind)|
            crate::npc_appearance::look(*kind).texture_key == *texture));
    }
    /// Register exactly the same appearance assets as spawn, without entities.
    #[allow(clippy::too_many_arguments)]
    fn upload(&mut self, name: &'static str, kind: CharacterKind, rig: &PersonRig,
        meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>, binds: &mut Assets<SkinnedMeshInverseBindposes>) {
        let prepared = &self.2;
        let textures = &mut self.1;
        self.0.entry((name, kind)).or_insert_with(|| {
            let _trace = crate::loading_trace::span("actor.build_body", || serde_json::json!({
                "rig":name,"kind":format!("{kind:?}")}));
            let look = crate::npc_appearance::look(kind);
            let painting = crate::npc_textures::get(look.texture_key)
                .expect("required production character painting");
            let staged = prepared.lock().expect("prepared actor lock").remove(&(name, kind));
            let PreparedAppearance { mesh, arm_deformation, bounds, head, costume, backpack } =
                staged.unwrap_or_else(|| prepare_appearance(&rig, kind));
            let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!("prepared feet require positions"); };
            let Some(VertexAttributeValues::Uint16x4(ids)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) else { panic!("prepared feet require skin indices"); };
            let Some(VertexAttributeValues::Float32x4(weights)) = mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT) else { panic!("prepared feet require skin weights"); };
            let shoe_support = Arc::new(crate::human_locomotion::ShoeSupport::from_surface(
                rig, positions, ids, weights));
            // The validated painting catalog is immutable. Different body
            // rigs can share its exact pixels without sharing their geometry.
            let texture = textures.entry(look.texture_key).or_insert_with(|| {
                images.add(crate::texture_filtering::painted_image(
                    painting.size, painting.rgba.clone()))
            }).clone();
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
            let mut costume = costume;
            #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
            let live_drape = (crate::garment_drape_trial::live::enabled()
                && crate::garment_body::hem(look).is_some()).then(|| {
                let template = Arc::new(crate::garment_drape_trial::live::Template::new(
                    look, rig, costume.as_mut().expect("live garment mesh")));
                let palette = binds.add(template.binds(rig));
                (template, palette)
            });
            let source_binds=binds.add(SkinnedMeshInverseBindposes::from(rig.inverse_binds.clone()));
            let body_binds=if let Some(recipe)=&arm_deformation {
                assert_eq!(recipe.original_count(),rig.skin_nodes.len());
                let mut palette=rig.inverse_binds.clone();
                palette.resize(palette.len()+recipe.helper_count(),Mat4::IDENTITY);
                binds.add(SkinnedMeshInverseBindposes::from(palette))
            } else {source_binds.clone()};
            Gpu {
                bounds,
                mesh: meshes.add(mesh),
                material: materials.add(material),
                remote: materials.add(ghost),
                binds: body_binds,
                source_binds,
                arm_deformation,
                shoe_support,
                head: head.map(|mesh| accessory_mesh_gpu(mesh, meshes, materials)),
                costume: costume.map(|mesh| accessory_mesh_gpu(mesh, meshes, materials)),
                backpack: backpack.map(|mesh| accessory_mesh_gpu(mesh, meshes, materials)),
                #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
                live_drape,
            }
        });
    }

    /// Publish at most one already-prepared appearance per host frame. Never
    /// fall back to body refinement on the interactive frame thread.
    pub(super) fn upload_next_prepared(&mut self, kinds: &[CharacterKind],
        meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>, binds: &mut Assets<SkinnedMeshInverseBindposes>) -> bool {
        for &kind in kinds {
            let name = crate::npc_appearance::look(kind).base;
            if self.0.contains_key(&(name, kind)) { continue; }
            if !self.2.lock().expect("prepared actor lock").contains_key(&(name, kind)) {
                return false;
            }
            let rig = crate::open_people::get(name).expect("prepared character rig");
            self.upload(name, kind, &rig, meshes, materials, images, binds);
            return kinds.iter().all(|&kind| self.0.contains_key(&(crate::npc_appearance::look(kind).base, kind)));
        }
        true
    }

    pub(super) fn adopt_uploaded_kinds(&mut self, other: &Self, kinds:&[CharacterKind]) {
        for (&key, gpu) in &other.0 {
            if kinds.contains(&key.1) {self.0.entry(key).or_insert_with(||gpu.clone());}
        }
        for (&key, texture) in &other.1 {
            if kinds.iter().any(|kind|crate::npc_appearance::look(*kind).texture_key==key) {
                self.1.entry(key).or_insert_with(||texture.clone());
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        &mut self,
        name: &'static str,
        kind: CharacterKind,
        rig: Arc<PersonRig>,
        root: Entity,
        remote: bool,
        bicycle: bool,
        uniform: Option<[u8; 3]>,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
        binds: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> Person {
        self.spawn_on_layer(name, kind, rig, root, remote, bicycle, uniform,
            VOXEL_RENDER_LAYER, commands, meshes, materials, images, binds)
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn spawn_on_layer(
        &mut self,
        name: &'static str,
        kind: CharacterKind,
        rig: Arc<PersonRig>,
        root: Entity,
        remote: bool,
        bicycle: bool,
        uniform: Option<[u8; 3]>,
        render_layer: usize,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
        binds: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> Person {
        self.upload(name, kind, &rig, meshes, materials, images, binds);
        let gpu = self.0.get(&(name, kind)).expect("uploaded appearance");
        let gpu = if let Some(color) = uniform.filter(|_| crate::npc_uniform::supported(kind)) {
            self.3.entry((name, kind, color)).or_insert_with(|| {
                let mut variant = gpu.clone();
                let mut look = crate::npc_appearance::look(kind);
                let painting = crate::npc_textures::get(look.texture_key).expect("validated character painting");
                let texture = images.add(crate::texture_filtering::painted_image(painting.size,
                    crate::npc_uniform::painting_for(kind, &painting.rgba, painting.size, color)));
                let mut material = materials.get(&gpu.material).expect("uploaded person material").clone();
                material.base_color_texture = Some(texture.clone());
                variant.material = materials.add(material);
                let mut remote = materials.get(&gpu.remote).expect("uploaded remote material").clone();
                remote.base_color_texture = Some(texture);
                variant.remote = materials.add(remote);
                look.shirt = color;
                if crate::npc_uniform::lower_follows_source(kind) { look.lower = color; }
                if kind == CharacterKind::CooltrainerF { look.hair = color; }
                if matches!(kind,CharacterKind::Fisher|CharacterKind::CooltrainerF|CharacterKind::Officer) {
                    variant.head = crate::npc_appearance::head_mesh_with_hat_color(&rig,look,
                        matches!(kind,CharacterKind::Fisher|CharacterKind::Officer).then_some(color))
                        .map(|mesh|accessory_mesh_gpu(mesh.into_mesh(),meshes,materials));
                }
                variant.costume = crate::npc_appearance::costume_mesh_with_uniform(&rig, look, color).map(|costume| {
                    let mut mesh = costume.surface.into_mesh();
                    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX, VertexAttributeValues::Uint16x4(costume.joints));
                    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, costume.weights);
                    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
                    if crate::garment_drape_trial::live::enabled() && crate::garment_body::hem(look).is_some() {
                        let template = Arc::new(crate::garment_drape_trial::live::Template::new(look, &rig, &mut mesh));
                        variant.live_drape = Some((template.clone(), binds.add(template.binds(&rig))));
                    }
                    accessory_mesh_gpu(mesh, meshes, materials)
                });
                variant
            })
        } else { gpu };
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
        let locomotion = crate::human_locomotion::HumanLocomotion::with_support(
            &rig, kind, gpu.shoe_support.clone());
        let idle_motion = crate::human_idle::HumanIdle::new(&rig, kind);
        let mut idle = rig.nodes.clone();
        idle_motion.sample(&rig, &locomotion, 0., &mut idle);
        if bicycle {
            rig.sample("Idle", 0., &mut idle);
            crate::bicycle::pose(&rig, &mut idle, 0., model_scale);
        }
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
        let arm_skin=gpu.arm_deformation.as_ref().map(|recipe| {
            let mut posed=Vec::with_capacity(recipe.helper_count());
            recipe.update(&rig,&idle,&mut posed);
            let helpers: Vec<_>=posed.iter().map(|pose| {
                let entity=commands.spawn((Visibility::default(),*pose,ModeledActor)).id();
                commands.entity(nodes[rig.mesh_node]).add_child(entity);
                entity
            }).collect();
            ArmInstance{recipe:recipe.clone(),nodes:helpers,posed}
        });
        let mut skin_joints: Vec<_>=rig.skin_nodes.iter().map(|&i|nodes[i]).collect();
        if let Some(arm)=&arm_skin {skin_joints.extend_from_slice(&arm.nodes);}
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
                joints: skin_joints,
            },
            Aabb::from_min_max(
                gpu.bounds.0 - Vec3::splat(0.02),
                gpu.bounds.1 + Vec3::splat(0.02),
            ),
            NoAutoAabb,
            RenderLayers::layer(render_layer),
        ));
        if let Some(gpu) = &gpu.head {
            let accessory = commands.spawn(((Mesh3d(gpu.mesh.clone()),
MeshMaterial3d(if remote { gpu.remote.clone() } else { gpu.material.clone() }),), ModeledActor, NotShadowReceiver, RenderLayers::layer(render_layer))).id();
            commands.entity(nodes[rig.skin_nodes[rig.head_joint]]).add_child(accessory);
        }
        if let Some(pack) = &gpu.backpack {
            let accessory = commands.spawn(((Mesh3d(pack.mesh.clone()),
MeshMaterial3d(if remote { pack.remote.clone() } else { pack.material.clone() }),), ModeledActor, NotShadowReceiver, RenderLayers::layer(render_layer))).id();
            commands.entity(nodes[rig.skin_nodes[rig.chest_joint]]).add_child(accessory);
        }
        if let Some(costume) = &gpu.costume {
            let accessory = commands.spawn(((Mesh3d(costume.mesh.clone()),
MeshMaterial3d(if remote { costume.remote.clone() } else { costume.material.clone() }),), SkinnedMesh {
                inverse_bindposes: gpu.source_binds.clone(),
                joints: rig.skin_nodes.iter().map(|&i| nodes[i]).collect(),
            }, Aabb::from_min_max(rig.bounds.0-Vec3::splat(0.6), rig.bounds.1+Vec3::splat(0.6)), NoAutoAabb,
                ModeledActor, NotShadowReceiver, RenderLayers::layer(render_layer))).id();
            commands.entity(nodes[rig.mesh_node]).add_child(accessory);
            #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
            if let Some((template, palette)) = &gpu.live_drape {
                let (skin, instance) = crate::garment_drape_trial::live::spawn(template.clone(), rig.clone(),
                    gpu.mesh.clone(), palette.clone(), rig.skin_nodes.iter().map(|&i| nodes[i]).collect(),
                    accessory, commands);
                commands.entity(accessory).insert((skin, instance));
            }
        }
        Person {
            arm_skin,
            run_blend: 0.0,
            locomotion,
            idle_motion,
            bicycle,
            surfing: false,
            surf_half_width: 0.25,
            hop_step: None,
            hop: Vec::new(),
            sitting: Default::default(),
            seated: idle.clone(),
            model_scale,
            head_scale,
            hand_scale,
            wrist_nodes,
            run: idle.clone(),
            posed: idle.clone(),
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
        let mut palette: Vec<_>=self.rig.skin_nodes.iter().zip(&self.rig.inverse_binds)
            .map(|(&node, bind)| (inverse * world(node, &self.rig, &self.nodes, transforms)
                * *bind).to_cols_array()).collect();
        if let Some(arm)=&self.arm_skin {
            palette.extend(arm.nodes.iter().map(|&node|transforms.get(node)
                .expect("review arm blend transform").to_matrix().to_cols_array()));
        }
        palette
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
        if self.bicycle || self.surfing {
            self.rig.sample("Idle", seconds.rem_euclid(self.rig.duration("Idle")), &mut self.idle);
        } else {
            self.idle_motion.sample(&self.rig, &self.locomotion, seconds, &mut self.idle);
        }
        // A fully idle or mounted actor never consumes a locomotion pose.
        // Keep idle breathing live without sampling an unused walking rig.
        let walking = weight != 0.0 && !self.bicycle && !self.surfing;
        if walking {
            self.locomotion.sample(&self.rig, phase, self.run_blend, &mut self.run);
        }
        if let Some((progress, _)) = self.hop_step {
            self.hop.resize(self.idle.len(), Transform::IDENTITY);
            self.rig.sample("Jump", progress.clamp(0.0, 1.0) * self.rig.duration("Jump"), &mut self.hop);
            crate::hand_pose::relax(&self.rig, &mut self.hop);
        }
        if self.surfing && !self.bicycle {
            crate::hand_pose::relax(&self.rig, &mut self.idle);
        }
        if self.surfing {
            crate::bicycle::surf_pose(&self.rig, &mut self.idle, self.model_scale, self.surf_half_width);
            self.run.clone_from(&self.idle);
        } else if self.bicycle {
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
        let sitting = if self.bicycle || self.surfing { 0. } else { self.sitting.eased_weight() };
        if sitting > 0. {
            self.rig.sample("Idle", seconds.rem_euclid(self.rig.duration("Idle")), &mut self.seated);
            crate::hand_pose::relax(&self.rig, &mut self.seated);
            crate::bicycle::seat_pose(&self.rig, &mut self.seated, self.model_scale, self.sitting.height);
        }
        for (index, ((t, a), b)) in self.posed.iter_mut().zip(&self.idle).zip(&self.run).enumerate() {
            *t = if weight == 0.0 {
                Transform { rotation: a.rotation.normalize(), ..*a }
            } else {
                Transform {
                    translation: a.translation.lerp(b.translation, weight),
                    rotation: a.rotation.slerp(b.rotation, weight).normalize(),
                    scale: a.scale.lerp(b.scale, weight),
                }
            };
            if let Some((_, hop_weight)) = self.hop_step {
                // Reuse authored limb articulation without importing its
                // root translations on top of the terrain hop arc.
                t.rotation = t.rotation.slerp(self.hop[index].rotation, hop_weight.clamp(0.0, 1.0)).normalize();
            }
            if sitting > 0. {
                let seated = &self.seated[index];
                t.translation = t.translation.lerp(seated.translation, sitting);
                t.rotation = t.rotation.slerp(seated.rotation, sitting).normalize();
                t.scale = t.scale.lerp(seated.scale, sitting);
            }
        }
        if weight > 0. && !self.bicycle && !self.surfing && self.hop_step.is_none() && sitting == 0. {
            self.locomotion.plant_blend(&self.rig, &self.idle, &self.run, weight, &mut self.posed);
        }
        // Compose proportions before publishing joints. Resetting and scaling
        // the same joints separately marked every unchanged finger/head dirty,
        // forcing transform propagation and skin extraction each frame.
        let head = self.rig.skin_nodes[self.rig.head_joint];
        for (index, (&entity, pose)) in self.nodes.iter().zip(&self.posed).enumerate() {
            let mut pose = *pose;
            if index == head { pose.scale *= self.head_scale; }
            for &wrist in &self.wrist_nodes {
                if index == wrist { pose.scale *= self.hand_scale; }
            }
            if let Ok(mut t) = transforms.get_mut(entity) { t.set_if_neq(pose); }
        }
        if let Some(arm)=&mut self.arm_skin {
            arm.recipe.update(&self.rig,&self.posed,&mut arm.posed);
            for (&entity,&pose) in arm.nodes.iter().zip(&arm.posed) {
                if let Ok(mut transform)=transforms.get_mut(entity) {transform.set_if_neq(pose);}
            }
        }
    }
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) mod tests {
    use super::*;
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    pub(crate) fn validate_instances() {
        spawned_characters_start_with_production_pose_and_proportions();
        costume_instances_share_gpu_assets_and_own_their_animated_joint_palettes();
        mounted_production_hands_and_shoes_fit_grips_and_pedals_through_full_cycle();
    }
    #[test]
    fn destination_upload_is_incremental_and_spawn_reuses_every_asset_handle() {
        use bevy::ecs::world::CommandQueue;
        let kinds = [CharacterKind::Elm, CharacterKind::Scientist];
        let mut staged = PeopleGpu::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
        assert!(!staged.upload_next_prepared(&kinds, &mut meshes, &mut materials, &mut images, &mut binds));
        assert_eq!(meshes.len(), 0, "no synchronous refinement fallback");
        for kind in kinds {
            let base = crate::npc_appearance::look(kind).base;
            let rig = crate::open_people::get(base).expect("character rig");
            staged.2.lock().unwrap().insert((base, kind), prepare_appearance(&rig, kind));
        }
        assert!(!staged.upload_next_prepared(&kinds, &mut meshes, &mut materials, &mut images, &mut binds));
        assert_eq!(staged.0.len(), 1, "one appearance per frame");
        assert!(staged.upload_next_prepared(&kinds, &mut meshes, &mut materials, &mut images, &mut binds));
        assert_eq!(staged.0.len(), 2);
        assert!(staged.2.lock().unwrap().is_empty());
        let mut live = PeopleGpu::default();
        live.adopt_uploaded_kinds(&staged,&kinds);
        let counts = (meshes.len(), materials.len(), images.len(), binds.len());
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        for kind in kinds {
            let base = crate::npc_appearance::look(kind).base;
            let rig = crate::open_people::get(base).unwrap();
            let mut commands = Commands::new(&mut queue, &world);
            let root = commands.spawn_empty().id();
            live.spawn(base, kind, rig.clone(), root, false, false, None, &mut commands,
                &mut meshes, &mut materials, &mut images, &mut binds);
            let a = &live.0[&(base, kind)];
            let b = &staged.0[&(base, kind)];
            assert_eq!((a.mesh.id(), a.material.id(), a.remote.id(), a.binds.id()),
                (b.mesh.id(), b.material.id(), b.remote.id(), b.binds.id()));
        }
        queue.apply(&mut world);
        assert_eq!(counts, (meshes.len(), materials.len(), images.len(), binds.len()),
            "arrival must only instantiate joints and reuse uploaded assets");
    }

    /// Both routes must publish identical vertices, index widths, joints,
    /// accessories, bounds and material/image ordering. The staged route uses
    /// an actual worker task and consumes its owned CPU result exactly once.
    #[test]
    #[ignore = "requires supplied character rigs and paintings"]
    fn background_appearance_is_byte_exact_with_direct_publication() {
        use bevy::ecs::world::CommandQueue;
        fn mesh_bits(mesh: &Mesh) -> Vec<u8> {
            let mut bytes = format!("{:?}{:?}",mesh.primitive_topology(),mesh.indices()).into_bytes();
            for (attribute, values) in mesh.attributes() {
                bytes.extend_from_slice(format!("{:?}{:?}",attribute.id,attribute.format).as_bytes());
                bytes.extend_from_slice(values.get_bytes());
            }
            bytes
        }
        let pool = bevy::tasks::TaskPoolBuilder::new().num_threads(2).build();
        for kind in [CharacterKind::Trainer, CharacterKind::TrainerFemale,
            CharacterKind::Nurse, CharacterKind::Kurt, CharacterKind::KimonoGirl] {
            let look = crate::npc_appearance::look(kind);
            let rig = crate::open_people::get(look.base).expect("supplied rig").clone();
            let worker_rig = rig.clone();
            let prepared = bevy::tasks::futures_lite::future::block_on(pool.spawn(async move {
                prepare_appearance(&worker_rig,kind)
            }));
            let mut results = Vec::new();
            for staged in [None,Some(prepared)] {
                let mut cache = PeopleGpu::default();
                if let Some(prepared) = staged {
                    cache.2.lock().unwrap().insert((look.base,kind),prepared);
                }
                let world = World::new();
                let mut queue = CommandQueue::default();
                let mut commands = Commands::new(&mut queue,&world);
                let root = commands.spawn_empty().id();
                let mut meshes = Assets::<Mesh>::default();
                let mut materials = Assets::<StandardMaterial>::default();
                let mut images = Assets::<Image>::default();
                let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
                cache.spawn(look.base,kind,rig.clone(),root,false,true,None,&mut commands,
                    &mut meshes,&mut materials,&mut images,&mut binds);
                assert!(cache.2.lock().unwrap().is_empty(),"staging consumed");
                let gpu = &cache.0[&(look.base,kind)];
                let mesh_handles = [Some(&gpu.mesh),gpu.head.as_ref().map(|a| &a.mesh),
                    gpu.costume.as_ref().map(|a| &a.mesh),gpu.backpack.as_ref().map(|a| &a.mesh)];
                let geometry = mesh_handles.map(|handle|handle.map(|h|mesh_bits(meshes.get(h).unwrap())));
                let bounds = [gpu.bounds.0.to_array().map(f32::to_bits),gpu.bounds.1.to_array().map(f32::to_bits)];
                let painting = images.iter().map(|(_,image)|image.data.clone()).collect::<Vec<_>>();
                results.push((geometry,bounds,painting,meshes.len(),materials.len(),binds.len()));
            }
            assert_eq!(results[0],results[1],"{kind:?}");
        }
    }
    #[test]
    #[ignore = "requires supplied character rigs and paintings"]
    fn different_body_rigs_share_exact_painting_upload_without_sharing_meshes() {
        use bevy::ecs::world::CommandQueue;
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
        let mut cache = PeopleGpu::default();
        let kind = CharacterKind::Trainer;
        for name in ["human-male", "skater-male"] {
            let rig = crate::open_people::get(name).expect("supplied rig").clone();
            let mut commands = Commands::new(&mut queue, &world);
            let actor = commands.spawn((Transform::default(), Visibility::default())).id();
            cache.spawn(name, kind, rig, actor, false, false, None, &mut commands,
                &mut meshes, &mut materials, &mut images, &mut binds);
        }
        let first = &cache.0[&("human-male", kind)];
        let second = &cache.0[&("skater-male", kind)];
        assert_ne!(first.mesh, second.mesh);
        assert_ne!(first.binds, second.binds);
        let texture = materials.get(&first.material).unwrap().base_color_texture.as_ref().unwrap();
        assert_eq!(Some(texture), materials.get(&second.material).unwrap().base_color_texture.as_ref());
        assert_eq!(Some(texture), materials.get(&second.remote).unwrap().base_color_texture.as_ref());
        assert_eq!(images.len(), 1, "one painting upload across different body rigs");
        let image = images.get(texture).unwrap();
        let painting = crate::npc_textures::get(crate::npc_appearance::look(kind).texture_key).unwrap();
        assert_eq!(&image.data.as_ref().unwrap()[..painting.rgba.len()], painting.rgba);
        assert!(image.texture_descriptor.mip_level_count > 1);
        assert_eq!((image.width(), image.height()), painting.size);
        assert_eq!(image.asset_usage, RenderAssetUsages::RENDER_WORLD);
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires shared Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT")]
    fn costume_instances_share_gpu_assets_and_own_their_animated_joint_palettes() {
        for kind in [CharacterKind::Mom, CharacterKind::PokefanF, CharacterKind::Blue,
            CharacterKind::Youngster, CharacterKind::StandingYoungster, CharacterKind::BugCatcher] {
            verify_costume_instance_animation(kind);
        }
    }

    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied rider rigs and paintings in CRYSTAL_OPEN_MODEL_ROOT")]
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
        for kind in [CharacterKind::Trainer, CharacterKind::TrainerFemale, CharacterKind::Biker] {
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
            let mut person = cache.spawn(look.base, kind, rig.clone(), actor, false, true, None,
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

    #[cfg_attr(test, test)]
    #[cfg_attr(test, ignore = "requires supplied character rigs and paintings")]
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
            (CharacterKind::Trainer,true),(CharacterKind::TrainerFemale,true),
            (CharacterKind::Biker,true)]);
        assert_eq!(cases.len(),78);
        for (kind,bicycle) in cases {
            let look = crate::npc_appearance::look(kind);
            let rig = crate::open_people::get(look.base).unwrap().clone();
            let mut commands = Commands::new(&mut queue,&world);
            let actor = commands.spawn((Transform::default(),Visibility::default())).id();
            let mut person = cache.spawn(look.base,kind,rig,actor,false,bicycle,None,
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
            world.clear_trackers();
            person.update(0.,0.,0.,&mut state.get_mut(&mut world).unwrap());
            for &node in &person.nodes {
                assert!(!world.entity(node).get_ref::<Transform>().unwrap().is_changed(),
                    "{}: unchanged joint marked dirty", look.texture_key);
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
            people.push(cache.spawn(base,kind,rig.clone(),root,remote,false,None,
                &mut commands,&mut meshes,&mut materials,&mut images,&mut binds));
            queue.apply(&mut world);
        }
        let gpu = &cache.0[&(base,kind)];
        let cloth = gpu.costume.as_ref();
        let mut skins = world.query::<(&SkinnedMesh,&Mesh3d,&ChildOf)>();
        let bodies: Vec<_> = skins.iter(&world).filter(|(_,mesh,_)| mesh.0==gpu.mesh).collect();
        assert_eq!(bodies.len(),2);
        assert_ne!(bodies[0].0.joints,bodies[1].0.joints);
        for (skin,_,_) in &bodies {
            assert_eq!(skin.inverse_bindposes,gpu.binds);
            assert_eq!(skin.joints.len(),binds.get(&gpu.binds).unwrap().len());
            assert!(skin.joints.len()<=256);
        }
        let mut helper_entities=Vec::new();
        if let (Some(a),Some(b))=(&people[0].arm_skin,&people[1].arm_skin) {
            assert!(Arc::ptr_eq(&a.recipe,&b.recipe),"instances must share the immutable blend recipe");
            assert!(a.nodes.iter().all(|node|!b.nodes.contains(node)),"instances share deformation helpers");
            helper_entities.extend_from_slice(&a.nodes);helper_entities.extend_from_slice(&b.nodes);
        }
        if let Some(cloth) = cloth {
        let costumes: Vec<_> = skins.iter(&world).filter(|(_,mesh,_)| mesh.0==cloth.mesh).collect();
        assert_eq!(costumes.len(),2);
        assert_eq!(skins.iter(&world).count(),4,"body and garment skins for both actors");
        assert_eq!(costumes[0].0.inverse_bindposes,costumes[1].0.inverse_bindposes);
        assert_ne!(costumes[0].0.joints,costumes[1].0.joints,"instances may not share live joints");
        for (skin,_,parent) in costumes {
            assert_eq!(world.get::<Mesh3d>(parent.parent()).map(|mesh| &mesh.0),Some(&gpu.mesh));
            assert_eq!(skin.inverse_bindposes,gpu.source_binds);
            let body=world.get::<SkinnedMesh>(parent.parent()).unwrap();
            assert_eq!(skin.joints.as_slice(),&body.joints[..rig.skin_nodes.len()],
                "garment and body must share the original source joints");
        }
        }
        let counts = (meshes.len(),materials.len(),images.len(),binds.len());
        let untouched = *world.get::<Transform>(people[1].nodes[rig.skin_nodes[rig.leg_joints[0][0]]]).unwrap();
        let finger = rig.finger_nodes[1];
        let other_finger = *world.get::<Transform>(people[1].nodes[finger]).unwrap();
        let other_helpers: Vec<_>=people[1].arm_skin.as_ref().map(|arm|arm.nodes.iter()
            .map(|&node|(node,*world.get::<Transform>(node).unwrap())).collect()).unwrap_or_default();
        let first_helpers: Vec<_>=people[0].arm_skin.as_ref().map(|arm|arm.nodes.iter()
            .map(|&node|*world.get::<Transform>(node).unwrap()).collect()).unwrap_or_default();
        let mut state = SystemState::<Query<&mut Transform,With<ModeledActor>>>::new(&mut world);
        people[0].update(std::f32::consts::FRAC_PI_2,1.0,0.0,&mut state.get_mut(&mut world).unwrap());
        assert_eq!(*world.get::<Transform>(people[1].nodes[rig.skin_nodes[rig.leg_joints[0][0]]]).unwrap(),untouched);
        assert_eq!(*world.get::<Transform>(people[1].nodes[finger]).unwrap(),other_finger);
        for &(node,pose) in &other_helpers {assert_eq!(*world.get::<Transform>(node).unwrap(),pose);}
        if let Some(arm)=&people[0].arm_skin {
            assert!(arm.nodes.iter().zip(&first_helpers).any(|(&node,old)|
                world.get::<Transform>(node).unwrap()!=old),"live helpers did not follow the running pose");
        }
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
        people[0].locomotion.sample(&rig,std::f32::consts::FRAC_PI_2,
            people[0].run_blend,&mut expected);
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
        assert!(helper_entities.into_iter().all(|entity|world.get_entity(entity).is_err()),
            "despawning a person leaked arm deformation helpers");
    }
    #[test]
    #[ignore = "requires supplied character rigs and paintings"]
    fn link_uniform_variants_share_body_and_joints_but_never_alias_paint_or_costume() {
        use bevy::ecs::world::CommandQueue;
        let kind=CharacterKind::LinkReceptionist;
        let look=crate::npc_appearance::look(kind);
        let rig=crate::open_people::get(look.base).unwrap().clone();
        let mut world=World::new();let mut queue=CommandQueue::default();
        let mut meshes=Assets::<Mesh>::default();let mut materials=Assets::<StandardMaterial>::default();
        let mut images=Assets::<Image>::default();let mut binds=Assets::<SkinnedMeshInverseBindposes>::default();
        let mut cache=PeopleGpu::default();
        let green=[80,192,48];let blue=[48,104,224];
        for color in [Some(green),Some(blue),Some(green),None] {
            let mut commands=Commands::new(&mut queue,&world);
            let root=commands.spawn((Transform::default(),Visibility::default())).id();
            cache.spawn(look.base,kind,rig.clone(),root,false,false,color,&mut commands,
                &mut meshes,&mut materials,&mut images,&mut binds);
            queue.apply(&mut world);
        }
        assert_eq!(cache.0.len(),1);assert_eq!(cache.3.len(),2);
        let base=&cache.0[&(look.base,kind)];
        let g=&cache.3[&(look.base,kind,green)];let b=&cache.3[&(look.base,kind,blue)];
        assert_eq!(g.mesh,base.mesh);assert_eq!(g.mesh,b.mesh);assert_eq!(g.binds,b.binds);
        assert_eq!(g.bounds,b.bounds);
        assert_eq!(g.head.as_ref().map(|h|&h.mesh),base.head.as_ref().map(|h|&h.mesh));
        let texture=|gpu:&Gpu|materials.get(&gpu.material).unwrap().base_color_texture.as_ref().unwrap();
        assert_ne!(texture(g),texture(b));assert_ne!(texture(g),texture(base));
        let original=crate::npc_textures::get(look.texture_key).unwrap();
        // Upload appends filtered mip levels; mip zero remains exact.
        assert!(images.get(texture(base)).unwrap().data.as_deref().unwrap().starts_with(&original.rgba));
        let painted=crate::npc_uniform::painting(&original.rgba,original.size,green);
        assert!(images.get(texture(g)).unwrap().data.as_deref().unwrap().starts_with(&painted));
        let gm=meshes.get(&g.costume.as_ref().unwrap().mesh).unwrap();
        let bm=meshes.get(&b.costume.as_ref().unwrap().mesh).unwrap();
        assert_eq!(format!("{:?}",gm.indices()),format!("{:?}",bm.indices()));
        for (attribute,values) in gm.attributes() {
            let other=bm.attribute(attribute.id).unwrap();
            if attribute.id!=Mesh::ATTRIBUTE_COLOR.id {assert_eq!(values.get_bytes(),other.get_bytes());}
            else {assert_ne!(values.get_bytes(),other.get_bytes());}
        }
        cache.retain_kinds(&std::collections::HashSet::new());
        assert!(cache.0.is_empty()&&cache.3.is_empty()&&cache.1.is_empty());
    }

    #[test]
    #[ignore = "requires supplied character rigs and paintings"]
    fn service_uniform_variants_keep_immutable_rigs_aprons_and_source_colored_skirts() {
        use bevy::ecs::world::CommandQueue;
        for kind in [CharacterKind::Clerk,CharacterKind::Receptionist,CharacterKind::Lass,
            CharacterKind::Youngster,CharacterKind::CooltrainerM,CharacterKind::SuperNerd,CharacterKind::KimonoGirl] {
        let look=crate::npc_appearance::look(kind);
        let rig=crate::open_people::get(look.base).unwrap().clone();
        let mut world=World::new();let mut queue=CommandQueue::default();
        let mut meshes=Assets::<Mesh>::default();let mut materials=Assets::<StandardMaterial>::default();
        let mut images=Assets::<Image>::default();let mut binds=Assets::<SkinnedMeshInverseBindposes>::default();
        let mut cache=PeopleGpu::default();
        let green=[80,192,48];let blue=[48,104,224];
        for color in [Some(green),Some(blue),Some(green),None] {
            let mut commands=Commands::new(&mut queue,&world);
            let root=commands.spawn((Transform::default(),Visibility::default())).id();
            cache.spawn(look.base,kind,rig.clone(),root,false,false,color,&mut commands,
                &mut meshes,&mut materials,&mut images,&mut binds);
            queue.apply(&mut world);
        }
        assert_eq!(cache.0.len(),1);assert_eq!(cache.3.len(),2);
        let base=&cache.0[&(look.base,kind)];
        let g=&cache.3[&(look.base,kind,green)];let b=&cache.3[&(look.base,kind,blue)];
        assert_eq!(g.mesh,base.mesh);assert_eq!(g.mesh,b.mesh);assert_eq!(g.binds,b.binds);
        assert_eq!(g.bounds,b.bounds);
        assert_eq!(g.head.as_ref().map(|h|&h.mesh),base.head.as_ref().map(|h|&h.mesh));
        let texture=|gpu:&Gpu|materials.get(&gpu.material).unwrap().base_color_texture.as_ref().unwrap();
        assert_ne!(texture(g),texture(b));assert_ne!(texture(g),texture(base));
        let original=crate::npc_textures::get(look.texture_key).unwrap();
        // Upload appends filtered mip levels; mip zero remains exact.
        assert!(images.get(texture(base)).unwrap().data.as_deref().unwrap().starts_with(&original.rgba));
        let painted=crate::npc_uniform::painting_for(kind,&original.rgba,original.size,green);
        assert!(images.get(texture(g)).unwrap().data.as_deref().unwrap().starts_with(&painted));
        let blue_painted=crate::npc_uniform::painting_for(kind,&original.rgba,original.size,blue);
        assert_ne!(painted,blue_painted,"{kind:?}: the actual painting must change with its source palette");
        let (w,h)=original.size;
        let mut changed=0;
        for (i,(before,after)) in original.rgba.chunks_exact(4).zip(painted.chunks_exact(4)).enumerate() {
            let (x,y)=((i as u32%w)*256/w,(i as u32/w)*256/h);
            assert_eq!(before[3],after[3]);
            if y < if kind==CharacterKind::KimonoGirl {120} else {128}
                || (x>=155 && !(matches!(kind,CharacterKind::Receptionist|CharacterKind::Lass|CharacterKind::CooltrainerM|CharacterKind::KimonoGirl)&&x>=156&&y>=192)) {
                assert_eq!(before,after,"{kind:?}: actual hair/face/accessory pixels stay immutable at {x},{y}");
            }
            changed+=usize::from(before!=after);
        }
        assert!(changed>256,"{kind:?}: real cloth islands, not an empty variant");
        if let (Some(gc),Some(bc))=(&g.costume,&b.costume) {
        let gm=meshes.get(&gc.mesh).unwrap();
        let bm=meshes.get(&bc.mesh).unwrap();
        assert_eq!(format!("{:?}",gm.indices()),format!("{:?}",bm.indices()));
        for (attribute,values) in gm.attributes() {
            let other=bm.attribute(attribute.id).unwrap();
            if attribute.id!=Mesh::ATTRIBUTE_COLOR.id {assert_eq!(values.get_bytes(),other.get_bytes());}
            else if matches!(kind,CharacterKind::Receptionist|CharacterKind::Lass|CharacterKind::KimonoGirl) {assert_ne!(values.get_bytes(),other.get_bytes());}
            else {assert_eq!(values.get_bytes(),other.get_bytes(),"clerk apron stays cream");}
        }
        if kind == CharacterKind::KimonoGirl {
            let original = meshes.get(&base.costume.as_ref().unwrap().mesh).unwrap();
            assert_eq!(format!("{:?}",gm.indices()),format!("{:?}",original.indices()));
            for (attribute,values) in gm.attributes() {
                if attribute.id != Mesh::ATTRIBUTE_COLOR.id {
                    assert_eq!(values.get_bytes(),original.attribute(attribute.id).unwrap().get_bytes(),
                        "palette variants retain the default robe's geometry and skin");
                }
            }
            let obi_vertices = crate::kimono_obi::build(&rig,look).unwrap().surface.positions.len();
            let colors = |mesh: &Mesh| match mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap() {
                VertexAttributeValues::Float32x4(values) => values.clone(),
                _ => panic!("kimono garment colors"),
            };
            let (gc,bc,oc) = (colors(gm),colors(bm),colors(original));
            let shell = gc.len() - obi_vertices;
            assert!(shell > 0 && obi_vertices > 0);
            assert_eq!(&gc[shell..],&oc[shell..],"green robe preserves the authored obi and bow");
            assert_eq!(&bc[shell..],&oc[shell..],"blue robe preserves the authored obi and bow");
            assert_ne!(&gc[..shell],&oc[..shell]);
            // The shell must sample the same recolored plain patch as the
            // painted body, rather than the immutable red atlas again.
            let (x,y) = ((w as f32*0.075) as i32,(h as f32*0.75) as i32);
            let mut sum = [0u32;3];
            for dy in -2..=2 {for dx in -2..=2 {
                let at = (((y+dy) as u32*w+(x+dx) as u32)*4) as usize;
                for c in 0..3 {sum[c] += u32::from(painted[at+c]);}
            }}
            let rgb = sum.map(|v| (v/25) as u8);
            assert_eq!(gc[0],Color::srgb_u8(rgb[0],rgb[1],rgb[2]).to_linear().to_f32_array());
        }
        } else {assert!(g.costume.is_none()&&b.costume.is_none());}
        cache.retain_kinds(&std::collections::HashSet::new());
        assert!(cache.0.is_empty()&&cache.3.is_empty()&&cache.1.is_empty());
        }
    }

    #[test]
    #[ignore = "requires supplied character rigs and paintings"]
    fn reviewed_uniform_cache_keeps_fisher_caps_and_cooltrainer_hair_rigid_and_isolated() {
        use bevy::ecs::world::CommandQueue;
        for kind in [CharacterKind::Fisher,CharacterKind::CooltrainerF,CharacterKind::Officer] {
            let look=crate::npc_appearance::look(kind);let rig=crate::open_people::get(look.base).unwrap().clone();
            let mut world=World::new();let mut queue=CommandQueue::default();
            let mut meshes=Assets::<Mesh>::default();let mut materials=Assets::<StandardMaterial>::default();
            let mut images=Assets::<Image>::default();let mut binds=Assets::<SkinnedMeshInverseBindposes>::default();
            let mut cache=PeopleGpu::default();let green=[57,189,24];let red=[255,57,8];
            for color in [Some(green),Some(red),Some(green),None] {
                let mut commands=Commands::new(&mut queue,&world);
                let root=commands.spawn((Transform::default(),Visibility::default())).id();
                cache.spawn(look.base,kind,rig.clone(),root,false,false,color,&mut commands,
                    &mut meshes,&mut materials,&mut images,&mut binds);queue.apply(&mut world);
            }
            assert_eq!(cache.0.len(),1);assert_eq!(cache.3.len(),2);
            let base=&cache.0[&(look.base,kind)];let g=&cache.3[&(look.base,kind,green)];let r=&cache.3[&(look.base,kind,red)];
            assert_eq!(g.mesh,base.mesh);assert_eq!(g.binds,base.binds);assert_eq!(g.bounds,r.bounds);
            let gh=meshes.get(&g.head.as_ref().unwrap().mesh).unwrap();
            let rh=meshes.get(&r.head.as_ref().unwrap().mesh).unwrap();
            let bh=meshes.get(&base.head.as_ref().unwrap().mesh).unwrap();
            for (a,values) in gh.attributes() {
                if a.id==Mesh::ATTRIBUTE_COLOR.id {assert!(values.get_bytes()!=rh.attribute(a.id).unwrap().get_bytes());}
                else {assert!(values.get_bytes()==rh.attribute(a.id).unwrap().get_bytes());
                    assert!(values.get_bytes()==bh.attribute(a.id).unwrap().get_bytes());}
            }
            assert_eq!(format!("{:?}",gh.indices()),format!("{:?}",bh.indices()));
            let texture=|gpu:&Gpu|materials.get(&gpu.material).unwrap().base_color_texture.as_ref().unwrap();
            assert_ne!(texture(g),texture(r));assert_ne!(texture(g),texture(base));
            let original=crate::npc_textures::get(look.texture_key).unwrap();
            assert!(images.get(texture(base)).unwrap().data.as_ref().unwrap().starts_with(&original.rgba));
            let painted=crate::npc_uniform::painting_for(kind,&original.rgba,original.size,green);
            assert!(images.get(texture(g)).unwrap().data.as_ref().unwrap().starts_with(&painted));
            if kind == CharacterKind::Officer {
                let mut shirt = 0; let mut trousers = 0;
                for (i,(before,after)) in original.rgba.chunks_exact(4).zip(painted.chunks_exact(4)).enumerate() {
                    let x=(i as u32%original.size.0)*256/original.size.0;
                    let y=(i as u32/original.size.0)*256/original.size.1;
                    assert_eq!(before[3],after[3]);
                    let upper=x<155&&y>=128; let lower=x>=156&&y>=192;
                    if !(upper||lower) { assert_eq!(before,after,"officer hair, face, hands and shoes"); }
                    if before!=after { shirt+=usize::from(upper); trousers+=usize::from(lower); }
                }
                assert!(shirt>256 && trousers>256);
            }
            if let (Some(gc),Some(bc))=(&g.costume,&base.costume) {
                let gm=meshes.get(&gc.mesh).unwrap();let bm=meshes.get(&bc.mesh).unwrap();
                for (a,values) in gm.attributes() {if a.id!=Mesh::ATTRIBUTE_COLOR.id {
                    assert!(values.get_bytes()==bm.attribute(a.id).unwrap().get_bytes());
                }}
                if kind == CharacterKind::Officer {
                    assert_ne!(gm.attribute(Mesh::ATTRIBUTE_COLOR).unwrap().get_bytes(),
                        bm.attribute(Mesh::ATTRIBUTE_COLOR).unwrap().get_bytes(),"officer collar follows source cloth");
                }
            }
            cache.retain_kinds(&std::collections::HashSet::new());assert!(cache.0.is_empty()&&cache.3.is_empty());
        }
    }

}
