//! Local party/position fixtures; each rod is actually awarded before fishing.
#[path = "minnesota_fishing_walk.rs"]
mod dry_walk;
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::{
    FieldPackPocket, StartMenuOption, VisibleShellController, mark_runtime_snapshot_dirty,
    visible_start_menu_options,
};
use super::interior_life::{collision, fixture, load, talk};
use super::r10_gallery::{continue_save, settle};
use super::winona::press;

fn shore(c: &mut VisibleShellController, r: &CrystalRuntime) -> ((i16, i16), GameButton) {
    let here = c.snapshot().expect("actual guru-house return").overworld;
    let map = &r.data().maps[&here.map_name];
    let mut choices = Vec::new();
    for y in 0..i16::try_from(map.attributes.height * 2).expect("native height") {
        for x in 0..i16::try_from(map.attributes.width * 2).expect("native width") {
            if !matches!(collision(r, map, (x, y)), Some("FLOOR" | "TALL_GRASS")) {
                continue;
            }
            for ((dx, dy), facing) in [
                ((0, -1), GameButton::Up),
                ((0, 1), GameButton::Down),
                ((-1, 0), GameButton::Left),
                ((1, 0), GameButton::Right),
            ] {
                if collision(r, map, (x + dx, y + dy)) == Some("WATER") {
                    choices.push((
                        (x - here.tile.x).abs() + (y - here.tile.y).abs(),
                        (x, y),
                        facing,
                    ));
                }
            }
        }
    }
    choices.sort_by_key(|choice| choice.0);
    choices
        .into_iter()
        .find_map(|(_, at, facing)| dry_walk::path(c, r, at).map(|_| (at, facing)))
        .unwrap_or_else(|| {
            panic!(
                "{}: no reachable original fishing shore from {:?}",
                here.map_name, here.tile
            )
        })
}

fn walk_to_shore(c: &mut VisibleShellController, r: &CrystalRuntime, at: (i16, i16)) {
    for _ in 0..512 {
        if c.snapshot()
            .expect("ordinary bank approach")
            .battle
            .is_some()
        {
            assert!(super::rochester_earned::try_battle(
                c,
                r,
                "BANK_APPROACH",
                ""
            ));
            continue;
        }
        let path = dry_walk::path(c, r, at).expect("actual dry bank path");
        let Some(button) = path.first() else {
            return;
        };
        press(c, *button);
    }
    panic!("did not reach fishing bank {at:?}");
}

fn use_awarded_rod(c: &mut VisibleShellController, rod: &str) {
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("Start for awarded rod");
    let target = visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|option| *option == StartMenuOption::Pack)
        .expect("PACK option");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if selected == target {
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::KeyItems) {
            break;
        }
        press(c, GameButton::Right);
    }
    let snapshot = c.snapshot().expect("actual awarded key item");
    assert_eq!(snapshot.bag.key_items.len(), 1);
    assert_eq!(snapshot.bag.key_items[0].item_id, rod);
    press(c, GameButton::A);
    press(c, GameButton::A);
    assert!(
        c.shell.visible_fishing_animation.is_some(),
        "native USE must start {rod}"
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the three native guru houses and regional fishing"]
fn minnesota_awarded_rods_native_gifts_pack_casts_encounters_and_continue() {
    let (root, runtime, _) = load();
    for (room, rod) in [
        ("MnDriftlessFishingGuruHouse", "OLD_ROD"),
        ("MnRiverValleyFishingGuruHouse", "GOOD_ROD"),
        ("MnPineCountryFishingGuruHouse", "SUPER_ROD"),
    ] {
        let mut c = fixture(&root, &runtime, room, (5, 4));
        let mut partner = runtime
            .data()
            .create_pokemon("BOREALYNX", 40, Dv::default())
            .expect("declared regional bank companion");
        partner.item = Some("SMOKE_BALL".into());
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut c.shell);
        // No bag, party, position, progression, HP/PP or RNG writes afterward.
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[rod]),
            0
        );
        talk(&mut c, (5, 3));
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[rod]),
            1
        );
        super::interior_life::walk(&mut c, &runtime, (2, 7));
        let (bank, facing) = shore(&mut c, &runtime);
        walk_to_shore(&mut c, &runtime, bank);
        press(&mut c, facing);
        let mut encountered = false;
        for _ in 0..64 {
            use_awarded_rod(&mut c, rod);
            for _ in 0..512 {
                c.wait_frames(1).expect("native fishing clock");
                if c.shell
                    .visible_fishing_animation
                    .as_ref()
                    .is_some_and(|animation| {
                        animation.phase == super::super::VisibleFishingPhase::AwaitText
                    })
                {
                    break;
                }
            }
            for _ in 0..128 {
                let snapshot = c.snapshot().expect("actual fishing notice or encounter");
                if snapshot.battle.is_some()
                    || (c.shell.visible_fishing_animation.is_none()
                        && c.shell.field_notice.is_none()
                        && !c.has_pending_script_work())
                {
                    break;
                }
                press(&mut c, GameButton::A);
            }
            if let Some(battle) = c.snapshot().expect("native rod result").battle {
                let species = &battle.enemy_pokemon.species.id;
                assert!(
                    runtime
                        .data()
                        .story_event_script_constants
                        .global
                        .contains_key(&format!("MN_DEX_MEMBER_{species}")),
                    "regional fish: {species}"
                );
                super::rochester::flee(&mut c);
                encountered = true;
                break;
            }
            settle(&mut c, "native unsuccessful cast");
            for _ in 0..8 {
                press(&mut c, GameButton::B);
            }
        }
        assert!(
            encountered,
            "awarded {rod} must select a native fish through ordinary RNG"
        );
        settle(&mut c, "native fishing escape");
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[rod]),
            1
        );
        let reached_map = c
            .snapshot()
            .expect("reached fishing bank")
            .overworld
            .map_name;
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[rod]),
            1
        );
        let here = c.snapshot().expect("continued fishing bank").overworld;
        assert_eq!((here.tile.x, here.tile.y), bank);
        assert_eq!(here.map_name, reached_map);
    }
}
