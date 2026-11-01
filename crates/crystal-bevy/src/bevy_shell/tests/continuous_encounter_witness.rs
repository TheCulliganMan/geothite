fn continuous_encounter_projection_fixture() -> (
    BevyRuntimeShell,
    RenderedViewport,
    crystal_render_api::VisualWorldFrame,
    VisibleBoundWalkingEncounter,
) {
    let (shell, mut rendered, mut frame, bound) = walking_anchor_projection_fixture();
    let step = visual_facing(bound.origin.source.facing) * frame.tile_size * 2.0;
    let displacement = Vec2::new(-12.0, 8.0);
    let facing = Vec2::new(-3.0, 1.0).normalize();
    rendered.tile = Some(bound.from);
    rendered.snapshot_revision = Some(bound.minimum_snapshot_revision.wrapping_sub(1));
    rendered.player_sprite_facing = Some(Direction::Left);
    rendered.visual_player_displacement = displacement;
    rendered.visual_player_facing = Some(facing);
    let player = frame
        .actors
        .iter_mut()
        .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
        .expect("fixture has a player");
    player.center += displacement - step * 0.5;
    player.facing = Some(facing);
    (shell, rendered, frame, bound)
}

#[test]
fn continuous_encounter_waits_for_committed_step_without_replacing_its_witness() {
    let (shell, rendered, frame, mut bound) = continuous_encounter_projection_fixture();
    let session = shell.shell.session().clone();
    let origin = bound.origin.clone();
    freeze_visible_walking_anchors(&mut bound, &rendered, Some(&frame), true);
    assert!(bound.warm_checked);
    assert!(!bound.capture_attempted);
    assert!(bound.anchors.is_none());
    assert!(bound.placement.witnessed_player_foot.is_none());

    let (_, current, committed, _) = walking_anchor_projection_fixture();
    freeze_visible_walking_anchors(&mut bound, &current, Some(&committed), true);
    let anchors = bound
        .anchors
        .as_ref()
        .expect("committed step owns the capture");
    assert!(anchors.terrain.matches_built_frame(&committed));
    assert!(Arc::ptr_eq(&origin, &bound.origin));
    assert_eq!(shell.shell.session(), &session);
}

#[test]
fn continuous_encounter_rejects_unpublished_or_incoherent_pose_without_late_repair() {
    for defect in 0..7 {
        let (_, mut rendered, mut frame, mut bound) = continuous_encounter_projection_fixture();
        match defect {
            0 => rendered.visual_player_facing = None,
            1 => rendered.visual_player_facing = Some(Vec2::X),
            2 => rendered.visual_player_facing = Some(Vec2::splat(f32::NAN)),
            3 => rendered.visual_player_displacement.x = TILE_SIZE * 2.0,
            4 => {
                frame
                    .actors
                    .iter_mut()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .expect("fixture has a player")
                    .center += Vec2::ONE;
            }
            5 => frame.terrain_revision += 1,
            6 => frame.map_id = Arc::from("OtherMap"),
            _ => unreachable!(),
        }
        freeze_visible_walking_anchors(&mut bound, &rendered, Some(&frame), true);
        assert!(bound.capture_attempted, "defect {defect}");
        assert!(bound.anchors.is_none(), "defect {defect}");
        let (_, current, committed, _) = walking_anchor_projection_fixture();
        freeze_visible_walking_anchors(&mut bound, &current, Some(&committed), true);
        assert!(bound.anchors.is_none(), "late repair of defect {defect}");
    }
}

#[test]
fn continuous_encounter_cannot_extend_the_original_entry_window() {
    let (_, rendered, frame, mut bound) = continuous_encounter_projection_fixture();
    freeze_visible_walking_anchors(&mut bound, &rendered, Some(&frame), true);
    assert!(!bound.capture_attempted);
    freeze_visible_walking_anchors(&mut bound, &rendered, Some(&frame), false);
    assert!(bound.capture_attempted);
    let (_, current, committed, _) = walking_anchor_projection_fixture();
    freeze_visible_walking_anchors(&mut bound, &current, Some(&committed), true);
    assert!(bound.anchors.is_none());
}
