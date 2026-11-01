//! Declared local setup, followed by real winter-town walks, battle and Continue.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life;
use super::northstar_ice_lair::{choice, event, talk, walk};
use super::r10_gallery::press;
use super::winona::{at, finish};

const TOWN: &str = "MnInternationalFalls";
const SKIER: &str = "MN_EXTRA_FALLS_SKIER";
const WON: &str = "EVENT_MN_EXTRA_FALLS_SKIER";

fn output(name: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_FALLS_WINDBREAK_REVIEW_DIR")
            .expect("fresh winter-town review directory"),
    )
    .join(name);
    assert!(!path.exists(), "retain every winter-town review");
    std::fs::create_dir_all(&path).expect("winter-town review directory");
    path
}

fn fixture(root: &AssetRoot, r: &CrystalRuntime) -> VisibleShellController {
    let mut c = interior_life::fixture(root, r, TOWN, (20, 25));
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["VOYAGEAR", "CINDORE", "FORTIGARD", "MUDMENDER"] {
        let mut partner = r
            .data()
            .create_pokemon(species, 60, Default::default())
            .expect("active original Minnesota partner");
        if species == "VOYAGEAR" {
            // Force an ordinary healing choice in the real battle. MAX_REPEL
            // precedes HYPER_POTION in this pocket but cannot be used in battle.
            partner.hp = partner.max_hp / 3;
        }
        partner.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(partner));
    }
    state.sync_party_from_storage();
    state.money = 5000;
    for (i, name) in BADGE_ENGINE_FLAGS[..8].iter().enumerate() {
        state.badges.johto[i] = i < 7;
        state
            .flags
            .set_engine_flag(name, i < 7)
            .expect("declared initial seven badges");
    }
    for (item, count) in [("MAX_REPEL", 10), ("HYPER_POTION", 10)] {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], count)
                .expect("declared local supplies")
        );
    }
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    // No position, party, flag or inventory writes after the initial fixture.
    c
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    path: PathBuf,
) -> VisibleShellController {
    assert!(!c.has_pending_script_work(), "released native scene");
    assert!(!path.exists(), "retain the prior checkpoint");
    c.save(&path)
        .expect("actual reached winter-town checkpoint");
    let mut next = VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("same-pack Continue");
    next.set_runtime_journal_enabled(false);
    next
}

fn no_story_grant(c: &VisibleShellController) {
    for name in [
        "EVENT_MN_BORDER_REPORT",
        "EVENT_MN_FALLS_VOYAGEURS_ARRIVAL",
        "EVENT_MN_BORDER_CIRCUIT_REWARDED",
        "EVENT_MN_LIGHTHOUSE_SIGNAL",
    ] {
        assert!(!event(c, name), "winter-town loop cannot grant {name}");
    }
}

#[test]
#[ignore = "requires the completed external winter-town pack and fresh review directory"]
fn falls_windbreak_controller_service_loops_optional_battle_pp_prize_and_continue() {
    let out = output("skier");
    let (root, r, _) = interior_life::load();
    let mut c = fixture(&root, &r);
    c.save(out.join("initial-local-town.crystalsave"))
        .expect("explicit initial local setup");
    for point in [
        (19, 22),
        (13, 20),
        (8, 17),
        (7, 12),
        (9, 8),
        (14, 11),
        (23, 10),
        (27, 12),
        (27, 14),
        (23, 17),
        (15, 14),
        (17, 13),
    ] {
        walk(&mut c, &r, point);
    }
    no_story_grant(&c);
    let species =
        serde_json::to_value(&r.data().pokemon["LUTEBRISK"]).expect("native Lutebrisk types");
    assert_eq!(species["type1"], "ICE");
    assert_eq!(species["type2"], "ICE");
    let party = c.snapshot().expect("before offer").party;
    talk(&mut c, GameButton::Up);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, WON));
    assert_eq!(c.snapshot().expect("declined party").party, party);
    assert_eq!(c.shell.shell.session().state().money, 5000);
    c = continued(&mut c, &root, &r, out.join("skier-declined.crystalsave"));
    let before = c.snapshot().expect("before actual battle");
    talk(&mut c, GameButton::Up);
    choice(&mut c, true);
    assert!(super::rochester_earned::try_battle(&mut c, &r, SKIER, WON));
    finish(&mut c);
    assert!(event(&c, WON));
    let after = c.snapshot().expect("native victory");
    assert!(
        after.party.slots.iter().any(|p| {
            before.party.slots.iter().any(|old| {
                old.index == p.index
                    && p.pokemon.moves.iter().any(|m| {
                        old.pokemon
                            .moves
                            .iter()
                            .any(|prior| prior.name == m.name && m.current_pp < prior.current_pp)
                    })
            })
        }),
        "the actual battle must consume native move PP"
    );
    assert!(c.shell.shell.session().state().money > 5000, "native prize");
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["HYPER_POTION"])
            < 10,
        "native Pack targeting must select and consume the carried healing item"
    );
    no_story_grant(&c);
    c = continued(&mut c, &root, &r, out.join("skier-won.crystalsave"));
    talk(&mut c, GameButton::Up);
    finish(&mut c);
    assert!(c.snapshot().expect("repeat conversation").battle.is_none());
    walk(&mut c, &r, (13, 20));
    walk(&mut c, &r, (20, 25));
    no_story_grant(&c);
    c.save(out.join("returned-southern-street.crystalsave"))
        .expect("actual full town return");
    press(&mut c, GameButton::Start);
    assert!(c.snapshot().expect("production Start").ui.menu.is_some());
    press(&mut c, GameButton::B);
}

#[test]
#[ignore = "requires the completed external winter-town pack and fresh review directory"]
fn falls_windbreak_controller_pines_block_bypasses_but_both_loops_remain_connected() {
    let out = output("windbreaks");
    let (root, r, _) = interior_life::load();
    let mut c = fixture(&root, &r);
    for (point, direction) in [
        ((12, 11), GameButton::Down),
        ((13, 20), GameButton::Up),
        ((27, 14), GameButton::Right),
    ] {
        walk(&mut c, &r, point);
        for _ in 0..3 {
            press(&mut c, direction);
            at(&mut c, TOWN, point);
        }
    }
    for point in [(23, 10), (14, 11), (9, 8), (8, 17), (13, 20), (20, 25)] {
        walk(&mut c, &r, point);
    }
    c = continued(&mut c, &root, &r, out.join("loops-continued.crystalsave"));
    at(&mut c, TOWN, (20, 25));
    walk(&mut c, &r, (9, 8));
    c.save(out.join("reached-clinic-front.crystalsave"))
        .expect("actual post-Continue clinic approach");
    no_story_grant(&c);
}
