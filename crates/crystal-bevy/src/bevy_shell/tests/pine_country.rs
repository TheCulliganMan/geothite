//! Populated Pine Country review through production inputs after explicit setup.
//! No earned BADGE5/Q04 journey or regional encounter balance is claimed.
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
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, opposite, position,
};

const CABIN: &str = "MnR06NeedleCabin";
fn fixture(at: Point, populated: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_PINE_COUNTRY_TEST_PACK").expect("external R06 pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified published pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("MnR06C{:02}R{:02}", at.0 / 64, at.1 / 48),
            tile_x: at.0 % 64,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit R06 fixture placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    if populated {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 60, Dv::default())
            .expect("source-created HM-compatible fixture partner");
        pokemon.moves = ["PSYCHIC_M", "THUNDERBOLT", "CUT", "SURF"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
        pokemon.hp = pokemon.max_hp - 60;
        pokemon.status = Some("POISON".into());
        let state = controller.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.badges.johto = [true, true, true, true, true, false, false, false];
        state.sync_party_from_storage();
        state.random_state = Default::default();
        *controller.shell.shell.session_mut().divider_mut_for_tests() =
            crystal_core::random::RuntimeDividerSource::replay(
                (0_u32..262144)
                    .map(|n| ((n * 73 + 19) % 256) as u8)
                    .collect::<Vec<u8>>(),
            );
    }
    // All mutation above is explicit initial fixture setup. Below, gameplay is
    // driven only through the production controller, including random battles.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(position(&mut controller), at);
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime, start: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR06",
            columns: 1,
            rows: 2,
            tileset: "mn_headwaters",
            start,
            doors: BTreeMap::from([((9, 61), CABIN)]),
        },
    )
}

fn event(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("authored R06 flag")
}

fn quantity(controller: &mut VisibleShellController, item: &str) -> u16 {
    let bag = controller.snapshot().expect("actual resources").bag;
    bag.items
        .iter()
        .chain(&bag.balls)
        .filter(|slot| slot.item_id == item)
        .map(|slot| slot.quantity)
        .sum()
}

#[derive(Default)]
struct Journey {
    opponents: BTreeMap<String, BTreeSet<String>>,
    pp_spending_victories: BTreeSet<String>,
    wild_encounters: usize,
}

impl Journey {
    fn settle(&mut self, controller: &mut VisibleShellController) {
        for _ in 0..128 {
            let snapshot = controller.snapshot().expect("native field activity");
            if snapshot.battle.is_some() {
                self.battle(controller);
            } else if snapshot.phase == RuntimeShellPhase::Text
                || controller.has_pending_script_work()
                || controller.shell.field_notice.is_some()
                || !controller.shell.battle_messages.is_empty()
            {
                controller
                    .press(GameButton::A)
                    .expect("native dialogue continuation");
                controller
                    .wait_frames(16)
                    .expect("native text presentation");
            } else {
                return;
            }
            assert_eq!(controller.shell.last_error, None);
        }
        panic!("Pine Country activity did not release input");
    }

    fn battle(&mut self, controller: &mut VisibleShellController) {
        let initial = controller.snapshot().expect("actual encounter");
        let kind = &initial.battle.as_ref().expect("encounter battle").kind;
        let trainer = match kind {
            RuntimeBattleKind::Trainer { trainer_id, .. } => Some(trainer_id.clone()),
            RuntimeBattleKind::Wild { .. } => None,
            other => panic!("unexpected route battle {other:?}"),
        };
        self.wild_encounters += usize::from(trainer.is_none());
        let initial_pp: u32 = initial
            .party
            .slots
            .iter()
            .flat_map(|slot| &slot.pokemon.moves)
            .map(|m| u32::from(m.current_pp))
            .sum();
        let initial_exp = initial.party.slots[0].pokemon.experience;
        let mut selected_attacks = 0;
        for _ in 0..2048 {
            let snapshot = controller.snapshot().expect("production battle");
            let Some(battle) = &snapshot.battle else {
                break;
            };
            if let Some(id) = &trainer {
                assert!(
                    matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_id, .. } if trainer_id == id)
                );
                self.opponents
                    .entry(id.clone())
                    .or_default()
                    .insert(battle.enemy_pokemon.species.id.clone());
            }
            if !controller.shell.battle_messages.is_empty() || snapshot.ui.menu.is_none() {
                controller
                    .wait_frames(16)
                    .expect("native battle narration frames");
                controller
                    .press(GameButton::A)
                    .expect("native battle narration");
                continue;
            }
            let button = if snapshot.ui.menu.as_ref().is_some_and(|menu| {
                menu.layout
                    .vertical_menus
                    .iter()
                    .flat_map(|m| &m.options)
                    .any(|s| s.contains("change <PKMN>"))
            }) {
                GameButton::B
            } else if let Some(cursor) = &controller.shell.battle_move_cursor {
                assert!(trainer.is_some(), "wild encounters must use native RUN");
                let preferred = if ["TICKLET", "NOTCHBILL", "BOOMRUFF"]
                    .contains(&battle.enemy_pokemon.species.id.as_str())
                {
                    "THUNDERBOLT"
                } else {
                    "PSYCHIC_M"
                };
                let target = battle
                    .player_moves
                    .iter()
                    .position(|m| m.name == preferred && m.current_pp > 0)
                    .expect("legal selected attack with PP");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    selected_attacks += 1;
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_action_cursor {
                let target = if trainer.is_some() { 0 } else { 3 };
                if cursor.option_index / 2 < target / 2 {
                    GameButton::Down
                } else if cursor.option_index / 2 > target / 2 {
                    GameButton::Up
                } else if cursor.option_index % 2 < target % 2 {
                    GameButton::Right
                } else if cursor.option_index % 2 > target % 2 {
                    GameButton::Left
                } else {
                    GameButton::A
                }
            } else {
                GameButton::A
            };
            controller.press(button).expect("production battle choice");
            controller
                .wait_frames(16)
                .expect("native battle presentation");
            assert_eq!(controller.shell.last_error, None);
        }
        let ended = controller.snapshot().expect("battle result");
        assert!(ended.battle.is_none(), "native battle did not end");
        assert_eq!(
            ended.overworld.map_name, initial.overworld.map_name,
            "review partner must not black out"
        );
        assert!(ended.party.slots[0].pokemon.hp > 0);
        if let Some(id) = trainer {
            let final_pp: u32 = ended
                .party
                .slots
                .iter()
                .flat_map(|slot| &slot.pokemon.moves)
                .map(|m| u32::from(m.current_pp))
                .sum();
            assert!(
                selected_attacks > 0 && final_pp < initial_pp,
                "selected attack must spend authoritative PP"
            );
            assert!(
                ended.party.slots[0].pokemon.experience > initial_exp,
                "actual trainer victory must award experience"
            );
            self.pp_spending_victories.insert(id);
        }
        finish_text(controller);
    }

    fn walk(&mut self, controller: &mut VisibleShellController, grid: &RouteGrid, to: Point) {
        for button in grid.path(position(controller), to) {
            let before = position(controller);
            for _ in 0..8 {
                controller
                    .press(button)
                    .expect("production populated movement");
                self.settle(controller);
                let snapshot = controller.snapshot().expect("settled journey tile");
                if snapshot.overworld.map_name == CABIN {
                    assert_eq!(to, (9, 61));
                    return;
                }
                assert!(
                    snapshot.overworld.map_name.starts_with("MnR06C"),
                    "unexpected journey map {}",
                    snapshot.overworld.map_name
                );
                if position(controller) != before {
                    break;
                }
            }
            assert_ne!(
                position(controller),
                before,
                "blocked journey input {before:?} {button:?}"
            );
        }
        assert_eq!(position(controller), to);
    }

    fn talk(
        &mut self,
        controller: &mut VisibleShellController,
        grid: &RouteGrid,
        at: Point,
        expected: &str,
    ) {
        let (approach, button) = neighbors(at)
            .into_iter()
            .find(|(p, _)| {
                grid.floor.contains(p)
                    && !grid.blocked.contains(p)
                    && !grid.hazards.contains(p)
                    && p.1 / 48 == at.1 / 48
            })
            .expect("safe interaction approach");
        self.walk(controller, grid, approach);
        self.read(controller, opposite(button), expected);
    }

    fn read(&mut self, controller: &mut VisibleShellController, face: GameButton, expected: &str) {
        controller.press(face).expect("face native interaction");
        controller.press(GameButton::A).expect("native interaction");
        let snapshot = controller.snapshot().expect("owned authored dialogue");
        assert!(
            snapshot
                .ui
                .text
                .as_ref()
                .and_then(|t| t.asm_text.as_ref())
                .is_some_and(|text| text.contains(expected)),
            "missing {expected}: {:?}",
            snapshot.ui
        );
        let before = snapshot.overworld.tile;
        controller
            .press(GameButton::Down)
            .expect("direction while text owns input");
        assert_eq!(
            controller
                .snapshot()
                .expect("dialogue blocks walking")
                .overworld
                .tile,
            before
        );
        self.settle(controller);
    }
}

fn opened_grid(runtime: &CrystalRuntime, controller: &mut VisibleShellController) -> RouteGrid {
    let mut grid = grid(runtime, position(controller));
    let snapshot = controller.snapshot().expect("authoritative post-Cut map");
    assert_eq!(
        snapshot.overworld.map_name, "MnR06C00R00",
        "lookout is in northern chunk"
    );
    let map = &controller.shell.shell.session().overworld().map;
    let collisions = &runtime.data().tilesets["mn_headwaters"].collision;
    // Read actual metatile overrides for fixture-only button planning. This
    // never rewrites map data, and production input validates every edge.
    for y in 0..48_i16 {
        for x in 0..64_i16 {
            let block = map.metatile_at(x / 2, y / 2).expect("native current block");
            if matches!(
                collisions[&format!("{block:02x}")][(y % 2 * 2 + x % 2) as usize].as_str(),
                "FLOOR" | "TALL_GRASS"
            ) {
                grid.floor.insert((x, y));
            }
        }
    }
    grid
}

fn cut_lookout(controller: &mut VisibleShellController) {
    assert_eq!(position(controller), (29, 33));
    controller.press(GameButton::Up).expect("face native gate");
    assert_eq!(position(controller), (29, 33));
    for _ in 0..96 {
        let block = controller
            .shell
            .shell
            .session()
            .overworld()
            .map
            .metatile_at(14, 16)
            .expect("native tree block");
        if block == 0x3c {
            finish_text(controller);
            controller.wait_frames(60).expect("Cut presentation");
            return;
        }
        controller
            .press(GameButton::A)
            .expect("native Cut prompt/confirmation/use");
    }
    panic!("native lookout Cut did not apply source5b→3c");
}

fn restore_in_cabin(
    journey: &mut Journey,
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
) {
    indoor_walk(controller, runtime, (5, 4));
    let tired = controller
        .snapshot()
        .expect("real tired party before keeper");
    assert!(tired.party.slots.iter().any(|slot| {
        slot.pokemon.hp < slot.pokemon.max_hp
            || slot.pokemon.status.is_some()
            || slot
                .pokemon
                .moves
                .iter()
                .any(|m| m.current_pp < runtime.data().moves[&m.name].pp)
    }));
    journey.read(controller, GameButton::Up, "Rest here a while");
    let rested = controller.snapshot().expect("actual authored HealParty");
    for slot in rested.party.slots {
        assert_eq!(slot.pokemon.hp, slot.pokemon.max_hp);
        assert_eq!(slot.pokemon.status, None);
        for learned in slot.pokemon.moves {
            assert_eq!(learned.current_pp, runtime.data().moves[&learned.name].pp);
        }
    }
}

fn leave_cabin(controller: &mut VisibleShellController, runtime: &CrystalRuntime, carpet_x: i16) {
    indoor_walk(controller, runtime, (carpet_x, 6));
    indoor_walk(controller, runtime, (carpet_x, 7));
    for _ in 0..4 {
        if controller
            .snapshot()
            .expect("carpet return")
            .overworld
            .map_name
            != CABIN
        {
            break;
        }
        controller
            .press(GameButton::Down)
            .expect("leave cabin carpet");
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("cabin safe exterior")
            .overworld
            .map_name,
        "MnR06C00R01"
    );
    assert_eq!(position(controller), (9, 62));
}

fn use_collected_super_potion(controller: &mut VisibleShellController) {
    finish_text(controller);
    controller
        .press(GameButton::Start)
        .expect("production Start");
    let snapshot = controller.snapshot().expect("Start options");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|o| *o == super::super::StartMenuOption::Pack)
        .expect("PACK option");
    for _ in 0..16 {
        let current = controller
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("open PACK");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select PACK");
    }
    let bag = controller.snapshot().expect("carried medicine").bag;
    let target = bag
        .items
        .iter()
        .filter(|s| s.quantity > 0)
        .position(|s| s.item_id == "SUPER_POTION")
        .expect("collected Super Potion");
    for _ in 0..16 {
        let current = controller
            .shell
            .bag_cursor
            .as_ref()
            .expect("production bag cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("medicine actions");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select medicine");
    }
    let snapshot = controller.snapshot().expect("medicine actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("production item actions");
    let target = actions
        .iter()
        .position(|o| *o == super::super::FieldPackAction::Use)
        .expect("USE action");
    for _ in 0..16 {
        let current = controller
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("use medicine");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select USE");
    }
    assert!(
        controller.shell.field_pack_target_mode
            == Some(super::super::FieldPackTargetMode::PartyPokemon),
        "medicine target did not open; selected={:?}, ui={:?}",
        super::super::selected_field_pack_item_id(&mut controller.shell),
        controller
            .snapshot()
            .expect("medicine target diagnostic")
            .ui
    );
    for _ in 0..6 {
        if controller.shell.party_cursor == 0 {
            break;
        }
        controller
            .press(GameButton::Up)
            .expect("select lead partner");
    }
    assert_eq!(controller.shell.party_cursor, 0);
    let before = controller.snapshot().expect("injured Mew").party.slots[0]
        .pokemon
        .hp;
    controller
        .press(GameButton::A)
        .expect("apply Super Potion to Mew");
    let healed = controller.snapshot().expect("native healing result");
    assert!(
        healed.party.slots[0].pokemon.hp == (before + 50).min(healed.party.slots[0].pokemon.max_hp),
        "medicine did not restore HP: before={before}, after={}, target={:?}, notice={:?}, field_notice={:?}, ui={:?}",
        healed.party.slots[0].pokemon.hp,
        controller.shell.field_pack_target_mode,
        controller.shell.visible_field_item_notice,
        controller.shell.field_notice,
        healed.ui
    );
    assert!(
        !healed
            .bag
            .items
            .iter()
            .any(|s| s.item_id == "SUPER_POTION" && s.quantity > 0),
        "medicine not consumed"
    );
    for _ in 0..16 {
        let current = controller.snapshot().expect("close field Pack");
        if controller.shell.field_notice.is_some() || current.ui.pending_text_wait.is_some() {
            controller
                .wait_frames(60)
                .expect("healing presentation frames");
            controller.press(GameButton::A).expect("healing notice");
        } else if current.ui.menu.is_some() {
            controller
                .press(GameButton::B)
                .expect("close medicine menu");
        } else {
            return;
        }
    }
    let final_ui = controller.snapshot().expect("healing close diagnostic").ui;
    panic!(
        "medicine did not return to walking: field_notice={:?}, item_notice={:?}, ui={:?}",
        controller.shell.field_notice, controller.shell.visible_field_item_notice, final_ui
    );
}

#[test]
#[ignore = "requires the central lake variation external pack"]
fn pine_country_controller_walks_shore_promontory_detour_both_directions() {
    let (_, runtime, mut controller) = fixture((50, 35), false);
    let grid = grid(&runtime, (50, 35));
    let mut journey = Journey::default();
    for _ in 0..4 {
        controller
            .press(GameButton::Down)
            .expect("the pine promontory blocks the old straight road");
    }
    assert_eq!(position(&mut controller), (50, 35));
    let bends = [(50, 34), (54, 34), (54, 40), (50, 40)];
    for at in bends {
        journey.walk(&mut controller, &grid, at);
    }
    assert_eq!(position(&mut controller), (50, 40));
    for at in bends.into_iter().rev() {
        journey.walk(&mut controller, &grid, at);
    }
    journey.walk(&mut controller, &grid, (50, 35));
    assert_eq!(position(&mut controller), (50, 35));
    assert_eq!(controller.shell.last_error, None);
}

#[test]
#[ignore = "requires the published external Pine Country or combined northern-lakes pack"]
fn pine_country_controller_populated_five_trainers_cabin_resources_cut_return_and_continue() {
    let (root, runtime, mut controller) = fixture((16, 80), true);
    let grid = grid(&runtime, (16, 80));
    let mut journey = Journey::default();
    let badges = controller.shell.shell.session().state().badges.clone();
    // Explicit overlevel review partner, injury, poison, five badges and seeded
    // DIV are setup only. The actual inherited battle policy stays intact.
    journey.walk(&mut controller, &grid, (16, 84));
    journey.talk(&mut controller, &grid, (13, 71), "ST CLOUD is south");
    // If the initial crossing rolls no wildlife, continue genuine walking in
    // that same required habitat. Do not inject an encounter or reset RNG.
    for _ in 0..32 {
        if journey.wild_encounters > 0 {
            break;
        }
        journey.walk(&mut controller, &grid, (16, 77));
        journey.walk(&mut controller, &grid, (16, 78));
    }
    assert!(
        journey.wild_encounters > 0,
        "required habitat must produce a real native encounter/RUN"
    );
    journey.talk(&mut controller, &grid, (23, 73), "put the");
    assert_eq!(quantity(&mut controller, "SUPER_POTION"), 1);
    assert!(event(&controller, "EVENT_MN_R06_ITEM_01"));
    use_collected_super_potion(&mut controller);
    assert_eq!(quantity(&mut controller, "SUPER_POTION"), 0);
    journey.talk(&mut controller, &grid, (21, 69), "CLINKLAW");
    assert!(event(&controller, "EVENT_MN_R06_TRAINER_01"));

    journey.walk(&mut controller, &grid, (9, 61));
    let room = controller.snapshot().expect("actual cabin entry");
    assert_eq!(room.overworld.map_name, CABIN);
    // This conventional native single-mouth room enters warp1 on its carpet;
    // the declared clear Floor is reached with real input, not a passive warp.
    assert_eq!((room.overworld.tile.x, room.overworld.tile.y), (5, 7));
    indoor_walk(&mut controller, &runtime, (5, 6));
    indoor_walk(&mut controller, &runtime, (6, 6));
    indoor_walk(&mut controller, &runtime, (5, 6));
    assert_eq!(
        controller
            .snapshot()
            .expect("ordinary room Floor")
            .overworld
            .map_name,
        CABIN
    );
    restore_in_cabin(&mut journey, &mut controller, &runtime);
    indoor_walk(&mut controller, &runtime, (7, 4));
    journey.read(&mut controller, GameButton::Up, "put the");
    assert!(event(&controller, "EVENT_MN_R06_ITEM_03"));
    assert_eq!(quantity(&mut controller, "REPEL"), 1);
    leave_cabin(&mut controller, &runtime, 5);
    journey.walk(&mut controller, &grid, (9, 63));
    journey.walk(&mut controller, &grid, (9, 62));
    assert_eq!(
        controller
            .snapshot()
            .expect("passive exterior Floor")
            .overworld
            .map_name,
        "MnR06C00R01"
    );
    journey.walk(&mut controller, &grid, (9, 61));
    assert_eq!(
        controller
            .snapshot()
            .expect("repeat cabin entry")
            .overworld
            .map_name,
        CABIN
    );
    assert!(event(&controller, "EVENT_MN_R06_ITEM_03"));
    assert_eq!(quantity(&mut controller, "REPEL"), 1);
    leave_cabin(&mut controller, &runtime, 6);

    for at in [(25, 67), (39, 67)] {
        journey.walk(&mut controller, &grid, at);
    }
    journey.talk(&mut controller, &grid, (45, 55), "Dry road or grass");
    journey.talk(&mut controller, &grid, (57, 53), "put the");
    assert_eq!(quantity(&mut controller, "GREAT_BALL"), 1);
    journey.talk(&mut controller, &grid, (43, 43), "No bites today");
    journey.walk(&mut controller, &grid, (31, 43));
    journey.talk(&mut controller, &grid, (33, 43), "Quiet water");
    journey.talk(&mut controller, &grid, (51, 31), "Listen by the lake");
    journey.walk(&mut controller, &grid, (29, 33));
    cut_lookout(&mut controller);
    let opened = opened_grid(&runtime, &mut controller);
    journey.talk(&mut controller, &opened, (27, 23), "pine lookout");
    journey.talk(&mut controller, &opened, (13, 27), "put the");
    assert_eq!(quantity(&mut controller, "ETHER"), 1);
    journey.walk(&mut controller, &opened, (13, 29));
    journey.walk(&mut controller, &opened, (21, 29));
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native lookout ledge");
        journey.settle(&mut controller);
        if position(&mut controller) == (21, 32) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (21, 32));
    controller
        .press(GameButton::Up)
        .expect("ledge reverse barrier");
    journey.settle(&mut controller);
    assert_eq!(position(&mut controller), (21, 32));
    // Native vegetation may reset on source map setup after an encounter.
    // Reopen through real input if needed, never restore an override directly.
    if controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(14, 16)
        == Some(0x5b)
    {
        journey.walk(&mut controller, &grid, (29, 33));
        cut_lookout(&mut controller);
        journey.walk(&mut controller, &opened, (21, 32));
    }
    let opened = opened_grid(&runtime, &mut controller);
    assert_eq!(
        opened.path((21, 32), (21, 29)).len(),
        21,
        "authored ordinary ramp return"
    );
    journey.walk(&mut controller, &opened, (21, 29));
    journey.walk(&mut controller, &opened, (29, 33));
    journey.walk(&mut controller, &grid, (42, 2));
    journey.talk(&mut controller, &grid, (41, 9), "patch of pines");

    assert_eq!(journey.pp_spending_victories.len(), 5);
    let expected = [
        ("MN_R06_T01", ["CLINKLAW", "DEWDANT"].as_slice()),
        ("MN_R06_T02", ["RICELING", "HISSIP"].as_slice()),
        ("MN_R06_T03", ["TICKLET"].as_slice()),
        ("MN_R06_T04", ["NOTCHBILL", "BOOMRUFF"].as_slice()),
        ("MN_R06_T05", ["VELVAWN"].as_slice()),
    ];
    for (id, species) in expected {
        assert_eq!(
            journey.opponents[id],
            species.iter().map(|s| s.to_string()).collect()
        );
    }
    let trainer_flags: BTreeMap<_, _> = (1..=5)
        .map(|i| {
            let name = format!("EVENT_MN_R06_TRAINER_{i:02}");
            let set = event(&controller, &name);
            (name, set)
        })
        .collect();
    assert!(trainer_flags.values().all(|set| *set));
    let item_flags: BTreeMap<_, _> = (1..=4)
        .map(|i| {
            let name = format!("EVENT_MN_R06_ITEM_{i:02}");
            let set = event(&controller, &name);
            (name, set)
        })
        .collect();
    assert!(item_flags.values().all(|set| *set));
    assert_eq!(controller.shell.shell.session().state().badges, badges);
    let before = controller
        .snapshot()
        .expect("completed populated Pine Country journey");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-pine-country-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller
        .save(&path)
        .expect("save actual route victories/resources");
    let mut loaded = VisibleShellController::load_save(root, runtime.clone(), path.clone(), None)
        .expect("Continue populated Pine Country");
    let after = loaded.snapshot().expect("continued journey");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(loaded.shell.shell.session().state().badges, badges);
    for (flag, set) in trainer_flags.into_iter().chain(item_flags) {
        assert_eq!(event(&loaded, &flag), set);
    }
    assert_eq!(quantity(&mut loaded, "REPEL"), 1);
    assert_eq!(quantity(&mut loaded, "GREAT_BALL"), 1);
    assert_eq!(quantity(&mut loaded, "ETHER"), 1);
    assert_eq!(quantity(&mut loaded, "SUPER_POTION"), 0);
    let mut continued_journey = Journey::default();
    continued_journey.talk(&mut loaded, &grid, (51, 31), "count wings");
    assert!(
        continued_journey.pp_spending_victories.is_empty(),
        "defeated trainer must not restart a battle"
    );
    if let Some(path) = std::env::var_os("CRYSTAL_PINE_COUNTRY_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export actual populated review save");
    }
    std::fs::remove_file(path).expect("remove unique journey save");
}

#[test]
#[ignore = "requires the published R06 northern-road refinement"]
fn pine_country_controller_northern_bends_watcher_nook_and_return() {
    let (_, runtime, mut controller) = fixture((42, 2), false);
    let grid = grid(&runtime, (42, 2));
    let mut journey = Journey::default();
    journey.talk(&mut controller, &grid, (44, 4), "BRAINERD NORTH");
    for at in [(42, 6), (42, 7), (42, 8), (42, 9)] {
        journey.walk(&mut controller, &grid, at);
    }
    journey.read(&mut controller, GameButton::Left, "A patch of pines");
    // Both lanes of the new dry downhill hop, then the ordinary grassy climb.
    for x in [44, 45] {
        journey.walk(&mut controller, &grid, (x, 9));
        for _ in 0..8 {
            controller
                .press(GameButton::Down)
                .expect("bird glade ledge");
            assert_ne!(
                position(&mut controller),
                (x, 11),
                "reverse wall is not a walking tile"
            );
            if position(&mut controller) == (x, 12) {
                break;
            }
        }
        assert_eq!(position(&mut controller), (x, 12));
        for _ in 0..4 {
            controller
                .press(GameButton::Up)
                .expect("blocked reverse hop");
        }
        assert_eq!(position(&mut controller), (x, 12));
        for at in [(48, 12), (48, 9), (x, 9)] {
            journey.walk(&mut controller, &grid, at);
        }
    }
    journey.walk(&mut controller, &grid, (44, 9));
    for _ in 0..8 {
        if position(&mut controller) == (44, 12) {
            break;
        }
        controller
            .press(GameButton::Down)
            .expect("southbound road hop");
    }
    assert_eq!(position(&mut controller), (44, 12));
    let below = [
        (44, 12),
        (42, 12),
        (40, 12),
        (40, 14),
        (40, 16),
        (40, 18),
        (42, 18),
        (42, 20),
        (42, 22),
        (42, 24),
    ];
    for at in below {
        journey.walk(&mut controller, &grid, at);
    }
    for at in below.into_iter().rev() {
        journey.walk(&mut controller, &grid, at);
    }
    for at in [
        (48, 12),
        (48, 9),
        (44, 9),
        (44, 8),
        (44, 6),
        (42, 6),
        (42, 2),
    ] {
        journey.walk(&mut controller, &grid, at);
    }
    assert_eq!(position(&mut controller), (42, 2));
    assert_eq!(controller.shell.last_error, None);
    assert!(
        controller
            .snapshot()
            .expect("geometry fixture")
            .party
            .slots
            .is_empty()
    );
}
