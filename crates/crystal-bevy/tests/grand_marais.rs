//! Play the town from a fresh game using only the production controller's buttons.
#![cfg(feature = "bevy-shell")]

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase, RuntimeShellSnapshot};

type Point = (i16, i16);

fn position(snapshot: &RuntimeShellSnapshot) -> Point {
    (snapshot.overworld.tile.x, snapshot.overworld.tile.y)
}

fn path(
    runtime: &CrystalRuntime,
    map: &str,
    from: Point,
    to: Point,
    grass: bool,
) -> Vec<GameButton> {
    let module = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(map)
        .expect("map module");
    let a = &module.attributes;
    let tileset = &runtime.data().tilesets[&a.tileset_name];
    let occupied = module
        .objects
        .iter()
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let doors = module
        .events
        .warps
        .iter()
        .map(|w| (w.x as i16, w.y as i16))
        .collect::<BTreeSet<_>>();
    let mut parents = BTreeMap::from([(from, (from, GameButton::A))]);
    let mut queue = VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        if p == to {
            break;
        }
        for (q, button) in [
            ((p.0 + 1, p.1), GameButton::Right),
            ((p.0 - 1, p.1), GameButton::Left),
            ((p.0, p.1 + 1), GameButton::Down),
            ((p.0, p.1 - 1), GameButton::Up),
        ] {
            if q.0 < 0
                || q.1 < 0
                || q.0 >= a.width as i16 * 2
                || q.1 >= a.height as i16 * 2
                || occupied.contains(&q)
                || parents.contains_key(&q)
            {
                continue;
            }
            if q != to && doors.contains(&q) {
                continue;
            }
            let block = module.blocks[(q.1 as usize / 2) * usize::from(a.width) + q.0 as usize / 2];
            let kind = &tileset.collision[&format!("{block:02x}")]
                [(q.1 as usize % 2) * 2 + q.0 as usize % 2];
            if !matches!(kind.as_str(), "FLOOR" | "DOOR" | "WARP_CARPET_DOWN")
                && !(grass && kind == "TALL_GRASS")
            {
                continue;
            }
            parents.insert(q, (p, button));
            queue.push_back(q);
        }
    }
    assert!(
        parents.contains_key(&to),
        "no walkable path on {map}: {from:?} -> {to:?}"
    );
    let mut buttons = Vec::new();
    let mut cursor = to;
    while cursor != from {
        let (parent, button) = parents[&cursor];
        buttons.push(button);
        cursor = parent;
    }
    buttons.reverse();
    buttons
}

fn walk(game: &mut VisibleShellController, runtime: &CrystalRuntime, to: Point, grass: bool) {
    let before = game.snapshot().expect("walk start");
    let map = before.overworld.map_name.clone();
    for button in path(runtime, &map, position(&before), to, grass) {
        let prior = game.snapshot().expect("before step");
        for _ in 0..8 {
            game.press(button).expect("production movement");
            let after = game.snapshot().expect("after step");
            if after.overworld.map_name != map
                || after.battle.is_some()
                || matches!(
                    after.phase,
                    RuntimeShellPhase::WildBattle | RuntimeShellPhase::StaticWildBattle
                )
            {
                return;
            }
            if position(&after) != position(&prior) {
                break;
            }
        }
        let after = game.snapshot().expect("completed step");
        assert_ne!(
            position(&prior),
            position(&after),
            "movement blocked on {map} at {:?}, target={to:?}, button={button:?}, phase={:?}, text={:?}, menu={:?}",
            position(&prior),
            after.phase,
            after.ui.text,
            after.ui.menu
        );
    }
    assert_eq!(position(&game.snapshot().expect("walk end")), to);
}

fn text_done(game: &mut VisibleShellController) {
    for _ in 0..64 {
        let s = game.snapshot().expect("text snapshot");
        if s.phase == RuntimeShellPhase::Overworld
            && s.ui.pending_text_wait.is_none()
            && !game.has_pending_script_work()
            && s.ui.menu.is_none()
            && s.ui
                .text
                .as_ref()
                .is_none_or(|text| text.label != "visible-shell:battle-message")
        {
            return;
        }
        game.press(GameButton::A)
            .expect("advance production dialogue");
    }
    panic!(
        "dialogue failed to finish: {:?}",
        game.snapshot().expect("stuck text").ui
    );
}

fn talk_below(game: &mut VisibleShellController, runtime: &CrystalRuntime, at: Point) -> String {
    walk(game, runtime, (at.0, at.1 + 1), false);
    game.press(GameButton::Up).expect("face town resident");
    game.press(GameButton::A).expect("talk to town resident");
    let snapshot = game.snapshot().expect("resident dialogue");
    assert!(snapshot.ui.pending_text_wait.is_some());
    let text = snapshot
        .ui
        .text
        .and_then(|text| text.asm_text)
        .expect("resident text body");
    text_done(game);
    text
}

fn enter(game: &mut VisibleShellController, runtime: &CrystalRuntime, target: &str) {
    let town = runtime
        .data()
        .assemble_map_module_from_compiled_payloads("MnGrandMarais")
        .expect("town");
    let door = town
        .events
        .warps
        .iter()
        .find(|w| w.target_map_constant == target)
        .expect("exterior door");
    walk(game, runtime, (door.x as i16, door.y as i16), false);
    for _ in 0..4 {
        if game.snapshot().expect("door state").overworld.map_name != "MnGrandMarais" {
            break;
        }
        game.press(GameButton::Up)
            .expect("step through exterior doorway");
    }
    assert_ne!(
        game.snapshot()
            .expect("entered interior")
            .overworld
            .map_name,
        "MnGrandMarais"
    );
}

fn leave(game: &mut VisibleShellController, runtime: &CrystalRuntime) {
    let s = game.snapshot().expect("interior");
    let m = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(&s.overworld.map_name)
        .expect("interior module");
    let exit = &m.events.warps[0];
    walk(game, runtime, (exit.x as i16, exit.y as i16), false);
    for _ in 0..4 {
        if game.snapshot().expect("exit state").overworld.map_name == "MnGrandMarais" {
            break;
        }
        game.press(GameButton::Down)
            .expect("leave on the exit carpet");
    }
    assert_eq!(
        game.snapshot().expect("return warp").overworld.map_name,
        "MnGrandMarais"
    );
}

#[test]
fn grand_marais_new_game_services_quest_battle_and_save() {
    let pack = std::env::var_os("CRYSTAL_MINNESOTA_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-grand-marais.crystalpack")
        });
    assert!(
        pack.is_file(),
        "build the external Grand Marais pack first: {}",
        pack.display()
    );
    let pack = pack.canonicalize().expect("canonical town pack path");
    let root = AssetRoot::new(pack.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack).expect("verified town pack"),
    )
    .expect("town runtime");
    let mut game = VisibleShellController::new_game(root.clone(), runtime.clone(), "CHRIS", None)
        .expect("production New Game");
    game.set_runtime_journal_enabled(false);
    let initial = game.snapshot().expect("new game snapshot");
    assert_eq!(initial.overworld.map_name, "MnGrandMaraisFieldStation");
    assert!(initial.party.slots.is_empty());
    game.press(GameButton::Down)
        .expect("field station exit intercept");
    let first = game.snapshot().expect("Tamarack dialogue");
    assert!(first.ui.pending_text_wait.is_some());
    game.press(GameButton::Down)
        .expect("movement during dialogue");
    assert_eq!(
        position(&game.snapshot().expect("dialogue owns input")),
        position(&first)
    );
    game.press(GameButton::A).expect("advance dialogue page");
    let next = game.snapshot().expect("next text page");
    assert_ne!(
        first.ui.text, next.ui.text,
        "A did not advance visible dialogue"
    );
    text_done(&mut game);
    let prepared = game.snapshot().expect("starter and supplies");
    assert_eq!(prepared.party.slots.len(), 1);
    assert_eq!(prepared.party.slots[0].pokemon.species.id, "TOTODILE");
    assert!(
        prepared
            .bag
            .key_items
            .iter()
            .any(|i| i.item_id == "MACHINE_PART")
    );
    leave(&mut game, &runtime);

    game.press(GameButton::Start)
        .expect("production Start menu");
    assert!(game.snapshot().expect("Start").ui.menu.is_some());
    game.press(GameButton::B).expect("close Start");
    for ((x, y), title) in [
        ((10, 10), "GRAND MARAIS"),
        ((12, 14), "HARBOR PARK"),
        ((26, 18), "ARTISTS POINT"),
    ] {
        walk(&mut game, &runtime, (x, y + 1), false);
        game.press(GameButton::Up).expect("face physical sign");
        game.press(GameButton::A).expect("read town sign");
        assert!(
            game.snapshot()
                .expect("sign text")
                .ui
                .text
                .as_ref()
                .and_then(|text| text.asm_text.as_deref())
                .is_some_and(|text| text.contains(title)),
            "physical sign at ({x}, {y}) did not display {title}"
        );
        text_done(&mut game);
    }
    walk(&mut game, &runtime, (2, 22), false);
    walk(&mut game, &runtime, (6, 20), false);
    assert!(talk_below(&mut game, &runtime, (7, 17)).contains("harbor"));
    assert!(talk_below(&mut game, &runtime, (19, 13)).contains("Boats are late."));
    for target in [
        "MN_GRAND_MARAIS_ARTIST_HOUSE",
        "MN_GRAND_MARAIS_FISHER_HOUSE",
    ] {
        enter(&mut game, &runtime, target);
        walk(&mut game, &runtime, (5, 4), false);
        game.press(GameButton::Up).expect("face resident");
        game.press(GameButton::A).expect("talk to resident");
        assert!(
            game.snapshot()
                .expect("resident text")
                .ui
                .pending_text_wait
                .is_some()
        );
        text_done(&mut game);
        leave(&mut game, &runtime);
    }
    enter(&mut game, &runtime, "MN_GRAND_MARAIS_LIGHTHOUSE");
    walk(&mut game, &runtime, (5, 4), false);
    game.press(GameButton::Up).expect("face keeper");
    game.press(GameButton::A).expect("deliver signal part");
    text_done(&mut game);
    let fixed = game.snapshot().expect("signal repaired");
    assert!(
        fixed
            .progression
            .active_event_flags
            .contains("EVENT_MN_GRAND_MARAIS_SIGNAL")
    );
    assert!(
        !fixed
            .bag
            .key_items
            .iter()
            .any(|i| i.item_id == "MACHINE_PART")
    );
    let reward = fixed
        .bag
        .items
        .iter()
        .find(|i| i.item_id == "SUPER_POTION")
        .expect("quest reward")
        .quantity;
    game.press(GameButton::Up).expect("face keeper again");
    game.press(GameButton::A).expect("talk again");
    assert!(
        game.snapshot()
            .expect("repeat keeper conversation")
            .ui
            .text
            .as_ref()
            .is_some_and(|text| text.label == "MnGrandMaraisSignalDoneText")
    );
    text_done(&mut game);
    assert_eq!(
        game.snapshot()
            .expect("idempotent quest")
            .bag
            .items
            .iter()
            .find(|i| i.item_id == "SUPER_POTION")
            .expect("reward retained")
            .quantity,
        reward
    );
    leave(&mut game, &runtime);

    assert!(talk_below(&mut game, &runtime, (7, 17)).contains("beacon turns"));
    assert!(talk_below(&mut game, &runtime, (19, 13)).contains("There! A boat"));
    assert!(talk_below(&mut game, &runtime, (12, 11)).contains("good news"));
    assert!(talk_below(&mut game, &runtime, (30, 29)).contains("beacon draws"));

    enter(&mut game, &runtime, "MN_GRAND_MARAIS_FIELD_STATION");
    walk(&mut game, &runtime, (5, 4), false);
    game.press(GameButton::Up).expect("face Tamarack");
    game.press(GameButton::A).expect("report repaired signal");
    assert!(
        game.snapshot()
            .expect("completion dialogue")
            .ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_deref())
            .is_some_and(|text| text.contains("harbor lamp"))
    );
    text_done(&mut game);
    let returned = game.snapshot().expect("one-time starter retained");
    assert_eq!(returned.party.slots.len(), 1);
    assert_eq!(
        returned
            .bag
            .balls
            .iter()
            .find(|item| item.item_id == "POKE_BALL")
            .expect("starter balls")
            .quantity,
        5
    );
    leave(&mut game, &runtime);

    // Enter the authored pine-grove grass, then walk until the real encounter engine fires.
    walk(&mut game, &runtime, (4, 10), true);
    for i in 0..160 {
        let encounter = game.snapshot().expect("encounter check");
        if encounter.battle.is_some()
            || matches!(
                encounter.phase,
                RuntimeShellPhase::WildBattle | RuntimeShellPhase::StaticWildBattle
            )
        {
            break;
        }
        walk(
            &mut game,
            &runtime,
            if i % 2 == 0 { (5, 10) } else { (4, 10) },
            true,
        );
    }
    for _ in 0..32 {
        let s = game.snapshot().expect("battle intro");
        if s.ui.menu.as_ref().is_some_and(|m| {
            m.layout
                .vertical_menus
                .iter()
                .flat_map(|v| &v.options)
                .any(|v| v.contains("FIGHT"))
        }) {
            break;
        }
        game.press(GameButton::A).expect("battle intro button");
    }
    assert!(
        game.snapshot()
            .expect("wild encounter started")
            .battle
            .is_some(),
        "pine grove never produced a wild battle"
    );
    game.press(GameButton::A).expect("FIGHT");
    let moves = game.snapshot().expect("move menu");
    let pp = moves.battle.as_ref().expect("battle engine").player_moves[0].current_pp;
    game.press(GameButton::A).expect("execute move");
    let turn = game.snapshot().expect("resolved turn");
    assert!(
        turn.party.slots[0].pokemon.moves[0].current_pp < pp,
        "move did not consume PP"
    );
    assert!(
        turn.ui.text.is_some(),
        "battle produced no visible dialogue"
    );
    for _ in 0..96 {
        if game.snapshot().expect("battle completion").battle.is_none() {
            break;
        }
        game.press(GameButton::A).expect("finish wild battle");
    }
    assert!(game.snapshot().expect("battle over").battle.is_none());
    text_done(&mut game);
    // Leave the grass before navigating to the center.
    let result = game.snapshot().expect("post-battle position");
    if result.overworld.map_name == "MnGrandMaraisFieldStation" {
        // A fresh game uses real, unseeded battle RNG. A legal loss must recover
        // inside this town rather than sending the player back to Johto.
        assert!(
            result.party.slots[0].pokemon.hp > 0,
            "blackout did not heal the team"
        );
        leave(&mut game, &runtime);
    } else {
        assert_eq!(result.overworld.map_name, "MnGrandMarais");
        walk(&mut game, &runtime, (3, position(&result).1), false);
    }
    enter(&mut game, &runtime, "MN_GRAND_MARAIS_POKECENTER");
    walk(&mut game, &runtime, (3, 3), false);
    game.press(GameButton::Up).expect("face nurse");
    game.press(GameButton::A).expect("request healing");
    text_done(&mut game);
    let healed = game.snapshot().expect("nurse restored team");
    let mon = &healed.party.slots[0].pokemon;
    assert_eq!(mon.hp, mon.max_hp);
    assert!(mon.moves[0].current_pp >= pp);
    walk(&mut game, &runtime, (9, 2), false);
    game.press(GameButton::Up).expect("face the visible PC");
    game.press(GameButton::A).expect("turn on the center PC");
    let mut pc_open = false;
    for _ in 0..16 {
        let snapshot = game.snapshot().expect("PC greeting or menu");
        pc_open = snapshot.ui.menu.as_ref().is_some_and(|menu| {
            menu.layout
                .vertical_menus
                .iter()
                .flat_map(|menu| &menu.options)
                .any(|entry| entry.contains("PC"))
        });
        if pc_open {
            break;
        }
        game.press(GameButton::A).expect("advance PC greeting");
    }
    assert!(
        pc_open,
        "the visible center PC did not open its production menu"
    );
    for _ in 0..32 {
        game.press(GameButton::B).expect("close center PC");
        let snapshot = game.snapshot().expect("PC closed");
        if snapshot.ui.menu.is_none()
            && !snapshot.ui.text_window_open
            && !game.has_pending_script_work()
        {
            break;
        }
    }
    leave(&mut game, &runtime);

    enter(&mut game, &runtime, "MN_GRAND_MARAIS_MART");
    walk(&mut game, &runtime, (3, 3), false);
    game.press(GameButton::Left).expect("face clerk");
    game.press(GameButton::A).expect("open shop");
    for _ in 0..8 {
        let surface = game.snapshot().expect("shop greeting");
        if surface.pending_shop.is_some() || surface.ui.menu.is_some() {
            break;
        }
        game.press(GameButton::A).expect("advance clerk greeting");
    }
    let shop = game.snapshot().expect("production shop");
    assert!(
        shop.pending_shop.is_some() || shop.ui.menu.is_some(),
        "production shop did not open"
    );
    let money = shop.trainer.money;
    for _ in 0..8 {
        game.press(GameButton::A).expect("buy first item");
        if game.snapshot().expect("shop transaction").trainer.money < money {
            break;
        }
    }
    assert!(
        game.snapshot().expect("purchase committed").trainer.money < money,
        "shop did not complete a purchase"
    );
    assert!(
        game.snapshot()
            .expect("purchased inventory")
            .bag
            .balls
            .iter()
            .find(|item| item.item_id == "POKE_BALL")
            .expect("ball inventory")
            .quantity
            > 5
    );
    for _ in 0..16 {
        game.press(GameButton::B).expect("close shop");
        let closed = game.snapshot().expect("closed shop");
        if closed.pending_shop.is_none()
            && closed.ui.menu.is_none()
            && !closed.ui.text_window_open
            && !game.has_pending_script_work()
        {
            break;
        }
    }
    leave(&mut game, &runtime);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("grand-marais-{}-{unique}", std::process::id()));
    std::fs::create_dir(&directory).expect("save test directory");
    let save = directory.join("town.crystalsave");
    let before = game.snapshot().expect("save state");
    game.save(&save).expect("save town");
    let result = VisibleShellController::load_save(root, runtime.clone(), save, None);
    std::fs::remove_dir_all(directory).expect("remove temporary saves");
    let mut resumed = result.expect("Continue town");
    let after = resumed.snapshot().expect("loaded town");
    assert_eq!(after.overworld.map_name, before.overworld.map_name);
    assert_eq!(position(&after), position(&before));
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    assert!(
        after
            .progression
            .active_event_flags
            .contains("EVENT_MN_GRAND_MARAIS_SIGNAL")
    );
    assert!(talk_below(&mut resumed, &runtime, (7, 17)).contains("beacon turns"));
}
