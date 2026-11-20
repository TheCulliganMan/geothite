//! Native Winona research choices, party capacity and earlier gift receipts.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton::{A, B, Down, Left, Up};
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::brainerd_guide::capture;
use super::super::clinic_pc_until;
use super::super::minnesota_fossils::finish;
use super::super::northstar_ice_lair::choice;
use super::super::r10_gallery::continue_save;
use super::super::winona::{load, press, travel};
use super::{event, lab_fixture};
use crate::bevy_shell::{VisibleShellController, selected_vertical_menu};

const SPECIES: [&str; 3] = ["POUCHID", "KINDLIT", "BIRCHIT"];

pub(in crate::bevy_shell::tests) fn installed(runtime: &CrystalRuntime) -> bool {
    runtime.data().maps["MnWinonaFieldStation"]
        .scripts
        .contains_key("MnResearchChoiceAfterPouchid")
}

pub(super) fn start(c: &mut VisibleShellController) {
    press(c, Left);
    press(c, A);
}

fn menu_rows(c: &mut VisibleShellController) -> Option<Vec<String>> {
    let menu = c.snapshot().expect("research menu snapshot").ui.menu?;
    if !menu.window_open || c.shell.menu_cursor.is_none() {
        return None;
    }
    let options = &selected_vertical_menu(&menu, &c.shell.menu_cursor)
        .expect("live native choices")
        .options;
    Some(
        options
            .iter()
            .map(|row| row.trim().trim_start_matches('>').trim().to_owned())
            .collect(),
    )
}

pub(super) fn open(c: &mut VisibleShellController, offers: [&str; 2]) {
    start(c);
    for _ in 0..64 {
        c.wait_frames(96).expect("aide greeting");
        if let Some(rows) = menu_rows(c) {
            assert_eq!(rows, [offers[0], offers[1], "CANCEL"]);
            assert_eq!(
                c.shell
                    .menu_cursor
                    .as_ref()
                    .expect("open cursor")
                    .option_index,
                0
            );
            return;
        }
        assert!(
            c.snapshot()
                .expect("waiting for choices")
                .ui
                .pending_yes_no
                .is_none(),
            "both remaining partners must be visible before confirmation"
        );
        press(c, A);
    }
    panic!("the aide never opened the research menu");
}

pub(in crate::bevy_shell::tests) fn select(c: &mut VisibleShellController, row: Option<&str>) {
    let rows = menu_rows(c).expect("research menu is open");
    if let Some(row) = row {
        let index = rows
            .iter()
            .position(|label| label == row)
            .expect("requested species or CANCEL");
        let before = c
            .shell
            .menu_cursor
            .as_ref()
            .expect("current choice")
            .option_index;
        for _ in before..index {
            press(c, Down);
        }
        for _ in index..before {
            press(c, Up);
        }
        assert_eq!(
            c.shell
                .menu_cursor
                .as_ref()
                .expect("highlighted row")
                .option_index,
            index
        );
        press(c, A);
    } else {
        press(c, B);
    }
}

fn direct_page(c: &mut VisibleShellController, label: &str) {
    for _ in 0..64 {
        c.wait_frames(96).expect("aide text");
        let s = c.snapshot().expect("aide response");
        if let Some(text) = s.ui.text
            && text.label == label
        {
            return;
        }
        assert!(
            menu_rows(c).is_none(),
            "unexpected ordinary menu before {label}"
        );
        assert!(
            s.ui.pending_yes_no.is_none(),
            "unexpected alternative before {label}"
        );
        assert!(
            s.ui.text_window_open || c.has_pending_script_work(),
            "conversation ended before {label}"
        );
        press(c, A);
    }
    panic!("unreached research text {label}");
}

fn answer(c: &mut VisibleShellController, species: &str, yes: bool) {
    direct_page(c, &format!("MnEarlyPartner{species}Text"));
    choice(c, yes);
}

pub(super) fn confirm(c: &mut VisibleShellController, species: &str, yes: bool) {
    answer(c, species, yes);
    finish(c);
}

fn check_receipts(c: &VisibleShellController, expected: &[&str]) {
    for species in SPECIES {
        assert_eq!(
            event(c, &format!("EVENT_MN_{species}_RESEARCH_RECEIVED")),
            expected.contains(&species),
            "{species}"
        );
    }
}

fn count(c: &VisibleShellController) -> usize {
    c.shell.shell.session().state().storage.party.filled_slots()
}

fn deposit_and_return(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(c, runtime, (4, 10));
    travel(c, Down, "MnWinona", (18, 20));
    indoor_walk(c, runtime, (17, 8));
    travel(c, Up, "MnWinonaPokecenter1F", (4, 6));
    indoor_walk(c, runtime, (9, 2));
    press(c, Up);
    assert_eq!(
        c.shell
            .shell
            .current_overworld_interaction_checked()
            .expect("Winona PC")
            .expect("face PC")
            .script,
        "MnWClinicPC"
    );
    press(c, A);
    clinic_pc_until(c, |s| {
        s.pc_hub_cursor.is_some()
            && s.special_boundary.is_none()
            && s.pc_notice.is_none()
            && s.field_notice.is_none()
    });
    press(c, A);
    clinic_pc_until(c, |s| {
        s.bill_pc_action_cursor.is_some()
            && s.special_boundary.is_none()
            && s.pc_notice.is_none()
            && s.field_notice.is_none()
    });
    press(c, Down);
    press(c, A);
    assert!(c.shell.bill_pc_deposit_open && c.shell.storage_cursor.is_some());
    press(c, Down);
    press(c, A);
    assert!(c.shell.bill_pc_pokemon_action_cursor.is_some());
    press(c, A);
    clinic_pc_until(c, |s| {
        s.pc_transfer_sequence.is_none()
            && s.pc_notice.is_none()
            && s.bill_pc_pokemon_action_cursor.is_none()
    });
    assert_eq!(count(c), 5);
    let boxes = c
        .snapshot()
        .expect("actual Winona PC deposit")
        .storage
        .boxes;
    let mut boxed = boxes.iter().flat_map(|b| b.slots.iter());
    assert_eq!(
        boxed.next().expect("deposited partner").pokemon.species.id,
        "CROOKIT"
    );
    assert!(boxed.next().is_none());
    for _ in 0..16 {
        if !c.shell.pc_hub_session_open {
            break;
        }
        press(c, B);
    }
    assert!(!c.shell.pc_hub_session_open);
    finish(c);
    indoor_walk(c, runtime, (4, 6));
    travel(c, Down, "MnWinona", (17, 8));
    indoor_walk(c, runtime, (18, 20));
    travel(c, Up, "MnWinonaFieldStation", (4, 10));
    indoor_walk(c, runtime, (3, 9));
}

pub(super) fn review_visit(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    mut c: VisibleShellController,
    chosen: &str,
    party_count: usize,
) {
    let offers: [&str; 2] = SPECIES
        .into_iter()
        .filter(|s| *s != chosen)
        .collect::<Vec<_>>()
        .try_into()
        .expect("two alternatives for each history");
    let money = c.shell.shell.session().state().money;
    if party_count == 6 {
        open(&mut c, offers);
        select(&mut c, Some(offers[1]));
        answer(&mut c, offers[1], true);
        direct_page(&mut c, "MnEarlyPartnerFullText");
        finish(&mut c);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        check_receipts(&c, &[]);
        assert_eq!(count(&c), 6);
        assert!(
            c.snapshot()
                .expect("gift stayed with aide")
                .storage
                .boxes
                .iter()
                .all(|b| b.slots.is_empty())
        );
        if chosen == "KINDLIT" {
            deposit_and_return(&mut c, runtime);
            open(&mut c, offers);
            select(&mut c, Some(offers[1]));
            confirm(&mut c, offers[1], true);
            check_receipts(&c, &[offers[1]]);
            let snapshot = c.snapshot().expect("accepted after making room");
            assert_eq!(snapshot.party.slots.len(), 6);
            assert_eq!(snapshot.party.slots[5].pokemon.species.id, offers[1]);
        }
        return;
    }
    for button in [None, Some("CANCEL"), Some(offers[0])] {
        open(&mut c, offers);
        if button.is_none() {
            let frames = std::env::var_os("MN_RESEARCH_CHOICE_FRAMES").map(PathBuf::from);
            c = capture(
                c,
                frames.as_deref(),
                &format!("research-menu-after-{}.png", chosen.to_ascii_lowercase()),
            );
        }
        select(&mut c, button);
        if button == Some(offers[0]) {
            answer(&mut c, offers[0], false);
        }
        finish(&mut c);
        assert_eq!(count(&c), 1);
        check_receipts(&c, &[]);
    }
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    open(&mut c, offers);
    select(&mut c, Some(offers[1]));
    confirm(&mut c, offers[1], true);
    check_receipts(&c, &[offers[1]]);
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    start(&mut c);
    confirm(&mut c, offers[0], true);
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    start(&mut c);
    direct_page(&mut c, "MnEarlyPartnerDoneText");
    finish(&mut c);
    check_receipts(&c, &offers);
    let snapshot = c.snapshot().expect("three earned partners");
    assert_eq!(snapshot.party.slots.len(), 3);
    for (slot, species) in snapshot
        .party
        .slots
        .iter()
        .zip([chosen, offers[1], offers[0]])
    {
        assert_eq!(slot.pokemon.species.id, species);
        assert_eq!(slot.pokemon.level, 5);
    }
    let state = c.shell.shell.session().state();
    assert!(
        offers
            .iter()
            .all(|species| state.pokedex.has_caught(species))
    );
    assert_eq!(state.money, money);
    assert!(!event(&c, "EVENT_MN_LEAGUE_CHAMPION"));
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with early research access"]
fn early_research_starters_existing_claims_survive_return_without_party_member() {
    let (root, runtime) = load();
    for (claimed, available) in [("KINDLIT", "POUCHID"), ("POUCHID", "KINDLIT")] {
        let mut c = lab_fixture(&root, &runtime, "BIRCHIT", "TOTODILE", 1, true, &[claimed]);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        start(&mut c);
        confirm(&mut c, available, false);
        assert_eq!(count(&c), 1);
        check_receipts(&c, &[claimed]);
        start(&mut c);
        confirm(&mut c, available, true);
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        start(&mut c);
        direct_page(&mut c, "MnEarlyPartnerDoneText");
        finish(&mut c);
        check_receipts(&c, &[claimed, available]);
        let party = c
            .snapshot()
            .expect("existing claim is never duplicated")
            .party
            .slots;
        assert_eq!(party.len(), 2);
        assert_eq!(party[0].pokemon.species.id, "BIRCHIT");
        assert_eq!(party[1].pokemon.species.id, available);
    }
}
