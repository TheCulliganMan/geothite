//! Production item mutations after explicit party, HP and inventory fixtures.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, GameDataSet, PreparedRootBeerLadyService, ROOT_BEER_FIRST_GIFT_EVENT,
    ROOT_BEER_ITEM_ID, ROOT_BEER_MANIFEST_ID, ROOT_BEER_MART_ID, ROOT_BEER_SERVICE_MANIFEST_ID,
    RootBeerRestPolicy, RuntimeMutationCommand, RuntimeMutationResult, RuntimeScriptCommandRef,
    build_root_beer_lady_modpack, prepare_root_beer_lady_service, read_verified_compiled_game_pack,
};
use crystal_core::battle::turn::active_battle_combat_state;
use crystal_core::models::Dv;
use crystal_core::models::bag::ITEM_POCKET_CAPACITY;
use crystal_core::random::ReplayDivider;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::systems::battle_items::{
    ITEM_EFFECT_BEHAVIOR_RESTORE_HP, active_battle_item_effect_plan,
};
use crystal_core::systems::script_control::ScriptControlAction;
use crystal_core::systems::script_items::ScriptItemGrantOutcome;
use crystal_core::world::encounters::TimeOfDay;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

fn packs() -> &'static (CompiledGamePack, CompiledGamePack) {
    static PACKS: OnceLock<(CompiledGamePack, CompiledGamePack)> = OnceLock::new();
    PACKS.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_ROOT_BEER_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let path = path
            .canonicalize()
            .expect("resolve external native source pack");
        let source = read_verified_compiled_game_pack(path).expect("external native source pack");
        let result = build_root_beer_lady_modpack(&source).expect("additive Root Beer overlay");
        (source, result)
    })
}

fn field_state(missing_hp: u16) -> GameState {
    let data = packs().1.data();
    let mut state = GameState::reset_wram_for_new_game();
    state.overworld = OverworldMemory::Active {
        map_name: "NewBarkTown".into(),
        tile: TilePosition::new(5, 8),
        facing: Direction::Down,
        mode: MovementMode::Normal,
    };
    let mut pokemon = data
        .create_pokemon("MEW", 60, Dv::default())
        .expect("legal fixture partner");
    assert!(pokemon.max_hp > missing_hp);
    pokemon.hp = pokemon.max_hp - missing_hp;
    assert!(state.storage.party.add_pokemon(pokemon));
    state.sync_party_from_storage();
    state
        .bag
        .add_item(&data.items[ROOT_BEER_ITEM_ID], 3)
        .expect("fixture Root Beer inventory");
    state
}

#[test]
fn root_beer_adds_exactly_one_native_drink_and_preserves_all_existing_content() {
    let (base, pack) = packs();
    let drink = &pack.data().items[ROOT_BEER_ITEM_ID];
    let mut expected = base.data().items["LEMONADE"].clone();
    expected.name = "ROOT BEER".into();
    expected.script_name = ROOT_BEER_ITEM_ID.into();
    expected.description = "Restores #MON\nHP by 60.".into();
    expected.parameter = 60;
    expected.price = 300;
    assert_eq!(drink, &expected);
    assert_eq!(drink.effect, "RestoreHPEffect");
    assert_eq!(drink.pocket, "ITEM");
    assert!(drink.field_usable && drink.battle_usable && drink.consumable);
    assert_eq!(drink.field_menu, "ITEMMENU_PARTY");
    assert_eq!(drink.battle_menu, "ITEMMENU_PARTY");
    let plan = active_battle_item_effect_plan(drink).expect("native healing adapter");
    assert_eq!(plan.item_id, ROOT_BEER_ITEM_ID);
    assert_eq!(plan.effect_id, "RestoreHPEffect");
    assert_eq!(plan.behavior_id, ITEM_EFFECT_BEHAVIOR_RESTORE_HP);
    let mut unchanged = pack.data().clone();
    assert_eq!(
        unchanged.items.remove(ROOT_BEER_ITEM_ID).as_ref(),
        Some(drink)
    );
    assert_eq!(
        &unchanged,
        base.data(),
        "all other items/maps/catalogs and scripts must remain exact"
    );
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("overlay audio"),
        base.audio_manifest().expect("base audio")
    );
    let mut report = base.report().clone();
    report.items += 1;
    report.manifests.push(ROOT_BEER_MANIFEST_ID.into());
    assert_eq!(pack.report(), &report);
    let identity = pack.identity().expect("verified new identity");
    let old_identity = base.identity().expect("verified source identity");
    assert_eq!(identity.items, old_identity.items + 1);
    assert_eq!(identity.maps, old_identity.maps);
    assert_ne!(identity.content_hash, old_identity.content_hash);
    assert!(
        build_root_beer_lady_modpack(pack).is_err(),
        "duplicate item overlay must be rejected"
    );
}

#[test]
fn root_beer_field_use_heals_sixty_or_caps_at_max_and_consumes_only_the_new_drink() {
    let data = packs().1.data();
    let drink = &data.items[ROOT_BEER_ITEM_ID];
    for missing in [100, 20] {
        let mut state = field_state(missing);
        state
            .bag
            .add_item(&data.items["LEMONADE"], 2)
            .expect("unrelated original drink");
        state
            .bag
            .add_item(&data.items["FRESH_WATER"], 2)
            .expect("unrelated original water");
        let pokemon = state.storage.party.pokemon[0]
            .as_mut()
            .expect("fixture partner");
        pokemon.status = Some("POISON".into());
        let before = pokemon.clone();
        let (used, effect) = data
            .use_bag_item_on_party_pokemon(&mut state, ROOT_BEER_ITEM_ID, 0, TimeOfDay::Day)
            .expect("authoritative production party-item use");
        let after = state.storage.party.pokemon[0]
            .as_ref()
            .expect("healed partner");
        assert_eq!(used.item_id, ROOT_BEER_ITEM_ID);
        assert!(used.consumed && effect.consumed);
        assert_eq!(effect.item_id, ROOT_BEER_ITEM_ID);
        assert_eq!(effect.hp_before, before.hp);
        assert_eq!(effect.hp_after, before.hp + missing.min(60));
        let mut expected = before.clone();
        expected.hp = effect.hp_after;
        assert_eq!(after, &expected, "Root Beer must only restore HP");
        assert_eq!(state.bag.quantity(drink), 2);
        assert_eq!(state.bag.quantity(&data.items["LEMONADE"]), 2);
        assert_eq!(state.bag.quantity(&data.items["FRESH_WATER"]), 2);
        let event = state
            .script_runtime
            .item_use_events
            .last()
            .expect("native use event");
        assert_eq!(event.item_id, ROOT_BEER_ITEM_ID);
        assert!(event.consumed);
    }
}

#[test]
fn root_beer_invalid_or_unchanged_targets_preserve_hp_inventory_and_state() {
    let data = packs().1.data();
    for case in [
        "full_hp",
        "fainted",
        "egg",
        "empty_slot",
        "outside_party",
        "not_held",
    ] {
        let mut state = field_state(if case == "full_hp" { 0 } else { 100 });
        let target = match case {
            "fainted" => {
                state.storage.party.pokemon[0]
                    .as_mut()
                    .expect("fixture partner")
                    .hp = 0;
                0
            }
            "egg" => {
                state.storage.party.pokemon[0]
                    .as_mut()
                    .expect("fixture partner")
                    .is_egg = true;
                0
            }
            "empty_slot" => 1,
            "outside_party" => 99,
            "not_held" => {
                assert!(
                    state
                        .bag
                        .remove_item(&data.items[ROOT_BEER_ITEM_ID], 3)
                        .expect("empty fixture drink stack")
                );
                0
            }
            _ => 0,
        };
        state.sync_party_from_storage();
        let before = state.clone();
        assert!(
            data.use_bag_item_on_party_pokemon(
                &mut state,
                ROOT_BEER_ITEM_ID,
                target,
                TimeOfDay::Day
            )
            .is_err(),
            "{case}"
        );
        assert_eq!(
            state, before,
            "failed {case} use must not consume or mutate anything"
        );
    }
}

#[test]
fn root_beer_battle_party_use_heals_active_and_reserve_through_native_mutation() {
    let data = packs().1.data();
    let mut state = field_state(100);
    state.overworld = OverworldMemory::Active {
        map_name: "Route36".into(),
        tile: TilePosition::new(20, 8),
        facing: Direction::Down,
        mode: MovementMode::Normal,
    };
    let mut reserve = data
        .create_pokemon("LAPRAS", 55, Dv::default())
        .expect("reserve partner");
    reserve.hp = reserve.max_hp - 100;
    assert!(state.storage.party.add_pokemon(reserve));
    state.sync_party_from_storage();
    data.start_scripted_wild_battle(
        &mut state,
        "Route36",
        "Route36",
        "WateredWeirdTreeScript",
        12,
        &mut ReplayDivider::new([0; 64]),
    )
    .expect("actual packed Sudowoodo battle and active combat");
    for index in [0, 1] {
        let before = state.storage.party.pokemon[index]
            .as_ref()
            .expect("native battle target")
            .hp;
        let (used, effect) = data
            .use_bag_item_on_battle_party_pokemon(&mut state, ROOT_BEER_ITEM_ID, index)
            .expect("native battle party-item mutation");
        assert!(used.consumed && effect.consumed);
        assert_eq!(used.item_id, ROOT_BEER_ITEM_ID);
        assert_eq!(effect.hp_after, before + 60);
        let after = state.storage.party.pokemon[index]
            .as_ref()
            .expect("healed battle target")
            .hp;
        assert_eq!(after, before + 60);
        let combat = active_battle_combat_state(&state).expect("updated authoritative combat");
        assert_eq!(combat.player_party[index].hp, after);
        if index == 0 {
            assert_eq!(combat.player.hp, after);
        } else {
            assert_eq!(
                combat.player.hp, combat.player_party[0].hp,
                "reserve healing must preserve the active partner"
            );
        }
    }
    assert_eq!(state.bag.quantity(&data.items[ROOT_BEER_ITEM_ID]), 1);
}

const SERVICE_FIXTURE_MAP: &str = "Route27SandstormHouse";
const SERVICE_FIXTURE_ACTOR: &str = "ROUTE27SANDSTORMHOUSE_GRANNY";

// An existing native house and actor are explicit component fixtures only. No
// Isle of Pines exterior, cabin, gift or shop is published by these tests.
fn service_fixture() -> &'static (PreparedRootBeerLadyService, GameDataSet) {
    static SERVICE: OnceLock<(PreparedRootBeerLadyService, GameDataSet)> = OnceLock::new();
    SERVICE.get_or_init(|| {
        let pack = &packs().1;
        let prepared = prepare_root_beer_lady_service(
            pack,
            pack.data()
                .map_module(SERVICE_FIXTURE_MAP)
                .expect("existing native house fixture"),
            SERVICE_FIXTURE_ACTOR,
            RootBeerRestPolicy::Offer,
        )
        .expect("prepare native service on an actual actor");
        let mut data = prepared.pack.data().clone();
        data.maps
            .insert(prepared.module.id.clone(), prepared.module.clone());
        (prepared, data)
    })
}

fn service_state() -> GameState {
    let mut state = field_state(100);
    state.bag.items.clear();
    state.overworld = OverworldMemory::Active {
        map_name: SERVICE_FIXTURE_MAP.into(),
        tile: TilePosition::new(2, 5),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
    state
}

fn fill_item_pocket(data: &GameDataSet, state: &mut GameState) {
    state.bag.items.clear();
    for item in data
        .items
        .values()
        .filter(|item| item.pocket == "ITEM" && item.script_name != ROOT_BEER_ITEM_ID)
        .take(ITEM_POCKET_CAPACITY)
    {
        assert!(state.bag.add_item(item, 1).expect("full-bag fixture"));
    }
    assert_eq!(state.bag.items.len(), ITEM_POCKET_CAPACITY);
}

#[test]
fn dorothy_service_prepares_native_commands_without_publishing_or_changing_existing_maps() {
    let source = &packs().1;
    let (prepared, _) = service_fixture();
    assert_eq!(prepared.pack.data().maps, source.data().maps);
    assert_eq!(prepared.pack.runtime_files(), source.runtime_files());
    assert_eq!(prepared.pack.compiled_audio(), source.compiled_audio());
    assert_eq!(prepared.pack.data().items, source.data().items);
    let mut unchanged = prepared.pack.data().clone();
    assert_eq!(
        unchanged.marts.0.remove(ROOT_BEER_MART_ID),
        Some(vec![ROOT_BEER_ITEM_ID.into()])
    );
    assert!(
        unchanged
            .story_event_script_constants
            .global
            .remove(ROOT_BEER_FIRST_GIFT_EVENT)
            .is_some()
    );
    assert_eq!(&unchanged, source.data());
    let mut report = source.report().clone();
    report.manifests.push(ROOT_BEER_SERVICE_MANIFEST_ID.into());
    assert_eq!(prepared.pack.report(), &report);

    let original = source
        .data()
        .map_module(SERVICE_FIXTURE_MAP)
        .expect("native fixture module");
    assert_eq!(prepared.module.blocks, original.blocks);
    assert_eq!(prepared.module.attributes, original.attributes);
    assert_eq!(prepared.module.events, original.events);
    let mut expected_actors = original.objects.clone();
    expected_actors[0].script = prepared.script_name.clone();
    assert_eq!(prepared.module.objects, expected_actors);
    for (name, commands) in &original.scripts {
        assert_eq!(prepared.module.scripts.get(name), Some(commands));
    }
    let gift = prepared
        .module
        .script_item_grants
        .iter()
        .find(|grant| grant.source_script == prepared.script_name)
        .expect("native giveitem parser");
    assert_eq!(gift.item_id, ROOT_BEER_ITEM_ID);
    assert_eq!(gift.quantity, 1);
    assert_eq!(gift.command_index, 7);
    let mart = prepared
        .module
        .script_shop_commands
        .iter()
        .find(|command| command.source_script == format!("{}Shop", prepared.script_name))
        .expect("native pokemart parser");
    assert_eq!(mart.mart_id, ROOT_BEER_MART_ID);
    assert_eq!(mart.command_index, 5);
    let service_runtime = prepared
        .module
        .script_runtime_commands
        .iter()
        .filter(|command| command.source_script.starts_with(&prepared.script_name))
        .collect::<Vec<_>>();
    let specials = service_runtime
        .iter()
        .filter(|command| command.command == "special")
        .collect::<Vec<_>>();
    assert_eq!(specials.len(), 1);
    assert_eq!(specials[0].args, ["HealParty"]);
    assert!(
        prepared
            .module
            .scripts
            .iter()
            .filter(|(name, _)| name.starts_with(&prepared.script_name))
            .all(|(_, commands)| !commands.to_string().to_ascii_lowercase().contains("safari")),
        "hospitality scripts must never renew Safari admission, balls or steps"
    );
}

#[test]
fn dorothy_first_gift_native_failure_guard_preserves_offer_then_delivery_sets_saved_flag() {
    let (prepared, data) = service_fixture();
    let script = &prepared.script_name;
    let mut state = service_state();
    fill_item_pocket(data, &mut state);
    let mut session = data
        .overworld_session(SERVICE_FIXTURE_MAP, TilePosition::new(2, 5), 0)
        .expect("native service session fixture");
    let audio_ids = data.audio_ids().into_iter().map(str::to_owned).collect();
    let before_bag = state.bag.clone();
    let before_money = state.money;
    let before_flags = state.flags.clone();
    let failed = data
        .apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::GrantScriptItem(RuntimeScriptCommandRef::new(
                SERVICE_FIXTURE_MAP,
                script,
                7,
            )),
            &audio_ids,
            &audio_ids,
            &audio_ids,
        )
        .expect("native giveitem full-pocket result");
    assert!(matches!(
        failed.result,
        RuntimeMutationResult::ScriptItemGranted(ScriptItemGrantOutcome::BagFull { .. })
    ));
    assert_eq!(state.script_runtime.script_value.as_deref(), Some("0"));
    assert_eq!(state.bag, before_bag);
    assert_eq!(state.money, before_money);
    assert_eq!(state.flags, before_flags);
    assert!(matches!(
        data.apply_script_control_command(
            &mut state, SERVICE_FIXTURE_MAP, SERVICE_FIXTURE_MAP, script, 8,
        )
        .expect("native iffalse after failed giveitem"),
        ScriptControlAction::Jump { target_script, .. }
            if target_script == format!("{script}BagFull")
    ));
    assert_eq!(
        state.flags.is_event_flag_set(ROOT_BEER_FIRST_GIFT_EVENT),
        Ok(false)
    );

    // Explicitly free one fixture slot, then retry the same production grant.
    let freed = state
        .bag
        .items
        .keys()
        .next()
        .expect("full pocket item")
        .clone();
    assert!(
        state
            .bag
            .remove_item(&data.items[&freed], 1)
            .expect("free slot")
    );
    let delivered = data
        .apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::GrantScriptItem(RuntimeScriptCommandRef::new(
                SERVICE_FIXTURE_MAP,
                script,
                7,
            )),
            &audio_ids,
            &audio_ids,
            &audio_ids,
        )
        .expect("native successful giveitem");
    assert!(matches!(
        delivered.result,
        RuntimeMutationResult::ScriptItemGranted(ScriptItemGrantOutcome::Granted {
            quantity: 1,
            ..
        })
    ));
    assert_eq!(state.bag.quantity(&data.items[ROOT_BEER_ITEM_ID]), 1);
    assert_eq!(state.script_runtime.script_value.as_deref(), Some("1"));
    assert!(matches!(
        data.apply_script_control_command(
            &mut state,
            SERVICE_FIXTURE_MAP,
            SERVICE_FIXTURE_MAP,
            script,
            8,
        )
        .expect("native successful-gift fallthrough"),
        ScriptControlAction::Continue { .. }
    ));
    data.apply_script_flag_mutation(
        &mut state,
        &mut session,
        SERVICE_FIXTURE_MAP,
        SERVICE_FIXTURE_MAP,
        script,
        9,
    )
    .expect("native saved first-gift event");
    assert_eq!(
        state.flags.is_event_flag_set(ROOT_BEER_FIRST_GIFT_EVENT),
        Ok(true)
    );
    data.check_script_flag(
        &mut state,
        SERVICE_FIXTURE_MAP,
        SERVICE_FIXTURE_MAP,
        script,
        2,
    )
    .expect("next visit native checkevent");
    assert!(matches!(
        data.apply_script_control_command(
            &mut state, SERVICE_FIXTURE_MAP, SERVICE_FIXTURE_MAP, script, 3,
        )
        .expect("next visit skips another free bottle"),
        ScriptControlAction::Jump { target_script, .. }
            if target_script == format!("{script}Choices")
    ));
    assert_eq!(state.bag.quantity(&data.items[ROOT_BEER_ITEM_ID]), 1);
}

#[test]
fn dorothy_native_standard_mart_charges_three_hundred_and_refuses_atomically() {
    let (prepared, data) = service_fixture();
    let mut state = service_state();
    state.money = 900;
    data.open_script_shop(
        &mut state,
        SERVICE_FIXTURE_MAP,
        SERVICE_FIXTURE_MAP,
        &format!("{}Shop", prepared.script_name),
        5,
    )
    .expect("native standard mart opens dedicated inventory");
    assert_eq!(
        crystal_core::systems::shop::max_buy_quantity(&state, &data.items[ROOT_BEER_ITEM_ID]),
        3
    );
    assert!(
        data.buy_shop_item(&mut state, ROOT_BEER_ITEM_ID, 2)
            .expect("buy two")
            .success
    );
    assert_eq!(state.money, 300);
    assert_eq!(state.bag.quantity(&data.items[ROOT_BEER_ITEM_ID]), 2);
    let before = (state.money, state.bag.clone(), state.flags.clone());
    assert!(
        !data
            .buy_shop_item(&mut state, ROOT_BEER_ITEM_ID, 2)
            .expect("insufficient funds")
            .success
    );
    assert_eq!(
        (state.money, state.bag.clone(), state.flags.clone()),
        before
    );
    state.money = 900;
    fill_item_pocket(data, &mut state);
    let before = (state.money, state.bag.clone(), state.flags.clone());
    assert!(
        !data
            .buy_shop_item(&mut state, ROOT_BEER_ITEM_ID, 1)
            .expect("full pocket")
            .success
    );
    assert_eq!(
        (state.money, state.bag.clone(), state.flags.clone()),
        before
    );
}

#[test]
fn dorothy_optional_native_heal_party_rest_preserves_safari_and_unrelated_resources() {
    let (_, data) = service_fixture();
    let mut state = service_state();
    state.money = 1_234;
    let partner = state.storage.party.pokemon[0]
        .as_mut()
        .expect("partner fixture");
    partner.status = Some("POISON".into());
    for learned in &mut partner.moves {
        learned.current_pp = 0;
    }
    state.sync_party_from_storage();
    state
        .flags
        .set_engine_flag("ENGINE_SAFARI_ZONE", true)
        .expect("active Safari fixture");
    for (name, value) in [("wSafariBalls", "7"), ("wSafariSteps", "123")] {
        state
            .script_runtime
            .memory
            .insert(name.into(), value.into());
    }
    let before = (
        state.money,
        state.bag.clone(),
        state.flags.clone(),
        state.script_runtime.memory.clone(),
    );
    data.apply_special_routine(&mut state, "HealParty", &BTreeSet::new())
        .expect("actual native rest routine");
    let partner = state.storage.party.pokemon[0]
        .as_ref()
        .expect("rested partner");
    assert_eq!(partner.hp, partner.max_hp);
    assert_eq!(partner.status, None);
    assert!(partner.moves.iter().all(|learned| learned.current_pp > 0));
    assert_eq!(
        (
            state.money,
            state.bag.clone(),
            state.flags.clone(),
            state.script_runtime.memory.clone()
        ),
        before
    );

    let source = &packs().1;
    let without_rest = prepare_root_beer_lady_service(
        source,
        source
            .data()
            .map_module(SERVICE_FIXTURE_MAP)
            .expect("native fixture"),
        SERVICE_FIXTURE_ACTOR,
        RootBeerRestPolicy::Omit,
    )
    .expect("native gift/shop without resting");
    assert!(
        without_rest
            .module
            .script_runtime_commands
            .iter()
            .all(|command| {
                !command.source_script.starts_with(&without_rest.script_name)
                    || command.command != "special"
            })
    );
}

#[test]
fn dorothy_preparation_rejects_missing_item_wrong_actor_and_invalid_or_overlapping_anchors() {
    let source = &packs().1;
    let original = source
        .data()
        .map_module(SERVICE_FIXTURE_MAP)
        .expect("native fixture");
    assert!(
        prepare_root_beer_lady_service(
            &packs().0,
            original,
            SERVICE_FIXTURE_ACTOR,
            RootBeerRestPolicy::Omit
        )
        .is_err()
    );
    assert!(
        prepare_root_beer_lady_service(
            source,
            original,
            "MISSING_DOROTHY",
            RootBeerRestPolicy::Omit
        )
        .is_err()
    );
    for case in [
        "wrong_sprite",
        "outside",
        "wall",
        "overlap",
        "malformed_dimensions",
    ] {
        let mut module = original.clone();
        match case {
            "wrong_sprite" => module.objects[0].sprite = "SPRITE_FISHER".into(),
            "outside" => module.objects[0].x = 99,
            "wall" => module.objects[0].x = 3,
            "overlap" => module.objects.push(module.objects[0].clone()),
            "malformed_dimensions" => module.attributes.width = u16::MAX,
            _ => unreachable!("listed fixture case"),
        }
        assert!(
            prepare_root_beer_lady_service(
                source,
                &module,
                SERVICE_FIXTURE_ACTOR,
                RootBeerRestPolicy::Omit
            )
            .is_err(),
            "{case}"
        );
    }
    let (prepared, _) = service_fixture();
    assert!(
        prepare_root_beer_lady_service(
            &prepared.pack,
            &prepared.module,
            SERVICE_FIXTURE_ACTOR,
            RootBeerRestPolicy::Offer
        )
        .is_err(),
        "duplicate script installation"
    );
}
