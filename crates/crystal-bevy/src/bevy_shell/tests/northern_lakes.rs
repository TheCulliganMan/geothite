//! A populated lake-route trip after explicit legal party/badge/position setup.
//!
//! This proves two trainer victories, resources and Continue, not all eight
//! battles, earned regional access or an unauthored healing service.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, neighbors, opposite, position};

fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_NORTHERN_LAKES_TEST_PACK")
            .expect("published external Northern Lakes pack"),
    )
    .canonicalize()
    .expect("canonical completed external pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("verified lake route pack"),
    )
    .expect("native Northern Lakes runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR08C00R01".into(),
            tile_x: 26,
            tile_y: 40,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit Leech Lake trailhead fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    for (species, level, moves) in [
        ("MEW", 60, vec!["PSYCHIC_M", "SURF"]),
        ("AMPHAROS", 54, vec!["THUNDERPUNCH"]),
    ] {
        let mut partner = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("native legal fixture partner");
        partner.moves = moves
            .into_iter()
            .map(|name| {
                assert!(
                    runtime.data().pokemon[species]
                        .tmhm_learnset
                        .iter()
                        .any(|entry| entry == name)
                );
                LearnedMove {
                    name: name.into(),
                    current_pp: runtime.data().moves[name].pp,
                    pp_ups: 0,
                }
            })
            .collect();
        assert!(
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(partner)
        );
    }
    let state = controller.shell.shell.session_mut().state_mut();
    state.badges.johto = [true; 8];
    state.sync_party_from_storage();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<u8>>(),
        );
    // Setup ends here. No party, status, clock, map, flags, movement mode or
    // divider is rewritten during the journey; native encounters stay enabled.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(position(&mut controller), (26, 88));
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR08",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start: (26, 88),
            doors: BTreeMap::new(),
        },
    )
}

fn event(controller: &VisibleShellController, flag: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("registered native R08 flag")
}

fn quantity(controller: &VisibleShellController, runtime: &CrystalRuntime, item: &str) -> u16 {
    controller
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[item])
}

#[derive(Default)]
struct Journey {
    maps: BTreeSet<String>,
    wild_battles: usize,
}

impl Journey {
    fn settle(
        &mut self,
        controller: &mut VisibleShellController,
        trainer: Option<&str>,
    ) -> BTreeSet<String> {
        let mut opponents = BTreeSet::new();
        for _ in 0..256 {
            let snapshot = controller.snapshot().expect("native lake activity");
            self.maps.insert(snapshot.overworld.map_name.clone());
            if snapshot.battle.is_some() {
                opponents.extend(self.battle(controller, trainer));
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
                controller
                    .press(GameButton::A)
                    .expect("native lake dialogue acknowledgment");
                controller
                    .wait_frames(16)
                    .expect("native text presentation");
            } else {
                assert_eq!(snapshot.phase, RuntimeShellPhase::Overworld);
                assert_eq!(controller.shell.last_error, None);
                return opponents;
            }
        }
        panic!("native lake activity failed to release input");
    }

    fn battle(
        &mut self,
        controller: &mut VisibleShellController,
        expected: Option<&str>,
    ) -> BTreeSet<String> {
        let first = controller.snapshot().expect("native encounter");
        let wild = match &first.battle.as_ref().expect("active battle").kind {
            RuntimeBattleKind::Wild { .. } => {
                self.wild_battles += 1;
                true
            }
            RuntimeBattleKind::Trainer { event_flag, .. } => {
                assert_eq!(
                    Some(event_flag.as_str()),
                    expected,
                    "unexpected trainer on a planned safe path"
                );
                false
            }
            other => panic!("unexpected lake battle kind {other:?}"),
        };
        let mut opponents = BTreeSet::new();
        for _ in 0..1024 {
            let snapshot = controller.snapshot().expect("production lake battle");
            let Some(battle) = &snapshot.battle else {
                return opponents;
            };
            if !wild {
                opponents.insert(battle.enemy_pokemon.species.id.clone());
            }
            let options = snapshot
                .ui
                .menu
                .as_ref()
                .map(|menu| {
                    menu.layout
                        .vertical_menus
                        .iter()
                        .flat_map(|m| &m.options)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let button = if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
                GameButton::A
            } else if options
                .iter()
                .any(|option| option.contains("change <PKMN>"))
            {
                GameButton::B
            } else if options.iter().any(|option| option.contains("FIGHT")) {
                let cursor = controller
                    .shell
                    .battle_action_cursor
                    .as_ref()
                    .expect("native action cursor")
                    .option_index;
                if wild {
                    match cursor {
                        0 | 1 => GameButton::Down,
                        2 => GameButton::Right,
                        3 => GameButton::A,
                        _ => panic!("invalid battle cursor"),
                    }
                } else if cursor / 2 > 0 {
                    GameButton::Up
                } else if !cursor.is_multiple_of(2) {
                    GameButton::Left
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_party_action_cursor {
                if cursor.option_index > 0 {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_switch_cursor {
                let slot = snapshot
                    .party
                    .slots
                    .iter()
                    .position(|slot| {
                        slot.pokemon.hp > 0 && Some(slot.index) != battle.active_player_party_index
                    })
                    .expect("living native replacement");
                if cursor.option_index < slot {
                    GameButton::Down
                } else if cursor.option_index > slot {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_move_cursor {
                let target = ["PSYCHIC_M", "SURF", "THUNDERPUNCH"]
                    .into_iter()
                    .find_map(|name| {
                        battle
                            .player_moves
                            .iter()
                            .position(|m| m.name == name && m.current_pp > 0)
                    })
                    .expect("legal usable selected move");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else {
                GameButton::A
            };
            if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
                controller
                    .wait_frames(30)
                    .expect("native battle presentation");
            }
            controller
                .press(button)
                .expect("production lake battle input");
        }
        panic!("native lake battle failed to finish");
    }

    fn walk(&mut self, controller: &mut VisibleShellController, grid: &RouteGrid, target: Point) {
        for button in grid.path(position(controller), target) {
            let before = position(controller);
            let next = neighbors(before)
                .into_iter()
                .find(|(_, direction)| *direction == button)
                .expect("adjacent planned input")
                .0;
            for _ in 0..8 {
                controller
                    .press(button)
                    .expect("production populated route step");
                self.settle(controller, None);
                if position(controller) == next {
                    break;
                }
                assert_eq!(
                    position(controller),
                    before,
                    "route step skipped its neighbor"
                );
            }
            assert_eq!(
                position(controller),
                next,
                "blocked lake route step {before:?}->{next:?}"
            );
        }
        assert_eq!(position(controller), target);
    }

    fn approach(
        &mut self,
        controller: &mut VisibleShellController,
        grid: &RouteGrid,
        actor: Point,
    ) {
        let (at, facing) = neighbors(actor)
            .into_iter()
            .find(|(point, _)| {
                grid.floor.contains(point)
                    && !grid.blocked.contains(point)
                    && !grid.hazards.contains(point)
                    && (point.0 / 64, point.1 / 48) == (actor.0 / 64, actor.1 / 48)
            })
            .expect("safe same-chunk actor approach");
        self.walk(controller, grid, at);
        controller
            .press(opposite(facing))
            .expect("face authored lake actor");
    }

    fn interact(
        &mut self,
        controller: &mut VisibleShellController,
        grid: &RouteGrid,
        actor: Point,
    ) -> String {
        self.approach(controller, grid, actor);
        controller
            .press(GameButton::A)
            .expect("native resident or resource");
        let snapshot = controller.snapshot().expect("authored lake dialogue");
        assert_eq!(snapshot.phase, RuntimeShellPhase::Text);
        let text = snapshot
            .ui
            .text
            .expect("visible native text")
            .asm_text
            .unwrap_or_default();
        let before = position(controller);
        controller
            .press(GameButton::Down)
            .expect("movement while native dialogue owns input");
        assert_eq!(position(controller), before);
        self.settle(controller, None);
        text
    }

    fn trainer(
        &mut self,
        controller: &mut VisibleShellController,
        grid: &RouteGrid,
        actor: Point,
        flag: &str,
    ) -> BTreeSet<String> {
        assert!(!event(controller, flag));
        self.approach(controller, grid, actor);
        controller
            .press(GameButton::A)
            .expect("native trainer challenge");
        let opponents = self.settle(controller, Some(flag));
        assert!(
            event(controller, flag),
            "actual trainer victory must set its native flag"
        );
        opponents
    }
}

#[test]
#[ignore = "requires the published external Northern Lakes pack"]
fn northern_lakes_populated_journey_spends_selected_pp_collects_open_resources_and_continues() {
    let (root, runtime, mut controller) = fixture();
    let grid = grid(&runtime);
    let mut journey = Journey::default();
    let initial = controller
        .snapshot()
        .expect("initial legal partner fixture");
    let initial_pp = initial.party.slots[0].pokemon.moves[0].current_pp;
    let initial_money = controller.shell.shell.session().state().money;
    assert!(
        journey
            .interact(&mut controller, &grid, (29, 89))
            .contains("BEMIDJI")
    );
    assert_eq!(
        journey.trainer(&mut controller, &grid, (21, 85), "EVENT_MN_R08_TRAINER_01"),
        BTreeSet::from(["PINECO".into(), "WEEPINBELL".into()])
    );
    assert!(
        controller
            .snapshot()
            .expect("selected Psychic PP")
            .party
            .slots[0]
            .pokemon
            .moves[0]
            .current_pp
            < initial_pp
    );
    assert!(controller.shell.shell.session().state().money > initial_money);
    assert!(
        !journey
            .interact(&mut controller, &grid, (21, 85))
            .is_empty()
    );
    let before_nell = controller
        .snapshot()
        .expect("second native trainer approach")
        .party
        .slots[0]
        .pokemon
        .moves[0]
        .current_pp;
    let money_before_nell = controller.shell.shell.session().state().money;
    assert_eq!(
        journey.trainer(&mut controller, &grid, (69, 85), "EVENT_MN_R08_TRAINER_08"),
        BTreeSet::from(["PIDGEOTTO".into(), "WEEPINBELL".into()])
    );
    assert!(
        controller
            .snapshot()
            .expect("second selected Psychic PP")
            .party
            .slots[0]
            .pokemon
            .moves[0]
            .current_pp
            < before_nell
    );
    assert!(controller.shell.shell.session().state().money > money_before_nell);
    assert!(
        !journey
            .interact(&mut controller, &grid, (69, 85))
            .is_empty()
    );
    for (actor, item, flag) in [
        ((59, 89), "ETHER", "EVENT_MN_R08_ITEM_04"),
        ((65, 27), "REPEL", "EVENT_MN_R08_ITEM_03"),
        ((31, 15), "SUPER_POTION", "EVENT_MN_R08_ITEM_01"),
    ] {
        let before = quantity(&controller, &runtime, item);
        assert!(!event(&controller, flag));
        assert!(!journey.interact(&mut controller, &grid, actor).is_empty());
        assert_eq!(quantity(&controller, &runtime, item), before + 1);
        assert!(event(&controller, flag));
        if item == "ETHER" {
            assert!(
                !journey
                    .interact(&mut controller, &grid, (65, 72))
                    .is_empty()
            );
        }
    }
    assert!(
        !journey
            .interact(&mut controller, &grid, (39, 39))
            .is_empty()
    );
    journey.walk(&mut controller, &grid, (40, 6));
    assert_eq!(
        journey.maps,
        BTreeSet::from([
            "MnR08C00R00".into(),
            "MnR08C01R00".into(),
            "MnR08C00R01".into(),
            "MnR08C01R01".into(),
        ])
    );
    for number in 2..=7 {
        assert!(!event(
            &controller,
            &format!("EVENT_MN_R08_TRAINER_{number:02}")
        ));
    }
    assert!(!event(&controller, "EVENT_MN_R08_ITEM_02"));
    assert_eq!(quantity(&controller, &runtime, "GREAT_BALL"), 0);
    let before = controller.snapshot().expect("actual Bemidji landing state");
    let saved_money = controller.shell.shell.session().state().money;
    assert!(before.party.slots.iter().any(|slot| slot.pokemon.hp > 0));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r08-journey-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller
        .save(&path)
        .expect("save actual populated lake trip");
    let mut loaded = VisibleShellController::load_save(root, runtime.clone(), path.clone(), None)
        .expect("Continue actual lake trip");
    let after = loaded.snapshot().expect("continued native lake result");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(loaded.shell.shell.session().state().money, saved_money);
    for flag in [
        "EVENT_MN_R08_TRAINER_01",
        "EVENT_MN_R08_TRAINER_08",
        "EVENT_MN_R08_ITEM_01",
        "EVENT_MN_R08_ITEM_03",
        "EVENT_MN_R08_ITEM_04",
    ] {
        assert!(event(&loaded, flag));
    }
    assert!(!event(&loaded, "EVENT_MN_R08_ITEM_02"));
    if let Some(save) = std::env::var_os("CRYSTAL_NORTHERN_LAKES_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(save))
            .expect("export actual local journey state");
    }
    std::fs::remove_file(path).expect("remove owned temporary lake journey save");
}

#[test]
#[ignore = "requires the native Cedar Lake shore-walk revision"]
fn northern_lakes_cedar_shore_walk_and_shorter_meadow_route_use_real_inputs() {
    let (_, runtime, mut controller) = fixture();
    let grid = grid(&runtime);
    let mut journey = Journey::default();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/routes/MN_R08.cedar.walk.json");
    let plan: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(path).expect("authored Cedar Lake playtest contract"),
    )
    .expect("Cedar Lake path contract");
    let dry: Vec<Point> = serde_json::from_value(plan["dry_path_global"].clone())
        .expect("authored dry walking inputs");
    let grass: Vec<Point> = serde_json::from_value(plan["grass_path_global"].clone())
        .expect("authored meadow walking inputs");
    assert_eq!(dry.len(), 37);
    assert_eq!(grass.len(), 29);
    assert_eq!(dry.first(), grass.first());
    assert_eq!(dry.last(), grass.last());
    journey.walk(&mut controller, &grid, dry[0]);
    let reverse_grass = grass.iter().copied().rev().collect::<Vec<_>>();
    for (path, expected_grass) in [(&dry, 0), (&reverse_grass, 15)] {
        assert_eq!(position(&mut controller), path[0]);
        let mut grass_tiles = 0;
        for pair in path.windows(2) {
            let (from, to) = (pair[0], pair[1]);
            assert_eq!((from.0 - to.0).abs() + (from.1 - to.1).abs(), 1);
            let name = format!("MnR08C{:02}R{:02}", to.0 / 64, to.1 / 48);
            let module = &runtime.data().maps[&name];
            let block = module.blocks[(to.1 as usize % 48 / 2)
                * usize::from(module.attributes.width)
                + to.0 as usize % 64 / 2];
            let kind = &runtime.data().tilesets["mn_headwaters"].collision[&format!("{block:02x}")]
                [(to.1 as usize % 2) * 2 + to.0 as usize % 2];
            assert!(matches!(kind.as_str(), "FLOOR" | "TALL_GRASS"));
            grass_tiles += usize::from(kind == "TALL_GRASS");
            journey.walk(&mut controller, &grid, to);
            assert_eq!(position(&mut controller), to);
        }
        assert_eq!(grass_tiles, expected_grass);
    }
    assert_eq!(position(&mut controller), dry[0]);
    assert_eq!(controller.shell.last_error, None);
}
