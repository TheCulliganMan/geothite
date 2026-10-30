//! Explicit local fixtures; all city actions use production controller inputs.
//! Initial position, partners, prior badges, money, inventory and RNG are setup,
//! not an earned regional journey. Gym6 itself and its Surf reward are earned.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::{AssetRoot, MapModule, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::{ITEM_POCKET_ITEM, is_mail_item_id};
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::{ScriptFlagCommand, apply_script_flag_mutation};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, cleanup_reached_save, indoor_walk, neighbors, reached_save_path,
};

#[path = "town_exterior_review.rs"]
mod town_exterior_review;

const TOWN: &str = "MnBrainerd";
const SURF_EVENT: &str = "EVENT_MN_SURF_GIFT_RECEIVED";
const CAMP_EVENT: &str = "EVENT_MN_BRAINERD_CAMP_REPEL";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}
fn load() -> (AssetRoot, CrystalRuntime) {
    let path = std::env::var_os("CRYSTAL_BRAINERD_CITY_CONTROLLER_PACK")
        .map_or_else(
            || repo().join("content-packs/minnesota-cities-review.crystalpack"),
            PathBuf::from,
        )
        .canonicalize()
        .expect("published external three-city pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified native city pack"),
    )
    .expect("city runtime");
    (root, runtime)
}
fn canonical_gym() -> Value {
    let path = std::env::var_os("CRYSTAL_CITY_GYMS_SOURCE").map_or_else(
        || repo().join(".local/minnesota-city-source-freeze/gyms-887afc43d19bc0573ca5ec361f4c141771347421040f70f100f02fea4997a189.json"),
        PathBuf::from,
    );
    let design: Value = serde_json::from_str(
        &std::fs::read_to_string(path).expect("exact installed canonical source"),
    )
    .expect("canonical gym design");
    design["gyms"]
        .as_array()
        .expect("gyms")
        .iter()
        .find(|g| g["slot"] == 6)
        .expect("canonical Gym6")
        .clone()
}
fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    map: &str,
    at: Point,
) -> VisibleShellController {
    let mut c = VisibleShellController {
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
        .expect("explicit local review placement"),
    };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    c.set_runtime_journal_enabled(false);
    c
}
fn at(c: &mut VisibleShellController) -> Point {
    let s = c.snapshot().expect("position");
    (s.overworld.tile.x, s.overworld.tile.y)
}
fn map(c: &mut VisibleShellController) -> String {
    c.snapshot().expect("map").overworld.map_name
}
fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production city input");
    c.wait_frames(20).expect("visible presentation settle");
    assert_eq!(c.shell.last_error, None);
}
fn finish(c: &mut VisibleShellController) {
    for _ in 0..128 {
        let s = c.snapshot().expect("dialogue continuation");
        assert!(s.battle.is_none(), "battle needs real battle inputs");
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
    panic!("city dialogue did not release input");
}
fn walk(c: &mut VisibleShellController, runtime: &CrystalRuntime, target: Point) {
    let before = map(c);
    indoor_walk(c, runtime, target);
    assert_eq!(map(c), before, "safe path crossed a warp");
    assert_eq!(at(c), target, "native walking did not reach its target");
}
fn talk_from(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    approach: Point,
    facing: GameButton,
) {
    walk(c, runtime, approach);
    press(c, facing);
    assert_eq!(
        at(c),
        approach,
        "actor/counter must block ordinary movement"
    );
    press(c, GameButton::A);
}
fn flag(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("native event flag")
}
fn quantity(c: &VisibleShellController, runtime: &CrystalRuntime, name: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[name])
}
fn semantic_flags(c: &VisibleShellController) -> (BTreeSet<String>, BTreeSet<String>) {
    let flags = &c.shell.shell.session().state().flags;
    // Flag getters treat missing and explicit false identically. Compare every
    // active flag in both namespaces, without masking travel or story flags.
    (
        flags.active_event_flags().cloned().collect(),
        flags
            .engine_flags
            .iter()
            .filter_map(|(name, value)| value.then_some(name.clone()))
            .collect(),
    )
}
fn continue_save(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
) -> VisibleShellController {
    let before = c.snapshot().expect("before save");
    let badges = c.shell.shell.session().state().badges.clone();
    let flags = semantic_flags(c);
    let money = c.shell.shell.session().state().money;
    let overrides = c.shell.shell.session().state().map_block_overrides.clone();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = reached_save_path(&format!(
        "brainerd-{}-{stamp}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("production save");
    let mut loaded =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path.clone(), None)
            .expect("production Continue");
    cleanup_reached_save(&path);
    let after = loaded.snapshot().expect("continued state");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    let state = loaded.shell.shell.session().state();
    assert_eq!(state.badges, badges);
    assert_eq!(semantic_flags(&loaded), flags);
    assert_eq!(state.money, money);
    assert_eq!(state.map_block_overrides, overrides);
    loaded
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn brainerd_city_controller_all_facades_both_carpets_and_safe_arrivals() {
    let (root, runtime) = load();
    let source: Value = serde_json::from_str(
        &std::fs::read_to_string(repo().join("planning/minnesota/gyms/brainerd-winch.json"))
            .expect("current installed winch Gym source"),
    )
    .expect("winch Gym JSON");
    let gym = &source["gyms"][0];
    let actual = &runtime.data().maps["MnBrainerdGym"];
    for (index, expected) in [
        (3, point(&gym["arrival_core"])),
        (1, point(&gym["exit_tiles_core"][0])),
        (2, point(&gym["exit_tiles_core"][1])),
    ] {
        let warp = actual
            .events
            .warps
            .iter()
            .find(|w| w.index == index)
            .expect("native Gym warp");
        assert_eq!((warp.x as i16, warp.y as i16), expected);
    }
    for (room, exterior, arrival, carpets) in [
        (
            "MnBrainerdPokecenter1F",
            (9, 22),
            (10, 13),
            [(10, 15), (11, 15)],
        ),
        ("MnBrainerdMart", (19, 22), (6, 5), [(6, 7), (7, 7)]),
        (
            "MnBrainerdGym",
            (12, 36),
            point(&gym["arrival_core"]),
            [
                point(&gym["exit_tiles_core"][0]),
                point(&gym["exit_tiles_core"][1]),
            ],
        ),
        (
            "MnBrainerdRangerStation",
            (23, 16),
            (8, 9),
            [(8, 11), (9, 11)],
        ),
        ("MnBrainerdOutfitter", (47, 36), (6, 5), [(6, 7), (7, 7)]),
    ] {
        let mut c = fixture(&root, &runtime, TOWN, exterior);
        for carpet in carpets {
            for _ in 0..6 {
                press(&mut c, GameButton::Up);
                if map(&mut c) == room {
                    break;
                }
            }
            assert_eq!(map(&mut c), room);
            assert_eq!(at(&mut c), arrival, "passive native arrival");
            c.wait_frames(120).expect("safe arrival settles");
            assert_eq!(map(&mut c), room, "passive arrival bounced");
            walk(&mut c, &runtime, (carpet.0, carpet.1 - 1));
            for _ in 0..6 {
                press(&mut c, GameButton::Down);
                if map(&mut c) == TOWN {
                    break;
                }
            }
            assert_eq!(map(&mut c), TOWN);
            assert_eq!(at(&mut c), exterior, "exact passive exterior return");
            c.wait_frames(120).expect("safe exterior settles");
            assert_eq!(map(&mut c), TOWN);
        }
        let _loaded = continue_save(&mut c, &root, &runtime);
    }
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn brainerd_city_controller_native_east_hop_reverse_wall_and_ordinary_ramp_return() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, TOWN, (23, 38));
    let mut saw_face = false;
    for _ in 0..8 {
        // Observe each authoritative input result before extra settling frames.
        c.press(GameButton::Right).expect("native east hop");
        let p = at(&mut c);
        assert!(
            matches!(p, (23, 38) | (24, 38) | (26, 38)),
            "unexpected hop {p:?}"
        );
        saw_face |= p == (24, 38);
        if p == (26, 38) {
            break;
        }
    }
    assert!(saw_face, "native HOP_RIGHT face must be traversed");
    assert_eq!(at(&mut c), (26, 38));
    for _ in 0..4 {
        press(&mut c, GameButton::Left);
        assert_eq!(at(&mut c), (26, 38), "reverse wall leaked");
    }
    // Existing native source planner excludes HOP/WALL; this return can only
    // walk the ordinary southern ramp, never reverse the one-way edge.
    walk(&mut c, &runtime, (20, 41));
    walk(&mut c, &runtime, (23, 38));
    let _loaded = continue_save(&mut c, &root, &runtime);
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn brainerd_city_controller_heals_hp_status_pp_and_opens_production_pc() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnBrainerdPokecenter1F", (10, 13));
    // Injury and depleted PP are initial service fixtures, never proof of combat.
    let mut partner = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("native partner");
    partner.hp = partner.max_hp.saturating_sub(30);
    partner.status = Some("POISON".into());
    for learned in &mut partner.moves {
        learned.current_pp = 0;
    }
    assert!(!partner.moves.is_empty());
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    talk_from(&mut c, &runtime, (7, 5), GameButton::Up);
    let waiting = c.snapshot().expect("nurse dialogue owns input");
    assert!(waiting.ui.text.is_some());
    let p = at(&mut c);
    press(&mut c, GameButton::Down);
    assert_eq!(at(&mut c), p);
    finish(&mut c);
    let healed = c.snapshot().expect("actual healed party").party.slots[0]
        .pokemon
        .clone();
    assert_eq!(healed.hp, healed.max_hp);
    assert_eq!(healed.status, None);
    for m in &healed.moves {
        assert_eq!(m.current_pp, runtime.data().moves[&m.name].pp);
    }
    talk_from(&mut c, &runtime, (15, 4), GameButton::Up);
    for _ in 0..96 {
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.shell.pc_hub_cursor.is_some(),
        "native PokemonCenterPC menu must open"
    );
    assert!(c.snapshot().expect("visible PC").ui.menu.is_some());
    for _ in 0..96 {
        if c.shell.pc_hub_cursor.is_none()
            && c.shell.special_boundary.is_none()
            && c.snapshot().expect("PC closure").phase == RuntimeShellPhase::Overworld
            && !c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::B);
    }
    assert!(c.shell.pc_hub_cursor.is_none());
    let _loaded = continue_save(&mut c, &root, &runtime);
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn brainerd_city_controller_mart_charges_native_price_and_delivers_one_item() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnBrainerdMart", (6, 5));
    c.shell.shell.session_mut().state_mut().money = 10_000;
    mark_runtime_snapshot_dirty(&mut c.shell);
    talk_from(&mut c, &runtime, (3, 3), GameButton::Left);
    for _ in 0..96 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    let shop = c
        .snapshot()
        .expect("native shop")
        .pending_shop
        .expect("actual mart");
    assert_eq!(shop.mart_id, "MART_MN_BRAINERD");
    let item = shop.inventory[0].clone();
    let price = u32::from(runtime.data().items[&item].price);
    assert!(price > 0);
    let before = quantity(&c, &runtime, &item);
    press(&mut c, GameButton::A); // BUY
    press(&mut c, GameButton::A); // first actual stock item
    let q = c.shell.shop_quantity.as_ref().expect("quantity selector");
    assert_eq!(q.item_id, item);
    assert_eq!(q.quantity, 1);
    assert_eq!(u32::from(q.unit_price), price);
    press(&mut c, GameButton::A); // confirm quantity, no mutation yet
    assert_eq!(c.shell.shell.session().state().money, 10_000);
    assert_eq!(quantity(&c, &runtime, &item), before);
    press(&mut c, GameButton::A); // YES -> actual transaction
    assert_eq!(c.shell.shell.session().state().money, 10_000 - price);
    assert_eq!(quantity(&c, &runtime, &item), before + 1);
    for _ in 0..96 {
        let s = c.snapshot().expect("shop closure");
        if s.pending_shop.is_none() && !s.ui.text_window_open && !c.has_pending_script_work() {
            break;
        }
        let button = if s.pending_shop.is_none() || c.shell.shop_notice.is_some() {
            GameButton::A
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
    assert!(c.snapshot().expect("closed shop").pending_shop.is_none());
    let loaded = continue_save(&mut c, &root, &runtime);
    assert_eq!(quantity(&loaded, &runtime, &item), before + 1);
}

#[test]
#[ignore = "requires the published external three-city pack"]
fn brainerd_city_controller_optional_gift_full_bag_retry_once_and_continue() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnBrainerdOutfitter", (6, 5));
    // Deliberate full-pocket fixture isolates atomic refusal, then retry.
    let state = c.shell.shell.session_mut().state_mut();
    state.bag.items.clear();
    for (id, item) in runtime
        .data()
        .items
        .iter()
        .filter(|(id, item)| {
            item.pocket == ITEM_POCKET_ITEM && id.as_str() != "SUPER_REPEL" && !is_mail_item_id(id)
        })
        .take(ITEM_POCKET_CAPACITY)
    {
        assert!(
            state
                .bag
                .add_item(item, 1)
                .unwrap_or_else(|e| panic!("fixture item {id}: {e}"))
        );
    }
    assert_eq!(state.bag.items.len(), ITEM_POCKET_CAPACITY);
    let full = state.bag.clone();
    mark_runtime_snapshot_dirty(&mut c.shell);
    talk_from(&mut c, &runtime, (3, 4), GameButton::Up);
    finish(&mut c);
    assert_eq!(c.shell.shell.session().state().bag, full);
    assert!(!flag(&c, CAMP_EVENT));
    // Capacity setup changes only one slot; actual next giveitem uses input.
    let bag = &mut c.shell.shell.session_mut().state_mut().bag;
    let first = bag.items.keys().next().expect("full pocket").clone();
    bag.items.remove(&first);
    mark_runtime_snapshot_dirty(&mut c.shell);
    talk_from(&mut c, &runtime, (3, 4), GameButton::Up);
    finish(&mut c);
    assert!(flag(&c, CAMP_EVENT));
    assert_eq!(quantity(&c, &runtime, "SUPER_REPEL"), 1);
    let mut loaded = continue_save(&mut c, &root, &runtime);
    let before = loaded.snapshot().expect("saved gift").bag;
    talk_from(&mut loaded, &runtime, (3, 4), GameButton::Up);
    finish(&mut loaded);
    assert_eq!(loaded.snapshot().expect("repeated gift").bag, before);
    assert_eq!(quantity(&loaded, &runtime, "SUPER_REPEL"), 1);
}

// Only Gym6 needs a planner over live block overrides. Ordinary town/room
// walking above reuses authored_route_fixture::indoor_walk unchanged.
fn live_gym(c: &mut VisibleShellController, runtime: &CrystalRuntime) -> MapModule {
    let name = map(c);
    let mut module = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(&name)
        .expect("native Gym6");
    if let Some(overrides) = c
        .shell
        .shell
        .session()
        .state()
        .map_block_overrides
        .get(&name)
    {
        for ((x, y), block) in overrides {
            module.blocks
                [usize::from(*y) * usize::from(module.attributes.width) + usize::from(*x)] = *block;
        }
    }
    module
}
fn gym_path(
    runtime: &CrystalRuntime,
    module: &MapModule,
    from: Point,
    to: Point,
) -> Option<Vec<GameButton>> {
    let blocked = module
        .objects
        .iter()
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let floor = |p: Point| {
        if p.0 < 0
            || p.1 < 0
            || p.0 >= module.attributes.width as i16 * 2
            || p.1 >= module.attributes.height as i16 * 2
        {
            return false;
        }
        let b = module.blocks
            [p.1 as usize / 2 * usize::from(module.attributes.width) + p.0 as usize / 2];
        runtime.data().tilesets[&module.attributes.tileset_name].collision[&format!("{b:02x}")]
            [p.1 as usize % 2 * 2 + p.0 as usize % 2]
            == "FLOOR"
    };
    let mut parents = BTreeMap::from([(from, (from, GameButton::A))]);
    let mut todo = VecDeque::from([from]);
    while let Some(p) = todo.pop_front() {
        if p == to {
            break;
        }
        for (q, b) in neighbors(p) {
            if floor(q) && !blocked.contains(&q) && !parents.contains_key(&q) {
                parents.insert(q, (p, b));
                todo.push_back(q);
            }
        }
    }
    if !parents.contains_key(&to) {
        return None;
    }
    let mut result = Vec::new();
    let mut q = to;
    while q != from {
        let (p, b) = parents[&q];
        result.push(b);
        q = p;
    }
    result.reverse();
    Some(result)
}
fn gym_walk(c: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point) {
    let from = at(c);
    let module = live_gym(c, runtime);
    for b in gym_path(runtime, &module, from, to).expect("safe live Gym6 path") {
        let before = at(c);
        for _ in 0..8 {
            press(c, b);
            if at(c) != before {
                break;
            }
        }
        assert_ne!(at(c), before, "native Gym6 step blocked");
    }
    assert_eq!(at(c), to);
}
fn gym_interact(c: &mut VisibleShellController, runtime: &CrystalRuntime, target: Point) {
    let from = at(c);
    let module = live_gym(c, runtime);
    let mut choices = Vec::new();
    for (approach, away) in neighbors(target) {
        let staging = (approach.0 * 2 - target.0, approach.1 * 2 - target.1);
        if gym_path(runtime, &module, staging, approach).is_none() {
            continue;
        }
        if let Some(path) = gym_path(runtime, &module, from, staging) {
            choices.push((
                path.len(),
                staging,
                approach,
                super::authored_route_fixture::opposite(away),
            ));
        }
    }
    choices.sort_by_key(|(steps, _, _, _)| *steps);
    let (_, staging, approach, facing) = choices
        .first()
        .copied()
        .expect("reachable real Gym6 interaction");
    // A real final walking step establishes facing. Station READ events can
    // stand on FLOOR without a blocking NPC; do not step onto them to turn.
    gym_walk(c, runtime, staging);
    for _ in 0..8 {
        press(c, facing);
        if at(c) == approach {
            break;
        }
    }
    assert_eq!(at(c), approach);
    press(c, GameButton::A);
}
fn point(v: &Value) -> Point {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}

fn battle(c: &mut VisibleShellController) {
    let before = c.snapshot().expect("before actual battle");
    let mut started = false;
    let mut attacks = 0;
    for _ in 0..4096 {
        let s = c.snapshot().expect("actual Gym6 battle");
        let Some(b) = s.battle.as_ref() else {
            if started {
                break;
            }
            press(c, GameButton::A);
            continue;
        };
        started = true;
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
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            let name = if matches!(
                b.enemy_pokemon.species.id.as_str(),
                "MAGNEMITE" | "SKARMORY" | "SNEASEL" | "PINECO"
            ) {
                "FLAMETHROWER"
            } else {
                "PSYCHIC_M"
            };
            let target = b
                .player_moves
                .iter()
                .position(|m| m.name == name && m.current_pp > 0)
                .expect("selected real move with PP");
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
        press(c, input);
    }
    let after = c.snapshot().expect("actual battle result");
    assert!(started && attacks > 0);
    assert!(after.battle.is_none());
    let total = |moves: &[LearnedMove]| moves.iter().map(|m| u32::from(m.current_pp)).sum::<u32>();
    assert!(
        total(&after.party.slots[0].pokemon.moves) < total(&before.party.slots[0].pokemon.moves),
        "selected moves must spend actual PP"
    );
    assert!(after.party.slots[0].pokemon.hp > 0);
}

#[test]
#[ignore = "requires the published external three-city pack and its frozen canonical JSON"]
fn brainerd_city_controller_earns_gym6_badge_tm_then_ranger_surf_once_and_continue() {
    let (root, runtime) = load();
    let gym = canonical_gym();
    let mut c = fixture(&root, &runtime, TOWN, (25, 20));
    // Explicit local battle fixture: first five badges and a legal strong party
    // partner. Badge6, puzzle events, trainer wins, TM and Surf remain unearned.
    let mut partner = runtime
        .data()
        .create_pokemon("MEW", 100, Dv::default())
        .expect("native legal partner");
    partner.moves = ["PSYCHIC_M", "FLAMETHROWER", "SURF", "THUNDERBOLT"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: runtime.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    for (index, flag) in [
        "ENGINE_ZEPHYRBADGE",
        "ENGINE_HIVEBADGE",
        "ENGINE_PLAINBADGE",
        "ENGINE_FOGBADGE",
        "ENGINE_STORMBADGE",
        "ENGINE_MINERALBADGE",
        "ENGINE_GLACIERBADGE",
        "ENGINE_RISINGBADGE",
    ]
    .into_iter()
    .enumerate()
    {
        // Initial prior-badge setup only. The production flag mutator keeps the
        // badge array and engine flag memory consistent before any game input.
        apply_script_flag_mutation(
            state,
            ScriptFlagCommand {
                command: if index < 5 { "setflag" } else { "clearflag" }.into(),
                flag_id: flag.into(),
                source_script: "BrainerdPriorBadgesFixture".into(),
                command_index: index,
            },
        )
        .expect("consistent prior-badge fixture");
    }
    assert_eq!(
        state.badges.johto,
        [true, true, true, true, true, false, false, false]
    );
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..524288)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!flag(&c, "EVENT_MN_GYM_6_DEFEATED"));
    assert!(!flag(&c, SURF_EVENT));
    assert_eq!(quantity(&c, &runtime, "HM_SURF"), 0);
    talk_from(&mut c, &runtime, (25, 20), GameButton::Up);
    finish(&mut c);
    assert_eq!(quantity(&c, &runtime, "HM_SURF"), 0);
    assert!(!flag(&c, SURF_EVENT));
    assert!(!flag(&c, "EVENT_MN_BRAINERD_REPORT"));
    walk(&mut c, &runtime, (12, 36));
    for _ in 0..6 {
        press(&mut c, GameButton::Up);
        if map(&mut c) == "MnBrainerdGym" {
            break;
        }
    }
    assert_eq!(map(&mut c), "MnBrainerdGym");
    assert_eq!(at(&mut c), point(&gym["arrival_core"]));
    // The closed court must not have a floor route to MARA before the puzzle.
    let from = at(&mut c);
    let module = live_gym(&mut c, &runtime);
    assert!(
        neighbors(point(&gym["leader_at"]))
            .iter()
            .all(|(q, _)| gym_path(&runtime, &module, from, *q).is_none())
    );
    assert!(
        gym["stations"]
            .as_array()
            .expect("actual stations")
            .is_empty(),
        "887 Gym6 has no station quiz"
    );
    let trainers = gym["trainers"]
        .as_array()
        .expect("actual canonical trainers");
    assert_eq!(trainers.len(), 2);
    let gates = gym["gates"].as_array().expect("actual gates");
    assert_eq!(gates.len(), 1);
    let mut trainer_wins = 0;
    for step in gym["walkthrough"].as_array().expect("canonical sequence") {
        let step = step.as_str().expect("trainer step");
        let trainer = trainers
            .iter()
            .find(|t| t["id"] == step)
            .expect("887 sequence names an actual trainer");
        gym_interact(&mut c, &runtime, point(&trainer["at"]));
        battle(&mut c);
        finish(&mut c);
        assert!(flag(&c, &format!("EVENT_{step}_DEFEATED")));
        trainer_wins += 1;
        assert!(
            !c.shell.shell.session().state().badges.johto[5],
            "practice cannot grant Badge6"
        );
        let complete = gates[0]["requires"]
            .as_array()
            .expect("actual gate prerequisites")
            .iter()
            .all(|required| flag(&c, required.as_str().expect("required event")));
        assert_eq!(
            flag(&c, "EVENT_MN_GYM_6_COURT_OPEN"),
            complete,
            "only both real wins open the actual upper gangway"
        );
        if !complete {
            let from = at(&mut c);
            let module = live_gym(&mut c, &runtime);
            assert!(
                neighbors(point(&gym["leader_at"]))
                    .iter()
                    .all(|(q, _)| gym_path(&runtime, &module, from, *q).is_none()),
                "first win cannot bypass the locked court"
            );
        }
    }
    assert_eq!(trainer_wins, 2);
    assert!(flag(&c, "EVENT_MN_GYM_6_COURT_OPEN"));
    c = continue_save(&mut c, &root, &runtime);
    gym_interact(&mut c, &runtime, point(&gym["leader_at"]));
    battle(&mut c);
    finish(&mut c);
    assert!(flag(&c, "EVENT_MN_GYM_6_DEFEATED"));
    assert!(flag(&c, "EVENT_MN_GYM_6_TM_RECEIVED"));
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, true, true, true, false, false]
    );
    let tm = gym["reward"].as_str().expect("canonical TM");
    assert_eq!(quantity(&c, &runtime, tm), 1);
    let before = c.snapshot().expect("before repeat leader");
    gym_interact(&mut c, &runtime, point(&gym["leader_at"]));
    finish(&mut c);
    let after = c.snapshot().expect("repeat leader");
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.party, after.party);
    let exit = point(&gym["exit_tiles_core"][0]);
    gym_walk(&mut c, &runtime, (exit.0, exit.1 - 1));
    for _ in 0..6 {
        press(&mut c, GameButton::Down);
        if map(&mut c) == TOWN {
            break;
        }
    }
    assert_eq!(map(&mut c), TOWN);
    assert_eq!(at(&mut c), (12, 36));
    talk_from(&mut c, &runtime, (25, 20), GameButton::Up);
    finish(&mut c);
    assert!(flag(&c, SURF_EVENT));
    assert!(flag(&c, "EVENT_MN_BRAINERD_REPORT"));
    assert_eq!(quantity(&c, &runtime, "HM_SURF"), 1);
    let mut loaded = continue_save(&mut c, &root, &runtime);
    let before = loaded.snapshot().expect("saved earned Surf");
    talk_from(&mut loaded, &runtime, (25, 20), GameButton::Up);
    finish(&mut loaded);
    let after = loaded.snapshot().expect("repeat ranger");
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.party, after.party);
    assert_eq!(quantity(&loaded, &runtime, "HM_SURF"), 1);
}
