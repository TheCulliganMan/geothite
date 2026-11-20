//! The pack and party are seeded once. All later changes use the station, keeper and Pack.
use std::path::Path;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton::{self, A, B, Down, Up};
use crystal_core::models::{BALL_POCKET_CAPACITY, Dv, ITEM_POCKET_CAPACITY, MAX_ITEM_STACK};
use crystal_runtime::CrystalRuntime;

use super::super::super::{FieldPackPocket, VisibleShellController};
use super::super::brainerd_guide::capture;
use super::super::northstar_ice_lair::{choice, event, page};
use super::super::northstar_progression_rewards::native_toss_first_pocket_item;
use super::{
    SIGNAL, WELCOME, continue_save, enter_from_town, finish_room_dialogue, fixture, leave_to_town,
    load, mark_runtime_snapshot_dirty, native_walk_with_dialogue, press, talk,
};

const STATION: &str = "MnGrandMaraisFieldStation";
const LIGHTHOUSE: &str = "MnGrandMaraisLighthouse";
const OPEN: &str = "EVENT_MN_MARAIS_SUPPLIES_OPEN";
const BALLS: &str = "EVENT_MN_MARAIS_BALLS_RECEIVED";
const POTIONS: &str = "EVENT_MN_MARAIS_POTIONS_RECEIVED";
const MEDICINE: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL_REWARD_PENDING";

fn quantity(c: &VisibleShellController, item: &str) -> u16 {
    let shell = &c.shell.shell;
    shell
        .session()
        .state()
        .bag
        .quantity(&shell.runtime().data().items[item])
}

fn before_visit(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    full_balls: bool,
    full_items: bool,
) -> VisibleShellController {
    let mut c = fixture(root, runtime, STATION, (5, 4));
    let state = c.shell.shell.session_mut().state_mut();
    let lead = runtime
        .data()
        .create_pokemon("POUCHID", 10, Dv::default())
        .expect("test partner");
    assert!(state.storage.party.add_pokemon(lead));
    state.sync_party_from_storage();
    for (full, first, second, filler, capacity) in [
        (
            full_balls,
            "GREAT_BALL",
            "LURE_BALL",
            "FAST_BALL",
            BALL_POCKET_CAPACITY,
        ),
        (
            full_items,
            "ANTIDOTE",
            "AWAKENING",
            "HARD_STONE",
            ITEM_POCKET_CAPACITY,
        ),
    ] {
        if full {
            // Single-item slots can be freed through TOSS. Full stacks occupy the other slots.
            for item in [first, second] {
                assert!(
                    state
                        .bag
                        .add_item(&runtime.data().items[item], 1)
                        .expect("tossable fixture")
                );
            }
            for _ in 2..capacity {
                assert!(
                    state
                        .bag
                        .add_item(&runtime.data().items[filler], MAX_ITEM_STACK)
                        .expect("full pocket")
                );
            }
        }
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c
}

fn speak(c: &mut VisibleShellController) {
    press(c, Up);
    press(c, A);
}

fn move_between(c: &mut VisibleShellController, runtime: &CrystalRuntime, to: &str) {
    leave_to_town(c, runtime);
    enter_from_town(c, runtime, to);
    native_walk_with_dialogue(c, runtime, (5, 4));
}

fn invitation(
    mut c: VisibleShellController,
    repaired: bool,
    yes: bool,
    frames: Option<&Path>,
) -> VisibleShellController {
    let label = if repaired {
        "MnMaraisGiftAfterRepairText"
    } else {
        "MnGrandMaraisWelcomeText"
    };
    speak(&mut c);
    page(&mut c, label);
    let final_page =
        super::super::northstar_dialogue::canonical_pages(c.shell.shell.runtime(), "", label)
            .pop()
            .expect("last offer page");
    for _ in 0..20 {
        c.wait_frames(128).expect("printed offer");
        let s = c.snapshot().expect("offer");
        if s.ui.text.as_ref().and_then(|text| text.asm_text.as_deref()) == Some(final_page.as_str())
            && s.ui.pending_yes_no.is_some()
            && c.shell.yes_no_cursor.is_some()
        {
            let name = if repaired {
                "birchit-after-lighthouse.png"
            } else {
                "birchit-before-lighthouse.png"
            };
            c = capture(c, frames, name);
            choice(&mut c, yes);
            if !yes {
                page(&mut c, "MnMaraisPartnerDeclinedText");
                finish_room_dialogue(&mut c);
            }
            return c;
        }
        press(&mut c, A);
    }
    panic!("missing printed Birchit question and Yes/No cursor");
}

fn supplies(c: &VisibleShellController, balls: bool, potions: bool) {
    assert!(event(c, WELCOME));
    assert_eq!(event(c, OPEN), !balls || !potions);
    assert_eq!(event(c, BALLS), balls);
    assert_eq!(event(c, POTIONS), potions);
    assert_eq!(quantity(c, "POKE_BALL"), if balls { 5 } else { 0 });
    assert_eq!(quantity(c, "POTION"), if potions { 3 } else { 0 });
    let state = c.shell.shell.session().state();
    let party = &state.storage.party;
    assert_eq!(party.filled_slots(), 2);
    let partner = party.pokemon[1].as_ref().expect("joined the party");
    assert_eq!(partner.species.id, "BIRCHIT");
    assert_eq!(partner.level, 8);
    assert_eq!(partner.item.as_deref(), Some("BERRY"));
    assert!(state.pokedex.has_caught("BIRCHIT"));
}

fn finish_claim(
    mut c: VisibleShellController,
    total: (bool, bool),
    newly_received: (bool, bool),
    dir: Option<&Path>,
) -> VisibleShellController {
    let pending = match total {
        (false, false) => Some(("MnMaraisBothWaitingText", "birchit-both-waiting.png")),
        (false, true) => Some(("MnMaraisBallsWaitingText", "birchit-balls-waiting.png")),
        (true, false) => Some(("MnMaraisPotionsWaitingText", "birchit-potions-waiting.png")),
        (true, true) => None,
    };
    let mut seen = (false, false);
    let mut held = false;
    for _ in 0..96 {
        c.wait_frames(64).expect("gift and supply pages");
        let s = c.snapshot().expect("actual supply receipt");
        if let Some(text) = &s.ui.text {
            match text.label.as_str() {
                "MnMaraisBallsReceivedText" => {
                    assert_eq!(quantity(&c, "POKE_BALL"), 5);
                    seen.0 = true;
                }
                "MnMaraisPotionsReceivedText" => {
                    assert_eq!(quantity(&c, "POTION"), 3);
                    seen.1 = true;
                }
                "MnMaraisBallsWaitingText"
                | "MnMaraisPotionsWaitingText"
                | "MnMaraisBothWaitingText" => {
                    let (label, name) = pending.expect("nothing should be put aside");
                    assert_eq!(text.label, label);
                    if !held {
                        c = capture(c, dir, name);
                        held = true;
                    }
                }
                _ => {}
            }
        }
        if s.phase == crystal_runtime::RuntimeShellPhase::Overworld
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.special_boundary.is_none()
            && c.shell.pending_name_choice.is_none()
        {
            assert_eq!(
                seen, newly_received,
                "only items that entered the Pack get a receipt"
            );
            assert_eq!(held, pending.is_some(), "missing the waiting message");
            supplies(&c, total.0, total.1);
            return c;
        }
        if let Some(nick) = &c.shell.pending_name_choice
            && nick.nickname_pages.len() <= 1
            && nick.selected != 1
        {
            press(&mut c, Down);
        }
        press(&mut c, A);
    }
    panic!("the station did not release the player after the supplies");
}

fn free_one(c: &mut VisibleShellController, pocket: FieldPackPocket) {
    native_toss_first_pocket_item(c, &pocket);
    for _ in 0..8 {
        press(c, B);
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Birchit invitation"]
fn marais_partner_controller_decisions_supply_capacity_and_lightkeeper() {
    let (root, runtime) = load();
    let frames = std::env::var_os("MN_MARAIS_PARTNER_FRAMES").map(std::path::PathBuf::from);
    for (full_balls, full_items) in [(false, false), (true, false), (false, true), (true, true)] {
        let both = full_balls && full_items;
        let mut c = before_visit(&root, &runtime, full_balls, full_items);
        talk(&mut c, Up, &[]);
        assert_eq!(quantity(&c, "MACHINE_PART"), 1);
        let before = c.shell.shell.session().state().storage.clone();
        let bag = c.shell.shell.session().state().bag.clone();
        c = invitation(c, false, false, frames.as_deref());
        assert_eq!(c.shell.shell.session().state().storage, before);
        assert_eq!(c.shell.shell.session().state().bag, bag);
        assert!(!event(&c, WELCOME) && !event(&c, OPEN));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        if both {
            move_between(&mut c, &runtime, LIGHTHOUSE);
            talk(&mut c, Up, &[]);
            assert!(event(&c, SIGNAL) && event(&c, MEDICINE));
            assert_eq!(quantity(&c, "MACHINE_PART"), 0);
            assert_eq!(quantity(&c, "SUPER_POTION"), 0);
            move_between(&mut c, &runtime, STATION);
            c = invitation(c, true, false, frames.as_deref());
            assert_eq!(c.shell.shell.session().state().storage, before);
            assert!(!event(&c, WELCOME) && !event(&c, OPEN));
        }

        c = invitation(c, both, true, frames.as_deref());
        c = finish_claim(
            c,
            (!full_balls, !full_items),
            (!full_balls, !full_items),
            frames.as_deref(),
        );
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        speak(&mut c);
        c = finish_claim(c, (!full_balls, !full_items), (false, false), None);

        if full_balls {
            free_one(&mut c, FieldPackPocket::Balls);
            speak(&mut c);
            c = finish_claim(c, (true, !full_items), (true, false), None);
        }
        if full_items {
            free_one(&mut c, FieldPackPocket::Items);
            speak(&mut c);
            c = finish_claim(c, (true, true), (false, true), None);
        }
        if both {
            assert!(event(&c, MEDICINE));
            free_one(&mut c, FieldPackPocket::Items);
            move_between(&mut c, &runtime, LIGHTHOUSE);
            talk(&mut c, Up, &[]);
            assert!(!event(&c, MEDICINE) && event(&c, SIGNAL));
            assert_eq!(quantity(&c, "SUPER_POTION"), 1);
            talk(&mut c, Up, &[]);
            assert_eq!(quantity(&c, "SUPER_POTION"), 1);
            move_between(&mut c, &runtime, STATION);
        }
        let delivered = c.shell.shell.session().state().storage.clone();
        let bag = c.shell.shell.session().state().bag.clone();
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        talk(&mut c, Up, &[]);
        supplies(&c, true, true);
        assert_eq!(c.shell.shell.session().state().bag, bag);
        assert_eq!(c.shell.shell.session().state().storage, delivered);
    }
}

fn clear_door(c: &mut VisibleShellController) {
    let s = c.snapshot().expect("station door state");
    assert!(
        s.ui.pending_yes_no.is_none()
            && s.ui.text.is_none()
            && !s.ui.text_window_open
            && c.shell.field_notice.is_none(),
        "the doorway must release travel: {} {:?} {:?}",
        s.overworld.map_name,
        s.overworld.tile,
        s.ui.text
    );
}

fn cross_door(c: &mut VisibleShellController, direction: GameButton, destination: &str) {
    for _ in 0..16 {
        clear_door(c);
        if c.snapshot().expect("door travel").overworld.map_name == destination {
            return;
        }
        press(c, direction);
    }
    panic!("walking through the station door did not reach {destination}");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with separate Grand Marais exit triggers"]
fn marais_partner_controller_both_exit_lanes_leave_the_offer_at_tamaracks_desk() {
    let (root, runtime) = load();
    let reference = runtime.data().maps["MnWinonaFieldStation"]
        .objects
        .iter()
        .find(|actor| actor.script == "MN_W_TAMARACK")
        .expect("Tamarack in Winona");
    assert_eq!(reference.sprite, "SPRITE_MN_PROF_TAMARACK");
    let station = &runtime.data().maps[STATION];
    let professor = station
        .objects
        .iter()
        .find(|actor| actor.script == "MnGrandMaraisWelcome")
        .expect("professor at the station");
    assert_eq!((professor.x, professor.y), (5, 3));
    assert_eq!(professor.sprite, reference.sprite);
    assert!(professor.sprite_has_facings);
    let town = "MnGrandMarais";
    let outside = runtime.data().maps[town]
        .events
        .warps
        .iter()
        .find(|w| {
            w.target_map_constant
                == station
                    .attributes
                    .map_constant
                    .as_deref()
                    .expect("station map constant")
        })
        .expect("station front door");
    let frames = std::env::var_os("MN_MARAIS_PARTNER_FRAMES").map(std::path::PathBuf::from);
    for (lane, repaired) in [(2, false), (3, false), (2, true), (3, true)] {
        let mut c = before_visit(&root, &runtime, false, false);
        // Walking toward either exit without the urgent part still triggers the emergency handoff.
        super::super::minnesota_gyms::walk_to(&mut c, &runtime, (lane, 5));
        super::super::winona::travel(&mut c, Down, STATION, (lane as i16, 6));
        page(&mut c, "MnRewardMachinePartGivenText");
        assert_eq!(quantity(&c, "MACHINE_PART"), 1);
        assert!(!event(&c, WELCOME));
        finish_room_dialogue(&mut c);
        if repaired {
            move_between(&mut c, &runtime, LIGHTHOUSE);
            talk(&mut c, Up, &[]);
            assert!(event(&c, SIGNAL));
            assert_eq!(quantity(&c, "MACHINE_PART"), 0);
            move_between(&mut c, &runtime, STATION);
        } else {
            super::super::minnesota_gyms::walk_to(&mut c, &runtime, (5, 4));
        }
        c = invitation(c, repaired, false, None);
        let after_no = c.shell.shell.session().state().bag.clone();
        if lane == 2 && !repaired {
            c = capture(c, frames.as_deref(), "tamarack-station.png");
        }
        super::super::minnesota_gyms::walk_to(&mut c, &runtime, (lane, 5));
        super::super::winona::travel(&mut c, Down, STATION, (lane as i16, 6));
        clear_door(&mut c);
        cross_door(&mut c, Down, town);
        super::super::minnesota_gyms::walk_to(&mut c, &runtime, (outside.x, outside.y + 1));
        cross_door(&mut c, Up, STATION);
        super::super::minnesota_gyms::walk_to(&mut c, &runtime, (5, 4));
        clear_door(&mut c);
        assert_eq!(c.shell.shell.session().state().bag, after_no);
        assert!(!event(&c, WELCOME));
        c = invitation(c, repaired, true, None);
        finish_room_dialogue(&mut c);
        supplies(&c, true, true);
    }
}
