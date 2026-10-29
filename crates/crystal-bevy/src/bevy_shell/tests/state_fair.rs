//! Explicit fair placement/party/money fixtures; play uses the production controller.
#[path = "state_fair_crowd.rs"]
mod crowd;
#[path = "state_fair_food.rs"]
mod food;
#[path = "state_fair_rooms.rs"]
mod rooms;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::{ITEM_POCKET_ITEM, is_mail_item_id};
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::super::{
    BevyShellConfig, BevyShellStart, FieldPackAction, FieldPackPocket, StartMenuOption,
    VisibleShellController, complete_visible_smoke_player_name_if_needed,
    initialize_bevy_runtime_shell, mark_runtime_snapshot_dirty, visible_selected_pack_item_actions,
    visible_start_menu_options,
};
use super::{Point, at, battle, continue_save, event, finish, module, press};

const FAIR: &str = "MnStateFairgrounds";
const STAMPS: [&str; 3] = [
    "EVENT_MN_FAIR_BARN_STAMP",
    "EVENT_MN_FAIR_DAIRY_STAMP",
    "EVENT_MN_FAIR_MIDWAY_STAMP",
];

fn fixture(full_bag: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    fixture_options(full_bag, (16, 68), false, true)
}

fn flee(c: &mut VisibleShellController, attack: bool) -> bool {
    let pp = |c: &VisibleShellController| {
        c.shell
            .shell
            .session()
            .state()
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .flat_map(|p| &p.moves)
            .map(|m| u32::from(m.current_pp))
            .sum::<u32>()
    };
    let before = pp(c);
    for _ in 0..256 {
        let s = c.snapshot().expect("wild battle");
        if s.battle.is_none() {
            finish(c);
            assert_eq!(
                c.snapshot().expect("escaped habitat").overworld.map_name,
                FAIR
            );
            return pp(c) < before;
        }
        let input = if s.ui.menu.is_none() && s.ui.text.is_some() {
            GameButton::A
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            let target = if attack && pp(c) == before { 0 } else { 3 };
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
        press(c, input);
    }
    panic!("native RUN failed to settle");
}

#[test]
#[ignore = "requires the published regional-species fair pack"]
fn state_fair_grass_encounters_all_29_food_and_carnival_species() {
    use std::collections::BTreeSet;
    let mut total = BTreeSet::new();
    // Position, healthy partner and deterministic DIV are explicit setup.
    // Every encounter, battle command and step after setup uses the controller.
    for id in [
        "growers",
        "cookie-side",
        "midway-meadow",
        "cheese",
        "orchard-east",
        "smokehouse",
    ] {
        let (_, r, _) = fixture(false);
        let zone = r.data().wild_encounters[FAIR]
            .zones
            .iter()
            .find(|zone| zone.id == id)
            .expect("authored habitat")
            .clone();
        let expected = zone
            .grass
            .day
            .iter()
            .map(|slot| slot.species.clone())
            .collect::<BTreeSet<_>>();
        let spawn = (zone.min_x as u16, zone.min_y as u16);
        let (_, _, mut c) = fixture_at(false, spawn, true);
        let mut seen = BTreeSet::new();
        let mut won = false;
        for _ in 0..12000 {
            let s = c.snapshot().expect("habitat walk");
            if let Some(b) = &s.battle {
                assert!(matches!(
                    b.kind,
                    crystal_runtime::RuntimeBattleKind::Wild { .. }
                ));
                let species = b.enemy_pokemon.species.id.clone();
                assert!(expected.contains(&species), "{id} unexpected {species}");
                seen.insert(species);
                if won {
                    flee(&mut c, false);
                } else {
                    won = flee(&mut c, true);
                }
                if seen == expected && won {
                    break;
                }
            } else {
                let input = if at(&mut c).0 == spawn.0 {
                    GameButton::Right
                } else {
                    GameButton::Left
                };
                press(&mut c, input);
            }
        }
        assert_eq!(
            seen, expected,
            "all forms must appear through grass input in {id}"
        );
        assert!(won, "each habitat must execute an actual move and spend PP");
        eprintln!("Verified native habitat {id}: {seen:?}");
        total.extend(seen);
    }
    assert_eq!(total.len(), 29);
}

fn fixture_at(
    full_bag: bool,
    spawn: Point,
    healthy: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    fixture_options(full_bag, spawn, healthy, false)
}

fn fixture_options(
    full_bag: bool,
    spawn: Point,
    healthy: bool,
    quiet_crowd: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    let (root, r) = LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_FAIR_TEST_PACK").expect("published native fair pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let r = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified fair pack"),
            )
            .expect("fair runtime");
            (root, r)
        })
        .clone();
    let mut c = VisibleShellController {
        shell: initialize_bevy_runtime_shell(
            root.clone(),
            r.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: r.title_new_game_spawn_identifier().expect("original HOME"),
                map_name: FAIR.into(),
                tile_x: spawn.0 as i16,
                tile_y: spawn.1 as i16,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("explicit local fair placement"),
    };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    let mut pokemon = r
        .data()
        .create_pokemon("POUCHID", 100, Dv::default())
        .expect("active Minnesota review partner");
    if !healthy {
        pokemon.hp = 1;
    } else {
        // The inherited Wild Challenge scales opponents to this review party.
        // A legal held Smoke Ball lets the encounter census leave each fight
        // through native RUN without accumulating failed-escape damage.
        pokemon.item = Some("SMOKE_BALL".into());
    }
    let moves = if healthy {
        ["PROTECT", "GIGA_DRAIN", "PSYCHIC_M", "REST"]
    } else {
        ["PSYCHIC_M", "GIGA_DRAIN", "SOLARBEAM", "REST"]
    };
    for name in moves {
        assert!(
            r.data().pokemon["POUCHID"]
                .tmhm_learnset
                .iter()
                .any(|m| m == name),
            "the active regional fixture partner must legally learn {name}"
        );
    }
    pokemon.moves = moves
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: r.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let s = c.shell.shell.session_mut().state_mut();
    assert!(s.storage.party.add_pokemon(pokemon));
    s.sync_party_from_storage();
    s.money = 100000;
    if quiet_crowd {
        // Service-only cases start after the twenty added fair battles.
        // Crowd cases below independently exercise every undefeated trainer.
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../planning/minnesota/zones/state-fairgrounds.json"
        ))
        .expect("fair source");
        for trainer in source["native_trainers"].as_array().expect("fair trainers") {
            let flag = trainer["defeated_flag"].as_str().expect("trainer flag");
            if ![
                "EVENT_MN_FAIR_RING_DEFEATED",
                "EVENT_MN_FAIR_JUNIOR_DEFEATED",
            ]
            .contains(&flag)
            {
                s.flags
                    .set_event_flag(flag, true)
                    .expect("explicit prior-battle fixture");
            }
        }
    }
    if healthy {
        s.random_state = Default::default();
    }
    if full_bag {
        assert!(
            s.bag
                .add_item(&r.data().items["POTION"], 1)
                .expect("tossable fixture medicine")
        );
        for (_, item) in r
            .data()
            .items
            .iter()
            .filter(|(id, i)| {
                i.pocket == ITEM_POCKET_ITEM
                    && ![
                        "POTION",
                        "RARE_CANDY",
                        "FAIR_CORN_STICK",
                        "FAIR_COOKIE_PAIL",
                    ]
                    .contains(&id.as_str())
                    && !is_mail_item_id(id)
            })
            .take(ITEM_POCKET_CAPACITY - 1)
        {
            assert!(s.bag.add_item(item, 1).expect("legal fixture filler"));
        }
        assert_eq!(s.bag.items.len(), ITEM_POCKET_CAPACITY);
        for flag in STAMPS {
            s.flags
                .set_event_flag(flag, true)
                .expect("explicit earned-stamp fixture for bag recovery only");
        }
    }
    c.set_runtime_journal_enabled(false);
    if healthy {
        *c.shell.shell.session_mut().divider_mut_for_tests() =
            crystal_core::random::RuntimeDividerSource::replay(
                (0..1_048_576)
                    .scan(0x9e3779b9_u32, |state, _| {
                        *state ^= *state << 13;
                        *state ^= *state >> 17;
                        *state ^= *state << 5;
                        Some((*state >> 16) as u8)
                    })
                    .collect::<Vec<_>>(),
            );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No position, inventory, HP, party, money or flag writes after setup.
    (root, r, c)
}

fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: Point) {
    let mut m = module(c, r);
    let actor_count = m.objects.len();
    // Service errands use the clear streets. Habitat walking has its own test.
    for y in 0..m.attributes.height * 2 {
        for x in 0..m.attributes.width * 2 {
            let b =
                m.blocks[usize::from(y / 2) * usize::from(m.attributes.width) + usize::from(x / 2)];
            if r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")]
                [usize::from(y % 2 * 2 + x % 2)]
                == "TALL_GRASS"
            {
                let mut body = r.data().maps["NewBarkTown"].objects[0].clone();
                body.x = x;
                body.y = y;
                m.objects.push(body);
            }
        }
    }
    let from = at(c);
    for w in m.events.warps.clone() {
        if (w.x, w.y) == to || (w.x, w.y) == from {
            continue;
        }
        let b =
            m.blocks[usize::from(w.y / 2) * usize::from(m.attributes.width) + usize::from(w.x / 2)];
        let kind = &r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")]
            [usize::from(w.y % 2 * 2 + w.x % 2)];
        if kind == "FLOOR" {
            continue;
        }
        let mut body = r.data().maps["NewBarkTown"].objects[0].clone();
        body.x = w.x;
        body.y = w.y;
        m.objects.push(body);
    }
    for _ in 0..128 {
        let from = at(c);
        if from == to {
            return;
        }
        for actor in &mut m.objects[..actor_count] {
            if let Some(id) = &actor.object_identifier
                && let Ok(tile) = c
                    .shell
                    .shell
                    .session()
                    .overworld()
                    .object_runtime_tile_by_id(id)
            {
                actor.x = tile.x as u16;
                actor.y = tile.y as u16;
            }
        }
        let Some(route) = super::path(r, &m, from, to) else {
            c.wait_frames(120).expect("let fair walkers pass");
            continue;
        };
        for button in route {
            let before = at(c);
            let expected = match button {
                GameButton::Up => (before.0, before.1.saturating_sub(1)),
                GameButton::Down => (before.0, before.1 + 1),
                GameButton::Left => (before.0.saturating_sub(1), before.1),
                GameButton::Right => (before.0 + 1, before.1),
                _ => unreachable!("native walk paths contain cardinal input"),
            };
            for _ in 0..8 {
                press(c, button);
                if at(c) != before {
                    break;
                }
            }
            assert_eq!(c.snapshot().expect("walking").overworld.map_name, m.id);
            // A walker can enter this route after it was planned. Never run
            // its remaining directions from the wrong tile: replan using the
            // live actor positions, with all movement still real input.
            if at(c) != expected {
                c.wait_frames(24).expect("changed route settles");
                break;
            }
        }
    }
    assert_eq!(at(c), to, "fair walk exhausted live route replans");
}

fn counter(c: &mut VisibleShellController, r: &CrystalRuntime, approach: Point) {
    walk(c, r, approach);
    for _ in 0..3 {
        press(c, GameButton::Up);
        assert_eq!(at(c), approach, "native counter blocks walking");
    }
    press(c, GameButton::A);
}
fn quantity(c: &VisibleShellController, r: &CrystalRuntime, id: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[id])
}
fn close_shop(c: &mut VisibleShellController) {
    for _ in 0..96 {
        let s = c.snapshot().expect("shop close");
        if s.pending_shop.is_none()
            && !s.ui.text_window_open
            && s.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            return;
        }
        let b = if s.pending_shop.is_none()
            || c.shell.shop_notice.is_some()
            || !c.shell.shop_welcome_seen
        {
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
        press(c, b);
    }
    panic!(
        "fair shop did not release input: {:#?}",
        c.snapshot().expect("shop diagnostic")
    );
}
fn buy(c: &mut VisibleShellController, r: &CrystalRuntime, approach: Point, id: &str) {
    counter(c, r, approach);
    for _ in 0..64 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let target = c
        .snapshot()
        .expect("shop")
        .pending_shop
        .expect("native mart")
        .inventory
        .iter()
        .position(|item| item == id)
        .expect("vendor stocks food");
    let money = c.shell.shell.session().state().money;
    let count = quantity(c, r, id);
    press(c, GameButton::A);
    cursor(c, target, |c| {
        c.shell
            .menu_cursor
            .as_ref()
            .expect("shop buy cursor")
            .option_index
    });
    press(c, GameButton::A);
    assert_eq!(
        c.shell.shop_quantity.as_ref().expect("quantity").item_id,
        id
    );
    press(c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        money,
        "unconfirmed purchase costs nothing"
    );
    press(c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        money - u32::from(r.data().items[id].price)
    );
    assert_eq!(quantity(c, r, id), count + 1);
    close_shop(c);
}
fn cursor(
    c: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..32 {
        let i = index(c);
        if i == target {
            return;
        }
        press(
            c,
            if i < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("fair menu cursor did not reach {target}");
}
fn eat(c: &mut VisibleShellController, r: &CrystalRuntime, id: &str, healing: u16) {
    let before = c.snapshot().expect("before food").party.slots[0].pokemon.hp;
    let carried = quantity(c, r, id);
    press(c, GameButton::Start);
    let s = c.snapshot().expect("Start");
    let target = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    cursor(c, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    press(c, GameButton::A);
    let target = c
        .snapshot()
        .expect("bag")
        .bag
        .items
        .iter()
        .filter(|i| i.quantity > 0)
        .position(|i| i.item_id == id)
        .expect("purchased food");
    cursor(c, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("bag cursor")
            .option_index
    });
    press(c, GameButton::A);
    let s = c.snapshot().expect("food action");
    let target = visible_selected_pack_item_actions(&s, &c.shell, &FieldPackPocket::Items, false)
        .expect("native actions")
        .iter()
        .position(|a| *a == FieldPackAction::Use)
        .expect("USE");
    cursor(c, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action")
            .option_index
    });
    press(c, GameButton::A);
    cursor(c, 0, |c| c.shell.party_cursor);
    c.wait_frames(120).expect("target presentation");
    press(c, GameButton::A);
    assert_eq!(
        c.snapshot().expect("food effect").party.slots[0].pokemon.hp,
        before + healing
    );
    assert_eq!(quantity(c, r, id), carried - 1, "food is consumed");
    for _ in 0..64 {
        if c.shell.field_notice.is_some()
            || c.snapshot().expect("notice").ui.pending_text_wait.is_some()
        {
            c.wait_frames(120).expect("notice clock");
            press(c, GameButton::A);
        } else if c.shell.field_pack_target_mode.is_some()
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.bag_cursor.is_some()
            || c.shell.field_pack_pocket.is_some()
            || c.shell.start_menu_cursor.is_some()
        {
            press(c, GameButton::B);
        } else {
            return;
        }
    }
    panic!("food menus did not close");
}
fn enter(c: &mut VisibleShellController, r: &CrystalRuntime, approach: Point, room: &str) {
    walk(c, r, approach);
    for _ in 0..4 {
        press(c, GameButton::Up);
        if c.snapshot().expect("door").overworld.map_name == room {
            return;
        }
    }
    panic!("fair door failed: {room}");
}
fn leave(c: &mut VisibleShellController, r: &CrystalRuntime, approach: Point) {
    walk(c, r, approach);
    for _ in 0..4 {
        press(c, GameButton::Down);
        if c.snapshot().expect("carpet").overworld.map_name == FAIR {
            return;
        }
    }
    panic!("fair room return failed");
}
fn quiz(c: &mut VisibleShellController, r: &CrystalRuntime, yes: bool) {
    counter(c, r, (9, 26));
    read_question(c);
    cursor(c, usize::from(!yes), |c| {
        c.shell
            .yes_no_cursor
            .as_ref()
            .expect("quiz YES/NO")
            .option_index
    });
    press(c, GameButton::A);
    finish(c);
}

fn read_question(c: &mut VisibleShellController) {
    for _ in 0..32 {
        c.wait_frames(240).expect("print native invitation page");
        let raw = c
            .shell
            .shell
            .presentation_snapshot()
            .expect("native text stream");
        if let Some(pages) = super::super::super::visible_field_dialog_pages(&raw, &c.shell)
            && let Some(reveal) = &c.shell.field_text_reveal
            && reveal.page_index + 1 == pages.len()
            && super::super::super::visible_field_text_reveal_is_complete(
                reveal,
                &pages[reveal.page_index],
            )
            && c.shell.yes_no_cursor.is_some()
        {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("native invitation did not expose its final YES/NO page");
}

fn toss(c: &mut VisibleShellController, r: &CrystalRuntime, cancel: bool) {
    counter(c, r, (17, 26));
    read_question(c);
    assert!(c.shell.yes_no_cursor.is_some(), "free toss must ask first");
    press(c, if cancel { GameButton::B } else { GameButton::A });
    finish(c);
}

#[test]
#[ignore = "requires the external native State Fair pack"]
fn state_fair_controller_counters_food_stamps_rooms_battle_and_continue() {
    let (root, r, mut c) = fixture(false);
    // Walk both actual woodland lanes to the existing road and back.
    for (fair_y, road_y) in [(64, 8), (65, 9)] {
        walk(&mut c, &r, (95, fair_y));
        for _ in 0..4 {
            press(&mut c, GameButton::Right);
            if c.snapshot().expect("road seam").overworld.map_name != FAIR {
                break;
            }
        }
        assert_eq!(
            c.snapshot().expect("road").overworld.map_name,
            "MnR05C00R02"
        );
        assert_eq!(at(&mut c), (0, road_y));
        walk(&mut c, &r, (19, road_y));
        walk(&mut c, &r, (0, road_y));
        for _ in 0..4 {
            press(&mut c, GameButton::Left);
            if c.snapshot().expect("fair seam").overworld.map_name == FAIR {
                break;
            }
        }
        assert_eq!(at(&mut c), (95, fair_y));
    }
    assert_eq!(c.snapshot().expect("fair").overworld.map_name, FAIR);
    counter(&mut c, &r, (21, 66));
    let stand = at(&mut c);
    press(&mut c, GameButton::Left);
    assert_eq!(at(&mut c), stand, "prize dialogue owns movement");
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 0);
    counter(&mut c, &r, (55, 12));
    close_shop(&mut c);
    assert_eq!(
        c.shell.shell.session().state().money,
        100000,
        "leaving the shop costs nothing"
    );
    assert_eq!(quantity(&c, &r, "FAIR_COOKIE_PAIL"), 0);
    buy(&mut c, &r, (55, 12), "FAIR_COOKIE_PAIL");
    eat(&mut c, &r, "FAIR_COOKIE_PAIL", 50);
    buy(&mut c, &r, (59, 12), "FAIR_CORN_STICK");
    eat(&mut c, &r, "FAIR_CORN_STICK", 20);
    buy(&mut c, &r, (57, 24), "LEMONADE");
    buy(&mut c, &r, (65, 24), "MOOMOO_MILK");
    enter(&mut c, &r, (7, 52), "MnFairBarn");
    super::interact(&mut c, &r, (5, 3));
    finish(&mut c);
    assert!(event(&c, STAMPS[0]));
    leave(&mut c, &r, (4, 6));
    assert_eq!(at(&mut c), (7, 52));
    enter(&mut c, &r, (75, 52), "MnFairDairy");
    counter(&mut c, &r, (3, 3));
    finish(&mut c);
    assert!(event(&c, STAMPS[1]));
    leave(&mut c, &r, (4, 6));
    assert_eq!(at(&mut c), (75, 52));
    enter(&mut c, &r, (11, 18), "MnFairGrandstand");
    super::interact(&mut c, &r, (5, 9));
    finish(&mut c);
    leave(&mut c, &r, (6, 12));
    quiz(&mut c, &r, false);
    assert!(!event(&c, STAMPS[2]));
    quiz(&mut c, &r, true);
    assert!(event(&c, STAMPS[2]));
    counter(&mut c, &r, (21, 66));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_FAIR_PRIZE_CLAIMED"));
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
    counter(&mut c, &r, (21, 66));
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
    walk(&mut c, &r, (31, 52));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_FAIR_RING_DEFEATED"));
    walk(&mut c, &r, (35, 52));
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_FAIR_JUNIOR_DEFEATED"));
    counter(&mut c, &r, (37, 64));
    read_question(&mut c);
    cursor(&mut c, 1, |c| {
        c.shell
            .yes_no_cursor
            .as_ref()
            .expect("care answer")
            .option_index
    });
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_FAIR_CARE_PRIZE"));
    let berries = quantity(&c, &r, "BERRY");
    counter(&mut c, &r, (37, 64));
    read_question(&mut c);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_FAIR_CARE_PRIZE"));
    assert_eq!(quantity(&c, &r, "BERRY"), berries + 1);
    counter(&mut c, &r, (37, 64));
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "BERRY"), berries + 1);
    counter(&mut c, &r, (79, 60));
    finish(&mut c);
    let healed = c.snapshot().expect("native first aid").party.slots[0]
        .pokemon
        .clone();
    assert_eq!(healed.hp, healed.max_hp);
    for learned in healed.moves {
        assert_eq!(learned.current_pp, r.data().moves[&learned.name].pp);
    }
    c = continue_save(&mut c, &root, &r);
    assert!(event(&c, "EVENT_MN_FAIR_JUNIOR_DEFEATED"));
    assert!(event(&c, "EVENT_MN_FAIR_CARE_PRIZE"));
    for flag in STAMPS {
        assert!(event(&c, flag));
    }
    assert!(event(&c, "EVENT_MN_FAIR_RING_DEFEATED"));
    assert!(event(&c, "EVENT_MN_FAIR_PRIZE_CLAIMED"));
}

#[test]
#[ignore = "requires the external native State Fair pack"]
fn state_fair_controller_full_bag_preserves_stamp_prize_then_recovers() {
    let (root, r, mut c) = fixture(true);
    counter(&mut c, &r, (21, 66));
    finish(&mut c);
    assert!(!event(&c, "EVENT_MN_FAIR_PRIZE_CLAIMED"));
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 0);
    toss(&mut c, &r, true);
    assert!(!event(&c, "EVENT_MN_FAIR_GAME_PENDING"));
    assert!(!event(&c, "EVENT_MN_FAIR_GAME_PRIZE"));
    for _ in 0..32 {
        toss(&mut c, &r, false);
        if event(&c, "EVENT_MN_FAIR_GAME_PENDING") {
            break;
        }
    }
    assert!(event(&c, "EVENT_MN_FAIR_GAME_PENDING"));
    assert!(!event(&c, "EVENT_MN_FAIR_GAME_PRIZE"));
    assert_eq!(quantity(&c, &r, "FAIR_CORN_STICK"), 0);
    assert_eq!(
        c.shell.shell.session().state().money,
        100000,
        "practice tosses are free"
    );
    eat(&mut c, &r, "POTION", 20);
    counter(&mut c, &r, (17, 26));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_FAIR_GAME_PRIZE"));
    assert!(!event(&c, "EVENT_MN_FAIR_GAME_PENDING"));
    assert_eq!(quantity(&c, &r, "FAIR_CORN_STICK"), 1);
    counter(&mut c, &r, (17, 26));
    finish(&mut c);
    assert_eq!(
        quantity(&c, &r, "FAIR_CORN_STICK"),
        1,
        "once-only game prize"
    );
    eat(&mut c, &r, "FAIR_CORN_STICK", 20);
    counter(&mut c, &r, (21, 66));
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_FAIR_PRIZE_CLAIMED"));
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
    c = continue_save(&mut c, &root, &r);
    counter(&mut c, &r, (21, 66));
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "RARE_CANDY"), 1);
}

#[test]
#[ignore = "requires the external expanded State Fair pack"]
fn state_fair_controller_every_expanded_shop_and_all_sixteen_food_effects() {
    let (_, r, mut c) = fixture(false);
    let path = std::env::var_os("CRYSTAL_FAIR_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/zones/state-fairgrounds.json")
        });
    let source: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("expanded fair source"))
            .expect("dossier");
    let merchants = source["native_map"]["residents"]
        .as_array()
        .expect("NPCs")
        .iter()
        .filter(|a| a["service"] == "pokemart")
        .collect::<Vec<_>>();
    assert_eq!(merchants.len(), 32);
    assert_eq!(source["crowd_contract"]["total_outdoor_npcs"], 255);
    assert_eq!(r.data().maps[FAIR].objects.len(), 255);
    for a in &merchants {
        let pos = super::point(&a["approach"]);
        counter(&mut c, &r, pos);
        for _ in 0..64 {
            if c.shell.shop_top_cursor.is_some()
                && c.shell.shop_welcome_seen
                && c.shell.shop_notice.is_none()
            {
                break;
            }
            press(&mut c, GameButton::A);
        }
        let expected = a["stock"]
            .as_array()
            .expect("merchant stock")
            .iter()
            .map(|s| s.as_str().expect("item").to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            c.snapshot()
                .expect("expanded shop")
                .pending_shop
                .expect("actual native mart")
                .inventory,
            expected
        );
        close_shop(&mut c);
    }
    assert_eq!(
        c.shell.shell.session().state().money,
        100000,
        "all merchant visits were cancelled without charges"
    );
    for (id, healing) in [
        ("FAIR_COOKIE_PAIL", 50),
        ("FAIR_CORN_STICK", 20),
        ("FAIR_PRETZEL", 20),
        ("FAIR_CHEESE_CURDS", 30),
        ("FAIR_ROOT_BEER", 60),
        ("FAIR_ICE_CREAM", 50),
        ("FAIR_PICKLE", 10),
        ("FAIR_KETTLE_CORN", 30),
        ("FAIR_HONEY", 20),
        ("FAIR_HOTDISH", 80),
        ("FAIR_PRONTO_PUP", 30),
        ("FAIR_MINI_DONUTS", 50),
        ("FAIR_FUNNEL_CAKE", 60),
        ("FAIR_FRIED_CANDY", 80),
        ("FAIR_WALLEYE", 100),
        ("FAIR_BEER", 100),
    ] {
        // Each explicit fixture starts with 1 HP; recovery must never be hidden
        // by the previous snack filling the party member's HP.
        let (root, _, mut c) = fixture(false);
        let seller = merchants
            .iter()
            .find(|a| {
                a["stock"]
                    .as_array()
                    .expect("stock")
                    .iter()
                    .any(|item| item == id)
            })
            .expect("food vendor");
        buy(&mut c, &r, super::point(&seller["approach"]), id);
        eat(&mut c, &r, id, healing);
        let confusion = c.shell.shell.session().state().storage.party.pokemon[0]
            .as_ref()
            .expect("snack target")
            .confusion_turns;
        assert_eq!(confusion, if id == "FAIR_BEER" { 4 } else { 0 }, "{id}");
        if id == "FAIR_BEER" {
            c = continue_save(&mut c, &root, &r);
            assert_eq!(
                c.shell.shell.session().state().storage.party.pokemon[0]
                    .as_ref()
                    .expect("continued beer target")
                    .confusion_turns,
                4
            );
            counter(&mut c, &r, (79, 60));
            finish(&mut c);
            assert_eq!(
                c.snapshot()
                    .expect("first aid cures beer confusion")
                    .party
                    .slots[0]
                    .pokemon
                    .confusion_turns,
                0
            );
        }
    }
}
