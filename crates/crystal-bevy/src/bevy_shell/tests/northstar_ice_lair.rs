//! Explicit local setup, then native recruitment, dungeon, battles and rescue.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::winona::{at, finish, load, press, travel};

const OFFICE: &str = "MnMinneapolisDispatchOffice";
const CARGO: &str = "MnIceCargoBasement";
const RELAY: &str = "MnIceRelayFloor";
const JOINED: &str = "EVENT_MN_REED_JOINED_ICE";
const FEED: &str = "EVENT_MN_ICE_CARGO_FEED_OFF";
const WON: &str = "EVENT_MN_REED_RELAY_DEFEATED";
const POWER: &str = "EVENT_MN_ICE_HOLDING_POWER_OFF";
const RESCUED: &str = "EVENT_MN_ICE_CAPTIVE_RESCUED";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

pub(super) fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("native result")
}
fn setup(
    losing: bool,
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
            map_name: OFFICE.into(),
            tile_x: 10,
            tile_y: 14,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("local dungeon fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let mut p = r
        .data()
        .create_pokemon(
            if losing { "CROOKIT" } else { "BOREALYNX" },
            if losing { 5 } else { 45 },
            Dv::default(),
        )
        .expect("native fixture partner");
    if losing {
        p.hp = 1;
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(p));
    state.sync_party_from_storage();
    state.money = 10000;
    state.badges.johto = [true, true, false, false, false, false, false, false];
    for (i, f) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state
            .flags
            .set_engine_flag(f, state.badges.johto[i])
            .expect("badge setup");
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("fixture presentation");
    // No state, position, party, flag or RNG writes beyond this boundary.
    (root, r, c)
}
pub(super) fn save(
    c: &mut VisibleShellController,
    root: &crystal_assets::AssetRoot,
    r: &CrystalRuntime,
    stage: &str,
) -> VisibleShellController {
    assert!(
        !c.has_pending_script_work(),
        "Continue checkpoints use completed scenes, as the production Save menu does"
    );
    let dir = std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR")
        .map(PathBuf::from)
        .expect("isolated checkpoint folder");
    std::fs::create_dir_all(&dir).expect("checkpoint folder");
    let path = dir.join(format!("ice-{stage}.crystalsave"));
    c.save(&path).expect("actual native checkpoint");
    VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("same-pack Continue")
}
pub(super) fn page(c: &mut VisibleShellController, label: &str) {
    for _ in 0..96 {
        c.wait_frames(128).expect("text presentation");
        let s = c.snapshot().expect("native scene");
        assert!(
            s.battle.is_none(),
            "unexpected fight while waiting for {label}"
        );
        if s.ui.text.as_ref().is_some_and(|t| t.label == label) {
            return;
        }
        press(c, GameButton::A);
    }
    let s = c.snapshot().expect("failed scene diagnostic");
    panic!(
        "scene never reached {label}: map={} tile={:?} text={:?} pending={}",
        s.overworld.map_name,
        s.overworld.tile,
        s.ui.text.as_ref().map(|t| &t.label),
        c.has_pending_script_work()
    );
}
pub(super) fn talk(c: &mut VisibleShellController, direction: GameButton) {
    press(c, direction);
    press(c, GameButton::A);
}
pub(super) fn choice(c: &mut VisibleShellController, yes: bool) {
    for _ in 0..96 {
        c.wait_frames(128).expect("choice presentation");
        let s = c.snapshot().expect("choice");
        if s.ui.pending_yes_no.is_some()
            && let Some(before) = c
                .shell
                .yes_no_cursor
                .as_ref()
                .map(|cursor| cursor.option_index)
        {
            // A displayed prompt can precede input ownership while text
            // pages remain. Prove ownership with an ordinary cursor move;
            // this also handles a final page containing only one line.
            press(
                c,
                if before == 0 {
                    GameButton::Down
                } else {
                    GameButton::Up
                },
            );
            if let Some(after) = c
                .shell
                .yes_no_cursor
                .as_ref()
                .map(|cursor| cursor.option_index)
                && before != after
            {
                let target = usize::from(!yes);
                if after != target {
                    press(
                        c,
                        if yes {
                            GameButton::Up
                        } else {
                            GameButton::Down
                        },
                    );
                }
                press(c, GameButton::A);
                return;
            }
        }
        press(c, GameButton::A);
    }
    let s = c.snapshot().expect("choice diagnostic");
    panic!(
        "choice did not appear: map={} tile={:?} text={:?} prompt={:?} cursor={:?} objects={:?}",
        s.overworld.map_name,
        s.overworld.tile,
        s.ui.text.as_ref().map(|t| &t.label),
        s.ui.pending_yes_no,
        c.shell.yes_no_cursor,
        s.visible_objects
            .iter()
            .map(|o| (&o.object_identifier, o.x, o.y))
            .collect::<Vec<_>>()
    );
}

// Plan over the authoritative live blocks and visible actors, including doors
// changed by native scripts. Planning never moves the player or grants results.
pub(super) fn path(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    to: (i16, i16),
) -> Option<Vec<GameButton>> {
    let s = c.snapshot().expect("live path");
    let live = &c.shell.shell.session().overworld().map;
    let name = &s.overworld.map_name;
    let module = &r.data().maps[name];
    let start = (s.overworld.tile.x, s.overworld.tile.y);
    let occupied: BTreeSet<_> = s
        .visible_objects
        .iter()
        .map(|o| {
            let tile = c
                .shell
                .shell
                .session()
                .overworld()
                .object_runtime_tile_by_id(
                    o.object_identifier.as_deref().expect("visible actor id"),
                )
                .expect("visible actor's live position");
            (tile.x, tile.y)
        })
        .collect();
    let mut previous = BTreeMap::new();
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([start]);
    while let Some((x, y)) = todo.pop_front() {
        if (x, y) == to {
            let mut at = to;
            let mut result = Vec::new();
            while at != start {
                let (old, button) = previous[&at];
                result.push(button);
                at = old;
            }
            result.reverse();
            return Some(result);
        }
        for ((xx, yy), button) in [
            ((x - 1, y), GameButton::Left),
            ((x + 1, y), GameButton::Right),
            ((x, y - 1), GameButton::Up),
            ((x, y + 1), GameButton::Down),
        ] {
            if xx < 0
                || yy < 0
                || xx >= live.width as i16 * 2
                || yy >= live.height as i16 * 2
                || occupied.contains(&(xx, yy))
                || seen.contains(&(xx, yy))
            {
                continue;
            }
            let block =
                live.metatile_ids[(yy as usize / 2) * live.width as usize + xx as usize / 2];
            let collision = &r.data().tilesets[&module.attributes.tileset_name].collision
                [&format!("{block:02x}")][yy as usize % 2 * 2 + xx as usize % 2];
            if !matches!(collision.as_str(), "FLOOR" | "WARP_CARPET_DOWN" | "LADDER")
                || (collision == "LADDER" && (xx, yy) != to)
            {
                continue;
            }
            seen.insert((xx, yy));
            previous.insert((xx, yy), ((x, y), button));
            todo.push_back((xx, yy));
        }
    }
    None
}
pub(super) fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: (i16, i16)) {
    let name = c.snapshot().expect("walking start").overworld.map_name;
    // Actors can enter the native viewport while walking. Replan after every
    // actual step so an initially offscreen worker is never treated as floor.
    for _ in 0..256 {
        let s = c.snapshot().expect("live walking input");
        if (s.overworld.tile.x, s.overworld.tile.y) == to {
            at(c, &name, to);
            return;
        }
        let button = *path(c, r, to)
            .unwrap_or_else(|| panic!("{name}: no live route to {to:?}"))
            .first()
            .expect("remaining walking step");
        let old = c.snapshot().expect("before walking input").overworld.tile;
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("walking input").overworld.tile != old {
                break;
            }
        }
        assert_ne!(
            c.snapshot().expect("completed step").overworld.tile,
            old,
            "{name}: native input {button:?} did not leave {old:?} toward {to:?}"
        );
    }
    panic!("{name}: walking route exceeded its bounded course to {to:?}");
}
fn join(c: &mut VisibleShellController, r: &CrystalRuntime, root: &crystal_assets::AssetRoot) {
    // A first directional input may only turn the player. Keep using native
    // input until stepping into the event, then let dialogue own the scene.
    for _ in 0..8 {
        press(c, GameButton::Up);
        if c.snapshot().expect("office entry").ui.text_window_open || c.has_pending_script_work() {
            break;
        }
    }
    page(c, "MnNICEEllisObjectsText");
    assert!(!event(c, JOINED));
    let before = c.snapshot().expect("witnessed recruitment");
    for id in [
        "MN_NICE_REED",
        "MN_ICE_ENFORCER",
        "MN_ICE_CAPTIVE_DISPATCHER",
    ] {
        assert!(
            before
                .visible_objects
                .iter()
                .any(|o| o.object_identifier.as_deref() == Some(id)),
            "live {id}"
        );
    }
    press(c, GameButton::Left);
    at(c, OFFICE, (11, 8));
    // Preserve the acted frame for headless review. Ordinary in-game Save is
    // unavailable during dialogue; this capture is not a Continue checkpoint.
    let frame_dir = PathBuf::from(
        std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR").expect("frame directory"),
    );
    std::fs::create_dir_all(&frame_dir).expect("frame directory");
    c.save(frame_dir.join("ice-recruitment-frame.crystalsave"))
        .expect("actual scene frame");
    page(c, "MnNICEReedAcceptText");
    assert!(!event(c, JOINED));
    finish(c);
    assert!(event(c, JOINED));
    *c = save(c, root, r, "joined-ice");
    assert!(!event(c, FEED) && !event(c, POWER) && !event(c, RESCUED) && !event(c, NOTE));
    assert!(
        !c.snapshot()
            .expect("Reed descends")
            .visible_objects
            .iter()
            .any(|o| o.object_identifier.as_deref() == Some("MN_NICE_REED"))
    );
}
fn down(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (5, 3));
    travel(c, GameButton::Up, CARGO, (4, 19));
}

#[test]
#[ignore = "requires the published event-driven ICE lair pack"]
fn ice_lair_acted_recruitment_two_floors_actual_battles_switches_rescue_witness_and_continue() {
    let (root, r, mut c) = setup(false);
    join(&mut c, &r, &root);
    walk(&mut c, &r, (11, 6));
    talk(&mut c, GameButton::Right);
    page(&mut c, "MnNICEPowerLockedText");
    finish(&mut c);
    down(&mut c, &r);
    assert!(
        path(&mut c, &r, (23, 4)).is_none(),
        "relay is physically sealed"
    );
    let long = path(&mut c, &r, (23, 18))
        .expect("real maze to breaker")
        .len();
    walk(&mut c, &r, (21, 16));
    talk(&mut c, GameButton::Down);
    page(&mut c, "MnNICECargoUmaText");
    finish(&mut c);
    walk(&mut c, &r, (23, 18));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, FEED));
    c = save(&mut c, &root, &r, "cargo-declined");
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, FEED) && !event(&c, POWER));
    assert!(
        path(&mut c, &r, (4, 19))
            .expect("earned return shortcut")
            .len()
            < long
    );
    c = save(&mut c, &root, &r, "cargo-power-off");
    walk(&mut c, &r, (4, 18));
    travel(&mut c, GameButton::Right, OFFICE, (5, 4));
    down(&mut c, &r);
    assert!(path(&mut c, &r, (23, 4)).is_some());
    walk(&mut c, &r, (23, 5));
    travel(&mut c, GameButton::Up, RELAY, (4, 15));
    walk(&mut c, &r, (20, 3));
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnNICEHoldingDeniedText");
    finish(&mut c);
    assert!(!event(&c, POWER));
    walk(&mut c, &r, (21, 5));
    talk(&mut c, GameButton::Up);
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &r,
        "MN_REED_RELAY",
        WON
    ));
    finish(&mut c);
    assert!(event(&c, WON) && !event(&c, POWER) && !event(&c, RESCUED));
    c = save(&mut c, &root, &r, "relay-rival-won");
    walk(&mut c, &r, (20, 3));
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, POWER));
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, POWER) && !event(&c, RESCUED));
    c = save(&mut c, &root, &r, "holding-power-off");
    walk(&mut c, &r, (4, 14));
    travel(&mut c, GameButton::Right, CARGO, (22, 5));
    walk(&mut c, &r, (4, 18));
    travel(&mut c, GameButton::Right, OFFICE, (5, 4));
    walk(&mut c, &r, (11, 6));
    talk(&mut c, GameButton::Right);
    page(&mut c, "MnMPCellLockedText");
    finish(&mut c);
    assert!(!event(&c, RESCUED));
    walk(&mut c, &r, (9, 9));
    talk(&mut c, GameButton::Left);
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &r,
        "MN_ICE_ENFORCER",
        "EVENT_MN_ICE_ENFORCER_DEFEATED"
    ));
    finish(&mut c);
    walk(&mut c, &r, (11, 6));
    talk(&mut c, GameButton::Right);
    choice(&mut c, true);
    finish(&mut c);
    assert!(event(&c, RESCUED) && !event(&c, NOTE));
    c = save(&mut c, &root, &r, "ellis-released");
    walk(&mut c, &r, (10, 11));
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert!(!event(&c, NOTE), "a found copy cannot award the quest");
    walk(&mut c, &r, (10, 14));
    travel(&mut c, GameButton::Down, "MnMinneapolis", (27, 34));
    super::authored_route_fixture::indoor_walk(&mut c, &r, (27, 14));
    travel(
        &mut c,
        GameButton::Up,
        "MnMinneapolisDispatcherHouse",
        (5, 6),
    );
    let ellis = c
        .snapshot()
        .expect("Ellis home")
        .visible_objects
        .into_iter()
        .find(|o| o.object_identifier.as_deref() == Some("MN_MP_RESCUED_DISPATCHER"))
        .expect("rescued witness");
    walk(&mut c, &r, (ellis.x as i16, ellis.y as i16 + 1));
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnNICEEllisQuestText");
    assert!(!event(&c, NOTE));
    finish(&mut c);
    assert!(event(&c, NOTE));
    c = save(&mut c, &root, &r, "witness-quest-acknowledged");
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnNICEEllisAfterText");
    finish(&mut c);
    assert!(event(&c, NOTE) && event(&c, RESCUED));
}

#[test]
#[ignore = "requires the event ICE pack with both entry lanes"]
fn ice_lair_right_entry_also_witnesses_reeds_recruitment() {
    let (root, r, mut c) = setup(false);
    walk(&mut c, &r, (11, 14));
    join(&mut c, &r, &root);
    assert!(event(&c, JOINED) && !event(&c, FEED) && !event(&c, POWER));
}
