//! Initial position/party are fixtures; seams, care, PC and Continue use input.
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use serde_json::Value;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, talk, walk};
use super::r10_gallery::{continue_save, press};

const ELY: &str = "MnEly";
const PASS: &str = "MnElyShorePass";
const SHORE: &str = "MnR11C00R02";
const CLINIC: &str = "MnSplitRockPokecenter1F";
fn at(c: &mut VisibleShellController) -> (String, (i16, i16)) {
    let s = c.snapshot().expect("native pass position");
    (
        s.overworld.map_name,
        (s.overworld.tile.x, s.overworld.tile.y),
    )
}
fn plan() -> Value {
    let path = std::env::var_os("CRYSTAL_ELY_PASS_INPUT_PLAN")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../planning/minnesota/revisions/ely-backroute-v01/source-v08/input-plan.json",
            )
        });
    serde_json::from_str(&std::fs::read_to_string(path).expect("explicit reviewed paths"))
        .expect("input plan")
}

fn course(c: &mut VisibleShellController, key: &str, reverse: bool) {
    let mut points = plan()[key]
        .as_array()
        .expect("course")
        .iter()
        .map(|p| {
            (
                p[0].as_i64().expect("x") as i16,
                p[1].as_i64().expect("y") as i16,
            )
        })
        .collect::<Vec<_>>();
    if reverse {
        points.reverse();
    }
    let (name, start) = at(c);
    assert_eq!(start, points[0]);
    for pair in points.windows(2) {
        let button = match (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1) {
            (1, 0) => GameButton::Right,
            (-1, 0) => GameButton::Left,
            (0, 1) => GameButton::Down,
            (0, -1) => GameButton::Up,
            other => panic!("nonadjacent plan {other:?}"),
        };
        for _ in 0..4 {
            if at(c).1 == pair[1] {
                break;
            }
            press(c, button);
        }
        assert_eq!(at(c), (name.clone(), pair[1]), "actual {key} input");
        assert!(c.snapshot().expect("walk").battle.is_none());
    }
}
fn cross(c: &mut VisibleShellController, button: GameButton, name: &str, point: (i16, i16)) {
    for _ in 0..6 {
        if at(c).0 == name {
            break;
        }
        press(c, button);
    }
    assert_eq!(at(c), (name.into(), point));
}
fn save_view(c: &mut VisibleShellController, name: &str) {
    if let Some(folder) = std::env::var_os("CRYSTAL_ELY_PASS_SAVE_DIR") {
        let folder = PathBuf::from(folder);
        std::fs::create_dir_all(&folder).expect("capture saves");
        c.save(folder.join(format!("{name}.crystalsave")))
            .expect("actual reached save");
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing Ely Shore Pass"]
fn ely_shore_pass_controller_round_trip_heals_pc_and_continue() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, ELY, (27, 14));
    // Explicit injured high-level partner/repel setup. Nothing is rewritten after this.
    let mut partner = r
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("fixture partner");
    partner.hp = 3;
    partner.status = Some("BURN".into());
    partner.moves[0].current_pp = 0;
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    // Three-member wild scaling can exceed a solo lead by one level. Lower-level
    // initial partners keep this care/PC fixture's real Repel above that anchor.
    for species in ["VOYAGEAR", "DOZEBEAR"] {
        let companion = r
            .data()
            .create_pokemon(species, 40, Dv::default())
            .expect("explicit lower-level regional fixture partner");
        assert!(state.storage.party.add_pokemon(companion));
    }
    state.sync_party_from_storage();
    assert!(
        state
            .bag
            .add_item(&r.data().items["MAX_REPEL"], 1)
            .expect("fixture repel")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    super::minneapolis_falls::grand_marais_harbor::use_max_repel(&mut c, &r);
    let badges = c.shell.shell.session().state().badges.clone();
    let money = c.shell.shell.session().state().money;
    course(&mut c, "ely_apron_to_east", false);
    save_view(&mut c, "ely-east-snow");
    cross(&mut c, GameButton::Right, PASS, (0, 14));
    course(&mut c, "pass_west_to_east", false);
    save_view(&mut c, "pass-coast-snowline");
    cross(&mut c, GameButton::Right, SHORE, (0, 30));
    course(&mut c, "shore_backroad_to_clinic", false);
    save_view(&mut c, "split-rock-center");
    cross(&mut c, GameButton::Up, CLINIC, (3, 5));
    walk(&mut c, &r, (3, 3));
    let pages = talk(&mut c, (3, 2));
    assert!(!pages.is_empty());
    let state = c.shell.shell.session().state();
    let p = state.storage.party.pokemon[0]
        .as_ref()
        .expect("healed partner");
    assert_eq!(p.hp, p.max_hp);
    assert!(p.status.is_none());
    assert!(
        p.moves
            .iter()
            .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    );
    assert_eq!(
        state.last_spawn_map_constant.as_deref(),
        Some("MN_SPLIT_ROCK_POKECENTER_1F")
    );
    c = continue_save(root.clone(), r.clone(), &mut c);
    // The real PC interaction must open the native menu.
    walk(&mut c, &r, (8, 1));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    for _ in 0..10 {
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(c.shell.pc_hub_cursor.is_some(), "production clinic PC menu");
    for _ in 0..20 {
        if c.shell.pc_hub_cursor.is_none() && !c.snapshot().expect("PC").ui.text_window_open {
            break;
        }
        press(&mut c, GameButton::B);
    }
    walk(&mut c, &r, (3, 7));
    assert_eq!(at(&mut c), (SHORE.into(), (33, 34)));
    course(&mut c, "shore_backroad_to_clinic", true);
    cross(&mut c, GameButton::Left, PASS, (13, 10));
    course(&mut c, "pass_west_to_east", true);
    cross(&mut c, GameButton::Left, ELY, (39, 14));
    course(&mut c, "ely_apron_to_east", true);
    c = continue_save(root, r, &mut c);
    assert_eq!(at(&mut c), (ELY.into(), (27, 14)));
    assert_eq!(c.shell.shell.session().state().badges, badges);
    assert_eq!(c.shell.shell.session().state().money, money);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing Ely Shore Pass"]
fn ely_shore_pass_controller_other_seam_lanes_and_original_ely_north_port() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, ELY, (39, 15));
    cross(&mut c, GameButton::Right, PASS, (0, 15));
    cross(&mut c, GameButton::Left, ELY, (39, 15));
    let mut c = fixture(&root, &r, PASS, (13, 11));
    cross(&mut c, GameButton::Right, SHORE, (0, 31));
    cross(&mut c, GameButton::Left, PASS, (13, 11));
    let mut c = fixture(&root, &r, ELY, (24, 0));
    cross(&mut c, GameButton::Up, "MnR13C00R02", (34, 31));
    cross(&mut c, GameButton::Down, ELY, (24, 0));
}

#[cfg(feature = "location-tester")]
#[test]
#[ignore = "requires reached Ely pass saves, a GPU and headless image capture"]
fn ely_shore_pass_headless_captures_actual_reached_scenes() {
    use super::super::{BevyShellConfig, BevyShellStart, run_bevy_shell};
    assert_eq!(
        std::env::var("CRYSTAL_RENDER_TEST_HEADLESS").as_deref(),
        Ok("1")
    );
    assert_eq!(
        std::env::var("CRYSTAL_RENDER_TEST_CAPTURE_TARGET").as_deref(),
        Ok("image")
    );
    let (root, runtime, _) = load();
    let saves =
        PathBuf::from(std::env::var_os("CRYSTAL_ELY_PASS_SAVE_DIR").expect("reached saves"));
    let output = PathBuf::from(
        std::env::var_os("CRYSTAL_ELY_PASS_CAPTURE_DIR").expect("fresh capture directory"),
    );
    std::fs::create_dir_all(&output).expect("capture directory");
    for name in ["ely-east-snow", "pass-coast-snowline", "split-rock-center"] {
        let screenshot = output.join(format!("{name}.png"));
        assert!(!screenshot.exists(), "preserve previous capture");
        run_bevy_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::LoadSave {
                save_path: saves.join(format!("{name}.crystalsave")),
            },
            BevyShellConfig {
                voxel_view_enabled: Some(false),
                render_test_screenshot: Some(screenshot.clone()),
                render_test_window_size: Some((640, 576)),
                ..Default::default()
            },
        )
        .expect("actual production headless capture");
        assert!(screenshot.is_file());
    }
}
