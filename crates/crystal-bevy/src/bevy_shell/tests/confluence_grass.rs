//! Explicit local party and badge fixtures, then production inputs only.
use super::*;
const BANK: &str = "MnMississippiConfluenceC02R00";
const REED: &str = "MnMississippiConfluenceC01R00";

fn initial(map: &str, p: Point) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = fixture(map, p, true, true, false);
    for species in ["VOYAGEAR", "DOZEBEAR"] {
        let partner = r
            .data()
            .create_pokemon(species, 40, crystal_core::models::Dv::default())
            .expect("initial native partner fixture");
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, r, c)
}
fn continued(
    root: AssetRoot,
    r: CrystalRuntime,
    mut c: VisibleShellController,
    label: &str,
) -> VisibleShellController {
    let before = c.snapshot().expect("reached state");
    let reached = save(&mut c, label);
    let mut loaded =
        VisibleShellController::load_save(root, r, reached, None).expect("native Continue");
    let after = loaded.snapshot().expect("continued state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(
        before.progression.active_event_flags,
        after.progression.active_event_flags
    );
    loaded
}
#[test]
#[ignore = "requires reviewed Confluence pack and retained saves"]
fn confluence_native_two_grassy_bank_loops_dry_drops_reverse_denial_continue() {
    for (x, expected) in [(40, 5), (41, 4)] {
        let (root, r, mut c) = initial(BANK, (42, 11));
        use_max_repel(&mut c, &r);
        mount(&mut c, GameButton::Down, BANK, (42, 12));
        step(&mut c, GameButton::Up, BANK, (42, 11), "native dismount");
        assert_eq!(terrain_path(&r, &mut c, (x, 8), false, true), 0);
        assert_eq!(terrain_path(&r, &mut c, (x, 5), false, true), expected);
        step(&mut c, GameButton::Down, BANK, (x, 6), "dry lip");
        step(&mut c, GameButton::Down, BANK, (x, 8), "native bank hop");
        for _ in 0..3 {
            c.press(GameButton::Up).expect("reverse input");
            c.wait_frames(16).expect("reverse settles");
            settle(&mut c);
            assert_at(&mut c, BANK, (x, 8), "reverse remains blocked");
        }
        terrain_path(&r, &mut c, (42, 11), false, true);
        continued(root, r, c, &format!("confluence-bank-lane-{x}"));
    }
}
#[test]
#[ignore = "requires reviewed Confluence pack and retained saves"]
fn confluence_native_new_grass_reached_checkpoint_without_repel() {
    let (root, r, mut c) = initial(BANK, (42, 11));
    assert_eq!(terrain_path(&r, &mut c, (44, 6), false, true), 0);
    assert_eq!(
        c.snapshot()
            .expect("no Repel used")
            .progression
            .repel_steps_remaining,
        0
    );
    continued(root, r, c, "confluence-new-grass-approach");
}
#[test]
#[ignore = "requires reviewed Confluence pack and retained saves"]
fn confluence_native_reed_grassy_approach_battle_rewards_repeat_continue() {
    let (root, r, mut c) = initial(REED, (15, 7));
    use_max_repel(&mut c, &r);
    mount(&mut c, GameButton::Down, REED, (15, 8));
    step(
        &mut c,
        GameButton::Up,
        REED,
        (15, 7),
        "native Reed bank dismount",
    );
    assert_eq!(terrain_path(&r, &mut c, (14, 3), false, true), 1);
    c.press(GameButton::Left).expect("face Reed");
    c.wait_frames(16).expect("facing settles");
    let before = c.snapshot().expect("before Reed");
    let money = c.shell.shell.session().state().money + c.shell.shell.session().state().moms_money;
    c.press(GameButton::A).expect("speak with Reed");
    assert!(super::super::super::rochester_earned::try_battle(
        &mut c,
        &r,
        "MN_RIVER_CONFLUENCE_FISHER",
        "EVENT_MN_RIVER_CONFLUENCE_FISHER_DEFEATED"
    ));
    settle(&mut c);
    let won = c.snapshot().expect("Reed won");
    assert!(
        won.party.slots[0].pokemon.moves[0].current_pp
            < before.party.slots[0].pokemon.moves[0].current_pp
    );
    let after_money =
        c.shell.shell.session().state().money + c.shell.shell.session().state().moms_money;
    assert!(after_money > money);
    c = continued(root, r, c, "confluence-reed-victory");
    c.press(GameButton::A).expect("repeat Reed");
    settle(&mut c);
    let again = c.snapshot().expect("repeat settled");
    assert!(again.battle.is_none());
    assert_eq!(again.party, won.party);
    assert_eq!(again.bag, won.bag);
    assert_eq!(
        c.shell.shell.session().state().money + c.shell.shell.session().state().moms_money,
        after_money
    );
}
#[test]
#[ignore = "requires reviewed Confluence pack and written native water course"]
fn confluence_native_all_four_water_chunks_st_paul_mouth_return_continue() {
    let plan: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var_os("CRYSTAL_CONFLUENCE_WATER_PLAN").expect("written course"))
            .expect("course bytes"),
    )
    .expect("course JSON");
    let (root, r, mut c) = initial(REED, (15, 7));
    use_max_repel(&mut c, &r);
    mount(&mut c, GameButton::Down, REED, (15, 8));
    for target in plan["course"]
        .as_array()
        .expect("water course")
        .iter()
        .skip(1)
    {
        if c.snapshot()
            .expect("native Repel state")
            .progression
            .repel_steps_remaining
            == 0
        {
            use_max_repel(&mut c, &r);
        }
        let button = match target["direction"].as_str().expect("direction") {
            "up" => GameButton::Up,
            "down" => GameButton::Down,
            "left" => GameButton::Left,
            "right" => GameButton::Right,
            _ => panic!("unknown course input"),
        };
        let p = (
            i16::try_from(target["point"][0].as_i64().expect("x")).expect("x range"),
            i16::try_from(target["point"][1].as_i64().expect("y")).expect("y range"),
        );
        step(
            &mut c,
            button,
            target["map"].as_str().expect("map"),
            p,
            "native four-chunk water course",
        );
        assert_eq!(
            c.snapshot().expect("water mode").overworld.mode,
            MovementMode::Surf
        );
    }
    step(
        &mut c,
        GameButton::Up,
        REED,
        (15, 7),
        "real return dismount",
    );
    continued(root, r, c, "confluence-four-chunk-water-return");
}
