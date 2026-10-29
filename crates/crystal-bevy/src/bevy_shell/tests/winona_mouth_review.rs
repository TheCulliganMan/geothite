//! Explicit local party/badges/rod setup; all later travel and casts use input.
use super::*;
const MOUTH: &str = "MnMississippiWinonaMouth";

fn continued(
    root: AssetRoot,
    r: CrystalRuntime,
    c: &mut VisibleShellController,
    label: &str,
) -> VisibleShellController {
    let before = c.snapshot().expect("reached mouth");
    let registered = c.shell.shell.session().state().registered_key_item.clone();
    let money = c.shell.shell.session().state().money;
    let mom = c.shell.shell.session().state().moms_money;
    let path = save(c, label);
    let mut loaded =
        VisibleShellController::load_save(root, r, path, None).expect("native Continue");
    let after = loaded.snapshot().expect("continued mouth");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.overworld.facing, after.overworld.facing);
    assert_eq!(
        loaded.shell.shell.session().state().registered_key_item,
        registered
    );
    assert_eq!(loaded.shell.shell.session().state().money, money);
    assert_eq!(loaded.shell.shell.session().state().moms_money, mom);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.progression.badges, after.progression.badges);
    assert_eq!(
        before.progression.active_engine_flags,
        after.progression.active_engine_flags
    );
    assert_eq!(
        before.progression.active_event_flags,
        after.progression.active_event_flags
    );
    loaded
}

#[test]
#[ignore = "requires current mouth review pack and retained-save directory"]
fn winona_mouth_review_native_all_river_lanes_bank_and_surf_continue() {
    let (root, r, mut c) = fixture(MOUTH, (17, 3), true, true, false);
    terrain_path(&r, &mut c, (12, 2), false, false);
    terrain_path(&r, &mut c, (21, 3), false, false);
    mount(&mut c, GameButton::Down, MOUTH, (21, 4));
    let north_height = i16::try_from(
        r.data().maps["MnMississippiLowerWoodsC00R02"]
            .attributes
            .height
            * 2,
    )
    .expect("north height");
    for x in 26..=29 {
        path(&r, &mut c, (x, 0), true);
        step(
            &mut c,
            GameButton::Up,
            "MnMississippiLowerWoodsC00R02",
            (x + 10, north_height - 1),
            "actual north water seam",
        );
        assert_eq!(
            c.snapshot().expect("north Surf").overworld.mode,
            MovementMode::Surf
        );
        step(
            &mut c,
            GameButton::Down,
            MOUTH,
            (x, 0),
            "actual north river return",
        );
        path(&r, &mut c, (x, 7), true);
        step(
            &mut c,
            GameButton::Down,
            "MnWinona",
            (x, 0),
            "actual Winona water seam",
        );
        assert_eq!(
            c.snapshot().expect("town Surf").overworld.mode,
            MovementMode::Surf
        );
        step(
            &mut c,
            GameButton::Up,
            MOUTH,
            (x, 7),
            "actual Winona river return",
        );
    }
    c = continued(root.clone(), r.clone(), &mut c, "winona-mouth-surf");
    path(&r, &mut c, (17, 4), true);
    step(
        &mut c,
        GameButton::Up,
        MOUTH,
        (17, 3),
        "real dry-bank dismount",
    );
    assert_eq!(
        c.snapshot().expect("dry bank").overworld.mode,
        MovementMode::Normal
    );
    let mut loaded = continued(root, r, &mut c, "winona-mouth-bank");
    if let Some(path) = std::env::var_os("CRYSTAL_WINONA_MOUTH_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retained actual bank save");
    }
}

#[test]
#[ignore = "requires current mouth review pack and retained-save directory"]
fn winona_mouth_review_native_bank_fishing_bite_run_and_continue() {
    use super::super::super::authored_route_fixture::escape_wild;
    let (root, r, mut c) = fixture_with_items_on_runtime(
        load(),
        MOUTH,
        (17, 3),
        false,
        false,
        false,
        &[("OLD_ROD", 1)],
    );
    c.shell
        .shell
        .register_key_item("OLD_ROD")
        .expect("initial carried registered rod fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Setup ends here. Neither pose nor RNG nor party is rewritten below.
    c.press(GameButton::Down).expect("face actual river bank");
    c.wait_frames(16).expect("facing settles");
    assert_at(&mut c, MOUTH, (17, 3), "dry casting bank");
    let mut encountered = false;
    for _ in 0..32 {
        c.press(GameButton::Select)
            .expect("native registered rod cast");
        assert!(c.shell.visible_fishing_animation.is_some());
        for _ in 0..113 {
            if c.shell
                .visible_fishing_animation
                .as_ref()
                .is_some_and(|a| a.phase == crate::bevy_shell::VisibleFishingPhase::AwaitText)
            {
                break;
            }
            c.wait_frames(1).expect("native casting clock");
            assert_at(&mut c, MOUTH, (17, 3), "casting retains bank");
        }
        for _ in 0..64 {
            if c.shell.visible_fishing_animation.is_none() {
                break;
            }
            c.press(GameButton::A)
                .expect("acknowledge native cast result");
            c.wait_frames(16).expect("cast result presentation");
        }
        if let Some(battle) = c.snapshot().expect("native cast result").battle {
            assert!(matches!(
                battle.kind,
                crystal_runtime::RuntimeBattleKind::Wild { .. }
            ));
            assert!(matches!(
                battle.enemy_pokemon.species.id.as_str(),
                "CHORIP" | "SCUTLET"
            ));
            escape_wild(&mut c);
            encountered = true;
        }
        settle(&mut c);
        assert_at(&mut c, MOUTH, (17, 3), "cast and RUN retain bank");
        if encountered {
            break;
        }
    }
    assert!(
        encountered,
        "actual pond-group fish must bite through native input"
    );
    assert_eq!(
        c.snapshot().expect("finished fishing").overworld.mode,
        MovementMode::Normal
    );
    continued(root, r, &mut c, "winona-mouth-fishing");
}
