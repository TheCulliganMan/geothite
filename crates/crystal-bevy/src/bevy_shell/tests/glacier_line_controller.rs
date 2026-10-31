//! Explicit local fixtures; visits, choices, rides and the courier win use input.
#[path = "glacier_line_capture.rs"]
mod capture;
#[path = "glacier_line_city_access.rs"]
mod city_access;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::super::{VisibleShellController, selected_vertical_menu};
use super::{fixture, mark_runtime_snapshot_dirty, walk};

const SERVICE: &str = "EVENT_MN_COURIER_DEFEATED";

fn load() -> (AssetRoot, CrystalRuntime, Vec<Value>) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime, Vec<Value>)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let (root, runtime, _) = super::load();
            let source_path = std::env::var_os("CRYSTAL_GLACIER_SOURCE")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../planning/minnesota/transit/glacier-service-v01.json")
                });
            let source: Value = serde_json::from_str(
                &std::fs::read_to_string(source_path).expect("five native boarding contracts"),
            )
            .expect("station source");
            assert!(
                runtime.data().maps["MnStPaulUnionDepot"]
                    .scripts
                    .contains_key("MnGlacierST_PAULRide0"),
                "published service required"
            );
            (
                root,
                runtime,
                source["stations"]
                    .as_array()
                    .expect("five stations")
                    .clone(),
            )
        })
        .clone()
}

fn point(value: &Value) -> (i16, i16) {
    (
        i16::try_from(value[0].as_u64().expect("x")).expect("native x"),
        i16::try_from(value[1].as_u64().expect("y")).expect("native y"),
    )
}

fn name(stop: &Value) -> &str {
    stop["map"].as_str().expect("station map")
}
fn visited(stop: &Value) -> &str {
    stop["visited_flag"].as_str().expect("visit event")
}
fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("native event")
}
fn location(c: &mut VisibleShellController) -> (String, (i16, i16)) {
    let s = c.snapshot().expect("authoritative location");
    (
        s.overworld.map_name,
        (s.overworld.tile.x, s.overworld.tile.y),
    )
}
fn press(c: &mut VisibleShellController, b: GameButton) {
    c.press(b).expect("production passenger input");
    c.wait_frames(16).expect("native presentation");
    assert_eq!(c.shell.last_error, None);
}

fn input_released(c: &VisibleShellController) -> bool {
    !c.has_pending_script_work()
        && c.shell.field_notice.is_none()
        && c.shell.pc_notice.is_none()
        && c.shell.special_boundary.is_none()
        && c.shell.battle_messages.is_empty()
}

fn setup(
    runtime: &CrystalRuntime,
    root: &AssetRoot,
    stops: &[Value],
    origin: usize,
    service: bool,
    visits: u8,
) -> VisibleShellController {
    let mut c = fixture(
        root,
        runtime,
        name(&stops[origin]),
        point(&stops[origin]["approach_core"]),
    );
    let state = c.shell.shell.session_mut().state_mut();
    state
        .flags
        .set_event_flag(SERVICE, service)
        .expect("declared local service fixture");
    for (i, stop) in stops.iter().enumerate() {
        state
            .flags
            .set_event_flag(visited(stop), visits & (1 << i) != 0)
            .expect("declared upstream visit fixture");
    }
    state.money = 10000;
    // MAPSETUP_TRAIN clears these native movement flags on arrival. Record
    // their initial false values so the exact flag comparison below does not
    // confuse an absent false entry with a progression change.
    for flag in [
        "ENGINE_STRENGTH_ACTIVE",
        "ENGINE_ALWAYS_ON_BIKE",
        "ENGINE_DOWNHILL",
    ] {
        state
            .flags
            .set_engine_flag(flag, false)
            .expect("declared inactive passenger movement flags");
    }
    // Initial damaged partner and spent PP make an accidental train heal visible.
    let mut partner = runtime
        .data()
        .create_pokemon("RIMEPELT", 40, Dv::default())
        .expect("legal local passenger partner");
    partner.hp = partner.hp.saturating_sub(7).max(1);
    let first = partner.moves.first_mut().expect("native learned move");
    first.current_pp = first.current_pp.saturating_sub(1);
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}

fn ticket(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    stop: &Value,
) -> (Option<Vec<String>>, String) {
    walk(c, r, point(&stop["approach_core"]));
    let at = location(c);
    press(c, GameButton::Up);
    assert_eq!(location(c), at, "conductor seals the only platform throat");
    press(c, GameButton::A);
    if c.snapshot().expect("ticket greeting").ui.text_window_open {
        press(c, GameButton::Right);
        assert_eq!(location(c), at, "dialogue owns movement");
    }
    let mut words = String::new();
    for _ in 0..128 {
        let s = c.snapshot().expect("native ticket surface");
        if let Some(menu) = s.ui.menu.filter(|m| m.window_open) {
            let vertical = selected_vertical_menu(&menu, &c.shell.menu_cursor)
                .expect("live native destination menu");
            let options = vertical.options.clone();
            capture::capture(
                c,
                &format!("ticket-{}", stop["id"].as_str().expect("stop id")),
                false,
            );
            return (Some(options), words);
        }
        if let Some(text) = s.ui.text {
            words.push_str(&text.asm_text.unwrap_or_default());
        }
        if !s.ui.text_window_open && input_released(c) {
            c.wait_frames(16).expect("release closing dialogue A");
            return (None, words);
        }
        press(c, GameButton::A);
    }
    panic!("ticket did not yield menu or input: {words}");
}

fn choose(c: &mut VisibleShellController, row: usize) {
    for _ in 0..row {
        press(c, GameButton::Down);
    }
    assert_eq!(
        c.shell
            .menu_cursor
            .as_ref()
            .expect("native choice")
            .option_index,
        row
    );
    // Isolate this choice before press can settle the whole native train clock.
    // This clears diagnostic history only, never authoritative game state.
    c.shell.last_audio_events.clear();
    press(c, GameButton::A);
}

fn drain(c: &mut VisibleShellController) {
    for _ in 0..128 {
        let s = c.snapshot().expect("closing ticket dialogue");
        assert!(
            s.ui.menu.is_none(),
            "refusal/cancel cannot open another menu"
        );
        if !s.ui.text_window_open && input_released(c) {
            c.wait_frames(16).expect("release closing ticket A");
            return;
        }
        press(c, GameButton::A);
    }
    panic!("ticket refusal retained input");
}

fn ride(c: &mut VisibleShellController, origin: &Value, destination: &Value) {
    // press/wait_frames settle automatic presentation through its native clock.
    // The terminal arrival cue proves that clock reached its real final phase;
    // it cannot be observed as an intermediate shell value after press returns.
    for _ in 0..1024 {
        let s = c.snapshot().expect("native train journey");
        if s.overworld.map_name == name(destination)
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.visible_magnet_train.is_none()
        {
            let events = &c.shell.last_audio_events;
            let train = events
                .iter()
                .position(|e| {
                    e.starts_with("drained script event Graphics:")
                        && e.contains("kind: MagnetTrain")
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{} -> {} must execute the native graphics special: {events:?}",
                        name(origin),
                        name(destination)
                    )
                });
            let at = point(&origin["conductor_core"]);
            let reset = format!(
                "script movement {} MnGlacier{}ConductorReset ",
                origin["conductor_script"].as_str().expect("conductor"),
                origin["id"].as_str().expect("source station")
            );
            let home = format!("->({},{}) steps=", at.0, at.1);
            assert!(
                events[..train].iter().any(|e| e.starts_with(&reset)
                    && e.contains(&home)
                    && (e.contains("steps=4 facing=Down") || e.contains("steps=0 "))),
                "source conductor must return home, including native offscreen map-memory reset: {events:?}"
            );
            assert!(
                events.iter().any(|e| e.contains("SFX_TRAIN_ARRIVED")),
                "the actual animation must reach its terminal arrival cue: {events:?}"
            );
            assert!(
                events
                    .iter()
                    .any(|e| e.contains("drained runtime delays") && e.contains("frames: 120")),
                "arrival must execute the retained native wait20: {events:?}"
            );
            assert_eq!(
                (s.overworld.tile.x, s.overworld.tile.y),
                point(&destination["ride_arrival_core"])
            );
            assert_eq!(s.overworld.facing, Direction::Down);
            assert!(s.ui.menu.is_none());
            return;
        }
        if s.ui.text_window_open {
            press(c, GameButton::A);
        } else {
            c.wait_frames(16)
                .expect("boarding, train clock and retained continuation");
        }
    }
    panic!("native ride did not finish at {}", name(destination));
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, label: &str) {
    press(c, GameButton::Start);
    assert!(
        c.shell.start_menu_cursor.is_some(),
        "arrival releases input to the production Start menu"
    );
    press(c, GameButton::B);
    let before = c.snapshot().expect("arrived before Save");
    let flags = c.shell.shell.session().state().flags.clone();
    let money = c.shell.shell.session().state().money;
    let dir = std::env::var_os("CRYSTAL_GLACIER_SAVE_DIR").map(PathBuf::from);
    let directory = dir.clone().unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&directory).expect("owned native save evidence directory");
    let path = directory.join(format!(
        "glacier-{label}-{}-{}.crystalsave",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    assert!(!path.exists());
    c.save(&path).expect("native Save after actual arrival");
    let mut loaded = VisibleShellController::load_save(root.clone(), r.clone(), path.clone(), None)
        .expect("native Continue on the same pack");
    loaded.set_runtime_journal_enabled(false);
    let after = loaded.snapshot().expect("Continued passenger");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(flags, loaded.shell.shell.session().state().flags);
    assert_eq!(money, loaded.shell.shell.session().state().money);
    *c = loaded;
    if dir.is_none() {
        std::fs::remove_file(path).expect("remove owned temporary save");
    }
}

#[test]
#[ignore = "requires the published Glacier service in CRYSTAL_INTERIOR_PACK"]
fn glacier_line_controller_five_real_foot_entries_set_only_their_visit_and_continue() {
    let (root, r, stops) = load();
    for stop in &stops {
        let parent = stop["parent_map"].as_str().expect("public city");
        let entry = r.data().maps[parent]
            .events
            .warps
            .iter()
            .find(|w| Some(u64::from(w.index)) == stop["parent_entry_warp"].as_u64())
            .expect("indexed native facade");
        let at = (
            i16::try_from(entry.x).expect("x"),
            i16::try_from(entry.y + 1).expect("safe porch"),
        );
        let mut c = fixture(&root, &r, parent, at);
        assert!(stops.iter().all(|s| !event(&c, visited(s))));
        walk(&mut c, &r, (at.0, at.1 - 1));
        assert_eq!(
            location(&mut c),
            (name(stop).into(), point(&stop["public_arrival_core"]))
        );
        for s in &stops {
            assert_eq!(event(&c, visited(s)), name(s) == name(stop));
        }
        assert!(!event(&c, SERVICE));
        continued(&mut c, &root, &r, stop["id"].as_str().expect("stop id"));
        for s in &stops {
            assert_eq!(event(&c, visited(s)), name(s) == name(stop));
        }
        let exit = &r.data().maps[name(stop)].events.warps[0];
        walk(
            &mut c,
            &r,
            (
                i16::try_from(exit.x).expect("exit x"),
                i16::try_from(exit.y).expect("exit y"),
            ),
        );
        assert_eq!(
            location(&mut c),
            (parent.into(), at),
            "same public exterior return"
        );
    }
}

#[test]
#[ignore = "requires the published Glacier service in CRYSTAL_INTERIOR_PACK"]
fn glacier_line_controller_denials_twenty_locked_rows_and_both_cancel_paths_release_input() {
    let (root, r, stops) = load();
    for (origin, stop) in stops.iter().enumerate() {
        for (service, mask) in [(false, 31), (true, 1 << origin)] {
            let mut c = setup(&r, &root, &stops, origin, service, mask);
            let state = c.shell.shell.session().state().clone();
            assert!(ticket(&mut c, &r, stop).0.is_none());
            assert_eq!(c.shell.shell.session().state().money, state.money);
            assert_eq!(c.shell.shell.session().state().flags, state.flags);
            walk(&mut c, &r, point(&stop["ride_arrival_core"]));
            assert_eq!(location(&mut c).1, point(&stop["ride_arrival_core"]));
        }
        let destinations: Vec<_> = (0..5).filter(|i| *i != origin).collect();
        for (row, destination) in destinations.iter().enumerate() {
            let unlocked = destinations
                .iter()
                .find(|d| *d != destination)
                .expect("another stop");
            let mask = (1 << origin) | (1 << unlocked);
            let mut c = setup(&r, &root, &stops, origin, true, mask);
            let state = c.shell.shell.session().state().clone();
            let (options, _) = ticket(&mut c, &r, stop);
            assert!(options.expect("native marked choices")[row].ends_with(" X"));
            choose(&mut c, row);
            drain(&mut c);
            assert_eq!(
                location(&mut c),
                (name(stop).into(), point(&stop["approach_core"]))
            );
            assert_eq!(c.shell.shell.session().state().flags, state.flags);
            assert_eq!(c.shell.shell.session().state().money, state.money);
            assert!(c.shell.visible_magnet_train.is_none());
        }
        for explicit_cancel in [false, true] {
            let mut c = setup(&r, &root, &stops, origin, true, 31);
            let state = c.shell.shell.session().state().clone();
            assert!(ticket(&mut c, &r, stop).0.is_some());
            if explicit_cancel {
                choose(&mut c, 4);
            } else {
                press(&mut c, GameButton::B);
            }
            drain(&mut c);
            assert_eq!(c.shell.shell.session().state().flags, state.flags);
            assert_eq!(c.shell.shell.session().state().money, state.money);
            press(&mut c, GameButton::Start);
            assert!(c.shell.start_menu_cursor.is_some());
        }
    }
}

#[test]
#[ignore = "requires the published Glacier service in CRYSTAL_INTERIOR_PACK"]
fn glacier_line_controller_every_visit_mask_has_correct_native_labels_and_cancels() {
    let (root, r, stops) = load();
    for (origin, stop) in stops.iter().enumerate() {
        let destinations: Vec<_> = stops
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != origin)
            .collect();
        for mask in 0_u8..16 {
            let visits = destinations
                .iter()
                .enumerate()
                .fold(1 << origin, |bits, (row, (i, _))| {
                    bits | if mask & (1 << row) == 0 { 0 } else { 1 << i }
                });
            let mut c = setup(&r, &root, &stops, origin, true, visits);
            let flags = c.shell.shell.session().state().flags.clone();
            let (options, _) = ticket(&mut c, &r, stop);
            if mask == 0 {
                assert!(
                    options.is_none(),
                    "no usable destinations must explain the foot-visit rule"
                );
            } else {
                let expected: Vec<_> = destinations
                    .iter()
                    .enumerate()
                    .map(|(row, (_, dest))| {
                        format!(
                            "{}{}",
                            dest["menu_label"].as_str().expect("label"),
                            if mask & (1 << row) == 0 { " X" } else { "" }
                        )
                    })
                    .chain(std::iter::once("CANCEL".into()))
                    .collect();
                assert_eq!(
                    options.expect("actual native menu for this mask"),
                    expected,
                    "{origin}/{mask}: decision tree must preserve all four destination identities"
                );
                press(&mut c, GameButton::B);
                drain(&mut c);
            }
            assert_eq!(c.shell.shell.session().state().flags, flags);
            walk(&mut c, &r, point(&stop["ride_arrival_core"]));
            assert_eq!(location(&mut c).1, point(&stop["ride_arrival_core"]));
        }
    }
}

#[test]
#[ignore = "requires the published Glacier service in CRYSTAL_INTERIOR_PACK"]
fn glacier_line_controller_all_twenty_real_rides_restore_conductors_resources_and_continue() {
    let (root, r, stops) = load();
    for (origin, _) in stops.iter().enumerate() {
        let destinations: Vec<_> = (0..5).filter(|i| *i != origin).collect();
        for (row, destination) in destinations.iter().copied().enumerate() {
            let mut c = setup(&r, &root, &stops, origin, true, 31);
            let before = c.snapshot().expect("local travel fixture");
            let flags = c.shell.shell.session().state().flags.clone();
            let (options, _) = ticket(&mut c, &r, &stops[origin]);
            assert_eq!(
                options.expect("all visited destinations")[row],
                stops[destination]["menu_label"]
                    .as_str()
                    .expect("destination")
            );
            choose(&mut c, row);
            ride(&mut c, &stops[origin], &stops[destination]);
            let after = c.snapshot().expect("actual arrival");
            assert_eq!(before.party, after.party);
            assert_eq!(before.bag, after.bag);
            assert_eq!(c.shell.shell.session().state().money, 10000, "free fare");
            assert_eq!(
                c.shell.shell.session().state().flags,
                flags,
                "no story/badge/visit bypass"
            );
            continued(&mut c, &root, &r, &format!("{origin}-to-{destination}"));
            walk(&mut c, &r, point(&stops[destination]["approach_core"]));
            assert_eq!(
                location(&mut c).1,
                point(&stops[destination]["approach_core"])
            );
            press(&mut c, GameButton::Up);
            assert_eq!(
                location(&mut c).1,
                point(&stops[destination]["approach_core"]),
                "destination conductor still blocks"
            );
            if (origin == 3 && destination == 2) || (origin == 0 && destination == 1) {
                assert!(ticket(&mut c, &r, &stops[destination]).0.is_some());
                let return_row = (0..5)
                    .filter(|i| *i != destination)
                    .position(|i| i == origin)
                    .expect("original station remains a native destination");
                choose(&mut c, return_row);
                ride(&mut c, &stops[destination], &stops[origin]);
                continued(&mut c, &root, &r, &format!("repeat-return-{origin}"));
                walk(&mut c, &r, point(&stops[origin]["approach_core"]));
                let id = stops[origin]["conductor_script"]
                    .as_str()
                    .expect("conductor");
                let at = point(&stops[origin]["conductor_core"]);
                assert_eq!(
                    c.shell
                        .shell
                        .session()
                        .overworld()
                        .object_runtime_tile_by_id(id)
                        .expect("authoritative conductor position"),
                    TilePosition::new(at.0, at.1)
                );
                assert_eq!(
                    c.shell.shell.session().overworld().object_facings.get(id),
                    Some(&Direction::Down),
                    "the reloaded source conductor faces the public approach"
                );
                press(&mut c, GameButton::Up);
                assert_eq!(
                    location(&mut c).1,
                    point(&stops[origin]["approach_core"]),
                    "returning source conductor must still seal its original throat"
                );
            }
        }
    }
}

#[test]
#[ignore = "local legal Minnesota party/upstream prerequisites, then actual Q03 victory and train"]
fn glacier_line_controller_actual_courier_win_unlocks_a_guarded_ride_without_granting_manifest() {
    let (root, r, stops) = load();
    let stop = &stops[3];
    // This case isolates the courier/service transaction. Public foot entries
    // have their own native course; an approach through the guarded hall can
    // enter courier sight before a movement-only helper reaches the conductor.
    let mut c = fixture(
        &root,
        &r,
        stop["map"].as_str().expect("St Paul depot"),
        point(&stop["approach_core"]),
    );
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto = [true, true, true, false, false, false, false, false];
    for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state
            .flags
            .set_engine_flag(flag, state.badges.johto[i])
            .expect("declared upstream badges");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_DISPATCH_NOTE", true)
        .expect("upstream dispatch fixture");
    state
        .flags
        .set_event_flag(visited(&stops[2]), true)
        .expect("declared earlier St Cloud visit");
    state
        .flags
        .set_event_flag(visited(stop), true)
        .expect("declared earlier St Paul foot visit");
    for species in ["RIMEPELT", "PORTABEAU", "SLIPPERINA"] {
        assert!(
            state.storage.party.add_pokemon(
                r.data()
                    .create_pokemon(species, 40, Dv::default())
                    .expect("legal Minnesota partner with native learned moves")
            )
        );
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(event(&c, visited(stop)) && !event(&c, SERVICE));
    assert!(ticket(&mut c, &r, stop).0.is_none());
    walk(&mut c, &r, (19, 16));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    assert!(super::super::rochester_earned::try_battle(
        &mut c,
        &r,
        "MN_LODE_COURIER",
        SERVICE
    ));
    assert!(event(&c, SERVICE));
    assert!(!event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ"));
    assert!(!event(&c, "EVENT_MN_DEPOT_CLEARED"));
    let before_ride = c.snapshot().expect("actual courier battle resources");
    assert!(ticket(&mut c, &r, stop).0.is_some());
    choose(&mut c, 2);
    ride(&mut c, stop, &stops[2]);
    let after_ride = c
        .snapshot()
        .expect("unlocked train preserves battle resources");
    assert_eq!(before_ride.party, after_ride.party);
    assert_eq!(before_ride.bag, after_ride.bag);
    continued(&mut c, &root, &r, "actual-courier-unlocks-st-cloud");
    assert!(event(&c, SERVICE));
    assert!(!event(&c, "EVENT_MN_DEEP_BORE_MANIFEST_READ"));
    assert!(!event(&c, "EVENT_MN_DEPOT_CLEARED"));
}
