//! Paid field departures and native environment refusals, using production menus.

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::time::GameDate;
use crystal_core::world::map::TilePosition;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use crate::bevy_shell::{
    self, FieldPackAction, FieldPackPocket, PartyAction, PartyFieldMove, StartMenuOption,
};

use super::{
    LakeGrid, VisibleShellController, active, admit, continue_save, controller_fixture, global,
    launch, press, save, step, travel,
};

fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime, mut c) = controller_fixture();
    // Previously earned regional partners and permissions are initial setup.
    // The actual ranger, menu choices, travel and returns below use only input.
    for (name, move_id) in [("LOOBIT", "FLY"), ("DUSTIP", "DIG")] {
        let mut partner = runtime
            .data()
            .create_pokemon(name, 40, Dv::default())
            .expect("regional field partner");
        assert!(partner.species.tmhm_learnset.iter().any(|m| m == move_id));
        partner.moves = vec![LearnedMove {
            name: move_id.into(),
            current_pp: runtime.data().moves[move_id].pp,
            pp_ups: 0,
        }];
        assert!(
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(partner)
        );
    }
    let partner = runtime
        .data()
        .create_pokemon("DRIFTILE", 25, Dv::default())
        .expect("regional natural Teleport learner");
    assert!(
        partner.moves.iter().any(|m| m.name == "TELEPORT"),
        "Teleport must come from the actual level25 learnset"
    );
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.badges.johto[4] = true;
    state
        .flags
        .set_engine_flag("ENGINE_STORMBADGE", true)
        .expect("coherent Fly badge fixture");
    state
        .flags
        .set_engine_flag("ENGINE_FLYPOINT_MN_WINONA", true)
        .expect("previously visited Fly destination");
    // Entering the clinic records its outdoor source town, as native Teleport
    // does; the clinic's interior map constant is not itself a SpawnPoints key.
    state.last_spawn_map_constant = Some("MN_GRAND_MARAIS".into());
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["ESCAPE_ROPE"], 2)
            .expect("previously obtained ropes")
    );
    c.shell
        .shell
        .update_clock_from_datetime(GameDate::new(2000, 1, 1), 12, 0, 0)
        .expect("initial noon fixture");
    bevy_shell::mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, c)
}

fn select(
    c: &mut VisibleShellController,
    target: usize,
    cursor: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..32 {
        let current = cursor(c);
        if current == target {
            return;
        }
        press(
            c,
            if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native menu did not reach option {target}");
}

fn start(c: &mut VisibleShellController, option: StartMenuOption) {
    press(c, GameButton::Start);
    let s = c.snapshot().expect("actual Start menu");
    let target = bevy_shell::visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == option)
        .expect("native Start option");
    select(c, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    press(c, GameButton::A);
}

fn party_action(c: &mut VisibleShellController, action: PartyFieldMove) {
    start(c, StartMenuOption::Pokemon);
    let slot = match action {
        PartyFieldMove::Fly => 1,
        PartyFieldMove::Dig => 2,
        PartyFieldMove::Teleport => 3,
        _ => panic!("unexpected departure move"),
    };
    select(c, slot, |c| c.shell.party_cursor);
    press(c, GameButton::A);
    let s = c.snapshot().expect("actual partner actions");
    let target = bevy_shell::visible_party_actions(&s, &c.shell)
        .expect("native actions")
        .iter()
        .position(|a| *a == PartyAction::FieldMove(action))
        .expect("legally learned field move");
    select(c, target, |c| {
        c.shell
            .party_action_cursor
            .as_ref()
            .expect("party action cursor")
            .option_index
    });
    let s = c.snapshot().expect("visible four-partner submenu");
    let entries =
        bevy_shell::visible_party_menu_entries(&s, &c.shell).expect("native submenu rows");
    let selected = format!(">{}", bevy_shell::party_field_move_label(action));
    assert!(
        entries.iter().any(|entry| entry == &selected),
        "four partners must not hide {selected}: {entries:?}"
    );
}

fn rope(c: &mut VisibleShellController) {
    start(c, StartMenuOption::Pack);
    let target = c
        .snapshot()
        .expect("native Items pocket")
        .bag
        .items
        .iter()
        .filter(|i| i.quantity > 0)
        .position(|i| i.item_id == "ESCAPE_ROPE")
        .expect("carried rope");
    select(c, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("Pack cursor")
            .option_index
    });
    press(c, GameButton::A);
    let s = c.snapshot().expect("actual rope actions");
    let target = bevy_shell::visible_selected_pack_item_actions(
        &s,
        &c.shell,
        &FieldPackPocket::Items,
        false,
    )
    .expect("item actions")
    .iter()
    .position(|a| *a == FieldPackAction::Use)
    .expect("rope USE");
    select(c, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action cursor")
            .option_index
    });
    press(c, GameButton::A);
}

fn close(c: &mut VisibleShellController) {
    for _ in 0..96 {
        let s = c.snapshot().expect("field menu settling");
        if c.shell.field_notice.is_some() || s.ui.text_window_open {
            c.wait_frames(120).expect("notice prints");
            press(c, GameButton::A);
        } else if s.ui.menu.is_some()
            || c.shell.party_menu_open
            || c.shell.start_menu_cursor.is_some()
            || c.shell.field_pack_pocket.is_some()
        {
            press(c, GameButton::B);
        } else if c.has_pending_script_work() {
            c.wait_frames(16).expect("native script continuation");
        } else {
            return;
        }
    }
    panic!("native field menus did not release");
}

#[test]
#[ignore = "requires enabled current-parent CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_full_party_keeps_field_actions_and_both_cancel_paths_visible() {
    let (_, runtime, mut c) = fixture();
    for species in ["VOYAGEAR", "DOZEBEAR"] {
        let partner = runtime
            .data()
            .create_pokemon(species, 40, Dv::default())
            .expect("active regional full-party fixture");
        assert!(
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(partner)
        );
    }
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    bevy_shell::mark_runtime_snapshot_dirty(&mut c.shell);
    admit(&mut c);
    let before = active(&mut c);
    let party = c.snapshot().expect("six regional partners").party;
    assert_eq!(party.slots.len(), 6);
    party_action(&mut c, PartyFieldMove::Fly);
    save(&mut c, "full-party-field-actions");
    let s = c.snapshot().expect("full-party actions");
    let target = bevy_shell::visible_party_actions(&s, &c.shell)
        .expect("native actions")
        .iter()
        .position(|action| *action == PartyAction::Cancel)
        .expect("submenu Cancel");
    select(&mut c, target, |c| {
        c.shell
            .party_action_cursor
            .as_ref()
            .expect("submenu cursor")
            .option_index
    });
    let s = c.snapshot().expect("actual selected submenu Cancel");
    assert!(
        bevy_shell::visible_party_menu_entries(&s, &c.shell)
            .expect("visible submenu")
            .iter()
            .any(|entry| entry == ">CANCEL")
    );
    save(&mut c, "full-party-action-cancel");
    press(&mut c, GameButton::A);
    assert!(c.shell.party_menu_open);
    assert!(c.shell.party_action_cursor.is_none());
    select(&mut c, party.slots.len(), |c| c.shell.party_cursor);
    let s = c.snapshot().expect("actual selected roster Cancel");
    assert!(
        bevy_shell::visible_party_menu_entries(&s, &c.shell)
            .expect("visible roster")
            .iter()
            .any(|entry| entry == ">CANCEL")
    );
    save(&mut c, "full-party-roster-cancel");
    press(&mut c, GameButton::A);
    assert!(!c.shell.party_menu_open);
    close(&mut c);
    assert_eq!(
        active(&mut c),
        before,
        "neither menu spends Safari resources"
    );
    assert_eq!(c.snapshot().expect("unchanged full party").party, party);
    assert_eq!(c.snapshot().expect("original fee only").trainer.money, 1000);
    save(&mut c, "full-party-cancelled-field");
}

#[test]
#[ignore = "requires enabled current-parent CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_fly_and_teleport_leave_at_actual_destinations_from_land_and_water() {
    for (tag, action, water) in [
        ("fly-land", PartyFieldMove::Fly, false),
        ("fly-water", PartyFieldMove::Fly, true),
        ("teleport-land", PartyFieldMove::Teleport, false),
        ("teleport-water", PartyFieldMove::Teleport, true),
    ] {
        let (root, runtime, mut c) = fixture();
        let grid = LakeGrid::new(&runtime);
        admit(&mut c);
        travel(&mut c, &grid, (3, 90), false);
        if water {
            launch(&mut c, (3, 89));
        }
        let before = active(&mut c);
        let original_party = c.snapshot().expect("departure party").party;
        let original_bag = c.shell.shell.session().state().bag.clone();
        let source = c.snapshot().expect("actual paid source").overworld;
        assert_eq!(before.balls_remaining, 30);
        assert_eq!(before.steps_remaining, if water { 472 } else { 473 });
        save(&mut c, &format!("{tag}-field"));
        party_action(&mut c, action);
        save(&mut c, &format!("{tag}-party-choice"));
        press(&mut c, GameButton::A);
        let expected = if action == PartyFieldMove::Fly {
            assert!(c.shell.fly_cursor.is_some());
            if tag == "fly-land" {
                save(&mut c, "fly-cancel-menu");
                press(&mut c, GameButton::B);
                close(&mut c);
                assert_eq!(active(&mut c), before, "cancelling Fly retains the permit");
                assert_eq!(
                    c.snapshot().expect("cancelled Fly source").overworld.tile,
                    source.tile
                );
                save(&mut c, "fly-cancelled-field");
                party_action(&mut c, action);
                press(&mut c, GameButton::A);
            }
            for _ in 0..32 {
                let s = c.snapshot().expect("actual Fly choices");
                if bevy_shell::visible_fly_destination_entries(&s, &c.shell)
                    .expect("native Fly names")
                    .iter()
                    .any(|line| line == ">WINONA")
                {
                    break;
                }
                press(&mut c, GameButton::Down);
            }
            let s = c.snapshot().expect("selected Winona");
            let rows = bevy_shell::active_fly_destinations(&s, &c.shell.shell)
                .expect("visited destinations");
            let index = c
                .shell
                .fly_cursor
                .as_ref()
                .expect("Fly cursor")
                .option_index;
            assert_eq!(rows[index].flypoint_flag, "ENGINE_FLYPOINT_MN_WINONA");
            assert!(
                s.ui.menu
                    .as_ref()
                    .expect("projected Fly chooser")
                    .layout
                    .vertical_menus
                    .last()
                    .expect("Fly menu rows")
                    .options
                    .iter()
                    .any(|entry| entry == ">WINONA"),
                "Fly destinations must own the visible menu above the party submenu"
            );
            save(&mut c, &format!("{tag}-destination-choice"));
            let spawn_id = rows[index].destination_spawn_identifier;
            assert_eq!(
                active(&mut c),
                before,
                "destination choices cost no permit resources"
            );
            press(&mut c, GameButton::A);
            runtime
                .data()
                .runtime_spawn_point(spawn_id)
                .expect("chosen native Fly spawn")
        } else {
            runtime
                .data()
                .runtime_spawn_point_for_map_constant("MN_GRAND_MARAIS")
                .expect("actual last clinic's outdoor town spawn")
        };
        if let Some(pending) = c
            .shell
            .shell
            .session()
            .state()
            .script_runtime
            .pending_field_travel
            .as_ref()
        {
            assert_eq!(pending.destination_map, expected.map_name);
            assert_eq!(
                pending.destination_tile,
                TilePosition::new(expected.tile_x, expected.tile_y)
            );
            assert_eq!(
                active(&mut c),
                before,
                "permit persists until the actual destination commits"
            );
            save(&mut c, &format!("{tag}-prepared"));
            press(&mut c, GameButton::Right);
            assert_eq!(
                c.snapshot().expect("departure owns input").overworld.tile,
                source.tile
            );
        } else {
            // Timed Teleport can settle during the controller's normal input
            // boundary; no pending travel must mean the real arrival committed.
            let arrived = c.snapshot().expect("already committed native departure");
            assert_eq!(
                arrived.overworld.map_name, expected.map_name,
                "{tag}: no pending travel must mean actual arrival"
            );
            assert_eq!(
                arrived.overworld.tile,
                TilePosition::new(expected.tile_x, expected.tile_y)
            );
            assert!(arrived.safari.is_none());
        }
        for _ in 0..512 {
            let s = c.snapshot().expect("native outward presentation");
            if s.overworld.map_name == expected.map_name {
                break;
            }
            if s.ui.text_window_open {
                press(&mut c, GameButton::A);
            } else {
                c.wait_frames(16).expect("departure presentation");
            }
        }
        close(&mut c);
        let landed = c.snapshot().expect("actual field departure arrival");
        assert_eq!(landed.overworld.map_name, expected.map_name);
        assert_eq!(
            landed.overworld.tile,
            TilePosition::new(expected.tile_x, expected.tile_y)
        );
        assert_eq!(landed.overworld.mode, MovementMode::Normal);
        assert!(
            landed.safari.is_none(),
            "no live permit at the selected destination"
        );
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .script_runtime
                .pending_field_travel
                .is_none()
        );
        assert_eq!(landed.party, original_party);
        assert_eq!(c.shell.shell.session().state().bag, original_bag);
        assert_eq!(landed.trainer.money, 1000);
        save(&mut c, &format!("{tag}-arrived"));
        let expected_map = expected.map_name.clone();
        let expected_tile = TilePosition::new(expected.tile_x, expected.tile_y);
        c = continue_save(root, runtime, &mut c);
        let continued = c.snapshot().expect("Continue at the chosen destination");
        assert_eq!(continued.overworld.map_name, expected_map);
        assert_eq!(continued.overworld.tile, expected_tile);
        assert!(continued.safari.is_none());
    }
}

fn refusal(c: &mut VisibleShellController, tag: &str, action: Option<PartyFieldMove>) {
    let before = active(c);
    let original = c.snapshot().expect("before native refusal");
    let bag = c.shell.shell.session().state().bag.clone();
    if let Some(action) = action {
        party_action(c, action);
        press(c, GameButton::A);
        if action == PartyFieldMove::Fly {
            assert!(
                c.shell.fly_cursor.is_some(),
                "native Fly chooses a destination before validating departure"
            );
            for _ in 0..32 {
                let s = c.snapshot().expect("indoor Fly destination menu");
                if bevy_shell::visible_fly_destination_entries(&s, &c.shell)
                    .expect("Fly names")
                    .iter()
                    .any(|line| line == ">WINONA")
                {
                    break;
                }
                press(c, GameButton::Down);
            }
            let s = c.snapshot().expect("selected refused Fly destination");
            let rows = bevy_shell::active_fly_destinations(&s, &c.shell.shell)
                .expect("actual Fly destinations");
            let index = c
                .shell
                .fly_cursor
                .as_ref()
                .expect("Fly selection")
                .option_index;
            assert_eq!(rows[index].flypoint_flag, "ENGINE_FLYPOINT_MN_WINONA");
            press(c, GameButton::A);
        }
    } else {
        rope(c);
    }
    c.wait_frames(120).expect("native refusal text");
    let notice = c
        .shell
        .field_notice
        .as_deref()
        .unwrap_or_else(|| panic!("native environment refusal missing for {tag}"));
    assert!(
        notice.to_ascii_uppercase().contains("CAN'T"),
        "{tag}: {notice}"
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .script_runtime
            .pending_field_travel
            .is_none()
    );
    save(c, &format!("{tag}-notice"));
    press(c, GameButton::Right);
    assert_eq!(
        c.snapshot().expect("refusal owns movement").overworld.tile,
        original.overworld.tile
    );
    close(c);
    assert_eq!(
        active(c),
        before,
        "environment refusal preserves the expedition"
    );
    let after = c.snapshot().expect("refusal released");
    assert_eq!(after.overworld.map_name, original.overworld.map_name);
    assert_eq!(after.overworld.tile, original.overworld.tile);
    assert_eq!(after.party, original.party);
    assert_eq!(c.shell.shell.session().state().bag, bag);
    assert_eq!(after.trainer.money, 1000);
}

#[test]
#[ignore = "requires enabled current-parent CRYSTAL_INTERIOR_PACK Safari review"]
fn safari_paid_route_and_real_cabin_refuse_invalid_escapes_without_spending_resources() {
    let (root, runtime, mut c) = fixture();
    let grid = LakeGrid::new(&runtime);
    admit(&mut c);
    travel(&mut c, &grid, (3, 90), false);
    refusal(&mut c, "route-dig", Some(PartyFieldMove::Dig));
    refusal(&mut c, "route-rope", None);
    launch(&mut c, (3, 89));
    travel(&mut c, &grid, (55, 58), true);
    step(&mut c, (56, 58));
    travel(&mut c, &grid, (71, 56), false);
    press(&mut c, GameButton::Up);
    assert_eq!(
        c.snapshot()
            .expect("actual cabin entrance")
            .overworld
            .map_name,
        "MnBoundaryRootBeerCabin"
    );
    c = continue_save(root, runtime.clone(), &mut c);
    for (tag, action) in [
        ("cabin-dig", Some(PartyFieldMove::Dig)),
        ("cabin-rope", None),
        ("cabin-fly", Some(PartyFieldMove::Fly)),
        ("cabin-teleport", Some(PartyFieldMove::Teleport)),
    ] {
        refusal(&mut c, tag, action);
    }
    press(&mut c, GameButton::Down);
    if c.snapshot()
        .expect("native cabin carpet")
        .overworld
        .map_name
        == "MnBoundaryRootBeerCabin"
    {
        press(&mut c, GameButton::Down);
    }
    assert_eq!(global(&mut c), (71, 56), "real cabin return");
    travel(&mut c, &grid, (56, 58), false);
    launch(&mut c, (55, 58));
    travel(&mut c, &grid, (3, 89), true);
    step(&mut c, (3, 90));
    travel(&mut c, &grid, (10, 104), false);
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("real retirement offer")
            .safari_prompt
            .is_some()
    );
    assert_eq!(active(&mut c).balls_remaining, 30);
    save(&mut c, "refused-escapes-return-offer");
    press(&mut c, GameButton::A);
    assert!(c.snapshot().expect("real retirement").safari.is_none());
    assert_eq!(
        c.snapshot().expect("real hub").overworld.map_name,
        "MnBoundaryWaters"
    );
    save(&mut c, "refused-escapes-retired");
}
