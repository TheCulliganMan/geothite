//! Declared party/badge/capacity setup; island travel and rewards use native input.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::LearnedMove;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::ITEM_POCKET_ITEM;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::super::{
    FieldPackAction, FieldPackPocket, StartMenuOption, VisibleShellController,
    mark_runtime_snapshot_dirty, visible_selected_pack_item_actions, visible_start_menu_options,
};
use super::super::northstar_ice_lair::{choice, talk};
use super::super::pepin_discovery::paddle;
use super::super::winona::{at, press, travel};
use super::{event, finish, itemfinder, teaching, use_move};

const LAKE: &str = "MnLeechLake";
const TRAINER: &str = "MN_EXTRA_LEECH_ISLAND_NATURALIST";
const WON: &str = "EVENT_MN_EXTRA_LEECH_ISLAND_NATURALIST";
const CLAIMED: &str = "EVENT_MN_LEECH_ISLAND_WATER";
const EDDY: &str = "EVENT_MN_LEECH_EDDY_CACHE";

#[path = "leech_lake_mainland.rs"]
mod mainland;

fn checkpoint(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let directory = PathBuf::from(
        std::env::var_os("CRYSTAL_LEECH_ISLAND_REVIEW_DIR").expect("fresh island review directory"),
    );
    std::fs::create_dir_all(&directory).expect("review directory");
    let path = directory.join(format!("{name}.crystalsave"));
    assert!(!path.exists(), "retain every reached island state");
    c.save(&path).expect("supported native reached save");
    path
}

fn fixture(
    at: (i16, i16),
    badge: bool,
    whirlpool: bool,
    full: bool,
    needs_care: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = super::load();
    let mut c = super::fixture(&root, &r, LAKE, at);
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["VOYAGEAR", "CINDORE", "FORTIGARD", "MUDMENDER"] {
        let mut mon = r
            .data()
            .create_pokemon(species, 60, Default::default())
            .expect("active Minnesota party");
        if species == "VOYAGEAR" {
            for name in ["SURF", "WHIRLPOOL"] {
                assert!(mon.species.tmhm_learnset.iter().any(|m| m == name));
            }
            mon.moves
                .retain(|m| m.name != "SURF" && m.name != "WHIRLPOOL");
            mon.moves.truncate(2);
            for name in [Some("SURF"), whirlpool.then_some("WHIRLPOOL")]
                .into_iter()
                .flatten()
            {
                mon.moves.push(LearnedMove {
                    name: name.into(),
                    current_pp: r.data().moves[name].pp,
                    pp_ups: 0,
                });
            }
        }
        mon.item = (species == "VOYAGEAR").then(|| "SMOKE_BALL".into());
        if needs_care {
            mon.hp = 1;
            for learned in &mut mon.moves {
                learned.current_pp = 0;
            }
        }
        assert!(state.storage.party.add_pokemon(mon));
    }
    state.sync_party_from_storage();
    for (i, flag) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        let earned = i < 6 || (i == 6 && badge);
        state.badges.johto[i] = earned;
        state
            .flags
            .set_engine_flag(flag, earned)
            .expect("explicit prior badge setup");
    }
    state.money = 5000;
    if full {
        for (_, item) in r
            .data()
            .items
            .iter()
            .filter(|(id, item)| {
                item.pocket == ITEM_POCKET_ITEM
                    && id.as_str() != "MYSTIC_WATER"
                    && id.as_str() != "PP_UP"
            })
            .take(ITEM_POCKET_CAPACITY)
        {
            assert!(state.bag.add_item(item, 1).expect("declared full pocket"));
        }
    } else {
        assert!(
            state
                .bag
                .add_item(&r.data().items["HYPER_POTION"], 10)
                .expect("local supplies")
        );
    }
    for item in ["HM_WHIRLPOOL", "ITEMFINDER"] {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], 1)
                .expect("declared prior HM/research reward")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    // Setup ends here. No later position, party, flag, bag or movement writes.
    (root, r, c)
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[item])
}

// The gallery's shared dry planner intentionally excludes grass. This planner
// reads the live island collision and plans inputs through grass as well.
fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    for _ in 0..512 {
        if c.snapshot().expect("island walking state").battle.is_some() {
            super::super::rochester::flee(c);
        }
        let s = c.snapshot().expect("live island route");
        let start = (s.overworld.tile.x, s.overworld.tile.y);
        if start == to {
            return;
        }
        let live = &c.shell.shell.session().overworld().map;
        let module = &r.data().maps[&s.overworld.map_name];
        let occupied = s
            .visible_objects
            .iter()
            .map(|o| (o.x as i16, o.y as i16))
            .collect::<BTreeSet<_>>();
        let mut previous = BTreeMap::new();
        let mut seen = BTreeSet::from([start]);
        let mut todo = VecDeque::from([start]);
        while let Some(p) = todo.pop_front() {
            if p == to {
                break;
            }
            for (q, b) in [
                ((p.0 - 1, p.1), GameButton::Left),
                ((p.0 + 1, p.1), GameButton::Right),
                ((p.0, p.1 - 1), GameButton::Up),
                ((p.0, p.1 + 1), GameButton::Down),
            ] {
                if q.0 < 0
                    || q.1 < 0
                    || q.0 >= live.width as i16 * 2
                    || q.1 >= live.height as i16 * 2
                    || occupied.contains(&q)
                    || seen.contains(&q)
                {
                    continue;
                }
                let block =
                    live.metatile_ids[q.1 as usize / 2 * live.width as usize + q.0 as usize / 2];
                let kind = &r.data().tilesets[&module.attributes.tileset_name].collision
                    [&format!("{block:02x}")][q.1 as usize % 2 * 2 + q.0 as usize % 2];
                if !matches!(
                    kind.as_str(),
                    "FLOOR" | "TALL_GRASS" | "WARP_CARPET_DOWN" | "LADDER"
                ) {
                    continue;
                }
                previous.insert(q, (p, b));
                seen.insert(q);
                todo.push_back(q);
            }
        }
        assert!(
            seen.contains(&to),
            "ordinary native island path {start:?} -> {to:?}"
        );
        let mut point = to;
        let mut first = None;
        while point != start {
            let (prior, b) = previous[&point];
            first = Some(b);
            point = prior;
        }
        press(c, first.expect("nonempty native walking path"));
        c.wait_frames(16).expect("native walking presentation");
    }
    panic!("native island route did not reach {to:?}");
}

fn current_pushback(c: &mut VisibleShellController, r: &CrystalRuntime) {
    // Check the gameplay gate after fully released input. The native current
    // returns the surfer to open water; printer/forced-movement presentation
    // can leave the facing interaction a tile farther west. Neither state may
    // release control on the cache approach or award the hidden item.
    for _ in 0..4 {
        paddle(c, r, (59, 20));
        c.wait_frames(64).expect("released Surf input");
        press(c, GameButton::Right);
        if c.snapshot()
            .expect("native lake encounter after current entry")
            .battle
            .is_some()
        {
            super::super::rochester::flee(c);
        }
        finish(c);
        let s = c.snapshot().expect("native current return");
        assert!(
            s.overworld.tile.y == 20 && (58..=59).contains(&s.overworld.tile.x),
            "uncleared current released the player beyond the gate: {:?}",
            s.overworld.tile
        );
        assert_eq!(s.overworld.mode, MovementMode::Surf);
        assert!(!event(c, EDDY));
    }
    paddle(c, r, (58, 20));
    paddle(c, r, (59, 20));
}

fn land(c: &mut VisibleShellController, r: &CrystalRuntime) {
    super::super::minneapolis_falls::mount(c, GameButton::Right, LAKE, (32, 29));
    paddle(c, r, (45, 30));
    super::super::minneapolis_falls::step(
        c,
        GameButton::Right,
        LAKE,
        (46, 30),
        "native island dismount",
    );
    assert_eq!(
        c.snapshot().expect("native island dismount").overworld.mode,
        MovementMode::Normal
    );
}

fn select(
    c: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..64 {
        let current = index(c);
        if current == target {
            press(c, GameButton::A);
            return;
        }
        press(
            c,
            if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native item cursor did not reach the visible target");
}

fn give_mystic_water(c: &mut VisibleShellController, r: &CrystalRuntime) {
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("native Start");
    let pack = visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    select(c, pack, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::Items) {
            break;
        }
        press(c, GameButton::Right);
    }
    let snapshot = c.snapshot().expect("real island reward in PACK");
    let reward = snapshot
        .bag
        .items
        .iter()
        .position(|i| i.item_id == "MYSTIC_WATER")
        .expect("Mystic Water");
    select(c, reward, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("item cursor")
            .option_index
    });
    let snapshot = c.snapshot().expect("native reward actions");
    let give =
        visible_selected_pack_item_actions(&snapshot, &c.shell, &FieldPackPocket::Items, false)
            .expect("item actions")
            .iter()
            .position(|a| *a == FieldPackAction::Give)
            .expect("GIVE");
    select(c, give, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index
    });
    select(c, 1, |c| c.shell.party_cursor);
    for _ in 0..8 {
        press(c, GameButton::B);
    }
    assert_eq!(quantity(c, r, "MYSTIC_WATER"), 0);
    assert_eq!(
        c.shell.shell.session().state().storage.party.pokemon[1]
            .as_ref()
            .expect("native recipient")
            .item
            .as_deref(),
        Some("MYSTIC_WATER")
    );
}

#[test]
#[ignore = "requires completed island pack and CRYSTAL_LEECH_ISLAND_REVIEW_DIR"]
fn leech_island_controller_surfs_grass_battles_reward_drop_return_and_continue() {
    let (root, r, mut c) = fixture((31, 29), true, true, false, false);
    checkpoint(&mut c, "journey-initial");
    land(&mut c, &r);
    checkpoint(&mut c, "island-landing");
    walk(&mut c, &r, (50, 32));
    let mut met = false;
    for i in 0..128 {
        let direction = if i % 2 == 0 {
            GameButton::Right
        } else {
            GameButton::Left
        };
        let target = if i % 2 == 0 { (51, 32) } else { (50, 32) };
        for _ in 0..8 {
            let s = c.snapshot().expect("native grass step");
            if s.battle.is_some() || (s.overworld.tile.x, s.overworld.tile.y) == target {
                break;
            }
            press(&mut c, direction);
            c.wait_frames(16).expect("grass step presentation");
        }
        if let Some(battle) = c.snapshot().expect("native island habitat").battle {
            assert!(
                ["LEECHIMP", "VELVAWN", "YODELOON", "LEECHURGE", "SHEDSTAG"]
                    .contains(&battle.enemy_pokemon.species.id.as_str())
            );
            checkpoint(&mut c, "island-wild-battle");
            super::super::rochester::flee(&mut c);
            met = true;
            break;
        }
    }
    assert!(met, "island grass must produce a real regional encounter");
    walk(&mut c, &r, (52, 30));
    checkpoint(&mut c, "naturalist-approach");
    let before = c.snapshot().expect("before declined trainer").party;
    talk(&mut c, GameButton::Right);
    choice(&mut c, false);
    finish(&mut c);
    assert_eq!(c.snapshot().expect("declined party").party, before);
    assert!(!event(&c, WON));
    talk(&mut c, GameButton::Right);
    choice(&mut c, true);
    assert!(super::super::rochester_earned::try_battle(
        &mut c, &r, TRAINER, WON
    ));
    finish(&mut c);
    assert!(event(&c, WON));
    assert!(c.shell.shell.session().state().money > 5000);
    let after = c.snapshot().expect("actual victory").party;
    assert!(
        after
            .slots
            .iter()
            .any(|p| before.slots.iter().any(|old| old.index == p.index
                && p.pokemon.moves.iter().any(|m| old
                    .pokemon
                    .moves
                    .iter()
                    .any(|prior| prior.name == m.name && m.current_pp < prior.current_pp))))
    );
    walk(&mut c, &r, (52, 33));
    checkpoint(&mut c, "water-reward-approach");
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "MYSTIC_WATER"), 1);
    let save = checkpoint(&mut c, "water-reward-claimed");
    c = VisibleShellController::load_save(root.clone(), r.clone(), save, None)
        .expect("island Continue");
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "MYSTIC_WATER"), 1);
    give_mystic_water(&mut c, &r);
    checkpoint(&mut c, "water-reward-held");
    super::super::minneapolis_falls::step(
        &mut c,
        GameButton::Down,
        LAKE,
        (52, 34),
        "approach native ledge lip",
    );
    checkpoint(&mut c, "ledge-upper-lip");
    super::super::minneapolis_falls::step(
        &mut c,
        GameButton::Down,
        LAKE,
        (52, 36),
        "native downward island hop",
    );
    checkpoint(&mut c, "ledge-landing");
    for _ in 0..3 {
        press(&mut c, GameButton::Up);
        c.wait_frames(16)
            .expect("blocked reverse ledge presentation");
        at(&mut c, LAKE, (52, 36));
    }
    walk(&mut c, &r, (49, 35));
    for _ in 0..3 {
        press(&mut c, GameButton::Right);
        c.wait_frames(16)
            .expect("blocked ledge course presentation");
        at(&mut c, LAKE, (49, 35));
    }
    walk(&mut c, &r, (48, 34));
    for _ in 0..3 {
        press(&mut c, GameButton::Right);
        c.wait_frames(16)
            .expect("blocked side approach presentation");
        at(&mut c, LAKE, (48, 34));
    }
    walk(&mut c, &r, (46, 30));
    super::super::minneapolis_falls::mount(&mut c, GameButton::Left, LAKE, (45, 30));
    paddle(&mut c, &r, (32, 29));
    super::super::minneapolis_falls::step(
        &mut c,
        GameButton::Left,
        LAKE,
        (31, 29),
        "returned mainland launch",
    );
    assert_eq!(
        c.snapshot()
            .expect("returned original shore")
            .overworld
            .mode,
        MovementMode::Normal
    );
    checkpoint(&mut c, "returned-original-launch");
}

#[test]
#[ignore = "requires completed island pack and CRYSTAL_LEECH_ISLAND_REVIEW_DIR"]
fn leech_island_controller_teaches_native_whirlpool_finds_cache_once_and_continues() {
    let (root, r, mut c) = fixture((31, 29), true, false, false, false);
    teaching::teach(&mut c, "HM_WHIRLPOOL", "WHIRLPOOL");
    super::super::minneapolis_falls::mount(&mut c, GameButton::Right, LAKE, (32, 29));
    paddle(&mut c, &r, (59, 20));
    itemfinder(&mut c, true);
    current_pushback(&mut c, &r);
    paddle(&mut c, &r, (59, 20));
    assert!(!event(&c, EDDY));
    checkpoint(&mut c, "current-closed");
    use_move(&mut c, super::super::super::PartyFieldMove::Whirlpool);
    if c.snapshot()
        .expect("native current confirmation")
        .ui
        .pending_yes_no
        .is_some()
    {
        choice(&mut c, true);
    }
    finish(&mut c);
    assert_eq!(
        c.shell.shell.session().overworld().map.metatile_at(30, 10),
        Some(0x36)
    );
    checkpoint(&mut c, "current-cleared-live-before-reload");
    super::super::minneapolis_falls::step(
        &mut c,
        GameButton::Right,
        LAKE,
        (60, 20),
        "enter cleared native current",
    );
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, EDDY));
    assert_eq!(quantity(&c, &r, "PP_UP"), 1);
    let path = checkpoint(&mut c, "eddy-cache-claimed");
    c = VisibleShellController::load_save(root, r.clone(), path, None)
        .expect("same-pack cache Continue");
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(quantity(&c, &r, "PP_UP"), 1);
    itemfinder(&mut c, false);
    super::super::minneapolis_falls::step(
        &mut c,
        GameButton::Left,
        LAKE,
        (59, 20),
        "leave reloaded cache cove",
    );
    paddle(&mut c, &r, (32, 29));
    super::super::minneapolis_falls::step(
        &mut c,
        GameButton::Left,
        LAKE,
        (31, 29),
        "return from current to original shore",
    );
    checkpoint(&mut c, "eddy-cache-returned-shore");
}

#[test]
#[ignore = "requires completed island pack and CRYSTAL_LEECH_ISLAND_REVIEW_DIR"]
fn leech_island_controller_whirlpool_denied_without_seventh_badge() {
    let (_, r, mut c) = fixture((31, 29), false, true, false, false);
    super::super::minneapolis_falls::mount(&mut c, GameButton::Right, LAKE, (32, 29));
    paddle(&mut c, &r, (59, 20));
    current_pushback(&mut c, &r);
    paddle(&mut c, &r, (59, 20));
    use_move(&mut c, super::super::super::PartyFieldMove::Whirlpool);
    finish(&mut c);
    assert_eq!(
        c.shell.shell.session().overworld().map.metatile_at(30, 10),
        Some(0x07)
    );
    current_pushback(&mut c, &r);
    assert!(!event(&c, EDDY));
    checkpoint(&mut c, "whirlpool-badge-denied");
}

#[test]
#[ignore = "requires completed island pack and CRYSTAL_LEECH_ISLAND_REVIEW_DIR"]
fn leech_island_controller_full_pocket_retains_ball_then_native_toss_retry() {
    let (root, r, mut c) = fixture((31, 29), true, true, true, false);
    land(&mut c, &r);
    walk(&mut c, &r, (52, 33));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(!event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "MYSTIC_WATER"), 0);
    let path = checkpoint(&mut c, "island-full-pocket-refused");
    c = VisibleShellController::load_save(root, r.clone(), path, None).expect("capacity Continue");
    super::super::northstar_progression_rewards::native_toss_first_pocket_item(
        &mut c,
        &super::super::super::FieldPackPocket::Items,
    );
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, CLAIMED));
    assert_eq!(quantity(&c, &r, "MYSTIC_WATER"), 1);
    checkpoint(&mut c, "island-capacity-retry-claimed");
}

#[test]
#[ignore = "requires completed island pack and CRYSTAL_LEECH_ISLAND_REVIEW_DIR"]
fn leech_island_controller_original_public_road_care_and_both_cabin_returns() {
    let (root, r, mut c) = fixture((32, 47), false, false, false, true);
    checkpoint(&mut c, "public-road-initial");
    walk(&mut c, &r, (11, 14));
    travel(&mut c, GameButton::Up, "MnLeechLakeCareCabin", (10, 13));
    walk(&mut c, &r, (7, 5));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::Down);
    at(&mut c, "MnLeechLakeCareCabin", (7, 5));
    finish(&mut c);
    assert!(
        c.snapshot()
            .expect("free native full-party care")
            .party
            .slots
            .iter()
            .all(|s| s.pokemon.hp == s.pokemon.max_hp
                && s.pokemon
                    .moves
                    .iter()
                    .all(|m| m.current_pp == r.data().moves[&m.name].pp))
    );
    assert_eq!(c.shell.shell.session().state().money, 5000);
    let saved = checkpoint(&mut c, "original-free-care");
    c = VisibleShellController::load_save(root, r.clone(), saved, None).expect("care Continue");
    for (room, porch, arrival, carpets) in [
        (
            "MnLeechLakeCareCabin",
            (11, 14),
            (10, 13),
            [(10, 15), (11, 15)],
        ),
        ("MnLeechLakeFisherHouse", (9, 36), (5, 6), [(5, 7), (6, 7)]),
    ] {
        for (index, carpet) in carpets.into_iter().enumerate() {
            if room != "MnLeechLakeCareCabin" || index > 0 {
                walk(&mut c, &r, porch);
                travel(&mut c, GameButton::Up, room, arrival);
            }
            walk(&mut c, &r, (carpet.0, carpet.1 - 1));
            travel(&mut c, GameButton::Down, LAKE, porch);
            press(&mut c, GameButton::Down);
            at(&mut c, LAKE, (porch.0, porch.1 + 1));
        }
    }
    walk(&mut c, &r, (35, 22));
    checkpoint(&mut c, "original-pier-passing-lane");
    walk(&mut c, &r, (31, 29));
    walk(&mut c, &r, (32, 47));
    assert_eq!(
        c.snapshot().expect("public walking return").overworld.mode,
        MovementMode::Normal
    );
    for flag in [
        "EVENT_MN_COURIER_DEFEATED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
        CLAIMED,
        EDDY,
        WON,
    ] {
        assert!(!event(&c, flag));
    }
    checkpoint(&mut c, "public-road-returned");
}
