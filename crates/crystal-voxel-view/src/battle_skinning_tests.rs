use super::*;
use crate::species_rig::{self, Species};
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

fn cue(kind: VisualBattleCueKind, progress: f32) -> VisualBattleCue {
    VisualBattleCue {
        kind,
        side: VisualBattleSide::Player,
        move_id: Arc::from(if kind == VisualBattleCueKind::Move {
            "EMBER"
        } else {
            "VISIBLE_HP_LOSS"
        }),
        element: Arc::from("FIRE"),
        progress,
        damaging: true,
    }
}

fn close_pose(a: &[Transform], b: &[Transform]) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
        assert_eq!(a.translation, b.translation);
        assert_eq!(a.scale, b.scale);
        assert!((1.0 - a.rotation.dot(b.rotation).abs()).abs() < 0.00001);
    }
}

#[test]
fn current_visibility_bounds_enclose_weighted_vertices_without_future_pose_padding() {
    for species in [Species::Cyndaquil, Species::Totodile] {
        let rig = species_rig::rig(species);
        let parts = influence_bounds(rig);
        let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
        let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
        for clip in Clip::ALL {
            for progress in [0.0, 0.25, 0.5, 1.0] {
                rig.sample_into(clip, progress * rig.clip(clip).duration, Playback::Clamp, &mut pose).unwrap();
                rig.skin_matrices_into(&pose, &mut matrices).unwrap();
                let (min, max) = posed_visibility_bounds(&parts, &matrices).unwrap();
                for (index, point) in rig.neutral.positions.iter().enumerate() {
                    let point = Vec3::from_array(*point);
                    let skinned = rig.joint_indices[index].iter().zip(rig.joint_weights[index])
                        .fold(Vec3::ZERO, |sum, (&joint, weight)|
                            sum + matrices[joint as usize].transform_point3(point) * weight);
                    assert!(skinned.cmpge(min - Vec3::splat(0.001)).all());
                    assert!(skinned.cmple(max + Vec3::splat(0.001)).all());
                }
            }
        }
        rig.sample_into(Clip::Idle, 0.0, Playback::Clamp, &mut pose).unwrap();
        rig.skin_matrices_into(&pose, &mut matrices).unwrap();
        let (min, max) = posed_visibility_bounds(&parts, &matrices).unwrap();
        assert!((max - min).length() < (rig.animated_bounds.1 - rig.animated_bounds.0).length());
    }
}

#[test]
fn lapras_battle_pose_keeps_the_seated_belly_on_its_authored_floor() {
    let rig = species_rig::rig(Species::Lapras);
    let mut pose = vec![Transform::IDENTITY; rig.joints.len()];
    let mut matrices = vec![Mat4::IDENTITY; rig.joints.len()];
    for clip in Clip::ALL {
        for step in 0..=32 {
            rig.sample_into(clip, rig.clip(clip).duration * step as f32 / 32.0,
                Playback::Clamp, &mut pose).unwrap();
            species_rig::motion::battle_float(rig, &mut pose);
            rig.skin_matrices_into(&pose, &mut matrices).unwrap();
            let floor = rig.neutral.positions.iter().enumerate().map(|(index, point)| {
                rig.joint_indices[index].iter().zip(rig.joint_weights[index])
                    .fold(Vec3::ZERO, |sum, (&joint, weight)| sum
                        + matrices[joint as usize].transform_point3(Vec3::from_array(*point)) * weight).y
            }).fold(f32::INFINITY, f32::min);
            assert!(floor.abs() < 0.00001, "{clip:?} step {step}: seated belly lifted to {floor}");
        }
    }
    for species in ["CLOYSTER", "PORYGON"] {
        let rig = species_rig::for_species(species).unwrap();
        let mut pose = rig.joints.iter().map(|joint| joint.bind).collect::<Vec<_>>();
        species_rig::motion::battle_float(rig, &mut pose);
        let clearance = (rig.neutral_bounds.1.y - rig.neutral_bounds.0.y) * 0.16;
        assert_eq!(pose[0].translation.y, rig.joints[0].bind.translation.y + clearance);
    }
}

#[test]
fn body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress() {
    for species in Species::ALL {
        let rig = species_rig::rig(species);
        let mut expected = vec![Transform::IDENTITY; rig.joints.len()];
        rig.sample_into(Clip::Idle, 2.0, Playback::Loop, &mut expected)
            .unwrap();
        for hz in [9, 30, 60] {
            let mut state = BodyPose::new(rig);
            let mut frame = VisualBattleFrame {
                active: true,
                ..default()
            };
            let original = frame.clone();
            let allocation = (
                state.current.as_ptr(),
                state.target.as_ptr(),
                state.from.as_ptr(),
            );
            for _ in 0..hz * 2 {
                state.advance(rig, &frame, VisualBattleSide::Player, true, 1.0 / hz as f32);
            }
            let mut rendered = expected.clone();
            species_rig::motion::battle_float(rig, &mut rendered);
            close_pose(&state.current, &rendered);
            assert_eq!(
                frame, original,
                "render sampling must not rewrite source presentation"
            );
            frame.cues = vec![cue(VisualBattleCueKind::Move, 0.6)];
            for _ in 0..20 {
                state.advance(rig, &frame, VisualBattleSide::Player, true, 1.0 / hz as f32);
            }
            rig.sample_into(
                Clip::Attack,
                rig.clip(Clip::Attack).duration * 0.6,
                Playback::Clamp,
                &mut expected,
            )
            .unwrap();
            let mut rendered = expected.clone();
            species_rig::motion::battle_float(rig, &mut rendered);
            close_pose(&state.current, &rendered);
            // A replay/rewind samples the requested source position directly,
            // without blending a pose from the future into this earlier tick.
            frame.cues[0].progress = 0.2;
            state.advance(rig, &frame, VisualBattleSide::Player, true, 1.0 / hz as f32);
            rig.sample_into(
                Clip::Attack,
                rig.clip(Clip::Attack).duration * 0.2,
                Playback::Clamp,
                &mut expected,
            )
            .unwrap();
            let mut rendered = expected.clone();
            species_rig::motion::battle_float(rig, &mut rendered);
            close_pose(&state.current, &rendered);
            assert_eq!(
                allocation,
                (
                    state.current.as_ptr(),
                    state.target.as_ptr(),
                    state.from.as_ptr()
                )
            );
            rig.sample_into(Clip::Idle, 2.0, Playback::Loop, &mut expected)
                .unwrap();
        }
    }
}

#[test]
fn crocodile_and_small_bird_brace_early_on_the_actual_move_cue() {
    for species in [Species::Totodile, Species::Spearow] {
        let rig = species_rig::rig(species);
        for hz in [9, 30, 60] {
            let mut state = BodyPose::new(rig);
            let mut frame = VisualBattleFrame {
                active: true,
                cues: vec![cue(VisualBattleCueKind::Move, 0.0)],
                ..default()
            };
            for step in 0..=hz / 4 {
                frame.cues[0].progress = step as f32 / hz as f32;
                let original = frame.clone();
                state.advance(rig, &frame, VisualBattleSide::Player, true, 1.0 / hz as f32);
                assert_eq!(frame, original);
                if step > 0 {
                    assert!(
                        state.current[1].rotation.x > 0.0,
                        "{species:?} torso at {hz} Hz"
                    );
                    assert!(
                        state.current[2].rotation.x > 0.0,
                        "{species:?} head at {hz} Hz"
                    );
                }
                assert_eq!(state.current[0], Transform::IDENTITY);
            }
        }
    }
}

#[test]
fn one_hp_impact_gets_a_complete_local_recoil_without_replaying_after_classic_view() {
    let rig = species_rig::rig(Species::Totodile);
    let mut state = BodyPose::new(rig);
    let mut frame = VisualBattleFrame {
        active: true,
        cues: vec![cue(VisualBattleCueKind::Impact, 47.0 / 48.0)],
        ..default()
    };
    state.advance(rig, &frame, VisualBattleSide::Player, true, 0.0);
    assert_eq!(state.clip, Clip::Hit);
    assert_eq!(state.hit_seconds, 0.0);
    frame.cues.clear();
    for _ in 0..6 {
        state.advance(rig, &frame, VisualBattleSide::Player, true, 1.0 / 60.0);
    }
    assert_eq!(
        state.clip,
        Clip::Hit,
        "short real HP loss must not skip the whole reaction"
    );
    assert!(state
        .current
        .iter()
        .zip(&rig.joints)
        .any(|(pose, joint)| pose.rotation != joint.bind.rotation));
    let frozen_idle = state.idle_seconds;
    for _ in 0..120 {
        state.advance(rig, &frame, VisualBattleSide::Player, false, 1.0 / 60.0);
    }
    assert_eq!(state.idle_seconds, frozen_idle, "hidden idle clocks pause");
    assert_eq!(
        state.clip,
        Clip::Idle,
        "a completed hidden reaction must not restart on F3 return"
    );
    state.advance(rig, &frame, VisualBattleSide::Player, true, 1.0 / 60.0);
    assert_eq!(state.clip, Clip::Idle);
}

fn pair_app(hz: u32) -> App {
    let mut app = super::super::tests::headless_battle_app();
    if !app.is_plugin_added::<bevy::transform::TransformPlugin>() {
        app.add_plugins(bevy::transform::TransformPlugin);
    }
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / hz as f64,
    )));
    {
        let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
        let enemy = frame.battlers[1].as_mut().unwrap();
        enemy.species_id = Arc::from("TOTODILE");
        enemy.pokedex_size_m = Some(0.6096);
    }
    app.update();
    app.update();
    app
}

#[test]
fn gpu_skin_entities_preserve_single_placement_and_immutable_vertex_buffers() {
    assert_pair_gpu(pair_app(60));
}

#[test]
fn gengar_instances_share_bind_assets_but_have_independent_live_joints() {
    let mut app = pair_app(60);
    for battler in app
        .world_mut()
        .resource_mut::<VisualBattleFrame>()
        .battlers
        .iter_mut()
        .flatten()
    {
        battler.species_id = Arc::from("GENGAR");
        battler.pokedex_size_m = Some(1.4986);
    }
    app.update();
    app.update();
    assert_pair_gpu(app);
}

#[test]
fn spearow_instances_activate_generic_gpu_skinning_at_pack_size() {
    let mut app = pair_app(60);
    for battler in app
        .world_mut()
        .resource_mut::<VisualBattleFrame>()
        .battlers
        .iter_mut()
        .flatten()
    {
        battler.species_id = Arc::from("SPEAROW");
        battler.pokedex_size_m = Some(0.3048);
    }
    app.update();
    app.update();
    let world = app.world();
    let layout = world.resource::<BattleSceneLayout>();
    let scene = world.resource::<BattleScene>();
    let mut bind_handles = Vec::new();
    for (side, actor) in scene.actors.iter().enumerate() {
        let actor = actor.as_ref().unwrap();
        let skin = actor.skinned.as_ref().expect("Spearow must be articulated");
        assert_eq!(skin.rig.species, Species::Spearow);
        assert_eq!(skin.joints.len(), 10);
        let component = world.get::<SkinnedMesh>(actor.entity).unwrap();
        bind_handles.push(component.inverse_bindposes.clone());
        let pose = world.get::<Transform>(actor.entity).unwrap();
        let (min, max) = skin.rig.neutral_bounds;
        assert!(
            ((max.y - min.y) * pose.scale.y / crate::battle_layout::WORLD_UNITS_PER_METER - 0.3048)
                .abs()
                < 0.000001
        );
        assert_eq!(pose.scale, Vec3::splat(pose.scale.y));
        assert!(
            (pose.transform_point(Vec3::Y * min.y).y - layout.origins[side].y).abs() < 0.000001
        );
    }
    assert_eq!(bind_handles[0], bind_handles[1]);
    assert_pair_gpu(app);
}

#[test]
fn chikorita_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("CHIKORITA", Species::Chikorita, 11, 0.889);
}

#[test]
fn bayleef_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("BAYLEEF", Species::Bayleef, 16, 1.1938);
}

#[test]
fn meganium_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("MEGANIUM", Species::Meganium, 23, 1.8034);
}

#[test]
fn wooper_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("WOOPER", Species::Wooper, 9, 0.4064);
}

#[test]
fn gyarados_instances_activate_generic_gpu_skinning_at_pack_length() {
    assert_species_gpu_at_pack_size("GYARADOS", Species::Gyarados, 9, 6.5024);
}

#[test]
fn sudowoodo_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("SUDOWOODO", Species::Sudowoodo, 9, 1.1938);
}

#[test]
fn lapras_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("LAPRAS", Species::Lapras, 9, 2.4892);
}

#[test]
fn snorlax_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("SNORLAX", Species::Snorlax, 9, 2.1082);
}

#[test]
fn pikachu_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("PIKACHU", Species::Pikachu, 11, 0.4064);
}

#[test]
fn bulbasaur_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("BULBASAUR", Species::Bulbasaur, 9, 0.7112);
}

#[test]
fn charmander_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("CHARMANDER", Species::Charmander, 11, 0.6096);
}

#[test]
fn squirtle_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("SQUIRTLE", Species::Squirtle, 9, 0.508);
}

fn assert_species_gpu_at_pack_size(
    species_name: &'static str, species: Species, joint_count: usize, size_m: f32,
) {
    let mut app = pair_app(60);
    for battler in app
        .world_mut()
        .resource_mut::<VisualBattleFrame>()
        .battlers
        .iter_mut()
        .flatten()
    {
        battler.species_id = Arc::from(if matches!(species, Species::Unown(_)) {"UNOWN"} else {species_name});
        battler.unown_form = if let Species::Unown(form) = species {Some(form + 1)} else {None};
        battler.pokedex_size_m = Some(size_m);
    }
    app.update();
    app.update();
    let world = app.world();
    let layout = world.resource::<BattleSceneLayout>();
    let scene = world.resource::<BattleScene>();
    let mut bind_handles = Vec::new();
    for (side, actor) in scene.actors.iter().enumerate() {
        let actor = actor.as_ref().unwrap();
        let skin = actor
            .skinned
            .as_ref()
            .expect("registered species must be articulated");
        assert_eq!(skin.rig.species, species);
        assert_eq!(skin.joints.len(), joint_count);
        let component = world.get::<SkinnedMesh>(actor.entity).unwrap();
        bind_handles.push(component.inverse_bindposes.clone());
        let pose = world.get::<Transform>(actor.entity).unwrap();
        let (min, max) = skin.rig.neutral_bounds;
        assert!(
            (crate::battle_layout::reference_span(species_name, min, max) * pose.scale.y / crate::battle_layout::WORLD_UNITS_PER_METER - size_m)
                .abs()
                < 0.000001
        );
        assert_eq!(pose.scale, Vec3::splat(pose.scale.y));
        assert!(
            (pose.transform_point(Vec3::Y * min.y).y - layout.origins[side].y).abs() < 0.000001
        );
    }
    assert_eq!(bind_handles[0], bind_handles[1]);
    assert_pair_gpu(app);
}

fn assert_pair_gpu(mut app: App) {
    let world = app.world();
    let scene = world.resource::<BattleScene>();
    let actors: Vec<_> = scene
        .actors
        .iter()
        .flatten()
        .map(|actor| {
            let skin = actor.skinned.as_ref().unwrap();
            let component = world.get::<SkinnedMesh>(actor.entity).unwrap();
            assert_eq!(component.joints, skin.joints);
            let mesh = world
                .resource::<Assets<Mesh>>()
                .get(actor.mesh.as_ref().unwrap())
                .unwrap();
            assert!(matches!(mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX),
            Some(VertexAttributeValues::Uint16x4(values)) if values == &skin.rig.joint_indices));
            assert!(matches!(mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT),
            Some(VertexAttributeValues::Float32x4(values)) if values == &skin.rig.joint_weights));
            let aabb = world.get::<Aabb>(actor.entity).unwrap();
            assert!((Vec3::from(aabb.min()) - skin.rig.animated_bounds.0).length() < 0.000001);
            assert!((Vec3::from(aabb.max()) - skin.rig.animated_bounds.1).length() < 0.000001);
            let placement = world
                .get::<GlobalTransform>(actor.entity)
                .unwrap()
                .to_matrix();
            assert!(placement != Mat4::IDENTITY);
            let mut local = vec![Mat4::IDENTITY; skin.joints.len()];
            skin.rig
                .skin_matrices_into(&skin.pose.current, &mut local)
                .unwrap();
            for (index, &joint) in skin.joints.iter().enumerate() {
                let gpu = world
                    .get::<GlobalTransform>(joint)
                    .unwrap()
                    .to_matrix()
                    * skin.rig.joints[index].inverse_bind;
                let expected = placement * local[index];
                assert!(
                    gpu.to_cols_array()
                        .iter()
                        .zip(expected.to_cols_array())
                        .all(|(a, b)| (a - b).abs() < 0.00001),
                    "placement must reach each skin matrix exactly once"
                );
            }
            (
                actor.entity,
                actor.mesh.clone().unwrap(),
                skin.joints.clone(),
                skin.rig.neutral.positions.clone(),
            )
        })
        .collect();
    assert!(actors[0].2.iter().all(|joint| !actors[1].2.contains(joint)));
    let counts = (
        world.resource::<Assets<Mesh>>().len(),
        world.resource::<Assets<StandardMaterial>>().len(),
        world
            .resource::<Assets<SkinnedMeshInverseBindposes>>()
            .len(),
    );
    for step in 0..120 {
        app.world_mut().resource_mut::<VisualBattleFrame>().cues = if step < 60 {
            vec![cue(VisualBattleCueKind::Move, step as f32 / 60.0)]
        } else {
            Vec::new()
        };
        app.update();
    }
    let world = app.world();
    assert_eq!(
        counts,
        (
            world.resource::<Assets<Mesh>>().len(),
            world.resource::<Assets<StandardMaterial>>().len(),
            world
                .resource::<Assets<SkinnedMeshInverseBindposes>>()
                .len()
        )
    );
    for (entity, handle, joints, positions) in actors {
        assert!(world.get::<SkinnedMesh>(entity).is_some());
        assert!(joints
            .iter()
            .all(|joint| world.get::<BattleSkinJoint>(*joint).is_some()));
        let mesh = world.resource::<Assets<Mesh>>().get(&handle).unwrap();
        assert!(matches!(mesh.attribute(Mesh::ATTRIBUTE_POSITION),
            Some(VertexAttributeValues::Float32x3(values)) if values == &positions));
    }
}

#[test]
fn switching_species_retires_joint_entities_and_keeps_inverse_bind_assets_bounded() {
    let mut app = pair_app(30);
    let initial = app
        .world()
        .resource::<Assets<SkinnedMeshInverseBindposes>>()
        .len();
    for _ in 0..3 {
        let old = app.world().resource::<BattleScene>().actors[0]
            .as_ref()
            .unwrap()
            .skinned
            .as_ref()
            .unwrap()
            .joints
            .clone();
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .species_id = Arc::from("UNAUTHORED_SPECIES");
        app.update();
        app.update();
        assert!(old
            .iter()
            .all(|entity| app.world().get_entity(*entity).is_err()));
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .species_id = Arc::from("CYNDAQUIL");
        app.update();
        app.update();
        assert!(app.world().resource::<BattleScene>().actors[0]
            .as_ref()
            .unwrap()
            .skinned
            .is_some());
        assert_eq!(
            app.world()
                .resource::<Assets<SkinnedMeshInverseBindposes>>()
                .len(),
            initial
        );
    }
}

#[test]
fn geodude_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("GEODUDE", Species::Geodude, 8, 0.4064);
}

#[test]
fn pidgey_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("PIDGEY", Species::Pidgey, 10, 0.3048);
}

#[test]
fn oddish_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("ODDISH", Species::Oddish, 12, 0.508);
}

#[test]
fn poliwag_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("POLIWAG", Species::Poliwag, 5, 0.6096);
}

#[test]
fn clefairy_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("CLEFAIRY", Species::Clefairy, 11, 0.6096);
}

#[test]
fn jigglypuff_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("JIGGLYPUFF", Species::Jigglypuff, 7, 0.508);
}

#[test]
fn rattata_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("RATTATA", Species::Rattata, 9, 0.3048);
}

#[test]
fn raticate_instances_activate_generic_gpu_skinning_at_pack_size() {
    assert_species_gpu_at_pack_size("RATICATE", Species::Raticate, 8, 0.7112);
}

#[test]
fn ducks_activate_generic_gpu_skinning_at_pack_sizes() {
    assert_species_gpu_at_pack_size("PSYDUCK", Species::Psyduck, 10, 0.7874);
    assert_species_gpu_at_pack_size("GOLDUCK", Species::Golduck, 11, 1.7018);
}

#[test]
fn bats_activate_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("ZUBAT", Species::Zubat, 10, 0.7874);
    assert_species_gpu_at_pack_size("GOLBAT", Species::Golbat, 10, 1.6002);
    assert_species_gpu_at_pack_size("CROBAT", Species::Crobat, 14, 1.8034);
}

#[test]
fn wing_insects_activate_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("BUTTERFREE", Species::Butterfree, 18, 1.0922);
    assert_species_gpu_at_pack_size("BEEDRILL", Species::Beedrill, 20, 0.9906);
}

#[test]
fn hoothoot_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("HOOTHOOT", Species::Hoothoot, 12, 0.7112);
}

#[test]
fn sentret_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SENTRET", Species::Sentret, 13, 0.7874);
}

#[test]
fn staryu_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("STARYU", Species::Staryu, 12, 0.7874);
}

#[test]
fn voltorb_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("VOLTORB", Species::Voltorb, 2, 0.5080);
}

#[test]
fn onix_activates_generic_gpu_skinning_at_verified_pack_body_length() {
    assert_species_gpu_at_pack_size("ONIX", Species::Onix, 12, 8.7884);
}

#[test]
fn diglett_activates_generic_gpu_skinning_at_verified_pack_capsule_height() {
    assert_species_gpu_at_pack_size("DIGLETT", Species::Diglett, 2, 0.2032);
}

#[test]
fn slowpoke_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SLOWPOKE", Species::Slowpoke, 11, 1.1938);
}

#[test]
fn unown_all_26_forms_activate_distinct_gpu_skins_at_verified_pack_height() {
    for form in 0..26 {
        let species = Species::Unown(form);
        assert_species_gpu_at_pack_size(species.name(), species, 3, 0.5080);
    }
}

#[test]
fn unown_form_change_replaces_gpu_geometry_without_changing_species_or_party_slot() {
    let mut app = pair_app(60);
    for form in [1,26,9,22] {
        for battler in app.world_mut().resource_mut::<VisualBattleFrame>().battlers.iter_mut().flatten() {
            battler.species_id = Arc::from("UNOWN");
            battler.unown_form = Some(form);
            battler.pokedex_size_m = Some(0.5080);
        }
        app.update();app.update();
        let frame = app.world().resource::<VisualBattleFrame>();
        let scene = app.world().resource::<BattleScene>();
        for (side,actor) in scene.actors.iter().enumerate() {
            let actor = actor.as_ref().unwrap();
            assert_eq!(actor.skinned.as_ref().unwrap().rig.species, Species::Unown(form-1));
            assert_eq!(actor.key.species.as_ref(), frame.battlers[side].as_ref().unwrap().model_id());
            assert_eq!(frame.battlers[side].as_ref().unwrap().species_id.as_ref(), "UNOWN");
            assert_eq!(actor.key.party_index, frame.battlers[side].as_ref().unwrap().party_index);
        }
    }
    for battler in app.world_mut().resource_mut::<VisualBattleFrame>().battlers.iter_mut().flatten() { battler.unown_form = None; }
    app.update();app.update();
    assert!(app.world().resource::<BattleScene>().actors.iter().flatten().all(|a| !a.key.modeled));
}

#[test]
fn entei_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("ENTEI",Species::Entei,16,2.0066);
}

#[test]
fn raikou_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("RAIKOU",Species::Raikou,16,1.9050);
}

#[test]
fn suicune_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SUICUNE",Species::Suicune,18,2.0066);
}

#[test]
fn furret_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("FURRET", Species::Furret, 14, 1.8034);
}

#[test]
fn noctowl_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("NOCTOWL", Species::Noctowl, 14, 1.6002);
}

#[test]
fn ladybirds_activate_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("LEDYBA",Species::Ledyba,14,0.9906);
    assert_species_gpu_at_pack_size("LEDIAN",Species::Ledian,16,1.3970);
}

#[test]
fn spiders_activate_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("SPINARAK",Species::Spinarak,18,0.5080);
    assert_species_gpu_at_pack_size("ARIADOS",Species::Ariados,18,1.0922);
}

#[test]
fn quagsire_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("QUAGSIRE",Species::Quagsire,13,1.3970);
}

#[test]
fn hoppip_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("HOPPIP",Species::Hoppip,11,0.4064);
}

#[test]
fn skiploom_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SKIPLOOM",Species::Skiploom,18,0.6096);
}

#[test]
fn jumpluff_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("JUMPLUFF",Species::Jumpluff,13,0.7874);
}

#[test]
fn aipom_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("AIPOM",Species::Aipom,20,0.7874);
}

#[test]
fn sunflower_family_activates_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("SUNKERN",Species::Sunkern,7,0.3048);
    assert_species_gpu_at_pack_size("SUNFLORA",Species::Sunflora,25,0.7874);
}


#[test]
fn bellossom_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("BELLOSSOM",Species::Bellossom,25,0.4064);
}

#[test]
fn qwilfish_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("QWILFISH",Species::Qwilfish,7,0.508);
}

#[test]
fn wobbuffet_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("WOBBUFFET",Species::Wobbuffet,11,1.2954);
}

#[test]
fn dunsparce_activates_generic_gpu_skinning_at_verified_pack_length() {
    assert_species_gpu_at_pack_size("DUNSPARCE",Species::Dunsparce,7,1.4986);
}

#[test]
fn shuckle_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SHUCKLE",Species::Shuckle,8,0.6096);
}

#[test]
fn gligar_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("GLIGAR",Species::Gligar,15,1.0922);
}

#[test]
fn yanma_activates_generic_gpu_skinning_at_verified_pack_length() {
    let rig=crate::species_rig::rig(Species::Yanma);let (min,max)=rig.neutral_bounds;
    assert_eq!(crate::battle_layout::reference_span("YANMA",min,max),(max-min).z);
    assert_species_gpu_at_pack_size("YANMA",Species::Yanma,16,1.1938);
}

#[test]
fn murkrow_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("MURKROW",Species::Murkrow,11,0.508);
}

#[test]
fn misdreavus_activates_generic_gpu_skinning_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("MISDREAVUS",Species::Misdreavus,15,0.7112);
}

#[test]
fn bear_family_activates_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("TEDDIURSA",Species::Teddiursa,12,0.6096);
    assert_species_gpu_at_pack_size("URSARING",Species::Ursaring,12,1.8034);
}

#[test]
fn bulldog_family_activates_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("SNUBBULL",Species::Snubbull,13,0.6096);
    assert_species_gpu_at_pack_size("GRANBULL",Species::Granbull,13,1.397);
}

#[test]
fn hellhound_family_activates_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("HOUNDOUR",Species::Houndour,13,0.6096);
    assert_species_gpu_at_pack_size("HOUNDOOM",Species::Houndoom,12,1.397);
}

#[test]
fn bovine_family_activates_generic_gpu_skinning_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("MILTANK",Species::Miltank,14,1.1938);
    let rig=crate::species_rig::rig(Species::Tauros);
    let body_height=rig.anatomy.iter().filter(|p|!p.name.starts_with("Tail /")).flat_map(|p|p.vertices.clone()).map(|v|rig.neutral.positions[v][1]).fold(0.0,f32::max);
    let (min,max)=rig.neutral_bounds;
    assert!(body_height<max.y);
    assert!((crate::battle_layout::reference_span("TAUROS",min,max)-body_height).abs()<0.000001);
    assert_species_gpu_at_pack_size("TAUROS",Species::Tauros,17,1.397);
}

#[test]
fn delibird_skin_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("DELIBIRD",Species::Delibird,8,0.889);
}

#[test]
fn stantler_skin_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("STANTLER",Species::Stantler,12,1.397);
}

#[test]
fn girafarig_skin_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("GIRAFARIG",Species::Girafarig,13,1.4986);
}

#[test]
fn elephant_family_activates_generic_gpu_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("PHANPY",Species::Phanpy,13,0.508);
    assert_species_gpu_at_pack_size("DONPHAN",Species::Donphan,13,1.0922);
}

#[test]
fn water_rabbit_family_activates_generic_gpu_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("MARILL",Species::Marill,12,0.4064);
    assert_species_gpu_at_pack_size("AZUMARILL",Species::Azumarill,12,0.7874);
}

#[test]
fn psychic_bird_family_activates_generic_gpu_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("NATU",Species::Natu,10,0.2032);
    assert_species_gpu_at_pack_size("XATU",Species::Xatu,11,1.4986);
}

#[test]
fn electric_sheep_family_activates_generic_gpu_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("MAREEP",Species::Mareep,13,0.6096);
    assert_species_gpu_at_pack_size("FLAAFFY",Species::Flaaffy,13,0.7874);
    assert_species_gpu_at_pack_size("AMPHAROS",Species::Ampharos,9,1.397);
}

#[test]
fn baby_family_activates_generic_gpu_at_verified_pack_heights() {
    assert_species_gpu_at_pack_size("PICHU",Species::Pichu,10,0.3048);
    assert_species_gpu_at_pack_size("CLEFFA",Species::Cleffa,9,0.3048);
    assert_species_gpu_at_pack_size("IGGLYBUFF",Species::Igglybuff,6,0.3048);
}

#[test]
fn magby_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("MAGBY",Species::Magby,11,0.7112);
}

#[test]
fn elekid_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("ELEKID",Species::Elekid,9,0.6096);
}

#[test]
fn smoochum_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SMOOCHUM",Species::Smoochum,11,0.4064);
}

#[test]
fn tyrogue_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("TYROGUE",Species::Tyrogue,12,0.7112);
}

#[test]
fn hitmontop_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("HITMONTOP",Species::Hitmontop,11,1.397);
}

#[test]
fn hitmonchan_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("HITMONCHAN",Species::Hitmonchan,12,1.397);
}

#[test]
fn hitmonlee_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("HITMONLEE",Species::Hitmonlee,10,1.4986);
}

#[test]
fn egg_fairies_activate_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("TOGEPI",Species::Togepi,7,0.3048);
    assert_species_gpu_at_pack_size("TOGETIC",Species::Togetic,10,0.6096);
}

#[test]
fn jet_fish_activate_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("REMORAID",Species::Remoraid,7,0.6096);
    assert_species_gpu_at_pack_size("OCTILLERY",Species::Octillery,11,0.889);
}

#[test]
fn fur_pigs_activate_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SWINUB",Species::Swinub,8,0.4064);
    assert_species_gpu_at_pack_size("PILOSWINE",Species::Piloswine,9,1.0922);
}

#[test]
fn sneasel_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SNEASEL",Species::Sneasel,13,0.889);
}

#[test]
fn skarmory_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SKARMORY",Species::Skarmory,10,1.7018);
}

#[test]
fn smeargle_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SMEARGLE",Species::Smeargle,13,1.1938);
}

#[test]
fn lava_mollusks_activate_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("SLUGMA",Species::Slugma,5,0.7112);
    assert_species_gpu_at_pack_size("MAGCARGO",Species::Magcargo,6,0.7874);
}

#[test]
fn porygon2_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("PORYGON2",Species::Porygon2,6,0.6096);
}

#[test]
fn pokemon_motion_legacy_species_follow_real_battle_cues_on_independent_gpu_skins() {
    for name in ["ABRA", "CHARIZARD", "LUGIA", "MANTINE", "EKANS", "CLOYSTER", "CATERPIE"] {
        let mut app=pair_app(60);
        for b in app.world_mut().resource_mut::<VisualBattleFrame>().battlers.iter_mut().flatten() {
            b.species_id=Arc::from(name);b.pokedex_size_m=Some(1.);
        }
        app.update();app.update();
        let source=app.world().resource::<VisualBattleFrame>().clone();
        app.world_mut().resource_mut::<VisualBattleFrame>().cues=vec![cue(VisualBattleCueKind::Move,0.4)];
        for _ in 0..12 {app.update();}
        let scene=app.world().resource::<BattleScene>();
        let player=scene.actors[0].as_ref().unwrap().skinned.as_ref().expect(name);
        let enemy=scene.actors[1].as_ref().unwrap().skinned.as_ref().unwrap();
        assert_eq!(player.pose.clip,Clip::Attack);
        assert_eq!(enemy.pose.clip,Clip::Idle);
        let mut expected=vec![Transform::IDENTITY;player.rig.joints.len()];
        player.rig.sample_into(Clip::Attack,player.rig.clip(Clip::Attack).duration*0.4,Playback::Clamp,&mut expected).unwrap();
        species_rig::motion::battle_float(player.rig, &mut expected);
        close_pose(&player.pose.current,&expected);
        for (&joint,expected) in player.joints.iter().zip(&expected) {
            close_pose(&[*app.world().get::<Transform>(joint).unwrap()],&[*expected]);
        }
        assert_eq!(app.world().resource::<VisualBattleFrame>().battlers,source.battlers);
        assert_pair_gpu(app);
    }
}

#[test]
fn porygon_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("PORYGON",Species::Porygon,7,0.7874);
}

#[test]
fn anglers_activate_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("CHINCHOU",Species::Chinchou,10,0.508);
    assert_species_gpu_at_pack_size("LANTURN",Species::Lanturn,12,1.1938);
}

#[test]
fn mantine_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("MANTINE",Species::Mantine,13,2.1082);
}

#[test]
fn corsola_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("CORSOLA",Species::Corsola,9,0.6096);
}

#[test]
fn moltres_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("MOLTRES",Species::Moltres,12,2.0066);
}

#[test]
fn zapdos_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("ZAPDOS",Species::Zapdos,9,1.6002);
}

#[test]
fn articuno_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("ARTICUNO",Species::Articuno,11,1.7018);
}

#[test]
fn lugia_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("LUGIA",Species::Lugia,13,5.207);
}

#[test]
fn ho_oh_activates_generic_gpu_at_verified_pack_height() {
    assert_species_gpu_at_pack_size("HO_OH",Species::HoOh,10,3.81);
}
