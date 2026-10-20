//! Read-only shot direction from the currently presented battle animation.
//! No timer, RNG or input: a held source frame always holds the same camera.
use super::*;

pub(super) fn shot(
    base: &BattleSceneLayout,
    bodies: [Option<BattleBody>; 2],
    frame: &VisualBattleFrame,
) -> BattleSceneLayout {
    // A faint is a short, source-timed picture-bank drop. Keep its established
    // camera steady rather than adding a rapid orbit around the disappearance.
    if frame.cues.iter().any(|cue| cue.kind == VisualBattleCueKind::Faint) {
        return base.clone();
    }
    // Captured source rows keep their registration. Retained-map cards have
    // an axis-aligned envelope proving every upright billboard yaw; older
    // arena cards keep the established projection instead.
    if bodies.iter().enumerate().any(|(index, body)| {
        body.is_some_and(|body| {
            !body.modeled
                && base.body_poses[index].is_none_or(|pose| pose.rotation != Quat::IDENTITY)
        })
    }) || frame
        .source
        .as_ref()
        .is_some_and(|source| source.battler_rows.iter().any(Option::is_some))
    {
        return base.clone();
    }
    let Some(cue) = frame
        .cues
        .iter()
        .filter(|cue| {
            matches!(
                cue.kind,
                VisualBattleCueKind::Move
                    | VisualBattleCueKind::Impact
                    | VisualBattleCueKind::SendOut
                    | VisualBattleCueKind::Faint
            )
        })
        .min_by_key(|cue| match cue.kind {
            VisualBattleCueKind::Impact => 0,
            VisualBattleCueKind::Faint => 1,
            VisualBattleCueKind::SendOut => 2,
            _ => 3,
        })
    else {
        return base.clone();
    };
    if !cue.progress.is_finite() {
        return base.clone();
    }
    let progress = cue.progress.clamp(0.0, 1.0);
    // Ease into a low attacking angle, track across the pair at impact, then
    // return to the establishing shot. Remain on one side of the action axis.
    let envelope = (PI * progress).sin().powi(2);
    if envelope < 0.000001 {
        return base.clone();
    }
    let direction = if cue.side == VisualBattleSide::Player {
        1.0
    } else {
        -1.0
    };
    let (lowering, orbit): (f32, f32) = match cue.kind {
        VisualBattleCueKind::Move => (10.0, 14.0),
        VisualBattleCueKind::Impact => (6.0, -18.0),
        VisualBattleCueKind::SendOut => (12.0, 10.0),
        VisualBattleCueKind::Faint => (8.0, -8.0),
        _ => return base.clone(),
    };
    let sweep = direction * (progress * 2.0 - 1.0) * orbit * envelope;
    let axis = (base.origins[1] - base.origins[0]) * Vec3::new(1.0, 0.0, 1.0);
    let Some(axis) = axis.try_normalize() else {
        return base.clone();
    };
    let back = base.camera.rotation * Vec3::Z;
    let yaw = back.dot(axis.cross(Vec3::Y)).atan2(back.dot(-axis));
    let elevation = back.y.clamp(-1.0, 1.0).asin();
    // Establishing views can already sit at six degrees. Subtracting the
    // entire send-out dip there points the camera upward through the floor.
    // Keep the orbit, but consume only the available downward headroom.
    let minimum_elevation = elevation.min(6.0_f32.to_radians());
    let shot_elevation = (elevation - lowering.to_radians() * envelope).max(minimum_elevation);
    base.battle_shot(
        bodies,
        yaw + sweep.to_radians(),
        shot_elevation,
    )
    .unwrap_or_else(|| base.clone())
}

/// Stay as close as possible to the requested shot along its angular path.
/// A terrain limit clamps the sweep rather than resetting the whole shot.
/// Every accepted candidate uses the original complete-body fit and terrain
/// proof; held source frames remain deterministic without camera history.
pub(super) fn constrain_shot(
    base: &BattleSceneLayout,
    requested: &BattleSceneLayout,
    bodies: [Option<BattleBody>; 2],
    mut accept: impl FnMut(&mut BattleSceneLayout) -> bool,
) -> Option<BattleSceneLayout> {
    let mut desired = requested.clone();
    if accept(&mut desired) { return Some(desired); }
    let mut selected = base.clone();
    if !accept(&mut selected) { return None; }
    let axis = ((base.origins[1] - base.origins[0]) * Vec3::new(1.0, 0.0, 1.0)).try_normalize()?;
    let angles = |layout: &BattleSceneLayout| {
        let back = layout.camera.rotation * Vec3::Z;
        Vec2::new(back.dot(axis.cross(Vec3::Y)).atan2(back.dot(-axis)), back.y.clamp(-1.0, 1.0).asin())
    };
    let start = angles(base);
    let end = angles(requested);
    let yaw = end.x - start.x;
    let delta = Vec2::new(yaw.sin().atan2(yaw.cos()), end.y - start.y);
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..10 {
        let fraction = (low + high) * 0.5;
        let angle = start + delta * fraction;
        let Some(mut candidate) = base.battle_shot(bodies, angle.x, angle.y) else {
            high = fraction;
            continue;
        };
        if accept(&mut candidate) {
            low = fraction;
            selected = candidate;
        } else { high = fraction; }
    }
    // Candidate checks may mutate opacity/fog. Publish the final proven state,
    // rather than leaving the renderer configured for the last rejected probe.
    accept(&mut selected).then_some(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        BattleSceneLayout,
        [Option<BattleBody>; 2],
        VisualBattleFrame,
    ) {
        let body = BattleBody::modeled(
            "TEST",
            Vec3::new(-0.3, 0.0, -0.3),
            Vec3::new(0.3, 1.0, 0.3),
            1.0,
        );
        let bodies = [Some(body); 2];
        let base = BattleSceneLayout::for_anchored_bodies(
            bodies,
            [Vec3::new(-4.0, 0.0, 0.0), Vec3::new(4.0, 0.0, 0.0)],
            Vec2::new(1280.0, 800.0),
        )
        .unwrap()
        .battle_shot(bodies, 70.0_f32.to_radians(), 24.0_f32.to_radians())
        .unwrap();
        let frame = VisualBattleFrame {
            cues: vec![VisualBattleCue {
                kind: VisualBattleCueKind::Move,
                side: VisualBattleSide::Player,
                move_id: Arc::from("TACKLE"),
                element: Arc::from("NORMAL"),
                progress: 0.5,
                damaging: true,
            }],
            ..default()
        };
        (base, bodies, frame)
    }
    #[test]
    fn faint_holds_the_establishing_camera_for_every_source_frame() {
        let (base, bodies, mut frame) = fixture();
        frame.cues[0].kind = VisualBattleCueKind::Faint;
        for tick in 0..=14 {
            frame.cues[0].progress = tick as f32 / 14.;
            assert_eq!(shot(&base, bodies, &frame), base);
        }
    }

    #[test]
    fn already_low_establishing_views_never_dip_through_the_ground() {
        let (initial, bodies, mut frame) = fixture();
        for degrees in [6.0_f32, 12.0, 24.0, 48.0] {
            let base = initial.battle_shot(bodies, 90.0_f32.to_radians(), degrees.to_radians()).unwrap();
            for kind in [VisualBattleCueKind::Move, VisualBattleCueKind::Impact,
                VisualBattleCueKind::SendOut, VisualBattleCueKind::Faint] {
                frame.cues[0].kind = kind;
                for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
                    frame.cues[0].side = side;
                    for tick in 0..=100 {
                        frame.cues[0].progress = tick as f32 / 100.0;
                        let next = shot(&base, bodies, &frame);
                        let elevation = (next.camera.rotation * Vec3::Z).y.asin();
                        assert!(elevation >= 6.0_f32.to_radians() - 0.00001,
                            "{kind:?} at {degrees} degrees dipped below the proven low view");
                        assert!(elevation <= degrees.to_radians() + 0.00001);
                        assert_eq!(next.origins, base.origins);
                        assert_eq!(next.body_poses, base.body_poses);
                        assert_eq!(next, shot(&base, bodies, &frame));
                    }
                }
            }
        }
    }

    #[test]
    fn terrain_limited_sweep_stays_continuous_and_publishes_the_final_proven_state() {
        let (base, bodies, mut frame) = fixture();
        let mut previous = base.camera.rotation;
        for progress in 0..=100 {
            frame.cues[0].progress = progress as f32 / 100.0;
            let requested = shot(&base, bodies, &frame);
            let mut published = None;
            let mut accept = |candidate: &mut BattleSceneLayout| {
                published = Some(candidate.camera);
                (candidate.camera.rotation * Vec3::Z).y.asin() >= 22.0_f32.to_radians()
            };
            let next = constrain_shot(&base, &requested, bodies, &mut accept).unwrap();
            assert_eq!(published, Some(next.camera));
            assert_eq!(next.origins, base.origins);
            assert_eq!(next.body_poses, base.body_poses);
            assert_eq!(next.hit_anchors, base.hit_anchors);
            assert!(previous.angle_between(next.camera.rotation) < 0.6_f32.to_radians(),
                "a terrain limit must not reset a low shot in one frame");
            previous = next.camera.rotation;
            let held = constrain_shot(&base, &requested, bodies, |candidate|
                (candidate.camera.rotation * Vec3::Z).y.asin() >= 22.0_f32.to_radians()).unwrap();
            assert_eq!(next, held);
        }
        assert!(constrain_shot(&base, &base, bodies, |_| false).is_none());
    }

    #[test]
    fn source_sprite_can_keep_literal_map_floor_without_claiming_a_model() {
        let (_, mut bodies, _) = fixture();
        let mut card = BattleBody::source_card(1.0);
        card.min.z = card.min.x;
        card.max.z = card.max.x;
        bodies[1] = Some(card);
        let anchors = [Vec3::new(-4.0, 0.0, 0.0), Vec3::new(4.0, 0.0, 0.0)];
        assert!(
            BattleSceneLayout::for_anchored_bodies(bodies, anchors, Vec2::new(1280.0, 800.0))
                .is_none()
        );
        let map =
            BattleSceneLayout::for_map_bodies(bodies, anchors, Vec2::new(1280.0, 800.0)).unwrap();
        assert_eq!(map.origins, anchors);
        let (_, _, frame) = fixture();
        let moving = shot(&map, bodies, &frame);
        assert_ne!(moving.camera, map.camera);
        assert_eq!(moving.body_poses, map.body_poses);

        assert_eq!(map.body_poses[1].unwrap().rotation, Quat::IDENTITY);
        assert_eq!(
            map.body_poses[1].unwrap().translation.y + card.support_y,
            anchors[1].y
        );
        // The envelope is wider than the actual sprite at every camera yaw.
        for degree in 0..360 {
            let rotation = Quat::from_rotation_y((degree as f32).to_radians());
            for x in [-0.95, 0.95] {
                let point = rotation * Vec3::new(x, 0.0, 0.0);
                assert!(point.x >= card.min.x && point.x <= card.max.x);
                assert!(point.z >= card.min.z && point.z <= card.max.z);
            }
        }
    }

    #[test]
    fn reverse_shots_keep_screen_order_and_original_source_registration() {
        let (initial, bodies, mut frame) = fixture();
        let base = initial.battle_shot(bodies, -70.0_f32.to_radians(), 24.0_f32.to_radians()).unwrap();
        let on_right = |layout: &BattleSceneLayout|
            (layout.origins[0] - layout.origins[1]).dot(*layout.camera.right()) > 0.0;
        assert!(on_right(&base));
        for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
            frame.cues[0].side = side;
            for progress in 0..=100 {
                frame.cues[0].progress = progress as f32 / 100.0;
                let next = shot(&base, bodies, &frame);
                assert_eq!(on_right(&next), on_right(&base), "a sweep must not cross the action axis");
                assert_eq!(next.body_poses, base.body_poses);
                assert_eq!(next.origins, base.origins);
                for (body, pose) in bodies.into_iter().zip(next.body_poses) {
                    for point in body.unwrap().corners(pose.unwrap()) {
                        let uv = next.project_point(point, Vec2::new(1280.0, 800.0));
                        assert!(uv.x >= 0.085 && uv.x <= 0.915 && uv.y >= 0.225 && uv.y <= 0.755);
                    }
                }
            }
        }
    }

    #[test]
    fn low_shots_hold_exact_feet_facing_size_and_source_frame() {
        let (base, bodies, mut frame) = fixture();
        for kind in [
            VisualBattleCueKind::Move,
            VisualBattleCueKind::Impact,
            VisualBattleCueKind::SendOut,
            VisualBattleCueKind::Faint,
        ] {
            frame.cues[0].kind = kind;
            for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
                frame.cues[0].side = side;
                for i in 0..=100 {
                    frame.cues[0].progress = i as f32 / 100.0;
                    let next = shot(&base, bodies, &frame);
                    assert_eq!(next, shot(&base, bodies, &frame));
                    assert_eq!(next.origins, base.origins);
                    assert_eq!(next.body_poses, base.body_poses);
                    assert_eq!(next.hit_anchors, base.hit_anchors);
                    assert!(next.camera.translation.is_finite());
                    for (body, pose) in bodies.into_iter().zip(next.body_poses) {
                        for point in body.unwrap().corners(pose.unwrap()) {
                            let uv = next.project_point(point, Vec2::new(1280.0, 800.0));
                            assert!(
                                uv.x >= 0.085 && uv.x <= 0.915 && uv.y >= 0.225 && uv.y <= 0.755,
                                "shot clips a battler: {uv:?}"
                            );
                        }
                    }
                    assert!(
                        (next.camera.rotation * Vec3::Z).y
                            <= (base.camera.rotation * Vec3::Z).y + 0.0001
                    );
                    // Opponents remain facing each other while the camera moves.
                    let axis = (next.origins[1] - next.origins[0]).normalize();
                    assert!((next.body_poses[0].unwrap().rotation * Vec3::Z).dot(axis) > 0.99);
                    assert!((next.body_poses[1].unwrap().rotation * Vec3::Z).dot(-axis) > 0.99);
                }
            }
        }
        frame.cues.clear();
        assert_eq!(shot(&base, bodies, &frame), base);
    }
}
