//! Add field oats and smokehouse wildlife without changing native geography.
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_assets::CompiledGamePack;
use crystal_core::world::encounters::{WildEncounter, WildEncounterZone};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    id: String,
    habitats: Vec<Habitat>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Habitat {
    id: String,
    map: String,
    label: String,
    rect: [i16; 4],
    species: String,
    level: u8,
}

fn build(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crystal_assets::verify_northstar_publication_geometry(base)?;
    let plan: Plan = serde_json::from_str(source)?;
    ensure!(!plan.habitats.is_empty(), "authored habitats required");
    let mut overlays = BTreeMap::new();
    for habitat in plan.habitats {
        ensure!(
            habitat.map.starts_with("Mn") && !habitat.label.is_empty(),
            "named Minnesota habitat required"
        );
        ensure!(
            base.data()
                .story_event_script_constants
                .global
                .contains_key(&format!("MN_DEX_MEMBER_{}", habitat.species)),
            "species must belong to the installed Minnesota roster"
        );
        let map = base
            .data()
            .maps
            .get(&habitat.map)
            .context("habitat map is absent")?;
        let original = base
            .data()
            .wild_encounters
            .get(&habitat.map)
            .context("habitat has no original encounter table")?;
        let table = overlays
            .entry(habitat.map.clone())
            .or_insert_with(|| original.clone());
        let [left, top, right, bottom] = habitat.rect;
        ensure!(
            left >= 0 && top >= 0 && left <= right && top <= bottom,
            "invalid habitat rectangle"
        );
        ensure!(
            right < i16::try_from(map.attributes.width * 2)?
                && bottom < i16::try_from(map.attributes.height * 2)?,
            "habitat outside map"
        );
        let collision = &base.data().tilesets[&map.attributes.tileset_name].collision;
        for y in top..=bottom {
            for x in left..=right {
                let x = usize::try_from(x)?;
                let y = usize::try_from(y)?;
                let block = map.blocks[y / 2 * usize::from(map.attributes.width) + x / 2];
                ensure!(
                    collision[&format!("{block:02x}")][y % 2 * 2 + x % 2] == "TALL_GRASS",
                    "{} must use existing encounter grass",
                    habitat.label
                );
                ensure!(
                    table
                        .zone_at(i16::try_from(x)?, i16::try_from(y)?)
                        .is_none(),
                    "habitat overlaps an installed zone"
                );
            }
        }
        let rates = table.grass_rates.clone().context("grass rates")?;
        ensure!(
            ["morning", "day", "night"]
                .iter()
                .all(|t| rates.get(*t).is_some_and(|r| *r > 0)),
            "habitat must be active all day"
        );
        let mut grass = table.grass.clone().context("native grass table")?;
        for slots in [&mut grass.morning, &mut grass.day, &mut grass.night] {
            ensure!(
                slots.len() == 7 && slots.iter().any(|slot| slot.level == habitat.level),
                "habitat must retain the local seven-slot level curve"
            );
            // Crystal's common residents remain in slots 0/1/4; the landmark
            // resident occupies 20/10/4/1 percent slots, rather than every roll.
            for index in [2, 3, 5, 6] {
                slots[index] = WildEncounter {
                    species: habitat.species.clone(),
                    level: habitat.level,
                };
            }
        }
        table.zones.push(WildEncounterZone {
            id: habitat.id,
            min_x: left,
            min_y: top,
            max_x: right,
            max_y: bottom,
            grass_rates: rates,
            grass,
        });
    }
    base.with_wild_encounter_overlay(&plan.id, overlays)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 3,
        "usage: pack_landmark_habitats PACK PLAN OUTPUT"
    );
    let output = Path::new(&args[2]);
    ensure!(!output.exists(), "output must be fresh");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(Path::new(&args[0]))?;
    let pack = build(&base, &std::fs::read_to_string(&args[1])?)?;
    pack.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(
        written.identity()? == pack.identity()?,
        "published identity differs"
    );
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build;

    #[test]
    #[ignore = "requires CRYSTAL_LANDMARK_PARENT external Minnesota pack"]
    fn landmark_overlay_preserves_parent_and_rejects_bad_sites() {
        let path = std::env::var("CRYSTAL_LANDMARK_PARENT").expect("external parent");
        let base = crystal_assets::read_verified_compiled_game_pack(path).expect("parent");
        let source = include_str!(
            "../../../../planning/minnesota/pokemon/acquisition-v01/landmark-habitats.json"
        );
        let child = build(&base, source).expect("landmark habitats");
        let before = serde_json::to_value(base.data()).expect("parent data");
        let mut after = serde_json::to_value(child.data()).expect("child data");
        after["wild_encounters"] = before["wild_encounters"].clone();
        assert_eq!(before, after, "only encounter tables may change");
        assert!(build(&child, source).is_err(), "duplicate installation");
        assert!(build(&base, &source.replace("CINDERFIN", "MAGIKARP")).is_err());
        assert!(build(&base, &source.replace("MnR02C00R00", "MnNoSuchMap")).is_err());
        assert!(build(&base, &source.replace("32,2,35,3", "0,0,1,1")).is_err());
    }
}
