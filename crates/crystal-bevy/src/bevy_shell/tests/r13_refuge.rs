//! Local legal party/position/RNG setup followed only by native controller input.
//! These fixtures do not earn regional survey, badge or League access.
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, indoor_walk, neighbors};
use super::r10_gallery::{
    completed_visible_page, continue_save, flags, map, press, remember_dialogue, settle, tile,
};
use super::r12_shelter::use_pp_item;

const ROOM: &str = "MnR13Cave1F";
const OUTSIDE: &str = "MnR13C00R01";
const EVENT: &str = "EVENT_MN_R13_SHELTER_ETHER";
#[derive(Clone, Copy, PartialEq, Eq)]
enum PartySetup {
    Empty,
    InjuredTeam,
    LowPpPartner,
}
fn source() -> Value {
    let path=std::env::var_os("CRYSTAL_R13_REFUGE_SOURCE").map(PathBuf::from).unwrap_or_else(||PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/renders/caves/r13-candidate/v02/MnR13Cave1F.candidate.json"));
    serde_json::from_str(
        &std::fs::read_to_string(path).expect("same frozen R13 publication source"),
    )
    .expect("refuge source")
}
fn point(v: &Value) -> Point {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}
fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R13_REFUGE_CONTROLLER_PACK")
                    .expect("actual revised R13 native pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path)
                    .expect("strict published refuge pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone()
}
fn fixture(
    name: &str,
    at: Point,
    setup: PartySetup,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("retained HOME"),
            map_name: name.into(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local refuge fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    if setup != PartySetup::Empty {
        for (species, level, status) in [("MEW", 30, "POISON"), ("TOTODILE", 20, "PARALYSIS")] {
            if setup == PartySetup::LowPpPartner && species != "MEW" {
                continue;
            }
            let mut pokemon = runtime
                .data()
                .create_pokemon(species, level, Dv::default())
                .expect("source-legal native learned moves");
            for learned in &mut pokemon.moves {
                learned.current_pp = 1;
            }
            if setup == PartySetup::InjuredTeam {
                pokemon.hp = pokemon.max_hp - 30;
                pokemon.status = Some(status.into());
            }
            let state = c.shell.shell.session_mut().state_mut();
            assert!(state.storage.party.add_pokemon(pokemon));
            state.sync_party_from_storage();
        }
    }
    let state = c.shell.shell.session_mut().state_mut();
    state.money = 1000;
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Explicit initial setup ends here. Healing, bag/PP use and progress are input-only.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(tile(&mut c), at);
    assert!(!collected(&c));
    let s = source();
    let m = &runtime.data().maps[ROOM];
    assert_eq!([m.attributes.width, m.attributes.height], [10, 8]);
    let expected = s["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(|r| r.as_str().expect("row").split_whitespace())
        .map(|t| u16::from_str_radix(t, 16).expect("native block"))
        .collect::<Vec<_>>();
    assert_eq!(
        m.blocks, expected,
        "actual pack must match exact frozen refuge geometry"
    );
    assert!(
        m.script_runtime_commands
            .iter()
            .any(|cmd| cmd.source_script == "MN_R13_HARPER"
                && cmd.command_index == 4
                && cmd.command == "special"
                && cmd.args == ["HealParty"])
    );
    (root, runtime, c)
}
fn collected(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(EVENT)
        .expect("original item event")
}
fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["MAX_ETHER"])
}
fn kind(r: &CrystalRuntime, p: Point) -> &str {
    let m = &r.data().maps[ROOM];
    let b = m.blocks[p.1 as usize / 2 * 10 + p.0 as usize / 2];
    &r.data().tilesets["cave"].collision[&format!("{b:02x}")]
        [p.1 as usize % 2 * 2 + p.0 as usize % 2]
}
fn direction(from: Point, to: Point) -> GameButton {
    neighbors(from)
        .into_iter()
        .find_map(|(p, b)| (p == to).then_some(b))
        .expect("adjacent native step")
}
fn blocked(c: &mut VisibleShellController, b: GameButton) {
    let at = tile(c);
    for _ in 0..4 {
        press(c, b);
        assert_eq!(tile(c), at);
    }
}
fn step(c: &mut VisibleShellController, b: GameButton, to: Point) {
    for _ in 0..8 {
        if tile(c) == to {
            return;
        }
        press(c, b);
    }
    assert_eq!(tile(c), to);
}
fn outside(r: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR13",
            columns: 1,
            rows: 3,
            tileset: if r.data().maps["MnR13C00R00"].attributes.tileset_name
                == "mn_winter_north_shore"
            {
                "mn_winter_north_shore"
            } else {
                "johto"
            },
            start: (14, 54),
            doors: BTreeMap::from([((14, 53), ROOM)]),
        },
    )
}
fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    outside(r).walk(c, (14, 53));
    assert_eq!(map(c), ROOM);
    assert_eq!(tile(c), (7, 15));
    indoor_walk(c, r, (7, 14));
    assert_eq!(tile(c), (7, 14));
}
fn leave(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (7, 14));
    indoor_walk(c, r, (7, 15));
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (14, 6));
    c.wait_frames(60).expect("true passive Floor return");
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (14, 6));
    let grid = outside(r);
    grid.walk(c, (14, 55));
    grid.walk(c, (14, 54));
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (14, 6));
}

#[test]
#[ignore = "requires actual revised R13 refuge pack and matching frozen source"]
fn r13_refuge_controller_true_passive_return_and_both_useful_stairs_bidirectional() {
    let (_, r, mut c) = fixture(OUTSIDE, (14, 6), PartySetup::Empty);
    enter(&mut c, &r);
    let s = source();
    let stairs = s["review_contract"]["controller_probes"]["stairs"]
        .as_array()
        .expect("two useful stairs");
    assert_eq!(stairs.len(), 2);
    for stair in stairs {
        let path = std::iter::once(point(&stair["before"]))
            .chain(
                stair["lane"]
                    .as_array()
                    .expect("ordinary lane")
                    .iter()
                    .map(point),
            )
            .chain(std::iter::once(point(&stair["after"])))
            .collect::<Vec<_>>();
        for &p in &path {
            assert_eq!(kind(&r, p), "FLOOR");
            indoor_walk(&mut c, &r, p);
            assert_eq!(tile(&mut c), p);
        }
        let from = point(&stair["blocked_from"]);
        let target = point(&stair["blocked_target"]);
        assert_eq!(kind(&r, target), "WALL");
        indoor_walk(&mut c, &r, from);
        blocked(&mut c, direction(from, target));
        indoor_walk(&mut c, &r, *path.last().expect("far landing"));
        for &p in path.iter().rev().skip(1) {
            indoor_walk(&mut c, &r, p);
            assert_eq!(tile(&mut c), p);
        }
    }
    for p in s["review_contract"]["ordinary_waypoints_core"]
        .as_array()
        .expect("ordinary refuge approaches")
    {
        indoor_walk(&mut c, &r, point(p));
        assert_eq!(map(&mut c), ROOM);
    }
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert!(!collected(&c));
}
fn harper(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (13, 6));
    press(c, GameButton::Up);
    let before = c.snapshot().expect("party before original Harper").party;
    let mut expected = before.clone();
    for slot in &mut expected.slots {
        let p = &mut slot.pokemon;
        p.hp = p.max_hp;
        p.status = None;
        for m in &mut p.moves {
            assert_eq!(m.pp_ups, 0);
            m.current_pp = r.data().moves[&m.name].pp;
        }
    }
    press(c, GameButton::A);
    let at = tile(c);
    press(c, GameButton::Down);
    assert_eq!(tile(c), at, "original Harper text owns movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    let mut healed = false;
    for _ in 0..48 {
        c.wait_frames(240).expect("print Harper's original page");
        remember_dialogue(c, &mut diagnostics);
        let snap = c.snapshot().expect("actual native healer state");
        if snap.party == expected && pages.len() >= 2 {
            healed = true;
            break;
        }
        assert_eq!(
            snap.party, before,
            "healing must wait for the original printed pages and waitbutton; {diagnostics:#?}"
        );
        if let Some((index, page)) = completed_visible_page(c) {
            assert_eq!(
                snap.ui.text.as_ref().map(|t| t.label.as_str()),
                Some("MN_R13_HARPERText"),
                "{diagnostics:#?}"
            );
            pages.insert(index, page);
        }
        press(c, GameButton::A);
    }
    assert_eq!(
        pages.get(&0).map(String::as_str),
        Some("Rest in the rock.\nI will tend to"),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1).map(String::as_str),
        Some("your whole team.\nEly is not far."),
        "{diagnostics:#?}"
    );
    assert!(
        healed,
        "native HealParty did not restore HP/status/all PP after acknowledgement: {diagnostics:#?}"
    );
    settle(c, "after original Harper HealParty/closetext/end");
    assert_eq!(c.snapshot().expect("entire healed party").party, expected);
}
#[test]
#[ignore = "requires actual revised R13 refuge pack retaining original HealParty service"]
fn r13_refuge_controller_original_harper_pages_heal_two_legal_partners_and_continue() {
    let (root, r, mut c) = fixture(ROOM, (7, 14), PartySetup::InjuredTeam);
    let before = c.snapshot().expect("injured/status/lowPP team").party;
    assert_eq!(before.slots.len(), 2);
    for slot in before.slots {
        assert!(slot.pokemon.hp < slot.pokemon.max_hp);
        assert!(slot.pokemon.status.is_some());
        assert!(slot.pokemon.moves.iter().all(|m| m.current_pp == 1));
    }
    let original_flags = flags(&c);
    let bag = c.snapshot().expect("initial carried inventory").bag;
    harper(&mut c, &r);
    assert_eq!(flags(&c), original_flags);
    assert_eq!(c.snapshot().expect("service is not an item grant").bag, bag);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    c = continue_save(root, r.clone(), &mut c);
    for slot in c.snapshot().expect("continued actual healing").party.slots {
        assert_eq!(slot.pokemon.hp, slot.pokemon.max_hp);
        assert_eq!(slot.pokemon.status, None);
        for m in slot.pokemon.moves {
            assert_eq!(m.current_pp, r.data().moves[&m.name].pp);
        }
    }
    leave(&mut c, &r);
    assert!(!collected(&c));
}
fn collect(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (14, 11));
    press(c, GameButton::Right);
    assert_eq!(tile(c), (14, 11));
    press(c, GameButton::A);
    assert!(collected(c));
    assert_eq!(quantity(c, r), 1);
    let at = tile(c);
    press(c, GameButton::Down);
    assert_eq!(tile(c), at, "native item notice owns input");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        c.wait_frames(240)
            .expect("print original cache pocket page");
        remember_dialogue(c, &mut diagnostics);
        if let Some((i, page)) = completed_visible_page(c) {
            assert_eq!(
                c.shell
                    .visible_field_item_notice
                    .as_ref()
                    .expect("native item notice")
                    .phase,
                super::super::VisibleFieldItemPhase::PocketText
            );
            pages.insert(i, page);
        }
        if c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let item = r.data().items["MAX_ETHER"].name.replace('_', " ");
    assert_eq!(
        pages.get(&0),
        Some(&format!("CHRIS put the\n{item} in")),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1),
        Some(&format!("{item} in\nthe ITEM POCKET.")),
        "{diagnostics:#?}"
    );
    settle(c, "after original R13 cache presentation");
}
#[test]
#[ignore = "requires actual revised R13 refuge pack retaining original Max Ether program"]
fn r13_refuge_controller_original_max_ether_restores_native_learned_pp_once_and_continues() {
    let (root, r, mut c) = fixture(ROOM, (7, 14), PartySetup::LowPpPartner);
    assert_eq!(quantity(&c, &r), 0);
    collect(&mut c, &r);
    use_pp_item(&mut c, &r, "MAX_ETHER", Some(0));
    assert_eq!(quantity(&c, &r), 0);
    leave(&mut c, &r);
    enter(&mut c, &r);
    indoor_walk(&mut c, &r, (14, 11));
    step(&mut c, GameButton::Right, (15, 11));
    press(&mut c, GameButton::A);
    settle(&mut c, "already collected R13 cache");
    assert_eq!(quantity(&c, &r), 0);
    assert!(collected(&c));
    c = continue_save(root, r.clone(), &mut c);
    assert!(collected(&c));
    assert_eq!(quantity(&c, &r), 0);
    assert_eq!(c.shell.shell.session().state().money, 1000);
}
