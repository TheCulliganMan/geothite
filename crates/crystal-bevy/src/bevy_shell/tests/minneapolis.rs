//! Explicit local setup, then production input for ICE battle/rescue/evidence.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::indoor_walk;

const CITY: &str = "MnMinneapolis";
const OFFICE: &str = "MnMinneapolisDispatchOffice";
const HOME: &str = "MnMinneapolisDispatcherHouse";
const CLINIC: &str = "MnMinneapolisPokecenter1F";
const DEFEATED: &str = "EVENT_MN_ICE_ENFORCER_DEFEATED";
const RESCUED: &str = "EVENT_MN_ICE_CAPTIVE_RESCUED";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

fn load() -> (AssetRoot, CrystalRuntime) {
    static INPUT: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    INPUT
        .get_or_init(|| {
            let p = PathBuf::from(
                std::env::var_os("CRYSTAL_MINNEAPOLIS_TEST_PACK")
                    .expect("published Minneapolis pack"),
            );
            let root = AssetRoot::new(p.parent().expect("pack directory"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(p).expect("written verified pack"),
            )
            .expect("runtime");
            (root, runtime)
        })
        .clone()
}
pub(super) fn fixture(
    map: &str,
    tile: (i16, i16),
    badge2: bool,
    losing: bool,
    injured: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: map.into(),
            tile_x: tile.0,
            tile_y: tile.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    {
        let mut pokemon = runtime
            .data()
            .create_pokemon(
                if losing { "MAGIKARP" } else { "MEW" },
                if losing { 5 } else { 45 },
                Dv::default(),
            )
            .expect("legal initial partner");
        if !losing {
            pokemon.moves = vec![LearnedMove {
                name: "PSYCHIC_M".into(),
                current_pp: runtime.data().moves["PSYCHIC_M"].pp,
                pp_ups: 0,
            }];
        }
        if injured {
            pokemon.hp = pokemon.max_hp.saturating_sub(15).max(1);
            pokemon.status = Some("PARALYSIS".into());
            for m in &mut pokemon.moves {
                m.current_pp = 0;
            }
        }
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
        state.money = 10000;
        state.badges.johto = [true, badge2, false, false, false, false, false, false];
        for (index, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
            state
                .flags
                .set_engine_flag(flag, state.badges.johto[index])
                .expect("consistent fixture badges");
        }
        for flag in [DEFEATED, RESCUED, NOTE] {
            state
                .flags
                .set_event_flag(flag, false)
                .expect("explicit unfinished chapter");
        }
    }
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..16384)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("initial map presentation");
    // No state/position/party/flag/RNG writes after this setup boundary.
    (root, runtime, c)
}
fn press(c: &mut VisibleShellController, b: GameButton) {
    c.press(b).expect("production input");
    c.wait_frames(24).expect("settle");
    assert_eq!(c.shell.last_error, None);
}
fn at(c: &mut VisibleShellController, map: &str, tile: (i16, i16)) {
    let s = c.snapshot().expect("position");
    assert_eq!(s.overworld.map_name, map);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), tile);
}
fn event(c: &VisibleShellController, e: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(e)
        .expect("native event")
}
fn progress(c: &VisibleShellController) -> [bool; 3] {
    [event(c, DEFEATED), event(c, RESCUED), event(c, NOTE)]
}
fn visible(c: &mut VisibleShellController, id: &str) -> bool {
    c.snapshot()
        .expect("visible actors")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(id))
}
fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    let map = c.snapshot().expect("map").overworld.map_name;
    indoor_walk(c, r, to);
    at(c, &map, to);
}
fn talk(c: &mut VisibleShellController, b: GameButton) {
    press(c, b);
    press(c, GameButton::A);
    c.wait_frames(128).expect("full visible page");
}
fn finish(c: &mut VisibleShellController) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    for _ in 0..192 {
        let s = c.snapshot().expect("dialogue");
        assert!(
            s.battle.is_none() && s.ui.pending_yes_no.is_none(),
            "ordinary dialogue owns no battle/choice"
        );
        if let Some(t) = s.ui.text {
            labels.insert(t.label);
        }
        if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.battle_messages.is_empty()
        {
            return labels;
        }
        press(c, GameButton::A);
    }
    panic!("dialogue did not release input");
}
fn advance(c: &mut VisibleShellController, b: GameButton, map: &str, tile: (i16, i16)) {
    for _ in 0..12 {
        let s = c.snapshot().expect("travel");
        if s.overworld.map_name == map && (s.overworld.tile.x, s.overworld.tile.y) == tile {
            return;
        }
        assert!(!s.ui.text_window_open);
        press(c, b);
    }
    panic!("did not reach {map} {tile:?}");
}
fn leave(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    inside: (i16, i16),
    outside: (i16, i16),
) {
    walk(c, r, inside);
    advance(c, GameButton::Down, CITY, outside);
}
fn enter(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    outside: (i16, i16),
    room: &str,
    inside: (i16, i16),
) {
    walk(c, r, outside);
    advance(c, GameButton::Up, room, inside);
}
fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    let dir = std::env::var_os("CRYSTAL_MINNEAPOLIS_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).expect("save directory");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = dir.join(format!(
        "ms03-{stage}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    let before = c.snapshot().expect("save resources");
    let flags = progress(c);
    c.save(&path).expect("real production save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), r.clone(), path, None).expect("Continue");
    let after = loaded.snapshot().expect("continued resources");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(flags, progress(&loaded));
    loaded
}
fn prompt(c: &mut VisibleShellController) {
    for _ in 0..64 {
        if let Some(p) = c.snapshot().expect("choice").ui.pending_yes_no {
            assert_eq!(p.source_script, "MnMPCellLatch");
            return;
        }
        assert!(!event(c, RESCUED));
        press(c, GameButton::A);
    }
    panic!("no actual cell choice");
}
fn answer(c: &mut VisibleShellController, yes: bool) {
    let original = c
        .snapshot()
        .expect("choice")
        .ui
        .pending_yes_no
        .expect("native yes/no");
    for _ in 0..64 {
        let source = c.snapshot().expect("choice presentation");
        let (_, selected) = c.text_view_selection_indices(&source);
        assert_eq!(
            selected,
            c.shell
                .yes_no_cursor
                .as_ref()
                .expect("production choice cursor")
                .option_index,
            "text view must mirror the authoritative choice after a retry"
        );
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
            .expect("choice result")
            .ui
            .pending_yes_no
            .as_ref()
            != Some(&original)
        {
            return;
        }
    }
    panic!("choice did not resolve");
}
fn battle(c: &mut VisibleShellController, losing: bool) {
    let before = c.snapshot().expect("battle resources");
    let mut saw = false;
    let mut attacks = 0;
    let mut opponents = BTreeSet::new();
    for _ in 0..1024 {
        let s = c.snapshot().expect("actual fight");
        let Some(b) = s.battle else {
            if saw {
                break;
            }
            press(c, GameButton::A);
            continue;
        };
        saw = true;
        assert!(
            matches!(b.kind,RuntimeBattleKind::Trainer{ref trainer_id,..}if trainer_id=="MN_ICE_ENFORCER")
        );
        opponents.insert(b.enemy_pokemon.species.id.clone());
        assert!(!event(c, DEFEATED));
        assert!(!event(c, RESCUED) && !event(c, NOTE));
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            c.wait_frames(30).expect("narration");
            GameButton::A
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let name = if losing { "SPLASH" } else { "PSYCHIC_M" };
            let target = b
                .player_moves
                .iter()
                .position(|m| m.name == name && m.current_pp > 0)
                .expect("selected legal attack with PP");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                attacks += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("battle cursor"),
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    assert!(saw && attacks > 0, "actual selected battle turns required");
    let after = c.snapshot().expect("battle terminal");
    assert!(after.battle.is_none());
    if !losing {
        assert_eq!(
            opponents,
            BTreeSet::from(["RATICATE".into(), "GROWLITHE".into()])
        );
        assert!(after.party.slots[0].pokemon.experience > before.party.slots[0].pokemon.experience);
        assert!(
            after.party.slots[0].pokemon.moves[0].current_pp
                < before.party.slots[0].pokemon.moves[0].current_pp
        );
    }
}

#[test]
#[ignore = "requires published Minneapolis pack"]
fn minneapolis_controller_denies_missing_badge_battle_and_release_without_rewards() {
    let (_, r, mut c) = fixture(OFFICE, (9, 9), false, false, false);
    let before = c.snapshot().expect("initial resources");
    talk(&mut c, GameButton::Left);
    assert!(finish(&mut c).contains("MnMPICEBadgeDeniedText"));
    walk(&mut c, &r, (11, 6));
    talk(&mut c, GameButton::Right);
    assert!(finish(&mut c).contains("MnMPCellLockedText"));
    press(&mut c, GameButton::Right);
    at(&mut c, OFFICE, (11, 6));
    walk(&mut c, &r, (10, 11));
    talk(&mut c, GameButton::Left);
    assert!(finish(&mut c).contains("MnMPDispatchLockedText"));
    assert_eq!(progress(&c), [false; 3]);
    let after = c.snapshot().expect("denied resources");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(c.shell.shell.session().state().money, 10000);
}

#[test]
#[ignore = "requires published Minneapolis pack"]
fn minneapolis_controller_actual_win_rescue_retry_open_cell_note_and_return_home() {
    let (root, r, c) = fixture(HOME, (5, 6), true, false, false);
    let _ = play_rescue(&root, &r, c);
}

/// Continues the actual office chapter from the dispatcher's home entrance.
/// Party and prior badges belong to the caller's explicit local fixture.
pub(super) fn play_rescue(
    root: &AssetRoot,
    r: &CrystalRuntime,
    mut c: VisibleShellController,
) -> VisibleShellController {
    assert!(!visible(&mut c, "MN_MP_RESCUED_DISPATCHER"));
    walk(&mut c, r, (6, 4));
    talk(&mut c, GameButton::Up);
    assert!(finish(&mut c).contains("MN_MP_DISPATCHER_PARTNERText"));
    leave(&mut c, r, (5, 6), (27, 14));
    enter(&mut c, r, (27, 34), OFFICE, (10, 14));
    walk(&mut c, r, (11, 6));
    assert!(visible(&mut c, "MN_ICE_CAPTIVE_DISPATCHER"));
    walk(&mut c, r, (9, 9));
    let money = c.shell.shell.session().state().money;
    talk(&mut c, GameButton::Left);
    battle(&mut c, false);
    assert!(finish(&mut c).contains("MnMPICEAfterText"));
    assert!(c.shell.shell.session().state().money > money);
    release_and_read(root, r, c)
}

pub(super) fn release_and_read(
    root: &AssetRoot,
    r: &CrystalRuntime,
    mut c: VisibleShellController,
) -> VisibleShellController {
    assert_eq!(progress(&c), [true, false, false]);
    let won = c.snapshot().expect("earned win");
    let money = c.shell.shell.session().state().money;
    c = continued(&mut c, root, r, "enforcer-win");
    walk(&mut c, r, (9, 9));
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().money, money);
    walk(&mut c, r, (10, 11));
    talk(&mut c, GameButton::Left);
    assert!(finish(&mut c).contains("MnMPDispatchLockedText"));
    assert!(!event(&c, NOTE));
    walk(&mut c, r, (11, 6));
    talk(&mut c, GameButton::Right);
    prompt(&mut c);
    answer(&mut c, false);
    assert!(finish(&mut c).contains("MnMPCellDeclinedText"));
    assert_eq!(progress(&c), [true, false, false]);
    c = continued(&mut c, root, r, "release-declined");
    talk(&mut c, GameButton::Right);
    prompt(&mut c);
    answer(&mut c, true);
    assert!(
        !event(&c, RESCUED),
        "accepted choice must still acknowledge release text"
    );
    super::northstar_dialogue::acknowledge(&mut c, "MS03_RESCUE", "MnMPCellReleaseText", RESCUED);
    finish(&mut c);
    assert_eq!(progress(&c), [true, true, false]);
    assert!(!visible(&mut c, "MN_ICE_CAPTIVE_DISPATCHER"));
    for x in 12..=14 {
        press(&mut c, GameButton::Right);
        at(&mut c, OFFICE, (x, 6));
    }
    advance(&mut c, GameButton::Up, OFFICE, (14, 5));
    advance(&mut c, GameButton::Down, OFFICE, (14, 6));
    advance(&mut c, GameButton::Left, OFFICE, (11, 6));
    c = continued(&mut c, root, r, "cell-open");
    assert!(!visible(&mut c, "MN_ICE_CAPTIVE_DISPATCHER"));
    walk(&mut c, r, (10, 11));
    talk(&mut c, GameButton::Left);
    super::northstar_dialogue::acknowledge(&mut c, "MS03", "MnMPDispatchReadText", NOTE);
    finish(&mut c);
    assert_eq!(progress(&c), [true; 3]);
    let done = c.snapshot().expect("released record resources");
    assert_eq!(won.party, done.party);
    assert_eq!(won.bag, done.bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    c = continued(&mut c, root, r, "dispatch-read");
    talk(&mut c, GameButton::Left);
    assert!(finish(&mut c).contains("MnMPDispatchDoneText"));
    leave(&mut c, r, (10, 14), (27, 34));
    enter(&mut c, r, (27, 14), HOME, (5, 6));
    assert!(visible(&mut c, "MN_MP_RESCUED_DISPATCHER"));
    walk(&mut c, r, (6, 4));
    talk(&mut c, GameButton::Up);
    assert!(finish(&mut c).contains("MN_MP_DISPATCHER_PARTNERAfterText"));
    c = continued(&mut c, root, r, "dispatcher-home");
    assert!(visible(&mut c, "MN_MP_RESCUED_DISPATCHER"));
    assert_eq!(progress(&c), [true; 3]);
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, false, false, false, false, false, false]
    );
    for flag in [
        "EVENT_MN_DEPOT_CLEARED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        "EVENT_MN_SURVEY_CHIEF_DEFEATED",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(!event(&c, flag), "ICE cannot grant later story {flag}");
    }
    c
}

#[test]
#[ignore = "requires published Minneapolis pack"]
fn minneapolis_controller_played_loss_recovers_in_real_clinic_without_freeing_anyone() {
    let (root, r, mut c) = fixture(CLINIC, (7, 5), true, true, false);
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    leave(&mut c, &r, (10, 14), (45, 12));
    enter(&mut c, &r, (27, 34), OFFICE, (10, 14));
    walk(&mut c, &r, (9, 9));
    talk(&mut c, GameButton::Left);
    battle(&mut c, true);
    finish(&mut c);
    assert_eq!(progress(&c), [false; 3]);
    let s = c.snapshot().expect("actual blackout");
    assert_eq!(s.overworld.map_name, CLINIC);
    assert!(s.party.slots[0].pokemon.hp > 0);
    c = continued(&mut c, &root, &r, "played-loss");
    assert_eq!(progress(&c), [false; 3]);
}

#[test]
#[ignore = "requires published Minneapolis pack"]
fn minneapolis_controller_native_healing_pc_priced_shop_and_both_gym_returns() {
    let (root, r, mut c) = fixture(CLINIC, (7, 5), false, false, true);
    let before = c.snapshot().expect("injured partner");
    assert!(before.party.slots[0].pokemon.hp < before.party.slots[0].pokemon.max_hp);
    assert!(before.party.slots[0].pokemon.status.is_some());
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    let healed = c.snapshot().expect("real free care");
    assert!(healed.party.slots.iter().all(|s| {
        s.pokemon.hp == s.pokemon.max_hp
            && s.pokemon.status.is_none()
            && s.pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    }));
    assert_eq!(c.shell.shell.session().state().money, 10000);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_MINNEAPOLIS_POKECENTER_1F")
    );
    walk(&mut c, &r, (13, 4));
    talk(&mut c, GameButton::Up);
    for _ in 0..32 {
        if c.shell.pc_hub_cursor.is_some()
            && c.shell.special_boundary.is_none()
            && c.shell.field_notice.is_none()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(c.shell.pc_hub_cursor.is_some());
    assert!(c.snapshot().expect("native PC").ui.menu.is_some());
    for _ in 0..32 {
        if c.shell.pc_hub_cursor.is_none() && c.snapshot().expect("PC exit").ui.menu.is_none() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    finish(&mut c);
    leave(&mut c, &r, (10, 14), (45, 12));
    for x in [4, 5] {
        enter(&mut c, &r, (42, 24), "MnMinneapolisGym", (4, 15));
        walk(&mut c, &r, (x, 16));
        advance(&mut c, GameButton::Down, CITY, (42, 24));
    }
    enter(&mut c, &r, (55, 16), "MnMinneapolisMart", (6, 5));
    walk(&mut c, &r, (3, 3));
    talk(&mut c, GameButton::Left);
    for _ in 0..32 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert_eq!(
        c.snapshot()
            .expect("native shop")
            .pending_shop
            .expect("mart")
            .mart_id,
        "MART_MN_MINNEAPOLIS"
    );
    let before = c.shell.shell.session().state().clone();
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shop_quantity.as_ref().expect("quantity").item_id,
        "SUPER_POTION"
    );
    let item = &r.data().items["SUPER_POTION"];
    press(&mut c, GameButton::A);
    assert_eq!(c.shell.shell.session().state().money, before.money);
    press(&mut c, GameButton::A);
    let state = c.shell.shell.session().state();
    assert_eq!(state.money, before.money - u32::from(item.price));
    assert_eq!(state.bag.quantity(item), before.bag.quantity(item) + 1);
    for _ in 0..96 {
        let s = c.snapshot().expect("shop logout");
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
    leave(&mut c, &r, (6, 6), (55, 16));
    assert_eq!(progress(&c), [false; 3]);
    let _continued = continued(&mut c, &root, &r, "ordinary-services");
}
