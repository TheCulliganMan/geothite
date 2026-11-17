//! Declared field party/time/items; actual walking, Surf and capture use input.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::time::GameDate;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::winona::{load, press};

fn fixture(
    map: &str,
    tile: [i16; 2],
    surf: bool,
    rod: bool,
) -> (
    crystal_assets::AssetRoot,
    CrystalRuntime,
    VisibleShellController,
) {
    fixture_at_hour(map, tile, surf, rod, 12)
}

fn fixture_at_hour(
    map: &str,
    tile: [i16; 2],
    surf: bool,
    rod: bool,
    hour: u8,
) -> (
    crystal_assets::AssetRoot,
    CrystalRuntime,
    VisibleShellController,
) {
    fixture_with_partner(
        map,
        tile,
        surf,
        rod,
        hour,
        if surf { "VOYAGEAR" } else { "BOREALYNX" },
        70,
    )
}

fn fixture_with_partner(
    map: &str,
    tile: [i16; 2],
    surf: bool,
    rod: bool,
    hour: u8,
    species: &str,
    level: u8,
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
            map_name: map.into(),
            tile_x: tile[0],
            tile_y: tile[1],
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("local field fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    let mut p = r
        .data()
        .create_pokemon(species, level, Dv::default())
        .expect("legal Minnesota partner");
    p.item = Some("SMOKE_BALL".into());
    if surf {
        assert!(p.species.tmhm_learnset.iter().any(|m| m == "SURF"));
        p.moves[0] = LearnedMove {
            name: "SURF".into(),
            current_pp: r.data().moves["SURF"].pp,
            pp_ups: 0,
        };
        state.badges.johto[3] = true;
        state
            .flags
            .set_engine_flag("ENGINE_FOGBADGE", true)
            .expect("prior Surf badge");
    }
    assert!(state.storage.party.add_pokemon(p));
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&r.data().items["MASTER_BALL"], 1)
            .expect("explicit capture-fixture ball")
    );
    if rod {
        assert!(
            state
                .bag
                .add_item(&r.data().items["GOOD_ROD"], 1)
                .expect("declared prior rod")
        );
        state.registered_key_item = Some("GOOD_ROD".into());
    }
    state.time.update_server_datetime(
        GameDate {
            year: 2000,
            month: 1,
            day: 3,
        },
        hour,
        0,
        0,
    );
    state
        .flags
        .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", false)
        .expect("normal field encounters");
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("field presentation");
    // No state, party, time, position, inventory or RNG writes after setup.
    (root, r, c)
}

fn encounter_and_capture(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    target: &str,
    a: [i16; 2],
    b: [i16; 2],
) {
    encounter_and_capture_one_of(c, r, &[target], a, b);
}

fn encounter_and_capture_one_of(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    targets: &[&str],
    a: [i16; 2],
    b: [i16; 2],
) -> String {
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    // A 4% slot needs more walking than the common-resident smoke checks.
    for step in 0..4096 {
        let s = c.snapshot().expect("native habitat step");
        if let Some(battle) = s.battle {
            *seen
                .entry(battle.enemy_pokemon.species.id.clone())
                .or_default() += 1;
            if targets.contains(&battle.enemy_pokemon.species.id.as_str()) {
                assert!(
                    super::rochester_earned::try_recruit_with_ball(c, r, "MASTER_BALL"),
                    "actual target capture"
                );
                return battle.enemy_pokemon.species.id;
            }
            super::rochester::flee(c);
            continue;
        }
        let here = [s.overworld.tile.x, s.overworld.tile.y];
        assert!(
            here == a || here == b,
            "habitat loop left its native cells: {here:?}"
        );
        let next = if here == a { b } else { a };
        let direction = match (next[0] - here[0], next[1] - here[1]) {
            (1, 0) => GameButton::Right,
            (-1, 0) => GameButton::Left,
            (0, 1) => GameButton::Down,
            (0, -1) => GameButton::Up,
            _ => panic!("adjacent loop"),
        };
        press(c, direction);
        c.wait_frames(17 + step % 11).expect("ordinary idle frames");
    }
    panic!(
        "did not naturally encounter {targets:?} through production movement; observed {seen:?}"
    );
}

fn checkpoint(c: &mut VisibleShellController, name: &str) {
    if let Some(dir) = std::env::var_os("CRYSTAL_MAP_RETAIN_SAVES_DIR") {
        let path = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&path).expect("review directory");
        let path = path.join(format!("habitat-{name}.crystalsave"));
        assert!(!path.exists());
        c.save(path).expect("production checkpoint");
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with authored habitats"]
fn habitats_cindore_real_grass_capture_consumes_ball_and_survives_continue() {
    let (root, r, mut c) = fixture("MnR09C00R00", [40, 24], false, false);
    checkpoint(&mut c, "cindore-start");
    encounter_and_capture(&mut c, &r, "CINDORE", [40, 24], [41, 24]);
    verify_capture(&c, &r, "CINDORE");
    c = super::r10_gallery::continue_save(root, r.clone(), &mut c);
    verify_capture(&c, &r, "CINDORE");
    checkpoint(&mut c, "cindore-caught");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with authored habitats"]
fn habitats_pinglet_actual_surf_mount_water_capture_and_continue() {
    let (root, r, mut c) = fixture("MnR03C00R00", [11, 6], true, false);
    super::north_shore::mount_surf(&mut c, GameButton::Right, (12, 6));
    checkpoint(&mut c, "pinglet-surf-start");
    encounter_and_capture(&mut c, &r, "PINGLET", [12, 6], [12, 7]);
    verify_capture(&c, &r, "PINGLET");
    c = super::r10_gallery::continue_save(root, r.clone(), &mut c);
    verify_capture(&c, &r, "PINGLET");
    checkpoint(&mut c, "pinglet-caught");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the southern cedar habitat"]
fn habitats_cedar_southern_approach_real_capture_and_continue() {
    let (root, r, mut c) = fixture("MnMaraisCedarApproach", [32, 20], false, false);
    for _ in 0..4 {
        let tile = c
            .snapshot()
            .expect("southern grass approach")
            .overworld
            .tile;
        if (tile.x, tile.y) == (32, 19) {
            break;
        }
        press(&mut c, GameButton::Up);
    }
    super::winona::at(&mut c, "MnMaraisCedarApproach", (32, 19));
    checkpoint(&mut c, "cedar-start");
    encounter_and_capture(&mut c, &r, "KNOTHUCK", [32, 19], [33, 19]);
    verify_capture(&c, &r, "KNOTHUCK");
    c = super::r10_gallery::continue_save(root, r.clone(), &mut c);
    verify_capture(&c, &r, "KNOTHUCK");
    checkpoint(&mut c, "cedar-caught");
}

fn verify_capture(c: &VisibleShellController, r: &CrystalRuntime, species: &str) {
    let state = c.shell.shell.session().state();
    assert!(state.pokedex.has_caught(species));
    assert_eq!(state.storage.party.filled_slots(), 2);
    assert!(
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .any(|p| p.species.id == species)
    );
    assert_eq!(state.bag.quantity(&r.data().items["MASTER_BALL"]), 0);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with local rod habitats"]
fn habitats_registered_rod_controller_clock_preserves_pose_and_captures_local_fish() {
    let (root, r, mut c) = fixture("MnR03C00R00", [11, 6], false, true);
    press(&mut c, GameButton::Right);
    checkpoint(&mut c, "pinglet-rod-start");
    for _ in 0..64 {
        c.press(GameButton::Select)
            .expect("registered Good Rod transaction");
        assert!(
            c.shell.visible_fishing_animation.is_some(),
            "actual casting sequence"
        );
        assert!(
            c.snapshot()
                .expect("retained shore during cast")
                .battle
                .is_none(),
            "the pending core encounter must not leak through the fishing owner"
        );
        let after_cast = c.shell.shell.session().clone();
        for _ in 0..113 {
            if c.shell
                .visible_fishing_animation
                .as_ref()
                .is_some_and(|a| a.phase == super::super::VisibleFishingPhase::AwaitText)
            {
                break;
            }
            c.press(GameButton::Up)
                .expect("casting owns movement and advances its production clock");
        }
        assert_eq!(
            c.shell
                .visible_fishing_animation
                .as_ref()
                .expect("fishing notice")
                .phase,
            super::super::VisibleFishingPhase::AwaitText
        );
        assert_eq!(
            c.shell.shell.session(),
            &after_cast,
            "casting presentation freezes authoritative state and DIV"
        );
        for button in [GameButton::Up, GameButton::Left] {
            c.press(button).expect("notice owns directional input");
            let notice = c.snapshot().expect("retained fishing notice");
            assert!(notice.battle.is_none());
            assert_eq!(
                notice
                    .ui
                    .text
                    .as_ref()
                    .expect("visible fishing notice")
                    .label,
                "visible-shell:field-notice"
            );
            assert_eq!(c.shell.shell.session(), &after_cast);
            assert_eq!(
                c.shell
                    .visible_fishing_animation
                    .as_ref()
                    .expect("unacknowledged notice")
                    .phase,
                super::super::VisibleFishingPhase::AwaitText
            );
        }
        for _ in 0..128 {
            let s = c.snapshot().expect("fishing notice/battle");
            if s.battle.is_some() {
                break;
            }
            if c.shell.visible_fishing_animation.is_none()
                && c.shell.field_notice.is_none()
                && !c.has_pending_script_work()
            {
                break;
            }
            press(&mut c, GameButton::A);
        }
        if let Some(battle) = c.snapshot().expect("cast result").battle {
            if battle.enemy_pokemon.species.id == "PINGLET" {
                assert!(super::rochester_earned::try_recruit_with_ball(
                    &mut c,
                    &r,
                    "MASTER_BALL"
                ));
                verify_capture(&c, &r, "PINGLET");
                c = super::r10_gallery::continue_save(root, r.clone(), &mut c);
                verify_capture(&c, &r, "PINGLET");
                checkpoint(&mut c, "pinglet-rod-caught");
                return;
            }
            super::rochester::flee(&mut c);
        }
    }
    panic!("local Good Rod never selected PINGLET through native RNG");
}

#[path = "minnesota_habitat_fishing.rs"]
mod fishing;

#[path = "minnesota_habitat_headbutt.rs"]
mod headbutt;

#[path = "minnesota_landmark_habitats.rs"]
mod landmarks;

#[path = "minnesota_water_caves.rs"]
mod water_caves;

#[path = "minnesota_ecology.rs"]
mod ecology;

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the spawn location revision"]
fn spawn_locations_native_walking_capture_and_continue() {
    for (map, from, to, hour, species) in [
        ("MnR01Cave1F", [5, 3], [6, 3], 12, "TESSLARV"),
        ("MnR03Cave1F", [5, 3], [6, 3], 12, "CHORIP"),
        ("MnR03C00R01", [20, 2], [21, 2], 22, "CHORIP"),
        ("MnR06C00R01", [52, 6], [53, 6], 12, "PITCHWICK"),
        ("MnR08C01R00", [2, 28], [3, 28], 12, "BRISTUB"),
        ("MnR19C00R01", [22, 2], [23, 2], 12, "TAPACITOR"),
    ] {
        let (root, runtime, mut controller) = fixture_at_hour(map, from, false, false, hour);
        encounter_and_capture(&mut controller, &runtime, species, from, to);
        verify_capture(&controller, &runtime, species);
        controller = super::r10_gallery::continue_save(root, runtime.clone(), &mut controller);
        verify_capture(&controller, &runtime, species);
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the regional ecology revision"]
fn region_ecology_walks_catches_and_continues_throughout_minnesota() {
    // Northstar follows the host clock, so the cases with a day/night turnover
    // accept authored targets across all periods.
    type Course<'a> = (&'a str, [i16; 2], [i16; 2], &'a [&'a str]);
    let courses: [Course<'_>; 14] = [
        ("MnR01Cave1F", [5, 3], [6, 3], &["HUSHBAT", "TACTUNNEL"]),
        ("MnR02C00R00", [6, 8], [7, 8], &["NIBBIN"]),
        ("MnR03C00R01", [20, 2], [21, 2], &["TESSLARV"]),
        ("MnR04C00R00", [10, 26], [11, 26], &["BURRBUD"]),
        ("MnR06C00R01", [52, 6], [53, 6], &["TRAILPUP", "BITTERRY"]),
        ("MnR08C01R00", [2, 28], [3, 28], &["LATCHQUILL"]),
        ("MnR09Cave1F", [5, 3], [6, 3], &["CARTICK", "DEEPULSE"]),
        (
            "MnR11C00R01",
            [16, 2],
            [17, 2],
            &["CUBLUFF", "LOOMLET", "TALURSA"],
        ),
        ("MnR12C00R00", [2, 28], [3, 28], &["RAKEHORN"]),
        ("MnR13C00R01", [32, 18], [33, 18], &["FORTIGARD"]),
        (
            "MnR16C00R00",
            [2, 32],
            [3, 32],
            &["STEMBOLT", "REKINK", "PENDULOP"],
        ),
        (
            "MnR17C00R01",
            [19, 30],
            [19, 31],
            &["SHEDSTAG", "LATCHQUILL", "CROOKIT", "BRISTUB", "DRIFTPEG"],
        ),
        ("MnR18C00R00", [6, 30], [7, 30], &["HISSIP"]),
        (
            "MnR19C00R01",
            [22, 2],
            [23, 2],
            &["REKINK", "PENDULOP", "SETTKIT"],
        ),
    ];
    for (map, from, to, targets) in courses {
        let (root, runtime, mut controller) = fixture(map, from, false, false);
        let species = encounter_and_capture_one_of(&mut controller, &runtime, targets, from, to);
        verify_capture(&controller, &runtime, &species);
        controller = super::r10_gallery::continue_save(root, runtime.clone(), &mut controller);
        verify_capture(&controller, &runtime, &species);
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the regional ecology revision"]
fn region_ecology_surfs_catches_and_continues_in_the_river_valley() {
    let (root, runtime, mut controller) = fixture("MnR03C00R00", [11, 6], true, false);
    super::north_shore::mount_surf(&mut controller, GameButton::Right, (12, 6));
    encounter_and_capture(&mut controller, &runtime, "CHORIP", [12, 6], [12, 7]);
    verify_capture(&controller, &runtime, "CHORIP");
    controller = super::r10_gallery::continue_save(root, runtime.clone(), &mut controller);
    verify_capture(&controller, &runtime, "CHORIP");
}
