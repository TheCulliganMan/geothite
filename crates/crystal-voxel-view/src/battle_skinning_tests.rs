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
            close_pose(&state.current, &expected);
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
            close_pose(&state.current, &expected);
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
            close_pose(&state.current, &expected);
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
        battler.species_id = Arc::from(species_name);
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
            ((max.y - min.y) * pose.scale.y / crate::battle_layout::WORLD_UNITS_PER_METER - size_m)
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
                .compute_matrix();
            assert!(placement != Mat4::IDENTITY);
            let mut local = vec![Mat4::IDENTITY; skin.joints.len()];
            skin.rig
                .skin_matrices_into(&skin.pose.current, &mut local)
                .unwrap();
            for (index, &joint) in skin.joints.iter().enumerate() {
                let gpu = world
                    .get::<GlobalTransform>(joint)
                    .unwrap()
                    .compute_matrix()
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
            .all(|entity| app.world().get_entity(*entity).is_none()));
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
