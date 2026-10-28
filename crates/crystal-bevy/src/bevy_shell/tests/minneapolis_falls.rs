//! Explicit local fixtures; all later Surf/HM/gate/seam actions use native input.
//! Initial party/badges/location are fixtures, not earned regional progression.
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::{ScriptFlagCommand, apply_script_flag_mutation};
use crystal_core::world::map::Direction;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::north_shore::settle_water;

type Point = (i16, i16);
const CITY: &str = "MnMinneapolis";
const NORTH: &str = "MnR05C00R02";
const SOUTH: &str = "MnR03C00R00";
const WATERFALL_DENIAL: &str = ".HugeWaterfallText@Script_CantDoWaterfall";

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = std::env::var_os("CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                        "../../content-packs/minnesota-minneapolis-compact-falls-v02.crystalpack",
                    )
                })
                .canonicalize()
                .expect("completed external Minneapolis Falls pack");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(&path).expect("strict native pack"),
            )
            .expect("native Falls runtime");
            (root, runtime)
        })
        .clone()
}

fn at(c: &mut VisibleShellController) -> (String, Point) {
    let s = c.snapshot().expect("authoritative local water position");
    (
        s.overworld.map_name,
        (s.overworld.tile.x, s.overworld.tile.y),
    )
}

fn assert_at(c: &mut VisibleShellController, map: &str, point: Point, context: &str) {
    assert_eq!(at(c), (map.to_owned(), point), "{context}");
    assert!(c.shell.last_error.is_none(), "{context}");
}

fn collision(r: &CrystalRuntime, c: &mut VisibleShellController, point: Point) -> String {
    let map_name = at(c).0;
    let tileset = &r.data().maps[&map_name].attributes.tileset_name;
    let block = c
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(point.0 / 2, point.1 / 2)
        .expect("actual current block including callback changes");
    r.data().tilesets[tileset].collision[&format!("{block:02x}")]
        [(point.1 % 2 * 2 + point.0 % 2) as usize]
        .clone()
}

fn fixture(
    map: &str,
    point: Point,
    plain: bool,
    rising: bool,
    waterfall_move: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    fixture_with_items(map, point, plain, rising, waterfall_move, &[])
}

fn fixture_with_items(
    map: &str,
    point: Point,
    plain: bool,
    rising: bool,
    waterfall_move: bool,
    items: &[(&str, u16)],
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    fixture_with_items_on_runtime((root, r), map, point, plain, rising, waterfall_move, items)
}

// Separate verified data lets child fixtures retain their own immutable pack
// cache without changing process-wide environment variables or old fixtures.
fn fixture_with_items_on_runtime(
    assets: (AssetRoot, CrystalRuntime),
    map: &str,
    point: Point,
    plain: bool,
    rising: bool,
    waterfall_move: bool,
    items: &[(&str, u16)],
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = assets;
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("original HOME"),
            map_name: map.into(),
            tile_x: point.0,
            tile_y: point.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit initial legal Floor placement");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("explicit trainer name");
    let mut pokemon = r
        .data()
        .create_pokemon("MEW", 80, Dv::default())
        .expect("legal native Surf/HM-compatible fixture partner");
    let moves = if waterfall_move {
        vec!["SURF", "WATERFALL"]
    } else {
        vec!["SURF"]
    };
    pokemon.moves = moves
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: r.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let state = c.shell.shell.session_mut().state_mut();
    for &(item, quantity) in items {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], quantity)
                .expect("explicit initial native inventory")
        );
    }
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    for (index, (flag, enabled)) in [
        ("ENGINE_ZEPHYRBADGE", false),
        ("ENGINE_HIVEBADGE", false),
        ("ENGINE_PLAINBADGE", plain),
        ("ENGINE_FOGBADGE", true),
        ("ENGINE_STORMBADGE", false),
        ("ENGINE_MINERALBADGE", false),
        ("ENGINE_GLACIERBADGE", false),
        ("ENGINE_RISINGBADGE", rising),
    ]
    .into_iter()
    .enumerate()
    {
        apply_script_flag_mutation(
            state,
            ScriptFlagCommand {
                command: if enabled { "setflag" } else { "clearflag" }.into(),
                flag_id: flag.into(),
                source_script: "MinneapolisFallsExplicitInitialFixture".into(),
                command_index: index,
            },
        )
        .expect("consistent badge bits and production engine flags");
    }
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Setup ends here. No subsequent state/position/flags/mode/party/RNG writes.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_at(&mut c, map, point, "initial explicit Floor fixture");
    assert_eq!(collision(&r, &mut c, point), "FLOOR");
    assert_eq!(
        c.snapshot().expect("initial mode").overworld.mode,
        MovementMode::Normal
    );
    (root, r, c)
}

// Reuse the production water test's RUN handling; collect authored notices here
// because automatic dismissal alone cannot prove a checkpoint explanation.
fn settle(c: &mut VisibleShellController) -> Vec<String> {
    let mut text = Vec::new();
    for _ in 0..256 {
        let s = c.snapshot().expect("native water/dialogue continuation");
        if s.battle.is_some() {
            settle_water(c);
            continue;
        }
        if let Some(page) = s.ui.text
            && let Some(body) = page.asm_text
        {
            text.push(body);
        }
        if let Some(notice) = &c.shell.field_notice {
            text.push(notice.clone());
        }
        assert!(
            s.ui.menu.is_none(),
            "unexpected non-field menu during water continuation"
        );
        if c.shell.visible_waterfall_animation.is_some() && c.shell.field_notice.is_none() {
            c.wait_frames(16).expect("native Waterfall animation clock");
        } else if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && s.ui.pending_text_wait.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            return text;
        } else {
            c.wait_frames(120).expect("fully print authored page");
            c.press(GameButton::A).expect("acknowledge native text");
            c.wait_frames(16).expect("native text continuation");
        }
    }
    panic!("native water activity failed to settle: {:?}", at(c));
}

pub(super) fn step(
    c: &mut VisibleShellController,
    button: GameButton,
    map: &str,
    point: Point,
    context: &str,
) -> Vec<String> {
    let target = (map.to_owned(), point);
    let mut text = Vec::new();
    for _ in 0..8 {
        if at(c) == target {
            return text;
        }
        let before = at(c);
        c.press(button)
            .expect("actual directional controller input");
        c.wait_frames(16).expect("native step presentation");
        text.extend(settle(c));
        let after = at(c);
        assert!(
            after == before || after == target,
            "{context}: {before:?} -> {after:?}; expected {target:?}"
        );
    }
    assert_at(c, map, point, context);
    text
}

pub(super) fn mount(c: &mut VisibleShellController, button: GameButton, map: &str, target: Point) {
    assert_eq!(
        c.snapshot().expect("dry bank").overworld.mode,
        MovementMode::Normal
    );
    c.press(button).expect("face actual adjacent Water");
    c.press(GameButton::A).expect("native Surf prompt");
    for _ in 0..128 {
        if c.snapshot().expect("Surf mount").overworld.mode == MovementMode::Surf {
            c.wait_frames(60).expect("Surf mount presentation");
            settle(c);
            assert_at(c, map, target, "actual Surf launch");
            return;
        }
        c.press(GameButton::A).expect("native Surf confirmation");
        c.wait_frames(16).expect("native Surf use text");
    }
    panic!("native Surf did not launch: {:?}", at(c));
}

fn face_up(c: &mut VisibleShellController, point: Point) {
    if c.snapshot().expect("cascade facing").overworld.facing != Direction::Up {
        c.press(GameButton::Up).expect("turn to native cascade");
        c.wait_frames(16).expect("turn presentation");
        settle(c);
    }
    assert_at(c, CITY, point, "face cascade without injecting facing");
    assert_eq!(
        c.snapshot().expect("uphill facing").overworld.facing,
        Direction::Up
    );
}

fn descend(c: &mut VisibleShellController, x: i16) {
    assert_at(c, CITY, (x, 5), "native cascade upper approach");
    c.press(GameButton::Down)
        .expect("enter actual downward current");
    for _ in 0..256 {
        c.wait_frames(8).expect("native forced current clock");
        settle(c);
        if at(c) == (CITY.into(), (x, 10)) {
            return;
        }
        let (map, (px, y)) = at(c);
        assert_eq!(map, CITY);
        assert_eq!(px, x);
        assert!(
            (5..=10).contains(&y),
            "native current left the authored cascade"
        );
        // A direction change may consume the first input. Once on the face,
        // only the native clock advances the downward current.
        if y == 5 {
            c.press(GameButton::Down)
                .expect("step into cascade after turn");
        }
    }
    panic!("native current did not land: {:?}", at(c));
}

#[test]
#[ignore = "requires the published compact Falls pack"]
fn minneapolis_falls_controller_surfs_and_denies_unqualified_uphill_use() {
    for (rising, known_move) in [(true, false), (false, true)] {
        let (_, _, mut c) = fixture(CITY, (31, 10), true, rising, known_move);
        mount(&mut c, GameButton::Right, CITY, (32, 10));
        face_up(&mut c, (32, 10));
        let before = c.snapshot().expect("native denied ascent preimage");
        let expected = before
            .presentation
            .asm_text_for_label(WATERFALL_DENIAL)
            .expect("native scoped waterfall refusal text")
            .to_owned();
        c.press(GameButton::A).expect("contextual denied Waterfall");
        assert_eq!(c.shell.field_notice.as_deref(), Some(expected.as_str()));
        assert_at(&mut c, CITY, (32, 10), "denied uphill location");
        c.press(GameButton::Down)
            .expect("notice owns movement input");
        assert_at(&mut c, CITY, (32, 10), "denial notice cannot move player");
        assert_eq!(
            before.party,
            c.snapshot().expect("denied party unchanged").party
        );
        assert!(!c.shell.flygon_field_actions.contains("waterfall"));
        settle(&mut c);
    }
}

#[test]
#[ignore = "requires the published compact Falls pack"]
fn minneapolis_falls_controller_uses_waterfall_and_preserves_native_downward_current() {
    let (_, r, mut c) = fixture(CITY, (31, 10), true, true, true);
    mount(&mut c, GameButton::Right, CITY, (32, 10));
    face_up(&mut c, (32, 10));
    let before_party = c.snapshot().expect("native climb preimage").party;
    c.press(GameButton::A).expect("actual Waterfall prompt");
    for _ in 0..256 {
        if at(&mut c) == (CITY.into(), (32, 5))
            && c.shell.visible_waterfall_animation.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            break;
        }
        if c.shell.visible_waterfall_animation.is_some() && c.shell.field_notice.is_none() {
            c.wait_frames(16).expect("native climb clock");
        } else {
            c.press(GameButton::A).expect("confirm native Waterfall");
            c.wait_frames(16).expect("native climb source continuation");
        }
    }
    assert_at(&mut c, CITY, (32, 5), "actual native Waterfall climb");
    assert_eq!(collision(&r, &mut c, (32, 5)), "WATER");
    assert!(c.shell.flygon_field_actions.contains("waterfall"));
    assert_eq!(
        before_party,
        c.snapshot().expect("field HM consumes no battle PP").party
    );
    descend(&mut c, 32);
    step(
        &mut c,
        GameButton::Left,
        CITY,
        (31, 10),
        "ordinary post-cascade dismount",
    );
    assert_eq!(
        c.snapshot().expect("dry return").overworld.mode,
        MovementMode::Normal
    );
}

#[test]
#[ignore = "requires the published compact Falls pack and native buoy gate"]
fn minneapolis_falls_controller_north_water_seams_both_ways_with_checkpoint_success() {
    // Fog+Plain only: no Rising badge or Waterfall move is needed downstream.
    let (root, r, mut c) = fixture(NORTH, (19, 11), true, false, false);
    mount(&mut c, GameButton::Down, NORTH, (19, 12));
    for y in 13..=15 {
        step(
            &mut c,
            GameButton::Down,
            NORTH,
            (19, y),
            "actual upstream stem",
        );
    }
    step(
        &mut c,
        GameButton::Down,
        CITY,
        (37, 0),
        "R05 to actual city Water seam",
    );
    for x in (32..=37).rev() {
        if at(&mut c).1.0 != x {
            step(
                &mut c,
                GameButton::Left,
                CITY,
                (x, 0),
                "upper pool lane selection",
            );
        }
        assert_eq!(collision(&r, &mut c, (x, 0)), "WATER");
        step(
            &mut c,
            GameButton::Up,
            NORTH,
            (x - 18, 15),
            "all six northbound Water pairs",
        );
        assert_eq!(collision(&r, &mut c, (x - 18, 15)), "WATER");
        step(
            &mut c,
            GameButton::Down,
            CITY,
            (x, 0),
            "all six southbound Water pairs",
        );
    }
    for y in 1..=2 {
        let text = step(
            &mut c,
            GameButton::Down,
            CITY,
            (32, y),
            "qualified six-lane water checkpoint",
        );
        assert!(!text.iter().any(|line| line.contains("River gate closed.")));
    }
    for x in 32..=37 {
        assert_eq!(
            collision(&r, &mut c, (x, 2)),
            "WATER",
            "qualified gate must open all six lanes"
        );
    }
    // The silent native tile callback must reopen the qualified gate after a
    // real menu return and Continue, including when the player stands on it.
    assert_at(&mut c, CITY, (32, 2), "qualified gate before reset checks");
    c.press(GameButton::Start)
        .expect("production Start on Surf");
    c.wait_frames(16).expect("production Start presentation");
    assert!(
        c.snapshot()
            .expect("production Start menu")
            .ui
            .menu
            .is_some()
    );
    c.press(GameButton::B)
        .expect("return from production Start");
    c.wait_frames(16).expect("native map callback after menu");
    assert!(c.snapshot().expect("Start closed").ui.menu.is_none());
    settle(&mut c);
    assert_at(&mut c, CITY, (32, 2), "menu must retain gate footing");
    let save = std::env::temp_dir().join(format!(
        "minneapolis-open-water-gate-{}-{}.crystalsave",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("fixture clock")
            .as_nanos()
    ));
    let before = c.snapshot().expect("native open-gate save preimage");
    let badges = c.shell.shell.session().state().badges.johto;
    c.save(&save)
        .expect("production save on qualified open Water");
    c = VisibleShellController::load_save(root, r.clone(), save.clone(), None)
        .expect("production Continue must restore callback-open Water before footing validation");
    let after = c.snapshot().expect("continued water gate");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(badges, c.shell.shell.session().state().badges.johto);
    assert_at(
        &mut c,
        CITY,
        (32, 2),
        "Continue retains actual Surf position",
    );
    for x in 32..=37 {
        assert_eq!(
            collision(&r, &mut c, (x, 2)),
            "WATER",
            "native callback restores all six open lanes"
        );
    }
    std::fs::remove_file(save).expect("remove only this fixture save");
    for y in 3..=5 {
        step(
            &mut c,
            GameButton::Down,
            CITY,
            (32, y),
            "qualified return through restored native gate",
        );
    }
    descend(&mut c, 32);
    assert!(
        !c.shell.flygon_field_actions.contains("waterfall"),
        "downstream uses authentic current, not an invented HM requirement"
    );
    step(
        &mut c,
        GameButton::Left,
        CITY,
        (31, 10),
        "qualified downstream dismount",
    );
}

#[test]
#[ignore = "requires the published compact Falls pack and native buoy gate"]
fn minneapolis_falls_controller_downstream_missing_plain_badge_cannot_bypass_checkpoint() {
    let (_, r, mut c) = fixture(NORTH, (19, 11), false, false, false);
    mount(&mut c, GameButton::Down, NORTH, (19, 12));
    for y in 13..=15 {
        step(
            &mut c,
            GameButton::Down,
            NORTH,
            (19, y),
            "actual R05 negative-fixture approach",
        );
    }
    step(
        &mut c,
        GameButton::Down,
        CITY,
        (37, 0),
        "real river entry before checkpoint",
    );
    for x in (34..37).rev() {
        step(
            &mut c,
            GameButton::Left,
            CITY,
            (x, 0),
            "native upper pool approach",
        );
    }
    let printed = step(
        &mut c,
        GameButton::Down,
        CITY,
        (34, 1),
        "actual authored water refusal",
    )
    .join("\n");
    assert!(printed.contains("River gate closed."), "{printed}");
    assert!(
        printed.contains("CURRENT") && printed.contains("CAPITOL"),
        "{printed}"
    );
    for x in 32..=37 {
        assert_eq!(
            collision(&r, &mut c, (x, 2)),
            "BUOY",
            "unqualified native closed gate"
        );
    }
    for _ in 0..4 {
        c.press(GameButton::Down)
            .expect("Surf against native visible buoy gate");
        c.wait_frames(16).expect("blocked Surf presentation");
        settle(&mut c);
        assert_at(
            &mut c,
            CITY,
            (34, 1),
            "Fog alone cannot reach downstream cascade",
        );
    }
    assert_eq!(
        c.snapshot().expect("denied gate Surf mode").overworld.mode,
        MovementMode::Surf
    );
    assert!(!c.shell.flygon_field_actions.contains("waterfall"));
    assert!(!c.shell.shell.session().state().badges.johto[2]);
    step(
        &mut c,
        GameButton::Up,
        CITY,
        (34, 0),
        "denial permits retreat",
    );
    step(
        &mut c,
        GameButton::Up,
        NORTH,
        (16, 15),
        "denied traveler can return upstream",
    );
}

#[test]
#[ignore = "requires the published compact Falls pack"]
fn minneapolis_falls_controller_south_water_seams_both_directions() {
    let (_, r, mut c) = fixture(CITY, (38, 60), false, false, false);
    mount(&mut c, GameButton::Left, CITY, (37, 60));
    for y in 61..=63 {
        step(
            &mut c,
            GameButton::Down,
            CITY,
            (37, y),
            "actual southern river stem",
        );
    }
    for x in (32..=37).rev() {
        if at(&mut c).1.0 != x {
            step(
                &mut c,
                GameButton::Left,
                CITY,
                (x, 63),
                "southern lane selection",
            );
        }
        assert_eq!(collision(&r, &mut c, (x, 63)), "WATER");
        step(
            &mut c,
            GameButton::Down,
            SOUTH,
            (x - 20, 0),
            "all six city-to-R03 Water pairs",
        );
        assert_eq!(collision(&r, &mut c, (x - 20, 0)), "WATER");
        step(
            &mut c,
            GameButton::Up,
            CITY,
            (x, 63),
            "all six R03-to-city Water pairs",
        );
    }
    for x in 33..=37 {
        step(
            &mut c,
            GameButton::Right,
            CITY,
            (x, 63),
            "return along actual river mouth",
        );
    }
    for y in (60..63).rev() {
        step(
            &mut c,
            GameButton::Up,
            CITY,
            (37, y),
            "ordinary southern bank return",
        );
    }
    step(
        &mut c,
        GameButton::Right,
        CITY,
        (38, 60),
        "southern native dismount",
    );
    assert_eq!(
        c.snapshot().expect("southern dry return").overworld.mode,
        MovementMode::Normal
    );
}

#[path = "grand_marais_harbor.rs"]
pub(super) mod grand_marais_harbor;

#[path = "mississippi_journey.rs"]
mod mississippi_journey;

#[path = "winter_region.rs"]
mod winter_region;

#[path = "river_continuity_bend.rs"]
mod river_continuity_bend;

#[path = "river_long_banks.rs"]
mod river_long_banks;

#[path = "park_pond_route.rs"]
mod park_pond_route;

#[path = "north_shore_warm.rs"]
mod north_shore_warm;

#[path = "grand_marais_superior.rs"]
mod grand_marais_superior;

#[path = "compact_superior_coast.rs"]
mod compact_superior_coast;

#[path = "superior_inland_geography.rs"]
mod superior_inland_geography;

#[path = "river_route_fun.rs"]
mod river_route_fun;
