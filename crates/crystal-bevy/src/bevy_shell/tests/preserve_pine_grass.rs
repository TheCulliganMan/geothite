//! Local setup followed exclusively by native controller input.
use super::*;
const ENTRY: &str = "MnPreservePineEntry";

#[test]
#[ignore = "requires reviewed Preserve Pine pack and reached-save directory"]
fn preserve_pine_grass_native_grassy_climb_dry_return_and_reverse_denial() {
    let (_, r, mut c) = fixture(ENTRY, (17, 24), true, true, false);
    assert!(terrain_path(&r, &mut c, (17, 21), false, true) >= 7);
    step(
        &mut c,
        GameButton::Down,
        ENTRY,
        (17, 22),
        "dry pine ledge lip",
    );
    step(
        &mut c,
        GameButton::Down,
        ENTRY,
        (17, 24),
        "native dry pine descent",
    );
    for _ in 0..3 {
        c.press(GameButton::Up).expect("native reverse attempt");
        c.wait_frames(16).expect("settled reverse");
        settle(&mut c);
        assert_at(&mut c, ENTRY, (17, 24), "native reverse remains blocked");
    }
    save(&mut c, "preserve-pine-hop");
}

#[test]
#[ignore = "requires reviewed Preserve Pine pack and reached-save directory"]
fn preserve_pine_grass_native_both_lanes_notice_free_returns_and_continue() {
    for lane in 0..2 {
        let (root, r, mut c) = fixture(ENTRY, (16 + lane, 31), true, true, false);
        assert!(terrain_path(&r, &mut c, (16 + lane, 0), false, true) >= 13);
        let height = i16::try_from(r.data().maps["MnBoundarySafariC00R02"].attributes.height * 2)
            .expect("court height");
        let money = c.shell.shell.session().state().money;
        step(
            &mut c,
            GameButton::Up,
            "MnBoundarySafariC00R02",
            (14 + lane, height - 1),
            "free court seam",
        );
        assert!(c.snapshot().expect("public court").safari.is_none());
        assert_eq!(c.shell.shell.session().state().money, money);
        step(
            &mut c,
            GameButton::Down,
            ENTRY,
            (16 + lane, 0),
            "free court return",
        );
        terrain_path(&r, &mut c, (13, 12), false, true);
        c.press(GameButton::Up).expect("face notice");
        c.wait_frames(16).expect("face settles");
        c.press(GameButton::A).expect("read notice");
        c.wait_frames(120).expect("print notice");
        assert!(c.snapshot().expect("notice owns input").ui.text_window_open);
        c.press(GameButton::Right).expect("notice rejects walking");
        assert_at(&mut c, ENTRY, (13, 12), "notice input ownership");
        let pages = settle(&mut c).join("\n");
        assert!(
            pages.contains("PRESERVE ENTRY")
                && pages.contains("closed.")
                && pages.contains("No fee is charged."),
            "{pages}"
        );
        assert!(terrain_path(&r, &mut c, (16 + lane, 31), false, true) >= 6);
        step(
            &mut c,
            GameButton::Down,
            "MnBoundaryWaters",
            (32 + lane, 0),
            "basecamp seam",
        );
        step(
            &mut c,
            GameButton::Up,
            ENTRY,
            (16 + lane, 31),
            "basecamp return",
        );
        assert!(
            c.snapshot()
                .expect("ordinary public trail")
                .safari
                .is_none()
        );
        assert_eq!(c.shell.shell.session().state().money, money);
        let before = c.snapshot().expect("reached public trail");
        let path = save(&mut c, "preserve-pine-ports");
        let mut loaded =
            VisibleShellController::load_save(root, r, path, None).expect("native Continue");
        let after = loaded.snapshot().expect("continued public trail");
        assert_eq!(before.overworld.map_name, after.overworld.map_name);
        assert_eq!(before.overworld.tile, after.overworld.tile);
        assert_eq!(before.party, after.party);
        assert_eq!(before.bag, after.bag);
        assert_eq!(
            before.progression.active_event_flags,
            after.progression.active_event_flags
        );
    }
}

#[test]
#[ignore = "requires reviewed Preserve Pine pack and reached-save directory"]
fn preserve_pine_grass_native_wild_run_resources_and_continue() {
    use super::super::super::authored_route_fixture::escape_wild;
    let (root, r, mut c) = fixture(ENTRY, (16, 0), true, true, false);
    terrain_path(&r, &mut c, (16, 2), false, true);
    let mut encountered = false;
    for index in 0..256 {
        let (input, target) = if index % 2 == 0 {
            (GameButton::Right, (17, 2))
        } else {
            (GameButton::Left, (16, 2))
        };
        assert_eq!(kind(&r, ENTRY, target), "TALL_GRASS");
        for _ in 0..8 {
            c.press(input).expect("native new grass step");
            c.wait_frames(16).expect("grass presentation");
            if c.snapshot().expect("wild snapshot").battle.is_some() {
                escape_wild(&mut c);
                encountered = true;
            }
            if at(&mut c) == (ENTRY.to_owned(), target) {
                break;
            }
        }
        assert_at(&mut c, ENTRY, target, "actual new grass movement");
        if encountered {
            break;
        }
    }
    assert!(encountered, "new habitat must start a native wild battle");
    terrain_path(&r, &mut c, (16, 0), false, true);
    let before = c.snapshot().expect("reached state");
    let path = save(&mut c, "preserve-pine-grass");
    let mut loaded =
        VisibleShellController::load_save(root, r, path, None).expect("native Continue");
    let after = loaded.snapshot().expect("continued state");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
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
    if let Some(path) = std::env::var_os("CRYSTAL_PRESERVE_PINE_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached Preserve Pine save");
    }
}
