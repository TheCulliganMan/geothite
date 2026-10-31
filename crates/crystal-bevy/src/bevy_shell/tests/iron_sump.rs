//! Declared local fixtures; all subsequent travel, flags, battles and Surf use input.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, load_tileset_art, mark_runtime_snapshot_dirty};
use super::interior_life;
use super::northstar_ice_lair::{event, path, talk, walk};
use super::r10_gallery::{press, settle};
use super::winona::{at, finish, load, travel};

const ROOM: &str = "MnIronSump";
const HAUL: &str = "EVENT_MN_IRON_HAUL_PRESSURE_OFF";
const VENT: &str = "EVENT_MN_IRON_VENT_PRESSURE_OFF";

fn output(name: &str) -> PathBuf {
    let p = PathBuf::from(std::env::var_os("CRYSTAL_IRON_SUMP_REVIEW_DIR").expect("fresh output"))
        .join(name);
    assert!(!p.exists(), "retain prior evidence");
    std::fs::create_dir_all(&p).expect("review folder");
    p
}

fn fixture(
    root: &AssetRoot,
    r: &CrystalRuntime,
    map: &str,
    point: (i16, i16),
    prior: Option<[bool; 2]>,
) -> VisibleShellController {
    let mut c = interior_life::fixture(root, r, map, point);
    let state = c.shell.shell.session_mut().state_mut();
    assert!(
        r.data().pokemon["VOYAGEAR"]
            .tmhm_learnset
            .iter()
            .any(|m| m == "SURF")
    );
    for species in ["VOYAGEAR", "CINDORE", "FORTIGARD", "MUDMENDER"] {
        let mut p = r
            .data()
            .create_pokemon(species, 55, Dv::from_non_hp(8, 9, 8, 9))
            .expect("original regional partner");
        if species == "VOYAGEAR" {
            p.moves[0] = LearnedMove {
                name: "SURF".into(),
                current_pp: r.data().moves["SURF"].pp,
                pp_ups: 0,
            };
            p.item = Some("SMOKE_BALL".into());
        }
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("declared badge fixture");
    }
    for (item, n) in [("HYPER_POTION", 10), ("MAX_REPEL", 10), ("OLD_ROD", 1)] {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], n)
                .expect("declared supplies")
        );
    }
    state.registered_key_item = Some("OLD_ROD".into());
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    if let Some(flags) = prior {
        for (flag, value) in [
            (HAUL, flags[0]),
            (VENT, flags[1]),
            ("EVENT_MN_LODE_ARC_SUMP_DEFEATED", true),
            ("EVENT_MN_LODE_ARC_DRAIN_DEFEATED", true),
        ] {
            state
                .flags
                .set_event_flag(flag, value)
                .expect("declared gate truth-table fixture");
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    c
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    path: &Path,
) -> VisibleShellController {
    c.save(path).expect("actual reached checkpoint");
    let mut next =
        VisibleShellController::load_save(root.clone(), r.clone(), path.to_path_buf(), None)
            .expect("real Continue");
    next.set_runtime_journal_enabled(false);
    next
}

fn cross(c: &mut VisibleShellController, r: &CrystalRuntime, from: &str, index: u16) {
    let port = r.data().maps[from]
        .events
        .warps
        .iter()
        .find(|w| w.index == index)
        .expect("active shaft");
    let target_name = r
        .data()
        .map_name_for_constant(&port.target_map_constant)
        .expect("paired room");
    let arrival = r.data().maps[&target_name]
        .events
        .warps
        .iter()
        .find(|w| i16::try_from(w.index).expect("native index") == port.target_warp_id)
        .expect("paired passive shaft");
    walk(c, r, (port.x as i16, port.y as i16 + 1));
    travel(
        c,
        GameButton::Up,
        &target_name,
        (arrival.x as i16, arrival.y as i16),
    );
    c.wait_frames(40).expect("passive shaft safety");
    at(c, &target_name, (arrival.x as i16, arrival.y as i16));
}

fn pair(c: &mut VisibleShellController, r: &CrystalRuntime, index: u16) {
    let port = r.data().maps[ROOM]
        .events
        .warps
        .iter()
        .find(|w| w.index == index)
        .expect("sump shaft");
    let target = r
        .data()
        .map_name_for_constant(&port.target_map_constant)
        .expect("target");
    let back = u16::try_from(port.target_warp_id).expect("paired index") - 1;
    cross(c, r, ROOM, index);
    cross(c, r, &target, back);
}

fn later_flags_unchanged(c: &VisibleShellController) {
    for flag in [
        "EVENT_MN_DRILL_FEED_ISOLATED",
        "EVENT_MN_DRILL_TEST_WITNESSED",
        "EVENT_MN_DRILL_STOPPED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
    ] {
        assert!(!event(c, flag), "sump cannot grant later progress: {flag}");
    }
}

#[test]
#[ignore = "requires current CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_IRON_SUMP_REVIEW_DIR; local fixtures only"]
fn iron_sump_controller_gate_requires_both_circuits_and_survives_continue() {
    let out = output("gate-states");
    let (root, r) = load();
    for flags in [[false, false], [true, false], [false, true], [true, true]] {
        let mut c = fixture(&root, &r, "MnIronHaulage", (28, 23), Some(flags));
        cross(&mut c, &r, "MnIronHaulage", 7);
        walk(&mut c, &r, (32, 8));
        if flags == [true, true] {
            assert!(path(&mut c, &r, (34, 5)).is_some());
            for target in [(32, 7), (32, 6), (32, 5)] {
                travel(&mut c, GameButton::Up, ROOM, target);
            }
            walk(&mut c, &r, (34, 5));
            c = continued(&mut c, &root, &r, &out.join("both-off-pocket.crystalsave"));
            walk(&mut c, &r, (32, 8));
            cross(&mut c, &r, ROOM, 7);
            cross(&mut c, &r, "MnIronPumpGallery", 1);
            walk(&mut c, &r, (32, 8));
        } else {
            assert!(path(&mut c, &r, (34, 5)).is_none());
            for _ in 0..3 {
                press(&mut c, GameButton::Up);
                at(&mut c, ROOM, (32, 8));
            }
            press(&mut c, GameButton::A);
            assert!(c.snapshot().expect("gate reading").ui.text_window_open);
            press(&mut c, GameButton::Down);
            at(&mut c, ROOM, (32, 8));
            finish(&mut c);
            settle(&mut c, "closed pump gate reading");
            c = continued(
                &mut c,
                &root,
                &r,
                &out.join(format!("closed-{}-{}.crystalsave", flags[0], flags[1])),
            );
            press(&mut c, GameButton::Up);
            at(&mut c, ROOM, (32, 8));
            assert!(path(&mut c, &r, (34, 5)).is_none());
        }
        assert_eq!([event(&c, HAUL), event(&c, VENT)], flags);
        later_flags_unchanged(&c);
        cross(&mut c, &r, ROOM, 5);
        cross(&mut c, &r, "MnIronHaulage", 7);
        walk(&mut c, &r, (32, 8));
        assert_eq!(path(&mut c, &r, (34, 5)).is_some(), flags == [true, true]);
    }
}

#[test]
#[ignore = "requires current CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_IRON_SUMP_REVIEW_DIR; local fixture only"]
fn iron_sump_controller_original_pressure_course_shafts_guards_and_entire_surf_basin() {
    let out = output("journey");
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnIronDrillApproach", (24, 5), None);
    c.save(out.join("initial-approach.crystalsave"))
        .expect("initial local fixture");
    super::northstar_iron_workings::enter_gallery(&mut c, &r, &root);
    assert!(event(&c, HAUL) && event(&c, VENT));
    later_flags_unchanged(&c);
    super::northstar_iron_workings::leave_gallery(&mut c, &r);
    cross(&mut c, &r, "MnIronDrillApproach", 3);
    cross(&mut c, &r, "MnIronHaulage", 3);
    for x in [4, 12] {
        walk(&mut c, &r, (x, 8));
        for target in [(x, 7), (x, 6), (x, 5)] {
            travel(&mut c, GameButton::Up, ROOM, target);
        }
        for target in [(x, 6), (x, 7), (x, 8)] {
            travel(&mut c, GameButton::Down, ROOM, target);
        }
    }
    walk(&mut c, &r, (13, 10));
    talk(&mut c, GameButton::Right);
    if !event(&c, "EVENT_MN_LODE_ARC_SUMP_DEFEATED") {
        assert!(super::rochester_earned::try_battle(
            &mut c,
            &r,
            "MN_LODE_ARC_SUMP",
            "EVENT_MN_LODE_ARC_SUMP_DEFEATED"
        ));
    }
    finish(&mut c);
    settle(&mut c, "sump guard victory");
    pair(&mut c, &r, 1);
    pair(&mut c, &r, 3);
    cross(&mut c, &r, ROOM, 3);
    cross(&mut c, &r, "MnIronVentilation", 7);
    for index in [5, 7, 9] {
        pair(&mut c, &r, index);
    }
    walk(&mut c, &r, (26, 17));
    talk(&mut c, GameButton::Left);
    if !event(&c, "EVENT_MN_LODE_ARC_DRAIN_DEFEATED") {
        assert!(super::rochester_earned::try_battle(
            &mut c,
            &r,
            "MN_LODE_ARC_DRAIN",
            "EVENT_MN_LODE_ARC_DRAIN_DEFEATED"
        ));
    }
    finish(&mut c);
    settle(&mut c, "drain guard victory");
    cross(&mut c, &r, ROOM, 9);
    cross(&mut c, &r, "MnIronVentilation", 1);
    walk(&mut c, &r, (12, 12));
    c.save(out.join("reached-bank.crystalsave"))
        .expect("actual reached bank");
    super::minneapolis_falls::mount(&mut c, GameButton::Left, ROOM, (11, 12));
    let module = &r.data().maps[ROOM];
    let water = (0_i16..24)
        .flat_map(|y| (0_i16..40).map(move |x| (x, y)))
        .filter(|&p| interior_life::collision(&r, module, p) == Some("WATER"))
        .collect::<BTreeSet<_>>();
    assert_eq!(water.len(), 38);
    for target in &water {
        let s = c.snapshot().expect("Surf origin");
        let start = (s.overworld.tile.x, s.overworld.tile.y);
        let mut parents = BTreeMap::from([(start, None)]);
        let mut queue = VecDeque::from([start]);
        while let Some(at) = queue.pop_front() {
            for (next, button) in super::authored_route_fixture::neighbors(at) {
                if water.contains(&next) && !parents.contains_key(&next) {
                    parents.insert(next, Some((at, button)));
                    queue.push_back(next);
                }
            }
        }
        let mut route = Vec::new();
        let mut cursor = *target;
        while let Some((prior, button)) = parents[&cursor] {
            route.push((cursor, button));
            cursor = prior;
        }
        for (next, button) in route.into_iter().rev() {
            for _ in 0..16 {
                if c.snapshot().expect("Surf step").ui.text_window_open {
                    finish(&mut c);
                }
                let p = c.snapshot().expect("Surf step").overworld.tile;
                if (p.x, p.y) == next {
                    break;
                }
                press(&mut c, button);
            }
            at(&mut c, ROOM, next);
            assert_eq!(
                c.snapshot().expect("real Surf").overworld.mode,
                MovementMode::Surf
            );
        }
    }
    // The solid southern rock lip cannot be crossed from the water.
    assert_eq!(interior_life::collision(&r, module, (10, 15)), Some("WALL"));
    // Reach this point using the same water-only native walking planner above
    // by revisiting its ordinary neighboring row through real inputs.
    super::pepin_discovery::paddle(&mut c, &r, (10, 14));
    for _ in 0..3 {
        press(&mut c, GameButton::Down);
        at(&mut c, ROOM, (10, 14));
    }
    c = continued(&mut c, &root, &r, &out.join("surf-lip.crystalsave"));
    assert_eq!(
        c.snapshot().expect("continued Surf").overworld.mode,
        MovementMode::Surf
    );
    super::pepin_discovery::paddle(&mut c, &r, (11, 12));
    travel(&mut c, GameButton::Right, ROOM, (12, 12));
    assert_eq!(
        c.snapshot().expect("dry dismount").overworld.mode,
        MovementMode::Normal
    );
    c.save(out.join("returned-bank.crystalsave"))
        .expect("real returned fishing bank");
    later_flags_unchanged(&c);
    assert!(
        event(&c, "EVENT_MN_LODE_ARC_SUMP_DEFEATED")
            && event(&c, "EVENT_MN_LODE_ARC_DRAIN_DEFEATED")
    );
    cross(&mut c, &r, ROOM, 1);
    cross(&mut c, &r, "MnIronHaulage", 1);
    at(&mut c, "MnIronDrillApproach", (24, 5));
    c.save(out.join("final-return.crystalsave"))
        .expect("final return");
}

#[test]
#[ignore = "requires current CRYSTAL_WINONA_TEST_PACK containing the sump native art"]
fn iron_sump_animation_preserves_native_cave_frames_pixels_and_clock_schedules() {
    let (_, r) = load();
    let root = r.materialize_runtime_files().expect("actual packed art");
    let mut images = Assets::<Image>::default();
    let native = load_tileset_art(
        &root,
        "cave",
        "nite",
        &r.data().tilesets["cave"].palette_map,
        &mut images,
    )
    .expect("native cave art");
    let sump = load_tileset_art(
        &root,
        "mn_iron_sump_rock",
        "nite",
        &r.data().tilesets["mn_iron_sump_rock"].palette_map,
        &mut images,
    )
    .expect("sump art");
    let keys = |art: &super::super::TilesetArt| {
        art.animated_tiles.keys().copied().collect::<BTreeSet<_>>()
    };
    assert_eq!(keys(&native), keys(&sump));
    assert!(sump.animated_tiles[&0x14].cave_water_composite);
    assert_eq!(sump.animated_tiles[&0x14].frames.len(), 32);
    for (key, a) in &native.animated_tiles {
        let b = &sump.animated_tiles[key];
        assert_eq!(
            (
                a.frame_ticks,
                a.phase_offset,
                a.cave_water_composite,
                a.advance_on_phase_offset
            ),
            (
                b.frame_ticks,
                b.phase_offset,
                b.cave_water_composite,
                b.advance_on_phase_offset
            )
        );
        assert_eq!(a.additional_schedule, b.additional_schedule);
        assert_eq!(a.frames.len(), b.frames.len());
        for (a, b) in a.frames.iter().zip(b.frames.iter()) {
            assert_eq!(
                images.get(a).expect("native frame").data,
                images.get(b).expect("sump frame").data
            );
        }
        assert!(
            b.frames
                .windows(2)
                .any(|f| images.get(&f[0]).expect("first frame").data
                    != images.get(&f[1]).expect("next frame").data)
        );
    }
}
