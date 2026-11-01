//! Explicit local fixtures followed exclusively by native input.
use super::*;
fn initial(point: Point, injured: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = fixture(FIELD, point, false);
    for species in ["VOYAGEAR", "DOZEBEAR"] {
        let partner = r
            .data()
            .create_pokemon(species, 40, crystal_core::models::Dv::default())
            .expect("initial regional partner");
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
    }
    if injured {
        let state = c.shell.shell.session_mut().state_mut();
        for p in state.storage.party.pokemon.iter_mut().flatten() {
            p.hp = p.max_hp.saturating_sub(20);
            p.status = Some("PARALYSIS".into());
            for m in &mut p.moves {
                m.current_pp = 1;
            }
        }
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, r, c)
}
#[test]
#[ignore = "requires reviewed Rainy grass descendant and retained native saves"]
fn rainy_grass_native_long_slides_grassy_shelter_walk_healing_return_continue() {
    let (root, r, mut c) = initial((54, 36), true);
    let before = flags(&c);
    use_max_repel(&mut c, &r);
    slide(&mut c, &r, "lower_west");
    slide(&mut c, &r, "lower_east");
    slide(&mut c, &r, "lower_west");
    assert_eq!(walk_course(&mut c, &r, "west_stop_to_hut", false), 12);
    step(&mut c, GameButton::Up, HUT, (5, 6), "real shelter door");
    shelter_heal(&mut c, &r);
    capture_save(&mut c, "rainy-hut-healed");
    c = continue_save(root, r.clone(), &mut c);
    step(&mut c, GameButton::Down, HUT, (5, 7), "real exit carpet");
    step(
        &mut c,
        GameButton::Down,
        FIELD,
        (9, 28),
        "real passive outdoor arrival",
    );
    assert_eq!(walk_course(&mut c, &r, "west_stop_to_hut", true), 12);
    slide(&mut c, &r, "lower_east");
    assert_eq!(flags(&c), before);
    capture_save(&mut c, "rainy-lower-ice-return");
}
#[test]
#[ignore = "requires reviewed Rainy grass descendant and retained native saves"]
fn rainy_grass_native_full_snow_route_two_trainers_rewards_repeat_continue() {
    let (root, r, mut c) = initial((37, 36), false);
    use_max_repel(&mut c, &r);
    assert_eq!(walk_course(&mut c, &r, "ordinary_snow_return", false), 24);
    assert_eq!(walk_course(&mut c, &r, "east_stop_to_skier", false), 0);
    trainer(
        &mut c,
        &r,
        "MN_ICE_SKIER_ELSA",
        "EVENT_MN_ICE_SKIER_ELSA",
        GameButton::Right,
    );
    assert_eq!(walk_course(&mut c, &r, "skier_to_boarder", false), 12);
    trainer(
        &mut c,
        &r,
        "MN_ICE_BOARDER_NILS",
        "EVENT_MN_ICE_BOARDER_NILS",
        GameButton::Down,
    );
    capture_save(&mut c, "rainy-both-trainer-victories");
    c = continue_save(root, r, &mut c);
    assert!(event(&c, "EVENT_MN_ICE_SKIER_ELSA") && event(&c, "EVENT_MN_ICE_BOARDER_NILS"));
}
#[test]
#[ignore = "requires reviewed Rainy grass descendant and retained native saves"]
fn rainy_grass_native_reached_new_grass_checkpoint_without_repel() {
    let (root, r, mut c) = initial((54, 36), false);
    for edge in plan()["courses"]["east_stop_to_skier"]
        .as_array()
        .expect("dry course")
    {
        let target = point(&edge["to_core"]);
        assert_eq!(collision(&r, &mut c, target), "FLOOR");
        step(
            &mut c,
            button(edge["button"].as_str().expect("input")),
            FIELD,
            target,
            "dry native approach without Repel",
        );
    }
    assert_at(&mut c, FIELD, (46, 22), "reached dry approach beside Elsa");
    assert_eq!(
        c.snapshot()
            .expect("no Repel used")
            .progression
            .repel_steps_remaining,
        0
    );
    capture_save(&mut c, "rainy-new-grass-approach");
    c = continue_save(root, r, &mut c);
    assert_at(&mut c, FIELD, (46, 22), "actual checkpoint Continue");
}
#[test]
#[ignore = "requires reviewed Rainy grass descendant and retained native saves"]
fn rainy_grass_native_both_r20_seam_lanes_continue() {
    let (root, r, mut c) = initial((63, 44), false);
    for (input, map, p) in [
        (GameButton::Right, R20, (0, 44)),
        (GameButton::Down, R20, (0, 45)),
        (GameButton::Left, FIELD, (63, 45)),
        (GameButton::Up, FIELD, (63, 44)),
        (GameButton::Down, FIELD, (63, 45)),
        (GameButton::Right, R20, (0, 45)),
        (GameButton::Up, R20, (0, 44)),
        (GameButton::Left, FIELD, (63, 44)),
    ] {
        step(&mut c, input, map, p, "actual reciprocal winter seam");
    }
    capture_save(&mut c, "rainy-both-seam-lanes");
    c = continue_save(root, r, &mut c);
    assert_at(&mut c, FIELD, (63, 44), "continued seam journey");
}
#[test]
#[ignore = "requires reviewed Rainy grass descendant and retained native saves"]
fn rainy_grass_native_cache_ice_gap_wall_stop_item_once_continue() {
    winter_region_controller_cache_requires_ice_once_wall_stop_turn_and_continue();
}
