//! Retained, render-only body fire and vapor. These are anatomy, not move cues.
//! Shared shader/quad assets and a fixed pool per emitter bound CPU/GPU cost.
use super::*;
use crate::species_rig::SpeciesRig;
use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    mesh::{Indices, MeshVertexBufferLayoutRef},
    pbr::{MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    render::render_resource::{RenderPipelineDescriptor, SpecializedMeshPipelineError},
};

const SHADER: Handle<Shader> = Handle::Uuid(
    bevy::asset::uuid::Uuid::from_u128(0xafb89ae0_0c55_4727_9210_ac0159c6ebac),
    std::marker::PhantomData,
);
const SLOTS: usize = 8;
const MAX_EMITTERS: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Fire,
    Ember,
    Vapor,
    FireVolume,
}
impl Kind {
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub(super) struct AuraMaterial {
    #[uniform(0)]
    parameters: Vec4, // local seconds, kind, reserved, reserved
    #[uniform(0)]
    cool: Vec4,
    #[uniform(0)]
    hot: Vec4,
}
impl Material for AuraMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    fn specialize(
        _: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

#[derive(Component)]
pub(super) struct AuraParticle {
    side: usize,
    emitter: usize,
    slot: usize,
}
#[derive(Component)]
pub(super) struct AuraVolume {
    side: usize,
    emitter: usize,
}

struct Emitter {
    root: Vec3,
    joints: [u16; 4],
    weights: [f32; 4],
    height: f32,
    width: f32,
    kind: Kind,
}
impl Emitter {
    fn posed_root(&self, matrices: &[Mat4]) -> Vec3 {
        self.joints
            .iter()
            .zip(self.weights)
            .fold(Vec3::ZERO, |point, (&joint, weight)| {
                point + matrices[joint as usize].transform_point3(self.root) * weight
            })
    }
}
struct Instance {
    owner: Entity,
    emitters: Vec<Emitter>,
    matrices: Vec<Mat4>,
    elapsed: f32,
}
#[derive(Resource, Default)]
pub(super) struct Auras {
    instances: [Option<Instance>; 2],
    quads: [Handle<Mesh>; SLOTS],
    materials: [[Handle<AuraMaterial>; 4]; 2],
}

pub(super) fn register(app: &mut App) {
    if app.world().contains_resource::<AssetServer>() {
        app.add_plugins(MaterialPlugin::<AuraMaterial>::default());
    } else {
        app.init_resource::<Assets<AuraMaterial>>();
    }
    load_internal_asset!(app, SHADER, "battle_aura.wesl", Shader::from_wesl);
    app.init_resource::<Auras>().add_systems(Startup, setup);
}

fn setup(
    mut auras: ResMut<Auras>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<AuraMaterial>>,
) {
    auras.quads = std::array::from_fn(|slot| {
        let mut quad = Rectangle::new(1.0, 1.0).mesh().build();
        // Constant vertex data carries a stable phase, rate and base-plume flag.
        // It avoids a unique material or a changing vertex buffer per particle.
        let (phase, rate) = timing(slot);
        quad.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![[phase, rate, f32::from(slot == 0), 1.0]; 4],
        );
        meshes.add(quad)
    });
    auras.materials = std::array::from_fn(|_| {
        std::array::from_fn(|index| {
            let kind = [Kind::Fire, Kind::Ember, Kind::Vapor, Kind::FireVolume][index];
            let (cool, hot) = colors(kind);
            materials.add(AuraMaterial {
                parameters: Vec4::new(0.0, index as f32, 0.0, 0.0),
                cool,
                hot,
            })
        })
    });
}

/// Explicit anatomy labels prevent burning orange bodies, painted belly marks,
/// tail flesh, or every Fire-type Pokémon. Inner layers are retired together.
fn role(species: &str, name: &str) -> Option<(Kind, bool)> {
    let outer = match species {
        "gastly" => {
            return name
                .starts_with("Vapor cloud lobe")
                .then_some((Kind::Vapor, true));
        }
        "ponyta" | "rapidash" => {
            if name.starts_with("Flame glow / ") {
                false
            } else if name.starts_with("Flame / ") {
                true
            } else {
                return None;
            }
        }
        "cyndaquil" => {
            if !name.contains(" flame quill / ") {
                return None;
            }
            name.ends_with(" / ember silhouette")
        }
        "quilava" | "typhlosion" => {
            if !(name.starts_with("Neck eruption ") || name.starts_with("Rump eruption ")) {
                return None;
            }
            name.contains(" eruption outer")
        }
        "charmander" => {
            if !name.starts_with("Flame / ") {
                return None;
            }
            name.ends_with(" / ember silhouette")
        }
        "charmeleon" | "charizard" => {
            if !name.starts_with("Tail flame ") {
                return None;
            }
            name.starts_with("Tail flame outer")
        }
        "magmar" => {
            if name.starts_with("Crown glow / ") || name.starts_with("Tail glow / ") {
                false
            } else if name.starts_with("Crown flame / ") || name.starts_with("Tail flame / ") {
                true
            } else {
                return None;
            }
        }
        "moltres" => {
            if [
                "Wing glow / ",
                "Wing heart / ",
                "Crown glow / ",
                "Crown heart / ",
                "Tail glow / ",
                "Tail heart / ",
            ]
            .iter()
            .any(|prefix| name.starts_with(prefix))
            {
                false
            } else if name.starts_with("Wing fire / ")
                || name == "Crown / flowing three-lick head flame"
                || name == "Tail / wide trailing closed flame plume"
            {
                true
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some((Kind::Fire, outer))
}

fn emitters(rig: &SpeciesRig) -> Vec<Emitter> {
    let parts: Vec<_> = rig
        .anatomy
        .iter()
        .filter(|p| role(rig.species.name(), &p.name).is_some_and(|(_, outer)| outer))
        .collect();
    let groups: Vec<Vec<_>> = if rig.species.name() == "cyndaquil" {
        vec![parts]
    } else {
        parts.into_iter().map(|p| vec![p]).collect()
    };
    let mut result = Vec::new();
    for parts in groups {
        let Some(first) = parts.first() else {
            continue;
        };
        let kind = role(rig.species.name(), &first.name).unwrap().0;
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for part in &parts {
            for p in &rig.neutral.positions[part.vertices.clone()] {
                lo = lo.min(Vec3::from_array(*p));
                hi = hi.max(Vec3::from_array(*p));
            }
        }
        let size = hi - lo;
        let axis = if size.x > size.z { 0 } else { 2 };
        let count = if rig.species.name() == "moltres" && first.name.starts_with("Wing fire / ") {
            4
        } else {
            1
        };
        for segment in 0..count {
            if result.len() == MAX_EMITTERS {
                return result;
            }
            // Each wing bank follows its own span section. Using the whole
            // wing's minimum Y for every section produces a flat floating wall.
            let mut patch_lo = lo;
            let mut patch_hi = hi;
            if count > 1 {
                let start = lo[axis] + size[axis] * segment as f32 / count as f32;
                let end = lo[axis] + size[axis] * (segment + 1) as f32 / count as f32;
                let mut local_lo = Vec3::splat(f32::INFINITY);
                let mut local_hi = Vec3::splat(f32::NEG_INFINITY);
                for part in &parts {
                    for p in &rig.neutral.positions[part.vertices.clone()] {
                        let p = Vec3::from_array(*p);
                        if p[axis] >= start && p[axis] <= end {
                            local_lo = local_lo.min(p);
                            local_hi = local_hi.max(p);
                        }
                    }
                }
                if local_lo.is_finite() && local_hi.is_finite() {
                    patch_lo = local_lo;
                    patch_hi = local_hi;
                }
            }
            let extent = patch_hi - patch_lo;
            let mut root = (patch_lo + patch_hi) * 0.5;
            if kind == Kind::Fire {
                root.y = patch_lo.y + extent.y * 0.04;
            }
            if rig.species.name() == "cyndaquil" {
                // Bury the source inside the back. A billboard placed at the
                // old quill minimum exposes its straight lower edge as a tail.
                root = rig.joints[1].bind.translation + Vec3::Y * 0.035;
            }
            let vertex = first.vertices.start;
            let (joints, weights) = if rig.species.name() == "cyndaquil" {
                ([1, 0, 0, 0], [1., 0., 0., 0.])
            } else {
                (rig.joint_indices[vertex], rig.joint_weights[vertex])
            };
            result.push(Emitter {
                root,
                joints,
                weights,
                kind,
                height: if rig.species.name() == "cyndaquil" {
                    (hi.y - root.y).max(0.035)
                } else {
                    extent.y.max(0.035)
                },
                width: if kind == Kind::Vapor {
                    size.max_element() * 0.85
                } else {
                    (size[axis] / count as f32).max(extent.y * 0.55)
                },
            });
        }
    }
    result
}

/// Instance indices alone change. Canonical geometry, skin weights, authored
/// bounds, appearance colors and source registration retain their original bits.
pub(super) fn remove_solid_effects(mesh: &mut Mesh, rig: &SpeciesRig) {
    if !rig
        .anatomy
        .iter()
        .any(|p| role(rig.species.name(), &p.name).is_some())
    {
        return;
    }
    let indices = rig
        .anatomy
        .iter()
        .filter(|p| role(rig.species.name(), &p.name).is_none())
        .flat_map(|p| rig.neutral.indices[p.indices.clone()].iter().copied())
        .collect();
    mesh.insert_indices(Indices::U32(indices));
}

fn colors(kind: Kind) -> (Vec4, Vec4) {
    match kind {
        Kind::Fire | Kind::Ember | Kind::FireVolume => (
            Vec4::new(1.0, 0.018, 0.001, 1.0),
            Vec4::new(1.0, 0.42, 0.035, 1.0),
        ),
        Kind::Vapor => (
            Vec4::new(0.15, 0.025, 0.28, 1.0),
            Vec4::new(0.54, 0.16, 0.78, 1.0),
        ),
    }
}
fn timing(slot: usize) -> (f32, f32) {
    (
        (slot as f32 * 0.618_034).fract(),
        0.72 + slot as f32 * 0.071,
    )
}

fn particle_pose(emitter: &Emitter, slot: usize, elapsed: f32, up: Vec3) -> (Vec3, Vec2) {
    let (phase, rate) = timing(slot);
    let age = (elapsed * rate + phase).fract();
    let angle = phase * TAU + elapsed * 1.7;
    if emitter.kind == Kind::Vapor {
        let drift =
            Vec3::new(angle.cos(), (angle * 0.7).sin() * 0.65, angle.sin()) * emitter.width * 0.17;
        return (
            drift + up * age * emitter.height * 0.28,
            Vec2::splat(emitter.width * (0.85 + age * 0.65)),
        );
    }
    let ember = slot >= 3;
    let rise = if slot == 0 { 0.0 } else { age };
    let drift = Vec3::new(angle.sin(), 0.0, (angle * 1.3).cos())
        * emitter.width
        * rise
        * if ember { 0.7 } else { 0.22 };
    let height = if ember {
        emitter.height * 0.055 * (1.0 - age * 0.6)
    } else if slot == 0 {
        emitter.height * 0.3
    } else {
        emitter.height * (0.3 - age * 0.17)
    };
    let width = if ember {
        height * 0.45
    } else {
        emitter.width * if slot == 0 { 1.15 } else { 0.7 - age * 0.4 }
    };
    (
        drift
            + up * (rise * emitter.height * if ember { 1.8 } else { 0.8 }
                + height * 0.48
                + emitter.height * 0.36),
        Vec2::new(width, height),
    )
}

pub(super) fn sync(
    mut commands: Commands,
    scene: Res<BattleScene>,
    status: Res<BattleViewStatus>,
    frame: Res<VisualBattleFrame>,
    layout: Res<BattleSceneLayout>,
    time: Res<Time>,
    flash: Res<BattleFlashMode>,
    mut auras: ResMut<Auras>,
    mut materials: ResMut<Assets<AuraMaterial>>,
    actors: Query<(&Transform, &RenderLayers), (With<BattleActor>, Without<AuraParticle>)>,
    mut particles: Query<
        (
            &AuraParticle,
            &mut Transform,
            &mut Visibility,
            &mut RenderLayers,
        ),
        Without<BattleActor>,
    >,
    mut surfaces: Query<
        (
            &AuraVolume,
            &mut Transform,
            &mut Visibility,
            &mut RenderLayers,
        ),
        (Without<BattleActor>, Without<AuraParticle>),
    >,
) {
    for side in 0..2 {
        let actor = scene.actors[side].as_ref();
        if auras.instances[side].as_ref().map(|i| i.owner) != actor.map(|a| a.entity) {
            // Children retire with their actor, including party switches.
            auras.instances[side] = None;
            if let Some(actor) = actor {
                if let Some(skin) = &actor.skinned {
                    let emitters = emitters(skin.rig);
                    {
                        for (index, emitter) in emitters.iter().enumerate() {
                            if emitter.kind == Kind::Fire {
                                let entity = commands
                                    .spawn((
                                        Mesh3d(auras.quads[index % SLOTS].clone()),
                                        MeshMaterial3d(
                                            auras.materials[side][Kind::FireVolume.index()].clone(),
                                        ),
                                        Transform::default(),
                                        Visibility::Hidden,
                                        RenderLayers::layer(BATTLE_LAYER),
                                        NotShadowCaster,
                                        NotShadowReceiver,
                                        AuraVolume {
                                            side,
                                            emitter: index,
                                        },
                                    ))
                                    .id();
                                commands.entity(actor.entity).add_child(entity);
                            }

                            for slot in 0..SLOTS {
                                let kind = if emitter.kind == Kind::Vapor {
                                    Kind::Vapor
                                } else if slot >= 3 {
                                    Kind::Ember
                                } else {
                                    Kind::Fire
                                };
                                let child = commands
                                    .spawn((
                                        Mesh3d(auras.quads[slot].clone()),
                                        MeshMaterial3d(auras.materials[side][kind.index()].clone()),
                                        Transform::default(),
                                        Visibility::Hidden,
                                        RenderLayers::layer(BATTLE_LAYER),
                                        NotShadowCaster,
                                        NotShadowReceiver,
                                        AuraParticle {
                                            side,
                                            emitter: index,
                                            slot,
                                        },
                                    ))
                                    .id();
                                commands.entity(actor.entity).add_child(child);
                            }
                        }
                        auras.instances[side] = Some(Instance {
                            owner: actor.entity,
                            emitters,
                            matrices: vec![Mat4::IDENTITY; skin.rig.joints.len()],
                            elapsed: 0.0,
                        });
                    }
                }
            }
        }
        let shown = status.active
            && frame.battlers[side]
                .as_ref()
                .is_some_and(|b| b.visible && b.allow_species_model);
        let Some(instance) = &mut auras.instances[side] else {
            continue;
        };
        if instance.emitters.is_empty() {
            continue;
        }
        if shown {
            instance.elapsed = (instance.elapsed + time.delta_secs()).min(1.0e6);
            if let Some(skin) = actor.and_then(|a| a.skinned.as_ref()) {
                skin.effect_matrices(&mut instance.matrices);
            }
        }
        let elapsed = instance.elapsed;
        for kind in [Kind::Fire, Kind::Ember, Kind::Vapor, Kind::FireVolume] {
            let (cool, hot) = colors(kind);
            let map = |value: Vec4| {
                let mut color = value.to_array();
                if let Some(palette) = frame.battlers[side].as_ref().and_then(|b| b.model_palette) {
                    color = crate::pokemon_appearance::color(color, palette);
                }
                if let Some(source) = &frame.source {
                    if source.battler_bgps[side] != 0xe4 {
                        color = source_model_color(
                            color,
                            source.battler_bgps[side],
                            &source.battler_palettes[side],
                            *flash,
                        );
                    }
                }
                Vec4::from_array(color)
            };
            if let Some(mut material) = materials.get_mut(&auras.materials[side][kind.index()]) {
                material.parameters.x = elapsed;
                material.cool = map(cool);
                material.hot = map(hot);
            }
        }
    }
    for (volume, mut transform, mut visibility, mut layers) in &mut surfaces {
        let side = volume.side;
        let shown = status.active
            && frame.battlers[side]
                .as_ref()
                .is_some_and(|b| b.visible && b.allow_species_model);
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if !shown {
            continue;
        }
        let Some(instance) = &auras.instances[side] else {
            continue;
        };
        let Ok((root, actor_layers)) = actors.get(instance.owner) else {
            continue;
        };
        let emitter = &instance.emitters[volume.emitter];
        let up = root.rotation.inverse() * Vec3::Y;
        transform.set_if_neq(Transform {
            translation: emitter.posed_root(&instance.matrices) + up * emitter.height * 0.5,
            rotation: root.rotation.inverse() * layout.camera.rotation,
            scale: Vec3::new(emitter.width * 1.3, emitter.height * 1.1, 1.),
        });
        layers.set_if_neq(actor_layers.clone().without(BATTLE_GRASS_MASK_LAYER));
    }
    for (particle, mut transform, mut visibility, mut layers) in &mut particles {
        let side = particle.side;
        let shown = status.active
            && frame.battlers[side]
                .as_ref()
                .is_some_and(|b| b.visible && b.allow_species_model);
        let Some(instance) = &auras.instances[side] else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let Ok((root, actor_layers)) = actors.get(instance.owner) else {
            continue;
        };
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if !shown {
            continue;
        }
        let emitter = &instance.emitters[particle.emitter];
        let up = root.rotation.inverse() * Vec3::Y;
        let (offset, size) = particle_pose(emitter, particle.slot, instance.elapsed, up);
        let rotation = root.rotation.inverse() * layout.camera.rotation;
        transform.set_if_neq(Transform {
            translation: emitter.posed_root(&instance.matrices) + offset,
            rotation,
            scale: Vec3::new(size.x, size.y, 1.0),
        });
        // Wisps must not punch grass-mask holes or change source-row receipts.
        layers.set_if_neq(actor_layers.clone().without(BATTLE_GRASS_MASK_LAYER));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_effects_replace_only_explicit_fire_or_vapor_and_keep_canonical_geometry() {
        for species in [
            "PONYTA",
            "RAPIDASH",
            "CYNDAQUIL",
            "QUILAVA",
            "TYPHLOSION",
            "CHARMANDER",
            "CHARMELEON",
            "CHARIZARD",
            "MAGMAR",
            "MOLTRES",
            "GASTLY",
        ] {
            let rig = crate::species_rig::for_species(species).unwrap();
            let emitters = emitters(rig);
            assert!(!emitters.is_empty(), "{species} needs attached emitters");
            assert!(emitters.len() <= MAX_EMITTERS);
            let mut mesh = rig.neutral.clone().into_mesh();
            remove_solid_effects(&mut mesh, rig);
            let expected: Vec<_> = rig
                .anatomy
                .iter()
                .filter(|p| role(rig.species.name(), &p.name).is_none())
                .flat_map(|p| rig.neutral.indices[p.indices.clone()].iter().copied())
                .collect();
            assert_eq!(
                mesh.indices().unwrap().iter().collect::<Vec<_>>(),
                expected.iter().map(|i| *i as usize).collect::<Vec<_>>()
            );
            assert!(!expected.is_empty());
            assert!(expected.len() < rig.neutral.indices.len());
            assert_eq!(mesh.count_vertices(), rig.neutral.positions.len());
            assert_eq!(
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                    .unwrap()
                    .as_float3()
                    .unwrap(),
                &rig.neutral.positions
            );
        }
        for species in ["SQUIRTLE", "VULPIX", "HAUNTER", "GENGAR"] {
            let rig = crate::species_rig::for_species(species).unwrap();
            assert!(
                emitters(rig).is_empty(),
                "{species} has no authored body fire or gas"
            );
            let mut mesh = rig.neutral.clone().into_mesh();
            remove_solid_effects(&mut mesh, rig);
            assert_eq!(mesh.indices().unwrap().len(), rig.neutral.indices.len());
        }
    }

    #[test]
    fn emission_follows_the_real_skin_and_particle_motion_is_frame_rate_independent() {
        use crate::species_rig::{Clip, Playback};
        for species in ["PONYTA", "CYNDAQUIL", "MOLTRES", "GASTLY"] {
            let rig = crate::species_rig::for_species(species).unwrap();
            let mut pose: Vec<_> = rig.joints.iter().map(|j| j.bind).collect();
            let mut matrices = vec![Mat4::IDENTITY; pose.len()];
            rig.sample_into(
                Clip::Attack,
                rig.clip(Clip::Attack).duration * 0.47,
                Playback::Clamp,
                &mut pose,
            )
            .unwrap();
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            for emitter in emitters(rig) {
                let expected = emitter
                    .joints
                    .iter()
                    .zip(emitter.weights)
                    .map(|(&joint, w)| matrices[joint as usize].transform_point3(emitter.root) * w)
                    .sum::<Vec3>();
                assert!(emitter.posed_root(&matrices).distance(expected) < 0.00001);
                for slot in 0..SLOTS {
                    let at_30 = particle_pose(&emitter, slot, 21.0 / 30.0, Vec3::Y);
                    let at_60 = particle_pose(&emitter, slot, 42.0 / 60.0, Vec3::Y);
                    assert_eq!(at_30, at_60);
                    for frame in 0..120 {
                        let (offset, size) =
                            particle_pose(&emitter, slot, frame as f32 / 60.0, Vec3::Y);
                        assert!(offset.is_finite() && size.is_finite() && size.min_element() > 0.0);
                    }
                }
            }
        }
    }

    fn counts(app: &mut App) -> (usize, usize, usize) {
        let world = app.world_mut();
        (
            world.query::<&AuraParticle>().iter(world).count(),
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<AuraMaterial>>().len(),
        )
    }

    #[test]
    fn retained_body_particles_pause_hide_and_retire_without_changing_source_state() {
        let mut app = super::super::tests::headless_battle_app();
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.update();
        app.update();
        let before = app.world().resource::<VisualBattleFrame>().clone();
        let initial = counts(&mut app);
        assert!(initial.0 >= SLOTS);
        for _ in 0..120 {
            app.update();
        }
        assert_eq!(
            counts(&mut app),
            initial,
            "idle must reuse quads, materials and entities"
        );
        assert_eq!(*app.world().resource::<VisualBattleFrame>(), before);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        let paused = app.world().resource::<Auras>().instances[0]
            .as_ref()
            .unwrap()
            .elapsed;
        app.update();
        assert_eq!(
            app.world().resource::<Auras>().instances[0]
                .as_ref()
                .unwrap()
                .elapsed,
            paused
        );
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&Visibility, With<AuraParticle>>()
                .iter(world)
                .all(|v| *v == Visibility::Hidden)
        );
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        app.update();
        let old_owner = app.world().resource::<Auras>().instances[0]
            .as_ref()
            .unwrap()
            .owner;
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .species_id = Arc::from("GASTLY");
        app.update();
        app.update();
        assert!(app.world().get_entity(old_owner).is_err());
        let instance = app.world().resource::<Auras>().instances[0]
            .as_ref()
            .unwrap();
        assert!(instance.emitters.iter().all(|e| e.kind == Kind::Vapor));
        let owner = instance.owner;
        let world = app.world_mut();
        assert!(
            world
                .query_filtered::<&ChildOf, With<AuraParticle>>()
                .iter(world)
                .all(|parent| parent.parent() == owner)
        );
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0] = None;
        app.update();
        assert_eq!(counts(&mut app).0, 0, "actor removal must retire all wisps");
    }
}
