//! Gym puzzles/battles use production input; prior badges/party are fixture setup.
#[path = "bemidji_return.rs"]
mod bemidji_return;
#[path = "brainerd_winch.rs"]
mod brainerd_winch;
#[path = "city_expansions.rs"]
mod city_expansions;
#[path = "current_gyms.rs"]
mod current_gyms;
#[path = "duluth_gates.rs"]
mod duluth_gates;
#[path = "ely_league.rs"]
mod ely_league;
#[path = "ely_pine_camp.rs"]
mod ely_pine_camp;
#[path = "gym_care.rs"]
mod gym_care;
#[path = "gym_eight_approach.rs"]
mod gym_eight_approach;
#[path = "gym_four_approach.rs"]
mod gym_four_approach;
#[path = "gym_transition_audit.rs"]
mod gym_transition_audit;
#[path = "itasca_story.rs"]
mod itasca_story;
#[path = "lake_country_story.rs"]
mod lake_country_story;
#[path = "leech_shore.rs"]
mod leech_shore;
#[path = "mankato_terraces.rs"]
mod mankato_terraces;
#[path = "minneapolis_circuit.rs"]
mod minneapolis_circuit;
#[path = "north_shore_journey.rs"]
mod north_shore_journey;
#[path = "north_shore_story.rs"]
mod north_shore_story;
#[path = "northstar_arc_campaign.rs"]
pub(super) mod northstar_arc_campaign;
#[path = "northstar_campaign.rs"]
mod northstar_campaign;
#[path = "northstar_campaign_courier.rs"]
mod northstar_campaign_courier;
#[path = "northstar_campaign_current.rs"]
mod northstar_campaign_current;
#[path = "northstar_campaign_finale.rs"]
pub(super) mod northstar_campaign_finale;
#[path = "northstar_campaign_ice.rs"]
pub(super) mod northstar_campaign_ice;
#[path = "northstar_campaign_iron.rs"]
mod northstar_campaign_iron;
#[path = "northstar_campaign_lake.rs"]
mod northstar_campaign_lake;
#[path = "northstar_campaign_league.rs"]
mod northstar_campaign_league;
#[path = "northstar_campaign_pine.rs"]
mod northstar_campaign_pine;
#[path = "northstar_campaign_review.rs"]
mod northstar_campaign_review;
#[path = "northstar_campaign_wind.rs"]
mod northstar_campaign_wind;
#[path = "northstar_earned_return.rs"]
mod northstar_earned_return;
#[path = "northstar_earned_services.rs"]
mod northstar_earned_services;
#[path = "northstar_finale.rs"]
mod northstar_finale;
#[path = "northstar_iron_course.rs"]
mod northstar_iron_course;
#[path = "northstar_league_dialogue.rs"]
mod northstar_league_dialogue;
#[path = "river_cities.rs"]
mod river_cities;
#[path = "rochester_gates.rs"]
mod rochester_gates;
#[path = "rochester_readable.rs"]
mod rochester_readable;
#[path = "state_cohesion.rs"]
mod state_cohesion;

#[path = "st_cloud_arrows.rs"]
mod st_cloud_arrows;

#[path = "st_paul_loop.rs"]
mod st_paul_loop;

#[path = "state_fair.rs"]
mod state_fair;

#[path = "city_life.rs"]
mod city_life;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::{AssetRoot, MapModule, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::collision::can_jump_ledge;
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};

type Point = (u16, u16);
const DIRECTIONS: [(i16, i16, GameButton); 4] = [
    (0, -1, GameButton::Up),
    (1, 0, GameButton::Right),
    (0, 1, GameButton::Down),
    (-1, 0, GameButton::Left),
];

fn load() -> (AssetRoot, CrystalRuntime, Value) {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_GYM_TEST_PACK")
            .expect("set CRYSTAL_GYM_TEST_PACK to the freshly published gym review pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("published external gym pack"),
    )
    .expect("gym runtime");
    let design = serde_json::from_str(
        &std::fs::read_to_string(repo.join("planning/minnesota/gyms/interiors.json"))
            .expect("authored gym source"),
    )
    .expect("design");
    (root, runtime, design)
}

fn point(value: &Value) -> Point {
    (
        value[0].as_u64().expect("x") as u16,
        value[1].as_u64().expect("y") as u16,
    )
}
fn at(c: &mut VisibleShellController) -> Point {
    let s = c.snapshot().expect("location");
    (s.overworld.tile.x as u16, s.overworld.tile.y as u16)
}
fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("gym event")
}
fn press(c: &mut VisibleShellController, b: GameButton) {
    c.press(b).expect("production gym input");
    c.wait_frames(1).expect("released native joypad frame");
    assert_eq!(c.shell.last_error, None);
}

fn controller(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    gym: &Value,
    losing: bool,
) -> VisibleShellController {
    let mut c = VisibleShellController {
        shell: initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime
                    .title_new_game_spawn_identifier()
                    .expect("original HOME"),
                map_name: gym["id"].as_str().expect("map").into(),
                tile_x: point(&gym["arrival_core"]).0 as i16,
                tile_y: point(&gym["arrival_core"]).1 as i16,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("explicit review placement"),
    };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let mut pokemon = runtime
        .data()
        .create_pokemon(
            if losing { "MAGIKARP" } else { "MEW" },
            if losing { 5 } else { 100 },
            Dv::default(),
        )
        .expect("legal review partner");
    if losing {
        pokemon.hp = 1;
    } else {
        let third_move = if gym["slot"] == 7 { "CUT" } else { "SURF" };
        pokemon.moves = ["PSYCHIC_M", "FLAMETHROWER", third_move, "THUNDERBOLT"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    let slot = gym["slot"].as_u64().expect("slot") as usize;
    state.badges.johto = std::array::from_fn(|i| i < slot - 1);
    if slot == 1 {
        state
            .flags
            .set_event_flag("EVENT_MN_DELIVERED_ROCHESTER_PARCEL", true)
            .expect("explicit Rochester prerequisite setup");
    }
    state.money = 0;
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..524288)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(at(&mut c), point(&gym["arrival_core"]));
    c
}

fn module(c: &mut VisibleShellController, runtime: &CrystalRuntime) -> MapModule {
    let name = c.snapshot().expect("map").overworld.map_name;
    let mut m = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(&name)
        .expect("native room");
    if let Some(overrides) = c
        .shell
        .shell
        .session()
        .state()
        .map_block_overrides
        .get(&name)
    {
        for ((x, y), block) in overrides {
            m.blocks[usize::from(*y) * usize::from(m.attributes.width) + usize::from(*x)] = *block;
        }
    }
    m
}

fn path(
    runtime: &CrystalRuntime,
    m: &MapModule,
    from: Point,
    to: Point,
) -> Option<Vec<GameButton>> {
    path_with_ledges(runtime, m, from, to, true)
}

fn path_with_ledges(
    runtime: &CrystalRuntime,
    m: &MapModule,
    from: Point,
    to: Point,
    allow_ledges: bool,
) -> Option<Vec<GameButton>> {
    let blocked = m
        .objects
        .iter()
        .map(|o| (o.x, o.y))
        .collect::<BTreeSet<_>>();
    let kind = |p: Point| {
        let b =
            m.blocks[usize::from(p.1 / 2) * usize::from(m.attributes.width) + usize::from(p.0 / 2)];
        runtime.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")]
            [usize::from(p.1 % 2 * 2 + p.0 % 2)]
        .as_str()
    };
    let native_map = OverworldMapData::from_attributes(&m.id, &m.attributes, m.blocks.clone());
    let collision = runtime
        .data()
        .tileset_collision(&m.attributes.tileset_name)
        .expect("native collision");
    let bodies = blocked
        .iter()
        .map(|&(x, y)| OccupiedTile {
            tile: TilePosition::new(x as i16, y as i16),
            object_identifier: None,
        })
        .collect::<Vec<_>>();
    let advance = |p: Point, dx: i16, dy: i16| -> Option<Point> {
        let mut direction = match (dx, dy) {
            (0, -1) => Direction::Up,
            (1, 0) => Direction::Right,
            (0, 1) => Direction::Down,
            (-1, 0) => Direction::Left,
            _ => unreachable!("cardinal test direction"),
        };
        let mut player = PlayerMovementState::new(TilePosition::new(p.0 as i16, p.1 as i16));
        let options = StepOptions {
            force_step_after_turn: true,
            ..Default::default()
        };
        let mut direct_wind = false;
        for _ in 0..64 {
            if direct_wind {
                // A scratch route planner mirrors native DoStep. Gameplay
                // still moves only through VisibleShellController::press.
                let (dx, dy) = direction.delta();
                player.tile.x += dx;
                player.tile.y += dy;
                if player.tile.x < 0
                    || player.tile.y < 0
                    || player.tile.x >= m.attributes.width as i16 * 2
                    || player.tile.y >= m.attributes.height as i16 * 2
                    || blocked.contains(&(player.tile.x as u16, player.tile.y as u16))
                {
                    return None;
                }
            } else if can_jump_ledge(&native_map, &collision, player.tile, direction, 1) {
                if !allow_ledges
                    || !matches!(
                        attempt_ledge_jump_with_occupied_tiles(
                            &mut player,
                            direction,
                            &native_map,
                            &collision,
                            options,
                            &bodies
                        ),
                        LedgeJumpOutcome::Jumped { .. }
                    )
                {
                    break;
                }
            } else if !matches!(
                attempt_step_with_occupied_tiles(
                    &mut player,
                    direction,
                    &native_map,
                    &collision,
                    options,
                    &bodies
                ),
                StepOutcome::Moved { .. }
            ) {
                break;
            }
            // A real connection can step beyond this map. This local planner
            // stops at its boundary; crossing is exercised separately by input.
            if player.tile.x < 0
                || player.tile.y < 0
                || player.tile.x >= m.attributes.width as i16 * 2
                || player.tile.y >= m.attributes.height as i16 * 2
            {
                return None;
            }
            let q = (player.tile.x as u16, player.tile.y as u16);
            if kind(q) == "WARP_PANEL" {
                // Plan a declared same-map lift; the test still travels through
                // production press/warp handling, never by editing the session.
                let source = m.events.warps.iter().find(|warp| (warp.x, warp.y) == q)?;
                if source.target_map != m.id
                    && Some(source.target_map_constant.as_str())
                        != m.attributes.map_constant.as_deref()
                {
                    return None;
                }
                let destination =
                    m.events.warps.iter().find(|warp| {
                        i16::try_from(warp.index).ok() == Some(source.target_warp_id)
                    })?;
                player.tile = TilePosition::new(destination.x as i16, destination.y as i16);
                break;
            }
            match kind(q) {
                "WALK_RIGHT" => {
                    direction = Direction::Right;
                    direct_wind = true;
                }
                "WALK_LEFT" => {
                    direction = Direction::Left;
                    direct_wind = true;
                }
                "ICE" => direct_wind = false,
                _ => break,
            }
        }
        let q = (player.tile.x as u16, player.tile.y as u16);
        (q != p).then_some(q)
    };
    let mut parents = BTreeMap::new();
    parents.insert(from, (from, GameButton::A));
    let mut todo = VecDeque::from([from]);
    while let Some(p) = todo.pop_front() {
        if p == to {
            break;
        }
        for (dx, dy, b) in DIRECTIONS {
            if let Some(q) = advance(p, dx, dy)
                && let std::collections::btree_map::Entry::Vacant(entry) = parents.entry(q)
            {
                entry.insert((p, b));
                todo.push_back(q);
            }
        }
    }
    if !parents.contains_key(&to) {
        return None;
    }
    let mut buttons = Vec::new();
    let mut q = to;
    while q != from {
        let (p, b) = parents[&q];
        buttons.push(b);
        q = p;
    }
    buttons.reverse();
    Some(buttons)
}

pub(super) fn interact(c: &mut VisibleShellController, runtime: &CrystalRuntime, target: Point) {
    let from = at(c);
    let m = module(c, runtime);
    let mut candidates = Vec::new();
    for (dx, dy, b) in DIRECTIONS {
        let x = target.0 as i16 - dx;
        let y = target.1 as i16 - dy;
        if x >= 0 && y >= 0 {
            let q = (x as u16, y as u16);
            if q.0 < m.attributes.width * 2
                && q.1 < m.attributes.height * 2
                && let Some(p) = path(runtime, &m, from, q)
            {
                candidates.push((p, q, b));
            }
        }
    }
    candidates.sort_by_key(|(p, _, _)| p.len());
    let (buttons, to, facing) = candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no real approach to {target:?} in {} from {from:?}", m.id));
    for b in buttons {
        let before = at(c);
        for _ in 0..8 {
            press(c, b);
            if at(c) != before {
                break;
            }
        }
        c.wait_frames(80).expect("ice slide settle");
    }
    assert_eq!(at(c), to, "native route differs from planned movement");
    press(c, facing);
    assert_eq!(at(c), to, "actor must own its collision");
    press(c, GameButton::A);
}

fn finish(c: &mut VisibleShellController) {
    for _ in 0..128 {
        let s = c.snapshot().expect("dialogue");
        assert!(s.battle.is_none());
        if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.battle_messages.is_empty()
        {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("gym dialogue did not release input");
}

fn await_choice(c: &mut VisibleShellController) {
    for _ in 0..96 {
        if c.shell.menu_cursor.is_some() {
            break;
        }
        press(c, GameButton::A);
    }
    assert!(
        c.shell.menu_cursor.is_some(),
        "native puzzle menu did not open"
    );
}

fn choose(c: &mut VisibleShellController, answer: usize) {
    await_choice(c);
    for _ in 0..16 {
        let selected = c
            .shell
            .menu_cursor
            .as_ref()
            .expect("menu cursor")
            .option_index;
        if selected == answer {
            press(c, GameButton::A);
            return;
        }
        press(
            c,
            if selected < answer {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("puzzle choice failed");
}

fn battle(c: &mut VisibleShellController, losing: bool) {
    battle_with_partner(c, losing, None);
}

fn battle_with_partner(
    c: &mut VisibleShellController,
    losing: bool,
    select: Option<fn(&str) -> Option<&'static str>>,
) {
    let before = c.snapshot().expect("before battle");
    let mut started = false;
    let mut attacks = 0;
    for _ in 0..4096 {
        let s = c.snapshot().expect("native gym battle");
        let Some(b) = s.battle.as_ref() else {
            if started {
                break;
            }
            press(c, GameButton::A);
            continue;
        };
        if !started
            && let crystal_runtime::RuntimeBattleKind::Trainer {
                trainer_id,
                trainer_class,
                trainer_name,
                ..
            } = &b.kind
        {
            let expected = match trainer_id.as_str() {
                "MN_GYM_1" => Some(("COOLTRAINERF", "VAREL")),
                "MN_GYM_1_TRAINER_1" => Some(("SCHOOLBOY", "TAVEL")),
                "MN_GYM_1_TRAINER_2" => Some(("SCHOOLBOY", "ELVIK")),
                "MN_GYM_2" => Some(("HIKER", "TORVEN")),
                "MN_GYM_2_TRAINER_1" => Some(("CAMPER", "CALDREN")),
                "MN_GYM_2_TRAINER_2" => Some(("PICNICKER", "JOREL")),
                "MN_GYM_3" => Some(("COOLTRAINERF", "AVRIN")),
                "MN_GYM_3_TRAINER_1" => Some(("SCIENTIST", "LENVI")),
                "MN_GYM_3_TRAINER_2" => Some(("SUPER_NERD", "JOSSA")),
                "MN_GYM_4" => Some(("GENTLEMAN", "BRANNIC")),
                "MN_GYM_4_TRAINER_1" => Some(("COOLTRAINERF", "BELVA")),
                "MN_GYM_4_TRAINER_2" => Some(("POKEFANM", "ORVEN")),
                "MN_GYM_5" => Some(("BIRD_KEEPER", "SAVEN")),
                "MN_GYM_5_TRAINER_1" => Some(("COOLTRAINERF", "WYREL")),
                "MN_GYM_5_TRAINER_2" => Some(("BIRD_KEEPER", "HALLEN")),
                "MN_GYM_6" => Some(("COOLTRAINERF", "SIVREN")),
                "MN_GYM_6_TRAINER_1" => Some(("SWIMMERF", "LORLA")),
                "MN_GYM_6_TRAINER_2" => Some(("FISHER", "BRAVEL")),
                "MN_GYM_7" => Some(("COOLTRAINERM", "ORLIS")),
                "MN_GYM_7_TRAINER_1" => Some(("COOLTRAINERM", "MOSLEN")),
                "MN_GYM_7_TRAINER_2" => Some(("COOLTRAINERF", "ELVREN")),
                "MN_GYM_7_TRAINER_3" => Some(("GENTLEMAN", "BRINDEL")),
                "MN_GYM_8" => Some(("SAILOR", "NOREK")),
                "MN_GYM_8_TRAINER_1" => Some(("COOLTRAINERF", "LIVRA")),
                "MN_GYM_8_TRAINER_2" => Some(("SAILOR", "TORDEL")),
                "MN_GYM_8_TRAINER_3" => Some(("COOLTRAINERM", "ANVER")),
                _ => None,
            };
            if let Some((class, name)) = expected {
                assert_eq!(trainer_class, class);
                assert!(
                    trainer_name.contains(name),
                    "battle must expose the original character name"
                );
            }
        }
        started = true;
        let opponent_is_dark = [
            &b.enemy_pokemon.species.type1,
            &b.enemy_pokemon.species.type2,
        ]
        .iter()
        .any(|kind| kind.trim_end_matches("_TYPE") == "DARK");
        let preferred = select
            .and_then(|choose| choose(&b.enemy_pokemon.species.id))
            .and_then(|species| {
                s.party.slots.iter().position(|p| {
                    p.pokemon.hp > 0
                        && p.pokemon.species.id == species
                        && Some(p.index) != b.active_player_party_index
                })
            });
        let input = if s.ui.menu.is_none() && s.ui.text.is_some() {
            c.wait_frames(30).expect("battle narration");
            GameButton::A
        } else if s.ui.menu.as_ref().is_some_and(|m| {
            m.layout
                .vertical_menus
                .iter()
                .flat_map(|v| &v.options)
                .any(|s| s.contains("change <PKMN>"))
        }) {
            GameButton::B
        } else if let Some(cursor) = &c.shell.battle_faint_prompt_cursor {
            if cursor.option_index > 0 {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_switch_cursor {
            let target = preferred
                .or_else(|| {
                    s.party.slots.iter().position(|p| {
                        p.pokemon.hp > 0 && Some(p.index) != b.active_player_party_index
                    })
                })
                .expect("living partner for a native switch");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_party_action_cursor {
            if cursor.option_index > 0 {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let name = if losing {
                "SPLASH"
            } else if matches!(
                b.enemy_pokemon.species.id.as_str(),
                "SEEL" | "DEWGONG" | "LAPRAS"
            ) {
                "THUNDERBOLT"
            } else if matches!(
                b.enemy_pokemon.species.id.as_str(),
                "MAGNEMITE" | "SKARMORY" | "SNEASEL" | "PINECO" | "SWINUB" | "PILOSWINE"
            ) || opponent_is_dark
            {
                "FLAMETHROWER"
            } else {
                "PSYCHIC_M"
            };
            let target = [
                name,
                "SURF",
                "PSYCHIC_M",
                "FLAMETHROWER",
                "THUNDERBOLT",
                "ICE_BEAM",
                "THUNDERPUNCH",
                "THUNDERSHOCK",
                if b.enemy_pokemon.species.type1 == "FLYING"
                    || b.enemy_pokemon.species.type2 == "FLYING"
                {
                    "ROCK_THROW"
                } else {
                    ""
                },
                "MAGNITUDE",
                "RAZOR_LEAF",
                "VINE_WHIP",
                "WATER_GUN",
                "FLAME_WHEEL",
                "EMBER",
                "CONFUSION",
                "HEADBUTT",
                "WING_ATTACK",
                "PECK",
                "GUST",
                "BITE",
                "SCRATCH",
                "TACKLE",
                "QUICK_ATTACK",
            ]
            .into_iter()
            .find_map(|choice| {
                if choice == "PSYCHIC_M" && opponent_is_dark {
                    return None;
                }
                b.player_moves
                    .iter()
                    .position(|m| m.name == choice && m.current_pp > 0)
            })
            .expect("selected legal move with PP");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                attacks += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            match (preferred.is_some(), cursor.option_index) {
                (true, 0) => GameButton::Right,
                (true, 1) | (false, 0) => GameButton::A,
                (false, 1) => GameButton::Left,
                (_, 2 | 3) => GameButton::Up,
                _ => panic!("battle cursor"),
            }
        } else {
            GameButton::A
        };
        press(c, input);
    }
    assert!(started && attacks > 0, "must actually play the battle");
    let after = c.snapshot().expect("battle result");
    assert!(after.battle.is_none());
    if !losing {
        let total =
            |moves: &[LearnedMove]| moves.iter().map(|m| u32::from(m.current_pp)).sum::<u32>();
        assert!(
            after
                .party
                .slots
                .iter()
                .map(|p| total(&p.pokemon.moves))
                .sum::<u32>()
                < before
                    .party
                    .slots
                    .iter()
                    .map(|p| total(&p.pokemon.moves))
                    .sum::<u32>(),
            "chosen attacks spend native PP"
        );
        assert!(
            after.party.slots.iter().any(|p| p.pokemon.hp > 0),
            "expected a win with legal selected attacks; party: {:?}",
            after.party
        );
    }
}

fn continue_save(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
) -> VisibleShellController {
    let before = c.snapshot().expect("before save");
    let badges = c.shell.shell.session().state().badges.clone();
    let overrides = c.shell.shell.session().state().map_block_overrides.clone();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("nonce")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("gym-{}-{nonce}.crystalsave", std::process::id()));
    c.save(&path).expect("native save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("Continue gym");
    let after = loaded.snapshot().expect("loaded snapshot");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(badges, loaded.shell.shell.session().state().badges);
    assert_eq!(
        overrides,
        loaded.shell.shell.session().state().map_block_overrides
    );
    std::fs::remove_file(path).expect("remove owned scratch save");
    loaded
}

fn solve_puzzle(c: &mut VisibleShellController, runtime: &CrystalRuntime, gym: &Value) {
    for step in gym["walkthrough"].as_array().expect("native walkthrough") {
        let step = step.as_str().expect("step");
        if let Some(t) = gym["trainers"]
            .as_array()
            .expect("trainers")
            .iter()
            .find(|t| t["id"] == step)
        {
            interact(c, runtime, point(&t["at"]));
            battle(c, false);
            finish(c);
            assert!(event(c, &format!("EVENT_{step}_DEFEATED")));
        } else if let Some(gate) = gym
            .get("cut_gates")
            .and_then(Value::as_array)
            .and_then(|gates| gates.iter().find(|gate| gate["id"] == step))
        {
            cut_gym_tree(c, runtime, gate);
        } else if let Some(pad) = gym
            .get("pads")
            .and_then(Value::as_array)
            .and_then(|pads| pads.iter().find(|p| p["id"] == step))
        {
            step_on_pad(c, runtime, pad);
        } else if let Some((router, index)) =
            gym.get("routers")
                .and_then(Value::as_array)
                .and_then(|routers| {
                    routers.iter().find_map(|router| {
                        router["modes"]
                            .as_array()
                            .expect("routing modes")
                            .iter()
                            .position(|mode| mode["id"] == step)
                            .map(|index| (router, index))
                    })
                })
        {
            route_power(c, runtime, router, index);
            assert!(event(
                c,
                router["modes"][index]["event"]
                    .as_str()
                    .expect("route event")
            ));
        } else if let Some(index) = gym.get("switch_puzzle").and_then(|puzzle| {
            puzzle["sequence"]
                .as_array()
                .expect("switch pair")
                .iter()
                .position(|id| id == step)
        }) {
            touch_switch(c, runtime, gym, step);
            assert!(event(
                c,
                gym["switch_puzzle"]["events"][index]
                    .as_str()
                    .expect("switch event")
            ));
        } else {
            let station = gym["stations"]
                .as_array()
                .expect("stations")
                .iter()
                .find(|s| s["id"] == step)
                .expect("station");
            let flag = station["event"].as_str().expect("station event");
            let answer = station["answer"].as_u64().expect("answer") as usize - 1;
            interact(c, runtime, point(&station["at"]));
            await_choice(c);
            press(c, GameButton::B);
            finish(c);
            assert!(!event(c, flag), "cancel must not advance");
            interact(c, runtime, point(&station["at"]));
            choose(
                c,
                (answer + 1) % station["options"].as_array().expect("choices").len(),
            );
            finish(c);
            assert!(!event(c, flag), "wrong choice must not advance");
            interact(c, runtime, point(&station["at"]));
            choose(c, answer);
            finish(c);
            assert!(event(c, flag));
        }
    }
}

fn route_power(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    router: &Value,
    index: usize,
) {
    interact(c, runtime, point(&router["at"]));
    choose(c, index);
    finish(c);
}

fn switch_position(gym: &Value, id: &str) -> Point {
    point(
        &gym["switch_puzzle"]["controls"]
            .as_array()
            .expect("switch controls")
            .iter()
            .find(|control| control["id"] == id)
            .expect("authored switch")["at"],
    )
}

fn touch_switch(c: &mut VisibleShellController, runtime: &CrystalRuntime, gym: &Value, id: &str) {
    interact(c, runtime, switch_position(gym, id));
    assert!(
        c.snapshot().expect("switch text").ui.menu.is_none(),
        "switches have no quiz menu"
    );
    finish(c);
}

fn assert_switches(c: &mut VisibleShellController, first: bool, second: bool) {
    assert_eq!(event(c, "EVENT_MN_GYM_3_FIRST_SWITCH_ON"), first);
    assert_eq!(event(c, "EVENT_MN_GYM_3_FIRST_GATE_OPEN"), first);
    assert_eq!(event(c, "EVENT_MN_GYM_3_SECOND_SWITCH_ON"), second);
    assert_eq!(event(c, "EVENT_MN_GYM_3_COURT_OPEN"), second);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_minneapolis_controller_switch_pair_requires_both_trainers_and_persists() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][2];
    let mut c = controller(&root, &runtime, gym, false);
    let lenvi = point(&gym["trainers"][0]["at"]);
    let jossa = point(&gym["trainers"][1]["at"]);
    let avrin = point(&gym["leader_at"]);
    assert!(!can_approach(&mut c, &runtime, avrin));
    touch_switch(&mut c, &runtime, gym, "UPPER_LEFT");
    assert_switches(&mut c, false, false);
    interact(&mut c, &runtime, jossa);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_3_TRAINER_2_DEFEATED"));
    interact(&mut c, &runtime, lenvi);
    battle(&mut c, false);
    finish(&mut c);
    c = continue_save(&mut c, &root, &runtime);
    touch_switch(&mut c, &runtime, gym, "UPPER_LEFT");
    assert_switches(&mut c, false, false);
    interact(&mut c, &runtime, jossa);
    battle(&mut c, false);
    finish(&mut c);
    c = continue_save(&mut c, &root, &runtime);
    touch_switch(&mut c, &runtime, gym, "UPPER_MIDDLE");
    assert_switches(&mut c, false, false); // Second switch cannot go first.
    interact(&mut c, &runtime, switch_position(gym, "UPPER_LEFT"));
    let before = at(&mut c);
    press(&mut c, GameButton::Right);
    assert_eq!(at(&mut c), before, "switch dialogue owns input");
    finish(&mut c);
    assert_switches(&mut c, true, false);
    walk_to(&mut c, &runtime, (4, 6));
    for _ in 0..3 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(
        at(&mut c),
        (4, 6),
        "second barrier remains physically closed"
    );
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(at(&mut c), (4, 6));
    assert_switches(&mut c, true, false);
    for carpet in gym["exit_tiles_core"].as_array().expect("both exits") {
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert_switches(&mut c, true, false);
    }
    touch_switch(&mut c, &runtime, gym, "UPPER_RIGHT");
    assert_switches(&mut c, false, false);
    walk_to(&mut c, &runtime, (4, 8));
    for _ in 0..3 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(
        at(&mut c),
        (4, 8),
        "a wrong choice physically restores the first barrier"
    );
    assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_2_DEFEATED"));
    touch_switch(&mut c, &runtime, gym, "UPPER_LEFT");
    touch_switch(&mut c, &runtime, gym, "UPPER_MIDDLE");
    assert_switches(&mut c, true, true);
    assert!(can_approach(&mut c, &runtime, avrin));
    c = continue_save(&mut c, &root, &runtime);
    touch_switch(&mut c, &runtime, gym, "LOWER_RIGHT");
    assert_switches(&mut c, true, true); // A solved circuit stays solved.
    interact(&mut c, &runtime, avrin);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[2]);
    let tm = &runtime.data().items["TM_THUNDERPUNCH"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &runtime);
    let before = c.snapshot().expect("before repeat");
    interact(&mut c, &runtime, avrin);
    finish(&mut c);
    let after = c.snapshot().expect("after repeat");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    for carpet in gym["exit_tiles_core"]
        .as_array()
        .expect("post-victory exits")
    {
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert_switches(&mut c, true, true);
        assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    }
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_minneapolis_controller_loss_keeps_switches_locked() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][2];
    let mut c = controller(&root, &runtime, gym, true);
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, true);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_3_TRAINER_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_3_FIRST_SWITCH_ON"));
    assert!(!event(&c, "EVENT_MN_GYM_3_DEFEATED"));
    assert!(!c.shell.shell.session().state().badges.johto[2]);
    assert_switches(&mut c, false, false);
    assert_ne!(
        c.snapshot().expect("ordinary recovery").overworld.map_name,
        "MnMinneapolisGym"
    );
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_minneapolis_controller_second_badge_is_required_for_avrin() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][2];
    let mut c = controller(&root, &runtime, gym, false);
    c.shell.shell.session_mut().state_mut().badges.johto[1] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    solve_puzzle(&mut c, &runtime, gym);
    assert_switches(&mut c, true, true);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_3_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_3_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[2]);
    assert!(c.snapshot().expect("leader refusal").battle.is_none());
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_THUNDERPUNCH"]),
        0
    );
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_gyms_controller_earns_all_eight_native_puzzles_badges_tms_and_continue() {
    let (root, runtime, design) = load();
    for gym in design["gyms"].as_array().expect("eight gyms") {
        let mut c = controller(&root, &runtime, gym, false);
        let slot = gym["slot"].as_u64().expect("slot");
        eprintln!("playing gym {slot}: {}", gym["city"]);
        let defeated = format!("EVENT_MN_GYM_{slot}_DEFEATED");
        let claimed = format!("EVENT_MN_GYM_{slot}_TM_RECEIVED");
        assert!(!event(&c, &defeated));
        solve_puzzle(&mut c, &runtime, gym);
        assert!(event(&c, &format!("EVENT_MN_GYM_{slot}_COURT_OPEN")));
        c = continue_save(&mut c, &root, &runtime);
        interact(&mut c, &runtime, point(&gym["leader_at"]));
        battle(&mut c, false);
        finish(&mut c);
        assert!(event(&c, &defeated));
        assert!(event(&c, &claimed));
        let reward = gym["reward"].as_str().expect("TM reward");
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[reward]),
            1,
            "actual TM inventory must match its successful-delivery flag"
        );
        assert!(c.shell.shell.session().state().badges.johto[slot as usize - 1]);
        assert_eq!(
            c.shell.shell.session().state().badges.johto,
            std::array::from_fn(|index| index < slot as usize)
        );
        c = continue_save(&mut c, &root, &runtime);
        let before = c.snapshot().expect("before repeat");
        interact(&mut c, &runtime, point(&gym["leader_at"]));
        finish(&mut c);
        let after = c.snapshot().expect("after repeat");
        assert_eq!(before.bag, after.bag);
        assert_eq!(before.party, after.party);
    }
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_gyms_controller_loss_does_not_grant_trainer_victory_or_badge() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][0];
    let mut c = controller(&root, &runtime, gym, true);
    let t = &gym["trainers"][0];
    interact(&mut c, &runtime, point(&t["at"]));
    battle(&mut c, true);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_1_TRAINER_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    assert!(!c.shell.shell.session().state().badges.johto[0]);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_gyms_controller_both_carpets_return_to_review_hall_and_reenter() {
    let (root, runtime, design) = load();
    for (index, gym) in design["gyms"].as_array().expect("gyms").iter().enumerate() {
        for carpet in gym["exit_tiles_core"].as_array().expect("carpets") {
            let (x, y) = point(carpet);
            let mut c = controller(&root, &runtime, gym, false);
            let m = module(&mut c, &runtime);
            let from = at(&mut c);
            for button in path(&runtime, &m, from, (x, y - 1)).expect("free carpet approach") {
                let before = at(&mut c);
                for _ in 0..8 {
                    press(&mut c, button);
                    if at(&mut c) != before {
                        break;
                    }
                }
            }
            assert_eq!(at(&mut c), (x, y - 1));
            for _ in 0..4 {
                press(&mut c, GameButton::Down);
                if c.snapshot().expect("exit").overworld.map_name == "MnGymReviewHall" {
                    break;
                }
            }
            assert_eq!(
                c.snapshot().expect("actual hall return").overworld.map_name,
                "MnGymReviewHall"
            );
            let door_x = 4 + index as u16 % 4 * 8;
            let door_y = if index < 4 { 2 } else { 14 };
            assert_eq!(at(&mut c), (door_x, door_y + 3));
            for _ in 0..6 {
                press(&mut c, GameButton::Up);
                if c.snapshot().expect("reenter").overworld.map_name
                    == gym["id"].as_str().expect("gym map")
                {
                    break;
                }
            }
            assert_eq!(
                c.snapshot().expect("actual gym arrival").overworld.map_name,
                gym["id"].as_str().expect("gym map")
            );
            assert_eq!(at(&mut c), point(&gym["arrival_core"]));
            assert!(!event(&c, &format!("EVENT_MN_GYM_{}_DEFEATED", index + 1)));
        }
    }
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_gyms_controller_open_court_refuses_unearned_challenge() {
    let (root, runtime, design) = load();
    for index in [0, 1, 6] {
        let gym = &design["gyms"][index];
        let mut c = controller(&root, &runtime, gym, false);
        let before = c.snapshot().expect("unearned challenge");
        let leader = point(&gym["leader_at"]);
        if index == 0 {
            // The planter maze has an open court, so Varel must enforce the
            // required practice wins even when the player walks around them.
            assert!(can_approach(&mut c, &runtime, leader));
            interact(&mut c, &runtime, leader);
            finish(&mut c);
        } else {
            // The remade rock stairs and Cut garden physically seal their
            // courts. The former quiz-room test must not assume open floors.
            assert!(!can_approach(&mut c, &runtime, leader));
        }
        let after = c.snapshot().expect("refused challenge");
        assert_eq!(before.party, after.party);
        assert_eq!(before.bag, after.bag);
        assert!(!event(&c, &format!("EVENT_MN_GYM_{}_DEFEATED", index + 1)));
        assert!(!c.shell.shell.session().state().badges.johto[index]);
    }
}

fn qualified_rochester_fixture(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    gym: &Value,
) {
    // Explicit completed-practice setup, used only to isolate leader refusal/loss.
    for flag in [
        "EVENT_MN_GYM_1_TRAINER_1_DEFEATED",
        "EVENT_MN_GYM_1_TRAINER_2_DEFEATED",
        "EVENT_MN_GYM_1_COURT_OPEN",
    ] {
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(flag, true)
            .expect("completed-practice fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    exit_and_reenter(c, runtime, gym, point(&gym["exit_tiles_core"][0]));
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_gyms_controller_parcel_refusal_and_leader_loss_grant_nothing() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][0];
    let mut c = controller(&root, &runtime, gym, false);
    qualified_rochester_fixture(&mut c, &runtime, gym);
    c = continue_save(&mut c, &root, &runtime);
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag("EVENT_MN_DELIVERED_ROCHESTER_PARCEL", false)
        .expect("undelivered parcel fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    assert!(!c.shell.shell.session().state().badges.johto[0]);

    let mut c = controller(&root, &runtime, gym, true);
    qualified_rochester_fixture(&mut c, &runtime, gym);
    c = continue_save(&mut c, &root, &runtime);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    battle(&mut c, true);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_1_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[0]);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_gyms_controller_full_tm_stack_can_be_used_then_claimed_once() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][0];
    let mut c = controller(&root, &runtime, gym, false);
    let tm = &runtime.data().items["TM_HEADBUTT"];
    {
        // Full-stack and free move-slot fixture, before any gameplay inputs.
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.bag.add_item(tm, 99).expect("full TM stack fixture"));
        state.storage.party.pokemon[0]
            .as_mut()
            .expect("fixture partner")
            .moves
            .truncate(3);
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    solve_puzzle(&mut c, &runtime, gym);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_1_TM_RECEIVED"));
    assert!(c.shell.shell.session().state().badges.johto[0]);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 99);
    c = continue_save(&mut c, &root, &runtime);

    press(&mut c, GameButton::Start);
    let entries = super::super::visible_start_menu_entries(&c.shell).expect("native Start entries");
    let target = entries
        .iter()
        .position(|s| s.trim_start_matches('>').trim() == "PACK")
        .expect("production PACK option");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            break;
        }
        press(
            &mut c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(&mut c, GameButton::A);
    for _ in 0..4 {
        if c.shell.tmhm_cursor.is_some() {
            break;
        }
        press(&mut c, GameButton::Right);
    }
    assert!(c.shell.tmhm_cursor.is_some());
    for _ in 0..128 {
        if c.shell.shell.session().state().bag.quantity(tm) == 98 {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 98);
    assert!(
        c.snapshot().expect("learned move").party.slots[0]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "HEADBUTT")
    );
    for _ in 0..128 {
        if !super::super::has_visible_shell_direction_action(&mut c.shell) {
            break;
        }
        let text = c.shell.pending_tmhm_text_stage.is_some() || c.shell.field_notice.is_some();
        press(&mut c, if text { GameButton::A } else { GameButton::B });
    }
    assert!(!super::super::has_visible_shell_direction_action(
        &mut c.shell
    ));
    let before = c.snapshot().expect("before pending reward");
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_TM_RECEIVED"));
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 99);
    assert_eq!(
        before.party,
        c.snapshot().expect("no repeated battle").party
    );
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 99);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_st_paul_controller_accepts_reverse_inspection_order() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][3];
    let mut c = controller(&root, &runtime, gym, false);
    interact(&mut c, &runtime, point(&gym["trainers"][1]["at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    assert!(!can_approach(&mut c, &runtime, point(&gym["leader_at"])));
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    assert!(can_approach(&mut c, &runtime, point(&gym["leader_at"])));
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_duluth_controller_slides_on_empty_frames_turns_at_stops_and_saves() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][7];
    let mut c = controller(&root, &runtime, gym, false);
    slide_gym_ice(
        &mut c,
        &runtime,
        (2, 16),
        (2, 8),
        GameButton::Up,
        GameButton::Down,
    );
    assert_eq!(gym_collision(&mut c, &runtime, (2, 8)), "ICE");
    c = continue_save(&mut c, &root, &runtime);
    slide_gym_ice(
        &mut c,
        &runtime,
        (2, 8),
        (4, 8),
        GameButton::Right,
        GameButton::Left,
    );
    assert_eq!(gym_collision(&mut c, &runtime, (4, 8)), "FLOOR");
    c = continue_save(&mut c, &root, &runtime);
    walk_to(&mut c, &runtime, (4, 7));
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, false);
    finish(&mut c);
    slide_gym_ice(
        &mut c,
        &runtime,
        (11, 16),
        (11, 10),
        GameButton::Up,
        GameButton::Down,
    );
    assert_eq!(gym_collision(&mut c, &runtime, (11, 10)), "ICE");
    c = continue_save(&mut c, &root, &runtime);
    slide_gym_ice(
        &mut c,
        &runtime,
        (11, 10),
        (9, 10),
        GameButton::Left,
        GameButton::Right,
    );
    assert_eq!(gym_collision(&mut c, &runtime, (9, 10)), "FLOOR");
    c = continue_save(&mut c, &root, &runtime);
    walk_to(&mut c, &runtime, (9, 7));
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_minneapolis_controller_all_wrong_switches_reset_only_the_circuit() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][2];
    let mut c = controller(&root, &runtime, gym, false);
    for trainer in gym["trainers"].as_array().expect("trainers") {
        interact(&mut c, &runtime, point(&trainer["at"]));
        battle(&mut c, false);
        finish(&mut c);
    }
    for wrong in [
        "UPPER_LEFT",
        "UPPER_RIGHT",
        "LOWER_LEFT",
        "LOWER_MIDDLE",
        "LOWER_RIGHT",
    ] {
        touch_switch(&mut c, &runtime, gym, "UPPER_LEFT");
        assert_switches(&mut c, true, false);
        touch_switch(&mut c, &runtime, gym, wrong);
        assert_switches(&mut c, false, false);
        assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_1_DEFEATED"));
        assert!(event(&c, "EVENT_MN_GYM_3_TRAINER_2_DEFEATED"));
        assert!(!c.shell.shell.session().state().badges.johto[2]);
    }
    c = continue_save(&mut c, &root, &runtime);
    touch_switch(&mut c, &runtime, gym, "UPPER_LEFT");
    touch_switch(&mut c, &runtime, gym, "UPPER_MIDDLE");
    for control in gym["switch_puzzle"]["controls"]
        .as_array()
        .expect("all cans")
    {
        touch_switch(
            &mut c,
            &runtime,
            gym,
            control["id"].as_str().expect("can id"),
        );
        assert_switches(&mut c, true, true);
    }
}

fn can_approach(c: &mut VisibleShellController, runtime: &CrystalRuntime, target: Point) -> bool {
    can_approach_with_ledges(c, runtime, target, true)
}

fn can_approach_with_ledges(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    target: Point,
    allow_ledges: bool,
) -> bool {
    let m = module(c, runtime);
    let from = at(c);
    DIRECTIONS.iter().any(|(dx, dy, _)| {
        let x = target.0 as i16 - dx;
        let y = target.1 as i16 - dy;
        x >= 0
            && y >= 0
            && (x as u16) < m.attributes.width * 2
            && (y as u16) < m.attributes.height * 2
            && path_with_ledges(runtime, &m, from, (x as u16, y as u16), allow_ledges).is_some()
    })
}

pub(super) fn walk_to(c: &mut VisibleShellController, runtime: &CrystalRuntime, target: Point) {
    let m = module(c, runtime);
    let from = at(c);
    for button in path(runtime, &m, from, target)
        .unwrap_or_else(|| panic!("native walking route on {}: {from:?} -> {target:?}", m.id))
    {
        let before = at(c);
        for _ in 0..8 {
            press(c, button);
            if at(c) != before {
                break;
            }
        }
        // Ice continues across released input frames. A changed tile alone
        // does not establish that the next planned direction owns input.
        c.wait_frames(80)
            .expect("native ice slide settles before the next direction");
    }
    assert_eq!(at(c), target);
}

fn ride_lift(c: &mut VisibleShellController, runtime: &CrystalRuntime, lift: &Value, side: usize) {
    let source = point(&lift["ends_core"][side]);
    let destination = point(&lift["ends_core"][1 - side]);
    let room = module(c, runtime);
    let from = at(c);
    let mut approaches = Vec::new();
    for (dx, dy, button) in DIRECTIONS {
        let x = source.0 as i16 - dx;
        let y = source.1 as i16 - dy;
        if x >= 0
            && y >= 0
            && x < room.attributes.width as i16 * 2
            && y < room.attributes.height as i16 * 2
            && let Some(route) = path(runtime, &room, from, (x as u16, y as u16))
        {
            approaches.push((route.len(), (x as u16, y as u16), button));
        }
    }
    approaches.sort_by_key(|(length, _, _)| *length);
    let (_, stand, button) = approaches.into_iter().next().expect("safe lift approach");
    walk_to(c, runtime, stand);
    for _ in 0..4 {
        press(c, button);
        if at(c) != stand {
            break;
        }
    }
    c.wait_frames(160).expect("settle native same-map lift");
    assert_eq!(at(c), destination, "native pad reaches its paired endpoint");
    c.wait_frames(160)
        .expect("arriving on a lift must not bounce back");
    assert_eq!(at(c), destination);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_st_paul_controller_lifts_require_both_trainers_and_persist_on_pads() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][3];
    let mut c = controller(&root, &runtime, gym, false);
    let brannic = point(&gym["leader_at"]);
    assert!(can_approach(
        &mut c,
        &runtime,
        point(&gym["trainers"][0]["at"])
    ));
    assert!(can_approach(
        &mut c,
        &runtime,
        point(&gym["trainers"][1]["at"])
    ));
    assert!(!can_approach(&mut c, &runtime, brannic));
    // The middle pad can be tried early; the far grille blocks the leader,
    // and the same pad always gives a safe way back.
    ride_lift(&mut c, &runtime, &gym["lifts"][3], 0);
    c = continue_save(&mut c, &root, &runtime);
    walk_to(&mut c, &runtime, (11, 6));
    for _ in 0..3 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(
        at(&mut c),
        (11, 6),
        "the unearned court grille really blocks movement"
    );
    ride_lift(&mut c, &runtime, &gym["lifts"][3], 1);
    ride_lift(&mut c, &runtime, &gym["lifts"][0], 0);
    c = continue_save(&mut c, &root, &runtime);
    c.wait_frames(120)
        .expect("Continue stays on the arrival pad");
    assert_eq!(at(&mut c), point(&gym["lifts"][0]["ends_core"][1]));
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    assert!(!can_approach(&mut c, &runtime, brannic));
    ride_lift(&mut c, &runtime, &gym["lifts"][0], 1);
    c = continue_save(&mut c, &root, &runtime);
    ride_lift(&mut c, &runtime, &gym["lifts"][0], 0);
    ride_lift(&mut c, &runtime, &gym["lifts"][1], 0);
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(at(&mut c), point(&gym["lifts"][1]["ends_core"][1]));
    interact(&mut c, &runtime, point(&gym["trainers"][1]["at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    ride_lift(&mut c, &runtime, &gym["lifts"][1], 1);
    ride_lift(&mut c, &runtime, &gym["lifts"][1], 0);
    ride_lift(&mut c, &runtime, &gym["lifts"][2], 0);
    c = continue_save(&mut c, &root, &runtime);
    ride_lift(&mut c, &runtime, &gym["lifts"][2], 1);
    ride_lift(&mut c, &runtime, &gym["lifts"][2], 0);
    ride_lift(&mut c, &runtime, &gym["lifts"][3], 0);
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(at(&mut c), point(&gym["lifts"][3]["ends_core"][1]));
    interact(&mut c, &runtime, brannic);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_DEFEATED"));
    assert!(c.shell.shell.session().state().badges.johto[3]);
    let tm = &runtime.data().items["TM_STEEL_WING"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &runtime);
    let before = c.snapshot().expect("earned badge");
    interact(&mut c, &runtime, brannic);
    finish(&mut c);
    let after = c.snapshot().expect("repeat leader");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    ride_lift(&mut c, &runtime, &gym["lifts"][3], 1);
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert!(event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
        assert!(c.shell.shell.session().state().badges.johto[3]);
        assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
        assert!(can_approach(&mut c, &runtime, brannic));
    }
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_st_paul_controller_loss_does_not_release_main_lift() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][3];
    let mut c = controller(&root, &runtime, gym, true);
    ride_lift(&mut c, &runtime, &gym["lifts"][0], 0);
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, true);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_4_TRAINER_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    assert!(!c.shell.shell.session().state().badges.johto[3]);
    assert_ne!(
        c.snapshot().expect("ordinary recovery").overworld.map_name,
        "MnStPaulGym"
    );
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_st_paul_controller_third_badge_is_required_for_brannic() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][3];
    let mut c = controller(&root, &runtime, gym, false);
    c.shell.shell.session_mut().state_mut().badges.johto[2] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    solve_puzzle(&mut c, &runtime, gym);
    assert!(event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_4_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_4_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[3]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_STEEL_WING"]),
        0
    );
}

fn take_drop(c: &mut VisibleShellController, runtime: &CrystalRuntime, ledge: &Value) {
    let from = point(&ledge["hop_from"]);
    let landing = point(&ledge["landing"]);
    walk_to(c, runtime, from);
    assert_eq!(ledge["direction"], "south");
    for _ in 0..4 {
        press(c, GameButton::Down);
        if at(c) != from {
            break;
        }
    }
    c.wait_frames(80).expect("finish ledge jump");
    assert_eq!(at(c), landing, "real native two-tile hop");
    for _ in 0..3 {
        press(c, GameButton::Up);
    }
    assert_eq!(at(c), landing, "bank blocks the reverse approach");
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_mankato_controller_terraces_require_drop_trainers_and_persist() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][1];
    let mut c = controller(&root, &runtime, gym, false);
    let caldren = point(&gym["trainers"][0]["at"]);
    let jorel = point(&gym["trainers"][1]["at"]);
    let torven = point(&gym["leader_at"]);
    assert!(can_approach(&mut c, &runtime, caldren));
    assert!(!can_approach(&mut c, &runtime, jorel));
    assert!(!can_approach(&mut c, &runtime, torven));
    interact(&mut c, &runtime, caldren);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_2_LOWER_STAIR_OPEN"));
    assert!(can_approach(&mut c, &runtime, jorel));
    assert!(!can_approach_with_ledges(&mut c, &runtime, jorel, false));
    assert!(!can_approach(&mut c, &runtime, torven));
    c = continue_save(&mut c, &root, &runtime);
    take_drop(&mut c, &runtime, &gym["ledges"][0]);
    c = continue_save(&mut c, &root, &runtime);
    assert!(can_approach(&mut c, &runtime, jorel));
    assert!(!can_approach(&mut c, &runtime, torven));
    // Escape is available before fighting Jorel, without granting her victory.
    take_drop(&mut c, &runtime, &gym["ledges"][2]);
    assert!(!event(&c, "EVENT_MN_GYM_2_TRAINER_2_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_2_COURT_OPEN"));
    walk_to(&mut c, &runtime, point(&gym["arrival_core"]));
    c = continue_save(&mut c, &root, &runtime);
    take_drop(&mut c, &runtime, &gym["ledges"][0]);
    interact(&mut c, &runtime, jorel);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_2_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_2_COURT_OPEN"));
    c = continue_save(&mut c, &root, &runtime);
    interact(&mut c, &runtime, torven);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_2_DEFEATED"));
    assert!(c.shell.shell.session().state().badges.johto[1]);
    let tm = &runtime.data().items["TM_MUD_SLAP"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    let pp = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("partner")
        .moves
        .clone();
    interact(&mut c, &runtime, torven);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    assert_eq!(
        c.shell.shell.session().state().storage.party.pokemon[0]
            .as_ref()
            .expect("partner")
            .moves,
        pp
    );
    let from = at(&mut c);
    let open_court = module(&mut c, &runtime);
    let mut stairs_only = open_court.clone();
    let drop = point(&gym["ledges"][1]["hop_from"]);
    let gate = point(&gym["gates"][1]["at"]);
    let closed = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(gym["id"].as_str().expect("gym map"))
        .expect("initial closed room");
    stairs_only.blocks[usize::from(drop.1 / 2) * usize::from(stairs_only.attributes.width)
        + usize::from(drop.0 / 2)] = closed.blocks
        [usize::from(gate.1) * usize::from(closed.attributes.width) + usize::from(gate.0)];
    let carpet = point(&gym["exit_tiles_core"][0]);
    let exit_approach = (carpet.0, carpet.1 - 1);
    assert!(
        path(&runtime, &open_court, from, exit_approach)
            .expect("drop return")
            .len()
            < path(&runtime, &stairs_only, from, exit_approach)
                .expect("stair return")
                .len(),
        "court drop shortens the real route to the entrance"
    );
    // Both upper and lower drops execute natively and block the reverse.
    take_drop(&mut c, &runtime, &gym["ledges"][1]);
    take_drop(&mut c, &runtime, &gym["ledges"][2]);
    let open = module(&mut c, &runtime);
    c = continue_save(&mut c, &root, &runtime);
    for carpet in gym["exit_tiles_core"].as_array().expect("both exits") {
        // Follow the compiled passive arrival and prove the matching re-entry.
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert_eq!(module(&mut c, &runtime).blocks, open.blocks);
        assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_1_DEFEATED"));
        assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_2_DEFEATED"));
        assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    }
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_mankato_controller_loss_keeps_the_ascent_closed() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][1];
    let mut c = controller(&root, &runtime, gym, true);
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, true);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_2_TRAINER_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_2_LOWER_STAIR_OPEN"));
    assert!(!c.shell.shell.session().state().badges.johto[1]);
    assert_ne!(
        c.snapshot().expect("ordinary recovery").overworld.map_name,
        "MnMankatoGym"
    );
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_mankato_controller_first_badge_is_required_for_torven() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][1];
    let mut c = controller(&root, &runtime, gym, false);
    // Missing prior badge is explicit initial setup, before any gameplay input.
    c.shell.shell.session_mut().state_mut().badges.johto[0] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    solve_puzzle(&mut c, &runtime, gym);
    assert!(event(&c, "EVENT_MN_GYM_2_COURT_OPEN"));
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_2_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_2_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[1]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_MUD_SLAP"]),
        0
    );
    assert!(c.snapshot().expect("leader refusal").battle.is_none());
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_rochester_controller_planter_maze_requires_both_trainers_and_persists() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][0];
    let mut c = controller(&root, &runtime, gym, false);
    let tavel = point(&gym["trainers"][0]["at"]);
    let elvik = point(&gym["trainers"][1]["at"]);
    let varel = point(&gym["leader_at"]);
    let terrain = module(&mut c, &runtime).blocks;
    // A static maze has an open court, but neither battle can be skipped.
    interact(&mut c, &runtime, elvik);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_1_TRAINER_2_DEFEATED"));
    interact(&mut c, &runtime, varel);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    assert!(!c.shell.shell.session().state().badges.johto[0]);
    interact(&mut c, &runtime, tavel);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_TRAINER_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_1_COURT_OPEN"));
    c = continue_save(&mut c, &root, &runtime);
    interact(&mut c, &runtime, varel);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    exit_and_reenter(&mut c, &runtime, gym, point(&gym["exit_tiles_core"][0]));
    interact(&mut c, &runtime, elvik);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_1_COURT_OPEN"));
    c = continue_save(&mut c, &root, &runtime);
    interact(&mut c, &runtime, varel);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[0]);
    let tm = &runtime.data().items["TM_HEADBUTT"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    interact(&mut c, &runtime, varel);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    assert_eq!(
        module(&mut c, &runtime).blocks,
        terrain,
        "planters stay planted"
    );
    for carpet in gym["exit_tiles_core"].as_array().expect("both exits") {
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert!(event(&c, "EVENT_MN_GYM_1_COURT_OPEN"));
        assert_eq!(module(&mut c, &runtime).blocks, terrain);
    }
    c = continue_save(&mut c, &root, &runtime);
    assert!(event(&c, "EVENT_MN_GYM_1_DEFEATED"));
}

fn exit_and_reenter(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    gym: &Value,
    (x, y): Point,
) {
    let m = module(c, runtime);
    // The same course may be bound to its city or the original review hall.
    // Read the actual native carpet/arrival records; every transition still
    // executes through press, including both city-bound exit carpets.
    let exit = m
        .events
        .warps
        .iter()
        .find(|warp| (warp.x, warp.y) == (x, y))
        .expect("real carpet warp");
    let outside_name = runtime
        .data()
        .map_name_for_constant(&exit.target_map_constant)
        .expect("actual gym exterior");
    let outside = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(&outside_name)
        .expect("native exterior module");
    let arrival = outside
        .events
        .warps
        .iter()
        .find(|warp| i16::try_from(warp.index).ok() == Some(exit.target_warp_id))
        .expect("passive exterior arrival");
    let expected = (arrival.x, arrival.y);
    let from = at(c);
    for button in path(runtime, &m, from, (x, y - 1)).expect("real exit approach") {
        let before = at(c);
        for _ in 0..8 {
            press(c, button);
            if at(c) != before {
                break;
            }
        }
        c.wait_frames(80).expect("finish native movement");
    }
    assert_eq!(at(c), (x, y - 1));
    for _ in 0..4 {
        press(c, GameButton::Down);
        if c.snapshot().expect("exit").overworld.map_name == outside_name {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("exterior").overworld.map_name,
        outside_name
    );
    assert_eq!(at(c), expected);
    for _ in 0..6 {
        press(c, GameButton::Up);
        if c.snapshot().expect("reentry").overworld.map_name == gym["id"].as_str().expect("map") {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("returned room").overworld.map_name,
        gym["id"].as_str().expect("map")
    );
    assert_eq!(at(c), point(&gym["arrival_core"]));
}

fn step_on_pad(c: &mut VisibleShellController, runtime: &CrystalRuntime, pad: &Value) {
    let m = module(c, runtime);
    let from = at(c);
    let mut routes = pad["tiles_core"]
        .as_array()
        .expect("pad tiles")
        .iter()
        .filter_map(|cell| {
            let at = point(cell);
            path(runtime, &m, from, at).map(|route| (route, at))
        })
        .collect::<Vec<_>>();
    routes.sort_by_key(|(route, _)| route.len());
    let (route, target) = routes.into_iter().next().expect("reachable pressure mat");
    for button in route {
        let before = at(c);
        for _ in 0..8 {
            press(c, button);
            if at(c) != before {
                break;
            }
        }
    }
    assert_eq!(at(c), target, "step onto the native mat trigger");
    finish(c);
    assert!(event(c, pad["event"].as_str().expect("pad event")));
}

fn cross_gust(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    from: Point,
    landing: Point,
    forward: GameButton,
    backward: GameButton,
) {
    walk_to(c, runtime, from);
    if landing == from {
        for _ in 0..3 {
            press(c, forward);
            c.wait_frames(80)
                .expect("opposing gust returns to its bank");
            assert_eq!(at(c), from);
        }
        return;
    }
    for _ in 0..4 {
        c.press(forward).expect("enter wind with production input");
        c.wait_frames(1).expect("wind entry frame");
        if at(c) != from {
            break;
        }
    }
    let entered = at(c);
    assert_ne!(entered, from);
    assert_ne!(entered, landing, "observe a live gust before its landing");
    c.press(backward)
        .expect("opposite input during forced wind");
    c.wait_frames(160).expect("empty frames carry the player");
    assert_eq!(at(c), landing, "wind owns direction until safe floor");
    c.wait_frames(80).expect("rest on the stone landing");
    assert_eq!(at(c), landing);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_st_cloud_controller_wind_lanes_require_both_trainers_and_persist() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][4];
    let mut c = controller(&root, &runtime, gym, false);
    let wyrel = point(&gym["trainers"][0]["at"]);
    let hallen = point(&gym["trainers"][1]["at"]);
    let saven = point(&gym["leader_at"]);
    assert!(can_approach(&mut c, &runtime, wyrel));
    assert!(!can_approach(&mut c, &runtime, hallen));
    assert!(!can_approach(&mut c, &runtime, saven));
    // Entering the return bridge against its wind returns to the same bank.
    cross_gust(
        &mut c,
        &runtime,
        (7, 16),
        (7, 16),
        GameButton::Right,
        GameButton::Right,
    );
    assert!(!event(&c, "EVENT_MN_GYM_5_TRAINER_1_DEFEATED"));
    interact(&mut c, &runtime, wyrel);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_5_LOWER_CROSSING_OPEN"));
    assert!(can_approach(&mut c, &runtime, hallen));
    assert!(!can_approach(&mut c, &runtime, saven));
    c = continue_save(&mut c, &root, &runtime);
    cross_gust(
        &mut c,
        &runtime,
        (7, 13),
        (12, 13),
        GameButton::Right,
        GameButton::Left,
    );
    c = continue_save(&mut c, &root, &runtime);
    // The upper return lane cannot bypass Hallen's locked forward lane.
    cross_gust(
        &mut c,
        &runtime,
        (12, 4),
        (12, 4),
        GameButton::Left,
        GameButton::Left,
    );
    assert!(!can_approach(&mut c, &runtime, saven));
    interact(&mut c, &runtime, hallen);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_5_COURT_OPEN"));
    c = continue_save(&mut c, &root, &runtime);
    cross_gust(
        &mut c,
        &runtime,
        (12, 9),
        (7, 9),
        GameButton::Left,
        GameButton::Right,
    );
    interact(&mut c, &runtime, saven);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_5_DEFEATED"));
    assert!(c.shell.shell.session().state().badges.johto[4]);
    let tm = &runtime.data().items["TM_SWIFT"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    let pp = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("partner")
        .moves
        .clone();
    interact(&mut c, &runtime, saven);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    assert_eq!(
        c.shell.shell.session().state().storage.party.pokemon[0]
            .as_ref()
            .expect("partner")
            .moves,
        pp
    );
    cross_gust(
        &mut c,
        &runtime,
        (7, 5),
        (12, 5),
        GameButton::Right,
        GameButton::Left,
    );
    cross_gust(
        &mut c,
        &runtime,
        (12, 17),
        (7, 17),
        GameButton::Left,
        GameButton::Right,
    );
    c = continue_save(&mut c, &root, &runtime);
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert!(event(&c, "EVENT_MN_GYM_5_TRAINER_1_DEFEATED"));
        assert!(event(&c, "EVENT_MN_GYM_5_TRAINER_2_DEFEATED"));
        assert!(event(&c, "EVENT_MN_GYM_5_COURT_OPEN"));
        assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    }
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_st_cloud_controller_loss_keeps_first_crossing_closed() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][4];
    let mut c = controller(&root, &runtime, gym, true);
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, true);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_5_TRAINER_1_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_5_LOWER_CROSSING_OPEN"));
    assert!(!event(&c, "EVENT_MN_GYM_5_COURT_OPEN"));
    assert!(!c.shell.shell.session().state().badges.johto[4]);
    assert_ne!(
        c.snapshot().expect("ordinary recovery").overworld.map_name,
        "MnStCloudGym"
    );
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_st_cloud_controller_fourth_badge_is_required_for_saven() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][4];
    let mut c = controller(&root, &runtime, gym, false);
    c.shell.shell.session_mut().state_mut().badges.johto[3] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    solve_puzzle(&mut c, &runtime, gym);
    assert!(event(&c, "EVENT_MN_GYM_5_COURT_OPEN"));
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_5_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_5_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[4]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_SWIFT"]),
        0
    );
}

fn gym_collision(c: &mut VisibleShellController, runtime: &CrystalRuntime, p: Point) -> String {
    let m = module(c, runtime);
    let block =
        m.blocks[usize::from(p.1 / 2) * usize::from(m.attributes.width) + usize::from(p.0 / 2)];
    runtime.data().tilesets[&m.attributes.tileset_name].collision[&format!("{block:02x}")]
        [usize::from(p.1 % 2 * 2 + p.0 % 2)]
    .clone()
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_brainerd_controller_shifts_native_piers_through_both_trainers() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][5];
    let mut c = controller(&root, &runtime, gym, false);
    let lorla = point(&gym["trainers"][0]["at"]);
    let bravel = point(&gym["trainers"][1]["at"]);
    let sivren = point(&gym["leader_at"]);
    let control = &gym["routers"][0];
    assert!(can_approach(&mut c, &runtime, lorla));
    assert!(!can_approach(&mut c, &runtime, bravel));
    assert!(!can_approach(&mut c, &runtime, sivren));
    // Original-style dry pool edging is bounded by the native map, not a warp.
    for (edge, button) in [((0, 14), GameButton::Left), ((9, 14), GameButton::Right)] {
        walk_to(&mut c, &runtime, edge);
        for _ in 0..3 {
            press(&mut c, button);
        }
        assert_eq!(at(&mut c), edge);
        assert_eq!(
            c.snapshot().expect("bounded pool room").overworld.map_name,
            "MnBrainerdGym"
        );
    }
    interact(&mut c, &runtime, lorla);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_6_TRAINER_1_DEFEATED"));
    assert!(
        !event(&c, "EVENT_MN_GYM_6_WALK_CENTER"),
        "winning is not a menu choice"
    );
    route_power(&mut c, &runtime, control, 1);
    assert!(event(&c, "EVENT_MN_GYM_6_WALK_LEFT"));
    assert!(!can_approach(&mut c, &runtime, bravel));
    assert!(!can_approach(&mut c, &runtime, sivren));
    // Repeat conversation opens the two choices; it never repeats Lorla's battle.
    interact(&mut c, &runtime, lorla);
    await_choice(&mut c);
    let stand = at(&mut c);
    press(&mut c, GameButton::Right);
    assert_eq!(at(&mut c), stand);
    press(&mut c, GameButton::B);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_6_WALK_LEFT"));
    route_power(&mut c, &runtime, control, 0);
    assert!(event(&c, "EVENT_MN_GYM_6_WALK_CENTER"));
    assert!(!event(&c, "EVENT_MN_GYM_6_WALK_LEFT"));
    assert_eq!(gym_collision(&mut c, &runtime, (5, 9)), "FLOOR");
    assert_eq!(gym_collision(&mut c, &runtime, (3, 9)), "WATER");
    assert!(can_approach(&mut c, &runtime, bravel));
    assert!(!can_approach(&mut c, &runtime, sivren));
    walk_to(&mut c, &runtime, (5, 9));
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(at(&mut c), (5, 9));
    exit_and_reenter(&mut c, &runtime, gym, point(&gym["exit_tiles_core"][0]));
    assert!(event(&c, "EVENT_MN_GYM_6_WALK_CENTER"));
    interact(&mut c, &runtime, bravel);
    battle(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_6_COURT_OPEN"));
    c = continue_save(&mut c, &root, &runtime);
    route_power(&mut c, &runtime, control, 1);
    assert!(event(&c, "EVENT_MN_GYM_6_COURT_OPEN"));
    assert_eq!(gym_collision(&mut c, &runtime, (5, 9)), "WATER");
    assert_eq!(gym_collision(&mut c, &runtime, (3, 9)), "FLOOR");
    assert!(can_approach(&mut c, &runtime, sivren));
    walk_to(&mut c, &runtime, (5, 10));
    for _ in 0..3 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(at(&mut c), (5, 10), "the old center pier has retracted");
    route_power(&mut c, &runtime, control, 0);
    assert!(!event(&c, "EVENT_MN_GYM_6_COURT_OPEN"));
    assert!(!can_approach(&mut c, &runtime, sivren));
    route_power(&mut c, &runtime, control, 1);
    walk_to(&mut c, &runtime, (3, 9));
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(at(&mut c), (3, 9));
    interact(&mut c, &runtime, sivren);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_6_DEFEATED"));
    assert!(c.shell.shell.session().state().badges.johto[5]);
    let tm = &runtime.data().items["TM_RAIN_DANCE"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    let pp = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("partner")
        .moves
        .clone();
    interact(&mut c, &runtime, sivren);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    assert_eq!(
        c.shell.shell.session().state().storage.party.pokemon[0]
            .as_ref()
            .expect("partner")
            .moves,
        pp
    );
    c = continue_save(&mut c, &root, &runtime);
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert!(event(&c, "EVENT_MN_GYM_6_COURT_OPEN"));
        assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    }
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_brainerd_controller_loss_grants_no_pier_control() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][5];
    let mut c = controller(&root, &runtime, gym, true);
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, true);
    finish(&mut c);
    for flag in [
        "EVENT_MN_GYM_6_TRAINER_1_DEFEATED",
        "EVENT_MN_GYM_6_WALK_CENTER",
        "EVENT_MN_GYM_6_COURT_OPEN",
    ] {
        assert!(!event(&c, flag));
    }
    assert!(!c.shell.shell.session().state().badges.johto[5]);
    assert_ne!(
        c.snapshot().expect("ordinary recovery").overworld.map_name,
        "MnBrainerdGym"
    );
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_brainerd_controller_fifth_badge_is_required_for_sivren() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][5];
    let mut c = controller(&root, &runtime, gym, false);
    c.shell.shell.session_mut().state_mut().badges.johto[4] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    solve_puzzle(&mut c, &runtime, gym);
    assert!(event(&c, "EVENT_MN_GYM_6_COURT_OPEN"));
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_6_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_6_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[5]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_RAIN_DANCE"]),
        0
    );
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_brainerd_controller_poolside_fixture_cannot_skip_the_trainers() {
    let (root, runtime, design) = load();
    let mut gym = design["gyms"][5].clone();
    // Explicit initial poolside placement models access from a later Surf visit;
    // it does not claim that this fixture earned Surf or traversed the region.
    gym["arrival_core"] = serde_json::json!([5, 4]);
    let mut c = controller(&root, &runtime, &gym, false);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_6_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_6_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[5]);
}

fn minneapolis_city_round_trip(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    gym: &Value,
    carpet: Point,
) {
    walk_to(c, runtime, (carpet.0, carpet.1 - 1));
    for _ in 0..8 {
        press(c, GameButton::Down);
        if c.snapshot().expect("city return").overworld.map_name == "MnMinneapolis" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("real city").overworld.map_name,
        "MnMinneapolis"
    );
    assert_eq!(at(c), (42, 24), "safe floor outside the actual gym facade");
    for _ in 0..8 {
        press(c, GameButton::Up);
        if c.snapshot().expect("facade entry").overworld.map_name == "MnMinneapolisGym" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("canonical gym").overworld.map_name,
        "MnMinneapolisGym"
    );
    assert_eq!(at(c), point(&gym["arrival_core"]));
}

#[test]
#[ignore = "requires the published Minneapolis city pack with the canonical switch gym"]
fn minnesota_minneapolis_city_controller_earns_both_switches_and_badge_through_the_facade() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][2];
    let mut c = controller(&root, &runtime, gym, false);
    // Only initial party, prior badges and location are seeded. Door travel and
    // all gym victories, switch states and rewards below use production input.
    minneapolis_city_round_trip(&mut c, &runtime, gym, point(&gym["exit_tiles_core"][0]));
    assert_switches(&mut c, false, false);
    for trainer in gym["trainers"].as_array().expect("trainers") {
        interact(&mut c, &runtime, point(&trainer["at"]));
        battle(&mut c, false);
        finish(&mut c);
    }
    touch_switch(&mut c, &runtime, gym, "UPPER_LEFT");
    walk_to(&mut c, &runtime, (4, 6));
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(at(&mut c), (4, 6));
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        minneapolis_city_round_trip(&mut c, &runtime, gym, point(carpet));
        assert_switches(&mut c, true, false);
    }
    touch_switch(&mut c, &runtime, gym, "UPPER_MIDDLE");
    assert_switches(&mut c, true, true);
    c = continue_save(&mut c, &root, &runtime);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[2]);
    let tm = &runtime.data().items["TM_THUNDERPUNCH"];
    for carpet in gym["exit_tiles_core"]
        .as_array()
        .expect("both earned returns")
    {
        minneapolis_city_round_trip(&mut c, &runtime, gym, point(carpet));
        assert_switches(&mut c, true, true);
        assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    }
    c = continue_save(&mut c, &root, &runtime);
    assert!(c.shell.shell.session().state().badges.johto[2]);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
}

fn cut_gym_tree(c: &mut VisibleShellController, runtime: &CrystalRuntime, gate: &Value) {
    let tree = point(&gate["tree_core"]);
    let approach = point(&gate["approach_core"]);
    walk_to(c, runtime, approach);
    assert_eq!(gym_collision(c, runtime, tree), "CUT_TREE");
    let direction = DIRECTIONS
        .iter()
        .find_map(|&(dx, dy, button)| {
            ((approach.0 as i16 + dx, approach.1 as i16 + dy) == (tree.0 as i16, tree.1 as i16))
                .then_some(button)
        })
        .expect("adjacent Cut approach");
    press(c, direction);
    assert_eq!(at(c), approach, "native tree blocks walking before Cut");
    for _ in 0..64 {
        if gym_collision(c, runtime, tree) == "FLOOR" {
            finish(c);
            c.wait_frames(80).expect("native Cut animation finishes");
            walk_to(c, runtime, tree);
            assert!(event(
                c,
                gate["event"].as_str().expect("pruned hedge event")
            ));
            return;
        }
        press(c, GameButton::A);
    }
    panic!("native Cut did not open the gym hedge");
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_bemidji_controller_cuts_both_hedges_earns_trainers_and_persists() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][6];
    let mut c = controller(&root, &runtime, gym, false);
    let moslen = point(&gym["trainers"][0]["at"]);
    let elvren = point(&gym["trainers"][1]["at"]);
    let orlis = point(&gym["leader_at"]);
    assert!(!can_approach(&mut c, &runtime, elvren));
    assert!(!can_approach(&mut c, &runtime, orlis));
    interact(&mut c, &runtime, moslen);
    battle(&mut c, false);
    finish(&mut c);
    cut_gym_tree(&mut c, &runtime, &gym["cut_gates"][0]);
    assert!(can_approach(&mut c, &runtime, elvren));
    assert!(!can_approach(&mut c, &runtime, orlis));
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(gym_collision(&mut c, &runtime, (3, 12)), "FLOOR");
    assert_eq!(gym_collision(&mut c, &runtime, (2, 12)), "WALL");
    assert_eq!(gym_collision(&mut c, &runtime, (2, 13)), "WALL");
    interact(&mut c, &runtime, elvren);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_7_COURT_OPEN"));
    cut_gym_tree(&mut c, &runtime, &gym["cut_gates"][1]);
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(gym_collision(&mut c, &runtime, (7, 6)), "FLOOR");
    assert!(
        !event(&c, "EVENT_MN_GYM_7_TRAINER_3_DEFEATED"),
        "side battle remains optional"
    );
    interact(&mut c, &runtime, orlis);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_7_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_7_TM_RECEIVED"));
    assert!(c.shell.shell.session().state().badges.johto[6]);
    let tm = &runtime.data().items["TM_GIGA_DRAIN"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    interact(&mut c, &runtime, orlis);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    interact(&mut c, &runtime, point(&gym["trainers"][2]["at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_7_TRAINER_3_DEFEATED"));
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert_eq!(gym_collision(&mut c, &runtime, (3, 12)), "FLOOR");
        assert_eq!(gym_collision(&mut c, &runtime, (7, 6)), "FLOOR");
    }
    for (edge, button) in [
        ((0, 18), GameButton::Left),
        ((11, 18), GameButton::Right),
        ((0, 19), GameButton::Down),
    ] {
        walk_to(&mut c, &runtime, edge);
        press(&mut c, button);
        assert_eq!(
            at(&mut c),
            edge,
            "original-style floor edge is bounded by the native map"
        );
    }
    c = continue_save(&mut c, &root, &runtime);
    assert!(event(&c, "EVENT_MN_GYM_7_DEFEATED"));
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_bemidji_controller_cut_requires_move_badge_and_confirmation() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][6];
    for missing_move in [true, false] {
        let mut c = controller(&root, &runtime, gym, false);
        // Initial permission fixtures only; all subsequent interaction is input.
        let state = c.shell.shell.session_mut().state_mut();
        if missing_move {
            state.storage.party.pokemon[0]
                .as_mut()
                .expect("fixture partner")
                .moves
                .retain(|m| m.name != "CUT");
            state.sync_party_from_storage();
        } else {
            state.badges.johto[1] = false;
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        walk_to(&mut c, &runtime, (3, 13));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        let s = c.snapshot().expect("native Cut denial");
        assert!(
            s.ui.text
                .as_ref()
                .and_then(|t| t.asm_text.as_ref())
                .is_some_and(|t| t.contains("CUT"))
        );
        finish(&mut c);
        assert_eq!(gym_collision(&mut c, &runtime, (3, 12)), "CUT_TREE");
        assert_eq!(at(&mut c), (3, 13));
        assert!(!can_approach(
            &mut c,
            &runtime,
            point(&gym["trainers"][1]["at"])
        ));
    }
    let mut c = controller(&root, &runtime, gym, false);
    walk_to(&mut c, &runtime, (3, 13));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(c.shell.yes_no_cursor.is_some(), "native Cut confirmation");
    press(&mut c, GameButton::B);
    finish(&mut c);
    assert_eq!(gym_collision(&mut c, &runtime, (3, 12)), "CUT_TREE");
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_bemidji_controller_cut_cannot_skip_either_required_trainer() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][6];
    let mut c = controller(&root, &runtime, gym, false);
    for gate in gym["cut_gates"].as_array().expect("both hedges") {
        cut_gym_tree(&mut c, &runtime, gate);
    }
    interact(&mut c, &runtime, point(&gym["trainers"][1]["at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_7_TRAINER_2_DEFEATED"));
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_7_DEFEATED"));
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, false);
    finish(&mut c);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_7_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_7_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[6]);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_bemidji_controller_loss_and_missing_sixth_badge_grant_nothing() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][6];
    let mut loser = controller(&root, &runtime, gym, true);
    interact(&mut loser, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut loser, true);
    finish(&mut loser);
    assert!(!event(&loser, "EVENT_MN_GYM_7_TRAINER_1_DEFEATED"));
    assert!(!event(&loser, "EVENT_MN_GYM_7_DEFEATED"));
    assert!(!loser.shell.shell.session().state().badges.johto[6]);
    let mut c = controller(&root, &runtime, gym, false);
    c.shell.shell.session_mut().state_mut().badges.johto[5] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    solve_puzzle(&mut c, &runtime, gym);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_7_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_7_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[6]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_GIGA_DRAIN"]),
        0
    );
}

fn learn_gym_cut_from_guide(c: &mut VisibleShellController, runtime: &CrystalRuntime, gym: &Value) {
    learn_gym_cut_replacing(c, runtime, gym, None);
}

fn learn_gym_cut_replacing(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    gym: &Value,
    forgotten: Option<&str>,
) {
    let hm = &runtime.data().items["HM_CUT"];
    let target_slot = c
        .snapshot()
        .expect("legal Cut carrier")
        .party
        .slots
        .iter()
        .position(|p| p.pokemon.species.tmhm_learnset.iter().any(|m| m == "CUT"))
        .expect("earned party includes a legal Cut carrier");
    assert_eq!(c.shell.shell.session().state().bag.quantity(hm), 0);
    assert!(
        !c.snapshot().expect("initial partner").party.slots[target_slot]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "CUT")
    );
    let forgotten_slot = forgotten.map(|name| {
        c.snapshot().expect("chosen HM replacement").party.slots[target_slot]
            .pokemon
            .moves
            .iter()
            .position(|m| m.name == name)
            .or_else(|| {
                c.snapshot().expect("regional HM replacement").party.slots[target_slot]
                    .pokemon
                    .moves
                    .iter()
                    .position(|m| runtime.data().moves[&m.name].power == 0)
            })
            .unwrap_or(0)
    });
    interact(c, runtime, point(&gym["guide_at"]));
    finish(c);
    assert!(event(c, "EVENT_MN_GYM_7_CUT_HM_RECEIVED"));
    assert_eq!(c.shell.shell.session().state().bag.quantity(hm), 1);
    let bag = c.shell.shell.session().state().bag.clone();
    interact(c, runtime, point(&gym["guide_at"]));
    finish(c);
    assert_eq!(
        c.shell.shell.session().state().bag,
        bag,
        "guide gift cannot repeat"
    );
    press(c, GameButton::Start);
    let entries = super::super::visible_start_menu_entries(&c.shell).expect("native Start");
    let target = entries
        .iter()
        .position(|s| s.trim_start_matches('>').trim() == "PACK")
        .expect("PACK");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    for _ in 0..4 {
        if c.shell.tmhm_cursor.is_some() {
            break;
        }
        press(c, GameButton::Right);
    }
    assert!(c.shell.tmhm_cursor.is_some());
    let hm_index = c
        .snapshot()
        .expect("carried HM inventory")
        .bag
        .tm_hm
        .iter()
        .position(|item| item.item_id == "HM_CUT")
        .expect("actual guide gift");
    for _ in 0..128 {
        let selected = c
            .shell
            .tmhm_cursor
            .as_ref()
            .expect("native HM cursor")
            .option_index;
        if selected == hm_index {
            break;
        }
        press(
            c,
            if selected < hm_index {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    assert_eq!(
        c.shell
            .tmhm_cursor
            .as_ref()
            .expect("selected Cut HM")
            .option_index,
        hm_index
    );
    for _ in 0..128 {
        if c.snapshot().expect("native HM learning").party.slots[target_slot]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "CUT")
        {
            break;
        }
        let button = if let Some(target) = forgotten_slot
            && c.shell.tmhm_forget_menu_open
            && c.shell.pending_tmhm_text_stage.is_none()
            && c.shell.field_notice.is_none()
        {
            let index = c
                .shell
                .party_move_cursor
                .as_ref()
                .expect("native HM forget cursor")
                .option_index;
            if index < target {
                GameButton::Down
            } else if index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if c.shell.field_pack_target_mode
            == Some(super::super::FieldPackTargetMode::TmHmPokemon)
        {
            if c.shell.party_cursor < target_slot {
                GameButton::Down
            } else if c.shell.party_cursor > target_slot {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    assert!(
        c.snapshot().expect("learned Cut").party.slots[target_slot]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "CUT")
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(hm),
        1,
        "HM is reusable"
    );
    assert_eq!(
        c.shell.shell.session().state().bag,
        bag,
        "teaching Cut cannot consume another TM"
    );
    for _ in 0..128 {
        if !super::super::has_visible_shell_direction_action(&mut c.shell) {
            break;
        }
        let text = c.shell.pending_tmhm_text_stage.is_some() || c.shell.field_notice.is_some();
        press(c, if text { GameButton::A } else { GameButton::B });
    }
    assert!(!super::super::has_visible_shell_direction_action(
        &mut c.shell
    ));
}

#[test]
#[ignore = "requires the published native Bemidji city and canonical gym pack"]
fn minnesota_bemidji_city_controller_facade_guide_hm_pack_cut_battles_and_both_exits() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][6];
    let mut initial = gym.clone();
    initial["id"] = serde_json::json!("MnBemidji");
    initial["arrival_core"] = serde_json::json!([10, 10]);
    let mut c = controller(&root, &runtime, &initial, false);
    // Initial party has three legal battle moves and no Cut or teaching item.
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon[0]
        .as_mut()
        .expect("fixture partner")
        .moves
        .retain(|m| m.name != "CUT");
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    for _ in 0..6 {
        press(&mut c, GameButton::Up);
        if c.snapshot().expect("gym door settling").overworld.map_name == "MnBemidjiGym" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("real gym door").overworld.map_name,
        "MnBemidjiGym"
    );
    assert_eq!(at(&mut c), point(&gym["arrival_core"]));
    learn_gym_cut_from_guide(&mut c, &runtime, gym);
    c = continue_save(&mut c, &root, &runtime);
    solve_puzzle(&mut c, &runtime, gym);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[6]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_GIGA_DRAIN"]),
        1
    );
    c = continue_save(&mut c, &root, &runtime);
    for carpet in gym["exit_tiles_core"].as_array().expect("both exits") {
        let (x, y) = point(carpet);
        walk_to(&mut c, &runtime, (x, y - 1));
        for _ in 0..4 {
            press(&mut c, GameButton::Down);
            if c.snapshot().expect("city return").overworld.map_name == "MnBemidji" {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("real safe street").overworld.map_name,
            "MnBemidji"
        );
        assert_eq!(at(&mut c), (10, 10));
        for _ in 0..6 {
            press(&mut c, GameButton::Up);
            if c.snapshot().expect("gym door settling").overworld.map_name == "MnBemidjiGym" {
                break;
            }
        }
        assert_eq!(at(&mut c), point(&gym["arrival_core"]));
        assert_eq!(gym_collision(&mut c, &runtime, (3, 12)), "FLOOR");
        assert_eq!(gym_collision(&mut c, &runtime, (7, 6)), "FLOOR");
    }
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_GIGA_DRAIN"]),
        1
    );
}

fn slide_gym_ice(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    from: Point,
    stop: Point,
    forward: GameButton,
    opposite: GameButton,
) {
    walk_to(c, runtime, from);
    for _ in 0..8 {
        c.press(forward).expect("enter native ice");
        c.wait_frames(1).expect("ice entry frame");
        if at(c) != from {
            break;
        }
    }
    let entered = at(c);
    assert_ne!(entered, from);
    if entered == stop {
        assert!(
            from.0.abs_diff(stop.0) + from.1.abs_diff(stop.1) <= 2,
            "long slides must expose a live intermediate tile"
        );
    } else {
        assert_eq!(gym_collision(c, runtime, entered), "ICE");
        c.press(opposite).expect("opposing input during live slide");
    }
    c.wait_frames(160)
        .expect("empty frames carry the player to the native stop");
    assert_eq!(
        at(c),
        stop,
        "ice owns direction until the floor patch or wall"
    );
    c.wait_frames(80).expect("rest at the native ice stop");
    assert_eq!(at(c), stop);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_duluth_controller_two_courses_require_all_three_trainers_and_persist() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][7];
    let mut c = controller(&root, &runtime, gym, false);
    let livra = point(&gym["trainers"][0]["at"]);
    let tordel = point(&gym["trainers"][1]["at"]);
    let anver = point(&gym["trainers"][2]["at"]);
    let norek = point(&gym["leader_at"]);
    assert!(!can_approach(&mut c, &runtime, tordel));
    assert!(!can_approach(&mut c, &runtime, norek));
    interact(&mut c, &runtime, anver);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_8_TRAINER_3_DEFEATED"));
    interact(&mut c, &runtime, livra);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_WEST_RETURN_OPEN"));
    assert!(event(&c, "EVENT_MN_GYM_8_EAST_ENTRY_OPEN"));
    assert_eq!(gym_collision(&mut c, &runtime, (4, 14)), "FLOOR");
    assert_eq!(gym_collision(&mut c, &runtime, (11, 14)), "ICE");
    assert!(!event(&c, "EVENT_MN_GYM_8_EAST_RETURN_OPEN"));
    assert!(!can_approach(&mut c, &runtime, norek));
    c = continue_save(&mut c, &root, &runtime);
    interact(&mut c, &runtime, anver);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_8_TRAINER_3_DEFEATED"));
    slide_gym_ice(
        &mut c,
        &runtime,
        (11, 16),
        (11, 10),
        GameButton::Up,
        GameButton::Down,
    );
    slide_gym_ice(
        &mut c,
        &runtime,
        (11, 10),
        (9, 10),
        GameButton::Left,
        GameButton::Right,
    );
    walk_to(&mut c, &runtime, (9, 7));
    walk_to(&mut c, &runtime, (11, 16));
    assert!(!event(&c, "EVENT_MN_GYM_8_EAST_RETURN_OPEN"));
    walk_to(&mut c, &runtime, (9, 7));
    interact(&mut c, &runtime, tordel);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_EAST_RETURN_OPEN"));
    assert!(!event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    assert_eq!(gym_collision(&mut c, &runtime, (6, 6)), "FLOOR");
    assert!(!can_approach(&mut c, &runtime, norek));
    c = continue_save(&mut c, &root, &runtime);
    interact(&mut c, &runtime, anver);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    assert_eq!(gym_collision(&mut c, &runtime, (6, 4)), "FLOOR");
    c = continue_save(&mut c, &root, &runtime);
    interact(&mut c, &runtime, norek);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_8_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_8_TM_RECEIVED"));
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    let tm = &runtime.data().items["TM_ICY_WIND"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    interact(&mut c, &runtime, norek);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    // Both routes remain usable after winning: west return and east ice in reverse.
    walk_to(&mut c, &runtime, (4, 14));
    walk_to(&mut c, &runtime, (9, 10));
    walk_to(&mut c, &runtime, (11, 16));
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        exit_and_reenter(&mut c, &runtime, gym, point(carpet));
        assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
        assert_eq!(gym_collision(&mut c, &runtime, (11, 14)), "ICE");
    }
    for (edge, button) in [
        ((0, 18), GameButton::Left),
        ((13, 18), GameButton::Right),
        ((0, 19), GameButton::Down),
    ] {
        walk_to(&mut c, &runtime, edge);
        press(&mut c, button);
        assert_eq!(at(&mut c), edge, "native floor edge remains bounded");
    }
    c = continue_save(&mut c, &root, &runtime);
    assert!(event(&c, "EVENT_MN_GYM_8_DEFEATED"));
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_duluth_controller_loss_keeps_east_course_and_return_closed() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][7];
    let mut c = controller(&root, &runtime, gym, true);
    interact(&mut c, &runtime, point(&gym["trainers"][0]["at"]));
    battle(&mut c, true);
    finish(&mut c);
    for name in [
        "EVENT_MN_GYM_8_TRAINER_1_DEFEATED",
        "EVENT_MN_GYM_8_EAST_ENTRY_OPEN",
        "EVENT_MN_GYM_8_WEST_RETURN_OPEN",
        "EVENT_MN_GYM_8_DEFEATED",
        "EVENT_MN_GYM_8_TM_RECEIVED",
    ] {
        assert!(!event(&c, name));
    }
    assert!(!c.shell.shell.session().state().badges.johto[7]);
}

#[test]
#[ignore = "requires the published external Minnesota gym pack"]
fn minnesota_duluth_controller_seventh_badge_is_required_for_norek() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][7];
    let mut c = controller(&root, &runtime, gym, false);
    c.shell.shell.session_mut().state_mut().badges.johto[6] = false;
    mark_runtime_snapshot_dirty(&mut c.shell);
    solve_puzzle(&mut c, &runtime, gym);
    assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_8_DEFEATED"));
    assert!(!event(&c, "EVENT_MN_GYM_8_TM_RECEIVED"));
    assert!(!c.shell.shell.session().state().badges.johto[7]);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_ICY_WIND"]),
        0
    );
}

#[test]
#[ignore = "requires the published native Duluth city and canonical gym pack"]
fn minnesota_duluth_city_controller_facade_all_three_trainers_badge_and_receiver() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][7];
    let mut initial = gym.clone();
    initial["id"] = serde_json::json!("MnDuluth");
    initial["arrival_core"] = serde_json::json!([16, 12]);
    let mut c = controller(&root, &runtime, &initial, false);
    // Only the upstream depot and first seven badges are fixtures. Badge8,
    // every gym victory, the TM and the receiver's signal must be earned.
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag("EVENT_MN_DEPOT_CLEARED", true)
        .expect("upstream quest fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    for _ in 0..6 {
        press(&mut c, GameButton::Up);
        if c.snapshot().expect("real facade").overworld.map_name == "MnDuluthGym" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("gym entrance").overworld.map_name,
        "MnDuluthGym"
    );
    assert_eq!(at(&mut c), point(&gym["arrival_core"]));
    assert!(!event(&c, "EVENT_MN_LIGHTHOUSE_SIGNAL"));
    assert!(!c.shell.shell.session().state().badges.johto[7]);
    solve_puzzle(&mut c, &runtime, gym);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    assert!(event(&c, "EVENT_MN_GYM_8_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_8_TM_RECEIVED"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_ICY_WIND"]),
        1
    );
    assert!(
        !event(&c, "EVENT_MN_LIGHTHOUSE_SIGNAL"),
        "badge alone cannot acknowledge the receiver"
    );
    c = continue_save(&mut c, &root, &runtime);
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        let (x, y) = point(carpet);
        walk_to(&mut c, &runtime, (x, y - 1));
        for _ in 0..4 {
            press(&mut c, GameButton::Down);
            if c.snapshot().expect("city return").overworld.map_name == "MnDuluth" {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("safe street").overworld.map_name,
            "MnDuluth"
        );
        assert_eq!(at(&mut c), (16, 12));
        for _ in 0..6 {
            press(&mut c, GameButton::Up);
            if c.snapshot().expect("gym return").overworld.map_name == "MnDuluthGym" {
                break;
            }
        }
        assert_eq!(at(&mut c), point(&gym["arrival_core"]));
        assert!(event(&c, "EVENT_MN_GYM_8_COURT_OPEN"));
    }
    walk_to(&mut c, &runtime, (6, 18));
    for _ in 0..4 {
        press(&mut c, GameButton::Down);
        if c.snapshot().expect("leave gym").overworld.map_name == "MnDuluth" {
            break;
        }
    }
    walk_to(&mut c, &runtime, (48, 36));
    for _ in 0..6 {
        press(&mut c, GameButton::Up);
        if c.snapshot().expect("workshop door").overworld.map_name == "MnDuluthHarborWorkshop" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("real workshop").overworld.map_name,
        "MnDuluthHarborWorkshop"
    );
    let before = c.shell.shell.session().state().clone();
    interact(&mut c, &runtime, (11, 5));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_LIGHTHOUSE_SIGNAL"));
    let mut flags = before.flags.clone();
    flags
        .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", true)
        .expect("expected receiver signal");
    assert_eq!(c.shell.shell.session().state().flags, flags);
    assert_eq!(c.shell.shell.session().state().bag, before.bag);
    assert_eq!(c.shell.shell.session().state().badges, before.badges);
    interact(&mut c, &runtime, (11, 5));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().flags, flags);
    c = continue_save(&mut c, &root, &runtime);
    assert!(event(&c, "EVENT_MN_LIGHTHOUSE_SIGNAL"));
}

#[test]
#[ignore = "requires the freshly published external Mankato city/gym pack"]
fn minnesota_mankato_city_controller_facade_terrace_trainers_badge_and_both_returns() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][1];
    let mut initial = gym.clone();
    initial["id"] = serde_json::json!("MnMankato");
    initial["arrival_core"] = serde_json::json!([40, 18]);
    // Explicit city placement, strong legal party and badge1 isolate local play.
    let mut c = controller(&root, &runtime, &initial, false);
    take_drop(
        &mut c,
        &runtime,
        &serde_json::json!({
            "hop_from": [10,18], "landing": [10,20], "direction": "south"
        }),
    );
    let city = module(&mut c, &runtime);
    assert!(
        path(&runtime, &city, (10, 20), (10, 17))
            .expect("ordinary return")
            .len()
            >= 24
    );
    walk_to(&mut c, &runtime, (10, 12));
    for _ in 0..6 {
        press(&mut c, GameButton::Up);
        if c.snapshot().expect("facade").overworld.map_name == "MnMankatoGym" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("real gym").overworld.map_name,
        "MnMankatoGym"
    );
    assert_eq!(at(&mut c), point(&gym["arrival_core"]));
    assert!(!can_approach(
        &mut c,
        &runtime,
        point(&gym["trainers"][1]["at"])
    ));
    assert!(!can_approach(&mut c, &runtime, point(&gym["leader_at"])));
    solve_puzzle(&mut c, &runtime, gym);
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_2_TRAINER_2_DEFEATED"));
    assert!(event(&c, "EVENT_MN_GYM_2_DEFEATED"));
    assert!(c.shell.shell.session().state().badges.johto[1]);
    let tm = &runtime.data().items["TM_MUD_SLAP"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &runtime);
    for carpet in gym["exit_tiles_core"].as_array().expect("two carpets") {
        let (x, y) = point(carpet);
        walk_to(&mut c, &runtime, (x, y - 1));
        for _ in 0..6 {
            press(&mut c, GameButton::Down);
            if c.snapshot().expect("city return").overworld.map_name == "MnMankato" {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("safe street").overworld.map_name,
            "MnMankato"
        );
        assert_eq!(at(&mut c), (10, 12));
        for _ in 0..6 {
            press(&mut c, GameButton::Up);
            if c.snapshot().expect("reenter").overworld.map_name == "MnMankatoGym" {
                break;
            }
        }
        assert_eq!(at(&mut c), point(&gym["arrival_core"]));
        assert!(can_approach(&mut c, &runtime, point(&gym["leader_at"])));
    }
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
}

fn city_enter_from_north(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    safe: Point,
    room: &str,
    arrival: Point,
) {
    walk_to(c, r, safe);
    for _ in 0..6 {
        press(c, GameButton::Up);
        if c.snapshot().expect("facade entry").overworld.map_name == room {
            break;
        }
    }
    assert_eq!(c.snapshot().expect("entered room").overworld.map_name, room);
    assert_eq!(at(c), arrival);
}
fn mankato_leave(c: &mut VisibleShellController, r: &CrystalRuntime, carpet: Point, safe: Point) {
    walk_to(c, r, (carpet.0, carpet.1 - 1));
    for _ in 0..6 {
        press(c, GameButton::Down);
        if c.snapshot().expect("room exit").overworld.map_name == "MnMankato" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("city return").overworld.map_name,
        "MnMankato"
    );
    assert_eq!(at(c), safe);
}

#[test]
#[ignore = "requires the freshly published external Mankato city/gym pack"]
fn minnesota_mankato_city_controller_care_pc_priced_shop_and_room_returns() {
    let (root, runtime, design) = load();
    let mut initial = design["gyms"][1].clone();
    initial["id"] = serde_json::json!("MnMankato");
    initial["arrival_core"] = serde_json::json!([40, 18]);
    let mut c = controller(&root, &runtime, &initial, false);
    // Injury, empty PP and spending money are explicit service fixtures.
    let state = c.shell.shell.session_mut().state_mut();
    let partner = state.storage.party.pokemon[0].as_mut().expect("partner");
    partner.hp = partner.max_hp.saturating_sub(15).max(1);
    partner.status = Some("PARALYSIS".into());
    for m in &mut partner.moves {
        m.current_pp = 0;
    }
    state.money = 10000;
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    city_enter_from_north(
        &mut c,
        &runtime,
        (31, 14),
        "MnMankatoPokecenter1F",
        (10, 13),
    );
    walk_to(&mut c, &runtime, (7, 5));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    let healed = c.snapshot().expect("care");
    assert!(healed.party.slots.iter().all(|s| {
        s.pokemon.hp == s.pokemon.max_hp
            && s.pokemon.status.is_none()
            && s.pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
    }));
    assert_eq!(c.shell.shell.session().state().money, 10000);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_MANKATO_POKECENTER_1F")
    );
    walk_to(&mut c, &runtime, (13, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
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
    assert!(c.snapshot().expect("PC").ui.menu.is_some());
    for _ in 0..32 {
        if c.shell.pc_hub_cursor.is_none() && c.snapshot().expect("PC exit").ui.menu.is_none() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    finish(&mut c);
    c = continue_save(&mut c, &root, &runtime);
    for x in [10, 11] {
        mankato_leave(&mut c, &runtime, (x, 15), (31, 14));
        city_enter_from_north(
            &mut c,
            &runtime,
            (31, 14),
            "MnMankatoPokecenter1F",
            (10, 13),
        );
    }
    mankato_leave(&mut c, &runtime, (10, 15), (31, 14));
    city_enter_from_north(&mut c, &runtime, (37, 28), "MnMankatoMart", (6, 5));
    walk_to(&mut c, &runtime, (3, 3));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
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
            .expect("shop")
            .pending_shop
            .expect("native mart")
            .mart_id,
        "MART_MN_MANKATO"
    );
    let before = c.shell.shell.session().state().clone();
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shop_quantity.as_ref().expect("quantity").item_id,
        "SUPER_POTION"
    );
    let item = &runtime.data().items["SUPER_POTION"];
    press(&mut c, GameButton::A);
    assert_eq!(c.shell.shell.session().state().money, before.money);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        before.money - u32::from(item.price)
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before.bag.quantity(item) + 1
    );
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
    finish(&mut c);
    c = continue_save(&mut c, &root, &runtime);
    for x in [6, 7] {
        mankato_leave(&mut c, &runtime, (x, 7), (37, 28));
        city_enter_from_north(&mut c, &runtime, (37, 28), "MnMankatoMart", (6, 5));
    }
}

#[test]
#[ignore = "requires the freshly published external St Paul city/gym pack"]
fn minnesota_st_paul_city_controller_route_facade_all_pad_pairs_trainers_badge_and_returns() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][3];
    let mut initial = gym.clone();
    initial["id"] = serde_json::json!("MnStPaul");
    initial["arrival_core"] = serde_json::json!([6, 24]);
    // Local fixture: street placement, legal strong party and badges1-3.
    let mut c = controller(&root, &runtime, &initial, false);
    for y in [24, 25] {
        walk_to(&mut c, &runtime, (0, y));
        for _ in 0..6 {
            press(&mut c, GameButton::Left);
            if c.snapshot().expect("west crossing").overworld.map_name == "MnR04C00R00" {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("route edge").overworld.map_name,
            "MnR04C00R00"
        );
        assert_eq!(at(&mut c), (55, y - 6));
        for _ in 0..6 {
            press(&mut c, GameButton::Right);
            if c.snapshot().expect("east crossing").overworld.map_name == "MnStPaul" {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("city edge").overworld.map_name,
            "MnStPaul"
        );
        assert_eq!(at(&mut c), (0, y));
    }
    take_drop(
        &mut c,
        &runtime,
        &serde_json::json!({"hop_from":[39,20],"landing":[39,22],"direction":"south"}),
    );
    city_enter_from_north(&mut c, &runtime, (54, 16), "MnStPaulGym", (2, 17));
    assert!(!can_approach(&mut c, &runtime, point(&gym["leader_at"])));
    for lift in gym["lifts"].as_array().expect("four reciprocal pairs") {
        ride_lift(&mut c, &runtime, lift, 0);
        c = continue_save(&mut c, &root, &runtime);
        assert_eq!(at(&mut c), point(&lift["ends_core"][1]));
        ride_lift(&mut c, &runtime, lift, 1);
    }
    solve_puzzle(&mut c, &runtime, gym);
    assert!(event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_4_DEFEATED"));
    assert!(c.shell.shell.session().state().badges.johto[3]);
    let tm = &runtime.data().items["TM_STEEL_WING"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &runtime);
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        let (x, y) = point(carpet);
        walk_to(&mut c, &runtime, (x, y - 1));
        for _ in 0..6 {
            press(&mut c, GameButton::Down);
            if c.snapshot().expect("gym return").overworld.map_name == "MnStPaul" {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("safe street").overworld.map_name,
            "MnStPaul"
        );
        assert_eq!(at(&mut c), (54, 16));
        city_enter_from_north(&mut c, &runtime, (54, 16), "MnStPaulGym", (2, 17));
        assert!(event(&c, "EVENT_MN_GYM_4_COURT_OPEN"));
        assert!(can_approach(&mut c, &runtime, point(&gym["leader_at"])));
    }
    interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
}

fn st_paul_leave(c: &mut VisibleShellController, r: &CrystalRuntime, carpet: Point, safe: Point) {
    walk_to(c, r, (carpet.0, carpet.1 - 1));
    for _ in 0..6 {
        press(c, GameButton::Down);
        if c.snapshot().expect("public room return").overworld.map_name == "MnStPaul" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("city return").overworld.map_name,
        "MnStPaul"
    );
    assert_eq!(at(c), safe);
}
#[test]
#[ignore = "requires the freshly published external St Paul city/gym pack"]
fn minnesota_st_paul_city_controller_all_public_facades_care_pc_and_priced_shop() {
    let (root, runtime, design) = load();
    let mut initial = design["gyms"][3].clone();
    initial["id"] = serde_json::json!("MnStPaul");
    initial["arrival_core"] = serde_json::json!([6, 24]);
    let mut c = controller(&root, &runtime, &initial, false);
    for (room, safe, arrival, xs, y) in [
        ("MnStPaulPokecenter1F", (7, 16), (3, 5), vec![3, 4], 7),
        ("MnStPaulMart", (23, 16), (3, 5), vec![2, 3], 7),
        ("MnStPaulCapitolHall", (34, 10), (3, 5), vec![2, 3], 7),
        (
            "MnStPaulUnionDepot",
            (44, 38),
            (15, 21),
            vec![14, 15, 16, 17],
            23,
        ),
    ] {
        for x in xs {
            city_enter_from_north(&mut c, &runtime, safe, room, arrival);
            c = continue_save(&mut c, &root, &runtime);
            if room == "MnStPaulUnionDepot" && x == 14 {
                walk_to(&mut c, &runtime, (17, 10));
                for _ in 0..3 {
                    press(&mut c, GameButton::Up);
                }
                assert_eq!(at(&mut c), (17, 10), "conductor seals unfinished boarding");
                let station = module(&mut c, &runtime);
                for target in [(17, 7), (12, 5), (17, 5)] {
                    assert!(
                        path(&runtime, &station, at(&mut c), target).is_none(),
                        "unbound train entrance is reachable"
                    );
                }
            }
            st_paul_leave(&mut c, &runtime, (x, y), safe);
        }
    }
    // Service fixtures supply injury, empty PP and spending money only.
    let state = c.shell.shell.session_mut().state_mut();
    let p = state.storage.party.pokemon[0].as_mut().expect("partner");
    p.hp = p.max_hp.saturating_sub(15).max(1);
    p.status = Some("PARALYSIS".into());
    for m in &mut p.moves {
        m.current_pp = 0;
    }
    state.money = 10000;
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    city_enter_from_north(&mut c, &runtime, (7, 16), "MnStPaulPokecenter1F", (3, 5));
    walk_to(&mut c, &runtime, (3, 3));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    let healed = c.snapshot().expect("native free care");
    assert!(healed.party.slots.iter().all(|s| {
        s.pokemon.hp == s.pokemon.max_hp
            && s.pokemon.status.is_none()
            && s.pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
    }));
    assert_eq!(c.shell.shell.session().state().money, 10000);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_ST_PAUL_POKECENTER_1F")
    );
    walk_to(&mut c, &runtime, (9, 2));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
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
    assert!(c.snapshot().expect("PC").ui.menu.is_some());
    for _ in 0..32 {
        if c.shell.pc_hub_cursor.is_none() && c.snapshot().expect("PC exit").ui.menu.is_none() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    finish(&mut c);
    st_paul_leave(&mut c, &runtime, (3, 7), (7, 16));
    city_enter_from_north(&mut c, &runtime, (23, 16), "MnStPaulMart", (3, 5));
    walk_to(&mut c, &runtime, (3, 3));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
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
        "MART_MN_ST_PAUL"
    );
    let item = &runtime.data().items["SUPER_POTION"];
    let before = (
        c.shell.shell.session().state().money,
        c.shell.shell.session().state().bag.quantity(item),
    );
    for button in [
        GameButton::A,
        GameButton::Down,
        GameButton::Down,
        GameButton::A,
    ] {
        press(&mut c, button);
    }
    assert_eq!(
        c.shell.shop_quantity.as_ref().expect("quantity").item_id,
        "SUPER_POTION"
    );
    press(&mut c, GameButton::A);
    assert_eq!(c.shell.shell.session().state().money, before.0);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        before.0 - u32::from(item.price)
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before.1 + 1
    );
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
    finish(&mut c);
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before.1 + 1
    );
    assert!(
        !event(&c, "EVENT_MN_COURIER_DEFEATED"),
        "ordinary depot access must not grant the unfinished quest"
    );
}

#[test]
#[ignore = "requires the freshly published external St Cloud city/gym pack"]
fn minnesota_st_cloud_city_controller_both_routes_wind_trainers_badge_and_returns() {
    let (root, runtime, design) = load();
    let gym = &design["gyms"][4];
    let mut initial = gym.clone();
    initial["id"] = serde_json::json!("MnStCloud");
    initial["arrival_core"] = serde_json::json!([24, 32]);
    // Local fixture: city placement, legal strong partner and badges1-4.
    let mut c = controller(&root, &runtime, &initial, false);
    city_enter_from_north(&mut c, &runtime, (8, 28), "MnStCloudGym", (4, 19));
    let wyrel = point(&gym["trainers"][0]["at"]);
    let hallen = point(&gym["trainers"][1]["at"]);
    let saven = point(&gym["leader_at"]);
    assert!(!can_approach(&mut c, &runtime, hallen));
    assert!(!can_approach(&mut c, &runtime, saven));
    cross_gust(
        &mut c,
        &runtime,
        (7, 16),
        (7, 16),
        GameButton::Right,
        GameButton::Right,
    );
    interact(&mut c, &runtime, wyrel);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_5_LOWER_CROSSING_OPEN"));
    assert!(!can_approach(&mut c, &runtime, saven));
    c = continue_save(&mut c, &root, &runtime);
    cross_gust(
        &mut c,
        &runtime,
        (7, 13),
        (12, 13),
        GameButton::Right,
        GameButton::Left,
    );
    c = continue_save(&mut c, &root, &runtime);
    cross_gust(
        &mut c,
        &runtime,
        (12, 4),
        (12, 4),
        GameButton::Left,
        GameButton::Left,
    );
    assert!(!can_approach(&mut c, &runtime, saven));
    interact(&mut c, &runtime, hallen);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_GYM_5_COURT_OPEN"));
    c = continue_save(&mut c, &root, &runtime);
    cross_gust(
        &mut c,
        &runtime,
        (12, 9),
        (7, 9),
        GameButton::Left,
        GameButton::Right,
    );
    interact(&mut c, &runtime, saven);
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[4]);
    assert!(event(&c, "EVENT_MN_GYM_5_DEFEATED"));
    let tm = &runtime.data().items["TM_SWIFT"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    cross_gust(
        &mut c,
        &runtime,
        (7, 5),
        (12, 5),
        GameButton::Right,
        GameButton::Left,
    );
    cross_gust(
        &mut c,
        &runtime,
        (12, 17),
        (7, 17),
        GameButton::Left,
        GameButton::Right,
    );
    c = continue_save(&mut c, &root, &runtime);
    for carpet in gym["exit_tiles_core"].as_array().expect("both carpets") {
        let (x, y) = point(carpet);
        walk_to(&mut c, &runtime, (x, y - 1));
        for _ in 0..6 {
            press(&mut c, GameButton::Down);
            if c.snapshot().expect("gym return").overworld.map_name == "MnStCloud" {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("safe city").overworld.map_name,
            "MnStCloud"
        );
        assert_eq!(at(&mut c), (8, 28));
        city_enter_from_north(&mut c, &runtime, (8, 28), "MnStCloudGym", (4, 19));
        assert!(event(&c, "EVENT_MN_GYM_5_TRAINER_1_DEFEATED"));
        assert!(event(&c, "EVENT_MN_GYM_5_TRAINER_2_DEFEATED"));
        assert!(can_approach(&mut c, &runtime, saven));
    }
    interact(&mut c, &runtime, saven);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    // Regional R06 departure requires the Wind Badge actually won above.
    st_cloud_leave(&mut c, &runtime, (4, 21), (8, 28));
    for (y, out, route, arrival_y, back, dx) in [
        (35, GameButton::Down, "MnR05C00R00", 0, GameButton::Up, 0),
        (0, GameButton::Up, "MnR06C00R01", 39, GameButton::Down, 8),
    ] {
        for x in [24, 25] {
            // Approach the real checkpoint before planning beyond its bodies.
            // Walking onto y=3 runs the native earned-badge script.
            walk_to(&mut c, &runtime, (x, if y == 0 { 4 } else { y }));
            for _ in 0..6 {
                press(&mut c, out);
                if c.snapshot().expect("route crossing").overworld.map_name == route {
                    break;
                }
            }
            assert_eq!(c.snapshot().expect("route").overworld.map_name, route);
            assert_eq!(at(&mut c), (x - dx, arrival_y));
            for _ in 0..6 {
                press(&mut c, back);
                if c.snapshot().expect("city crossing").overworld.map_name == "MnStCloud" {
                    break;
                }
            }
            assert_eq!(c.snapshot().expect("city").overworld.map_name, "MnStCloud");
            assert_eq!(at(&mut c), (x, y));
            if y == 0 {
                // Re-enter through the native inbound trigger before a path
                // planner considers walking across the guarded lanes.
                for _ in 0..12 {
                    if at(&mut c) == (x, 4) {
                        break;
                    }
                    press(&mut c, GameButton::Down);
                }
                assert_eq!(at(&mut c), (x, 4));
            }
        }
    }
}

fn st_cloud_leave(c: &mut VisibleShellController, r: &CrystalRuntime, carpet: Point, safe: Point) {
    walk_to(c, r, (carpet.0, carpet.1 - 1));
    for _ in 0..6 {
        press(c, GameButton::Down);
        if c.snapshot().expect("public room return").overworld.map_name == "MnStCloud" {
            break;
        }
    }
    assert_eq!(
        c.snapshot().expect("city return").overworld.map_name,
        "MnStCloud"
    );
    assert_eq!(at(c), safe);
}
#[test]
#[ignore = "requires the freshly published external St Cloud city/gym pack"]
fn minnesota_st_cloud_city_controller_all_public_facades_care_pc_and_priced_shop() {
    let (root, runtime, design) = load();
    let mut initial = design["gyms"][4].clone();
    initial["id"] = serde_json::json!("MnStCloud");
    initial["arrival_core"] = serde_json::json!([24, 32]);
    let mut c = controller(&root, &runtime, &initial, false);
    for (room, safe, arrival, xs, y) in [
        ("MnStCloudPokecenter1F", (6, 10), (10, 13), vec![10, 11], 15),
        ("MnStCloudMart", (14, 10), (6, 5), vec![6, 7], 7),
        ("MnStCloudStation", (35, 12), (11, 17), vec![10, 11], 19),
    ] {
        for x in xs {
            city_enter_from_north(&mut c, &runtime, safe, room, arrival);
            c = continue_save(&mut c, &root, &runtime);
            if room == "MnStCloudStation" && x == 10 {
                walk_to(&mut c, &runtime, (11, 10));
                for _ in 0..3 {
                    press(&mut c, GameButton::Up);
                }
                assert_eq!(at(&mut c), (11, 10), "conductor seals unfinished boarding");
                let station = module(&mut c, &runtime);
                for target in [(11, 7), (6, 5), (11, 5)] {
                    assert!(
                        path(&runtime, &station, at(&mut c), target).is_none(),
                        "unbound train entrance is reachable"
                    );
                }
            }
            st_cloud_leave(&mut c, &runtime, (x, y), safe);
        }
    }
    // Service fixtures supply injury, empty PP and spending money only.
    let state = c.shell.shell.session_mut().state_mut();
    let p = state.storage.party.pokemon[0].as_mut().expect("partner");
    p.hp = p.max_hp.saturating_sub(15).max(1);
    p.status = Some("PARALYSIS".into());
    for m in &mut p.moves {
        m.current_pp = 0;
    }
    state.money = 10000;
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    city_enter_from_north(&mut c, &runtime, (6, 10), "MnStCloudPokecenter1F", (10, 13));
    walk_to(&mut c, &runtime, (7, 5));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    let healed = c.snapshot().expect("native free care");
    assert!(healed.party.slots.iter().all(|s| {
        s.pokemon.hp == s.pokemon.max_hp
            && s.pokemon.status.is_none()
            && s.pokemon
                .moves
                .iter()
                .all(|m| m.current_pp == runtime.data().moves[&m.name].pp)
    }));
    assert_eq!(c.shell.shell.session().state().money, 10000);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_ST_CLOUD_POKECENTER_1F")
    );
    walk_to(&mut c, &runtime, (15, 4));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
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
    assert!(c.snapshot().expect("PC").ui.menu.is_some());
    for _ in 0..32 {
        if c.shell.pc_hub_cursor.is_none() && c.snapshot().expect("PC exit").ui.menu.is_none() {
            break;
        }
        press(&mut c, GameButton::B);
    }
    finish(&mut c);
    st_cloud_leave(&mut c, &runtime, (10, 15), (6, 10));
    city_enter_from_north(&mut c, &runtime, (14, 10), "MnStCloudMart", (6, 5));
    walk_to(&mut c, &runtime, (3, 3));
    press(&mut c, GameButton::Left);
    press(&mut c, GameButton::A);
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
        "MART_MN_ST_CLOUD"
    );
    let item = &runtime.data().items["SUPER_POTION"];
    let before = (
        c.shell.shell.session().state().money,
        c.shell.shell.session().state().bag.quantity(item),
    );
    for button in [
        GameButton::A,
        GameButton::Down,
        GameButton::Down,
        GameButton::A,
    ] {
        press(&mut c, button);
    }
    assert_eq!(
        c.shell.shop_quantity.as_ref().expect("quantity").item_id,
        "SUPER_POTION"
    );
    press(&mut c, GameButton::A);
    assert_eq!(c.shell.shell.session().state().money, before.0);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        before.0 - u32::from(item.price)
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before.1 + 1
    );
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
    finish(&mut c);
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before.1 + 1
    );
    assert!(
        !event(&c, "EVENT_MN_COURIER_DEFEATED"),
        "ordinary depot access must not grant the unfinished quest"
    );
}

fn real_city_gym_returns(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    gym: &Value,
    city: &str,
    safe: Point,
) {
    for carpet in gym["exit_tiles_core"]
        .as_array()
        .expect("two native carpets")
    {
        let (x, y) = point(carpet);
        walk_to(c, r, (x, y - 1));
        for _ in 0..6 {
            press(c, GameButton::Down);
            if c.snapshot().expect("city exit").overworld.map_name == city {
                break;
            }
        }
        assert_eq!(c.snapshot().expect("real city").overworld.map_name, city);
        assert_eq!(at(c), safe);
        city_enter_from_north(
            c,
            r,
            safe,
            gym["id"].as_str().expect("gym"),
            point(&gym["arrival_core"]),
        );
    }
}
#[test]
#[ignore = "requires a regional candidate with current Rochester and Brainerd facades"]
fn minnesota_rochester_city_controller_canonical_trainers_badge_tm_and_returns() {
    let (root, r, design) = load();
    let gym = &design["gyms"][0];
    let mut initial = gym.clone();
    initial["id"] = serde_json::json!("MnRochester");
    initial["arrival_core"] = serde_json::json!([30, 24]);
    let mut c = controller(&root, &r, &initial, false); // Explicit legal party and completed delivery fixture.
    city_enter_from_north(&mut c, &r, (30, 24), "MnRochesterGym", (6, 17));
    interact(&mut c, &r, point(&gym["trainers"][1]["at"]));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_GYM_1_TRAINER_2_DEFEATED"));
    solve_puzzle(&mut c, &r, gym);
    interact(&mut c, &r, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[0]);
    let tm = &r.data().items["TM_HEADBUTT"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    real_city_gym_returns(&mut c, &r, gym, "MnRochester", (30, 24));
    interact(&mut c, &r, point(&gym["leader_at"]));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
}
#[test]
#[ignore = "requires a regional candidate with current Rochester and Brainerd facades"]
fn minnesota_brainerd_city_controller_canonical_piers_trainers_badge_tm_and_returns() {
    let (root, r, design) = load();
    let gym = &design["gyms"][5];
    let mut initial = gym.clone();
    initial["id"] = serde_json::json!("MnBrainerd");
    initial["arrival_core"] = serde_json::json!([12, 36]);
    let mut c = controller(&root, &r, &initial, false); // Explicit legal party and prior badges1-5.
    city_enter_from_north(
        &mut c,
        &r,
        (12, 36),
        "MnBrainerdGym",
        point(&gym["arrival_core"]),
    );
    assert!(!can_approach(&mut c, &r, point(&gym["trainers"][1]["at"])));
    assert!(!can_approach(&mut c, &r, point(&gym["leader_at"])));
    solve_puzzle(&mut c, &r, gym);
    assert!(event(&c, "EVENT_MN_GYM_6_COURT_OPEN"));
    assert_eq!(gym_collision(&mut c, &r, (5, 9)), "WATER");
    assert_eq!(gym_collision(&mut c, &r, (3, 9)), "FLOOR");
    interact(&mut c, &r, point(&gym["leader_at"]));
    battle(&mut c, false);
    finish(&mut c);
    assert!(c.shell.shell.session().state().badges.johto[5]);
    let tm = &r.data().items["TM_RAIN_DANCE"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    c = continue_save(&mut c, &root, &r);
    real_city_gym_returns(&mut c, &r, gym, "MnBrainerd", (12, 36));
    interact(&mut c, &r, point(&gym["leader_at"]));
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
}
#[test]
#[ignore = "requires the single published regional candidate containing all eight current gyms"]
fn minnesota_regional_gyms_controller_all_eight_actual_facades_and_local_courses() {
    // Eight separately initialized local fixtures share one compiled pack.
    // This deliberately does not claim a continuous regional journey.
    minnesota_rochester_city_controller_canonical_trainers_badge_tm_and_returns();
    minnesota_mankato_city_controller_facade_terrace_trainers_badge_and_both_returns();
    minnesota_minneapolis_city_controller_earns_both_switches_and_badge_through_the_facade();
    minnesota_st_paul_city_controller_route_facade_all_pad_pairs_trainers_badge_and_returns();
    minnesota_st_cloud_city_controller_both_routes_wind_trainers_badge_and_returns();
    minnesota_brainerd_city_controller_canonical_piers_trainers_badge_tm_and_returns();
    minnesota_bemidji_city_controller_facade_guide_hm_pack_cut_battles_and_both_exits();
    minnesota_duluth_city_controller_facade_all_three_trainers_badge_and_receiver();
}

#[test]
#[ignore = "full ordinary-party arc campaign; no review boosts or injected progress"]
pub(super) fn northstar_arc_unboosted_new_game_through_legendary_release_and_hall_of_fame() {
    earned_arc_through_hall_of_fame(true);
}

pub(super) fn earned_arc_through_hall_of_fame(verify_service_return: bool) {
    northstar_campaign_review::require_unboosted();
    let (root, r, c) = northstar_campaign_league::arrival();
    for flag in [
        "EVENT_MN_REED_WINONA_DEFEATED",
        "EVENT_MN_REED_ROCHESTER_DEFEATED",
        "EVENT_MN_REED_RELAY_DEFEATED",
        "EVENT_MN_REED_STPAUL_DEFEATED",
        "EVENT_MN_REED_WORKER_RELEASED",
        "EVENT_MN_REED_MINE_DEFEATED",
        "EVENT_MN_REED_BOUNDARY_DEFEATED",
        "EVENT_MN_KETTLURK_RELEASED",
        "EVENT_MN_SURVEY_RESTORED",
    ] {
        assert!(event(&c, flag), "earned narrative beat {flag}");
    }
    assert_eq!(c.shell.shell.session().state().badges.johto, [true; 8]);
    let (root, r, mut c) = northstar_campaign_league::play_chapter(root, r, c);
    if verify_service_return {
        northstar_earned_return::play(&mut c, &r, &root);
    }
}
