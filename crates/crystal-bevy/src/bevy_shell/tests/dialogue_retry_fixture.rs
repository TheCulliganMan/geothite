//! A native loss must leave the automatic courier encounter available.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::native_city_ports::city_port;
use super::super::rochester_earned::try_battle;
use super::super::winona::{at, continue_in_place, finish, press, travel};
use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::fixture;

const CITY: &str = "MnStPaul";
const CLINIC: &str = "MnStPaulPokecenter1F";
const DEPOT: &str = "MnStPaulUnionDepot";
const COURIER: &str = "EVENT_MN_COURIER_DEFEATED";
const MANIFEST: &str = "EVENT_MN_DEEP_BORE_MANIFEST_READ";

fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("declared courier event")
}

fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, city_port(r, CITY, DEPOT).0);
    for _ in 0..12 {
        if c.snapshot().expect("native entry").overworld.map_name == DEPOT {
            break;
        }
        press(c, GameButton::Up);
    }
    c.wait_frames(512).expect("authored visible approach");
    at(c, DEPOT, (19, 16));
    let s = c.snapshot().expect("courier retry stages participants");
    assert_eq!(
        s.ui.text.expect("first speaker").label,
        "MnNSTPMinaInterruptText"
    );
    assert!(s.battle.is_none());
    for (id, expected) in [
        ("MN_NSTP_MINA", (18, 16)),
        ("MN_ST_PAUL_DEPOT_COURIER", (20, 16)),
    ] {
        let tile = s.visible_object_runtime_tiles[id];
        assert_eq!((tile.x, tile.y), expected);
    }
    press(c, GameButton::Down);
    at(c, DEPOT, (19, 16));
    assert!(!event(c, COURIER) && !event(c, MANIFEST));
}

#[test]
#[ignore = "declared losing party; requires revised pack and isolated native save output"]
fn overhaul_courier_defeat_continue_and_reentry_retry_the_automatic_scene() {
    let (root, r, mut c) = fixture(CLINIC, (3, 5), &["EVENT_MN_DISPATCH_NOTE"]);
    // This is explicit initial fixture setup. Subsequent loss, recovery,
    // Continue, movement and reentry all use the production controller.
    let state = c.shell.shell.session_mut().state_mut();
    state.badges.johto[..3].fill(true);
    state
        .flags
        .set_engine_flag("ENGINE_PLAINBADGE", true)
        .expect("declared Current badge");
    state.storage.party.pokemon[0] = Some(
        r.data()
            .create_pokemon("POUCHID", 5, Dv::default())
            .expect("native low-level Minnesota partner"),
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);

    // Enter the clinic through its real door so blackout recovery has the
    // actual native spawn. Starting at a fixture tile cannot stand in for it.
    indoor_walk(&mut c, &r, (3, 6));
    travel(
        &mut c,
        GameButton::Down,
        CITY,
        city_port(&r, CITY, CLINIC).0,
    );
    travel(&mut c, GameButton::Up, CLINIC, (3, 5));
    indoor_walk(&mut c, &r, (3, 3));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_ST_PAUL_POKECENTER_1F")
    );
    indoor_walk(&mut c, &r, (3, 6));
    travel(
        &mut c,
        GameButton::Down,
        CITY,
        city_port(&r, CITY, CLINIC).0,
    );
    enter(&mut c, &r);
    let money = c.shell.shell.session().state().money;
    assert!(!try_battle(&mut c, &r, "MN_LODE_COURIER", COURIER));
    at(&mut c, CLINIC, (3, 5));
    assert!(!event(&c, COURIER) && !event(&c, MANIFEST));
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, true, false, false, false, false, false]
    );
    assert!(c.shell.shell.session().state().money < money);
    continue_in_place(&mut c, &root, &r, "overhaul-courier-defeat");
    at(&mut c, CLINIC, (3, 5));
    assert!(!event(&c, COURIER) && !event(&c, MANIFEST));

    indoor_walk(&mut c, &r, (3, 6));
    travel(
        &mut c,
        GameButton::Down,
        CITY,
        city_port(&r, CITY, CLINIC).0,
    );
    enter(&mut c, &r);
}
