//! Buy two servings from a real vendor, share one and give the other to a partner.
use crystal_assets::AssetRoot;
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleEvent, BattleSide, BattleTurnInput,
};
use crystal_core::input::GameButton::{self, A, Left, Up};
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::models::item::{ITEM_POCKET_ITEM, is_mail_item_id};
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::random::{CrystalRandomState, Random, RuntimeDividerSource};
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::{
    FAIR, buy_units, close_food_pack, close_shop, eat, order_units, press, quantity, target_food,
};
use crate::bevy_shell::tests::interior_life::{collision, fixture, load, walk};
use crate::bevy_shell::{FieldPackAction, mark_runtime_snapshot_dirty};

const FAIR_MENU: &str =
    include_str!("../../../../../planning/minnesota/zones/state-fairgrounds.json");
type Position = (i16, i16);

// The counter displays these prices. The shopper never grants itself an item.
const FAIR_FOODS: &[(&str, u16, u16)] = &[
    ("FAIR_COOKIE_PAIL", 500, 50),
    ("FAIR_CORN_STICK", 300, 20),
    ("FAIR_PRETZEL", 200, 20),
    ("FAIR_CHEESE_CURDS", 400, 30),
    ("FAIR_ROOT_BEER", 500, 60),
    ("FAIR_ICE_CREAM", 450, 50),
    ("FAIR_PICKLE", 100, 10),
    ("FAIR_KETTLE_CORN", 300, 30),
    ("FAIR_HONEY", 250, 20),
    ("FAIR_HOTDISH", 650, 80),
    ("FAIR_PRONTO_PUP", 250, 30),
    ("FAIR_MINI_DONUTS", 350, 50),
    ("FAIR_FUNNEL_CAKE", 500, 60),
    ("FAIR_FRIED_CANDY", 600, 80),
    ("FAIR_WALLEYE", 800, 100),
    ("FAIR_BEER", 500, 100),
];

fn two_servings(
    root: &AssetRoot,
    r: &CrystalRuntime,
    map: &str,
    start: Position,
    approach: Position,
    facing: GameButton,
    (id, price, hp): (&str, u16, u16),
) {
    let mut shopper = fixture(root, r, map, start);
    let item = &r.data().items[id];
    assert_eq!(item.price, price, "{id} posted price");
    assert_eq!(item.held_effect, "HELD_BERRY", "{id} held recovery");
    let mut partner = r
        .data()
        .create_pokemon("POUCHID", 85, Dv::default())
        .expect("regional partner");
    assert!(partner.max_hp > hp + 1);
    partner.hp = 1;
    let state = shopper.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.money = u32::from(price) * 2;
    // Hold the counter's optional wild encounter on a losing roll for this
    // transaction check. The surprise tests exercise the winning lottery.
    state.random_state = CrystalRandomState { add: 0, sub: 128 };
    *shopper.shell.shell.session_mut().divider_mut_for_tests() =
        RuntimeDividerSource::replay(vec![0; 4096]);
    mark_runtime_snapshot_dirty(&mut shopper.shell);
    walk(&mut shopper, r, approach);
    press(&mut shopper, facing);
    press(&mut shopper, A);
    if id == "FAIR_BEER" || id == "FAIR_ROOT_BEER" {
        assert_eq!(
            shopper
                .snapshot()
                .expect("before drinks menu")
                .ui
                .text
                .expect("drink warning")
                .label,
            "MnFairTwoBeersText"
        );
    }
    buy_units(&mut shopper, r, id, 2);
    let bag = shopper.snapshot().expect("two packed servings").bag;
    let mut rows = bag.items.iter().filter(|i| i.item_id == id);
    assert_eq!(rows.next().expect("purchased item row").quantity, 2);
    assert!(rows.next().is_none());
    assert_eq!(shopper.shell.shell.session().state().money, 0);

    eat(&mut shopper, r, id, hp);
    let after = shopper.snapshot().expect("ate first serving");
    assert_eq!(
        after.party.slots[0].pokemon.confusion_turns,
        if id == "FAIR_BEER" { 4 } else { 0 }
    );
    target_food(&mut shopper, id, FieldPackAction::Give);
    close_food_pack(&mut shopper);
    assert_eq!(quantity(&shopper, r, id), 0);
    let state = shopper.shell.shell.session().state();
    let held = state.storage.party.pokemon[0]
        .as_ref()
        .expect("food holder");
    assert_eq!(held.item.as_deref(), Some(id));
    assert_eq!(
        held.hp,
        1 + hp,
        "holding a snack saves the second serving for battle"
    );
    assert_eq!(state.money, 0);
}

#[test]
#[ignore = "requires Fair Table in CRYSTAL_INTERIOR_PACK"]
fn all_sixteen_fair_specialties_can_be_bought_shared_and_held() {
    let (root, runtime, _) = load();
    let menu: Value = serde_json::from_str(FAIR_MENU).expect("authored food vendors");
    let residents = menu["native_map"]["residents"]
        .as_array()
        .expect("Fairgrounds vendors");
    let room = &runtime.data().maps[FAIR];
    for &(id, price, hp) in FAIR_FOODS {
        let vendor = residents
            .iter()
            .find(|r| {
                r["stock"]
                    .as_array()
                    .is_some_and(|stock| stock.iter().any(|item| item == id))
            })
            .expect("a Fair vendor stocks this food");
        let x =
            i16::try_from(vendor["approach"][0].as_i64().expect("counter column")).expect("column");
        let y = i16::try_from(vendor["approach"][1].as_i64().expect("counter row")).expect("row");
        assert_eq!(collision(&runtime, room, (x, y - 1)), Some("COUNTER"));
        two_servings(&root, &runtime, FAIR, (x, y), (x, y), Up, (id, price, hp));
    }
}

#[test]
#[ignore = "requires Fair Table in CRYSTAL_INTERIOR_PACK"]
fn west_seventh_baked_goods_and_duluth_soup_can_be_bought_shared_and_held() {
    let (root, runtime, _) = load();
    for food in [("MN_BUTTER_ROLL", 200, 20), ("MN_CARDAMOM_BUN", 350, 50)] {
        two_servings(
            &root,
            &runtime,
            "MnStPaulWestSeventhBakery",
            (3, 6),
            (3, 3),
            Left,
            food,
        );
    }
    two_servings(
        &root,
        &runtime,
        "MnDuluthCommunityKitchen",
        (8, 10),
        (5, 6),
        Up,
        ("MN_WILD_RICE_SOUP", 500, 60),
    );
}

#[test]
#[ignore = "requires Fair Table in CRYSTAL_INTERIOR_PACK"]
fn every_prepared_food_is_eaten_once_in_a_real_battle_turn() {
    let (_, runtime, _) = load();
    let data = runtime.data();
    let mut holder = data
        .create_pokemon("KEYPOUT", 100, Dv::default())
        .expect("Keypout");
    let mut opponent = data
        .create_pokemon("POUCHID", 100, Dv::default())
        .expect("Pouchid");
    for mon in [&mut holder, &mut opponent] {
        assert!(mon.species.tmhm_learnset.iter().any(|id| id == "PROTECT"));
        mon.moves = vec![LearnedMove {
            name: "PROTECT".into(),
            current_pp: data.moves["PROTECT"].pp,
            pp_ups: 0,
        }];
    }
    let input = BattleTurnInput {
        player: BattleAction::Move { slot: 0 },
        enemy: BattleAction::Move { slot: 0 },
    };
    let local_foods = [
        ("MN_BUTTER_ROLL", 200, 20),
        ("MN_CARDAMOM_BUN", 350, 50),
        ("MN_WILD_RICE_SOUP", 500, 60),
        ("MN_BLUEBERRY_SLICE", 350, 50),
    ];
    for &(id, _, hp) in FAIR_FOODS.iter().chain(&local_foods) {
        holder.hp = 1;
        holder.item = Some(id.into());
        let hp_after = holder.hp + hp;
        assert!(
            hp_after * 2 < holder.max_hp,
            "still eligible next turn: {id}"
        );
        let combat = BattleCombatState::new(holder.clone(), opponent.clone());
        let mut rng = Random::new(47);
        let first = data
            .resolve_battle_turn_with_items(combat, input.clone(), &mut rng)
            .expect("first turn");
        let mut heals = first
            .events
            .iter()
            .filter(|event| matches!(event, BattleEvent::HeldItemHpHealed { .. }));
        assert_eq!(
            heals.next(),
            Some(&BattleEvent::HeldItemHpHealed {
                side: BattleSide::Player,
                item_id: id.into(),
                hp_before: 1,
                hp_after,
                amount: hp,
            })
        );
        assert!(heals.next().is_none(), "{id} healed twice in one turn");
        assert_eq!(first.state.player.hp, hp_after, "{id}");
        assert_eq!(first.state.player.item, None, "{id}");
        assert_eq!(
            first.state.player.confusion_turns,
            if id == "FAIR_BEER" { 4 } else { 0 },
            "{id}"
        );

        let second = data
            .resolve_battle_turn_with_items(first.state, input.clone(), &mut rng)
            .expect("following turn");
        assert_eq!(second.state.player.item, None, "{id}");
        assert!(
            !second
                .events
                .iter()
                .any(|event| matches!(event, BattleEvent::HeldItemHpHealed { .. })),
            "{id} healed again next turn"
        );
        if id == "FAIR_BEER" {
            assert!(second.state.player.hp <= hp_after);
            assert_eq!(second.state.player.confusion_turns, 3);
        } else {
            assert_eq!(second.state.player.hp, hp_after, "{id}");
        }
    }
}

#[test]
#[ignore = "requires Fair Table in CRYSTAL_INTERIOR_PACK"]
fn fair_food_refusals_keep_the_wallet_and_pack_intact() {
    let (root, runtime, _) = load();
    let corn = "FAIR_CORN_STICK";
    let price = runtime.data().items[corn].price;
    for full in [false, true] {
        let mut shopper = fixture(&root, &runtime, FAIR, (59, 12));
        let state = shopper.shell.shell.session_mut().state_mut();
        state.money = u32::from(price);
        if full {
            for (_, filler) in runtime
                .data()
                .items
                .iter()
                .filter(|(id, i)| {
                    id.as_str() != corn && i.pocket == ITEM_POCKET_ITEM && !is_mail_item_id(id)
                })
                .take(ITEM_POCKET_CAPACITY)
            {
                assert!(state.bag.add_item(filler, 1).expect("fill one bag slot"));
            }
            assert_eq!(state.bag.items.len(), ITEM_POCKET_CAPACITY);
        }
        let before = state.bag.clone();
        mark_runtime_snapshot_dirty(&mut shopper.shell);
        press(&mut shopper, Up);
        press(&mut shopper, A);
        order_units(&mut shopper, &runtime, corn, if full { 1 } else { 2 });
        press(&mut shopper, A);
        let notice = shopper.shell.shop_notice.as_ref().expect("clerk's refusal");
        assert!(
            notice
                .to_lowercase()
                .contains(if full { "carry" } else { "money" }),
            "{notice}"
        );
        let after = shopper.shell.shell.session().state();
        assert_eq!(after.money, u32::from(price));
        assert_eq!(after.bag, before);
        close_shop(&mut shopper);
        if !full {
            press(&mut shopper, Up);
            press(&mut shopper, A);
            buy_units(&mut shopper, &runtime, corn, 1);
            assert_eq!(shopper.shell.shell.session().state().money, 0);
            assert_eq!(quantity(&shopper, &runtime, corn), 1);
        }
    }
}
