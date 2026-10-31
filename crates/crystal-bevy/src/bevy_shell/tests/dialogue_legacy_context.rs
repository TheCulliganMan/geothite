//! Surrounding directions must agree with the staged survey campaign.
use crystal_core::input::GameButton;

use super::super::northstar_dialogue::acknowledge;
use super::super::northstar_ice_lair::event;
use super::super::winona::{continue_in_place, finish, press};
use super::fixture;

const WELCOME: &str = "EVENT_MN_GRAND_MARAIS_STARTER";
const LAMP: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL";
const SURVEY: &str = "EVENT_MN_SURVEY_RESTORED";

#[test]
#[ignore = "declared local dialogue states; requires the expanded storyboard revision"]
fn overhaul_surrounding_hints_keep_ivy_tamarack_and_the_local_lamp_consistent() {
    for (map, tile, flags, label) in [
        ("MnR01Farmhouse", (5, 4), vec![], "MN_R01_FARMERText"),
        (
            "MnGrandMarais",
            (12, 12),
            vec![],
            "MnGrandMaraisStudentText",
        ),
        (
            "MnGrandMarais",
            (12, 12),
            vec![LAMP],
            "MnGrandMaraisStudentSignalText",
        ),
        (
            "MnGrandMaraisLighthouse",
            (5, 4),
            vec![],
            "MnGrandMaraisSignalMissingText",
        ),
        (
            "MnGrandMaraisFieldStation",
            (5, 4),
            vec![WELCOME, LAMP],
            "MnGrandMaraisTamarackThanksText",
        ),
    ] {
        let (root, runtime, mut c) = fixture(map, tile, &flags);
        let before = c.shell.shell.session().state().clone();
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        continue_in_place(
            &mut c,
            &root,
            &runtime,
            &format!("storyboard-context-{label}"),
        );
        acknowledge(&mut c, "CONTEXT", label, SURVEY);
        finish(&mut c);
        let after = c.shell.shell.session().state();
        assert_eq!(after.storage, before.storage);
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.badges, before.badges);
        assert!(!event(&c, SURVEY));
    }
}

#[test]
#[ignore = "declared local gift state; requires the expanded storyboard revision and isolated saves"]
fn overhaul_northern_researcher_gift_is_not_a_second_professor_opening() {
    let (root, runtime, mut c) = fixture("MnGrandMaraisFieldStation", (5, 4), &[]);
    let before = c.snapshot().expect("declared field team").party;
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("local researcher")
            .ui
            .text
            .and_then(|text| text.asm_text)
            .is_some_and(|page| page.starts_with("RESEARCHER:")),
        "the northern scientist must not introduce himself as Tamarack"
    );
    continue_in_place(
        &mut c,
        &root,
        &runtime,
        "storyboard-northern-researcher-introduction",
    );
    acknowledge(&mut c, "CONTEXT", "MnGrandMaraisWelcomeText", WELCOME);
    // The ordinary gift's receipt and nickname chooser own input before the
    // researcher's directions. Explicitly keep the original name through it.
    for _ in 0..64 {
        if let Some(name) = &c.shell.pending_name_choice {
            assert!(!event(&c, WELCOME));
            if name.nickname_pages.len() > 1 {
                press(&mut c, GameButton::A);
            } else {
                if name.selected != 1 {
                    press(&mut c, GameButton::Down);
                }
                press(&mut c, GameButton::A);
            }
            continue;
        }
        let snapshot = c.snapshot().expect("gift directions after nickname");
        if snapshot.ui.text_window_open
            && snapshot
                .ui
                .text
                .as_ref()
                .is_some_and(|text| text.label == "MnGrandMaraisPartnerText")
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    acknowledge(&mut c, "CONTEXT", "MnGrandMaraisPartnerText", LAMP);
    finish(&mut c);
    let after = c.snapshot().expect("actual field gift").party;
    assert_eq!(after.slots.len(), before.slots.len() + 1);
    assert_eq!(
        after.slots.last().expect("new partner").pokemon.species.id,
        "BIRCHIT"
    );
    assert_eq!(
        after
            .slots
            .last()
            .expect("field partner level")
            .pokemon
            .level,
        8
    );
    assert!(event(&c, WELCOME));
    assert!(!event(&c, LAMP) && !event(&c, SURVEY));
    assert!(!event(&c, "EVENT_MN_STARTER_RECEIVED"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["MACHINE_PART"]),
        1
    );
    continue_in_place(
        &mut c,
        &root,
        &runtime,
        "storyboard-northern-field-gift-received",
    );
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    acknowledge(&mut c, "CONTEXT", "MnGrandMaraisReminderText", LAMP);
    finish(&mut c);
    assert_eq!(c.snapshot().expect("repeat keeps the gift").party, after);
}
