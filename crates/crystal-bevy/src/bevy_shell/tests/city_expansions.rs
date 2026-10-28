//! Explicit city/party/money setup; all subsequent travel and purchases are input.

#[path = "city_expansion_duluth.rs"]
mod duluth;
#[path = "city_expansion_minneapolis.rs"]
mod minneapolis;
#[path = "city_expansion_st_paul.rs"]
mod st_paul;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::{
    DIRECTIONS, VisibleShellController, at, battle, continue_save, controller, event, finish, load,
    module, point, press,
};

// A local walk must not enter an unrelated door or carpet. The parent planner
// also serves gym lifts, so constrain only this city's scratch planning module.
fn path(
    r: &CrystalRuntime,
    map: &crystal_assets::MapModule,
    from: (u16, u16),
    to: (u16, u16),
) -> Option<Vec<GameButton>> {
    let mut local = map.clone();
    for warp in &map.events.warps {
        if active(r, &map.id, warp.x, warp.y) {
            let mut obstacle = r.data().maps["NewBarkTown"].objects[0].clone();
            obstacle.x = warp.x;
            obstacle.y = warp.y;
            local.objects.push(obstacle);
        }
    }
    super::path(r, &local, from, to)
}

fn walk_to(c: &mut VisibleShellController, r: &CrystalRuntime, to: (u16, u16)) {
    let map = module(c, r);
    let from = at(c);
    let route = path(r, &map, from, to)
        .unwrap_or_else(|| panic!("local city walk {}: {from:?} -> {to:?}", map.id));
    for button in route {
        let before = at(c);
        for _ in 0..8 {
            press(c, button);
            if at(c) != before {
                break;
            }
        }
        // press() already settles twenty frames. City walking has no ice or
        // conveyor continuation requiring the gym planner's extra eighty.
        assert_eq!(c.snapshot().expect("local walk").overworld.map_name, map.id);
    }
    assert_eq!(at(c), to);
}

fn face_actor(c: &mut VisibleShellController, tile: (u16, u16)) {
    let from = at(c);
    let button = match (tile.0 as i16 - from.0 as i16, tile.1 as i16 - from.1 as i16) {
        (0, dy) if dy < 0 => GameButton::Up,
        (0, _) => GameButton::Down,
        (dx, 0) if dx < 0 => GameButton::Left,
        (_, 0) => GameButton::Right,
        _ => panic!("actor approach must be aligned"),
    };
    press(c, button);
    assert_eq!(at(c), from, "actor or counter must own collision");
    press(c, GameButton::A);
}

fn interact(c: &mut VisibleShellController, r: &CrystalRuntime, tile: (u16, u16)) {
    let map = module(c, r);
    let from = at(c);
    let mut choices = Vec::new();
    for (dx, dy, _) in DIRECTIONS {
        let (x, y) = (tile.0 as i16 - dx, tile.1 as i16 - dy);
        if x >= 0
            && y >= 0
            && x < map.attributes.width as i16 * 2
            && y < map.attributes.height as i16 * 2
        {
            let stand = (x as u16, y as u16);
            if let Some(route) = path(r, &map, from, stand) {
                choices.push((route.len(), stand));
            }
        }
    }
    choices.sort_by_key(|(length, _)| *length);
    let (_, stand) = choices.first().copied().expect("local actor approach");
    walk_to(c, r, stand);
    face_actor(c, tile);
}

fn contract() -> Value {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_CITY_EXPANSION_CONTRACT")
            .expect("set CRYSTAL_CITY_EXPANSION_CONTRACT to the published city inputs"),
    );
    serde_json::from_str(&std::fs::read_to_string(path).expect("explicit city contract"))
        .expect("city contract")
}

fn active(r: &CrystalRuntime, map: &str, x: u16, y: u16) -> bool {
    let m = &r.data().maps[map];
    let block = m.blocks[usize::from(y / 2) * usize::from(m.attributes.width) + usize::from(x / 2)];
    let collision = &r.data().tilesets[&m.attributes.tileset_name].collision
        [&format!("{block:02x}")][usize::from(y % 2 * 2 + x % 2)];
    collision != "FLOOR"
}

fn warp(c: &mut VisibleShellController, r: &CrystalRuntime, index: u16) {
    let map = module(c, r);
    let source = map
        .events
        .warps
        .iter()
        .find(|warp| warp.index == index)
        .expect("actual native warp");
    let target = r
        .data()
        .runtime_map_metadata
        .get(&source.target_map_constant)
        .expect("native target metadata");
    let destination = r.data().maps[&target.name]
        .events
        .warps
        .iter()
        .find(|warp| i16::try_from(warp.index).ok() == Some(source.target_warp_id))
        .expect("actual native destination");
    let target_map = &r.data().maps[&target.name];
    let block = target_map.blocks[usize::from(destination.y / 2)
        * usize::from(target_map.attributes.width)
        + usize::from(destination.x / 2)];
    let destination_kind = &r.data().tilesets[&target_map.attributes.tileset_name].collision
        [&format!("{block:02x}")][usize::from(destination.y % 2 * 2 + destination.x % 2)];
    // Native CheckTile walks down after arriving on 7a stairs/doors/caves.
    // Declared passive Floor destinations must still remain exactly in place.
    let settled = (
        destination.x,
        destination.y
            + u16::from(matches!(
                destination_kind.as_str(),
                "STAIRCASE" | "DOOR" | "DOOR_79" | "CAVE"
            )),
    );
    let before = at(c);
    let block = map.blocks
        [usize::from(source.y / 2) * usize::from(map.attributes.width) + usize::from(source.x / 2)];
    let token = &r.data().tilesets[&map.attributes.tileset_name].collision[&format!("{block:02x}")]
        [usize::from(source.y % 2 * 2 + source.x % 2)];
    let required = match token.as_str() {
        "WARP_CARPET_DOWN" => Some(GameButton::Down),
        "WARP_CARPET_UP" => Some(GameButton::Up),
        "WARP_CARPET_LEFT" => Some(GameButton::Left),
        "WARP_CARPET_RIGHT" => Some(GameButton::Right),
        _ => None,
    };
    let mut approaches = Vec::new();
    for (dx, dy, button) in DIRECTIONS {
        if required.is_some_and(|required| required != button) {
            continue;
        }
        let (x, y) = (
            i16::try_from(source.x).expect("x") - dx,
            i16::try_from(source.y).expect("y") - dy,
        );
        if x >= 0
            && y >= 0
            && x < map.attributes.width as i16 * 2
            && y < map.attributes.height as i16 * 2
            && let Some(route) = path(r, &map, before, (x as u16, y as u16))
        {
            approaches.push((route.len(), (x as u16, y as u16), button));
        }
    }
    approaches.sort_by_key(|(length, _, _)| *length);
    let (_, stand, button) = approaches.first().copied().expect("real walk to warp");
    walk_to(c, r, stand);
    for _ in 0..8 {
        press(c, button);
        let snapshot = c.snapshot().expect("warp input");
        if snapshot.overworld.map_name == target.name
            && snapshot.overworld.tile.x == settled.0 as i16
            && snapshot.overworld.tile.y == settled.1 as i16
        {
            return;
        }
    }
    let snapshot = c.snapshot().expect("failed warp");
    panic!(
        "native warp {} {} did not reach {} ({},{}); actual={} {:?}, text={:?}",
        map.id,
        index,
        target.name,
        destination.x,
        destination.y,
        snapshot.overworld.map_name,
        snapshot.overworld.tile,
        snapshot.ui.text
    );
}

fn navigate(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    target: &str,
    allowed: &BTreeSet<String>,
) {
    let start = c.snapshot().expect("city location").overworld.map_name;
    if start == target {
        return;
    }
    let mut queue = VecDeque::from([start.clone()]);
    let mut parents = BTreeMap::from([(start.clone(), None)]);
    while let Some(name) = queue.pop_front() {
        if name == target {
            break;
        }
        for source in &r.data().maps[&name].events.warps {
            if !active(r, &name, source.x, source.y) {
                continue;
            }
            let destination = &r.data().runtime_map_metadata[&source.target_map_constant].name;
            if allowed.contains(destination) && !parents.contains_key(destination) {
                parents.insert(destination.clone(), Some((name.clone(), source.index)));
                queue.push_back(destination.clone());
            }
        }
    }
    let mut course = Vec::new();
    let mut current = target.to_owned();
    while current != start {
        let (previous, index) = parents
            .get(&current)
            .expect("real building connection graph")
            .as_ref()
            .expect("predecessor");
        course.push(*index);
        current = previous.clone();
    }
    for index in course.into_iter().rev() {
        warp(c, r, index);
    }
    assert_eq!(
        c.snapshot()
            .expect("building destination")
            .overworld
            .map_name,
        target
    );
}

fn close_shop(c: &mut VisibleShellController) {
    for _ in 0..96 {
        let s = c.snapshot().expect("shop logoff");
        if s.pending_shop.is_none()
            && !s.ui.text_window_open
            && s.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            return;
        }
        let button = if s.pending_shop.is_none() || c.shell.shop_notice.is_some() {
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
        press(c, button);
    }
    panic!("city shop did not release input");
}

fn buy(c: &mut VisibleShellController, r: &CrystalRuntime, actor: &Value) {
    if let Some(approach) = actor.get("approach").filter(|value| value.is_array()) {
        walk_to(c, r, point(approach));
        let p = point(&actor["tile"]);
        let from = at(c);
        let button = match (p.0 as i16 - from.0 as i16, p.1 as i16 - from.1 as i16) {
            (0, _) if p.1 < from.1 => GameButton::Up,
            (0, _) => GameButton::Down,
            (_, 0) if p.0 < from.0 => GameButton::Left,
            (_, 0) => GameButton::Right,
            _ => panic!("vendor approach is not aligned"),
        };
        press(c, button);
        press(c, GameButton::A);
    } else {
        interact(c, r, point(&actor["tile"]));
    }
    for _ in 0..64 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let shop = c
        .snapshot()
        .expect("new native shop")
        .pending_shop
        .expect("native mart pending");
    let stock = actor["stock"]
        .as_array()
        .expect("explicit stock")
        .iter()
        .map(|v| v.as_str().expect("item").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(shop.inventory, stock);
    let item = &r.data().items[&stock[0]];
    let before = (
        c.shell.shell.session().state().money,
        c.shell.shell.session().state().bag.quantity(item),
    );
    press(c, GameButton::A);
    press(c, GameButton::A);
    assert_eq!(
        c.shell
            .shop_quantity
            .as_ref()
            .expect("native quantity")
            .item_id,
        stock[0]
    );
    press(c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        before.0,
        "confirmation charges nothing"
    );
    press(c, GameButton::A);
    assert_eq!(
        c.shell.shell.session().state().money,
        before.0 - u32::from(item.price)
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before.1 + 1
    );
    close_shop(c);
}

#[test]
#[ignore = "requires the published expanded city and its exact contract"]
fn city_expansion_controller_all_facades_stairs_and_distinct_priced_shops() {
    let plan = contract();
    let (root, r, gyms) = load();
    let mut setup = gyms["gyms"][2].clone();
    setup["id"] = plan["city"].clone();
    setup["arrival_core"] = plan["arrival_core"].clone();
    let mut c = controller(&root, &r, &setup, false);
    c.shell.shell.session_mut().state_mut().money = 999999; // Explicit spending fixture.
    let city = plan["city"].as_str().expect("city");
    let rooms = plan["rooms"].as_array().expect("rooms");
    let mut allowed = rooms
        .iter()
        .map(|room| room["id"].as_str().expect("room").to_owned())
        .chain([city.to_owned()])
        .collect::<BTreeSet<_>>();
    let first_new = plan["doors"]
        .as_array()
        .expect("new doors")
        .first()
        .expect("new facade")["warp_index"]
        .as_u64()
        .expect("first new warp");
    let original_doors = r.data().maps[city]
        .events
        .warps
        .iter()
        .filter(|warp| u64::from(warp.index) < first_new && active(&r, city, warp.x, warp.y))
        .cloned()
        .collect::<Vec<_>>();
    for door in &original_doors {
        allowed.insert(
            r.data().runtime_map_metadata[&door.target_map_constant]
                .name
                .clone(),
        );
    }
    for actor in plan["residents"].as_array().expect("city residents") {
        if actor["service"] == "pokemart" {
            buy(&mut c, &r, actor);
        }
    }
    for door in original_doors {
        navigate(&mut c, &r, city, &allowed);
        warp(&mut c, &r, door.index);
        finish(&mut c);
        let name = c
            .snapshot()
            .expect("preserved original facade")
            .overworld
            .map_name;
        let exits = r.data().maps[&name]
            .events
            .warps
            .iter()
            .filter(|exit| {
                exit.target_map_constant
                    == r.data().maps[city]
                        .attributes
                        .map_constant
                        .as_deref()
                        .expect("city constant")
                    && active(&r, &name, exit.x, exit.y)
            })
            .cloned()
            .collect::<Vec<_>>();
        assert!(!exits.is_empty(), "original room must return to its city");
        for exit in exits {
            navigate(&mut c, &r, &name, &allowed);
            warp(&mut c, &r, exit.index);
            finish(&mut c);
            assert_eq!(
                c.snapshot()
                    .expect("original room return")
                    .overworld
                    .map_name,
                city
            );
        }
    }
    for door in plan["doors"].as_array().expect("real facades") {
        navigate(&mut c, &r, city, &allowed);
        warp(
            &mut c,
            &r,
            door["warp_index"].as_u64().expect("door index") as u16,
        );
        assert_eq!(
            c.snapshot().expect("entered real room").overworld.map_name,
            door["target"].as_str().expect("facade target")
        );
        let entered = module(&mut c, &r);
        let arrival = entered
            .events
            .warps
            .iter()
            .find(|warp| {
                u64::from(warp.index) == door["target_warp"].as_u64().expect("room arrival index")
            })
            .expect("passive room arrival");
        assert_eq!(at(&mut c), (arrival.x, arrival.y));
        navigate(&mut c, &r, city, &allowed);
        assert_eq!(at(&mut c), point(&door["return_tile"]));
    }
    for room in rooms {
        let name = room["id"].as_str().expect("room");
        navigate(&mut c, &r, name, &allowed);
        for actor in room["residents"].as_array().expect("room residents") {
            if actor["service"] == "pokemart" {
                buy(&mut c, &r, actor);
            }
        }
        let exits = room["warps"].as_array().expect("native exits");
        for exit in exits.iter().filter(|exit| exit["passive"] == false) {
            navigate(&mut c, &r, name, &allowed);
            warp(
                &mut c,
                &r,
                exit["index"].as_u64().expect("warp index") as u16,
            );
        }
    }
    navigate(&mut c, &r, city, &allowed);
    let before = (
        at(&mut c),
        c.shell.shell.session().state().money,
        c.shell.shell.session().state().bag.clone(),
    );
    c = continue_save(&mut c, &root, &r);
    assert_eq!(at(&mut c), before.0);
    assert_eq!(c.shell.shell.session().state().money, before.1);
    assert_eq!(c.shell.shell.session().state().bag, before.2);
}

#[test]
#[ignore = "strong legal review party; optional city battles require published expansion"]
fn city_expansion_controller_optional_trainers_spend_pp_and_persist() {
    let plan = contract();
    let (root, r, gyms) = load();
    let mut setup = gyms["gyms"][2].clone();
    setup["id"] = plan["city"].clone();
    setup["arrival_core"] = plan["arrival_core"].clone();
    let mut c = controller(&root, &r, &setup, false);
    let city = plan["city"].as_str().expect("city");
    let rooms = plan["rooms"].as_array().expect("rooms");
    let allowed = rooms
        .iter()
        .map(|room| room["id"].as_str().expect("room").to_owned())
        .chain([city.to_owned()])
        .collect::<BTreeSet<_>>();
    for (name, trainers) in std::iter::once((city, &plan["trainers"])).chain(
        rooms
            .iter()
            .map(|room| (room["id"].as_str().expect("room"), &room["trainers"])),
    ) {
        for trainer in trainers.as_array().expect("optional trainer records") {
            navigate(&mut c, &r, name, &allowed);
            let flag = trainer["defeated_flag"].as_str().expect("trainer flag");
            assert!(!event(&c, flag));
            let pp = c.snapshot().expect("legal battle partner").party.slots[0]
                .pokemon
                .moves
                .iter()
                .map(|m| u16::from(m.current_pp))
                .sum::<u16>();
            interact(&mut c, &r, point(&trainer["tile"]));
            battle(&mut c, false);
            finish(&mut c);
            assert!(event(&c, flag));
            assert!(
                c.snapshot().expect("native PP").party.slots[0]
                    .pokemon
                    .moves
                    .iter()
                    .map(|m| u16::from(m.current_pp))
                    .sum::<u16>()
                    < pp
            );
            c = continue_save(&mut c, &root, &r);
            assert!(event(&c, flag));
            interact(&mut c, &r, point(&trainer["tile"]));
            finish(&mut c);
            assert!(c.snapshot().expect("already defeated").battle.is_none());
        }
    }
    assert!(
        !event(&c, "EVENT_MN_GYM_3_DEFEATED"),
        "optional trainers do not grant a city badge"
    );
}

#[test]
#[ignore = "injury and empty-PP setup; actual new clinic care and Continue require expansion"]
fn city_expansion_controller_free_care_owns_input_restores_resources_and_binds_clinic() {
    let plan = contract();
    let (root, r, gyms) = load();
    let mut services = vec![(
        plan["city"].as_str().expect("city"),
        &plan["arrival_core"],
        &plan["residents"],
    )];
    for room in plan["rooms"].as_array().expect("rooms") {
        services.push((
            room["id"].as_str().expect("room"),
            &room["arrival_core"],
            &room["residents"],
        ));
    }
    let mut count = 0;
    for (name, arrival, residents) in services {
        for healer in residents
            .as_array()
            .expect("resident records")
            .iter()
            .filter(|actor| actor["service"] == "heal_party")
        {
            count += 1;
            let mut setup = gyms["gyms"][2].clone();
            setup["id"] = Value::String(name.into());
            setup["arrival_core"] = arrival.clone();
            let mut c = controller(&root, &r, &setup, false);
            let state = c.shell.shell.session_mut().state_mut();
            let partner = state.storage.party.pokemon[0]
                .as_mut()
                .expect("legal review partner");
            partner.hp = partner.max_hp.saturating_sub(30).max(1);
            partner.status = Some("PARALYSIS".into());
            for learned in &mut partner.moves {
                learned.current_pp = 0;
            }
            state.money = 4321;
            state.sync_party_from_storage();
            super::mark_runtime_snapshot_dirty(&mut c.shell);
            if let Some(approach) = healer.get("approach").filter(|value| value.is_array()) {
                walk_to(&mut c, &r, point(approach));
                let p = point(&healer["tile"]);
                let from = at(&mut c);
                let button = if p.0 == from.0 {
                    if p.1 < from.1 {
                        GameButton::Up
                    } else {
                        GameButton::Down
                    }
                } else if p.0 < from.0 {
                    GameButton::Left
                } else {
                    GameButton::Right
                };
                press(&mut c, button);
                press(&mut c, GameButton::A);
            } else {
                interact(&mut c, &r, point(&healer["tile"]));
            }
            let stand = at(&mut c);
            assert!(c.snapshot().expect("healer dialogue").ui.text.is_some());
            press(&mut c, GameButton::Left);
            assert_eq!(at(&mut c), stand, "dialogue owns movement");
            finish(&mut c);
            let snapshot = c.snapshot().expect("native care");
            let partner = &snapshot.party.slots[0].pokemon;
            assert_eq!(partner.hp, partner.max_hp);
            assert!(partner.status.is_none());
            assert!(
                partner
                    .moves
                    .iter()
                    .all(|learned| learned.current_pp == r.data().moves[&learned.name].pp)
            );
            assert_eq!(c.shell.shell.session().state().money, 4321);
            assert_eq!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .last_spawn_map_constant
                    .as_deref(),
                plan["clinic_constant"].as_str()
            );
            c = continue_save(&mut c, &root, &r);
            let snapshot = c.snapshot().expect("continued care");
            assert_eq!(snapshot.party.slots[0].pokemon.hp, partner.max_hp);
            assert!(
                snapshot.party.slots[0]
                    .pokemon
                    .moves
                    .iter()
                    .all(|learned| learned.current_pp == r.data().moves[&learned.name].pp)
            );
        }
    }
    assert!(
        count > 0,
        "the expanded city needs its authored actual care services"
    );
}

#[test]
#[ignore = "legal weak-party setup; care and actual trainer loss must use published city"]
fn city_expansion_controller_actual_optional_loss_recovers_at_the_bound_clinic() {
    let plan = contract();
    let (root, r, gyms) = load();
    let rooms = plan["rooms"].as_array().expect("rooms");
    let city = plan["city"].as_str().expect("city");
    let allowed = rooms
        .iter()
        .map(|room| room["id"].as_str().expect("room").to_owned())
        .chain([city.to_owned()])
        .collect::<BTreeSet<_>>();
    let (care, healer) = rooms
        .iter()
        .find_map(|room| {
            room["residents"]
                .as_array()
                .expect("residents")
                .iter()
                .find(|actor| actor["service"] == "heal_party")
                .map(|actor| (room, actor))
        })
        .expect("real new care");
    let mut setup = gyms["gyms"][2].clone();
    setup["id"] = care["id"].clone();
    setup["arrival_core"] = care["arrival_core"].clone();
    let mut c = controller(&root, &r, &setup, true);
    if let Some(approach) = healer.get("approach").filter(|value| value.is_array()) {
        walk_to(&mut c, &r, point(approach));
        face_actor(&mut c, point(&healer["tile"]));
    } else {
        interact(&mut c, &r, point(&healer["tile"]));
    }
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        plan["clinic_constant"].as_str()
    );
    let (name, trainer) = rooms
        .iter()
        .find_map(|room| {
            room["trainers"]
                .as_array()
                .expect("trainers")
                .first()
                .map(|trainer| (room["id"].as_str().expect("room"), trainer))
        })
        .or_else(|| {
            plan["trainers"]
                .as_array()
                .expect("city trainers")
                .first()
                .map(|trainer| (city, trainer))
        })
        .expect("optional battle destination");
    navigate(&mut c, &r, name, &allowed);
    interact(&mut c, &r, point(&trainer["tile"]));
    battle(&mut c, true);
    finish(&mut c);
    assert!(!event(
        &c,
        trainer["defeated_flag"].as_str().expect("trainer flag")
    ));
    let clinic =
        &r.data().runtime_map_metadata[plan["clinic_constant"].as_str().expect("clinic")].name;
    assert_eq!(
        c.snapshot()
            .expect("real blackout recovery")
            .overworld
            .map_name,
        *clinic
    );
    let partner = &c.snapshot().expect("blackout care").party.slots[0].pokemon;
    assert_eq!(partner.hp, partner.max_hp);
    c = continue_save(&mut c, &root, &r);
    assert_eq!(
        c.snapshot()
            .expect("continued clinic recovery")
            .overworld
            .map_name,
        *clinic
    );
}

#[test]
#[ignore = "explicit prior-badge/empty-party road fixture; actual moved seams and guard input"]
fn city_expansion_controller_both_road_lanes_return_freely_and_keep_original_refusals() {
    let plan = contract();
    let (root, r, gyms) = load();
    let city = plan["city"].as_str().expect("city");
    for link in plan["connections"].as_array().expect("native seams") {
        let (dx, dy, outward, inward) = match link["direction"].as_str().expect("direction") {
            "north" => (0, -1, GameButton::Up, GameButton::Down),
            "south" => (0, 1, GameButton::Down, GameButton::Up),
            "west" => (-1, 0, GameButton::Left, GameButton::Right),
            "east" => (1, 0, GameButton::Right, GameButton::Left),
            _ => panic!("cardinal city seam"),
        };
        for lane in 0..2 {
            let mut setup = gyms["gyms"][2].clone();
            setup["id"] = link["target"].clone();
            setup["arrival_core"] = link["route_edge"][lane].clone();
            let mut c = controller(&root, &r, &setup, false);
            let state = c.shell.shell.session_mut().state_mut();
            state.storage.party.pokemon.fill(None);
            state.sync_party_from_storage();
            super::mark_runtime_snapshot_dirty(&mut c.shell);
            let edge = point(&link["town_edge"][lane]);
            let inside = (
                (edge.0 as i16 - dx * 4) as u16,
                (edge.1 as i16 - dy * 4) as u16,
            );
            for _ in 0..16 {
                press(&mut c, inward);
                let snapshot = c.snapshot().expect("unqualified city return");
                assert!(
                    snapshot.ui.text.is_none(),
                    "inbound return must be free of qualification dialogue"
                );
                if snapshot.overworld.map_name == city && at(&mut c) == inside {
                    break;
                }
            }
            assert_eq!(
                c.snapshot().expect("native city seam").overworld.map_name,
                city
            );
            assert_eq!(at(&mut c), inside);
            let trigger = (
                (edge.0 as i16 - dx * 3) as u16,
                (edge.1 as i16 - dy * 3) as u16,
            );
            // A compact fixture declares the complete original checkpoint
            // inventory at its new coordinates. Losing a guard must fail this
            // test, rather than silently select the open-road branch.
            let actual_events = &r.data().maps[city].events.coord_events;
            let has_guard = if let Some(expected) = plan.get("controller_coordinate_events") {
                assert_eq!(
                    &serde_json::to_value(actual_events).expect("native checkpoints"),
                    expected,
                    "all original checkpoint records must retain their explicit coordinates"
                );
                expected
                    .as_array()
                    .expect("declared checkpoints")
                    .iter()
                    .any(|event| {
                        event["x"].as_u64() == Some(u64::from(trigger.0))
                            && event["y"].as_u64() == Some(u64::from(trigger.1))
                            && !event["script_name"]
                                .as_str()
                                .expect("checkpoint script")
                                .ends_with("Inward")
                    })
            } else {
                actual_events.iter().any(|event| {
                    (event.x, event.y) == trigger && !event.script_name.ends_with("Inward")
                })
            };
            if has_guard {
                for _ in 0..8 {
                    press(&mut c, outward);
                    if c.snapshot().expect("original checkpoint").ui.text.is_some() {
                        break;
                    }
                }
                assert!(
                    c.snapshot()
                        .expect("qualification refusal")
                        .ui
                        .text
                        .is_some()
                );
                let held = at(&mut c);
                press(&mut c, outward);
                assert_eq!(at(&mut c), held, "refusal dialogue owns input");
                finish(&mut c);
                assert_eq!(at(&mut c), inside);
            } else {
                let target = link["target"].as_str().expect("neighbor");
                for _ in 0..16 {
                    press(&mut c, outward);
                    if c.snapshot()
                        .expect("open reciprocal road")
                        .overworld
                        .map_name
                        == target
                    {
                        break;
                    }
                }
                assert_eq!(
                    c.snapshot()
                        .expect("native reciprocal seam")
                        .overworld
                        .map_name,
                    target
                );
                assert_eq!(at(&mut c), point(&link["route_edge"][lane]));
            }
            assert!(!event(&c, "EVENT_MN_GYM_3_DEFEATED"));
            assert_eq!(c.shell.shell.session().state().money, 0);
        }
    }
}
