use super::*;
use crate::day_care::{BEMIDJI_DAY_CARE, MANKATO_DAY_CARE};

fn pokemon(id: &str) -> Pokemon {
    let mut partner = super::pokemon(id);
    partner.species.int_id = match id {
        "CHIKORITA" => 152,
        "CYNDAQUIL" => 155,
        "TOTODILE" => 158,
        _ => 1,
    };
    partner
}

fn visit(state: &mut GameState, map: &str) {
    state.overworld = OverworldMemory::Active {
        map_name: map.into(),
        tile: TilePosition::new(2, 6),
        facing: Direction::Up,
        mode: MovementMode::Normal,
    };
}

fn act(state: &mut GameState, caretaker: &str, input: DayCareInput) -> SpecialRoutineEffect {
    state.script_runtime.pending_day_care_input = Some(input);
    let catalog = moves();
    let species = BTreeMap::new();
    let learnsets = BTreeMap::from([
        ("CHIKORITA".into(), Vec::new()),
        ("CYNDAQUIL".into(), Vec::new()),
        ("TOTODILE".into(), Vec::new()),
    ]);
    let items = BTreeMap::new();
    let context = full_context(&catalog, &species, &learnsets, &items);
    apply_random_special_routine_with_context(
        state,
        context,
        caretaker,
        &mut ReplayDivider::new(divider_trace_for_sub_values([150, 0x23, 0x45])),
    )
    .expect("daycare action")
    .effect
}

fn party(state: &mut GameState, ids: &[&str]) {
    for id in ids {
        assert!(state.storage.party.add_pokemon(pokemon(id)));
    }
    state.sync_party_from_storage();
}

fn success(effect: SpecialRoutineEffect) {
    assert!(
        matches!(
            effect,
            SpecialRoutineEffect::DayCareInteraction { success: true, .. }
        ),
        "{effect:?}"
    );
}

fn denied(effect: SpecialRoutineEffect, expected: &str) {
    assert!(
        matches!(effect, SpecialRoutineEffect::DayCareInteraction { success: false, reason: Some(ref reason), .. } if reason == expected),
        "{effect:?}"
    );
}

#[test]
fn day_care_sites_keep_original_custody_and_fees_after_reload() {
    let mut state = GameState::default();
    party(&mut state, &["CHIKORITA", "CYNDAQUIL", "TOTODILE"]);
    state.money = 1000;
    state.storage.party.pokemon[0].as_mut().unwrap().nickname = "LEAF".into();
    let original = state.storage.party.pokemon[0].clone().unwrap();
    visit(&mut state, "MnMankatoDayCare");
    success(act(
        &mut state,
        "DayCareMan",
        DayCareInput::Deposit { party_slot: 0 },
    ));
    let deposited = state.day_care_locations[MANKATO_DAY_CARE].clone();
    visit(&mut state, "MnBemidjiDayCare");
    assert!(state.current_day_care().man.pokemon.is_none());
    denied(
        act(&mut state, "DayCareMan", DayCareInput::Withdraw {}),
        "empty",
    );
    success(act(
        &mut state,
        "DayCareMan",
        DayCareInput::Deposit { party_slot: 0 },
    ));
    assert_eq!(state.day_care_locations[MANKATO_DAY_CARE], deposited);
    let identity = crate::save::SaveModpackIdentity::new("daycare-test", "01".repeat(32)).unwrap();
    let path = std::env::temp_dir().join(format!(
        "geothite-daycares-{}.crystalsave",
        std::process::id()
    ));
    crate::save::write_save_game_for_modpack(&path, state, &identity, &"02".repeat(32)).unwrap();
    state = crate::save::read_save_game_for_modpack(&path, &identity, &"02".repeat(32))
        .expect("both sites reload")
        .into_state();
    crate::save::erase_save_game(path).unwrap();
    assert_eq!(
        state
            .current_day_care()
            .man
            .pokemon
            .as_ref()
            .unwrap()
            .species
            .id,
        "CYNDAQUIL"
    );
    success(act(&mut state, "DayCareMan", DayCareInput::Withdraw {}));
    assert_eq!(state.money, 900);
    assert_eq!(state.day_care_locations[MANKATO_DAY_CARE], deposited);
    visit(&mut state, "MnMankatoDayCare");
    success(act(&mut state, "DayCareMan", DayCareInput::Withdraw {}));
    assert_eq!(state.money, 800);
    let retrieved = state
        .storage
        .party
        .pokemon
        .iter()
        .flatten()
        .find(|p| p.nickname == "LEAF")
        .unwrap();
    assert_eq!(retrieved.species, original.species);
    assert_eq!(retrieved.dvs, original.dvs);
    assert_eq!(retrieved.original_trainer_id, original.original_trainer_id);
    assert_eq!(
        retrieved.moves.iter().map(|m| &m.name).collect::<Vec<_>>(),
        original.moves.iter().map(|m| &m.name).collect::<Vec<_>>()
    );
    assert!(
        state
            .day_care_locations
            .values()
            .all(|site| site.man.pokemon.is_none())
    );
    assert_eq!(state.day_care, crate::state::DayCareState::default());
}

#[test]
fn day_care_sites_keep_eggs_local_and_ready_eggs_survive_parent_pickup() {
    let mut state = GameState {
        player_name: "CHRIS".into(),
        ..GameState::default()
    };
    party(
        &mut state,
        &["TOTODILE", "TOTODILE", "TOTODILE", "TOTODILE", "TOTODILE"],
    );
    for (map, species) in [
        ("MnMankatoDayCare", "CHIKORITA"),
        ("MnBemidjiDayCare", "CYNDAQUIL"),
    ] {
        visit(&mut state, map);
        let mut female = pokemon(species);
        female.dvs = Dv::from_non_hp(0, 0, 0, 0);
        let mut male = pokemon(species);
        male.dvs = Dv::from_non_hp(15, 1, 0, 0);
        let site = state.current_day_care_mut();
        site.man.pokemon = Some(female);
        site.lady.pokemon = Some(male);
        site.man.active = true;
        site.lady.active = true;
        let mut divider = ReplayDivider::new(divider_trace_for_sub_values([150, 0x23, 0x45]));
        update_day_care_compatibility(
            &mut state,
            &mut CrystalRandom::new(Default::default(), &mut divider),
        )
        .unwrap();
        assert!(state.current_day_care().egg.as_ref().unwrap().is_egg);
        state.current_day_care_mut().steps_until_next_egg = 1;
    }
    // The same walking step advances both independent breeding timers.
    visit(&mut state, "MnMankato");
    let mut divider = ReplayDivider::new(divider_trace_for_sub_values([180, 0, 180, 0]));
    advance_day_care_step(
        &mut state,
        &TEST_GROWTH_RATES,
        &mut CrystalRandom::new(Default::default(), &mut divider),
    )
    .unwrap();
    assert!(state.day_care_locations.values().all(|s| s.egg_present));
    visit(&mut state, "MnBemidjiDayCare");
    let other = state.day_care_locations[MANKATO_DAY_CARE].clone();
    party(&mut state, &["TOTODILE"]);
    denied(
        act(&mut state, "DayCareMan", DayCareInput::CollectEgg {}),
        "party_full",
    );
    assert_eq!(state.day_care_locations[MANKATO_DAY_CARE], other);
    state.storage.party.pokemon[5] = None;
    state.sync_party_from_storage();
    success(act(&mut state, "DayCareMan", DayCareInput::CollectEgg {}));
    assert_eq!(
        state.storage.party.pokemon[5].as_ref().unwrap().species.id,
        "CYNDAQUIL"
    );
    assert!(!state.current_day_care().egg_present);
    denied(
        act(&mut state, "DayCareMan", DayCareInput::CollectEgg {}),
        "no_egg",
    );
    assert_eq!(state.day_care_locations[MANKATO_DAY_CARE], other);
    visit(&mut state, "MnMankatoDayCare");
    state.storage.party.pokemon[5] = None;
    state.sync_party_from_storage();
    state.money = 1000;
    success(act(&mut state, "DayCareLady", DayCareInput::Withdraw {}));
    assert!(state.current_day_care().egg_present);
    state
        .validate_saved_state()
        .expect("ready egg remains saveable without both parents");
    let saved = serde_json::to_string(&state).unwrap();
    state = serde_json::from_str(&saved).unwrap();
    state.storage.party.pokemon[5] = None;
    state.sync_party_from_storage();
    success(act(&mut state, "DayCareMan", DayCareInput::CollectEgg {}));
    assert_eq!(
        state.storage.party.pokemon[5].as_ref().unwrap().species.id,
        "CHIKORITA"
    );
    denied(
        act(&mut state, "DayCareMan", DayCareInput::CollectEgg {}),
        "no_egg",
    );
}

#[test]
fn day_care_sites_grow_away_from_home_without_evolving_and_charge_their_own_fee() {
    let mut state = GameState::default();
    party(&mut state, &["CHIKORITA", "CYNDAQUIL", "TOTODILE"]);
    visit(&mut state, "MnMankatoDayCare");
    success(act(
        &mut state,
        "DayCareLady",
        DayCareInput::Deposit { party_slot: 0 },
    ));
    state
        .current_day_care_mut()
        .lady
        .pokemon
        .as_mut()
        .unwrap()
        .experience =
        calculate_experience(&TEST_GROWTH_RATES, "GROWTH_MEDIUM_FAST", 6).unwrap() - 1;
    visit(&mut state, "MnBemidjiDayCare");
    success(act(
        &mut state,
        "DayCareLady",
        DayCareInput::Deposit { party_slot: 0 },
    ));
    let before = state.day_care_locations[BEMIDJI_DAY_CARE]
        .lady
        .pokemon
        .as_ref()
        .unwrap()
        .experience;
    visit(&mut state, "MnMankato");
    advance_day_care_step(
        &mut state,
        &TEST_GROWTH_RATES,
        &mut CrystalRandom::new(Default::default(), ReplayDivider::new([])),
    )
    .unwrap();
    assert_eq!(
        state.day_care_locations[BEMIDJI_DAY_CARE]
            .lady
            .pokemon
            .as_ref()
            .unwrap()
            .experience,
        before + 1
    );
    state.money = 199;
    visit(&mut state, "MnMankatoDayCare");
    let before = state.current_day_care().clone();
    denied(
        act(&mut state, "DayCareLady", DayCareInput::Withdraw {}),
        "not_enough_money",
    );
    assert_eq!(state.current_day_care(), &before);
    state.money = 200;
    success(act(&mut state, "DayCareLady", DayCareInput::Withdraw {}));
    assert_eq!(state.money, 0);
    assert_eq!(
        state.storage.party.pokemon[1].as_ref().unwrap().species.id,
        "CHIKORITA"
    );
    assert_eq!(state.storage.party.pokemon[1].as_ref().unwrap().level, 6);
    visit(&mut state, "MnBemidjiDayCare");
    state.money = 100;
    success(act(&mut state, "DayCareLady", DayCareInput::Withdraw {}));
    assert_eq!(state.money, 0);
}

#[test]
fn day_care_sites_reject_ineligible_deposits_and_full_party_without_losing_custody() {
    for map in ["MnMankatoDayCare", "MnBemidjiDayCare"] {
        let mut state = GameState::default();
        visit(&mut state, map);
        party(&mut state, &["CHIKORITA"]);
        denied(
            act(
                &mut state,
                "DayCareMan",
                DayCareInput::Deposit { party_slot: 0 },
            ),
            "only_one_mon",
        );
        party(&mut state, &["CYNDAQUIL"]);
        state.storage.party.pokemon[0].as_mut().unwrap().is_egg = true;
        denied(
            act(
                &mut state,
                "DayCareMan",
                DayCareInput::Deposit { party_slot: 0 },
            ),
            "egg",
        );
        state.storage.party.pokemon[0].as_mut().unwrap().is_egg = false;
        state.storage.party.pokemon[1].as_mut().unwrap().hp = 0;
        denied(
            act(
                &mut state,
                "DayCareMan",
                DayCareInput::Deposit { party_slot: 0 },
            ),
            "last_healthy_mon",
        );
        state.storage.party.pokemon[1].as_mut().unwrap().hp = 1;
        state.storage.party.pokemon[0].as_mut().unwrap().item = Some("FLOWER_MAIL".into());
        denied(
            act(
                &mut state,
                "DayCareMan",
                DayCareInput::Deposit { party_slot: 0 },
            ),
            "mail",
        );
        state.storage.party.pokemon[0].as_mut().unwrap().item = None;
        success(act(
            &mut state,
            "DayCareMan",
            DayCareInput::Deposit { party_slot: 0 },
        ));
        let custody = state.current_day_care().clone();
        denied(
            act(
                &mut state,
                "DayCareMan",
                DayCareInput::Deposit { party_slot: 0 },
            ),
            "occupied",
        );
        party(
            &mut state,
            &["TOTODILE", "TOTODILE", "TOTODILE", "TOTODILE", "TOTODILE"],
        );
        state.money = 1000;
        denied(
            act(&mut state, "DayCareMan", DayCareInput::Withdraw {}),
            "party_full",
        );
        assert_eq!(state.current_day_care(), &custody);
        assert_eq!(state.money, 1000);
    }
}
