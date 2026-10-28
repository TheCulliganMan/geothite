//! Explicit local fixtures followed only by native travel, dialogue and Pack input.
//! These optional activities do not earn city access, badges or main-story progress.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::PokemonStorage;
use crystal_core::models::bag::{Bag, ITEM_POCKET_CAPACITY};
use crystal_core::models::item::{ITEM_POCKET_ITEM, is_mail_item_id};
use crystal_core::systems::script_flags::{ScriptFlagCommand, apply_script_flag_mutation};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use serde_json::Value;

use super::super::super::super::{
    FieldPackAction, FieldPackPocket, FieldPackTargetMode, StartMenuOption, VisibleShellController,
    mark_runtime_snapshot_dirty, visible_selected_pack_item_actions, visible_start_menu_options,
};
use super::super::super::r10_gallery::completed_visible_page;
use super::{
    at, continue_save, contract, controller, event, interact, load, navigate, point, press,
};

const CITY: &str = "MnMinneapolis";
const GIVER: &str = "MnMinneapolisGrainExchange";
const RECEIVER: &str = "MnMinneapolisPrintShop";
const MUSEUM: &str = "MnMinneapolisFoundryMuseum";
const GIVEN: &str = "EVENT_MN_MP_PRINT_MAIL_GIVEN";
const DELIVERED: &str = "EVENT_MN_MP_PRINT_MAIL_DELIVERED";
const TRAIL_REWARD: &str = "EVENT_MN_MP_MILL_TRAIL_REWARDED";
const PLAQUES: [(&str, &str, &str); 3] = [
    (
        "MN_MP_MILL_WATER_PLAQUE",
        "EVENT_MN_MP_PLAQUE_WATER_READ",
        "RIVER POWER The current drove the mill wheels.",
    ),
    (
        "MN_MP_MILL_GRAIN_PLAQUE",
        "EVENT_MN_MP_PLAQUE_GRAIN_READ",
        "GRAIN AND RAIL Flour left the city by the rail yards.",
    ),
    (
        "MN_MP_MILL_WORKERS_PLAQUE",
        "EVENT_MN_MP_PLAQUE_WORKERS_READ",
        "MILL WORKERS Many hands built this river city.",
    ),
];

type TrueFlags = (BTreeSet<String>, BTreeSet<String>);

#[derive(Default)]
struct Setup<'a> {
    items: &'a [(&'a str, u16)],
    flags: &'a [&'a str],
    full_pocket_except: Option<&'a str>,
    injury: u16,
}

fn fixture(setup: Setup<'_>) -> (AssetRoot, CrystalRuntime, Value, VisibleShellController) {
    // Only immutable loaded data is shared; every authoritative session is fresh.
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime, Value)> = OnceLock::new();
    let (root, runtime, gyms) = LOADED.get_or_init(load).clone();
    let plan = contract();
    assert_eq!(plan["city"], CITY);
    let city = &runtime.data().maps[CITY];
    assert_eq!(
        (city.attributes.width, city.attributes.height),
        point(&plan["dimensions_metatiles"]),
        "fixture extent must match the actual published city"
    );
    let mut location = gyms["gyms"][2].clone();
    location["id"] = plan["city"].clone();
    location["arrival_core"] = plan["arrival_core"].clone();
    let mut c = controller(&root, &runtime, &location, false);
    let state = c.shell.shell.session_mut().state_mut();
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
                source_script: "MinneapolisLocalActivityFixture".into(),
                command_index: index,
            },
        )
        .expect("consistent initial no-badge fixture");
    }
    state.money = 43_210;
    state.bag.items.clear();
    for &(id, quantity) in setup.items {
        assert!(
            state
                .bag
                .add_item(&runtime.data().items[id], quantity)
                .expect("explicit initial legal item stack")
        );
    }
    if let Some(excluded) = setup.full_pocket_except {
        assert_eq!(runtime.data().items[excluded].pocket, ITEM_POCKET_ITEM);
        assert_ne!(excluded, "POTION", "retry medicine must not be the reward");
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["POTION"], 1)
                .expect("initial usable medicine")
        );
        let initial_ids = state.bag.items.keys().cloned().collect::<BTreeSet<_>>();
        let remaining = ITEM_POCKET_CAPACITY - state.bag.items.len();
        for (id, item) in runtime
            .data()
            .items
            .iter()
            .filter(|(id, item)| {
                item.pocket == ITEM_POCKET_ITEM
                    && id.as_str() != "POTION"
                    && id.as_str() != excluded
                    && !is_mail_item_id(id)
                    && !initial_ids.contains(&item.script_name)
            })
            .take(remaining)
        {
            assert!(
                state
                    .bag
                    .add_item(item, 1)
                    .unwrap_or_else(|error| panic!("initial native filler {id}: {error}"))
            );
        }
        assert_eq!(state.bag.items.len(), ITEM_POCKET_CAPACITY);
        assert_eq!(state.bag.quantity(&runtime.data().items[excluded]), 0);
    }
    for flag in setup.flags {
        state
            .flags
            .set_event_flag(flag, true)
            .expect("explicit initial activity prerequisite");
    }
    let partner = state.storage.party.pokemon[0]
        .as_mut()
        .expect("legal initial review partner");
    assert!(partner.max_hp > setup.injury);
    partner.hp = partner.max_hp - setup.injury;
    state.sync_party_from_storage();
    // No subsequent state, cursor, position, party, inventory or flag writes.
    mark_runtime_snapshot_dirty(&mut c.shell);
    (root, runtime, plan, c)
}

fn flags(c: &VisibleShellController) -> TrueFlags {
    let state = &c.shell.shell.session().state().flags;
    (
        state.active_event_flags().cloned().collect(),
        state
            .engine_flags
            .iter()
            .filter_map(|(name, enabled)| enabled.then_some(name.clone()))
            .collect(),
    )
}

fn assert_storage_after_walking(c: &VisibleShellController, before: &PokemonStorage) {
    let mut comparison = c.shell.shell.session().state().storage.clone();
    for (current, prior) in comparison
        .party
        .pokemon
        .iter_mut()
        .zip(&before.party.pokemon)
    {
        if let (Some(current), Some(prior)) = (current, prior) {
            assert!(
                current.happiness >= prior.happiness,
                "native walking may earn friendship"
            );
            // Normalize only the comparison clone. Native friendship is retained
            // in the authoritative session and compared exactly over Continue.
            current.happiness = prior.happiness;
        }
    }
    assert_eq!(
        comparison, *before,
        "party resources and all PC boxes stay exact"
    );
}

fn quantity(c: &VisibleShellController, r: &CrystalRuntime, id: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items[id])
}

fn assert_bag_delta(
    c: &VisibleShellController,
    r: &CrystalRuntime,
    before: &Bag,
    deltas: &[(&str, i16)],
) {
    let mut expected = before.clone();
    for &(id, delta) in deltas {
        let changed = if delta < 0 {
            expected.remove_item(&r.data().items[id], delta.unsigned_abs())
        } else {
            expected.add_item(&r.data().items[id], delta.unsigned_abs())
        };
        assert!(changed.expect("legal expected inventory transaction"));
    }
    assert_eq!(
        c.shell.shell.session().state().bag,
        expected,
        "all bag pockets equal the explicit item transaction"
    );
}

fn room<'a>(plan: &'a Value, id: &str) -> &'a Value {
    plan["rooms"]
        .as_array()
        .expect("new authored rooms")
        .iter()
        .find(|room| room["id"] == id)
        .unwrap_or_else(|| panic!("explicit room {id}"))
}

fn travel(c: &mut VisibleShellController, r: &CrystalRuntime, plan: &Value, id: &str) {
    let allowed = plan["rooms"]
        .as_array()
        .expect("new room graph")
        .iter()
        .map(|room| room["id"].as_str().expect("room ID").to_owned())
        .chain([CITY.to_owned()])
        .collect();
    navigate(c, r, id, &allowed);
}

fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn pages(c: &mut VisibleShellController, stage: &str) -> String {
    let mut printed = Vec::new();
    let stand = at(c);
    for _ in 0..128 {
        c.wait_frames(240).expect("native full-page printing");
        let snapshot = c.snapshot().expect("actual dialogue state");
        assert!(snapshot.battle.is_none(), "{stage}: unexpected battle");
        assert!(
            snapshot.ui.pending_yes_no.is_none(),
            "{stage}: unexpected choice"
        );
        if let Some((_, page)) = completed_visible_page(c) {
            printed.push(page);
            press(c, GameButton::Left);
            assert_eq!(at(c), stand, "{stage}: dialogue retains movement ownership");
        }
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && snapshot.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
        {
            assert!(!printed.is_empty(), "{stage}: no fully printed page");
            return normalized(&printed.join(" "));
        }
        press(c, GameButton::A);
    }
    panic!("{stage}: dialogue failed to release input; printed={printed:#?}");
}

fn talk(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    plan: &Value,
    map: &str,
    actor: &str,
) -> String {
    let storage = c.shell.shell.session().state().storage.clone();
    let money = c.shell.shell.session().state().money;
    travel(c, r, plan, map);
    let actor = room(plan, map)["residents"]
        .as_array()
        .expect("room residents")
        .iter()
        .find(|entry| entry["id"] == actor)
        .unwrap_or_else(|| panic!("explicit activity actor {actor}"));
    interact(c, r, point(&actor["tile"]));
    let printed = pages(c, map);
    assert_storage_after_walking(c, &storage);
    assert_eq!(c.shell.shell.session().state().money, money);
    printed
}

fn mail(c: &mut VisibleShellController, r: &CrystalRuntime, plan: &Value, giver: bool) -> String {
    talk(
        c,
        r,
        plan,
        if giver { GIVER } else { RECEIVER },
        if giver {
            "MN_MP_GRAINEXCHANGE_HOST"
        } else {
            "MN_MP_PRINTSHOP_HOST"
        },
    )
}

fn museum(c: &mut VisibleShellController, r: &CrystalRuntime, plan: &Value) -> String {
    talk(c, r, plan, MUSEUM, "MN_MP_FOUNDRYMUSEUM_HOST")
}

fn assert_text(actual: String, expected: &str) {
    assert_eq!(
        actual,
        normalized(expected),
        "all authored dialogue pages fully printed"
    );
}

fn continued(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
) -> VisibleShellController {
    let before_flags = flags(c);
    let state = c.shell.shell.session().state();
    let money = state.money;
    let storage = state.storage.clone();
    let loaded = continue_save(c, root, r);
    assert_eq!(
        flags(&loaded),
        before_flags,
        "all semantic true flags survive Continue"
    );
    assert_eq!(loaded.shell.shell.session().state().money, money);
    assert_eq!(loaded.shell.shell.session().state().storage, storage);
    loaded
}

fn cursor(
    c: &mut VisibleShellController,
    target: usize,
    selected: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..64 {
        let index = selected(c);
        if index == target {
            return;
        }
        press(
            c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native menu cursor did not reach {target}");
}

fn use_medicine(c: &mut VisibleShellController, r: &CrystalRuntime, id: &str) {
    let before_quantity = quantity(c, r, id);
    let before_bag = c.shell.shell.session().state().bag.clone();
    let before = c.snapshot().expect("initial injured partner").party;
    let mut expected = before.clone();
    let amount = u16::try_from(r.data().items[id].parameter).expect("native healing amount");
    let partner = &mut expected.slots[0].pokemon;
    partner.hp = partner.hp.saturating_add(amount).min(partner.max_hp);
    assert_ne!(before, expected, "medicine must have a real native effect");
    let before_flags = flags(c);
    let money = c.shell.shell.session().state().money;
    press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("native Start menu");
    let target = visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|option| *option == StartMenuOption::Pack)
        .expect("native PACK");
    cursor(c, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    press(c, GameButton::A);
    let target = c
        .snapshot()
        .expect("native carried medicine")
        .bag
        .items
        .iter()
        .filter(|item| item.quantity > 0)
        .position(|item| item.item_id == id)
        .expect("actual medicine stack");
    cursor(c, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("Pack cursor")
            .option_index
    });
    press(c, GameButton::A);
    let snapshot = c.snapshot().expect("native item actions");
    let target =
        visible_selected_pack_item_actions(&snapshot, &c.shell, &FieldPackPocket::Items, false)
            .expect("native medicine actions")
            .iter()
            .position(|action| *action == FieldPackAction::Use)
            .expect("native USE");
    cursor(c, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action cursor")
            .option_index
    });
    press(c, GameButton::A);
    assert_eq!(
        c.shell.field_pack_target_mode,
        Some(FieldPackTargetMode::PartyPokemon)
    );
    cursor(c, 0, |c| c.shell.party_cursor);
    press(c, GameButton::A);
    assert_eq!(
        c.snapshot().expect("native medicine effect").party,
        expected
    );
    assert_eq!(quantity(c, r, id), before_quantity - 1);
    assert_bag_delta(c, r, &before_bag, &[(id, -1)]);
    assert_eq!(flags(c), before_flags);
    assert_eq!(c.shell.shell.session().state().money, money);
    let stand = at(c);
    press(c, GameButton::Down);
    assert_eq!(at(c), stand, "medicine notice owns movement");
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("close native Pack");
        if c.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
            c.wait_frames(60).expect("medicine notice clock");
            press(c, GameButton::A);
        } else if snapshot.ui.menu.is_some()
            || c.shell.field_pack_target_mode.is_some()
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.bag_cursor.is_some()
            || c.shell.field_pack_pocket.is_some()
            || c.shell.start_menu_cursor.is_some()
        {
            press(c, GameButton::B);
        } else {
            return;
        }
    }
    panic!("native medicine menus failed to release input");
}

#[test]
#[ignore = "requires the published tight64 Minneapolis activity contract and native pack"]
fn minneapolis_expansion_controller_mail_delivery_is_atomic_once_and_continues() {
    let (root, r, plan, mut c) = fixture(Setup::default());
    let initial_bag = c.shell.shell.session().state().bag.clone();
    let initial_flags = flags(&c);
    let party = c.shell.shell.session().state().storage.clone();
    let money = c.shell.shell.session().state().money;
    assert_text(
        mail(&mut c, &r, &plan, false),
        "The exchange has our new print order.",
    );
    assert!(!event(&c, GIVEN));
    assert!(!event(&c, DELIVERED));
    assert_bag_delta(&c, &r, &initial_bag, &[]);
    assert_text(
        mail(&mut c, &r, &plan, true),
        "Take this print order to the shop. It is across the upper river bridge.",
    );
    assert!(event(&c, GIVEN));
    assert!(!event(&c, DELIVERED));
    assert_eq!(quantity(&c, &r, "FLOWER_MAIL"), 1);
    assert_eq!(quantity(&c, &r, "SUPER_POTION"), 0);
    assert_bag_delta(&c, &r, &initial_bag, &[("FLOWER_MAIL", 1)]);
    let given_bag = c.shell.shell.session().state().bag.clone();
    assert_text(
        mail(&mut c, &r, &plan, true),
        "The print shop is across the upper river bridge.",
    );
    assert_eq!(quantity(&c, &r, "FLOWER_MAIL"), 1);
    assert_bag_delta(&c, &r, &given_bag, &[]);
    assert_text(
        mail(&mut c, &r, &plan, false),
        "The new mill map! Thanks for bringing the print order. Take these POTIONS.",
    );
    assert_eq!(quantity(&c, &r, "FLOWER_MAIL"), 0);
    assert_eq!(quantity(&c, &r, "SUPER_POTION"), 3);
    assert!(event(&c, GIVEN) && event(&c, DELIVERED));
    assert_bag_delta(
        &c,
        &r,
        &given_bag,
        &[("SUPER_POTION", 3), ("FLOWER_MAIL", -1)],
    );
    let mut expected_flags = initial_flags;
    expected_flags
        .0
        .extend([GIVEN.to_owned(), DELIVERED.to_owned()]);
    assert_eq!(flags(&c), expected_flags);
    let awarded = c.snapshot().expect("complete delivery").bag;
    for giver in [false, true] {
        assert_text(
            mail(&mut c, &r, &plan, giver),
            if giver {
                "You delivered it. Thank you."
            } else {
                "The order is here. The press is ready."
            },
        );
    }
    c = continued(&mut c, &root, &r);
    assert_text(
        mail(&mut c, &r, &plan, false),
        "The order is here. The press is ready.",
    );
    assert_text(
        mail(&mut c, &r, &plan, true),
        "You delivered it. Thank you.",
    );
    assert_eq!(c.snapshot().expect("once-only delivery").bag, awarded);
    assert_eq!(flags(&c), expected_flags);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_storage_after_walking(&c, &party);
}

#[test]
#[ignore = "requires published mail refusal programs and native item capacity"]
fn minneapolis_expansion_controller_mail_missing_and_full_giver_refuse() {
    for (items, initial_flags, giver, full_pocket_except, expected) in [
        (
            &[("FLOWER_MAIL", 1)][..],
            &[][..],
            false,
            None,
            "The exchange has our new print order.",
        ),
        (
            &[][..],
            &[GIVEN][..],
            false,
            None,
            "The exchange has our new print order.",
        ),
        (
            &[][..],
            &[][..],
            true,
            Some("FLOWER_MAIL"),
            "Your PACK is full. Come back later.",
        ),
    ] {
        let (root, r, plan, mut c) = fixture(Setup {
            items,
            flags: initial_flags,
            full_pocket_except,
            ..Default::default()
        });
        let before = c.snapshot().expect("refusal inventory").bag;
        let before_flags = flags(&c);
        let storage = c.shell.shell.session().state().storage.clone();
        let money = c.shell.shell.session().state().money;
        assert_text(mail(&mut c, &r, &plan, giver), expected);
        assert_eq!(flags(&c), before_flags);
        assert_eq!(
            c.snapshot().expect("refusal unchanged inventory").bag,
            before
        );
        assert!(!event(&c, DELIVERED));
        c = continued(&mut c, &root, &r);
        assert_text(mail(&mut c, &r, &plan, giver), expected);
        if initial_flags.contains(&GIVEN) {
            assert_text(
                mail(&mut c, &r, &plan, true),
                "The print shop is across the upper river bridge.",
            );
            assert_eq!(
                quantity(&c, &r, "FLOWER_MAIL"),
                0,
                "pending giver does not duplicate lost mail"
            );
        }
        assert_eq!(c.snapshot().expect("repeat refusal inventory").bag, before);
        assert_eq!(flags(&c), before_flags);
        assert_storage_after_walking(&c, &storage);
        assert_eq!(c.shell.shell.session().state().money, money);
    }
}

#[test]
#[ignore = "requires native Pack medicine and atomic mail reward retry"]
fn minneapolis_expansion_controller_mail_full_reward_retries_through_native_pack() {
    let (root, r, plan, mut c) = fixture(Setup {
        items: &[("FLOWER_MAIL", 1)],
        flags: &[GIVEN],
        full_pocket_except: Some("SUPER_POTION"),
        injury: 30,
    });
    let before = c.snapshot().expect("full reward stack").bag;
    let before_flags = flags(&c);
    assert_text(
        mail(&mut c, &r, &plan, false),
        "Make room for the POTIONS first.",
    );
    assert_eq!(c.snapshot().expect("failed reward is atomic").bag, before);
    assert_eq!(flags(&c), before_flags);
    use_medicine(&mut c, &r, "POTION");
    assert_eq!(quantity(&c, &r, "SUPER_POTION"), 0);
    assert_eq!(quantity(&c, &r, "FLOWER_MAIL"), 1);
    let before_reward = c.shell.shell.session().state().bag.clone();
    assert_text(
        mail(&mut c, &r, &plan, false),
        "The new mill map! Thanks for bringing the print order. Take these POTIONS.",
    );
    assert_eq!(quantity(&c, &r, "SUPER_POTION"), 3);
    assert_eq!(quantity(&c, &r, "FLOWER_MAIL"), 0);
    assert!(event(&c, DELIVERED));
    assert_bag_delta(
        &c,
        &r,
        &before_reward,
        &[("SUPER_POTION", 3), ("FLOWER_MAIL", -1)],
    );
    let mut expected_flags = before_flags;
    expected_flags.0.insert(DELIVERED.to_owned());
    assert_eq!(flags(&c), expected_flags);
    let awarded = c.shell.shell.session().state().bag.clone();
    c = continued(&mut c, &root, &r);
    assert_text(
        mail(&mut c, &r, &plan, false),
        "The order is here. The press is ready.",
    );
    assert_eq!(quantity(&c, &r, "SUPER_POTION"), 3);
    assert_eq!(quantity(&c, &r, "FLOWER_MAIL"), 0);
    assert_bag_delta(&c, &r, &awarded, &[]);
    assert_eq!(flags(&c), expected_flags);
}

#[test]
#[ignore = "requires three actual exterior readings and the museum reward program"]
fn minneapolis_expansion_controller_museum_reads_three_plaques_once_and_continues() {
    let (root, r, plan, mut c) = fixture(Setup::default());
    let storage = c.shell.shell.session().state().storage.clone();
    let money = c.shell.shell.session().state().money;
    let missing = "Three river signs tell the mill story. Read all three, then come back.";
    assert_text(museum(&mut c, &r, &plan), missing);
    for (index, (id, flag, text)) in PLAQUES.iter().enumerate() {
        travel(&mut c, &r, &plan, CITY);
        let sign = plan["signs"]
            .as_array()
            .expect("exterior signs")
            .iter()
            .find(|entry| entry["id"] == *id)
            .expect("exact mill plaque");
        let before_flags = flags(&c);
        interact(&mut c, &r, point(&sign["tile"]));
        assert_text(pages(&mut c, id), text);
        let mut expected = before_flags;
        expected.0.insert((*flag).to_owned());
        assert_eq!(
            flags(&c),
            expected,
            "each actual reading sets only its own flag"
        );
        for (other_index, (_, other, _)) in PLAQUES.iter().enumerate() {
            assert_eq!(event(&c, other), other_index <= index);
        }
        if index < 2 {
            assert_text(museum(&mut c, &r, &plan), missing);
            assert_eq!(quantity(&c, &r, "MAGNET"), 0);
            assert!(!event(&c, TRAIL_REWARD));
        }
    }
    let before_reward = c.shell.shell.session().state().bag.clone();
    let before_reward_flags = flags(&c);
    assert_text(
        museum(&mut c, &r, &plan),
        "You followed the whole mill walk. This MAGNET kept our model turning.",
    );
    assert_bag_delta(&c, &r, &before_reward, &[("MAGNET", 1)]);
    let mut expected_flags = before_reward_flags;
    expected_flags.0.insert(TRAIL_REWARD.to_owned());
    assert_eq!(flags(&c), expected_flags);
    assert_eq!(quantity(&c, &r, "MAGNET"), 1);
    assert!(event(&c, TRAIL_REWARD));
    assert_text(
        museum(&mut c, &r, &plan),
        "The model still turns with help from the river.",
    );
    c = continued(&mut c, &root, &r);
    assert_text(
        museum(&mut c, &r, &plan),
        "The model still turns with help from the river.",
    );
    assert_eq!(quantity(&c, &r, "MAGNET"), 1);
    assert_bag_delta(&c, &r, &before_reward, &[("MAGNET", 1)]);
    assert_eq!(flags(&c), expected_flags);
    assert_storage_after_walking(&c, &storage);
    assert_eq!(c.shell.shell.session().state().money, money);
}

#[test]
#[ignore = "requires native item-pocket capacity and museum full-bag retry"]
fn minneapolis_expansion_controller_museum_full_bag_retries_after_native_medicine() {
    let (root, r, plan, mut c) = fixture(Setup {
        flags: &[PLAQUES[0].1, PLAQUES[1].1, PLAQUES[2].1],
        full_pocket_except: Some("MAGNET"),
        injury: 30,
        ..Default::default()
    });
    let before = c.snapshot().expect("full item pocket").bag;
    let before_flags = flags(&c);
    assert_text(
        museum(&mut c, &r, &plan),
        "Your PACK is full. Come back later.",
    );
    assert_eq!(c.snapshot().expect("museum refused atomically").bag, before);
    assert_eq!(flags(&c), before_flags);
    use_medicine(&mut c, &r, "POTION");
    assert_eq!(
        c.shell.shell.session().state().bag.items.len(),
        ITEM_POCKET_CAPACITY - 1
    );
    let before_reward = c.shell.shell.session().state().bag.clone();
    assert_text(
        museum(&mut c, &r, &plan),
        "You followed the whole mill walk. This MAGNET kept our model turning.",
    );
    assert_eq!(quantity(&c, &r, "MAGNET"), 1);
    assert!(event(&c, TRAIL_REWARD));
    assert_bag_delta(&c, &r, &before_reward, &[("MAGNET", 1)]);
    let mut expected_flags = before_flags;
    expected_flags.0.insert(TRAIL_REWARD.to_owned());
    assert_eq!(flags(&c), expected_flags);
    c = continued(&mut c, &root, &r);
    assert_text(
        museum(&mut c, &r, &plan),
        "The model still turns with help from the river.",
    );
    assert_eq!(quantity(&c, &r, "MAGNET"), 1);
    assert_bag_delta(&c, &r, &before_reward, &[("MAGNET", 1)]);
    assert_eq!(flags(&c), expected_flags);
}

#[test]
#[ignore = "requires the three native infill gifts, capacity refusal and Pack retries"]
fn minneapolis_expansion_controller_three_infill_gifts_retry_once_and_continue() {
    for (map, actor, item, count, flag, text) in [
        (
            "MnMinneapolisToolFlat",
            "MN_MP_TOOLFLAT_HOST",
            "POKE_DOLL",
            1,
            "EVENT_MN_MP_GIFT_TOOLFLAT",
            "A POKE DOLL can help on the road. Keep one for a safe retreat.",
        ),
        (
            "MnMinneapolisRiverFamilyHome",
            "MN_MP_RIVERFAMILYHOME_HOST",
            "MYSTIC_WATER",
            1,
            "EVENT_MN_MP_GIFT_RIVERFAMILYHOME",
            "We found this on the quiet bank. A water partner can hold it.",
        ),
        (
            "MnMinneapolisGardenFlat",
            "MN_MP_GARDENFLAT_HOST",
            "BERRY",
            3,
            "EVENT_MN_MP_GIFT_GARDENFLAT",
            "A little garden feeds our partners. Take some BERRIES for your trip.",
        ),
    ] {
        let (root, r, plan, mut c) = fixture(Setup {
            full_pocket_except: Some(item),
            injury: 30,
            ..Default::default()
        });
        let before = c.snapshot().expect("full gift pocket").bag;
        let before_flags = flags(&c);
        assert_text(
            talk(&mut c, &r, &plan, map, actor),
            "Your PACK is full. Come back later.",
        );
        assert_eq!(
            c.snapshot().expect("gift failure unchanged inventory").bag,
            before
        );
        assert_eq!(flags(&c), before_flags);
        assert!(!event(&c, flag));
        use_medicine(&mut c, &r, "POTION");
        let before_reward = c.shell.shell.session().state().bag.clone();
        assert_text(talk(&mut c, &r, &plan, map, actor), text);
        assert_eq!(quantity(&c, &r, item), count);
        assert_bag_delta(&c, &r, &before_reward, &[(item, count as i16)]);
        let mut expected = before_flags;
        expected.0.insert(flag.to_owned());
        assert_eq!(flags(&c), expected, "gift sets only its own once flag");
        let awarded = c.snapshot().expect("native gift inventory").bag;
        assert_text(
            talk(&mut c, &r, &plan, map, actor),
            "Make good use of it out there.",
        );
        c = continued(&mut c, &root, &r);
        assert_text(
            talk(&mut c, &r, &plan, map, actor),
            "Make good use of it out there.",
        );
        assert_eq!(
            c.snapshot().expect("gift repeat after Continue").bag,
            awarded
        );
        assert_eq!(flags(&c), expected);
    }
}

#[test]
#[ignore = "requires native readable library halves, museum displays and reachable music host"]
fn minneapolis_expansion_controller_library_and_cultural_readings_are_passive() {
    let (root, r, plan, mut c) = fixture(Setup::default());
    let initial = c.snapshot().expect("reading fixture");
    let storage = c.shell.shell.session().state().storage.clone();
    let money = c.shell.shell.session().state().money;
    let initial_flags = flags(&c);
    for map in ["MnMinneapolisCommunityLibrary", MUSEUM] {
        travel(&mut c, &r, &plan, map);
        for sign in room(&plan, map)["signs"]
            .as_array()
            .expect("authored cultural readings")
        {
            let expected = sign["lines"]
                .as_array()
                .expect("authored reading lines")
                .iter()
                .map(|line| line.as_str().expect("reading line"))
                .collect::<Vec<_>>()
                .join(" ");
            interact(&mut c, &r, point(&sign["tile"]));
            assert_text(
                pages(&mut c, sign["id"].as_str().expect("reading ID")),
                &expected,
            );
        }
    }
    assert_text(
        talk(
            &mut c,
            &r,
            &plan,
            "MnMinneapolisMusicRoom",
            "MN_MP_MUSICROOM_HOST",
        ),
        "The river keeps its own rhythm. Our little stage borrows the beat.",
    );
    assert_eq!(
        c.snapshot().expect("passive readings inventory").bag,
        initial.bag
    );
    assert_eq!(flags(&c), initial_flags);
    assert_storage_after_walking(&c, &storage);
    assert_eq!(c.shell.shell.session().state().money, money);
    c = continued(&mut c, &root, &r);
    assert_eq!(flags(&c), initial_flags);
    assert_eq!(
        c.snapshot().expect("Continue passive readings").bag,
        initial.bag
    );
}
