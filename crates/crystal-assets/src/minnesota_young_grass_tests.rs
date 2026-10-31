use std::sync::OnceLock;

use crystal_core::models::Dv;
use crystal_core::random::{CrystalRandom, ReplayDivider};
use crystal_core::world::map::TilePosition;

use super::{YoungGrassHabitat, install};
use crate::CompiledGamePack;

fn parent() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        crate::read_verified_compiled_game_pack(
            std::env::var_os("CRYSTAL_PROGRESSION_PARENT").expect("explicit ecology parent"),
        )
        .expect("verified ecology parent")
    })
}

fn source() -> serde_json::Value {
    let plan: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            std::env::var_os("CRYSTAL_PROGRESSION_SOURCE").expect("explicit progression source"),
        )
        .expect("progression plan"),
    )
    .expect("plan JSON");
    plan["groups"]
        .as_array()
        .expect("groups")
        .iter()
        .find(|g| g["id"] == "grand-marais")
        .expect("Grand Marais group")["young_grass_zones"]
        .clone()
}

#[test]
#[ignore = "requires CRYSTAL_PROGRESSION_PARENT and CRYSTAL_PROGRESSION_SOURCE"]
fn young_grass_retains_main_tables_and_produces_low_basics_with_a_late_party() {
    let base = parent();
    let habitats: Vec<YoungGrassHabitat> = serde_json::from_value(source()).expect("pocket source");
    let mut data = base.data().clone();
    let old = data.wild_encounters["MnGrandMarais"].clone();
    let mut habitat = old.clone();
    install(base, "MnGrandMarais", &mut habitat, &habitats).expect("reachable grass pocket");
    assert_eq!(habitat.grass, old.grass);
    assert_eq!(habitat.water, old.water);
    assert_eq!(habitat.grass_rates, old.grass_rates);
    assert_eq!(habitat.water_rate, old.water_rate);
    assert_eq!(habitat.swarm_overrides, old.swarm_overrides);
    assert_eq!(habitat.zones.len(), old.zones.len() + 1);
    data.wild_encounters.insert("MnGrandMarais".into(), habitat);
    let session = data
        .overworld_session("MnGrandMarais", TilePosition::new(4, 10), 0)
        .expect("native grass tile");
    let mut state = data
        .start_overworld_session_from_spawn(
            data.runtime_spawn_point(0).expect("native spawn"),
            &data.audio_ids().into_iter().map(str::to_owned).collect(),
        )
        .expect("native state")
        .0;
    state.storage.party.add_pokemon(
        data.create_pokemon("POUCHID", 80, Dv::default())
            .expect("late party"),
    );
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("consistent late-game badge flags");
    }
    data.commit_overworld_snapshot(
        &mut state,
        &session,
        crystal_core::systems::map_context::SpawnMemoryUpdate::Preserve,
    );
    let mut divider = ReplayDivider::new((0..=255).cycle().take(200000));
    let mut rng = CrystalRandom::new(state.random_state, &mut divider);
    let (mut low, mut scaled) = (0, 0);
    for _ in 0..2048 {
        if let Some(roll) = data
            .check_wild_encounter_after_step(&mut state, &session, &mut rng)
            .expect("production encounter roll")
            && let Some(actual) = roll.resolved
        {
            if actual.level <= 10 {
                assert!((3..=7).contains(&actual.level));
                low += 1;
            } else {
                assert!(actual.level >= 77);
                scaled += 1;
            }
            let mon = data
                .create_pokemon(&actual.encounter.species, actual.level, Dv::default())
                .expect("exact resident species");
            assert_eq!(mon.species.id, actual.encounter.species);
            assert!(data.saved_wild_encounter_exists("MnGrandMarais", &mon.species.id, mon.level));
        }
    }
    assert!(
        low > 0 && scaled > 0,
        "both native young and challenge encounters repeat"
    );
}

#[test]
#[ignore = "requires CRYSTAL_PROGRESSION_PARENT and CRYSTAL_PROGRESSION_SOURCE"]
fn young_grass_rejects_inaccessible_evolved_nonresident_and_overlapping_sources() {
    let base = parent();
    let original = source();
    for field in [
        "inaccessible",
        "evolved",
        "nonresident",
        "entrance",
        "overlap",
    ] {
        let mut value = original.clone();
        match field {
            "inaccessible" => {
                value[0]["zone"]["minX"] = 34.into();
                value[0]["zone"]["maxX"] = 35.into();
            }
            "evolved" => value[0]["zone"]["grass"]["day"][0]["species"] = "GOUGECREST".into(),
            "nonresident" => value[0]["zone"]["grass"]["day"][0]["species"] = "OATLET".into(),
            "entrance" => value[0]["entrance"] = serde_json::json!([1, 1]),
            "overlap" => value
                .as_array_mut()
                .expect("habitats")
                .push(original[0].clone()),
            _ => unreachable!(),
        }
        let habitats: Vec<YoungGrassHabitat> =
            serde_json::from_value(value).expect("well-formed rejected source");
        let mut data = base.data().wild_encounters["MnGrandMarais"].clone();
        assert!(
            install(base, "MnGrandMarais", &mut data, &habitats).is_err(),
            "{field}"
        );
    }
}
