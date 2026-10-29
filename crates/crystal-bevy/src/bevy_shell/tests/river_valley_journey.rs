//! Input-only route traversal after explicit party, position and clock setup.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{finish_text, position, use_super_potion};
use super::river_valley::fixture;

type Point = (i16, i16);

struct Journey<'a> {
    runtime: &'a CrystalRuntime,
    wild_encounters: usize,
    trainer_opponents: BTreeSet<String>,
    attacks_spent_pp: usize,
    medicine_uses: usize,
}

impl Journey<'_> {
    fn settle(&mut self, controller: &mut VisibleShellController) {
        for _ in 0..64 {
            let s = controller.snapshot().expect("visible activity");
            if s.battle.is_some() {
                self.battle(controller);
            } else if s.phase == RuntimeShellPhase::Text
                || controller.has_pending_script_work()
                || controller.shell.field_notice.is_some()
                || controller.shell.visible_field_item_notice.is_some()
                || !controller.shell.battle_messages.is_empty()
            {
                controller.press(GameButton::A).expect("native text page");
                controller
                    .wait_frames(16)
                    .expect("native text presentation");
            } else {
                // This fixture carries supplies from its explicit setup. Use
                // the normal field Pack before continuing on low health.
                if s.party
                    .slots
                    .iter()
                    .any(|p| p.pokemon.hp > 0 && p.pokemon.hp <= p.pokemon.max_hp / 2)
                    && s.bag
                        .items
                        .iter()
                        .any(|item| item.item_id == "SUPER_POTION" && item.quantity > 0)
                {
                    use_super_potion(controller);
                    self.medicine_uses += 1;
                    continue;
                }
                return;
            }
        }
        panic!("route activity failed to settle");
    }

    fn battle(&mut self, controller: &mut VisibleShellController) {
        let initial = controller.snapshot().expect("encounter");
        let wild = matches!(
            initial.battle.as_ref().expect("battle").kind,
            RuntimeBattleKind::Wild { .. }
        );
        self.wild_encounters += usize::from(wild);
        let initial_moves: Vec<_> = initial
            .party
            .slots
            .iter()
            .flat_map(|p| {
                p.pokemon
                    .moves
                    .iter()
                    .map(move |m| (p.index, m.name.clone(), m.current_pp))
            })
            .collect();
        for _ in 0..768 {
            let s = controller.snapshot().expect("production battle");
            let Some(battle) = &s.battle else {
                break;
            };
            if !wild {
                self.trainer_opponents
                    .insert(battle.enemy_pokemon.species.id.clone());
            }
            // Visible narration owns A even when a replacement/move cursor
            // already exists behind it. Do not steer that hidden cursor.
            if !controller.shell.battle_messages.is_empty() || s.ui.menu.is_none() {
                controller
                    .press(GameButton::A)
                    .expect("native battle message");
                controller
                    .wait_frames(16)
                    .expect("native battle narration presentation");
                continue;
            }
            let options =
                s.ui.menu
                    .as_ref()
                    .map(|menu| {
                        menu.layout
                            .vertical_menus
                            .iter()
                            .flat_map(|m| &m.options)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            let button = if options.iter().any(|o| o.contains("change <PKMN>")) {
                GameButton::B
            } else if options.iter().any(|o| o.contains("FIGHT")) {
                let target = if wild { 3 } else { 0 };
                let cursor = controller
                    .shell
                    .battle_action_cursor
                    .as_ref()
                    .expect("action cursor")
                    .option_index;
                if cursor / 2 < target / 2 {
                    GameButton::Down
                } else if cursor / 2 > target / 2 {
                    GameButton::Up
                } else if cursor % 2 < target % 2 {
                    GameButton::Right
                } else if cursor % 2 > target % 2 {
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
                let target = s
                    .party
                    .slots
                    .iter()
                    .position(|p| {
                        p.pokemon.hp > 0 && Some(p.index) != battle.active_player_party_index
                    })
                    .expect("living replacement");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_move_cursor {
                let slot = battle.active_player_party_index.expect("active partner");
                let pokemon = &s.party.slots[slot].pokemon;
                // Read visible learned moves, PP and opposing types rather
                // than limiting Minnesota partners to an old Johto move list.
                let target = super::rochester_earned::best_attack(self.runtime, battle, pokemon);
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
            controller.press(button).expect("production battle input");
            controller
                .wait_frames(16)
                .expect("native battle presentation");
        }
        let ended = controller.snapshot().expect("battle result");
        assert!(ended.battle.is_none(), "battle did not end: {:?}", ended.ui);
        if !wild {
            // Level-up moves can increase the total PP. Compare each original
            // move on the same partner instead of summing the whole party.
            let spent_pp = initial_moves.iter().any(|(slot, name, before)| {
                ended.party.slots.iter().any(|p| {
                    p.index == *slot
                        && p.pokemon
                            .moves
                            .iter()
                            .any(|m| m.name == *name && m.current_pp < *before)
                })
            });
            assert!(
                spent_pp,
                "trainer battle never spent move PP: battle={:?}, initial={initial_moves:?}, ended={:?}",
                initial.battle, ended.party
            );
            self.attacks_spent_pp += 1;
        }
        finish_text(controller);
    }

    fn read(
        &mut self,
        controller: &mut VisibleShellController,
        facing: GameButton,
        expected: &str,
    ) {
        controller.press(facing).expect("face native interaction");
        controller.press(GameButton::A).expect("read or speak");
        let snapshot = controller.snapshot().expect("visible directions");
        let shown = snapshot
            .ui
            .text
            .as_ref()
            .and_then(|t| t.asm_text.as_ref())
            .is_some_and(|text| text.contains(expected));
        // Item presentation can already be on its "put ... in PACK" page.
        // Retain evidence of the actual first page, as the room fixture does;
        // the journey separately checks the native inventory and pickup flags.
        let first_item_page = controller
            .shell
            .visible_field_item_notice
            .as_ref()
            .is_some_and(|notice| notice.sound_trigger_text.contains(expected));
        assert!(
            shown || first_item_page,
            "missing {expected}: {:?}",
            snapshot.ui
        );
        let tile = snapshot.overworld.tile;
        controller
            .press(GameButton::Down)
            .expect("movement while text owns input");
        assert_eq!(
            controller
                .snapshot()
                .expect("dialogue input owner")
                .overworld
                .tile,
            tile
        );
        self.settle(controller);
    }

    fn walk(&mut self, controller: &mut VisibleShellController, target: Point) {
        let start = position(controller);
        assert!(
            start.0 == target.0 || start.1 == target.1,
            "hand-authored cardinal segment"
        );
        let button = if target.0 > start.0 {
            GameButton::Right
        } else if target.0 < start.0 {
            GameButton::Left
        } else if target.1 > start.1 {
            GameButton::Down
        } else {
            GameButton::Up
        };
        for _ in 0..256 {
            if position(controller) == target {
                return;
            }
            controller.press(button).expect("route movement");
            self.settle(controller);
        }
        panic!(
            "blocked hand-authored segment {start:?} -> {target:?}, at {:?}",
            position(controller)
        );
    }
}

#[test]
#[ignore = "requires the external River Valley pack"]
fn river_valley_journey_crosses_both_bridges_habitat_and_all_trainers_with_saved_pp() {
    let (root, runtime, mut controller) = fixture();
    // Normal source-created partners after the second badge. This placement,
    // party, badge setup and replay clock do not prove earned Q02 progression.
    for (species, level) in [("PORTABEAU", 18), ("ZAPSAP", 16)] {
        let pokemon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("source-legal partner");
        assert!(
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(pokemon)
        );
    }
    let state = controller.shell.shell.session_mut().state_mut();
    // Explicit carried supplies, not a claim of earned regional progression.
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["SUPER_POTION"], 6)
            .expect("fixture carried medicine")
    );
    state.badges.johto = [true, true, false, false, false, false, false, false];
    state.sync_party_from_storage();
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let mut journey = Journey {
        runtime: &runtime,
        wild_encounters: 0,
        trainer_opponents: BTreeSet::new(),
        attacks_spent_pp: 0,
        medicine_uses: 0,
    };
    for p in [
        (18, 56),
        (24, 56),
        (24, 54),
        (39, 54),
        (39, 42),
        (30, 42),
        (30, 28),
        (16, 28),
        // Uphill travel goes around the reverse ledge wall through the grass.
        (16, 16),
        (15, 16),
    ] {
        journey.walk(&mut controller, p);
    }
    assert!(
        journey.wild_encounters > 0,
        "habitat produced no native encounter"
    );
    controller.press(GameButton::Down).expect("face Jo");
    controller.press(GameButton::A).expect("challenge Jo");
    journey.settle(&mut controller);
    for p in [(14, 16), (14, 14), (20, 14), (20, 2), (20, 8), (21, 8)] {
        journey.walk(&mut controller, p);
    }
    journey.read(&mut controller, GameButton::Right, "MINNEAPOLIS");
    for p in [
        (20, 8),
        (20, 14),
        // The return uses the dry woodland trail and native southbound hop.
        (11, 14),
        (11, 19),
        (11, 22),
        (14, 22),
        (14, 28),
        (30, 28),
        (30, 42),
        (41, 42),
        (41, 39),
        (39, 39),
        (39, 36),
    ] {
        journey.walk(&mut controller, p);
    }
    journey.read(&mut controller, GameButton::Up, "found");
    journey.walk(&mut controller, (37, 36));
    controller.press(GameButton::Down).expect("face Ed");
    controller.press(GameButton::A).expect("challenge Ed");
    journey.settle(&mut controller);
    for p in [(39, 36), (39, 39), (37, 39), (37, 42)] {
        journey.walk(&mut controller, p);
    }
    for _ in 0..4 {
        controller
            .press(GameButton::Up)
            .expect("try terrace reverse");
    }
    assert_eq!(position(&mut controller), (37, 42));
    for p in [(38, 42), (38, 48), (37, 48)] {
        journey.walk(&mut controller, p);
    }
    controller.press(GameButton::Down).expect("face Cal");
    controller.press(GameButton::A).expect("challenge Cal");
    journey.settle(&mut controller);
    for p in [(39, 48), (39, 54), (39, 55), (40, 55)] {
        journey.walk(&mut controller, p);
    }
    journey.read(&mut controller, GameButton::Right, "found");
    if controller
        .snapshot()
        .expect("pickup party")
        .party
        .slots
        .iter()
        .any(|p| p.pokemon.hp > 0 && p.pokemon.hp < p.pokemon.max_hp)
    {
        use_super_potion(&mut controller);
        journey.medicine_uses += 1;
    }
    assert!(
        journey.medicine_uses > 0,
        "journey never used carried medicine"
    );
    for p in [(39, 55), (39, 54), (24, 54), (24, 56), (18, 56), (18, 58)] {
        journey.walk(&mut controller, p);
    }
    assert_eq!(journey.attacks_spent_pp, 4);
    for species in [
        "GILLPUP", "CHORIP", "PODLET", "PECKIT", "POUCHID", "ZAPSAP", "TICKLET",
    ] {
        assert!(
            journey.trainer_opponents.contains(species),
            "missing {species}"
        );
    }
    let flags = [
        "EVENT_MN_R03_TRAINER_01",
        "EVENT_MN_R03_TRAINER_02",
        "EVENT_MN_R03_TRAINER_03",
        "EVENT_MN_R03_TRAINER_04",
        "EVENT_MN_R03_ITEM_04",
        "EVENT_MN_R03_ITEM_01",
    ];
    for f in flags {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(f)
                .expect("journey flag"),
            "{f}"
        );
    }
    let before = controller.snapshot().expect("journey party");
    assert!(
        before
            .bag
            .items
            .iter()
            .any(|s| s.item_id == "ETHER" && s.quantity == 1)
    );
    let wild_encounters = journey.wild_encounters;
    let save = std::env::temp_dir().join(format!(
        "geothite-river-valley-journey-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save journey");
    let mut loaded =
        VisibleShellController::load_save(root, runtime, save.clone(), None).expect("Continue");
    assert_eq!(position(&mut loaded), (18, 58));
    let after = loaded.snapshot().expect("continued journey");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    for f in flags {
        assert!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(f)
                .expect("continued flag")
        );
    }
    if let Some(path) = std::env::var_os("CRYSTAL_RIVER_VALLEY_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export native journey");
    }
    std::fs::remove_file(save).expect("remove scratch save");
    eprintln!(
        "River Valley: {} wild encounters, both bridge crossings, four trainers/seven opponents defeated with spent PP, Ether, used Super Potion and party/flags survive Continue",
        wild_encounters
    );
}
