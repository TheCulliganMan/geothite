//! Explicit local access only; subsequent town, pier and battle actions use input.
use std::path::PathBuf;

use bevy::asset::Assets;
use bevy::image::Image;
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::LearnedMove;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, load_tileset_art, mark_runtime_snapshot_dirty};
use super::interior_life;
use super::northstar_ice_lair::{choice, event, talk, walk};
use super::r10_gallery::{press, settle};
use super::winona::{at, finish, load, travel};

const TOWN: &str = "MnGrandRapids";
const FISHER: &str = "MN_EXTRA_GR_BANK_FISHER";
const WON: &str = "EVENT_MN_EXTRA_GR_BANK_FISHER";

#[test]
#[ignore = "requires the completed external riverfront pack via CRYSTAL_WINONA_TEST_PACK"]
fn gr_riverfront_controller_native_animation_retains_source_frames_at_all_times() {
    let (_, runtime) = load();
    let root = runtime
        .materialize_runtime_files()
        .expect("actual riverfront runtime art");
    let mut images = Assets::<Image>::default();
    for time in ["morn", "day", "nite"] {
        let mut load_art = |id: &str| {
            load_tileset_art(
                &root,
                id,
                time,
                &runtime.data().tilesets[id].palette_map,
                &mut images,
            )
            .expect("native time-aware town art")
        };
        let source = load_art("johto_modern");
        let town = load_art("mn_grand_rapids_riverfront");
        assert_eq!(source.animated_tiles.len(), town.animated_tiles.len());
        for glyph in [0x03, 0x14] {
            let expected = &source.animated_tiles[&glyph];
            let actual = &town.animated_tiles[&glyph];
            assert_eq!(actual.frame_ticks, expected.frame_ticks);
            assert_eq!(actual.phase_offset, expected.phase_offset);
            assert_eq!(
                actual.advance_on_phase_offset,
                expected.advance_on_phase_offset
            );
            assert_eq!(
                actual.requires_forest_restless,
                expected.requires_forest_restless
            );
            assert_eq!(actual.cave_water_composite, expected.cave_water_composite);
            assert_eq!(actual.frames.len(), expected.frames.len());
            for (a, b) in actual.frames.iter().zip(expected.frames.iter()) {
                assert_eq!(
                    images.get(a).expect("actual animation frame").data,
                    images.get(b).expect("original animation frame").data,
                    "native glyph {glyph:02x} at {time}"
                );
            }
        }
    }
}

fn output(name: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_GR_RIVERFRONT_REVIEW_DIR").expect("fresh riverfront output"),
    )
    .join(name);
    assert!(!path.exists(), "retain every riverfront review");
    std::fs::create_dir_all(&path).expect("riverfront review directory");
    path
}

fn fixture(
    root: &AssetRoot,
    r: &CrystalRuntime,
    surf: bool,
    badge: bool,
) -> VisibleShellController {
    let mut c = interior_life::fixture(root, r, TOWN, (6, 24));
    let state = c.shell.shell.session_mut().state_mut();
    for species in ["VOYAGEAR", "CINDORE", "FORTIGARD", "MUDMENDER"] {
        let mut p = r
            .data()
            .create_pokemon(species, 50, Default::default())
            .expect("original active regional partner");
        assert!(p.moves.iter().all(|m| m.name != "SURF"));
        if species == "VOYAGEAR" && surf {
            assert!(
                r.data().pokemon[species]
                    .tmhm_learnset
                    .iter()
                    .any(|m| m == "SURF")
            );
            let m = LearnedMove {
                name: "SURF".into(),
                current_pp: r.data().moves["SURF"].pp,
                pp_ups: 0,
            };
            if p.moves.len() == 4 {
                p.moves[3] = m;
            } else {
                p.moves.push(m);
            }
        }
        p.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(p));
    }
    state.sync_party_from_storage();
    state.money = 5000;
    state.badges.johto[3] = badge;
    state
        .flags
        .set_engine_flag("ENGINE_FOGBADGE", badge)
        .expect("declared initial Surf prerequisite");
    for (item, count) in [("OLD_ROD", 1), ("MAX_REPEL", 10), ("HYPER_POTION", 10)] {
        assert!(
            state
                .bag
                .add_item(&r.data().items[item], count)
                .expect("declared local supplies")
        );
    }
    state.registered_key_item = Some("OLD_ROD".into());
    state.repel_steps_remaining = 250;
    state.active_repel_item = Some("MAX_REPEL".into());
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    c
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
    path: PathBuf,
) -> VisibleShellController {
    assert!(
        !c.has_pending_script_work(),
        "Continue uses released scenes"
    );
    assert!(!path.exists());
    c.save(&path).expect("actual reached riverfront checkpoint");
    let mut next = VisibleShellController::load_save(root.clone(), r.clone(), path, None)
        .expect("production Continue");
    next.set_runtime_journal_enabled(false);
    next
}

fn no_story_grant(c: &VisibleShellController) {
    for flag in [
        "EVENT_MN_DEPOT_FOREMAN_DEFEATED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
        "EVENT_MN_DEPOT_CLEARED",
    ] {
        assert!(
            !event(c, flag),
            "optional riverfront activity cannot grant {flag}"
        );
    }
}

#[test]
#[ignore = "requires current CRYSTAL_WINONA_TEST_PACK and fresh CRYSTAL_GR_RIVERFRONT_REVIEW_DIR; declared local party only"]
fn gr_riverfront_controller_optional_fisher_refusal_real_pp_victory_continue_and_town_loop() {
    let out = output("fisher");
    let (root, r) = load();
    let mut c = fixture(&root, &r, true, true);
    c.save(out.join("initial-local-town.crystalsave"))
        .expect("explicit initial local setup");
    // Walk the complete service and waterfront circuits using real collision.
    for point in [
        (9, 12),
        (23, 12),
        (27, 14),
        (39, 16),
        (38, 23),
        (16, 25),
        (20, 31),
        (31, 33),
        (26, 33),
        (26, 36),
    ] {
        walk(&mut c, &r, point);
    }
    no_story_grant(&c);
    assert!(!event(&c, WON));
    let party = c.snapshot().expect("party before offer").party;
    talk(&mut c, GameButton::Left);
    choice(&mut c, false);
    finish(&mut c);
    assert!(!event(&c, WON));
    assert_eq!(c.snapshot().expect("declined party").party, party);
    assert_eq!(c.shell.shell.session().state().money, 5000);
    c = continued(&mut c, &root, &r, out.join("fisher-declined.crystalsave"));
    let before = c.snapshot().expect("before actual battle");
    talk(&mut c, GameButton::Left);
    choice(&mut c, true);
    assert!(super::rochester_earned::try_battle(&mut c, &r, FISHER, WON));
    finish(&mut c);
    assert!(event(&c, WON));
    let after = c.snapshot().expect("actual native victory");
    assert!(
        after
            .party
            .slots
            .iter()
            .any(|p| before.party.slots.iter().any(|old| old.index == p.index
                && p.pokemon.moves.iter().any(|m| old
                    .pokemon
                    .moves
                    .iter()
                    .any(|om| om.name == m.name && m.current_pp < om.current_pp)))),
        "the real battle must consume native move PP"
    );
    assert!(
        c.shell.shell.session().state().money > 5000,
        "actual trainer prize"
    );
    c = continued(&mut c, &root, &r, out.join("fisher-won.crystalsave"));
    talk(&mut c, GameButton::Left);
    finish(&mut c);
    assert!(c.snapshot().expect("repeat conversation").battle.is_none());
    walk(&mut c, &r, (26, 38));
    c.save(out.join("reached-pier-cap.crystalsave"))
        .expect("actual cap approach");
    walk(&mut c, &r, (6, 24));
    no_story_grant(&c);
    c.save(out.join("returned-main-street.crystalsave"))
        .expect("ordinary main-street return");
    press(&mut c, GameButton::Start);
    assert!(c.snapshot().expect("Start").ui.menu.is_some());
    press(&mut c, GameButton::B);
}

#[test]
#[ignore = "requires current pack and fresh review output; separate declared missing-move and missing-badge fixtures"]
fn gr_riverfront_controller_pier_walks_without_surf_and_retains_move_and_badge_denial() {
    let (root, r) = load();
    for (surf, badge, label) in [
        (false, true, "missing-move"),
        (true, false, "missing-badge"),
    ] {
        let out = output(label);
        let mut c = fixture(&root, &r, surf, badge);
        c.save(out.join("initial-local-access.crystalsave"))
            .expect("declared missing prerequisite");
        walk(&mut c, &r, (26, 38));
        talk(&mut c, GameButton::Down);
        settle(&mut c, "native field denial");
        for _ in 0..4 {
            press(&mut c, GameButton::Down);
            at(&mut c, TOWN, (26, 38));
        }
        assert_eq!(
            c.snapshot().expect("native denial").overworld.mode,
            MovementMode::Normal
        );
        no_story_grant(&c);
        c.save(out.join("denied-at-pier.crystalsave"))
            .expect("actual denied field action");
        walk(&mut c, &r, (26, 33));
        walk(&mut c, &r, (9, 12));
    }
}

#[test]
#[ignore = "requires current pack and fresh review output; explicit local Surf/registered Old Rod setup"]
fn gr_riverfront_controller_native_surf_continue_dismount_and_registered_fishing() {
    let out = output("water");
    let (root, r) = load();
    let mut c = fixture(&root, &r, true, true);
    c.save(out.join("initial-local-water-access.crystalsave"))
        .expect("declared local access");
    walk(&mut c, &r, (26, 38));
    super::minneapolis_falls::mount(&mut c, GameButton::Down, TOWN, (26, 39));
    super::pepin_discovery::paddle(&mut c, &r, (29, 40));
    c = continued(&mut c, &root, &r, out.join("actual-pond-surf.crystalsave"));
    assert_eq!(
        c.snapshot().expect("continued Surf").overworld.mode,
        MovementMode::Surf
    );
    super::pepin_discovery::paddle(&mut c, &r, (26, 39));
    travel(&mut c, GameButton::Up, TOWN, (26, 38));
    assert_eq!(
        c.snapshot().expect("actual dismount").overworld.mode,
        MovementMode::Normal
    );
    press(&mut c, GameButton::Down);
    at(&mut c, TOWN, (26, 38));
    c.save(out.join("actual-fishing-bank.crystalsave"))
        .expect("reached bank with registered rod");
    let mut caught_notice = false;
    for _ in 0..64 {
        press(&mut c, GameButton::Select);
        c.wait_frames(160).expect("native cast animation");
        let notice = c.shell.field_notice.clone().expect("native fishing notice");
        assert!(notice.contains("A bite!") || notice.contains("Not even a nibble!"));
        press(&mut c, GameButton::Left);
        at(&mut c, TOWN, (26, 38));
        press(&mut c, GameButton::A);
        if notice.contains("A bite!") {
            let snapshot = c.snapshot().expect("acknowledged fishing battle");
            let battle = snapshot.battle.as_ref().expect("native fishing battle");
            assert_eq!(battle.battle_type, "BATTLETYPE_FISH");
            let fishing = serde_json::to_value(&r.data().fishing).expect("native fishing catalog");
            let slots = fishing["groups"]["FISHGROUP_MN_SHORE"]["rod_tables"]["OLD_ROD"]["slots"]
                .as_array()
                .expect("current native Old Rod slots");
            assert!(
                slots
                    .iter()
                    .any(|slot| slot["species"] == battle.enemy_pokemon.species.id),
                "the real bite must use this town's original regional rod table"
            );
            c.save(out.join("actual-fishing-bite.crystalsave"))
                .expect("actual caught-line battle for inspection");
            super::north_shore::settle_water(&mut c);
            caught_notice = true;
            break;
        }
        settle(&mut c, "completed no-bite cast");
    }
    assert!(
        caught_notice,
        "native casting must eventually produce an actual bite"
    );
    at(&mut c, TOWN, (26, 38));
    no_story_grant(&c);
    c.save(out.join("returned-from-fishing.crystalsave"))
        .expect("actual native escape/return");
}
