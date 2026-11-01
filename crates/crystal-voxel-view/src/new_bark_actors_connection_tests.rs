#[test]
fn connected_motion_preserves_the_player_rig_and_replaces_map_local_npcs() {
    use crystal_render_api::VisualSourceRegion;
    let mut app = actor_test_app();
    {
        let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
        frame.map_id = "Route29".into();
        frame.source_regions = vec![
            VisualSourceRegion {
                map_id: "Route29".into(),
                origin: IVec2::ZERO,
                size: UVec2::new(120, 36),
            },
            VisualSourceRegion {
                map_id: "CherrygroveCity".into(),
                origin: IVec2::new(-80, 0),
                size: UVec2::new(80, 36),
            },
        ]
        .into();
        let mut npc = frame.actors[0].clone();
        npc.id = VisualActorId::Object(7);
        npc.center += Vec2::new(0.0, 20.0);
        frame.actors.push(npc);
    }
    for _ in 0..30 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0]
            .center
            .x += 1.0;
        app.update();
    }
    let state = app.world().resource::<ModeledActors>();
    let player = &state.instances[&VisualActorId::Player];
    let (entity, shadow, motion) = (player.entity, player.shadow, player.motion);
    let npc = state.instances[&VisualActorId::Object(7)].entity;
    assert!(
        motion.walk_weight > 0.9,
        "fixture must be visibly walking before the seam"
    );
    {
        let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
        frame.map_id = "CherrygroveCity".into();
        frame.source_regions = vec![
            VisualSourceRegion {
                map_id: "CherrygroveCity".into(),
                origin: IVec2::ZERO,
                size: UVec2::new(80, 36),
            },
            VisualSourceRegion {
                map_id: "Route29".into(),
                origin: IVec2::new(80, 0),
                size: UVec2::new(120, 36),
            },
        ]
        .into();
        for actor in &mut frame.actors {
            actor.center.x += 640.0;
        }
        frame.actors[0].center.x += 1.0;
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
    app.update();
    let state = app.world().resource::<ModeledActors>();
    let player = &state.instances[&VisualActorId::Player];
    assert_eq!(
        player.entity, entity,
        "keep the actual rig, not a newly spawned standing pose"
    );
    assert_eq!(
        player.shadow, shadow,
        "the player's contact shadow shares the retained lifetime"
    );
    assert!(player.motion.walk_weight >= motion.walk_weight);
    let phase_advance = (player.motion.phase - motion.phase).rem_euclid(TAU);
    assert!(
        phase_advance > 0.0001 && phase_advance < 1.0,
        "the next ordinary step must advance the existing gait: {phase_advance}"
    );
    assert_ne!(
        state.instances[&VisualActorId::Object(7)].entity,
        npc,
        "the same NPC index in another map must construct the new authoritative appearance"
    );
    assert!(app.world().get_entity(npc).is_err());
}
