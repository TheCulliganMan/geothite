//! Explicit city/party fixtures; doors, services and bridge walks use input.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::{ScriptFlagCommand, apply_script_flag_mutation};
use crystal_core::world::movement::MovementMode;

use super::super::VisibleShellController;
use super::super::mark_runtime_snapshot_dirty;
use super::interior_life::{fixture, load, target_name, travel, walk};

const CITY: &str = "MnStPaul";

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production city input");
    c.wait_frames(16).expect("settle city input");
}

fn surf_step(c: &mut VisibleShellController, button: GameButton, map: &str, target: (i16, i16)) {
    for _ in 0..8 {
        press(c, button);
        let s = c.snapshot().expect("native river step");
        if s.overworld.map_name == map && (s.overworld.tile.x, s.overworld.tile.y) == target {
            assert_eq!(s.overworld.mode, MovementMode::Surf);
            return;
        }
    }
    panic!("native river step did not reach {map} {target:?}");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the repaired connected river"]
fn st_paul_refinement_surf_passes_both_bridges_and_river_seams_and_continues() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CITY, (9, 51));
    let mut partner = r
        .data()
        .create_pokemon("MEW", 80, Dv::default())
        .expect("legal Surf partner");
    partner.moves = vec![LearnedMove {
        name: "SURF".into(),
        current_pp: r.data().moves["SURF"].pp,
        pp_ups: 0,
    }];
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.repel_steps_remaining = 200; // Explicit active Repel fixture.
    state.active_repel_item = Some("MAX_REPEL".into());
    apply_script_flag_mutation(
        state,
        ScriptFlagCommand {
            command: "setflag".into(),
            flag_id: "ENGINE_FOGBADGE".into(),
            source_script: "StPaulRiverInitialFixture".into(),
            command_index: 0,
        },
    )
    .expect("initial earned Surf badge fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    // The fixture ends here: mount, bridge passes, seams and Continue use input.
    press(&mut c, GameButton::Right);
    for _ in 0..128 {
        if c.snapshot().expect("native Surf mount").overworld.mode == MovementMode::Surf {
            break;
        }
        press(&mut c, GameButton::A);
    }
    let s = c.snapshot().expect("mounted river");
    assert_eq!(s.overworld.mode, MovementMode::Surf);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), (10, 51));
    surf_step(&mut c, GameButton::Down, CITY, (10, 52));
    for x in 11..=79 {
        surf_step(&mut c, GameButton::Right, CITY, (x, 52));
        if x == 26 {
            c = super::r10_gallery::continue_save(root.clone(), r.clone(), &mut c);
            let s = c.snapshot().expect("Continue under actual bridge");
            assert_eq!(s.overworld.mode, MovementMode::Surf);
            assert_eq!((s.overworld.tile.x, s.overworld.tile.y), (26, 52));
        }
    }
    surf_step(
        &mut c,
        GameButton::Right,
        "MnMississippiLowerWoodsC00R01",
        (0, 24),
    );
    surf_step(&mut c, GameButton::Left, CITY, (79, 52));
    for y in 53..=54 {
        surf_step(&mut c, GameButton::Down, CITY, (79, y));
    }
    for x in (75..=78).rev() {
        surf_step(&mut c, GameButton::Left, CITY, (x, 54));
    }
    for y in 55..=67 {
        surf_step(&mut c, GameButton::Down, CITY, (75, y));
    }
    surf_step(
        &mut c,
        GameButton::Down,
        "MnMississippiConfluenceC03R00",
        (3, 0),
    );
    surf_step(&mut c, GameButton::Up, CITY, (75, 67));
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the repaired St Paul exterior"]
fn st_paul_refinement_all_ordinary_facades_and_both_bridges_return_and_continue() {
    let (root, r, _) = load();
    let mut c = fixture(&root, &r, CITY, (6, 48));
    // These optional opponents are already beaten in this geometry fixture.
    // No session position or progress is rewritten after the first input.
    for flag in ["EVENT_MN_SPX_TRAINER_1", "EVENT_MN_SPX_TRAINER_8"] {
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(flag, true)
            .expect("initial optional trainer fixture");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Walk the original northern and southern landings through both real decks.
    for point in [(26, 48), (26, 56), (56, 56), (56, 48), (26, 48)] {
        walk(&mut c, &r, point);
    }
    let doors = r.data().maps[CITY]
        .events
        .warps
        .iter()
        .filter(|warp| warp.index % 2 == 1 && target_name(&r, &warp.target_map) != "MnStPaulGym")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        doors.len(),
        39,
        "every ordinary facade, including the depot"
    );
    for door in doors {
        let destination = target_name(&r, &door.target_map);
        walk(&mut c, &r, (door.x as i16, door.y as i16));
        assert_eq!(
            c.snapshot().expect("real facade entry").overworld.map_name,
            destination
        );
        travel(&mut c, &r, CITY);
        let back = &r.data().maps[CITY].events.warps[usize::from(door.index)];
        let s = c.snapshot().expect("paired exterior return");
        assert_eq!(
            (s.overworld.tile.x, s.overworld.tile.y),
            (back.x as i16, back.y as i16)
        );
    }
    let before = c.snapshot().expect("city before Continue").overworld;
    c = super::r10_gallery::continue_save(root.clone(), r.clone(), &mut c);
    let after = c.snapshot().expect("city Continue").overworld;
    assert_eq!(after.map_name, before.map_name);
    assert_eq!(after.tile, before.tile);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with both St Paul clinics"]
fn st_paul_refinement_both_clinics_keep_free_care_and_native_pc_input() {
    let (root, r, _) = load();
    for name in ["MnStPaulPokecenter1F", "MnStPaulEastAidClinic"] {
        let mut c = fixture(&root, &r, name, (3, 5));
        let mut partner = r
            .data()
            .create_pokemon("CHIKORITA", 20, Dv::default())
            .expect("legal partner");
        partner.hp = 1;
        partner.status = Some("POISON".into());
        for move_ in &mut partner.moves {
            move_.current_pp = 0;
        }
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
        state.money = 0;
        mark_runtime_snapshot_dirty(&mut c.shell);
        walk(&mut c, &r, (3, 3));
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        assert!(c.snapshot().expect("clinic dialogue").ui.text_window_open);
        press(&mut c, GameButton::Right);
        assert_eq!(c.snapshot().expect("care owns input").overworld.tile.x, 3);
        for _ in 0..200 {
            let s = c.snapshot().expect("care presentation");
            if !s.ui.text_window_open
                && c.shell.special_boundary.is_none()
                && !c.has_pending_script_work()
            {
                break;
            }
            press(&mut c, GameButton::A);
        }
        let healed = c.snapshot().expect("native free clinic care");
        let partner = &healed.party.slots[0].pokemon;
        assert_eq!(partner.hp, partner.max_hp);
        assert!(partner.status.is_none());
        assert!(
            partner
                .moves
                .iter()
                .all(|m| m.current_pp == r.data().moves[&m.name].pp)
        );
        assert_eq!(c.shell.shell.session().state().money, 0);
        walk(&mut c, &r, (9, 2));
        press(&mut c, GameButton::Up);
        for _ in 0..96 {
            press(&mut c, GameButton::A);
            if c.shell.pc_hub_cursor.is_some() {
                break;
            }
        }
        assert!(
            c.shell.pc_hub_cursor.is_some(),
            "production PC opens in {name}"
        );
        let before = c.snapshot().expect("PC position").overworld.tile;
        press(&mut c, GameButton::Left);
        assert_eq!(c.snapshot().expect("PC owns input").overworld.tile, before);
        for _ in 0..96 {
            press(&mut c, GameButton::B);
            if c.shell.pc_hub_cursor.is_none()
                && c.shell.special_boundary.is_none()
                && !c.has_pending_script_work()
                && !c.snapshot().expect("PC closure").ui.text_window_open
            {
                break;
            }
        }
        assert!(c.shell.pc_hub_cursor.is_none() && !c.has_pending_script_work());
        travel(&mut c, &r, CITY);
    }
}
