//! Explicit local partners/rod/badge fixtures; all subsequent play uses native input.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::LearnedMove;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleFishingPhase, VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::indoor_walk;
use super::interior_life::fixture;
use super::winona::{at, finish, load, press, travel};

const CITY: &str = "MnMankato";

fn frontage(r: &CrystalRuntime, room: &str) -> (i16, i16) {
    super::native_city_ports::city_port(r, CITY, room).0
}

fn setup() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    assert_eq!(frontage(&r, "MnMankatoPokecenter1F"), (21, 12));
    assert_eq!(frontage(&r, "MnMankatoMart"), (33, 22));
    assert_eq!(frontage(&r, "MnMankatoGym"), (10, 12));
    let mut c = fixture(&root, &r, CITY, (40, 18));
    let mut partner = r
        .data()
        .create_pokemon("CINDORE", 25, Default::default())
        .expect("original local pond companion");
    partner.item = Some("SMOKE_BALL".into());
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&r.data().items["OLD_ROD"], 1)
            .expect("declared local rod")
    );
    state.registered_key_item = Some("OLD_ROD".into());
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Local setup ends here. No later state, position, party, flag or RNG writes.
    (root, r, c)
}

fn checkpoint(c: &mut VisibleShellController, name: &str) -> PathBuf {
    let folder = PathBuf::from(
        std::env::var_os("CRYSTAL_TERRACE_POND_REVIEW_DIR")
            .expect("fresh ignored terrace/pond output"),
    );
    std::fs::create_dir_all(&folder).expect("native pond review saves");
    let path = folder.join(format!("{name}.crystalsave"));
    assert!(!path.exists(), "retain every played checkpoint");
    c.save(&path).expect("save actual pond progress");
    path
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, name: &str) {
    let before = c.snapshot().expect("before pond Continue");
    let state = c.shell.shell.session().state().clone();
    *c = VisibleShellController::load_save(root.clone(), r.clone(), checkpoint(c, name), None)
        .expect("native terrace/pond Continue");
    c.set_runtime_journal_enabled(false);
    let after = c.snapshot().expect("continued pond state");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(state.flags, c.shell.shell.session().state().flags);
    assert_eq!(state.badges, c.shell.shell.session().state().badges);
    assert_eq!(state.money, c.shell.shell.session().state().money);
}

#[test]
#[ignore = "requires current terrace/pond candidate and fresh CRYSTAL_TERRACE_POND_REVIEW_DIR"]
fn terrace_pond_controller_three_doors_six_carpets_four_posts_and_continue() {
    let (root, r, mut c) = setup();
    let town = r
        .data()
        .assemble_map_module_from_compiled_payloads(CITY)
        .expect("installed native split-payload town");
    assert_eq!(town.events.warps.len(), 6);
    let (pairs, rest) = town.events.warps.as_chunks::<2>();
    assert!(rest.is_empty());
    let mut carpets = 0;
    for pair in pairs {
        let room = r
            .data()
            .maps
            .values()
            .find(|m| m.attributes.map_constant.as_deref() == Some(&pair[0].target_map_constant))
            .expect("unchanged actual room");
        let arrival = room
            .events
            .warps
            .iter()
            .find(|warp| i16::try_from(warp.index).ok() == Some(pair[0].target_warp_id))
            .expect("native passive room arrival");
        let inside = (
            i16::try_from(arrival.x).expect("x"),
            i16::try_from(arrival.y).expect("y"),
        );
        let outside = frontage(&r, &room.id);
        for exit in room.events.warps.iter().filter(|warp| {
            warp.target_map_constant == "MN_MANKATO"
                && i16::try_from(warp.index).ok() != Some(pair[0].target_warp_id)
        }) {
            indoor_walk(&mut c, &r, outside);
            travel(&mut c, GameButton::Up, &room.id, inside);
            indoor_walk(
                &mut c,
                &r,
                (
                    i16::try_from(exit.x).expect("x"),
                    i16::try_from(exit.y).expect("y"),
                ),
            );
            if c.snapshot().expect("carpet").overworld.map_name != CITY {
                travel(&mut c, GameButton::Down, CITY, outside);
            }
            at(&mut c, CITY, outside);
            continued(
                &mut c,
                &root,
                &r,
                &format!("{}-carpet-{}", room.id, exit.index),
            );
            carpets += 1;
        }
    }
    assert_eq!(carpets, 6);
    for event in &town.events.bg_events {
        let stand = (
            i16::try_from(event.x + 1).expect("x"),
            i16::try_from(event.y).expect("y"),
        );
        indoor_walk(&mut c, &r, stand);
        press(&mut c, GameButton::Left);
        at(&mut c, CITY, stand);
        press(&mut c, GameButton::A);
        let snapshot = c.snapshot().expect("actual sign reading");
        assert_eq!(town.scripts[&event.script][0]["command"], "jumptext");
        assert_eq!(
            snapshot.ui.text.expect("native text").label,
            town.scripts[&event.script][0]["args"][0]
                .as_str()
                .expect("authored sign target")
        );
        press(&mut c, GameButton::Down);
        at(&mut c, CITY, stand);
        finish(&mut c);
    }
    checkpoint(&mut c, "three-door-court");
}

#[test]
#[ignore = "requires terrace/pond pack and fresh CRYSTAL_TERRACE_POND_REVIEW_DIR"]
fn terrace_pond_controller_clear_promenades_open_banks_casting_and_continue() {
    let (root, r, mut c) = setup();
    for (x, y, width, height) in [(6, 12, 20, 2), (30, 32, 14, 2), (36, 21, 6, 2)] {
        for yy in y..y + height {
            indoor_walk(&mut c, &r, (x, yy));
            for xx in x + 1..x + width {
                travel(&mut c, GameButton::Right, CITY, (xx, yy));
            }
        }
    }
    for (x, y, height) in [(30, 26, 8), (42, 22, 12)] {
        for xx in [x, x + 1] {
            indoor_walk(&mut c, &r, (xx, y));
            for yy in y + 1..y + height {
                travel(&mut c, GameButton::Down, CITY, (xx, yy));
            }
        }
    }
    for (stand, blocked) in [((38, 22), GameButton::Down), ((36, 31), GameButton::Up)] {
        indoor_walk(&mut c, &r, stand);
        press(&mut c, blocked);
        at(&mut c, CITY, stand);
        checkpoint(&mut c, &format!("rail-{}-{}", stand.0, stand.1));
    }
    for (bank, facing) in [((31, 26), GameButton::Right), ((42, 26), GameButton::Left)] {
        indoor_walk(&mut c, &r, bank);
        assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
        let before = c.snapshot().expect("ordinary dry pond bank");
        assert_eq!(before.overworld.mode, MovementMode::Normal);
        assert!(before.party.slots.iter().all(|slot| {
            slot.pokemon
                .moves
                .iter()
                .all(|m| !matches!(m.name.as_str(), "SURF" | "CUT" | "WATERFALL"))
        }));
        press(&mut c, facing);
        at(&mut c, CITY, bank);
        assert_eq!(
            c.snapshot()
                .expect("missing Surf remains dry")
                .overworld
                .mode,
            MovementMode::Normal
        );
        let mut bite = false;
        for _ in 0..64 {
            press(&mut c, GameButton::Select);
            assert!(
                c.shell.visible_fishing_animation.is_some(),
                "native registered rod animation"
            );
            for _ in 0..512 {
                c.wait_frames(1).expect("native casting clock");
                if c.shell
                    .visible_fishing_animation
                    .as_ref()
                    .is_some_and(|a| a.phase == VisibleFishingPhase::AwaitText)
                {
                    break;
                }
            }
            for _ in 0..128 {
                let s = c.snapshot().expect("native rod result");
                if s.battle.is_some()
                    || (c.shell.visible_fishing_animation.is_none()
                        && c.shell.field_notice.is_none()
                        && !c.has_pending_script_work())
                {
                    break;
                }
                press(&mut c, GameButton::A);
            }
            if let Some(battle) = c.snapshot().expect("native pond fish").battle {
                assert!(matches!(
                    battle.enemy_pokemon.species.id.as_str(),
                    "SCUTLET" | "CHORIP"
                ));
                checkpoint(&mut c, &format!("actual-fish-{}-{}", bank.0, bank.1));
                super::rochester::flee(&mut c);
                bite = true;
                break;
            }
            finish(&mut c);
        }
        assert!(bite, "ordinary native pond casting must produce a bite");
        at(&mut c, CITY, bank);
        assert_eq!(c.snapshot().expect("after native RUN").party, before.party);
        assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
        continued(
            &mut c,
            &root,
            &r,
            &format!("bank-{}-{}-continue", bank.0, bank.1),
        );
    }
    indoor_walk(&mut c, &r, (40, 18));
    press(&mut c, GameButton::Start);
    assert!(
        c.snapshot()
            .expect("Start after pond return")
            .ui
            .menu
            .is_some()
    );
}

fn paddle(c: &mut VisibleShellController, water: &BTreeSet<(i16, i16)>, target: (i16, i16)) {
    let tile = c.snapshot().expect("actual pond position").overworld.tile;
    let start = (tile.x, tile.y);
    let mut previous = BTreeMap::new();
    let mut found = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == target {
            break;
        }
        for (at, button) in [
            ((x - 1, y), GameButton::Left),
            ((x + 1, y), GameButton::Right),
            ((x, y - 1), GameButton::Up),
            ((x, y + 1), GameButton::Down),
        ] {
            if water.contains(&at) && found.insert(at) {
                previous.insert(at, ((x, y), button));
                queue.push_back(at);
            }
        }
    }
    assert!(found.contains(&target), "connected native pond water");
    let mut at = target;
    let mut route = Vec::new();
    while at != start {
        let (before, button) = previous[&at];
        route.push((button, at));
        at = before;
    }
    for (button, at) in route.into_iter().rev() {
        travel(c, button, CITY, at);
        assert_eq!(
            c.snapshot().expect("native paddle mode").overworld.mode,
            MovementMode::Surf
        );
    }
}

#[test]
#[ignore = "requires terrace/pond pack and fresh CRYSTAL_TERRACE_POND_REVIEW_DIR"]
fn terrace_pond_controller_all_water_native_surf_rails_continue_and_dismount() {
    let (root, r) = load();
    let mut c = fixture(&root, &r, CITY, (31, 26));
    let mut partner = r
        .data()
        .create_pokemon("VOYAGEAR", 25, Default::default())
        .expect("original Minnesota Surf learner");
    assert!(partner.species.tmhm_learnset.iter().any(|m| m == "SURF"));
    partner.moves.retain(|m| m.name != "SURF");
    partner.moves.truncate(3);
    partner.moves.push(LearnedMove {
        name: "SURF".into(),
        current_pp: r.data().moves["SURF"].pp,
        pp_ups: 0,
    });
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.badges.johto[3] = true;
    state
        .flags
        .set_engine_flag("ENGINE_FOGBADGE", true)
        .expect("declared local Surf permission");
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Initial local partner/permission ends here; all later travel uses input.
    press(&mut c, GameButton::Right);
    for _ in 0..128 {
        if c.snapshot().expect("native Surf prompt").overworld.mode == MovementMode::Surf {
            break;
        }
        press(&mut c, GameButton::A);
    }
    c.wait_frames(60).expect("native mounting presentation");
    at(&mut c, CITY, (32, 26));
    assert_eq!(
        c.snapshot().expect("mounted pond").overworld.mode,
        MovementMode::Surf
    );
    let map = &r.data().maps[CITY];
    let water = (0..36)
        .flat_map(|y| (0..48).map(move |x| (x, y)))
        .filter(|&at| super::interior_life::collision(&r, map, at) == Some("WATER"))
        .collect::<BTreeSet<_>>();
    assert_eq!(water.len(), 52);
    for &target in &water {
        paddle(&mut c, &water, target);
    }
    checkpoint(&mut c, "all-52-water-tiles-played");
    for (at, button) in [((38, 24), GameButton::Up), ((36, 29), GameButton::Down)] {
        paddle(&mut c, &water, at);
        press(&mut c, button);
        super::winona::at(&mut c, CITY, at);
        assert_eq!(
            c.snapshot().expect("rail rejects Surf").overworld.mode,
            MovementMode::Surf
        );
    }
    continued(&mut c, &root, &r, "surf-at-south-rail-continue");
    paddle(&mut c, &water, (32, 26));
    travel(&mut c, GameButton::Left, CITY, (31, 26));
    c.wait_frames(60).expect("native dismount presentation");
    at(&mut c, CITY, (31, 26));
    assert_eq!(
        c.snapshot().expect("dry bank return").overworld.mode,
        MovementMode::Normal
    );
    checkpoint(&mut c, "native-surf-bank-return");
}
