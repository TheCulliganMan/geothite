//! Local Minnesota party/position fixtures; all walking and ledges use input.
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use serde_json::Value;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, talk, walk};
use super::r10_gallery::{continue_save, press};

const SHORE: &str = "MnR11C00R02";
fn at(c: &mut VisibleShellController) -> (String, (i16, i16)) {
    let snapshot = c.snapshot().expect("pine-knoll snapshot");
    (
        snapshot.overworld.map_name,
        (snapshot.overworld.tile.x, snapshot.overworld.tile.y),
    )
}
fn point(v: &Value) -> (i16, i16) {
    (
        v[0].as_i64().expect("course x") as i16,
        v[1].as_i64().expect("course y") as i16,
    )
}
fn course(c: &mut VisibleShellController, key: &str) -> usize {
    let path = PathBuf::from(std::env::var_os("CRYSTAL_ELY_KNOLL_PLAN").expect("native course"));
    let plan: Value = serde_json::from_slice(&std::fs::read(path).expect("course file"))
        .expect("authored native course");
    use sha2::{Digest, Sha256};
    let pack = std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("published pack");
    assert_eq!(
        plan["pack_sha256"]
            .as_str()
            .expect("native proof pack hash"),
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(pack).expect("pack bytes"))
        )
    );
    let mut jumps = 0;
    for step in plan[key]["steps"].as_array().expect("course steps") {
        assert_eq!(at(c), (SHORE.into(), point(&step["from_core"])));
        let button = match step["button"].as_str().expect("input") {
            "left" => GameButton::Left,
            "right" => GameButton::Right,
            "up" => GameButton::Up,
            "down" => GameButton::Down,
            unknown => panic!("unknown authored input {unknown}"),
        };
        if step["jump"] == true {
            save_view(c, "ledge-lip");
        }
        for _ in 0..4 {
            if at(c).1 == point(&step["to_core"]) {
                break;
            }
            press(c, button);
        }
        assert_eq!(at(c), (SHORE.into(), point(&step["to_core"])));
        assert!(c.snapshot().expect("walking").battle.is_none());
        if step["jump"] == true {
            save_view(c, "ledge-landing");
            for _ in 0..3 {
                press(c, GameButton::Right);
                assert_eq!(at(c), (SHORE.into(), point(&step["to_core"])));
            }
        }
        jumps += usize::from(step["jump"] == true);
    }
    jumps
}
fn save_view(c: &mut VisibleShellController, name: &str) {
    if let Some(directory) = std::env::var_os("CRYSTAL_ELY_KNOLL_SAVES") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("reached saves directory");
        c.save(directory.join(format!("{name}.crystalsave")))
            .expect("actual reached native save");
    }
}
fn add_partner(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime, repel: bool) {
    let mut partner = r
        .data()
        .create_pokemon("KINDLIT", 60, Dv::default())
        .expect("active Minnesota fixture partner");
    if repel {
        partner.hp = 3;
        partner.status = Some("BURN".into());
        partner.moves[0].current_pp = 0;
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    if repel {
        // The preserved wild-challenge rules anchor at the highest three
        // levels. A solo lead can meet equal-level wilds through Repel; these
        // explicit reserves make this geometry fixture's native Repel reliable.
        for species in ["BIRCHIT", "POUCHID"] {
            let reserve = r
                .data()
                .create_pokemon(species, 30, Dv::default())
                .expect("active regional reserve fixture");
            assert!(state.storage.party.add_pokemon(reserve));
        }
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&r.data().items["MAX_REPEL"], 1)
                .expect("fixture repel")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    if repel {
        super::minneapolis_falls::grand_marais_harbor::use_max_repel(c, r);
    }
}

#[test]
#[ignore = "requires published Minnesota pine-knoll pack and authored course"]
fn ely_knolls_grass_climb_ledge_return_clinic_and_continue_use_native_input() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, SHORE, (0, 30));
    add_partner(&mut c, &r, true);
    let before_money = c.shell.shell.session().state().money;
    assert_eq!(course(&mut c, "eastbound"), 0);
    save_view(&mut c, "center-arrival");
    for _ in 0..6 {
        if at(&mut c).0 != SHORE {
            break;
        }
        press(&mut c, GameButton::Up);
    }
    assert_eq!(at(&mut c), ("MnSplitRockPokecenter1F".into(), (3, 5)));
    walk(&mut c, &r, (3, 3));
    assert!(!talk(&mut c, (3, 2)).is_empty());
    let healed = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("healed Minnesota partner");
    assert_eq!(healed.hp, healed.max_hp);
    assert!(healed.status.is_none());
    assert!(
        healed
            .moves
            .iter()
            .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    );
    walk(&mut c, &r, (4, 7));
    assert_eq!(at(&mut c), (SHORE.into(), (33, 34)));
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert_eq!(course(&mut c, "dry_return"), 1);
    save_view(&mut c, "pine-knoll-return");
    assert_eq!(at(&mut c), (SHORE.into(), (0, 30)));
    c = continue_save(root.clone(), r.clone(), &mut c);
    assert_eq!(at(&mut c), (SHORE.into(), (0, 30)));
    assert_eq!(c.shell.shell.session().state().money, before_money);

    // The reverse check happens at the actual landing inside course(). The
    // ordinary uphill detour remains available after returning and continuing.
    course(&mut c, "eastbound");
}

#[test]
#[ignore = "requires published Minnesota pine-knoll pack"]
fn ely_knolls_required_grass_produces_a_native_minnesota_encounter() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, SHORE, (4, 30));
    add_partner(&mut c, &r, false);
    for i in 0..256 {
        press(
            &mut c,
            if (i / 4) % 2 == 0 {
                GameButton::Up
            } else {
                GameButton::Down
            },
        );
        if let Some(battle) = c.snapshot().expect("native encounter").battle {
            assert!(matches!(
                battle.kind,
                crystal_runtime::RuntimeBattleKind::Wild { .. }
            ));
            assert!(battle.enemy_pokemon.species.int_id > 251);
            // The table has already been bound and audited by the native
            // regional publisher; this additionally proves encounters execute.
            super::rochester::flee(&mut c);
            assert!(c.snapshot().expect("native RUN returned").battle.is_none());
            save_view(&mut c, "wild-grass-return");
            return;
        }
    }
    panic!("required pine-knoll grass never produced an encounter");
}
