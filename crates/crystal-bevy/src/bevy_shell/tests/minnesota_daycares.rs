//! Explicit location/party fixtures. Travel, menus, custody and Continue use
//! production inputs; these tests do not claim earned regional progression.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::indoor_walk;
use super::interior_life::fixture;
use super::r10_gallery::{continue_save, press, settle};
use super::winona::travel;

fn load() -> (AssetRoot, CrystalRuntime) {
    static PACK: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    PACK.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_DAY_CARE_TEST_PACK").expect("published daycare pack"),
        );
        let root = AssetRoot::new(path.parent().expect("pack parent"));
        let runtime = CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(path).expect("verified daycare pack"),
        )
        .expect("daycare runtime");
        (root, runtime)
    })
    .clone()
}

fn prepare(
    root: &AssetRoot,
    r: &CrystalRuntime,
    city: &str,
    landing: (i16, i16),
) -> VisibleShellController {
    let mut c = fixture(root, r, city, landing);
    let state = c.shell.shell.session_mut().state_mut();
    for (i, species) in ["POUCHID", "POUCHID", "JOTLET"].into_iter().enumerate() {
        let dvs = if i == 1 {
            Dv::from_non_hp(15, 1, 1, 1)
        } else {
            Dv::from_non_hp(0, 2, 2, 2)
        };
        let mut partner = r
            .data()
            .create_pokemon(species, 5, dvs)
            .expect("regional partner");
        partner.nickname = format!("PARTNER{i}");
        partner.original_trainer_name = state.player_name.clone();
        partner.original_trainer_id = state.player_id;
        if i == 0 {
            partner.item = Some("BERRY".into());
        }
        partner.experience = crystal_core::systems::experience::calculate_experience(
            r.growth_rates(),
            &partner.species.growth_rate,
            6,
        )
        .unwrap()
            - 15;
        assert!(state.storage.party.add_pokemon(partner));
    }
    state.sync_party_from_storage();
    state.money = 10_000;
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}

fn choice(c: &mut VisibleShellController, yes: bool) {
    for _ in 0..128 {
        c.wait_frames(128).expect("daycare prompt presentation");
        if c.shell.pc_confirmation.is_some()
            && let Some(before) = c
                .shell
                .yes_no_cursor
                .as_ref()
                .map(|cursor| cursor.option_index)
        {
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
    panic!(
        "native daycare confirmation did not accept input: {:?}",
        c.snapshot()
    );
}

fn begin(c: &mut VisibleShellController) {
    press(c, GameButton::Up);
    press(c, GameButton::A);
}

fn deposit(c: &mut VisibleShellController) {
    begin(c);
    choice(c, true);
    for _ in 0..128 {
        c.wait_frames(64).unwrap();
        if c.shell.party_menu_open {
            break;
        }
        press(c, GameButton::A);
    }
    assert!(c.shell.party_menu_open, "native daycare party picker");
    press(c, GameButton::A);
    settle(c, "daycare deposit");
}

fn at(c: &mut VisibleShellController, map: &str, tile: (i16, i16)) {
    let s = c.snapshot().unwrap();
    assert_eq!(s.overworld.map_name, map);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), tile);
    if let Some(id) = crystal_core::day_care::location_id(map) {
        assert_eq!(
            c.shell.shell.session().state().current_day_care_id(),
            Some(id),
            "the production warp must select this site's custody"
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_DAY_CARE_TEST_PACK"]
fn day_care_controller_both_houses_enter_cancel_deposit_grow_save_retrieve_exit() {
    let (root, r) = load();
    for (city, room, landing) in [
        ("MnMankato", "MnMankatoDayCare", (21, 28)),
        ("MnBemidji", "MnBemidjiDayCare", (13, 34)),
    ] {
        let mut c = prepare(&root, &r, city, landing);
        c = capture(c, &format!("{city}-exterior-v1.png"));
        travel(&mut c, GameButton::Up, room, (2, 6));
        at(&mut c, room, (2, 6));
        indoor_walk(&mut c, &r, (2, 4));
        // Declining the introduction and cancelling the picker leave custody empty.
        begin(&mut c);
        press(&mut c, GameButton::Down);
        at(&mut c, room, (2, 4)); // Dialog owns movement.
        choice(&mut c, false);
        settle(&mut c, "declined daycare");
        assert!(c.snapshot().unwrap().day_care.man.pokemon.is_none());
        begin(&mut c);
        choice(&mut c, true);
        for _ in 0..128 {
            if c.shell.party_menu_open {
                break;
            }
            press(&mut c, GameButton::A);
        }
        assert!(c.shell.party_menu_open);
        press(&mut c, GameButton::B);
        settle(&mut c, "cancelled party selection");
        assert_eq!(c.snapshot().unwrap().party.slots.len(), 3);
        assert!(c.snapshot().unwrap().day_care.man.pokemon.is_none());
        deposit(&mut c);
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .day_care
                .man
                .pokemon
                .is_none(),
            "regional deposits cannot enter the legacy shared slot"
        );
        assert_eq!(c.shell.shell.session().state().day_care_locations.len(), 1);
        assert_eq!(c.snapshot().unwrap().party.slots.len(), 2);
        assert_eq!(
            c.snapshot()
                .unwrap()
                .day_care
                .man
                .pokemon
                .as_ref()
                .unwrap()
                .nickname,
            "PARTNER0"
        );
        c = continue_save(root.clone(), r.clone(), &mut c);
        assert_eq!(
            c.snapshot()
                .unwrap()
                .day_care
                .man
                .pokemon
                .as_ref()
                .unwrap()
                .nickname,
            "PARTNER0"
        );
        indoor_walk(&mut c, &r, (2, 5));
        for _ in 0..30 {
            indoor_walk(&mut c, &r, (3, 5));
            indoor_walk(&mut c, &r, (2, 5));
        }
        indoor_walk(&mut c, &r, (2, 4));
        let custody = c.snapshot().unwrap().day_care;
        begin(&mut c);
        choice(&mut c, true); // Request the growth and fee report.
        choice(&mut c, false); // Cancel at the actual price confirmation.
        settle(&mut c, "cancelled daycare withdrawal");
        assert_eq!(c.snapshot().unwrap().day_care, custody);
        assert_eq!(c.shell.shell.session().state().money, 10_000);
        begin(&mut c);
        choice(&mut c, true); // Growth report.
        c = capture(c, &format!("{city}-fee-v1.png"));
        choice(&mut c, true); // Quoted fee.
        settle(&mut c, "retrieved daycare partner");
        assert!(c.snapshot().unwrap().day_care.man.pokemon.is_none());
        let p = c
            .shell
            .shell
            .session()
            .state()
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .find(|p| p.nickname == "PARTNER0")
            .unwrap();
        assert_eq!(p.species.id, "POUCHID");
        assert_eq!(p.level, 6);
        assert_eq!(p.item.as_deref(), Some("BERRY"));
        assert_eq!(c.shell.shell.session().state().money, 9800);
        c = capture(c, &format!("{city}-interior-v1.png"));
        indoor_walk(&mut c, &r, (2, 6));
        travel(&mut c, GameButton::Down, city, landing);
        at(&mut c, city, landing);
    }
}

#[test]
#[ignore = "requires CRYSTAL_DAY_CARE_TEST_PACK"]
fn day_care_controller_each_site_breeds_and_offers_only_one_egg() {
    let (root, r) = load();
    for (city, room, landing) in [
        ("MnMankato", "MnMankatoDayCare", (21, 28)),
        ("MnBemidji", "MnBemidjiDayCare", (13, 34)),
    ] {
        let mut c = prepare(&root, &r, city, landing);
        travel(&mut c, GameButton::Up, room, (2, 6));
        at(&mut c, room, (2, 6));
        indoor_walk(&mut c, &r, (2, 4));
        deposit(&mut c);
        indoor_walk(&mut c, &r, (5, 4));
        deposit(&mut c);
        assert!(c.snapshot().unwrap().day_care.compatibility_score > 0);
        indoor_walk(&mut c, &r, (2, 5));
        for _ in 0..4000 {
            if c.snapshot().unwrap().day_care.egg_present {
                break;
            }
            indoor_walk(&mut c, &r, (3, 5));
            indoor_walk(&mut c, &r, (2, 5));
        }
        assert!(
            c.snapshot().unwrap().day_care.egg_present,
            "compatible regional parents produce an egg"
        );
        c = continue_save(root.clone(), r.clone(), &mut c);
        indoor_walk(&mut c, &r, (6, 6));
        begin(&mut c);
        choice(&mut c, true);
        c = capture(c, &format!("{city}-egg-received-v1.png"));
        settle(&mut c, "egg handoff");
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .storage
                .party
                .pokemon
                .iter()
                .flatten()
                .filter(|p| p.is_egg)
                .count(),
            1
        );
        assert!(!c.snapshot().unwrap().day_care.egg_present);
        begin(&mut c);
        settle(&mut c, "no duplicate egg");
        assert_eq!(c.snapshot().unwrap().party.slots.len(), 2);
    }
}

#[test]
#[ignore = "requires CRYSTAL_DAY_CARE_TEST_PACK"]
fn day_care_controller_save_during_intro_does_not_commit_or_trap_the_player() {
    let (root, r) = load();
    for (city, room, landing) in [
        ("MnMankato", "MnMankatoDayCare", (21, 28)),
        ("MnBemidji", "MnBemidjiDayCare", (13, 34)),
    ] {
        let mut c = prepare(&root, &r, city, landing);
        travel(&mut c, GameButton::Up, room, (2, 6));
        indoor_walk(&mut c, &r, (2, 4));
        begin(&mut c);
        assert!(c.shell.pc_confirmation.is_some());
        let party = c.shell.shell.session().state().storage.party.clone();
        let custody = c.shell.shell.session().state().day_care_locations.clone();
        let money = c.shell.shell.session().state().money;
        let path = super::authored_route_fixture::reached_save_path(&format!(
            "daycare-interrupted-{city}-{}.crystalsave",
            std::process::id()
        ));
        c.save(&path)
            .expect("checkpoint while an uncommitted service offer is open");
        assert!(
            c.shell.pc_confirmation.is_some(),
            "saving cannot consume the live offer"
        );
        let mut c = VisibleShellController::load_save(root.clone(), r.clone(), path.clone(), None)
            .expect("Continue an interrupted offer");
        super::authored_route_fixture::cleanup_reached_save(&path);
        for _ in 0..128 {
            let snapshot = c.snapshot().unwrap();
            if !snapshot.ui.text_window_open
                && !c.has_pending_script_work()
                && c.shell.pc_confirmation.is_none()
            {
                break;
            }
            if c.shell.pc_confirmation.is_some() {
                choice(&mut c, false);
            } else if c.shell.party_menu_open {
                press(&mut c, GameButton::B);
            } else {
                press(&mut c, GameButton::A);
            }
        }
        assert!(
            !c.snapshot().unwrap().ui.text_window_open,
            "interrupted daycare offer must release input"
        );
        assert_eq!(c.shell.shell.session().state().storage.party, party);
        assert_eq!(c.shell.shell.session().state().day_care_locations, custody);
        assert_eq!(c.shell.shell.session().state().money, money);
        at(&mut c, room, (2, 4));
        deposit(&mut c);
        assert_eq!(c.snapshot().unwrap().party.slots.len(), 2);
    }
}

fn capture(mut c: VisibleShellController, name: &str) -> VisibleShellController {
    use bevy::prelude::{Assets, Image, Mut};
    let Some(directory) = std::env::var_os("CRYSTAL_DAY_CARE_RENDER_DIR") else {
        return c;
    };
    c.wait_frames(128).unwrap();
    let mut app = super::menu_render_test_app(c.shell);
    app.update();
    app.update();
    let world = app.world_mut();
    let image = world.resource_scope(|world, images: Mut<Assets<Image>>| {
        super::pokemon_overworld_context_canvas(world, &images)
    });
    let path = PathBuf::from(directory).join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    assert!(!path.exists(), "retain native daycare renders");
    image.save(path).expect("native daycare capture");
    VisibleShellController {
        shell: world
            .remove_resource::<super::super::BevyRuntimeShell>()
            .unwrap(),
    }
}
