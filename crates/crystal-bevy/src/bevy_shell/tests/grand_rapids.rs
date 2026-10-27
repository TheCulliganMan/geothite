//! Explicit upstream custody/partner fixtures, then native inputs and Continue.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyRuntimeShell, BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, indoor_walk};

const TOWN: &str = "MnGrandRapids";
const DEPOT: &str = "MnGrandRapidsSurveyDepot";
const CLINIC: &str = "MnGrandRapidsPokecenter1F";
const MART: &str = "MnGrandRapidsMart";
const DEFEATED: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
const RECOVERED: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
const CLEARED: &str = "EVENT_MN_DEPOT_CLEARED";
const REPORT: &str = "EVENT_MN_HEADWATERS_REPORT";
const R09: &str = "MnR09C01R00";
const R10: &str = "MnR10C00R00";

fn fixture(defeated: bool, recovered: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    fixture_at(defeated, recovered, true, true, TOWN, (6, 24))
}
fn fixture_at(
    defeated: bool,
    recovered: bool,
    badge: bool,
    report: bool,
    map: &str,
    start: Point,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_GRAND_RAPIDS_TEST_PACK")
            .expect("published external Grand Rapids pack"),
    )
    .canonicalize()
    .expect("canonical input");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("verified external pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: map.into(),
            tile_x: start.0,
            tile_y: start.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit initial local placement");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("initial name");
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["MEW", "CYNDAQUIL"] {
        let mut p = runtime
            .data()
            .create_pokemon(species, 40, Dv::default())
            .expect("legal partner");
        p.hp = p.max_hp.saturating_sub(15).max(1);
        p.status = Some("PARALYSIS".into());
        for m in &mut p.moves {
            m.current_pp = 0;
        }
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    state.money = 10000;
    state.badges.johto[..6].fill(true);
    state.badges.johto[6] = badge;
    state
        .flags
        .set_event_flag(REPORT, report)
        .expect("explicit upstream report fixture");
    for (i, flag) in BADGE_ENGINE_FLAGS.iter().enumerate() {
        state
            .flags
            .set_engine_flag(flag, i < 6 || (i == 6 && badge))
            .expect("explicit badge fixture flags");
    }
    state
        .flags
        .set_event_flag(DEFEATED, defeated)
        .expect("explicit upstream win fixture");
    state
        .flags
        .set_event_flag(RECOVERED, recovered)
        .expect("explicit upstream recovery fixture");
    assert_eq!(state.flags.is_event_flag_set(CLEARED), Ok(false));
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    (root, runtime, c)
}
fn press(c: &mut VisibleShellController, b: GameButton) {
    c.press(b).expect("native input");
    c.wait_frames(16).expect("presentation");
    assert_eq!(c.shell.last_error, None);
}
fn event(c: &VisibleShellController, f: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(f)
        .expect("native story event")
}
fn position(c: &mut VisibleShellController) -> Point {
    let s = c.snapshot().expect("position");
    (s.overworld.tile.x, s.overworld.tile.y)
}
fn at(c: &mut VisibleShellController, map: &str, p: Point) {
    let s = c.snapshot().expect("position");
    assert_eq!(s.overworld.map_name, map);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), p);
}
fn walk(c: &mut VisibleShellController, runtime: &CrystalRuntime, p: Point) {
    indoor_walk(c, runtime, p);
    assert_eq!(position(c), p);
}
fn talk(c: &mut VisibleShellController, b: GameButton) {
    let p = position(c);
    press(c, b);
    assert_eq!(position(c), p, "actor/counter collision");
    press(c, GameButton::A);
}
fn finish(c: &mut VisibleShellController) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    for _ in 0..192 {
        let s = c.snapshot().expect("dialogue");
        assert!(s.battle.is_none() && s.ui.pending_yes_no.is_none());
        if let Some(t) = &s.ui.text {
            labels.insert(t.label.clone());
        }
        if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && s.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            return labels;
        }
        press(c, GameButton::A);
    }
    panic!("dialogue did not finish");
}
fn settle(c: &mut VisibleShellController, ready: impl Fn(&BevyRuntimeShell) -> bool) {
    for _ in 0..192 {
        if ready(&c.shell) {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("native menu did not settle");
}
fn enter(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    outside: Point,
    room: &str,
    arrival: Point,
) {
    walk(c, runtime, outside);
    for _ in 0..6 {
        press(c, GameButton::Up);
        if c.snapshot().expect("warp").overworld.map_name == room {
            break;
        }
    }
    at(c, room, arrival);
}
fn exit(c: &mut VisibleShellController, runtime: &CrystalRuntime, carpet: Point, outside: Point) {
    walk(c, runtime, (carpet.0, carpet.1 - 1));
    for _ in 0..6 {
        press(c, GameButton::Down);
        if c.snapshot().expect("return").overworld.map_name == TOWN {
            break;
        }
    }
    at(c, TOWN, outside);
    c.wait_frames(120).expect("passive return");
    at(c, TOWN, outside);
}
fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    let before = c.snapshot().expect("save view");
    let state = c.shell.shell.session().state();
    let flags = state.flags.clone();
    let persistent = (
        state.money,
        state.badges.clone(),
        state.last_spawn_map_constant.clone(),
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("nonce")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "grand-rapids-{stage}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("native save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("native Continue");
    std::fs::remove_file(path).expect("owned scratch save");
    let after = loaded.snapshot().expect("continued view");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.storage, after.storage);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    let state = loaded.shell.shell.session().state();
    assert_eq!(
        (
            state.money,
            state.badges.clone(),
            state.last_spawn_map_constant.clone()
        ),
        persistent
    );
    // Continue materializes absent false temporary flags. Compare their
    // native values, including every unrelated event and engine flag.
    for flag in flags
        .event_flags
        .keys()
        .chain(state.flags.event_flags.keys())
    {
        assert_eq!(
            flags.is_event_flag_set(flag),
            state.flags.is_event_flag_set(flag),
            "Continue event {flag}"
        );
    }
    for flag in flags
        .engine_flags
        .keys()
        .chain(state.flags.engine_flags.keys())
    {
        assert_eq!(
            flags.is_engine_flag_set(flag),
            state.flags.is_engine_flag_set(flag),
            "Continue engine flag {flag}"
        );
    }
    if let Some(dir) = std::env::var_os("CRYSTAL_GRAND_RAPIDS_SAVE_DIR") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("review save directory");
        loaded
            .save(dir.join(format!(
                "ms09-{stage}-{}-{nonce}.crystalsave",
                std::process::id()
            )))
            .expect("retained review save");
    }
    loaded
}
fn offer(c: &mut VisibleShellController) {
    talk(c, GameButton::Up);
    for _ in 0..32 {
        if c.snapshot().expect("choice").ui.pending_yes_no.is_some() {
            return;
        }
        assert!(!event(c, CLEARED));
        press(c, GameButton::A);
    }
    panic!("native handover choice missing");
}

fn answer(c: &mut VisibleShellController, yes: bool) {
    let original = c
        .snapshot()
        .expect("choice")
        .ui
        .pending_yes_no
        .expect("native prompt");
    for _ in 0..32 {
        let selected = c
            .shell
            .yes_no_cursor
            .as_ref()
            .expect("native choice cursor")
            .option_index;
        if selected != usize::from(!yes) {
            press(
                c,
                if yes {
                    GameButton::Up
                } else {
                    GameButton::Down
                },
            );
        }
        press(c, GameButton::A);
        if c.snapshot()
            .expect("choice continuation")
            .ui
            .pending_yes_no
            .as_ref()
            != Some(&original)
        {
            return;
        }
    }
    panic!("native choice did not resolve");
}

#[test]
#[ignore = "requires published external Grand Rapids story pack"]
fn grand_rapids_controller_requires_both_win_and_recovered_evidence() {
    for (win, recovered) in [(false, false), (true, false), (false, true)] {
        let (root, runtime, mut c) = fixture(win, recovered);
        enter(&mut c, &runtime, (27, 14), DEPOT, (12, 17));
        walk(&mut c, &runtime, (11, 10));
        let before = c.shell.shell.session().state().clone();
        talk(&mut c, GameButton::Up);
        let p = position(&mut c);
        press(&mut c, GameButton::Left);
        assert_eq!(position(&mut c), p, "text owns movement");
        assert!(finish(&mut c).contains("MnGrandRapidsDepotMissingText"));
        assert!(!event(&c, CLEARED));
        let state = c.shell.shell.session().state();
        assert_eq!(state.money, before.money);
        assert_eq!(state.bag, before.bag);
        assert_eq!(state.flags, before.flags);
        let mut c = continued(&mut c, &root, &runtime, "missing");
        talk(&mut c, GameButton::Up);
        assert!(finish(&mut c).contains("MnGrandRapidsDepotMissingText"));
        assert!(!event(&c, CLEARED));
    }
}

#[test]
#[ignore = "requires published external Grand Rapids story pack"]
fn grand_rapids_controller_decline_continue_retry_acknowledgment_and_repeat() {
    let (root, runtime, mut c) = fixture(true, true);
    enter(&mut c, &runtime, (27, 14), DEPOT, (12, 17));
    walk(&mut c, &runtime, (11, 10));
    let before = c.snapshot().expect("pre-handover");
    let state = c.shell.shell.session().state();
    let resources = (state.money, state.bag.clone(), state.badges.clone());
    offer(&mut c);
    answer(&mut c, false);
    assert!(finish(&mut c).contains("MnGrandRapidsDepotDeclinedText"));
    assert!(!event(&c, CLEARED));
    let mut c = continued(&mut c, &root, &runtime, "declined");
    offer(&mut c);
    answer(&mut c, true);
    let mut acknowledgments = 0;
    while !event(&c, CLEARED) && acknowledgments < 32 {
        let s = c.snapshot().expect("reading");
        assert_eq!(
            s.ui.text.as_ref().expect("visible evidence reading").label,
            "MnGrandRapidsDepotReadText"
        );
        let p = position(&mut c);
        press(&mut c, GameButton::Left);
        assert_eq!(position(&mut c), p);
        assert!(!event(&c, CLEARED), "movement cannot acknowledge evidence");
        press(&mut c, GameButton::A);
        acknowledgments += 1;
    }
    assert!(
        acknowledgments >= 5,
        "all five real dialogue pages must be acknowledged"
    );
    assert!(event(&c, CLEARED));
    finish(&mut c);
    assert!(event(&c, DEFEATED) && event(&c, RECOVERED));
    let after = c.snapshot().expect("handover resources");
    assert_eq!(before.party, after.party);
    let state = c.shell.shell.session().state();
    assert_eq!(
        (state.money, state.bag.clone(), state.badges.clone()),
        resources
    );
    let mut c = continued(&mut c, &root, &runtime, "cleared");
    talk(&mut c, GameButton::Up);
    assert!(finish(&mut c).contains("MnGrandRapidsDepotDoneText"));
    for carpet in [(12, 19), (13, 19)] {
        exit(&mut c, &runtime, carpet, (27, 14));
        enter(&mut c, &runtime, (27, 14), DEPOT, (12, 17));
        walk(&mut c, &runtime, (11, 10));
        talk(&mut c, GameButton::Up);
        assert!(finish(&mut c).contains("MnGrandRapidsDepotDoneText"));
    }
    exit(&mut c, &runtime, (12, 19), (27, 14));
    walk(&mut c, &runtime, (40, 24));
    assert!(event(&c, CLEARED));
    let _loaded = continued(&mut c, &root, &runtime, "east-road");
}

#[test]
#[ignore = "requires published external Grand Rapids story pack"]
fn grand_rapids_controller_native_heal_pc_priced_shop_and_safe_returns() {
    let (root, runtime, mut c) = fixture(false, false);
    enter(&mut c, &runtime, (9, 12), CLINIC, (10, 13));
    walk(&mut c, &runtime, (7, 5));
    talk(&mut c, GameButton::Up);
    assert!(finish(&mut c).contains("MN_GR_NURSEText"));
    let healed = c.snapshot().expect("actual healing");
    assert!(healed.party.slots.iter().all(|s| {
        s.pokemon.hp == s.pokemon.max_hp
            && s.pokemon.status.is_none()
            && s.pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
    }));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_GRAND_RAPIDS_POKECENTER_1F")
    );
    walk(&mut c, &runtime, (13, 4));
    talk(&mut c, GameButton::Up);
    settle(&mut c, |s| {
        s.pc_hub_cursor.is_some()
            && s.special_boundary.is_none()
            && s.pc_notice.is_none()
            && s.field_notice.is_none()
    });
    assert!(c.snapshot().expect("PC menu").ui.menu.is_some());
    let original = c
        .shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    press(&mut c, GameButton::A);
    settle(&mut c, |s| {
        s.bill_pc_action_cursor.is_some()
            && s.special_boundary.is_none()
            && s.pc_notice.is_none()
            && s.field_notice.is_none()
    });
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    assert!(c.shell.bill_pc_deposit_open && c.shell.storage_cursor.is_some());
    press(&mut c, GameButton::A);
    assert!(c.shell.bill_pc_pokemon_action_cursor.is_some());
    press(&mut c, GameButton::A);
    settle(&mut c, |s| {
        s.pc_transfer_sequence.is_none()
            && s.pc_notice.is_none()
            && s.bill_pc_pokemon_action_cursor.is_none()
    });
    let deposited = c.snapshot().expect("actual PC deposit");
    assert_eq!(deposited.party.slots.len(), 1);
    assert_eq!(deposited.party.slots[0].pokemon.species.id, "CYNDAQUIL");
    let boxed = deposited
        .storage
        .boxes
        .iter()
        .find(|b| b.index == deposited.storage.current_pc_box)
        .expect("native box");
    assert_eq!(boxed.slots.len(), 1);
    assert_eq!(boxed.slots[0].pokemon.species.id, "MEW");
    press(&mut c, GameButton::B);
    assert_eq!(
        c.shell
            .bill_pc_action_cursor
            .as_ref()
            .map(|v| v.option_index),
        Some(0)
    );
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    settle(&mut c, |s| {
        s.pc_transfer_sequence.is_none()
            && s.pc_notice.is_none()
            && s.bill_pc_pokemon_action_cursor.is_none()
    });
    let restored = c.snapshot().expect("actual PC withdrawal");
    assert_eq!(restored.party.slots.len(), 2);
    assert!(restored.storage.boxes.iter().all(|b| b.slots.is_empty()));
    let party = c
        .shell
        .shell
        .session()
        .state()
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .collect::<Vec<_>>();
    assert!(original.iter().all(|p| party.contains(&p)));
    for _ in 0..24 {
        if !c.shell.pc_hub_session_open {
            break;
        }
        press(&mut c, GameButton::B);
    }
    finish(&mut c);
    let mut c = continued(&mut c, &root, &runtime, "healed");
    exit(&mut c, &runtime, (10, 15), (9, 12));
    enter(&mut c, &runtime, (39, 16), MART, (6, 5));
    walk(&mut c, &runtime, (3, 3));
    let before = c.shell.shell.session().state().clone();
    talk(&mut c, GameButton::Left);
    settle(&mut c, |s| {
        s.shop_top_cursor.is_some() && s.shop_welcome_seen && s.shop_notice.is_none()
    });
    assert_eq!(
        c.snapshot()
            .expect("mart")
            .pending_shop
            .expect("native shop")
            .mart_id,
        "MART_MN_GRAND_RAPIDS"
    );
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    let item = &runtime.data().items["HYPER_POTION"];
    assert_eq!(
        c.shell.shop_quantity.as_ref().expect("quantity").item_id,
        "HYPER_POTION"
    );
    press(&mut c, GameButton::A);
    assert_eq!(c.shell.shell.session().state().money, before.money);
    press(&mut c, GameButton::A);
    let state = c.shell.shell.session().state();
    assert_eq!(state.money, before.money - u32::from(item.price));
    assert_eq!(state.bag.quantity(item), before.bag.quantity(item) + 1);
    assert_eq!(state.flags, before.flags);
    for _ in 0..96 {
        let s = c.snapshot().expect("shop logoff");
        if s.pending_shop.is_none()
            && !s.ui.text_window_open
            && s.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            break;
        }
        let b = if s.pending_shop.is_none() || c.shell.shop_notice.is_some() {
            GameButton::A
        } else if c.shell.shop_quantity.is_some() {
            GameButton::B
        } else if let Some(cursor) = &c.shell.shop_top_cursor {
            if cursor.option_index == 2 {
                GameButton::A
            } else {
                GameButton::Down
            }
        } else {
            GameButton::B
        };
        press(&mut c, b);
    }
    assert!(c.snapshot().expect("closed shop").pending_shop.is_none());
    exit(&mut c, &runtime, (7, 7), (39, 16));
    walk(&mut c, &runtime, (30, 32));
    assert!(!event(&c, CLEARED));
    let _loaded = continued(&mut c, &root, &runtime, "services");
}

fn advance(c: &mut VisibleShellController, button: GameButton, map: &str, target: Point) {
    for _ in 0..12 {
        let view = c.snapshot().expect("native crossing position");
        if view.overworld.map_name == map
            && (view.overworld.tile.x, view.overworld.tile.y) == target
        {
            return;
        }
        press(c, button);
        let labels = finish(c);
        assert!(
            !labels
                .iter()
                .any(|s| s.contains("CheckBadgesText") || s.contains("CheckEvidenceText")),
            "unexpected refusal while crossing: {labels:?}"
        );
    }
    at(c, map, target);
}
fn refusal(c: &mut VisibleShellController, button: GameButton, label: &str) {
    for _ in 0..8 {
        press(c, button);
        let labels = finish(c);
        if !labels.is_empty() {
            assert!(labels.contains(label), "wrong checkpoint text: {labels:?}");
            return;
        }
    }
    panic!("checkpoint input never produced {label}");
}

fn connected(runtime: &CrystalRuntime) {
    let town = &runtime.data().maps[TOWN];
    assert_eq!(
        town.attributes.connections.len(),
        2,
        "requires actual connected publication"
    );
    for (direction, target) in [("west", R09), ("east", R10)] {
        assert!(
            town.attributes
                .connections
                .iter()
                .any(|p| p.direction == direction && p.target_map == target)
        );
    }
}

#[test]
#[ignore = "requires the external connected Grand Rapids story pack"]
fn grand_rapids_controller_checkpoint_cannot_substitute_victory_or_pickup_for_handover() {
    for (badge, win, recovered, label) in [
        (false, true, true, "MnGrandRapidsEastCheckBadgesText"),
        (true, false, false, "MnGrandRapidsEastCheckEvidenceText"),
        (true, true, false, "MnGrandRapidsEastCheckEvidenceText"),
        (true, true, true, "MnGrandRapidsEastCheckEvidenceText"),
    ] {
        let (root, runtime, mut c) = fixture_at(win, recovered, badge, true, TOWN, (40, 24));
        connected(&runtime);
        for y in [24, 25] {
            walk(&mut c, &runtime, (43, y));
            let before = c.shell.shell.session().state().clone();
            refusal(&mut c, GameButton::Right, label);
            assert!(!event(&c, CLEARED));
            at(&mut c, TOWN, (43, y));
            let state = c.shell.shell.session().state();
            assert_eq!(state.money, before.money);
            assert_eq!(state.bag, before.bag);
            assert_eq!(state.badges, before.badges);
            assert_eq!(state.flags, before.flags);
        }
        let mut c = continued(&mut c, &root, &runtime, "checkpoint-denied");
        refusal(&mut c, GameButton::Right, label);
        at(&mut c, TOWN, (43, 25));
    }
    let (_root, runtime, mut c) = fixture_at(false, false, true, false, TOWN, (6, 24));
    connected(&runtime);
    for y in [24, 25] {
        walk(&mut c, &runtime, (4, y));
        refusal(
            &mut c,
            GameButton::Left,
            "MnGrandRapidsWestCheckEvidenceText",
        );
        at(&mut c, TOWN, (4, y));
        assert!(!event(&c, REPORT));
    }
}

#[test]
#[ignore = "requires the external connected Grand Rapids story pack"]
fn grand_rapids_controller_handover_releases_real_r10_and_both_seams_survive_continue() {
    let (root, runtime, mut c) = fixture(true, true);
    connected(&runtime);
    walk(&mut c, &runtime, (43, 24));
    refusal(
        &mut c,
        GameButton::Right,
        "MnGrandRapidsEastCheckEvidenceText",
    );
    at(&mut c, TOWN, (43, 24));
    enter(&mut c, &runtime, (27, 14), DEPOT, (12, 17));
    walk(&mut c, &runtime, (11, 10));
    offer(&mut c);
    answer(&mut c, true);
    assert!(finish(&mut c).contains("MnGrandRapidsDepotReadText"));
    assert!(event(&c, CLEARED));
    exit(&mut c, &runtime, (12, 19), (27, 14));
    walk(&mut c, &runtime, (43, 24));
    let state = c.shell.shell.session().state();
    let resources = (state.money, state.bag.clone(), state.badges.clone());
    advance(&mut c, GameButton::Right, R10, (0, 10));
    let mut c = continued(&mut c, &root, &runtime, "earned-r10");
    advance(&mut c, GameButton::Left, TOWN, (47, 24));
    advance(&mut c, GameButton::Left, TOWN, (44, 24));
    let mut c = continued(&mut c, &root, &runtime, "r10-return");
    walk(&mut c, &runtime, (4, 24));
    advance(&mut c, GameButton::Left, R09, (63, 24));
    let mut c = continued(&mut c, &root, &runtime, "r09-return");
    advance(&mut c, GameButton::Right, TOWN, (0, 24));
    advance(&mut c, GameButton::Right, TOWN, (3, 24));
    assert!(event(&c, CLEARED) && event(&c, DEFEATED) && event(&c, RECOVERED));
    let state = c.shell.shell.session().state();
    assert_eq!(
        (state.money, state.bag.clone(), state.badges.clone()),
        resources
    );
    walk(&mut c, &runtime, (6, 24));
    let _loaded = continued(&mut c, &root, &runtime, "midgame-round-trip");
}

#[test]
#[ignore = "requires the external connected Grand Rapids story pack"]
fn grand_rapids_controller_inbound_travelers_reach_care_without_outbound_permission() {
    for (map, start, button, town_edge, inside) in [
        (R09, (63, 24), GameButton::Right, (0, 24), (3, 24)),
        (R10, (0, 10), GameButton::Left, (47, 24), (44, 24)),
    ] {
        let (root, runtime, mut c) = fixture_at(false, false, false, false, map, start);
        connected(&runtime);
        advance(&mut c, button, TOWN, town_edge);
        advance(&mut c, button, TOWN, inside);
        assert!(!event(&c, CLEARED) && !event(&c, REPORT));
        enter(&mut c, &runtime, (9, 12), CLINIC, (10, 13));
        walk(&mut c, &runtime, (7, 5));
        talk(&mut c, GameButton::Up);
        assert!(finish(&mut c).contains("MN_GR_NURSEText"));
        let healed = c.snapshot().expect("actual free care");
        assert!(healed.party.slots.iter().all(|s| {
            s.pokemon.hp == s.pokemon.max_hp
                && s.pokemon.status.is_none()
                && s.pokemon
                    .moves
                    .iter()
                    .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
        }));
        assert!(!event(&c, CLEARED) && !event(&c, REPORT));
        assert_eq!(c.shell.shell.session().state().money, 10000);
        let _loaded = continued(&mut c, &root, &runtime, "inbound-care");
    }
}
