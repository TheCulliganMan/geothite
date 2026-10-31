//! Explicit optional-chapter fixtures, followed only by production input.
use std::collections::BTreeMap;

use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec};
use super::northstar_ice_lair::{choice, event, page, save, talk};
use super::winona::{at, finish, load, press, travel};

const MAP: &str = "MnR08C00R00";
const READY: &str = "EVENT_MN_ANCIENT_MIRE_TRACKS_READ";
const CAUGHT: &str = "EVENT_MN_ANCIENT_MIRE_VERGLOOM_CAUGHT";
const ACTOR: &str = "MN_ANCIENT_MIRE_VERGLOOM";
const CLUES: [&str; 3] = [
    "EVENT_MN_ANCIENT_MIRE_TIMBER",
    "EVENT_MN_ANCIENT_MIRE_FOOTPRINT",
    "EVENT_MN_ANCIENT_MIRE_MOSS",
];

fn setup(
    ready: bool,
    weak: bool,
    ball: bool,
) -> (
    crystal_assets::AssetRoot,
    CrystalRuntime,
    VisibleShellController,
) {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("HOME"),
            map_name: MAP.into(),
            tile_x: if ready { 59 } else { 45 },
            tile_y: if ready { 24 } else { 37 },
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared optional chapter fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    let mut p = r
        .data()
        .create_pokemon(
            if weak { "CROOKIT" } else { "BOREALYNX" },
            if weak { 1 } else { 60 },
            Dv::from_non_hp(8, 9, 8, 9),
        )
        .expect("active Minnesota fixture partner");
    if !weak {
        p.item = Some("SMOKE_BALL".into());
        assert!(p.species.tmhm_learnset.iter().any(|m| m == "HEADBUTT"));
        p.moves[3] = LearnedMove {
            name: "HEADBUTT".into(),
            current_pp: r.data().moves["HEADBUTT"].pp,
            pp_ups: 0,
        };
    }
    assert!(state.storage.party.add_pokemon(p));
    if !weak && !ball {
        let mut reserve = r
            .data()
            .create_pokemon("GOOSNIDE", 60, Dv::from_non_hp(8, 9, 8, 9))
            .expect("declared Minnesota knockout-fixture reserve");
        reserve.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(reserve));
    }
    state.sync_party_from_storage();
    state.last_spawn_map_constant = Some("MN_ROCHESTER_POKECENTER_1F".into());
    // The one Master Ball is an explicit capture-fixture input, never an earned
    // campaign claim. Its actual native consumption and capture are verified.
    if ball {
        state
            .bag
            .add_item(&r.data().items["MASTER_BALL"], 1)
            .expect("declared capture item");
    }
    if ready {
        for flag in CLUES.into_iter().chain([READY]) {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared prior observations");
        }
    }
    let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
    r.data()
        .apply_map_setup_callbacks(state, overworld, MAP, "MAPSETUP_WARP")
        .expect("native entry object roster");
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("fixture presentation");
    // No state, party, item, flag, position or RNG writes below this boundary.
    (root, r, c)
}

fn visible(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("native creature visibility")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(ACTOR))
}
fn settle(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if let Some(b) = c.snapshot().expect("walking encounter").battle {
        match b.kind {
            RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. } => {
                super::rochester::flee(c)
            }
            RuntimeBattleKind::Trainer { trainer_id, .. } => {
                let flag = r
                    .data()
                    .maps
                    .values()
                    .flat_map(|m| m.trainer_scripts.values())
                    .find(|t| t.trainer_id == trainer_id)
                    .expect("actual trainer event")
                    .event_flag
                    .clone();
                assert!(super::rochester_earned::try_battle(
                    c,
                    r,
                    &trainer_id,
                    &flag
                ));
            }
        }
    }
    if c.snapshot().expect("walking page").ui.text_window_open || c.has_pending_script_work() {
        finish(c);
    }
}
fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR08",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start: (45, 37),
            doors: BTreeMap::new(),
        },
    );
    let s = c.snapshot().expect("mire trail");
    assert_eq!(s.overworld.map_name, MAP);
    for button in grid.path((s.overworld.tile.x, s.overworld.tile.y), to) {
        let old = c.snapshot().expect("before step").overworld.tile;
        for _ in 0..8 {
            press(c, button);
            settle(c, r);
            if c.snapshot().expect("native step").overworld.tile != old {
                break;
            }
        }
        assert_ne!(c.snapshot().expect("actual movement").overworld.tile, old);
    }
    at(c, MAP, to);
}
fn reenter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (45, 47));
    travel(c, GameButton::Down, "MnR08C00R01", (45, 0));
    settle(c, r);
    travel(c, GameButton::Up, MAP, (45, 47));
    settle(c, r);
    walk(c, r, (59, 24));
}
fn encounter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let party = c
        .snapshot()
        .expect("native scaling party")
        .party
        .slots
        .iter()
        .map(|p| p.pokemon.clone())
        .collect::<Vec<_>>();
    let expected_level = r
        .data()
        .wild_challenge
        .map_or(40, |rules| rules.encounter_level(&party, 40, 2));
    talk(c, GameButton::Up);
    choice(c, true);
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("legendary encounter");
        if let Some(b) = snapshot.battle {
            assert!(
                matches!(&b.kind, RuntimeBattleKind::StaticWild { source_script, species, level, .. } if source_script == ACTOR && species == "VERGLOOM" && *level == expected_level)
            );
            assert_eq!(b.enemy_pokemon.species.id, "VERGLOOM");
            assert_eq!(b.enemy_pokemon.level, expected_level);
            let options = snapshot
                .ui
                .menu
                .as_ref()
                .map(|m| {
                    m.layout
                        .vertical_menus
                        .iter()
                        .flat_map(|v| v.options.iter())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if options.iter().any(|o| o.contains("FIGHT")) {
                for action in ["FIGHT", "PKMN", "PACK", "RUN"] {
                    assert!(
                        options.iter().any(|o| o.contains(action)),
                        "missing production battle command {action}"
                    );
                }
                return;
            }
        }
        press(c, GameButton::A);
    }
    panic!("native optional encounter did not begin");
}

#[test]
#[ignore = "requires regional ancient-mire pack and isolated Save/Continue directory"]
fn ancient_swamp_three_real_clues_refusal_flee_reentry_capture_and_continue() {
    let (root, r, mut c) = setup(false, false, true);
    assert!(!event(&c, READY) && !event(&c, CAUGHT));
    walk(&mut c, &r, (59, 24));
    assert!(!visible(&mut c));
    let observations = [
        ((48, 33), GameButton::Left, "MnAncientTimberText"),
        ((56, 29), GameButton::Left, "MnAncientFootprintText"),
        ((59, 25), GameButton::Right, "MnAncientMossText"),
    ];
    for (i, (tile, direction, label)) in observations.into_iter().enumerate() {
        walk(&mut c, &r, tile);
        talk(&mut c, direction);
        page(&mut c, label);
        assert!(
            !event(&c, CLUES[i]),
            "unacknowledged text cannot grant an observation"
        );
        if i < 2 {
            finish(&mut c);
            assert!(event(&c, CLUES[i]) && !event(&c, READY));
            c = save(&mut c, &root, &r, &format!("mire-clue-{i}"));
        } else {
            page(&mut c, "MnAncientRevealedText");
            assert!(!event(&c, READY));
            finish(&mut c);
        }
    }
    assert!(event(&c, READY) && !event(&c, CAUGHT));
    walk(&mut c, &r, (59, 24));
    assert!(visible(&mut c));
    c = save(&mut c, &root, &r, "mire-revealed");
    assert!(visible(&mut c));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, CAUGHT) && visible(&mut c));
    encounter(&mut c, &r);
    super::rochester::flee(&mut c);
    assert!(!event(&c, CAUGHT));
    c = save(&mut c, &root, &r, "mire-fled");
    assert!(!visible(&mut c));
    reenter(&mut c, &r);
    assert!(visible(&mut c));
    encounter(&mut c, &r);
    assert!(super::rochester_earned::try_recruit_with_ball(
        &mut c,
        &r,
        "MASTER_BALL"
    ));
    finish(&mut c);
    assert!(event(&c, CAUGHT) && !visible(&mut c));
    let s = c.snapshot().expect("native capture result");
    assert_eq!(s.party.slots.len(), 2);
    assert!(
        s.party
            .slots
            .iter()
            .any(|p| p.pokemon.species.id == "VERGLOOM")
    );
    assert!(
        s.bag
            .balls
            .iter()
            .all(|b| b.item_id != "MASTER_BALL" || b.quantity == 0)
    );
    c = save(&mut c, &root, &r, "mire-captured");
    reenter(&mut c, &r);
    assert!(event(&c, CAUGHT) && !visible(&mut c));
    assert!(!event(&c, "EVENT_MN_SURVEY_RESTORED"));
}

#[test]
#[ignore = "requires regional ancient-mire pack and isolated Save/Continue directory"]
fn ancient_swamp_knockout_and_native_loss_do_not_consume_capture() {
    let (root, r, mut c) = setup(true, false, false);
    encounter(&mut c, &r);
    let before = c.snapshot().expect("initial native PP").party.slots[0]
        .pokemon
        .moves
        .clone();
    assert!(super::rochester_earned::try_battle(&mut c, &r, "WILD", ""));
    finish(&mut c);
    let after = c.snapshot().expect("actual native PP spending");
    assert!(after.party.slots[0].pokemon.moves.iter().any(|m| {
        before
            .iter()
            .any(|old| old.name == m.name && m.current_pp < old.current_pp)
    }));
    assert!(!event(&c, CAUGHT) && event(&c, READY) && !visible(&mut c));
    c = save(&mut c, &root, &r, "mire-knockout");
    reenter(&mut c, &r);
    assert!(visible(&mut c) && !event(&c, CAUGHT));
    let (root, r, mut weak) = setup(true, true, false);
    encounter(&mut weak, &r);
    assert!(!super::rochester_earned::try_battle(
        &mut weak, &r, "WILD", ""
    ));
    assert!(!event(&weak, CAUGHT) && event(&weak, READY));
    let continued = save(&mut weak, &root, &r, "mire-native-loss");
    assert!(!event(&continued, CAUGHT));
}
