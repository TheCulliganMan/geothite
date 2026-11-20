//! AREA must include time-specific local grass habitats, not just fallback slots.
use crystal_core::world::encounters::{
    WildEncounter, WildEncounterData, WildEncounterTable, WildEncounterZone,
};

#[test]
fn pokedex_habitats_area_includes_zone_only_species_and_keeps_base_water() {
    let entry = |species: &str| WildEncounter {
        species: species.into(),
        level: 10,
    };
    let mut data = WildEncounterData {
        map_name: "MnR09C00R00".into(),
        grass: Some(WildEncounterTable {
            day: vec![entry("AGATOT")],
            ..Default::default()
        }),
        water: Some(WildEncounterTable {
            morning: vec![entry("GILLPUP")],
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(!super::super::wild_encounters_include_species(
        &data, "CINDORE"
    ));
    data.zones.push(WildEncounterZone {
        id: "ORE_AIR".into(),
        min_x: 40,
        max_x: 41,
        min_y: 24,
        max_y: 33,
        grass_rates: Default::default(),
        grass: WildEncounterTable {
            night: vec![entry("CINDORE")],
            ..Default::default()
        },
    });
    for species in ["CINDORE", "AGATOT", "GILLPUP"] {
        assert!(
            super::super::wild_encounters_include_species(&data, species),
            "{species}"
        );
    }
    assert!(!super::super::wild_encounters_include_species(
        &data, "KETTLURK"
    ));
}

#[test]
#[ignore = "requires a Minnesota Fair Table or later pack in MN_FIELD_GUIDE_PACK"]
fn pokedex_habitats_show_real_rod_and_tree_sites_on_the_regional_atlas() {
    let path = std::env::var_os("MN_FIELD_GUIDE_PACK").expect("regional pack path");
    let pack =
        crystal_assets::read_verified_compiled_game_pack(path).expect("verified regional pack");
    let rows = |species| {
        super::super::regional_dex_areas::collect(pack.data(), species, &[])
            .expect("regional habitats")
    };
    let ray = rows("PADDARRAY");
    assert!(
        ray.iter()
            .any(|a| a.name == "WINONA" && a.method == "SUPER ROD")
    );
    let fox = rows("PATHFANG");
    assert!(
        fox.iter()
            .any(|a| a.name == "BOUNDARY WATERS" && a.method == "HEADBUTT")
    );
    assert!(
        fox.iter()
            .any(|a| a.name == "ROUTE 13" && a.method == "HEADBUTT")
    );
    assert!(
        rows("KRABBY").is_empty(),
        "dry lodge rooms cannot be rod locations"
    );
    assert!(
        rows("KETTLURK").is_empty(),
        "a scripted discovery is not a random nest"
    );
    for (species, place, method) in [
        ("WRIGGLET", "REED MARSH", "SAFARI GRASS"),
        ("MUDNIP", "REED MARSH", "SAFARI WATER"),
        ("PINIT", "PINE THICKET", "SAFARI GRASS"),
    ] {
        assert!(
            rows(species)
                .iter()
                .any(|a| a.name == place && a.method == method && a.time.is_empty()),
            "{species} can be found at {place} throughout the day"
        );
    }
    let loon = rows("LOOBIT");
    assert!(
        loon.iter()
            .any(|a| a.name == "PINE PORTAGE" && a.method == "SAFARI WATER")
    );
    assert!(
        !loon
            .iter()
            .any(|a| a.name == "PINE THICKET" && a.method == "SAFARI WATER"),
        "Pine Thicket has a configured water table but no water tiles"
    );
}

#[test]
#[ignore = "requires a Minnesota pack in MN_FIELD_GUIDE_PACK"]
fn regional_area_tracks_only_the_current_native_roaming_slots() -> anyhow::Result<()> {
    use super::super::regional_dex_areas;
    use crystal_core::state::RoamingPokemonState;

    let path = std::env::var_os("MN_FIELD_GUIDE_PACK").expect("regional pack path");
    let pack = crystal_assets::read_verified_compiled_game_pack(path)?;
    let data = pack.data();
    let at = |species: &str, name: &str| {
        let map = data
            .runtime_map_metadata
            .values()
            .find(|m| m.name == name)
            .expect("pack map");
        RoamingPokemonState {
            species: Some(species.into()),
            level: 40,
            map_group: u8::try_from(map.group_id).expect("roaming map group"),
            map_number: u8::try_from(map.map_id).expect("roaming map number"),
            ..Default::default()
        }
    };
    let locations = |species: &str, slots: &[RoamingPokemonState]| -> anyhow::Result<Vec<String>> {
        Ok(regional_dex_areas::collect(data, species, slots)?
            .into_iter()
            .filter(|area| area.method == "ROAMING")
            .map(|area| area.name)
            .collect())
    };
    let mut slots = [
        RoamingPokemonState::default(),
        RoamingPokemonState::default(),
        at("LOOBIT", "MnR01C00R00"),
    ];
    assert!(
        locations("LOOBIT", &slots)?.is_empty(),
        "slot three is not tracked"
    );
    slots[0] = at("LOOBIT", "MnR01C00R00");
    assert_eq!(locations("LOOBIT", &slots)?, ["ROUTE 1"]);
    slots[0] = at("LOOBIT", "MnR02C00R00");
    assert_eq!(locations("LOOBIT", &slots)?, ["ROUTE 2"]);
    slots[0] = at("LOOBIT", "Route29");
    assert!(
        locations("LOOBIT", &slots)?.is_empty(),
        "Johto sightings stay off the Minnesota atlas"
    );
    slots[1] = at("LOOBIT", "MnR01C00R00");
    assert_eq!(locations("LOOBIT", &slots)?, ["ROUTE 1"]);
    slots[1] = at("RAIKOU", "MnR01C00R00");
    assert!(locations("LOOBIT", &slots)?.is_empty());
    assert!(
        locations("RAIKOU", &slots)?.is_empty(),
        "foreign species do not become regional entries"
    );
    Ok(())
}
