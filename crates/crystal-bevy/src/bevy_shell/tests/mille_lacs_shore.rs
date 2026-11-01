//! SQ04 through production input after explicit initial badge/inventory setup.
//!
//! Empty parties separate the walking quest from battle balance. A separate
//! legal Surf-partner fixture covers the optional island. No reports,
//! completion events or items are injected after initial setup.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    AssetRoot, SHORE_WATCH_REPORT_EVENTS, SHORE_WATCH_REWARDED_EVENT,
    read_loaded_verified_compiled_game_pack,
};
use crystal_core::input::GameButton;
use crystal_core::models::bag::KEY_ITEM_POCKET_CAPACITY;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, neighbors, opposite, position};

const RANGER: Point = (27, 23);
const VOLUNTEERS: [(Point, &str); 3] = [
    ((15, 33), "Reed watch here."),
    ((31, 9), "Meadow watch here."),
    ((75, 17), "Beach watch here."),
];

fn runtime() -> (AssetRoot, CrystalRuntime) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_MILLE_LACS_SHORE_TEST_PACK")
            .expect("published external R18 pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified R18 pack"),
    )
    .expect("native R18 runtime");
    (root, runtime)
}

fn fixture(
    sixth_badge: bool,
    adequate_rod: Option<&str>,
) -> (AssetRoot, CrystalRuntime, VisibleShellController, RouteGrid) {
    let (root, runtime) = runtime();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR18C00R00".into(),
            tile_x: 6,
            tile_y: 10,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit western shore placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture trainer name");
    let state = controller.shell.shell.session_mut().state_mut();
    state.badges.johto = [true, true, true, true, true, sixth_badge, false, false];
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["OLD_ROD"], 1)
            .expect("initial old rod")
    );
    if let Some(rod) = adequate_rod {
        assert!(matches!(rod, "GOOD_ROD" | "SUPER_ROD"));
        assert!(
            state
                .bag
                .add_item(&runtime.data().items[rod], 1)
                .expect("explicit existing adequate rod")
        );
    }
    // Only initial badge bits, inventory and position are fixtures. In
    // particular, every report below is earned by walking and acknowledgment.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR18",
            columns: 2,
            rows: 1,
            tileset: "mn_headwaters",
            start: (6, 10),
            doors: BTreeMap::new(),
        },
    );
    assert!(
        controller
            .snapshot()
            .expect("empty party")
            .party
            .slots
            .is_empty()
    );
    (root, runtime, controller, grid)
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller
        .press(button)
        .expect("production shore-watch input");
    controller
        .wait_frames(16)
        .expect("native dialogue presentation");
    assert_eq!(controller.shell.last_error, None);
}

fn event(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("registered SQ04 event")
}

fn quantity(controller: &VisibleShellController, runtime: &CrystalRuntime, id: &str) -> u16 {
    controller
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[id])
}

fn reports(controller: &VisibleShellController) -> [bool; 3] {
    SHORE_WATCH_REPORT_EVENTS.map(|name| event(controller, name))
}

fn talk_open(controller: &mut VisibleShellController, grid: &RouteGrid, at: Point) {
    let (approach, facing) = neighbors(at)
        .into_iter()
        .find(|(p, _)| {
            grid.floor.contains(p)
                && !grid.blocked.contains(p)
                && !grid.hazards.contains(p)
                && (p.0 / 64, p.1 / 48) == (at.0 / 64, at.1 / 48)
        })
        .expect("ordinary walking approach to volunteer");
    grid.walk(controller, approach);
    press(controller, opposite(facing));
    press(controller, GameButton::A);
    let snapshot = controller.snapshot().expect("visible authored interaction");
    assert_eq!(snapshot.phase, RuntimeShellPhase::Text);
    assert!(snapshot.ui.text.is_some());
    let at = position(controller);
    press(controller, GameButton::Down);
    assert_eq!(
        position(controller),
        at,
        "authored dialogue must own movement"
    );
}

#[derive(Default)]
struct Dialogue {
    labels: BTreeSet<String>,
    text: Vec<String>,
}

impl Dialogue {
    fn has_label(&self, suffix: &str) -> bool {
        self.labels.iter().any(|label| label.ends_with(suffix))
    }

    fn has_text(&self, expected: &str) -> bool {
        self.text.iter().any(|text| text.contains(expected))
    }
}

fn close_dialogue(controller: &mut VisibleShellController) -> Dialogue {
    let mut dialogue = Dialogue::default();
    for _ in 0..128 {
        let snapshot = controller.snapshot().expect("native script continuation");
        assert!(
            snapshot.battle.is_none(),
            "walking volunteers must not become a trainer gauntlet"
        );
        assert!(snapshot.ui.pending_yes_no.is_none());
        assert!(snapshot.pending_shop.is_none());
        if let Some(text) = snapshot.ui.text {
            dialogue.labels.insert(text.label);
            if let Some(asm_text) = text.asm_text {
                dialogue.text.push(asm_text);
            }
        }
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && snapshot.ui.pending_text_wait.is_none()
            && !controller.has_pending_script_work()
            && controller.shell.field_notice.is_none()
        {
            return dialogue;
        }
        press(controller, GameButton::A);
    }
    panic!(
        "SQ04 dialogue failed to return walking: {:?}",
        controller.snapshot().expect("diagnosis").ui
    );
}

fn speak(controller: &mut VisibleShellController, grid: &RouteGrid, at: Point) -> Dialogue {
    talk_open(controller, grid, at);
    close_dialogue(controller)
}

fn collect_reports(controller: &mut VisibleShellController, grid: &RouteGrid) {
    for (index, (at, text)) in VOLUNTEERS.into_iter().enumerate() {
        assert!(!event(controller, SHORE_WATCH_REPORT_EVENTS[index]));
        talk_open(controller, grid, at);
        assert!(
            !event(controller, SHORE_WATCH_REPORT_EVENTS[index]),
            "report cannot precede dialogue acknowledgment"
        );
        assert!(close_dialogue(controller).has_text(text));
        for (n, set) in reports(controller).into_iter().enumerate() {
            assert_eq!(set, n <= index, "only actual reports may be set");
        }
        assert!(!event(controller, SHORE_WATCH_REWARDED_EVENT));
    }
}

fn assert_optional_islet_untouched(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
) {
    assert!(!event(controller, "EVENT_MN_R18_ITEM_04"));
    assert_eq!(quantity(controller, runtime, "REVIVE"), 0);
    let snapshot = controller.snapshot().expect("walking-only SQ04");
    assert!(snapshot.party.slots.is_empty());
    assert_eq!(snapshot.overworld.mode, MovementMode::Normal);
}

#[test]
#[ignore = "requires the published external Mille Lacs Shore pack"]
fn mille_lacs_shore_controller_ranger_badge_denial_preserves_reports_and_inventory() {
    let (_, runtime, mut controller, grid) = fixture(false, None);
    let before = controller.snapshot().expect("initial bag").bag;
    let dialogue = speak(&mut controller, &grid, RANGER);
    assert!(dialogue.has_label("MN_R18_WatchBadgeDeniedText"));
    assert!(dialogue.has_text("sixth gym badge"));
    assert_eq!(reports(&controller), [false; 3]);
    assert!(!event(&controller, SHORE_WATCH_REWARDED_EVENT));
    assert_eq!(quantity(&controller, &runtime, "GOOD_ROD"), 0);
    assert_eq!(
        before,
        controller.snapshot().expect("unchanged denial bag").bag
    );
    assert_optional_islet_untouched(&mut controller, &runtime);
    // Completing observations cannot replace the actual sixth-badge gate.
    // Reports are earned through the same production conversations; no badge,
    // inventory or report state is rewritten after the initial fixture setup.
    collect_reports(&mut controller, &grid);
    assert_eq!(reports(&controller), [true; 3]);
    let completed_reports_bag = controller.snapshot().expect("reports without badge").bag;
    let dialogue = speak(&mut controller, &grid, RANGER);
    assert!(dialogue.has_label("MN_R18_WatchBadgeDeniedText"));
    assert_eq!(reports(&controller), [true; 3]);
    assert!(!event(&controller, SHORE_WATCH_REWARDED_EVENT));
    assert_eq!(quantity(&controller, &runtime, "GOOD_ROD"), 0);
    assert_eq!(
        controller
            .snapshot()
            .expect("badge denial retains reports")
            .bag,
        completed_reports_bag
    );
    assert_optional_islet_untouched(&mut controller, &runtime);
}

#[test]
#[ignore = "requires the published external Mille Lacs Shore pack"]
fn mille_lacs_shore_controller_three_walking_reports_deliver_good_rod_and_continue() {
    let (root, runtime, mut controller, grid) = fixture(true, None);
    assert!(speak(&mut controller, &grid, RANGER).has_label("MN_R18_WatchNeedsNewsText"));
    assert_eq!(reports(&controller), [false; 3]);
    for (index, (at, text)) in VOLUNTEERS.into_iter().enumerate() {
        talk_open(&mut controller, &grid, at);
        assert!(!event(&controller, SHORE_WATCH_REPORT_EVENTS[index]));
        assert!(close_dialogue(&mut controller).has_text(text));
        let expected_reports = [true, index >= 1, index >= 2];
        assert_eq!(reports(&controller), expected_reports);
        let before_repeat = controller.snapshot().expect("reported volunteer bag").bag;
        assert!(speak(&mut controller, &grid, at).has_text(text));
        assert_eq!(reports(&controller), expected_reports);
        assert_eq!(
            controller.snapshot().expect("repeated volunteer bag").bag,
            before_repeat
        );
        assert!(!event(&controller, SHORE_WATCH_REWARDED_EVENT));
        assert_eq!(quantity(&controller, &runtime, "GOOD_ROD"), 0);
        if index < 2 {
            assert!(speak(&mut controller, &grid, RANGER).has_label("MN_R18_WatchNeedsNewsText"));
            assert!(!event(&controller, SHORE_WATCH_REWARDED_EVENT));
            assert_eq!(quantity(&controller, &runtime, "GOOD_ROD"), 0);
        }
    }
    // Beach is in the clipped eastern chunk; the same walking grid must bring
    // the player back across the real reciprocal seam for reward delivery.
    assert_eq!(
        controller
            .snapshot()
            .expect("eastern report")
            .overworld
            .map_name,
        "MnR18C01R00"
    );
    talk_open(&mut controller, &grid, RANGER);
    assert_eq!(quantity(&controller, &runtime, "GOOD_ROD"), 0);
    assert!(!event(&controller, SHORE_WATCH_REWARDED_EVENT));
    let delivered = close_dialogue(&mut controller);
    assert!(delivered.has_label("MN_R18_WatchRewardIntroText"));
    assert!(delivered.has_label("MN_R18_WatchRewardGivenText"));
    assert_eq!(quantity(&controller, &runtime, "GOOD_ROD"), 1);
    assert_eq!(quantity(&controller, &runtime, "OLD_ROD"), 1);
    assert!(event(&controller, SHORE_WATCH_REWARDED_EVENT));
    assert_optional_islet_untouched(&mut controller, &runtime);
    let before = controller.snapshot().expect("actual quest completion");
    assert!(speak(&mut controller, &grid, RANGER).has_label("MN_R18_WatchAlreadyRewardedText"));
    assert_eq!(
        controller.snapshot().expect("idempotent reward").bag,
        before.bag
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r18-watch-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller.save(&path).expect("production quest save");
    let mut loaded = VisibleShellController::load_save(root, runtime.clone(), path.clone(), None)
        .expect("Continue SQ04");
    let after = loaded.snapshot().expect("continued report and rod");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(reports(&loaded), [true; 3]);
    assert!(event(&loaded, SHORE_WATCH_REWARDED_EVENT));
    assert!(speak(&mut loaded, &grid, RANGER).has_label("MN_R18_WatchAlreadyRewardedText"));
    assert_eq!(quantity(&loaded, &runtime, "GOOD_ROD"), 1);
    assert_optional_islet_untouched(&mut loaded, &runtime);
    if let Some(save) = std::env::var_os("CRYSTAL_MILLE_LACS_SHORE_QUEST_SAVE") {
        loaded
            .save(PathBuf::from(save))
            .expect("export actual walking quest state");
    }
    std::fs::remove_file(path).expect("remove owned temporary quest save");
}

#[test]
#[ignore = "requires the published external Mille Lacs Shore pack"]
fn mille_lacs_shore_controller_existing_good_or_super_rod_is_never_duplicated_or_downgraded() {
    for rod in ["GOOD_ROD", "SUPER_ROD"] {
        let (_, runtime, mut controller, grid) = fixture(true, Some(rod));
        let before = controller
            .snapshot()
            .expect("explicit existing rod bag")
            .bag;
        collect_reports(&mut controller, &grid);
        let dialogue = speak(&mut controller, &grid, RANGER);
        assert!(dialogue.has_label("MN_R18_WatchHasRodText"));
        assert!(event(&controller, SHORE_WATCH_REWARDED_EVENT));
        assert_eq!(
            controller
                .snapshot()
                .expect("adequate rod acknowledgment")
                .bag,
            before
        );
        assert_eq!(
            quantity(&controller, &runtime, "GOOD_ROD"),
            u16::from(rod == "GOOD_ROD")
        );
        assert_eq!(
            quantity(&controller, &runtime, "SUPER_ROD"),
            u16::from(rod == "SUPER_ROD")
        );
        assert_eq!(quantity(&controller, &runtime, "OLD_ROD"), 1);
        assert!(speak(&mut controller, &grid, RANGER).has_label("MN_R18_WatchAlreadyRewardedText"));
        assert_eq!(
            controller.snapshot().expect("adequate rod repeat bag").bag,
            before
        );
        assert_optional_islet_untouched(&mut controller, &runtime);
    }
}

#[test]
#[ignore = "requires the published external Mille Lacs Shore pack"]
fn mille_lacs_shore_controller_all_report_subsets_use_actual_walked_dialogues() {
    for mask in 0_u8..8 {
        let (root, runtime, mut controller, grid) = fixture(true, None);
        for (index, (at, text)) in VOLUNTEERS.into_iter().enumerate() {
            if mask & (1 << index) != 0 {
                assert!(speak(&mut controller, &grid, at).has_text(text));
            }
        }
        assert_eq!(
            reports(&controller),
            [mask & 1 != 0, mask & 2 != 0, mask & 4 != 0]
        );
        if mask == 3 {
            // Save an unfinished quest before the ranger transaction. Continue
            // must retain exactly the two actual reports and no free rod/event.
            let before = controller.snapshot().expect("partial reports before save");
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("fixture clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "geothite-r18-partial-{}-{nonce}.crystalsave",
                std::process::id()
            ));
            controller
                .save(&path)
                .expect("save actual unfinished shore reports");
            controller =
                VisibleShellController::load_save(root, runtime.clone(), path.clone(), None)
                    .expect("Continue unfinished SQ04");
            let after = controller.snapshot().expect("continued partial reports");
            assert_eq!(before.party, after.party);
            assert_eq!(before.bag, after.bag);
            assert_eq!(before.overworld.map_name, after.overworld.map_name);
            assert_eq!(before.overworld.tile, after.overworld.tile);
            assert_eq!(reports(&controller), [true, true, false]);
            assert!(!event(&controller, SHORE_WATCH_REWARDED_EVENT));
            assert_eq!(quantity(&controller, &runtime, "GOOD_ROD"), 0);
            std::fs::remove_file(path).expect("remove owned partial quest save");
        }
        let dialogue = speak(&mut controller, &grid, RANGER);
        assert_eq!(event(&controller, SHORE_WATCH_REWARDED_EVENT), mask == 7);
        assert_eq!(
            quantity(&controller, &runtime, "GOOD_ROD"),
            u16::from(mask == 7)
        );
        assert!(
            dialogue.has_label(if mask == 7 {
                "MN_R18_WatchRewardGivenText"
            } else {
                "MN_R18_WatchNeedsNewsText"
            }),
            "report subset {mask:03b}"
        );
        assert_optional_islet_untouched(&mut controller, &runtime);
    }
}

#[test]
#[ignore = "documents full-pocket fixture limits against the published external catalog"]
fn mille_lacs_shore_controller_catalog_cannot_construct_a_valid_full_key_pocket_without_adequate_rod()
 {
    let (_, runtime, _, _) = fixture(true, None);
    let non_reward_keys = runtime
        .data()
        .items
        .values()
        .filter(|item| {
            item.pocket == "KEY_ITEM"
                && !matches!(item.script_name.as_str(), "GOOD_ROD" | "SUPER_ROD")
        })
        .count();
    // The actual pack has20 eligible distinct keys against25 slots. Duplicating
    // stacks or adding aliases would manufacture a different production pack.
    // The native component suite covers mod-filled storage refusal/retry using
    // explicitly cloned definitions. It is not this controller's input proof.
    assert!(
        non_reward_keys < KEY_ITEM_POCKET_CAPACITY,
        "the published catalog changed; add a real full-pocket controller transaction fixture"
    );
}

fn settle_islet_activity(controller: &mut VisibleShellController) {
    for _ in 0..256 {
        let snapshot = controller.snapshot().expect("native island input owner");
        if let Some(battle) = snapshot.battle {
            assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
            let button = if !controller.shell.battle_messages.is_empty() {
                GameButton::A
            } else if let Some(cursor) = &controller.shell.battle_action_cursor {
                match cursor.option_index {
                    0 | 1 => GameButton::Down,
                    2 => GameButton::Right,
                    3 => GameButton::A,
                    _ => panic!("invalid native battle action cursor"),
                }
            } else {
                GameButton::A
            };
            controller
                .press(button)
                .expect("native RUN from incidental lake encounter");
            controller
                .wait_frames(16)
                .expect("native battle presentation");
        } else if snapshot.phase == RuntimeShellPhase::Text
            || controller.has_pending_script_work()
            || controller.shell.field_notice.is_some()
            || !controller.shell.battle_messages.is_empty()
            || snapshot
                .ui
                .text
                .as_ref()
                .is_some_and(|text| text.label == "visible-shell:battle-message")
        {
            press(controller, GameButton::A);
        } else {
            assert_eq!(snapshot.phase, RuntimeShellPhase::Overworld);
            assert_eq!(controller.shell.last_error, None);
            return;
        }
    }
    panic!("native Surf/battle activity failed to release input");
}

fn islet_line(controller: &mut VisibleShellController, target: Point) {
    let start = position(controller);
    assert!(
        start.0 == target.0 || start.1 == target.1,
        "straight authored lake leg required"
    );
    while position(controller) != target {
        let from = position(controller);
        let (next, button) = neighbors(from)
            .into_iter()
            .find(|(point, _)| {
                (point.0 - target.0).abs() + (point.1 - target.1).abs()
                    < (from.0 - target.0).abs() + (from.1 - target.1).abs()
            })
            .expect("next adjacent input on authored lake leg");
        for _ in 0..8 {
            press(controller, button);
            settle_islet_activity(controller);
            if position(controller) == next {
                break;
            }
            assert_eq!(
                position(controller),
                from,
                "lake movement skipped its adjacent target"
            );
        }
        assert_eq!(
            position(controller),
            next,
            "blocked native lake/landing step {from:?}->{next:?}"
        );
    }
}

fn launch_islet_surf(controller: &mut VisibleShellController, facing: GameButton, water: Point) {
    assert_eq!(
        controller.snapshot().expect("dry launch").overworld.mode,
        MovementMode::Normal
    );
    press(controller, facing);
    press(controller, GameButton::A);
    for _ in 0..128 {
        if controller
            .snapshot()
            .expect("native Surf prompt state")
            .overworld
            .mode
            == MovementMode::Surf
        {
            controller
                .wait_frames(60)
                .expect("native Surf launch animation");
            settle_islet_activity(controller);
            assert_eq!(position(controller), water);
            return;
        }
        press(controller, GameButton::A);
    }
    panic!("native public Surf prompt failed to mount");
}

#[test]
#[ignore = "requires the published external R18 pack and native Surf controller"]
fn mille_lacs_shore_controller_surfs_to_optional_revive_continues_and_returns() {
    let (root, runtime) = runtime();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR18C00R00".into(),
            tile_x: 49,
            tile_y: 19,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit public-launch fixture placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture trainer name");
    let mut partner = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("native legal HM-compatible fixture partner");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: runtime.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.badges.johto = [true; 8];
    state.sync_party_from_storage();
    // Initial legal party/badges and launch placement only. Encounters remain
    // enabled; all following movement, mode changes and rewards use input.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(position(&mut controller), (49, 19));
    assert_eq!(reports(&controller), [false; 3]);
    assert!(!event(&controller, SHORE_WATCH_REWARDED_EVENT));
    assert!(!event(&controller, "EVENT_MN_R18_ITEM_04"));
    assert_eq!(quantity(&controller, &runtime, "REVIVE"), 0);

    let spec = |start| RouteSpec {
        prefix: "MnR18",
        columns: 2,
        rows: 1,
        tileset: "mn_headwaters",
        start,
        doors: BTreeMap::new(),
    };
    let mainland = RouteGrid::new_with_grass(&runtime, spec((49, 19)));
    assert!(
        !mainland.floor.contains(&(60, 32)),
        "islet must be outside the walking component"
    );
    launch_islet_surf(&mut controller, GameButton::Down, (49, 20));
    islet_line(&mut controller, (49, 31));
    islet_line(&mut controller, (60, 31));
    assert_eq!(
        controller
            .snapshot()
            .expect("water before islet")
            .overworld
            .mode,
        MovementMode::Surf
    );
    islet_line(&mut controller, (60, 32));
    assert_eq!(
        controller
            .snapshot()
            .expect("native islet dismount")
            .overworld
            .mode,
        MovementMode::Normal
    );
    let island = RouteGrid::new_with_grass(&runtime, spec((60, 32)));
    assert!(
        !island.floor.contains(&(49, 19)),
        "islet must have no walking return bypass"
    );
    assert!(!island.talk(&mut controller, (61, 35)).is_empty());
    assert!(event(&controller, "EVENT_MN_R18_ITEM_04"));
    assert_eq!(quantity(&controller, &runtime, "REVIVE"), 1);
    assert_eq!(reports(&controller), [false; 3]);
    assert!(!event(&controller, SHORE_WATCH_REWARDED_EVENT));
    let before = controller.snapshot().expect("actual island pickup state");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r18-islet-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller.save(&path).expect("production island save");
    let mut loaded = VisibleShellController::load_save(root, runtime.clone(), path.clone(), None)
        .expect("Continue on actual islet");
    let after = loaded.snapshot().expect("continued island reward");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(after.overworld.mode, MovementMode::Normal);
    assert!(event(&loaded, "EVENT_MN_R18_ITEM_04"));
    assert_eq!(quantity(&loaded, &runtime, "REVIVE"), 1);
    assert_eq!(reports(&loaded), [false; 3]);
    // The collected actor's old tile must be walkable after native Continue.
    islet_line(&mut loaded, (61, 35));
    islet_line(&mut loaded, (60, 35));
    islet_line(&mut loaded, (60, 32));
    launch_islet_surf(&mut loaded, GameButton::Up, (60, 31));
    islet_line(&mut loaded, (49, 31));
    islet_line(&mut loaded, (49, 20));
    islet_line(&mut loaded, (49, 19));
    assert_eq!(
        loaded
            .snapshot()
            .expect("public shore return")
            .overworld
            .mode,
        MovementMode::Normal
    );
    assert_eq!(quantity(&loaded, &runtime, "REVIVE"), 1);
    assert!(event(&loaded, "EVENT_MN_R18_ITEM_04"));
    assert_eq!(reports(&loaded), [false; 3]);
    assert!(!event(&loaded, SHORE_WATCH_REWARDED_EVENT));
    assert_eq!(quantity(&loaded, &runtime, "GOOD_ROD"), 0);
    std::fs::remove_file(path).expect("remove owned temporary islet save");
}
