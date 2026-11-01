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
