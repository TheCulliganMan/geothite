//! Bemidji production inputs after explicit local review fixtures.
//! Initial placement, legal partners/injury, prior badges, money and RNG are
//! setup. Every subsequent action uses VisibleShellController input, including
//! the losing battle, recovery, PC transfers and Party-menu Teleport.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::state::EventFlagMemory;
use crystal_core::systems::script_flags::reconcile_saved_badge_flags;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyRuntimeShell, BevyShellConfig, BevyShellStart, PartyAction, PartyFieldMove,
    StartMenuOption, VisibleShellController, complete_visible_smoke_player_name_if_needed,
    initialize_bevy_runtime_shell, mark_runtime_snapshot_dirty, visible_party_actions,
    visible_start_menu_options,
};
use super::authored_route_fixture::{Point, cleanup_reached_save, indoor_walk, reached_save_path};

const TOWN: &str = "MnBemidji";
const CLINIC: &str = "MnBemidjiPokecenter1F";
const GYM: &str = "MnBemidjiGym";
const MART: &str = "MnBemidjiMart";
const TOWN_CONSTANT: &str = "MN_BEMIDJI";
const CLINIC_CONSTANT: &str = "MN_BEMIDJI_POKECENTER_1F";

fn load() -> (AssetRoot, CrystalRuntime) {
    let path = std::env::var_os("CRYSTAL_BEMIDJI_CITY_CONTROLLER_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-cities-review.crystalpack")
        })
        .canonicalize()
        .expect("published external three-city pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("verified native Bemidji pack"),
    )
    .expect("production city runtime");
    for map in [TOWN, CLINIC, GYM, MART] {
        assert!(runtime.data().maps.contains_key(map), "missing {map}");
    }
    // These tests use the current canonical 6x10 Cut garden and trainer anchors.
    let gym = &runtime.data().maps[GYM];
    assert_eq!((gym.attributes.width, gym.attributes.height), (6, 10));
    (root, runtime)
}

#[derive(Clone, Copy)]
enum Partners {
    Healthy,
    Injured,
    Splash,
    Teleport,
    RouteReview,
}

fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    map: &str,
    at: Point,
    partners: Partners,
) -> VisibleShellController {
    let mut controller = VisibleShellController {
        shell: initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime
                    .title_new_game_spawn_identifier()
                    .expect("original HOME"),
                map_name: map.into(),
                tile_x: at.0,
                tile_y: at.1,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("explicit local initial placement"),
    };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("initial name");
    let species = match partners {
        // Survive an initial Grass hit so HP and PP changes remain observable
        // before the losing battle's automatic healing and recovery.
        Partners::Splash => vec![("MAGIKARP", 70)],
        Partners::Teleport => vec![("ABRA", 10)],
        Partners::RouteReview => vec![("MEW", 50)],
        Partners::Healthy | Partners::Injured => vec![("MEW", 45), ("CYNDAQUIL", 12)],
    };
    let state = controller.shell.shell.session_mut().state_mut();
    for (species, level) in species {
        let mut pokemon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("legal native fixture partner");
        if matches!(partners, Partners::RouteReview) {
            pokemon.moves = vec![LearnedMove {
                name: "SURF".into(),
                current_pp: runtime.data().moves["SURF"].pp,
                pp_ups: 0,
            }];
        }
        if matches!(partners, Partners::Splash) {
            // This legal initial injury persists until either the authored
            // nurse or native whiteout cures it. Burn does not poison-tick
            // during the ordinary walk to the real trainer.
            pokemon.status = Some("BURN".into());
        }
        if matches!(partners, Partners::Injured) {
            pokemon.hp = pokemon.max_hp.saturating_sub(15).max(1);
            pokemon.status = Some("POISON".into());
            for movement in &mut pokemon.moves {
                movement.current_pp = 0;
            }
        }
        assert!(state.storage.party.add_pokemon(pokemon));
    }
    state.sync_party_from_storage();
    state.money = 10_000;
    state.badges.johto = std::array::from_fn(|slot| {
        slot < if matches!(partners, Partners::RouteReview) {
            7
        } else {
            6
        }
    });
    reconcile_saved_badge_flags(state);
    if matches!(partners, Partners::RouteReview) {
        // Declared prior qualifications for the local road tour, not earned
        // story progress. No state is changed after the first gameplay input.
        for flag in ["EVENT_MN_HEADWATERS_READING", "EVENT_MN_HEADWATERS_REPORT"] {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("prior report fixture");
        }
    }
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..524288)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Setup ends here. No coordinates, flags, party, menu cursor or battle
    // state are written by any test after this boundary.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_at(&mut controller, map, at);
    controller
}

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production Bemidji input");
    c.wait_frames(20).expect("visible presentation settle");
    assert_eq!(c.shell.last_error, None);
}

fn position(c: &mut VisibleShellController) -> Point {
    let tile = c.snapshot().expect("native position").overworld.tile;
    (tile.x, tile.y)
}

fn assert_at(c: &mut VisibleShellController, map: &str, at: Point) {
    let snapshot = c.snapshot().expect("actual map and position");
    assert_eq!(
        (
            snapshot.overworld.map_name.as_str(),
            (snapshot.overworld.tile.x, snapshot.overworld.tile.y)
        ),
        (map, at)
    );
}

fn walk(c: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point) {
    let map = c.snapshot().expect("walking origin").overworld.map_name;
    indoor_walk(c, runtime, to);
    assert_at(c, &map, to);
}

fn settle(c: &mut VisibleShellController, ready: impl Fn(&BevyRuntimeShell) -> bool) {
    for _ in 0..192 {
        if ready(&c.shell) {
            return;
        }
        press(c, GameButton::A);
    }
    panic!(
        "native input boundary did not settle: {:?}",
        c.snapshot().expect("diagnosis").ui
    );
}

fn finish_text(c: &mut VisibleShellController) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    for _ in 0..192 {
        let snapshot = c.snapshot().expect("native dialogue");
        assert!(snapshot.battle.is_none(), "battle requires battle commands");
        assert!(
            snapshot.ui.pending_yes_no.is_none(),
            "choices require explicit input"
        );
        if let Some(text) = &snapshot.ui.text {
            labels.insert(text.label.clone());
        }
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && snapshot.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.battle_messages.is_empty()
            && c.shell.visible_blackout_phase.is_none()
            && !c.shell.pending_poison_blackout
            && !c.shell.pending_field_travel_arrival
            && c.shell.pending_field_travel_delay_frames.is_none()
            && c.shell.visible_field_travel_animation.is_none()
        {
            return labels;
        }
        press(c, GameButton::A);
    }
    panic!("city dialogue did not release walking input");
}

fn talk(c: &mut VisibleShellController, facing: GameButton) {
    let before = position(c);
    press(c, facing);
    assert_eq!(position(c), before, "actor or counter must block movement");
    press(c, GameButton::A);
}

fn enter(c: &mut VisibleShellController, room: &str, arrival: Point) {
    for _ in 0..6 {
        press(c, GameButton::Up);
        if c.snapshot().expect("door transition").overworld.map_name == room {
            break;
        }
    }
    assert_at(c, room, arrival);
    c.wait_frames(120).expect("passive arrival settle");
    assert_at(c, room, arrival);
}

fn exit(c: &mut VisibleShellController, runtime: &CrystalRuntime, carpet: Point, outside: Point) {
    walk(c, runtime, (carpet.0, carpet.1 - 1));
    for _ in 0..6 {
        press(c, GameButton::Down);
        if c.snapshot().expect("carpet transition").overworld.map_name == TOWN {
            break;
        }
    }
    assert_at(c, TOWN, outside);
    c.wait_frames(120).expect("safe exterior return settle");
    assert_at(c, TOWN, outside);
}

fn semantic_flags(flags: &EventFlagMemory) -> (BTreeSet<String>, BTreeSet<String>) {
    // The native flag API treats an absent bit and an explicit false bit
    // identically. Continue may canonicalize those sparse representations.
    let enabled = |bits: &std::collections::BTreeMap<String, bool>| {
        bits.iter()
            .filter_map(|(name, &set)| set.then_some(name.clone()))
            .collect()
    };
    (enabled(&flags.event_flags), enabled(&flags.engine_flags))
}

fn continue_save(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
) -> VisibleShellController {
    let before = c.snapshot().expect("before save");
    let state = c.shell.shell.session().state();
    let persistent = (
        state.money,
        state.badges.clone(),
        semantic_flags(&state.flags),
        state.last_spawn_map_constant.clone(),
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("nonce")
        .as_nanos();
    let path = reached_save_path(&format!(
        "bemidji-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("production save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("production Continue");
    cleanup_reached_save(&path);
    let after = loaded.snapshot().expect("continued city state");
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
            semantic_flags(&state.flags),
            state.last_spawn_map_constant.clone()
        ),
        persistent
    );
    loaded
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn bemidji_city_controller_six_facades_both_carpets_and_safe_passive_arrivals() {
    let (root, runtime) = load();
    for (room, outside, arrival, carpets) in [
        (GYM, (10, 10), (4, 17), [(4, 19), (5, 19)]),
        ("MnBemidjiShelfHouse", (7, 16), (6, 5), [(6, 7), (7, 7)]),
        (CLINIC, (31, 10), (10, 13), [(10, 15), (11, 15)]),
        ("MnBemidjiNorthHouse", (33, 18), (6, 5), [(6, 7), (7, 7)]),
        (MART, (42, 20), (6, 5), [(6, 7), (7, 7)]),
        (
            "MnBemidjiHeritageHouse",
            (9, 38),
            (8, 9),
            [(8, 11), (9, 11)],
        ),
    ] {
        let mut c = fixture(&root, &runtime, TOWN, outside, Partners::Healthy);
        for carpet in carpets {
            enter(&mut c, room, arrival);
            exit(&mut c, &runtime, carpet, outside);
        }
        let _loaded = continue_save(&mut c, &root, &runtime);
    }
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn bemidji_city_controller_downhill_hop_reverse_wall_and_ordinary_return() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, TOWN, (11, 19), Partners::Healthy);
    let mut saw_lip = false;
    for _ in 0..8 {
        c.press(GameButton::Down).expect("native downhill input");
        let at = position(&mut c);
        assert!(
            matches!(at, (11, 19) | (11, 20) | (11, 22)),
            "unexpected hop {at:?}"
        );
        saw_lip |= at == (11, 20);
        if at == (11, 22) {
            break;
        }
    }
    assert!(saw_lip, "actual HOP_DOWN quadrant must be traversed");
    assert_at(&mut c, TOWN, (11, 22));
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
        assert_at(&mut c, TOWN, (11, 22));
    }
    // This planner includes only ordinary FLOOR/carpet, never HOP/WALL.
    walk(&mut c, &runtime, (20, 24));
    walk(&mut c, &runtime, (11, 19));
}

fn exterior_travel(c: &mut VisibleShellController, button: GameButton, map: &str, target: Point) {
    for _ in 0..32 {
        press(c, button);
        let snapshot = c.snapshot().expect("native road transition");
        if snapshot.overworld.map_name == map
            && (snapshot.overworld.tile.x, snapshot.overworld.tile.y) == target
        {
            assert_at(c, map, target);
            return;
        }
        assert!(!snapshot.ui.text_window_open, "qualified road must be open");
        assert!(snapshot.battle.is_none(), "public lane must stay dry");
    }
    assert_at(c, map, target);
}

#[test]
#[ignore = "requires current Minnesota pack; initial seven badges/report/Surf partner are local fixtures"]
fn bemidji_city_controller_exterior_residents_signs_eight_road_lanes_and_real_surf() {
    let (root, runtime) = load();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/towns/MnBemidji.json");
    let plan: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(source).expect("current Bemidji dossier"))
            .expect("Bemidji JSON");
    let mut c = fixture(&root, &runtime, TOWN, (20, 24), Partners::RouteReview);
    for resident in plan["residents"].as_array().expect("outside residents") {
        let x = i16::try_from(resident["tile"][0].as_i64().expect("x")).unwrap();
        let y = i16::try_from(resident["tile"][1].as_i64().expect("y")).unwrap();
        let (approach, facing) = match resident["facing"].as_str().expect("facing") {
            "north" => ((x, y - 1), GameButton::Down),
            "south" => ((x, y + 1), GameButton::Up),
            "east" => ((x + 1, y), GameButton::Left),
            "west" => ((x - 1, y), GameButton::Right),
            _ => panic!("cardinal resident facing"),
        };
        walk(&mut c, &runtime, approach);
        talk(&mut c, facing);
        assert!(!finish_text(&mut c).is_empty(), "actual resident dialogue");
    }
    for sign in plan["signs"].as_array().expect("public signs") {
        let x = i16::try_from(sign["tile"][0].as_i64().expect("x")).unwrap();
        let y = i16::try_from(sign["tile"][1].as_i64().expect("y")).unwrap();
        walk(&mut c, &runtime, (x, y + 1));
        talk(&mut c, GameButton::Up);
        assert!(!finish_text(&mut c).is_empty(), "actual sign text");
    }
    let flags = semantic_flags(&c.shell.shell.session().state().flags);
    for lane in 0..2 {
        for (start, outward, neighbor, there, inward) in [
            (
                (4, 24 + lane),
                GameButton::Left,
                "MnR14C01R00",
                (31, 26 + lane),
                GameButton::Right,
            ),
            (
                (43, 24 + lane),
                GameButton::Right,
                "MnR09C00R01",
                (0, 2 + lane),
                GameButton::Left,
            ),
            (
                (24 + lane, 44),
                GameButton::Down,
                "MnR08C00R00",
                (24 + lane, 0),
                GameButton::Up,
            ),
            (
                (24 + lane, 4),
                GameButton::Up,
                "MnBemidjiNorthwoods",
                (8 + lane, 27),
                GameButton::Down,
            ),
        ] {
            walk(&mut c, &runtime, start);
            exterior_travel(&mut c, outward, neighbor, there);
            exterior_travel(&mut c, inward, TOWN, start);
        }
    }
    assert_eq!(
        semantic_flags(&c.shell.shell.session().state().flags),
        flags
    );
    walk(&mut c, &runtime, (29, 41));
    press(&mut c, GameButton::Right);
    for _ in 0..96 {
        let s = c.snapshot().expect("native low-launch Surf prompt");
        if s.overworld.mode == crystal_core::world::movement::MovementMode::Surf
            && (s.overworld.tile.x, s.overworld.tile.y) == (30, 41)
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert_at(&mut c, TOWN, (30, 41));
    assert_eq!(
        c.snapshot().unwrap().overworld.mode,
        crystal_core::world::movement::MovementMode::Surf
    );
    for (button, target) in [
        (GameButton::Down, (30, 42)),
        (GameButton::Up, (30, 41)),
        (GameButton::Left, (29, 41)),
    ] {
        for _ in 0..8 {
            press(&mut c, button);
            if position(&mut c) == target {
                break;
            }
        }
        assert_at(&mut c, TOWN, target);
    }
    assert_eq!(
        c.snapshot().unwrap().overworld.mode,
        crystal_core::world::movement::MovementMode::Normal
    );
    walk(&mut c, &runtime, (20, 24));
    let _loaded = continue_save(&mut c, &root, &runtime);
}

#[test]
#[ignore = "requires current Minnesota pack; initial six badges are local fixtures"]
fn bemidji_city_controller_north_guard_denies_outbound_and_allows_unqualified_return() {
    let (root, runtime) = load();
    for lane in 0..2 {
        let mut c = fixture(&root, &runtime, TOWN, (24 + lane, 4), Partners::Healthy);
        let before = semantic_flags(&c.shell.shell.session().state().flags);
        for _ in 0..8 {
            press(&mut c, GameButton::Up);
            if c.snapshot().unwrap().ui.text_window_open {
                break;
            }
        }
        assert!(!finish_text(&mut c).is_empty(), "native PINE BADGE refusal");
        assert_at(&mut c, TOWN, (24 + lane, 4));
        assert_eq!(
            semantic_flags(&c.shell.shell.session().state().flags),
            before
        );
        let mut inbound = fixture(
            &root,
            &runtime,
            "MnBemidjiNorthwoods",
            (8 + lane, 27),
            Partners::Healthy,
        );
        exterior_travel(&mut inbound, GameButton::Down, TOWN, (24 + lane, 4));
        assert_eq!(
            semantic_flags(&inbound.shell.shell.session().state().flags),
            before
        );
        let _loaded = continue_save(&mut inbound, &root, &runtime);
    }
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn bemidji_city_controller_care_and_actual_pc_deposit_withdraw() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, CLINIC, (7, 5), Partners::Injured);
    let initial = c.snapshot().expect("injured initial partners");
    assert!(
        initial
            .party
            .slots
            .iter()
            .all(|slot| slot.pokemon.hp < slot.pokemon.max_hp
                && slot.pokemon.status.is_some()
                && slot.pokemon.moves.iter().all(|m| m.current_pp == 0))
    );
    let state = c.shell.shell.session().state();
    let resources = (
        state.money,
        state.bag.clone(),
        state.badges.clone(),
        state.flags.clone(),
    );
    talk(&mut c, GameButton::Up);
    let waiting = position(&mut c);
    press(&mut c, GameButton::Down);
    assert_eq!(position(&mut c), waiting, "nurse text owns input");
    assert!(finish_text(&mut c).contains("MN_BEMIDJI_NURSEText"));
    let healed = c.snapshot().expect("actual native healing");
    assert!(healed.party.slots.iter().all(|slot| {
        slot.pokemon.hp == slot.pokemon.max_hp
            && slot.pokemon.status.is_none()
            && slot
                .pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
    }));
    let state = c.shell.shell.session().state();
    assert_eq!(
        (
            state.money,
            state.bag.clone(),
            state.badges.clone(),
            state.flags.clone()
        ),
        resources
    );
    assert_eq!(
        state.last_spawn_map_constant.as_deref(),
        Some(CLINIC_CONSTANT)
    );
    let original = state
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    walk(&mut c, &runtime, (13, 4));
    talk(&mut c, GameButton::Up);
    settle(&mut c, |shell| {
        shell.pc_hub_cursor.is_some()
            && shell.special_boundary.is_none()
            && shell.pc_notice.is_none()
            && shell.field_notice.is_none()
    });
    assert!(c.snapshot().expect("visible PC menu").ui.menu.is_some());
    press(&mut c, GameButton::A); // BILL's PC.
    settle(&mut c, |shell| {
        shell.bill_pc_action_cursor.is_some()
            && shell.special_boundary.is_none()
            && shell.pc_notice.is_none()
            && shell.field_notice.is_none()
    });
    press(&mut c, GameButton::Down); // DEPOSIT.
    press(&mut c, GameButton::A);
    assert!(c.shell.bill_pc_deposit_open && c.shell.storage_cursor.is_some());
    press(&mut c, GameButton::A); // first actual party Pokemon.
    assert!(c.shell.bill_pc_pokemon_action_cursor.is_some());
    press(&mut c, GameButton::A); // deposit confirmation.
    settle(&mut c, |shell| {
        shell.pc_transfer_sequence.is_none()
            && shell.pc_notice.is_none()
            && shell.bill_pc_pokemon_action_cursor.is_none()
    });
    let deposited = c.snapshot().expect("native deposit mutation");
    assert_eq!(deposited.party.slots.len(), 1);
    assert_eq!(deposited.party.slots[0].pokemon.species.id, "CYNDAQUIL");
    let stored = deposited
        .storage
        .boxes
        .iter()
        .find(|b| b.index == deposited.storage.current_pc_box)
        .expect("current native box");
    assert_eq!(stored.slots.len(), 1);
    assert_eq!(stored.slots[0].pokemon.species.id, "MEW");
    press(&mut c, GameButton::B);
    assert_eq!(
        c.shell
            .bill_pc_action_cursor
            .as_ref()
            .map(|cursor| cursor.option_index),
        Some(0)
    );
    press(&mut c, GameButton::A); // WITHDRAW.
    press(&mut c, GameButton::A); // boxed MEW.
    press(&mut c, GameButton::A); // withdrawal.
    settle(&mut c, |shell| {
        shell.pc_transfer_sequence.is_none()
            && shell.pc_notice.is_none()
            && shell.bill_pc_pokemon_action_cursor.is_none()
    });
    assert_eq!(
        c.snapshot().expect("native withdrawal").party.slots.len(),
        2
    );
    assert!(
        c.snapshot()
            .expect("box emptied")
            .storage
            .boxes
            .iter()
            .all(|b| b.slots.is_empty())
    );
    let state = c.shell.shell.session().state();
    let restored = state
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .collect::<Vec<_>>();
    assert!(original.iter().all(|mon| restored.contains(&mon)));
    for _ in 0..24 {
        if !c.shell.pc_hub_session_open {
            break;
        }
        press(&mut c, GameButton::B);
    }
    assert!(!c.shell.pc_hub_session_open);
    finish_text(&mut c);
    walk(&mut c, &runtime, (13, 5));
    let _loaded = continue_save(&mut c, &root, &runtime);
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn bemidji_city_controller_priced_shop_mutates_money_and_bag() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, MART, (3, 3), Partners::Healthy);
    let before = c.shell.shell.session().state().clone();
    talk(&mut c, GameButton::Left);
    settle(&mut c, |shell| {
        shell.shop_top_cursor.is_some() && shell.shop_welcome_seen && shell.shop_notice.is_none()
    });
    let shop = c
        .snapshot()
        .expect("native mart")
        .pending_shop
        .expect("actual mart");
    assert_eq!(shop.mart_id, "MART_MN_BEMIDJI");
    assert_eq!(
        shop.inventory,
        [
            "ULTRA_BALL",
            "HYPER_POTION",
            "FULL_HEAL",
            "REVIVE",
            "MAX_REPEL"
        ]
    );
    press(&mut c, GameButton::A); // BUY.
    press(&mut c, GameButton::Down); // HYPER POTION.
    press(&mut c, GameButton::A);
    let quantity = c.shell.shop_quantity.as_ref().expect("quantity selector");
    let item = &runtime.data().items["HYPER_POTION"];
    assert_eq!(quantity.item_id, "HYPER_POTION");
    assert_eq!(quantity.quantity, 1);
    assert_eq!(quantity.unit_price, item.price);
    press(&mut c, GameButton::A); // Quantity confirmation, no purchase yet.
    assert_eq!(c.shell.shell.session().state().money, before.money);
    assert_eq!(c.shell.shell.session().state().bag, before.bag);
    press(&mut c, GameButton::A); // YES, actual transaction.
    let state = c.shell.shell.session().state();
    assert_eq!(state.money, before.money - u32::from(item.price));
    assert_eq!(state.bag.quantity(item), before.bag.quantity(item) + 1);
    assert_eq!(state.badges, before.badges);
    assert_eq!(state.flags, before.flags);
    for _ in 0..96 {
        let snapshot = c.snapshot().expect("shop logoff");
        if snapshot.pending_shop.is_none()
            && !snapshot.ui.text_window_open
            && snapshot.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            break;
        }
        let button = if snapshot.pending_shop.is_none() || c.shell.shop_notice.is_some() {
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
        press(&mut c, button);
    }
    assert!(c.snapshot().expect("shop closed").pending_shop.is_none());
    walk(&mut c, &runtime, (3, 4));
    let _loaded = continue_save(&mut c, &root, &runtime);
}

fn lose_actual_battle(c: &mut VisibleShellController) {
    let splash_pp = c.snapshot().expect("prebattle partner").party.slots[0]
        .pokemon
        .moves
        .iter()
        .find(|m| m.name == "SPLASH")
        .expect("learned legal Splash")
        .current_pp;
    let mut started = false;
    let mut selected_attacks = 0;
    let mut saw_spent_pp = false;
    let mut saw_hp_damage = false;
    let mut lowest_combat_hp = None;
    for _ in 0..4096 {
        let snapshot = c.snapshot().expect("actual losing native battle");
        // Scripted trainer combat owns its active Pokemon while a turn is in
        // flight. The overworld party projection may still show the prebattle
        // partner until the battle commits (and whiteout then heals it).
        if let Some(combat) = c
            .shell
            .shell
            .session()
            .state()
            .script_runtime
            .active_battle_combat
            .as_ref()
        {
            saw_hp_damage |= combat.player.hp < combat.player.max_hp;
            lowest_combat_hp =
                Some(lowest_combat_hp.map_or(combat.player.hp, |hp: u16| hp.min(combat.player.hp)));
            saw_spent_pp |= combat
                .player
                .moves
                .iter()
                .any(|m| m.name == "SPLASH" && m.current_pp < splash_pp);
        }
        let Some(battle) = snapshot.battle.as_ref() else {
            if started {
                break;
            }
            press(c, GameButton::A);
            continue;
        };
        started = true;
        saw_hp_damage |= snapshot
            .party
            .slots
            .iter()
            .any(|slot| slot.pokemon.hp < slot.pokemon.max_hp);
        saw_spent_pp |= battle
            .player_moves
            .iter()
            .any(|m| m.name == "SPLASH" && m.current_pp < splash_pp);
        let button = if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
            c.wait_frames(30).expect("battle narration");
            GameButton::A
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let target = battle
                .player_moves
                .iter()
                .position(|m| m.name == "SPLASH" && m.current_pp > 0)
                .expect("legal non-damaging move");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                selected_attacks += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("battle command cursor"),
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    assert!(
        started && selected_attacks > 0 && saw_spent_pp && saw_hp_damage,
        "must execute native battle turns, consume Splash PP and damage actual HP: \
         started={started}, selected={selected_attacks}, pp_spent={saw_spent_pp}, \
         hp_damaged={saw_hp_damage}, lowest_combat_hp={lowest_combat_hp:?}"
    );
    assert!(c.snapshot().expect("loss result").battle.is_none());
    finish_text(c);
    assert_eq!(c.shell.shell.session().state().battle_result & 0x3f, 1);
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn bemidji_city_controller_real_clinic_memory_battle_loss_recovery_and_continue() {
    let (root, runtime) = load();
    for heal in [false, true] {
        let mut c = fixture(&root, &runtime, TOWN, (31, 10), Partners::Splash);
        assert_ne!(
            c.shell
                .shell
                .session()
                .state()
                .last_spawn_map_constant
                .as_deref(),
            Some(TOWN_CONSTANT)
        );
        enter(&mut c, CLINIC, (10, 13));
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .last_spawn_map_constant
                .as_deref(),
            Some(TOWN_CONSTANT),
            "actual exterior-to-clinic warp must remember its exterior source"
        );
        if heal {
            walk(&mut c, &runtime, (7, 5));
            talk(&mut c, GameButton::Up);
            finish_text(&mut c);
        }
        let (constant, expected_map, arrival) = if heal {
            (CLINIC_CONSTANT, CLINIC, (10, 13))
        } else {
            (TOWN_CONSTANT, TOWN, (31, 10))
        };
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .last_spawn_map_constant
                .as_deref(),
            Some(constant),
            "authored nurse blackoutmod alone changes recovery to the indoor floor"
        );
        exit(&mut c, &runtime, (10, 15), (31, 10));
        walk(&mut c, &runtime, (10, 10));
        enter(&mut c, GYM, (4, 17));
        walk(&mut c, &runtime, (6, 14));
        let before = c.shell.shell.session().state().clone();
        let partner = before.storage.party.pokemon[0]
            .as_ref()
            .expect("initial recovery partner");
        assert_eq!(
            partner.hp, partner.max_hp,
            "battle starts at actual full HP"
        );
        assert_eq!(
            partner.status.as_deref(),
            if heal { None } else { Some("BURN") },
            "only the nurse branch cures the explicit initial injury before battle"
        );
        talk(&mut c, GameButton::Right); // Original Moslen / Tangela 35.
        lose_actual_battle(&mut c);
        assert_at(&mut c, expected_map, arrival);
        let state = c.shell.shell.session().state();
        assert_eq!(state.money, before.money / 2, "native blackout money loss");
        assert_eq!(state.bag, before.bag);
        assert_eq!(state.badges, before.badges);
        assert_eq!(state.last_spawn_map_constant.as_deref(), Some(constant));
        assert!(
            !state
                .flags
                .is_event_flag_set("EVENT_MN_GYM_7_TRAINER_1_DEFEATED")
                .expect("trainer flag"),
            "a loss must not resume the gym's victory script"
        );
        assert!(!state.badges.johto[6], "loss never awards badge7");
        let recovered = c.snapshot().expect("native recovered party");
        assert!(recovered.party.slots.iter().all(|slot| {
            slot.pokemon.hp == slot.pokemon.max_hp
                && slot.pokemon.status.is_none()
                && slot
                    .pokemon
                    .moves
                    .iter()
                    .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
        }));
        c.wait_frames(120).expect("safe passive recovery floor");
        assert_at(&mut c, expected_map, arrival);
        let mut loaded = continue_save(&mut c, &root, &runtime);
        assert_at(&mut loaded, expected_map, arrival);
    }
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn bemidji_city_controller_party_menu_teleport_to_healed_clinic_recovery() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, TOWN, (31, 10), Partners::Teleport);
    enter(&mut c, CLINIC, (10, 13));
    walk(&mut c, &runtime, (7, 5));
    talk(&mut c, GameButton::Up);
    finish_text(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some(CLINIC_CONSTANT)
    );
    exit(&mut c, &runtime, (10, 15), (31, 10));
    walk(&mut c, &runtime, (21, 22));
    let before = c.snapshot().expect("before Party Teleport");
    let money = c.shell.shell.session().state().money;
    press(&mut c, GameButton::Start);
    let snapshot = c.snapshot().expect("production Start");
    let target = visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|option| *option == StartMenuOption::Pokemon)
        .expect("actual Pokemon option");
    for _ in 0..16 {
        let current = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if current == target {
            press(&mut c, GameButton::A);
            break;
        }
        press(
            &mut c,
            if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    assert!(c.shell.party_menu_open);
    press(&mut c, GameButton::A); // Abra's actual action menu.
    let snapshot = c.snapshot().expect("production party actions");
    let target = visible_party_actions(&snapshot, &c.shell)
        .expect("native actions")
        .iter()
        .position(|action| *action == PartyAction::FieldMove(PartyFieldMove::Teleport))
        .expect("learned Teleport option");
    for _ in 0..16 {
        let current = c
            .shell
            .party_action_cursor
            .as_ref()
            .expect("party action cursor")
            .option_index;
        if current == target {
            press(&mut c, GameButton::A);
            break;
        }
        press(
            &mut c,
            if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    finish_text(&mut c);
    assert_at(&mut c, CLINIC, (10, 13));
    let after = c.snapshot().expect("actual native Teleport arrival");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some(CLINIC_CONSTANT)
    );
    let _loaded = continue_save(&mut c, &root, &runtime);
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn bemidji_city_controller_fixed_officer_is_dialogue_without_fabricated_report() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, TOWN, (21, 20), Partners::Healthy);
    let before = c.shell.shell.session().state().clone();
    talk(&mut c, GameButton::Up);
    let labels = finish_text(&mut c);
    assert!(labels.contains("MN_BEMIDJI_OFFICERText"));
    let state = c.shell.shell.session().state();
    assert_eq!(
        state.flags, before.flags,
        "unbound Q05 officer cannot grant report progression"
    );
    assert_eq!(state.badges, before.badges);
    assert_eq!(state.bag, before.bag);
    assert_eq!(state.money, before.money);
    assert_at(&mut c, TOWN, (21, 20));
}
