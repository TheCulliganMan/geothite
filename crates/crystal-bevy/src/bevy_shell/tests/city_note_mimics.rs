//! Explicit note-side fixtures, then production inputs and same-pack Continue.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty, visible_battle_command_menu_entries,
};
use super::winona::{finish, press};

fn load() -> (AssetRoot, CrystalRuntime) {
    static PACK: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    PACK.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_CITY_NOTE_TEST_PACK").expect("published note candidate"),
        );
        let root = AssetRoot::new(path.parent().expect("parent"));
        let runtime = CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(path).expect("verified pack"),
        )
        .expect("runtime");
        (root, runtime)
    })
    .clone()
}

fn fixture(
    map: &str,
    native: Option<bool>,
    party: Option<u8>,
) -> (
    AssetRoot,
    CrystalRuntime,
    VisibleShellController,
    String,
    String,
) {
    let (root, runtime) = load();
    let data = runtime.data();
    let module = &data.maps[map];
    let object = module
        .objects
        .iter()
        .find(|o| {
            if !o.script.starts_with("MnCityNote") {
                return false;
            }
            let battle = module
                .scripted_wild_battles
                .iter()
                .find(|b| b.source_script.starts_with(&o.script));
            match native {
                Some(native) => {
                    battle.is_some_and(|b| b.source_script.ends_with("Native") == native)
                }
                None => battle.is_none(),
            }
        })
        .expect("eligible note")
        .clone();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: object.x as i16 - 1,
            tile_y: object.y as i16,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared note fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS")).expect("name");
    let state = c.shell.shell.session_mut().state_mut();
    if let Some(level) = party {
        let mut mon = data
            .create_pokemon(
                if level == 1 { "CROOKIT" } else { "BOREALYNX" },
                level,
                Dv::default(),
            )
            .expect("regional fixture partner");
        mon.item = Some("SMOKE_BALL".into());
        if level == 1 {
            mon.hp = 1;
            for learned in &mut mon.moves {
                learned.current_pp = 0;
            }
        } else {
            mon.moves = vec![LearnedMove {
                name: "HEADBUTT".into(),
                current_pp: data.moves["HEADBUTT"].pp,
                pp_ups: 0,
            }];
        }
        assert!(state.storage.party.add_pokemon(mon));
        state.sync_party_from_storage();
        state.last_spawn_map_constant = Some("MN_ROCHESTER_POKECENTER_1F".into());
        state
            .bag
            .add_item(&data.items["MASTER_BALL"], 1)
            .expect("declared capture item");
    }
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.wait_frames(64).expect("initial presentation");
    // No party/position/flag/RNG writes after the declared setup boundary.
    (
        root,
        runtime,
        c,
        object.object_identifier.expect("identifier"),
        object.script,
    )
}

fn visible(c: &mut VisibleShellController, id: &str) -> bool {
    c.snapshot()
        .expect("actors")
        .visible_objects
        .iter()
        .any(|o| o.object_identifier.as_deref() == Some(id))
}

fn save(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    name: &str,
) -> VisibleShellController {
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_CITY_NOTE_SAVE_DIR").expect("isolated evidence folder"),
    );
    std::fs::create_dir_all(&dir).expect("evidence directory");
    let path = dir.join(format!("{name}.crystalsave"));
    c.save(&path).expect("native save");
    VisibleShellController::load_save(root.clone(), runtime.clone(), path, None)
        .expect("same-pack Continue")
}

fn encounter(c: &mut VisibleShellController) {
    for _ in 0..128 {
        let s = c.snapshot().expect("encounter");
        if s.battle.is_some() && c.shell.battle_action_cursor.is_some() {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("note did not enter production wild battle");
}

fn talk(c: &mut VisibleShellController) {
    press(c, GameButton::Right);
    press(c, GameButton::A);
    c.wait_frames(128).expect("readable paper reveal");
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_TEST_PACK and CRYSTAL_CITY_NOTE_SAVE_DIR"]
fn city_notes_controller_catch_defeat_flee_reveal_reload_and_revisit() {
    for map in ["MnMinneapolis", "MnStPaul", "MnDuluth"] {
        for outcome in ["catch", "defeat", "flee"] {
            let (root, runtime, mut c, id, script) = fixture(map, Some(true), Some(80));
            assert!(visible(&mut c, &id));
            c = save(&mut c, &root, &runtime, &format!("{map}-{outcome}-before"));
            talk(&mut c);
            let s = c.snapshot().expect("reveal");
            assert_eq!(
                s.ui.text.as_ref().expect("native textbox").label,
                format!("{script}RevealText")
            );
            let flag = format!("EVENT_{id}_REVEALED");
            assert!(!super::northstar_ice_lair::event(&c, &flag));
            c = save(&mut c, &root, &runtime, &format!("{map}-{outcome}-reveal"));
            encounter(&mut c);
            let b = c
                .snapshot()
                .expect("wild battle")
                .battle
                .expect("native encounter");
            assert!(matches!(b.kind, RuntimeBattleKind::StaticWild { .. }));
            assert_eq!(
                (b.enemy_pokemon.species.id.as_str(), b.enemy_pokemon.level),
                ("JOTLET", 6)
            );
            assert!(super::northstar_ice_lair::event(&c, &flag));
            if outcome == "flee" {
                c = save(&mut c, &root, &runtime, &format!("{map}-during-battle"));
                assert!(c.snapshot().expect("continued battle").battle.is_some());
            }
            match outcome {
                "catch" => assert!(super::rochester_earned::try_recruit_with_ball(
                    &mut c,
                    &runtime,
                    "MASTER_BALL"
                )),
                "defeat" => assert!(super::rochester_earned::train_wild(&mut c, &runtime)),
                "flee" => super::rochester::flee(&mut c),
                _ => unreachable!(),
            }
            finish(&mut c);
            assert!(!visible(&mut c, &id));
            c = save(&mut c, &root, &runtime, &format!("{map}-{outcome}-after"));
            assert!(!visible(&mut c, &id));
            assert!(super::northstar_ice_lair::event(&c, &flag));
            press(&mut c, GameButton::Right);
            press(&mut c, GameButton::A);
            assert!(c.snapshot().expect("no duplicate").battle.is_none());
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_TEST_PACK and CRYSTAL_CITY_NOTE_SAVE_DIR"]
fn city_notes_controller_ordinary_paper_and_empty_party_never_start_battles() {
    for (native, party) in [(None, Some(80)), (Some(true), None)] {
        let (root, runtime, mut c, id, script) = fixture("MnMinneapolis", native, party);
        talk(&mut c);
        let label = c
            .snapshot()
            .expect("paper text")
            .ui
            .text
            .expect("readable text")
            .label;
        assert_eq!(
            label,
            format!(
                "{script}{}",
                if native.is_none() {
                    "Text"
                } else {
                    "NoPartyText"
                }
            )
        );
        finish(&mut c);
        assert!(visible(&mut c, &id));
        assert!(c.snapshot().expect("ordinary overworld").battle.is_none());
        c = save(
            &mut c,
            &root,
            &runtime,
            if party.is_some() {
                "ordinary-paper"
            } else {
                "empty-party"
            },
        );
        assert!(visible(&mut c, &id));
    }
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_TEST_PACK and CRYSTAL_CITY_NOTE_SAVE_DIR"]
fn city_notes_controller_blackout_consumes_the_mimic_without_quest_progress() {
    let (root, runtime, mut c, id, _) = fixture("MnMinneapolis", Some(true), Some(1));
    talk(&mut c);
    encounter(&mut c);
    let flag = format!("EVENT_{id}_REVEALED");
    assert!(super::northstar_ice_lair::event(&c, &flag));
    assert!(!super::rochester_earned::train_wild(&mut c, &runtime));
    finish(&mut c);
    assert_eq!(
        c.snapshot().expect("blackout recovery").overworld.map_name,
        "MnRochesterPokecenter1F"
    );
    c = save(&mut c, &root, &runtime, "blackout");
    assert!(super::northstar_ice_lair::event(&c, &flag));
    for story in ["EVENT_MN_DISPATCH_NOTE", "EVENT_MN_ICE_CAPTIVE_RESCUED"] {
        assert!(!super::northstar_ice_lair::event(&c, story));
    }
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_TEST_PACK and CRYSTAL_CITY_NOTE_SAVE_DIR"]
fn city_notes_graphical_continue_has_a_valid_battle_menu_before_the_first_tick() {
    let (root, runtime, mut c, _, _) = fixture("MnMinneapolis", Some(true), Some(80));
    talk(&mut c);
    encounter(&mut c);
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_CITY_NOTE_SAVE_DIR").expect("isolated evidence folder"),
    );
    std::fs::create_dir_all(&dir).expect("evidence directory");
    let path = dir.join("MnMinneapolis-native-battle-continue.crystalsave");
    c.save(&path).expect("native battle save");
    // Use the graphical initializer directly. The controller's load helper
    // settles presentation frames and would hide a missing initial chooser.
    let shell = initialize_bevy_runtime_shell(
        root,
        runtime,
        BevyShellStart::LoadSave { save_path: path },
        BevyShellConfig::default(),
    )
    .expect("graphical Continue");
    let snapshot = shell.shell.snapshot().expect("restored battle");
    let battle = snapshot.battle.as_ref().expect("continued encounter");
    assert_eq!(battle.enemy_pokemon.species.id, "JOTLET");
    assert_eq!(battle.enemy_pokemon.level, 6);
    let entries = visible_battle_command_menu_entries(&snapshot, &shell, battle)
        .expect("first native frame has a valid chooser");
    assert!(entries.iter().any(|entry| entry.contains("FIGHT")));
    assert!(entries.iter().any(|entry| entry.contains("RUN")));
}
