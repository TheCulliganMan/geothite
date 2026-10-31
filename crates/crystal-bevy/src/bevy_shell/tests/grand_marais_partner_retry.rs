//! Declared party fixtures; partner delivery and the lighthouse use native input.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, MAX_BOX_MONS, PcBox};

use super::super::mark_runtime_snapshot_dirty;
use super::interior_life::fixture;
use super::minnesota_fossils::talk;
use super::r10_gallery::continue_save;
use super::winona::{load, press};

const WELCOME: &str = "EVENT_MN_GRAND_MARAIS_STARTER";
const SIGNAL: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL";

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Grand Marais partner retry"]
fn marais_partner_controller_full_party_retains_offer_and_success_survives_continue() {
    let (root, runtime) = load();
    for (count, full_box) in [(1, false), (6, false), (6, true)] {
        let mut c = fixture(&root, &runtime, "MnGrandMaraisFieldStation", (5, 4));
        {
            let state = c.shell.shell.session_mut().state_mut();
            for slot in 0..count {
                state.storage.party.pokemon[slot] = Some(
                    runtime
                        .data()
                        .create_pokemon("POUCHID", 10, Dv::default())
                        .expect("declared Minnesota partner"),
                );
            }
            if full_box {
                while state.storage.pc_boxes.len() <= state.current_pc_box {
                    state
                        .storage
                        .pc_boxes
                        .push(PcBox::new(state.storage.pc_boxes.len()));
                }
                for _ in 0..MAX_BOX_MONS {
                    let partner = runtime
                        .data()
                        .create_pokemon("POUCHID", 10, Dv::default())
                        .expect("declared full selected box");
                    assert!(state.storage.pc_boxes[state.current_pc_box].add_pokemon(partner));
                }
            }
            state.sync_party_from_storage();
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let original_storage = c.shell.shell.session().state().storage.clone();
        // No state, party, inventory, flag, position or RNG writes below.
        talk(&mut c, GameButton::Up, &[]);
        let received = !full_box;
        let party_received = received && count < 6;
        let state = c.shell.shell.session().state();
        assert_eq!(
            state.flags.is_event_flag_set(WELCOME).expect("welcome"),
            received
        );
        assert_eq!(
            state.bag.quantity(&runtime.data().items["MACHINE_PART"]),
            u16::from(received)
        );
        assert_eq!(
            state.storage.party.filled_slots(),
            count + usize::from(party_received)
        );
        if received {
            let partner = if party_received {
                state.storage.party.pokemon[count].as_ref()
            } else {
                state.storage.pc_boxes[state.current_pc_box].pokemon[0].as_ref()
            }
            .expect("actual partner in party or selected PC");
            assert_eq!(partner.species.id, "BIRCHIT");
            assert_eq!(partner.level, 8);
            assert_eq!(partner.item.as_deref(), Some("BERRY"));
            assert!(state.pokedex.has_caught("BIRCHIT"));
        } else {
            assert_eq!(
                state.storage, original_storage,
                "failed gift changed storage"
            );
            assert!(!state.pokedex.has_caught("BIRCHIT"));
        }
        let bag = state.bag.clone();
        let storage = state.storage.clone();
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        talk(&mut c, GameButton::Up, &[]);
        let state = c.shell.shell.session().state();
        assert_eq!(state.bag, bag, "repeat must not duplicate supplies");
        assert_eq!(
            state.storage, storage,
            "Continue/repeat changed the partner"
        );
        assert_eq!(
            state
                .flags
                .is_event_flag_set(WELCOME)
                .expect("saved welcome"),
            received
        );
        assert_eq!(
            state.storage.party.filled_slots(),
            count + usize::from(party_received)
        );
        if received {
            assert!(state.pokedex.has_caught("BIRCHIT"));
        }
        if full_box {
            native_box_space_and_station_return(&mut c, &runtime);
            talk(&mut c, GameButton::Up, &[]);
            let state = c.shell.shell.session().state();
            assert!(
                state
                    .flags
                    .is_event_flag_set(WELCOME)
                    .expect("native retry claim")
            );
            assert!(state.pokedex.has_caught("BIRCHIT"));
            assert_eq!(state.storage.party.filled_slots(), 6);
            assert_eq!(
                state.storage.pc_boxes[state.current_pc_box].count,
                MAX_BOX_MONS
            );
            assert_eq!(
                state.storage.pc_boxes[state.current_pc_box]
                    .pokemon
                    .iter()
                    .flatten()
                    .filter(|pokemon| pokemon.species.id == "BIRCHIT")
                    .count(),
                1
            );
            assert_eq!(state.bag.quantity(&runtime.data().items["MACHINE_PART"]), 1);
            let granted = state.storage.clone();
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            talk(&mut c, GameButton::Up, &[]);
            assert_eq!(c.shell.shell.session().state().storage, granted);
        }
        if party_received {
            // Walk out of the station, across town and into the lighthouse.
            super::minnesota_gyms::walk_to(&mut c, &runtime, (2, 6));
            for _ in 0..16 {
                if c.snapshot().expect("station exit").overworld.map_name == "MnGrandMarais" {
                    break;
                }
                press(&mut c, GameButton::Down);
            }
            assert_eq!(
                c.snapshot()
                    .expect("actual town arrival")
                    .overworld
                    .map_name,
                "MnGrandMarais"
            );
            super::minnesota_gyms::walk_to(&mut c, &runtime, (15, 28));
            for _ in 0..16 {
                if c.snapshot().expect("lighthouse door").overworld.map_name
                    == "MnGrandMaraisLighthouse"
                {
                    break;
                }
                press(&mut c, GameButton::Up);
            }
            assert_eq!(
                c.snapshot()
                    .expect("actual lighthouse arrival")
                    .overworld
                    .map_name,
                "MnGrandMaraisLighthouse"
            );
            super::minnesota_gyms::walk_to(&mut c, &runtime, (5, 4));
            talk(&mut c, GameButton::Up, &[]);
            let state = c.shell.shell.session().state();
            assert!(
                state
                    .flags
                    .is_event_flag_set(SIGNAL)
                    .expect("actual repair")
            );
            assert_eq!(state.bag.quantity(&runtime.data().items["MACHINE_PART"]), 0);
            assert_eq!(state.bag.quantity(&runtime.data().items["SUPER_POTION"]), 1);
            let repaired_bag = state.bag.clone();
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            talk(&mut c, GameButton::Up, &[]);
            assert_eq!(c.shell.shell.session().state().bag, repaired_bag);
            assert!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(SIGNAL)
                    .expect("saved repair")
            );
        }
    }
}

fn native_box_space_and_station_return(
    c: &mut super::super::VisibleShellController,
    runtime: &crystal_runtime::CrystalRuntime,
) {
    const TOWN: &str = "MnGrandMarais";
    const CLINIC: &str = "MnGrandMaraisPokecenter";
    const STATION: &str = "MnGrandMaraisFieldStation";
    leave_to_town(c, runtime);
    enter_from_town(c, runtime, CLINIC);
    let module = &runtime.data().maps[CLINIC];
    let width = usize::from(module.attributes.width);
    let computers: Vec<_> = module
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| **block == 8)
        .map(|(index, _)| {
            (
                u16::try_from(index % width * 2 + 1).expect("PC column"),
                u16::try_from(index / width * 2 + 2).expect("PC approach row"),
            )
        })
        .collect();
    assert_eq!(computers.len(), 1, "one authentic native clinic PC");
    native_walk_with_dialogue(c, runtime, computers[0]);
    press(c, GameButton::Up);
    assert_eq!(
        c.shell
            .shell
            .current_overworld_interaction_checked()
            .expect("native PC interaction")
            .map(|interaction| interaction.script),
        Some("PCScript".into())
    );
    press(c, GameButton::A);
    pc_until(c, |shell| {
        shell.pc_hub_cursor.is_some()
            && shell.special_boundary.is_none()
            && shell.pc_notice.is_none()
            && shell.field_notice.is_none()
    });
    press(c, GameButton::A);
    pc_until(c, |shell| {
        shell.bill_pc_action_cursor.is_some()
            && shell.special_boundary.is_none()
            && shell.pc_notice.is_none()
            && shell.field_notice.is_none()
    });
    press(c, GameButton::A); // WITHDRAW opens the actual selected-box list.
    assert!(c.shell.storage_cursor.is_some() && !c.shell.bill_pc_deposit_open);
    press(c, GameButton::A);
    assert!(c.shell.bill_pc_pokemon_action_cursor.is_some());
    press(c, GameButton::Down);
    press(c, GameButton::Down);
    press(c, GameButton::A); // RELEASE one of the disposable Pouchid fixtures.
    assert!(c.shell.pending_pc_release.is_some());
    press(c, GameButton::A);
    pc_until(c, |shell| {
        shell.pc_release_sequence.is_none() && shell.pc_notice.is_none()
    });
    let state = c.shell.shell.session().state();
    assert_eq!(
        state.storage.pc_boxes[state.current_pc_box].count,
        MAX_BOX_MONS - 1
    );
    assert_eq!(state.storage.party.filled_slots(), 6);
    for _ in 0..16 {
        press(c, GameButton::B);
    }
    finish_room_dialogue(c);
    leave_to_town(c, runtime);
    assert_eq!(
        c.snapshot().expect("actual clinic exit").overworld.map_name,
        TOWN
    );
    enter_from_town(c, runtime, STATION);
    native_walk_with_dialogue(c, runtime, (5, 4));
}

fn pc_until(
    c: &mut super::super::VisibleShellController,
    ready: impl Fn(&super::super::BevyRuntimeShell) -> bool,
) {
    for _ in 0..256 {
        if ready(&c.shell) {
            return;
        }
        press(c, GameButton::A);
    }
    panic!(
        "native PC did not reach the expected input boundary: {:?}",
        c.snapshot().expect("PC diagnostic").ui
    );
}

fn finish_room_dialogue(c: &mut super::super::VisibleShellController) {
    for _ in 0..128 {
        c.wait_frames(64).expect("room text presentation");
        if let Some(name) = &c.shell.pending_name_choice {
            if name.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                if name.selected != 1 {
                    press(c, GameButton::Down);
                }
                press(c, GameButton::A);
            }
            continue;
        }
        if c.snapshot().expect("room dialogue").phase
            == crystal_runtime::RuntimeShellPhase::Overworld
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.special_boundary.is_none()
        {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("room dialogue did not release native input");
}

fn leave_to_town(
    c: &mut super::super::VisibleShellController,
    runtime: &crystal_runtime::CrystalRuntime,
) {
    let name = c.snapshot().expect("current room").overworld.map_name;
    let exit = runtime.data().maps[&name]
        .events
        .warps
        .iter()
        .find(|warp| {
            warp.target_map == "MnGrandMarais"
                || Some(warp.target_map_constant.as_str())
                    == runtime.data().maps["MnGrandMarais"]
                        .attributes
                        .map_constant
                        .as_deref()
        })
        .unwrap_or_else(|| {
            panic!(
                "actual town exit in {name}: {:?}",
                runtime.data().maps[&name].events.warps
            )
        });
    let approach = (exit.x, exit.y.checked_sub(1).expect("door approach y"));
    native_walk_with_dialogue(c, runtime, approach);
    finish_room_dialogue(c);
    for _ in 0..16 {
        if c.snapshot().expect("town arrival").overworld.map_name == "MnGrandMarais" {
            return;
        }
        press(c, GameButton::Down);
        finish_room_dialogue(c);
    }
    panic!("native room exit failed");
}

fn enter_from_town(
    c: &mut super::super::VisibleShellController,
    runtime: &crystal_runtime::CrystalRuntime,
    room: &str,
) {
    assert_eq!(
        c.snapshot().expect("door approach map").overworld.map_name,
        "MnGrandMarais"
    );
    let exit = runtime.data().maps[room]
        .events
        .warps
        .iter()
        .find(|warp| {
            warp.target_map == "MnGrandMarais"
                || Some(warp.target_map_constant.as_str())
                    == runtime.data().maps["MnGrandMarais"]
                        .attributes
                        .map_constant
                        .as_deref()
        })
        .expect("reciprocal room exit");
    let index = usize::try_from(exit.target_warp_id - 1).expect("positive reciprocal warp");
    let door = &runtime.data().maps["MnGrandMarais"].events.warps[index];
    native_walk_with_dialogue(
        c,
        runtime,
        (door.x, door.y.checked_add(1).expect("town approach y")),
    );
    for _ in 0..16 {
        if c.snapshot().expect("indoor arrival").overworld.map_name == room {
            finish_room_dialogue(c);
            return;
        }
        press(c, GameButton::Up);
    }
    panic!("native town door failed: {room}");
}

fn native_walk_with_dialogue(
    c: &mut super::super::VisibleShellController,
    runtime: &crystal_runtime::CrystalRuntime,
    target: (u16, u16),
) {
    let target = (
        i16::try_from(target.0).expect("walking column"),
        i16::try_from(target.1).expect("walking row"),
    );
    for _ in 0..512 {
        finish_room_dialogue(c);
        let before = c
            .snapshot()
            .expect("actual walking position")
            .overworld
            .tile;
        if (before.x, before.y) == target {
            return;
        }
        let button = *super::northstar_ice_lair::path(c, runtime, target)
            .expect("native walking path")
            .first()
            .expect("remaining walking step");
        for _ in 0..8 {
            press(c, button);
            if c.snapshot().expect("actual walking step").overworld.tile != before {
                break;
            }
        }
    }
    panic!(
        "native walk did not reach {target:?}: {:?}",
        c.snapshot().expect("walking diagnostic").overworld
    );
}
