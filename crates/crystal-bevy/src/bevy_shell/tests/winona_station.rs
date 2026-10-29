//! Initial route/party placement is setup; all visits, conversations and gifts use input.
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, talk, walk};
use super::r10_gallery::{continue_save, press};

const ROOM: &str = "MnWinonaStation";
const VISITED: &str = "EVENT_MN_STATION_WINONA_VISITED";
const READ: &str = "EVENT_MN_WINONA_DEPOT_LOG_READ";
const HEARD: &str = "EVENT_MN_WINONA_DEPOT_MASON_HEARD";
const GIVEN: &str = "EVENT_MN_WINONA_DEPOT_DRY_KIT_GIVEN";

fn flag(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("native event")
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["ICE_HEAL"])
}

fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (19, 13));
    let snapshot = c.snapshot().expect("native depot door");
    assert_eq!(snapshot.overworld.map_name, ROOM);
    assert_eq!(
        (snapshot.overworld.tile.x, snapshot.overworld.tile.y),
        (11, 15)
    );
    assert!(flag(c, VISITED));
}

fn read_log(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (5, 11));
    assert!(talk(c, (5, 10)).contains("WINTER REGISTER"));
    assert!(flag(c, READ));
}

fn hear_mason(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk(c, r, (6, 13));
    assert!(talk(c, (5, 13)).contains("steady hand"));
    assert!(flag(c, HEARD));
}

fn porter(c: &mut VisibleShellController, r: &CrystalRuntime) -> String {
    walk(c, r, (9, 13));
    talk(c, (9, 12))
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing the Winona station"]
fn winona_station_controller_road_entry_residents_carpets_and_sealed_platform() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, "MnR01C00R00", (62, 24));
    // The retained checkpoint prerequisite is explicit setup, not earned story.
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag("EVENT_MN_STARTER_RECEIVED", true)
        .expect("checkpoint fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(!flag(&c, VISITED));
    for _ in 0..8 {
        if c.snapshot().expect("road crossing").overworld.map_name == "MnWinona" {
            break;
        }
        press(&mut c, GameButton::Right);
    }
    assert_eq!(
        c.snapshot()
            .expect("real road connection")
            .overworld
            .map_name,
        "MnWinona"
    );
    for _ in 0..4 {
        if c.snapshot().expect("inward checkpoint").overworld.tile.x >= 3 {
            break;
        }
        press(&mut c, GameButton::Right);
    }
    walk(&mut c, &r, (22, 13));
    assert!(talk(&mut c, (22, 12)).contains("WINONA DEPOT"));
    assert!(
        !flag(&c, VISITED),
        "an exterior sign cannot unlock a station"
    );
    let money = c.shell.shell.session().state().money;
    enter(&mut c, &r);
    walk(&mut c, &r, (11, 10));
    for _ in 0..4 {
        press(&mut c, GameButton::Up);
    }
    let at = c.snapshot().expect("conductor blocks boarding").overworld;
    assert_eq!((at.tile.x, at.tile.y), (11, 10));
    assert!(talk(&mut c, (11, 9)).contains("not open yet"));
    press(&mut c, GameButton::Start);
    assert!(
        c.shell.start_menu_cursor.is_some(),
        "conductor releases input"
    );
    press(&mut c, GameButton::B);
    hear_mason(&mut c, &r);
    assert!(porter(&mut c, &r).contains("ice log"));
    walk(&mut c, &r, (13, 15));
    assert!(talk(&mut c, (14, 15)).contains("headwind"));
    read_log(&mut c, &r);
    assert!(!flag(&c, GIVEN));
    for exit in [(10, 17), (11, 17)] {
        walk(&mut c, &r, exit);
        let s = c.snapshot().expect("same native town return").overworld;
        assert_eq!(s.map_name, "MnWinona");
        assert_eq!((s.tile.x, s.tile.y), (19, 14));
        press(&mut c, GameButton::Down);
        press(&mut c, GameButton::Up);
        assert_eq!(
            c.snapshot()
                .expect("passive return does not reenter")
                .overworld
                .map_name,
            "MnWinona"
        );
        enter(&mut c, &r);
    }
    c = continue_save(root, r.clone(), &mut c);
    assert!(flag(&c, VISITED) && flag(&c, READ) && flag(&c, HEARD));
    assert!(!flag(&c, GIVEN));
    assert!(
        !flag(&c, "EVENT_MN_COURIER_DEFEATED"),
        "a visit never activates the service"
    );
    assert_eq!(c.shell.shell.session().state().money, money);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing the Winona station"]
fn winona_station_controller_both_story_orders_and_once_only_continue() {
    let (root, r, _) = load();
    for read_first in [true, false] {
        let mut c = fixture(&root, &r, "MnWinona", (19, 14));
        enter(&mut c, &r);
        let before = quantity(&c, &r);
        porter(&mut c, &r);
        assert_eq!(quantity(&c, &r), before);
        if read_first {
            read_log(&mut c, &r);
        } else {
            hear_mason(&mut c, &r);
        }
        porter(&mut c, &r);
        assert!(!flag(&c, GIVEN), "one clue alone cannot give the kit");
        assert_eq!(quantity(&c, &r), before);
        c = continue_save(root.clone(), r.clone(), &mut c);
        if read_first {
            hear_mason(&mut c, &r);
        } else {
            read_log(&mut c, &r);
        }
        assert!(porter(&mut c, &r).contains("road north"));
        assert!(flag(&c, GIVEN));
        assert_eq!(quantity(&c, &r), before + 1);
        c = continue_save(root.clone(), r.clone(), &mut c);
        assert!(porter(&mut c, &r).contains("gets home"));
        assert_eq!(quantity(&c, &r), before + 1);
        walk(&mut c, &r, (11, 17));
        enter(&mut c, &r);
        porter(&mut c, &r);
        assert_eq!(quantity(&c, &r), before + 1);
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing the Winona station"]
fn winona_station_controller_full_pocket_native_retry_and_no_continue_visit_grant() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, "MnWinona", (19, 14));
    enter(&mut c, &r);
    // A deliberately unvisited saved-room fixture isolates Continue callbacks.
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_event_flag(VISITED, false)
        .expect("Continue callback fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert!(
        !flag(&c, VISITED),
        "Continue cannot invent a first public visit"
    );
    walk(&mut c, &r, (10, 17));
    enter(&mut c, &r);
    c.shell.shell.session_mut().state_mut().bag.items.clear();
    c.shell
        .shell
        .add_bag_item("POTION", 1)
        .expect("tossable item fixture");
    while c.shell.shell.session().state().bag.items.len()
        < crystal_core::models::bag::ITEM_POCKET_CAPACITY
    {
        c.shell
            .shell
            .add_bag_item("BERRY", 99)
            .expect("full physical pocket fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    read_log(&mut c, &r);
    hear_mason(&mut c, &r);
    assert!(porter(&mut c, &r).contains("hold it"));
    assert!(!flag(&c, GIVEN));
    assert_eq!(quantity(&c, &r), 0);
    c = continue_save(root.clone(), r.clone(), &mut c);
    super::detroit_lakes::toss_potion(&mut c);
    porter(&mut c, &r);
    assert!(flag(&c, GIVEN));
    assert_eq!(quantity(&c, &r), 1);
    c = continue_save(root, r.clone(), &mut c);
    porter(&mut c, &r);
    assert_eq!(quantity(&c, &r), 1);
}

#[cfg(feature = "voxel-view")]
#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing all four public station halls"]
fn minnesota_station_preload_builds_real_furniture_without_granting_a_visit() {
    use bevy::prelude::{Assets, Image};

    use super::super::{RenderedTilesetArt, TilePosition, destination_preload};

    let (root, r, _) = load();
    let c = fixture(&root, &r, "MnWinona", (19, 14));
    let snapshot = c.shell.shell.snapshot().expect("authoritative town state");
    let before = snapshot.state_checksum.clone();
    assert!(!flag(&c, VISITED));
    for (name, at) in [
        ("MnWinonaStation", [11, 15]),
        ("MnDetroitLakesStation", [11, 17]),
        ("MnStCloudStation", [11, 17]),
        ("MnStPaulUnionDepot", [15, 21]),
    ] {
        let map = snapshot
            .maps
            .iter()
            .find(|map| map.map_name == name)
            .expect("real station");
        let mut images = Assets::<Image>::default();
        let frame = destination_preload::source_frame(
            &snapshot,
            &c.shell,
            map,
            TilePosition::new(at[0], at[1]),
            "indoor",
            &mut RenderedTilesetArt::default(),
            &mut images,
        )
        .expect("production speculative station source");
        let mesh = crystal_voxel_view::audit_terrain_mesh_with_images(&frame, &images)
            .expect("real source-backed station models");
        assert!(
            mesh.authored_cells.contains(&Some("interior/long_planter")),
            "{name}: native plants must become actual modeled fixtures"
        );
        assert!(
            mesh.authored_cells
                .contains(&Some("interior/wall_clinical")),
            "{name}: complete back walls must retain native model ownership"
        );
        let notices = frame
            .tiles
            .iter()
            .filter(|tile| {
                tile.source.tileset_id.as_ref() == "train_station"
                    && tile.source.metatile_id == 0x25
                    && tile.source.subtile_row == 0
                    && tile.source.subtile_column % 2 == 0
            })
            .collect::<Vec<_>>();
        assert!(
            !notices.is_empty(),
            "{name}: the real waiting-room notices must be present"
        );
        for tile in notices {
            let index = tile.row as usize * frame.grid_size.x as usize + tile.column as usize;
            assert_eq!(
                mesh.authored_cells[index],
                Some("interior/picture_frame"),
                "{name}: each complete notice needs its own real model"
            );
        }
        assert!(
            mesh.solid
                .positions
                .iter()
                .any(|position| position[1] > 8.0),
            "{name}: source classification alone is not raised geometry"
        );
    }
    assert!(!flag(&c, VISITED));
    assert_eq!(
        c.shell
            .shell
            .snapshot()
            .expect("after scenery preparation")
            .state_checksum,
        before
    );
}
