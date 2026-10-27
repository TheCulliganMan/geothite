//! Local city/party/inventory fixtures, followed only by production input.
//! These cases do not earn regional access, courier progress, badges or rail travel.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::{ITEM_POCKET_ITEM, is_mail_item_id};
use crystal_core::systems::script_flags::{ScriptFlagCommand, apply_script_flag_mutation};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use serde_json::Value;

use super::super::super::super::{
    FieldPackAction, FieldPackPocket, StartMenuOption, VisibleShellController,
    mark_runtime_snapshot_dirty, visible_selected_pack_item_actions, visible_start_menu_options,
};
use super::super::super::r10_gallery::completed_visible_page;
use super::{
    at, continue_save, contract, controller, event, interact, load, navigate, press, walk_to,
};

const CITY: &str = "MnStPaul";
const QUIZ_FLAG: &str = "EVENT_MN_SPX_QUIZ_REWARDED";
const REWARDS: [&str; 3] = ["SILVERPOWDER", "MAGNET", "MYSTIC_WATER"];

type TrueFlags = (BTreeSet<String>, BTreeSet<String>);

#[derive(Clone, Copy)]
enum Inventory {
    Empty,
    FullItems,
}

fn fixture(inventory: Inventory) -> (AssetRoot, CrystalRuntime, Value, VisibleShellController) {
    // Immutable verified runtime data is shared; every session and cursor is fresh.
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime, Value)> = OnceLock::new();
    let (root, runtime, gyms) = LOADED.get_or_init(load).clone();
    let plan = contract();
    assert_eq!(plan["city"], CITY);
    let city = &runtime.data().maps[CITY];
    assert_eq!([city.attributes.width, city.attributes.height], [64, 48]);
    let mut setup = gyms["gyms"][2].clone();
    setup["id"] = plan["city"].clone();
    setup["arrival_core"] = plan["arrival_core"].clone();
    let mut c = controller(&root, &runtime, &setup, false);
    let state = c.shell.shell.session_mut().state_mut();
    // The common helper supplies a legal strong review partner. Clear its prior
    // badge fixture through production mutation: these walking tasks need none.
    for (index, flag) in [
        "ENGINE_ZEPHYRBADGE",
        "ENGINE_HIVEBADGE",
        "ENGINE_PLAINBADGE",
        "ENGINE_FOGBADGE",
        "ENGINE_STORMBADGE",
        "ENGINE_MINERALBADGE",
        "ENGINE_GLACIERBADGE",
        "ENGINE_RISINGBADGE",
    ]
    .into_iter()
    .enumerate()
    {
        apply_script_flag_mutation(
            state,
            ScriptFlagCommand {
                command: "clearflag".into(),
                flag_id: flag.into(),
                source_script: "StPaulLocalActivityFixture".into(),
                command_index: index,
            },
        )
        .expect("consistent initial no-badge fixture");
    }
    state.money = 54_321;
    state.bag.items.clear();
    if matches!(inventory, Inventory::FullItems) {
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["POTION"], 1)
                .expect("initial tossable medicine")
        );
        for (id, item) in runtime
            .data()
            .items
            .iter()
            .filter(|(id, item)| {
                item.pocket == ITEM_POCKET_ITEM
                    && id.as_str() != "POTION"
                    && !REWARDS.contains(&id.as_str())
                    && !is_mail_item_id(id)
            })
            .take(ITEM_POCKET_CAPACITY - 1)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .unwrap_or_else(|error| panic!("initial legal filler {id}: {error}"))
            );
        }
        assert_eq!(state.bag.items.len(), ITEM_POCKET_CAPACITY);
    }
    for item in REWARDS {
        assert_eq!(state.bag.quantity(&runtime.data().items[item]), 0);
    }
    // No position, inventory, party, flag, RNG or money writes after this point.
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, plan, c)
}

fn flags(c: &VisibleShellController) -> TrueFlags {
    let f = &c.shell.shell.session().state().flags;
    (
        f.active_event_flags().cloned().collect(),
        f.engine_flags
            .iter()
            .filter_map(|(name, enabled)| enabled.then_some(name.clone()))
            .collect(),
    )
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[item])
}

fn allowed(plan: &Value) -> BTreeSet<String> {
    plan["rooms"]
        .as_array()
        .expect("exact city rooms")
        .iter()
        .map(|r| r["id"].as_str().expect("room name").to_owned())
        .chain([CITY.to_owned()])
        .collect()
}

fn room(c: &mut VisibleShellController, r: &CrystalRuntime, plan: &Value, name: &str) {
    navigate(c, r, name, &allowed(plan));
}

fn pages(c: &mut VisibleShellController, stage: &str) -> BTreeSet<String> {
    let mut printed = BTreeSet::new();
    let stand = at(c);
    for _ in 0..64 {
        c.wait_frames(240).expect("native text printer clock");
        let s = c.snapshot().expect("actual current dialogue");
        if let Some((_, page)) = completed_visible_page(c) {
            printed.insert(page);
            press(c, GameButton::Left);
            assert_eq!(at(c), stand, "{stage}: printed dialogue owns movement");
        }
        assert!(
            s.ui.pending_yes_no.is_none(),
            "{stage}: unexpected unanswered native choice"
        );
        if s.phase == RuntimeShellPhase::Overworld
            && !s.ui.text_window_open
            && s.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
        {
            return printed;
        }
        press(c, GameButton::A);
    }
    panic!(
        "{stage}: dialogue did not release input; printed={printed:#?}; snapshot={:#?}",
        c.snapshot().expect("bounded final diagnostic")
    );
}

fn assert_page(printed: &BTreeSet<String>, expected: &str) {
    assert!(
        printed.contains(expected),
        "expected fully printed {expected:?}; observed={printed:#?}"
    );
}

fn quiz(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    plan: &Value,
    answers: &[bool],
) -> BTreeSet<String> {
    room(c, r, plan, "MnStPaulFairQuizPavilion");
    interact(c, r, (5, 3));
    let questions = [
        "MINT BERRY wakes\na sleeping friend?",
        "A POTION restores\na fainted friend?",
        "A LURE BALL suits\na fishing catch?",
    ];
    for (i, &yes) in answers.iter().enumerate() {
        c.wait_frames(240)
            .expect("fully print native quiz question");
        let observed = completed_visible_page(c).map(|(_, page)| page);
        assert_eq!(observed.as_deref(), Some(questions[i]), "quiz question{i}");
        assert!(
            c.snapshot()
                .expect("native yes/no prompt")
                .ui
                .pending_yes_no
                .is_some()
        );
        let stand = at(c);
        press(c, GameButton::Left);
        assert_eq!(at(c), stand, "native quiz owns movement");
        let answer = usize::from(!yes);
        for _ in 0..4 {
            let selected = c
                .shell
                .yes_no_cursor
                .as_ref()
                .expect("native YES/NO cursor")
                .option_index;
            if selected == answer {
                break;
            }
            press(
                c,
                if selected < answer {
                    GameButton::Down
                } else {
                    GameButton::Up
                },
            );
        }
        assert_eq!(
            c.shell
                .yes_no_cursor
                .as_ref()
                .expect("selected answer")
                .option_index,
            answer
        );
        press(c, GameButton::A);
    }
    pages(c, "quiz result")
}

fn host(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    plan: &Value,
    task: &str,
) -> BTreeSet<String> {
    room(
        c,
        r,
        plan,
        if task == "ARCHIVE" {
            "MnStPaulCapitolArchive"
        } else {
            "MnStPaulCivicMuseum"
        },
    );
    interact(c, r, (5, 3));
    pages(c, "record task host")
}

fn record(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    plan: &Value,
    task: &str,
    index: usize,
) {
    let (name, tile, expected) = match (task, index) {
        ("ARCHIVE", 1) => (
            "MnStPaulCapitolArchive",
            (4, 9),
            "Early rail roads\nran by the river.",
        ),
        ("ARCHIVE", 2) => (
            "MnStPaulCivicMuseum",
            (11, 9),
            "The CAPITOL stands\non the high bluff.",
        ),
        ("ARCHIVE", 3) => (
            "MnStPaulArchiveAnnex",
            (11, 9),
            "Two banks share\none public city.",
        ),
        ("RIVER", 1) => (CITY, (33, 59), "Measure north-bank\nwater at the rail."),
        ("RIVER", 2) => (CITY, (87, 87), "The bridge spans\nboth river banks."),
        ("RIVER", 3) => (CITY, (67, 61), "Read high marks\nfrom a safe path."),
        _ => panic!("unknown explicit record case"),
    };
    room(c, r, plan, name);
    let flag = format!("EVENT_MN_SPX_{task}_{index}");
    assert!(!event(c, &flag));
    if task == "ARCHIVE" && index == 1 {
        walk_to(c, r, (4, 8));
        press(c, GameButton::Down);
        assert_eq!(
            at(c),
            (4, 8),
            "record occupies its declared tile, never the passive arrival"
        );
        press(c, GameButton::A);
    } else {
        interact(c, r, tile);
    }
    c.wait_frames(240)
        .expect("native record printing without acknowledgement");
    assert!(
        !event(c, &flag),
        "record flag must await native acknowledgement"
    );
    assert_eq!(
        completed_visible_page(c).map(|(_, page)| page).as_deref(),
        Some(expected)
    );
    let printed = pages(c, "record acknowledgement");
    assert_page(&printed, expected);
    assert!(event(c, &flag));
}

fn assert_no_unrelated_events(
    before: &TrueFlags,
    c: &VisibleShellController,
    additions: &[String],
) {
    let mut expected = before.clone();
    expected.0.extend(additions.iter().cloned());
    assert_eq!(
        flags(c),
        expected,
        "only authored optional task events may change"
    );
}

#[test]
#[ignore = "requires the exact published tight St Paul city contract and native pack"]
fn st_paul_expansion_controller_quiz_wrong_success_once_and_continue() {
    let (root, r, plan, mut c) = fixture(Inventory::Empty);
    let before = flags(&c);
    let before_party = c.snapshot().expect("initial legal review party").party;
    let mut expected_bag = c.shell.shell.session().state().bag.clone();
    assert!(
        expected_bag
            .add_item(&r.data().items["SILVERPOWDER"], 1)
            .expect("expected native reward inventory")
    );
    assert_page(
        &quiz(&mut c, &r, &plan, &[false]),
        "Use the field book\nand visit again.",
    );
    assert!(!event(&c, QUIZ_FLAG));
    assert_eq!(quantity(&c, &r, "SILVERPOWDER"), 0);
    assert_eq!(flags(&c), before);
    assert_page(
        &quiz(&mut c, &r, &plan, &[true, false, true]),
        "All answers right!\nSILVERPOWDER gift.",
    );
    assert!(event(&c, QUIZ_FLAG));
    assert_eq!(quantity(&c, &r, "SILVERPOWDER"), 1);
    room(&mut c, &r, &plan, "MnStPaulFairQuizPavilion");
    interact(&mut c, &r, (5, 3));
    assert_page(
        &pages(&mut c, "quiz repeat"),
        "All answers right!\nEnjoy the fair.",
    );
    assert_eq!(quantity(&c, &r, "SILVERPOWDER"), 1);
    assert_no_unrelated_events(&before, &c, &[QUIZ_FLAG.into()]);
    assert_eq!(
        c.snapshot().expect("quiz preserves party").party,
        before_party
    );
    assert_eq!(c.shell.shell.session().state().bag, expected_bag);
    let saved_flags = flags(&c);
    c = continue_save(&mut c, &root, &r);
    assert_eq!(flags(&c), saved_flags);
    assert_eq!(quantity(&c, &r, "SILVERPOWDER"), 1);
    assert_eq!(c.shell.shell.session().state().money, 54_321);
}

#[test]
#[ignore = "requires exact optional city record scripts, native entrances and walking bridges"]
fn st_paul_expansion_controller_archive_and_river_acknowledgements_once_and_continue() {
    let (root, r, plan, mut c) = fixture(Inventory::Empty);
    let before = flags(&c);
    let before_party = c.snapshot().expect("initial walking review party").party;
    let mut expected_bag = c.shell.shell.session().state().bag.clone();
    for item in ["MAGNET", "MYSTIC_WATER"] {
        assert!(
            expected_bag
                .add_item(&r.data().items[item], 1)
                .expect("expected exact record rewards")
        );
    }
    let mut additions = Vec::new();
    for (task, item, need, reward) in [
        (
            "ARCHIVE",
            "MAGNET",
            "Visit the archive,\nmuseum and annex.",
            "MAGNET received!\nFor your records.",
        ),
        (
            "RIVER",
            "MYSTIC_WATER",
            "Read three river\nmeasurement posts.",
            "MYSTIC WATER gift.\nFor your research.",
        ),
    ] {
        assert_page(&host(&mut c, &r, &plan, task), need);
        assert_eq!(quantity(&c, &r, item), 0);
        record(&mut c, &r, &plan, task, 1);
        assert_page(&host(&mut c, &r, &plan, task), need);
        assert!(!event(&c, &format!("EVENT_MN_SPX_{task}_REWARDED")));
        record(&mut c, &r, &plan, task, 2);
        assert_page(&host(&mut c, &r, &plan, task), need);
        record(&mut c, &r, &plan, task, 3);
        assert_page(&host(&mut c, &r, &plan, task), reward);
        assert_eq!(quantity(&c, &r, item), 1);
        let reward_flag = format!("EVENT_MN_SPX_{task}_REWARDED");
        assert!(event(&c, &reward_flag));
        assert_page(
            &host(&mut c, &r, &plan, task),
            "Our work is saved.\nEnjoy the city.",
        );
        assert_eq!(quantity(&c, &r, item), 1);
        additions.extend((1..=3).map(|i| format!("EVENT_MN_SPX_{task}_{i}")));
        additions.push(reward_flag);
    }
    assert_no_unrelated_events(&before, &c, &additions);
    let mut walking_party = c.snapshot().expect("records preserve party and PP").party;
    assert_eq!(walking_party.slots.len(), before_party.slots.len());
    for (after, before) in walking_party.slots.iter_mut().zip(&before_party.slots) {
        assert!(
            after.pokemon.happiness >= before.pokemon.happiness,
            "native walking may earn friendship"
        );
        // Normalize only this comparison copy. Native walking friendship stays
        // in the actual party and is checked exactly across Continue below.
        after.pokemon.happiness = before.pokemon.happiness;
    }
    assert_eq!(walking_party, before_party);
    assert_eq!(c.shell.shell.session().state().bag, expected_bag);
    let saved_flags = flags(&c);
    c = continue_save(&mut c, &root, &r);
    assert_eq!(flags(&c), saved_flags);
    for item in ["MAGNET", "MYSTIC_WATER"] {
        assert_eq!(quantity(&c, &r, item), 1);
    }
    assert_eq!(c.shell.shell.session().state().money, 54_321);
}

fn toss_initial_potion(c: &mut VisibleShellController, r: &CrystalRuntime) {
    press(c, GameButton::Start);
    let s = c.snapshot().expect("production Start menu");
    let target = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("actual PACK option");
    for _ in 0..16 {
        let selected = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("native Start cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    let target = c
        .snapshot()
        .expect("actual carried items")
        .bag
        .items
        .iter()
        .filter(|slot| slot.quantity > 0)
        .position(|slot| slot.item_id == "POTION")
        .expect("initial one Potion remains carried");
    for _ in 0..32 {
        let selected = c
            .shell
            .bag_cursor
            .as_ref()
            .expect("native item list")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    let s = c.snapshot().expect("native item action menu");
    let actions = visible_selected_pack_item_actions(&s, &c.shell, &FieldPackPocket::Items, false)
        .expect("native legal item actions");
    let target = actions
        .iter()
        .position(|action| *action == FieldPackAction::Toss)
        .expect("native TOSS action");
    for _ in 0..8 {
        let selected = c
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("actual Pack action cursor")
            .option_index;
        if selected == target {
            press(c, GameButton::A);
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    let toss = c
        .shell
        .pack_toss
        .as_ref()
        .expect("native toss quantity selection");
    assert_eq!(toss.item_id, "POTION");
    assert_eq!(toss.quantity, 1);
    assert!(!toss.confirming);
    assert_eq!(quantity(c, r, "POTION"), 1);
    press(c, GameButton::A);
    assert!(
        c.shell
            .pack_toss
            .as_ref()
            .expect("native yes/no confirmation")
            .confirming
    );
    assert_eq!(
        quantity(c, r, "POTION"),
        1,
        "quantity approval alone removes nothing"
    );
    c.wait_frames(240)
        .expect("native toss confirmation printer");
    press(c, GameButton::A);
    assert_eq!(
        quantity(c, r, "POTION"),
        0,
        "native confirmed toss makes space"
    );
    for _ in 0..64 {
        let s = c.snapshot().expect("native Pack closure");
        if s.ui.menu.is_none()
            && !s.ui.text_window_open
            && c.shell.field_notice.is_none()
            && c.shell.bag_cursor.is_none()
            && !c.has_pending_script_work()
        {
            return;
        }
        c.wait_frames(240).expect("native Pack notice clock");
        press(
            c,
            if c.shell.field_notice.is_some() || s.ui.pending_text_wait.is_some() {
                GameButton::A
            } else {
                GameButton::B
            },
        );
    }
    panic!("native Pack did not release input after confirmed toss");
}

#[test]
#[ignore = "initial full-pocket fixtures; every refusal, make-space retry and reward uses native input"]
fn st_paul_expansion_controller_all_rewards_refuse_full_pocket_then_native_toss_retry() {
    for (task, item) in [
        ("QUIZ", "SILVERPOWDER"),
        ("ARCHIVE", "MAGNET"),
        ("RIVER", "MYSTIC_WATER"),
    ] {
        let (root, r, plan, mut c) = fixture(Inventory::FullItems);
        let before = c.shell.shell.session().state().bag.clone();
        let before_flags = flags(&c);
        let before_party = c
            .snapshot()
            .expect("initial full-pocket legal partner")
            .party;
        let mut expected_bag = before.clone();
        assert!(
            expected_bag
                .remove_item(&r.data().items["POTION"], 1)
                .expect("expected native toss")
        );
        assert!(
            expected_bag
                .add_item(&r.data().items[item], 1)
                .expect("expected retried reward")
        );
        if task != "QUIZ" {
            for index in 1..=3 {
                record(&mut c, &r, &plan, task, index);
            }
        }
        let flag = format!("EVENT_MN_SPX_{task}_REWARDED");
        let refused = if task == "QUIZ" {
            quiz(&mut c, &r, &plan, &[true, false, true])
        } else {
            host(&mut c, &r, &plan, task)
        };
        assert_page(&refused, "Your PACK is full.\nReturn with room.");
        assert!(!event(&c, &flag));
        assert_eq!(quantity(&c, &r, item), 0);
        assert_eq!(
            c.shell.shell.session().state().bag,
            before,
            "refusal cannot consume or grant items"
        );
        toss_initial_potion(&mut c, &r);
        assert_eq!(
            c.shell.shell.session().state().bag.items.len(),
            ITEM_POCKET_CAPACITY - 1
        );
        if task == "QUIZ" {
            quiz(&mut c, &r, &plan, &[true, false, true]);
        } else {
            host(&mut c, &r, &plan, task);
        }
        assert!(event(&c, &flag));
        assert_eq!(quantity(&c, &r, item), 1);
        assert_eq!(
            c.shell.shell.session().state().bag.items.len(),
            ITEM_POCKET_CAPACITY
        );
        let mut additions = if task == "QUIZ" {
            Vec::new()
        } else {
            (1..=3)
                .map(|i| format!("EVENT_MN_SPX_{task}_{i}"))
                .collect()
        };
        additions.push(flag);
        assert_no_unrelated_events(&before_flags, &c, &additions);
        assert_eq!(
            c.snapshot().expect("retry preserves party and PP").party,
            before_party
        );
        assert_eq!(c.shell.shell.session().state().bag, expected_bag);
        let before_continue = flags(&c);
        c = continue_save(&mut c, &root, &r);
        assert_eq!(flags(&c), before_continue);
        assert_eq!(quantity(&c, &r, item), 1);
        assert_eq!(quantity(&c, &r, "POTION"), 0);
        assert_eq!(c.shell.shell.session().state().money, 54_321);
    }
}

#[test]
#[ignore = "requires the published new East Aid clinic and native PokemonCenterPC special"]
fn st_paul_expansion_controller_new_clinic_pc_owns_input_and_closes() {
    let (root, r, plan, mut c) = fixture(Inventory::Empty);
    room(&mut c, &r, &plan, "MnStPaulEastAidClinic");
    let before_party = c.snapshot().expect("initial review partner").party;
    let before_flags = flags(&c);
    interact(&mut c, &r, (9, 1));
    for _ in 0..96 {
        if c.shell.pc_hub_cursor.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        c.shell.pc_hub_cursor.is_some(),
        "real PokemonCenterPC menu must open"
    );
    assert!(c.snapshot().expect("public PC surface").ui.menu.is_some());
    let stand = at(&mut c);
    press(&mut c, GameButton::Left);
    assert_eq!(at(&mut c), stand, "PC menu owns directional input");
    for _ in 0..96 {
        if c.shell.pc_hub_cursor.is_none()
            && c.shell.special_boundary.is_none()
            && c.snapshot().expect("PC closure").ui.menu.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(&mut c, GameButton::B);
    }
    assert!(c.shell.pc_hub_cursor.is_none());
    assert!(!c.has_pending_script_work());
    assert_eq!(
        c.snapshot().expect("PC leaves partner unchanged").party,
        before_party
    );
    assert_eq!(flags(&c), before_flags);
    c = continue_save(&mut c, &root, &r);
    assert_eq!(at(&mut c), stand);
    assert_eq!(flags(&c), before_flags);
}
