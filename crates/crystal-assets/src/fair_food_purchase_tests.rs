use std::sync::OnceLock;

use crystal_core::random::{CrystalRandomState, ReplayDivider};
use crystal_core::state::{BattleMemory, GameState};
use crystal_core::systems::fair_food::{FAIR_MAP, PendingFairFood, species_for_purchase};
use crystal_core::systems::shop::{apply_script_shop_command, close_active_shop};
use crystal_core::world::map::TilePosition;
use crystal_core::world::session::OverworldSession;

use crate::{CompiledGamePack, GameDataSet, read_verified_compiled_game_pack};

fn saved_food(state: &GameState) -> Option<PendingFairFood> {
    PendingFairFood::read(state).expect("valid saved food extension")
}

fn binary_roundtrip(state: &GameState, stage: &str) -> GameState {
    use crystal_core::save::{
        SaveModpackIdentity, erase_save_game, read_save_game_for_modpack,
        write_save_game_for_modpack,
    };
    let identity =
        SaveModpackIdentity::new("fair_food_test", "1".repeat(64)).expect("test identity");
    let content_hash = "2".repeat(64);
    let path = std::env::temp_dir().join(format!(
        "fair-food-binary-{stage}-{}.crystalsave",
        std::process::id()
    ));
    write_save_game_for_modpack(&path, state.clone(), &identity, &content_hash)
        .expect("production binary save");
    let restored = read_save_game_for_modpack(&path, &identity, &content_hash)
        .expect("production binary Continue")
        .into_state();
    erase_save_game(path).expect("remove disposable save generations");
    restored
}

fn fixture(item: &str, party: bool) -> (&'static GameDataSet, GameState, OverworldSession) {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    let data = &PACK
        .get_or_init(|| {
            read_verified_compiled_game_pack(
                std::env::var("CRYSTAL_FAIR_TEST_PACK").expect("explicit Fair pack"),
            )
            .expect("verified Fair pack")
        })
        .data;
    let (mut state, session) = data
        .start_overworld_session_at_runtime_tile(
            FAIR_MAP,
            TilePosition { x: 67, y: 12 },
            &data.audio_ids().into_iter().map(str::to_string).collect(),
        )
        .expect("explicit Fair placement");
    state.player_name = "CHRIS".into();
    state.money = 100_000;
    state.random_state = CrystalRandomState { add: 0, sub: 49 };
    if party {
        let pokemon = data
            .create_pokemon("POUCHID", 80, Default::default())
            .expect("healthy partner");
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
    }
    let command = data
        .map_module(FAIR_MAP)
        .expect("Fair")
        .script_shop_commands
        .iter()
        .find(|command| {
            data.marts
                .inventory_ids(&command.mart_id)
                .is_ok_and(|stock| stock.iter().any(|id| id == item))
        })
        .expect("authored food counter")
        .clone();
    apply_script_shop_command(&mut state, &data.marts, &data.items, command)
        .expect("open authored counter");
    (data, state, session)
}

fn random_trace() -> ReplayDivider {
    let bytes = [0, 0]
        .into_iter()
        .chain((0..4096).scan(0x9e3779b9_u32, |seed, _| {
            *seed ^= *seed << 13;
            *seed ^= *seed >> 17;
            *seed ^= *seed << 5;
            Some((*seed >> 16) as u8)
        }));
    ReplayDivider::new(bytes)
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK"]
fn fair_food_purchase_bulk_accounting_pending_save_and_single_consumption() {
    let (data, mut state, session) = fixture("FAIR_CHEESE_CURDS", true);
    let before = state.clone();
    let ordinary = binary_roundtrip(&state, "ordinary");
    assert!(saved_food(&ordinary).is_none());
    assert_eq!(ordinary.money, state.money);
    assert_eq!(ordinary.bag, state.bag);
    let mut rng = random_trace();
    let result = data
        .buy_shop_item_with_surprise(&mut state, &session, "FAIR_CHEESE_CURDS", 5, &mut rng)
        .expect("purchase");
    assert!(result.success);
    assert_eq!(state.money, 98_000);
    assert_eq!(state.bag.quantity(&data.items["FAIR_CHEESE_CURDS"]), 5);
    let pending = saved_food(&state).expect("winning reveal");
    assert_eq!(pending.battle.enemy_pokemon.species.id, "CURDLET");
    assert_eq!(
        pending.battle.enemy_pokemon.level, 10,
        "level-80 partner does not scale the basic form"
    );
    assert!(matches!(state.battle, BattleMemory::Inactive));
    let binary_restored = binary_roundtrip(&state, "pending");
    assert_eq!(saved_food(&binary_restored), Some(pending.clone()));
    assert_eq!(binary_restored.money, state.money);
    assert_eq!(binary_restored.bag, state.bag);
    assert_eq!(binary_restored.random_state, state.random_state);
    let encoded = serde_json::to_string(&state).expect("serialize pending purchase");
    let mut restored: GameState =
        serde_json::from_str(&encoded).expect("continue pending purchase");
    assert_eq!(saved_food(&restored), Some(pending.clone()));
    assert_eq!(restored.random_state, state.random_state);
    assert!(
        data.start_pending_fair_food_battle(&mut restored, &session)
            .is_err(),
        "shop must close first"
    );
    assert_eq!(saved_food(&restored), Some(pending.clone()));
    assert!(
        data.buy_shop_item_with_surprise(
            &mut restored,
            &session,
            "FAIR_CHEESE_CURDS",
            1,
            &mut ReplayDivider::new([])
        )
        .is_err()
    );
    assert_eq!(
        restored.money, 98_000,
        "pending surprise cannot be charged twice"
    );
    close_active_shop(&mut restored).expect("close shop");
    restored.script_runtime.text_window_open = false;
    restored.script_runtime.next_script = None;
    assert!(
        data.start_pending_fair_food_battle(&mut restored, &session)
            .expect("start battle")
    );
    assert!(saved_food(&restored).is_none());
    assert!(matches!(restored.battle, BattleMemory::Wild { .. }));
    assert!(
        !data
            .start_pending_fair_food_battle(&mut restored, &session)
            .expect("repeated start")
    );
    let battle_saved = serde_json::to_string(&restored).expect("save active battle");
    let continued: GameState = serde_json::from_str(&battle_saved).expect("continue active battle");
    assert_eq!(continued.battle, restored.battle);
    assert_eq!(continued.money, 98_000);
    let mut one = before;
    let mut one_rng = random_trace();
    data.buy_shop_item_with_surprise(&mut one, &session, "FAIR_CHEESE_CURDS", 1, &mut one_rng)
        .expect("single unit");
    assert_eq!(
        one_rng.consumed(),
        rng.consumed(),
        "bulk quantity gets one roll and one materialization"
    );
    assert_eq!(saved_food(&one), Some(pending));
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK"]
fn fair_food_purchase_failures_cancel_full_bag_and_rng_failure_are_atomic() {
    let (data, mut state, session) = fixture("FAIR_CHEESE_CURDS", true);
    let original = state.clone();
    let mut empty = ReplayDivider::new([]);
    assert!(
        data.buy_shop_item_with_surprise(&mut state, &session, "FAIR_CHEESE_CURDS", 0, &mut empty)
            .is_err()
    );
    assert_eq!(state, original);
    state.money = 399;
    assert!(
        !data
            .buy_shop_item_with_surprise(&mut state, &session, "FAIR_CHEESE_CURDS", 1, &mut empty)
            .expect("insufficient money")
            .success
    );
    assert_eq!(state.money, 399);
    assert_eq!(empty.consumed(), 0);
    state = original.clone();
    for _ in 0..crystal_core::models::bag::ITEM_POCKET_CAPACITY {
        assert!(
            state
                .bag
                .add_item(&data.items["FAIR_CHEESE_CURDS"], 99)
                .expect("fill a bag slot")
        );
    }
    assert!(
        !data
            .buy_shop_item_with_surprise(&mut state, &session, "FAIR_CHEESE_CURDS", 1, &mut empty)
            .expect("full bag purchase")
            .success
    );
    assert_eq!(state.money, original.money);
    assert_eq!(empty.consumed(), 0);
    state = original.clone();
    assert!(
        data.buy_shop_item_with_surprise(&mut state, &session, "FAIR_CHEESE_CURDS", 1, &mut empty)
            .is_err(),
        "missing RNG trace rejects atomically"
    );
    assert_eq!(state, original);
    close_active_shop(&mut state).expect("cancel browsing");
    assert_eq!(state.money, original.money);
    assert_eq!(empty.consumed(), 0);
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK"]
fn fair_food_purchase_losing_roll_no_party_and_ordinary_purchases() {
    let (data, mut state, session) = fixture("FAIR_CHEESE_CURDS", false);
    let mut rng = ReplayDivider::new([0, 0]);
    assert!(
        data.buy_shop_item_with_surprise(&mut state, &session, "FAIR_CHEESE_CURDS", 2, &mut rng)
            .expect("no party food")
            .success
    );
    assert_eq!(state.money, 99_200);
    assert!(saved_food(&state).is_none());
    assert_eq!(rng.consumed(), 2);
    let (_, mut state, _) = fixture("FAIR_CHEESE_CURDS", true);
    state.random_state.sub = 50;
    let mut rng = ReplayDivider::new([0, 0]);
    assert!(
        data.buy_shop_item_with_surprise(&mut state, &session, "FAIR_CHEESE_CURDS", 1, &mut rng)
            .expect("losing roll")
            .success
    );
    assert!(saved_food(&state).is_none());
    assert_eq!(rng.consumed(), 2);
    let (_, mut state, _) = fixture("POTION", true);
    let before_rng = state.random_state;
    let mut rng = ReplayDivider::new([]);
    assert!(
        data.buy_shop_item_with_surprise(&mut state, &session, "POTION", 2, &mut rng)
            .expect("ordinary Fair shop")
            .success
    );
    assert_eq!(state.random_state, before_rng);
    assert_eq!(rng.consumed(), 0);
    assert!(saved_food(&state).is_none());
    let (_, mut state, mut session) = fixture("FAIR_CHEESE_CURDS", true);
    session.map.name = "CherrygroveMart".into();
    assert!(
        data.buy_shop_item_with_surprise(&mut state, &session, "FAIR_CHEESE_CURDS", 1, &mut rng)
            .expect("non-Fair item")
            .success
    );
    assert_eq!(rng.consumed(), 0);
    assert!(saved_food(&state).is_none());
    for command in &data
        .map_module(FAIR_MAP)
        .expect("Fair")
        .script_shop_commands
    {
        for item in data
            .marts
            .inventory_ids(&command.mart_id)
            .expect("authored mart")
        {
            if let Some(species) = species_for_purchase(FAIR_MAP, &command.mart_id, item) {
                assert!(data.pokemon.contains_key(species));
                assert!(data.saved_wild_encounter_exists(FAIR_MAP, species, 10));
            }
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK"]
fn fair_food_purchase_all_mappings_and_party_lost_before_reveal() {
    for (item, species) in [
        ("FAIR_CHEESE_CURDS", "CURDLET"),
        ("FAIR_COOKIE_PAIL", "DOUGHLIN"),
        ("FAIR_KETTLE_CORN", "PIPCORN"),
        ("FAIR_HOTDISH", "TOTLIT"),
    ] {
        let (data, mut state, session) = fixture(item, true);
        data.buy_shop_item_with_surprise(&mut state, &session, item, 1, &mut random_trace())
            .expect("mapped purchase");
        let pending = saved_food(&state).expect("winning food roll");
        assert_eq!(pending.battle.enemy_pokemon.species.id, species);
        assert_eq!(pending.battle.enemy_pokemon.level, 10);
        state
            .storage
            .party
            .pokemon
            .iter_mut()
            .flatten()
            .for_each(|pokemon| pokemon.hp = 0);
        close_active_shop(&mut state).expect("close food shop");
        state.script_runtime.text_window_open = false;
        state.script_runtime.next_script = None;
        let money = state.money;
        assert!(
            !data
                .start_pending_fair_food_battle(&mut state, &session)
                .expect("no usable partner")
        );
        assert!(saved_food(&state).is_none());
        assert_eq!(state.money, money);
        assert_eq!(state.bag.quantity(&data.items[item]), 1);
        assert!(matches!(state.battle, BattleMemory::Inactive));
    }
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK"]
fn fair_food_purchase_capture_uses_normal_storage_and_no_extra_food_or_charge() {
    let (data, mut state, session) = fixture("FAIR_CHEESE_CURDS", true);
    assert!(
        state
            .bag
            .add_item(&data.items["MASTER_BALL"], 1)
            .expect("capture fixture ball")
    );
    data.buy_shop_item_with_surprise(
        &mut state,
        &session,
        "FAIR_CHEESE_CURDS",
        1,
        &mut random_trace(),
    )
    .expect("purchase");
    close_active_shop(&mut state).expect("close food shop");
    state.script_runtime.text_window_open = false;
    state.script_runtime.next_script = None;
    assert!(
        data.start_pending_fair_food_battle(&mut state, &session)
            .expect("start")
    );
    let money = state.money;
    let outcome = data
        .throw_ball_at_active_battle_with_divider(&mut state, "MASTER_BALL", &mut random_trace())
        .expect("ordinary ball throw");
    assert!(outcome.caught && !outcome.blocked);
    data.complete_active_wild_capture(&mut state, &outcome, None, &mut random_trace())
        .expect("ordinary capture completion");
    assert!(
        state
            .storage
            .party
            .pokemon
            .iter()
            .flatten()
            .any(|pokemon| pokemon.species.id == "CURDLET" && pokemon.level == 10)
    );
    assert!(matches!(state.battle, BattleMemory::Inactive));
    assert_eq!(state.bag.quantity(&data.items["MASTER_BALL"]), 0);
    assert_eq!(state.bag.quantity(&data.items["FAIR_CHEESE_CURDS"]), 1);
    assert_eq!(state.money, money);
    assert!(
        !data
            .start_pending_fair_food_battle(&mut state, &session)
            .expect("capture cannot replay surprise")
    );
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK"]
fn fair_food_purchase_real_victory_and_blackout_do_not_repeat_purchase() {
    use crate::RuntimeMutationCommand;
    use crystal_core::battle::turn::BattleAction;

    for lose in [false, true] {
        let (data, mut state, mut session) = fixture("FAIR_CHEESE_CURDS", true);
        if lose {
            let mut partner = data
                .create_pokemon("POUCHID", 1, Default::default())
                .expect("weak legal partner");
            partner.hp = 1;
            for known in &mut partner.moves {
                known.current_pp = 0;
            }
            state.storage.party.pokemon[0] = Some(partner);
            state.sync_party_from_storage();
        }
        data.buy_shop_item_with_surprise(
            &mut state,
            &session,
            "FAIR_CHEESE_CURDS",
            1,
            &mut random_trace(),
        )
        .expect("purchase");
        close_active_shop(&mut state).expect("shop exit");
        state.script_runtime.text_window_open = false;
        state.script_runtime.next_script = None;
        assert!(
            data.start_pending_fair_food_battle(&mut state, &session)
                .expect("food battle")
        );
        let mut rng = random_trace();
        let player_slot = state.storage.party.pokemon[0]
            .as_ref()
            .expect("partner")
            .moves
            .iter()
            .enumerate()
            .max_by_key(|(_, known)| data.moves[&known.name].power)
            .map(|(index, _)| index)
            .expect("legal moves");
        for _ in 0..32 {
            let turn = data
                .resolve_active_battle_turn_with_divider(
                    &mut state,
                    BattleAction::Move { slot: player_slot },
                    BattleAction::Move { slot: 0 },
                    &mut rng,
                )
                .expect("real battle turn");
            if turn.state.player.hp == 0 || turn.state.enemy.hp == 0 {
                break;
            }
        }
        if lose {
            assert_eq!(
                state.storage.party.pokemon[0]
                    .as_ref()
                    .expect("fainted partner")
                    .hp,
                0
            );
            assert!(
                matches!(state.battle, BattleMemory::Inactive),
                "normal loss cleanup"
            );
            assert_eq!(state.battle_result & 0x3f, 1);
            let ids = data.audio_ids().into_iter().map(str::to_string).collect();
            data.apply_runtime_mutation_command(
                &mut state,
                &mut session,
                RuntimeMutationCommand::ResolveBlackoutToLastSpawn,
                &ids,
                &ids,
                &ids,
            )
            .expect("normal blackout recovery");
            assert!(
                state.storage.party.pokemon[0]
                    .as_ref()
                    .expect("healed at spawn")
                    .hp
                    > 0
            );
        } else {
            data.claim_active_wild_battle_rewards_now(&mut state, &mut rng)
                .expect("normal victory rewards");
            assert!(matches!(state.battle, BattleMemory::Inactive));
            assert_eq!(
                state.money, 99_600,
                "wild victory cannot reapply shop payment"
            );
        }
        assert!(saved_food(&state).is_none());
        assert_eq!(state.bag.quantity(&data.items["FAIR_CHEESE_CURDS"]), 1);
        assert!(
            !data
                .start_pending_fair_food_battle(&mut state, &session)
                .expect("terminal does not repeat")
        );
    }
}
