//! Repair the inherited grass-only Grand Marais record for legal native Surf.
use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{
    EncounterSurface, TimeOfDay, WildEncounter, WildEncounterData, WildEncounterTable,
    base_encounter_rate, select_wild_encounter,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const MAP: &str = "MnGrandMarais";
const TEMPLATE: &str = "MnR11C00R00";
const PARENT: &str = "53ae7d31d3f73eab9341b2be2b0bdc6ba782a13ea56584162b8ca32d66c050f6";
const SOURCE: &str = "5a818e11016e1b6ef556e94b22967b5990e6de2081afa302dde6a991154044bf";
const BEFORE: &str = "e420d4258165c94526ba87003a249468889726712d2644dc4ed552350aacb615";
const DONOR: &str = "2c043ffac20a1a2bcd7dfd6c4a1dd63868299b092132d9ab6a8e461b3b53dd2d";
const AFTER: &str = "0e555f66bd068c7a96a407aae537b8d7c46d7926e0b7ff3b0e1a84b04a087e4b";

/// Supply Grand Marais's missing Water encounter rate and table from North Shore.
///
/// The exact completed harbor/cedar/R01 parent and existing `MN_R11.json` source
/// are required. That source's three native Water slots and rate must match the
/// installed neighboring R11 template before use. Only Grand Marais's existing
/// `WildEncounterData::water_rate` and `water` fields and the derived identity
/// change. Its grass, every native map/program/actor, other encounter records,
/// global slot weights, art/audio, report and recovery data remain exact.
///
/// This repairs content required by the production encounter transaction; it
/// neither disables encounters nor grants progress. The separate terrain-only
/// buoy revision is retained. The caller owns writing and strict readback.
///
/// # Errors
/// Rejects a changed parent/source/target/template preimage, source/native
/// mismatch, invalid species/slot/rate, unrelated mutation or failed native pack
/// verification. Borrowed inputs remain unchanged on failure.
pub fn repair_grand_marais_water_encounters(
    base: &CompiledGamePack,
    north_shore_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        base.identity.content_hash == PARENT && base.data.maps.len() == 681,
        "exact completed 58b9 Grand Marais terrain parent required"
    );
    ensure!(
        hash(north_shore_json.as_bytes()) == SOURCE,
        "exact existing North Shore encounter source required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let old = base
        .data
        .wild_encounters
        .get(MAP)
        .context("existing town encounters")?;
    let donor = base
        .data
        .wild_encounters
        .get(TEMPLATE)
        .context("actual North Shore encounters")?;
    ensure!(
        encounter_hash(old)? == BEFORE && old.water.is_none() && old.water_rate.is_none(),
        "Grand Marais grass-only native preimage differs"
    );
    ensure!(
        encounter_hash(donor)? == DONOR,
        "native North Shore template preimage differs"
    );
    let source: Value = serde_json::from_str(north_shore_json)?;
    let rate = u8::try_from(
        source["encounters"]["water_rate"]
            .as_u64()
            .context("authored North Shore Water rate")?,
    )?;
    let slots: Vec<WildEncounter> = serde_json::from_value(source["encounters"]["water"].clone())
        .context("authored North Shore Water slots")?;
    ensure!(
        rate == 4 && slots.len() == 3,
        "exact existing three-slot low-rate Water table required"
    );
    let table = WildEncounterTable {
        morning: slots.clone(),
        day: slots.clone(),
        night: slots,
    };
    ensure!(
        donor.water_rate == Some(rate) && donor.water.as_ref() == Some(&table),
        "authored source and native neighboring Water data differ"
    );
    let mut result = base.clone();
    let target = result
        .data
        .wild_encounters
        .get_mut(MAP)
        .context("retained town encounters")?;
    target.water_rate = Some(rate);
    target.water = Some(table);
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    validate_preservation(base, &result)?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn encounter_hash(data: &WildEncounterData) -> Result<String> {
    Ok(hash(&serde_json::to_vec(&serde_json::to_value(data)?)?))
}

fn validate_water(pack: &CompiledGamePack, data: &WildEncounterData) -> Result<()> {
    ensure!(
        data.water_rate == Some(4),
        "actual Water rate must remain four"
    );
    let water = data
        .water
        .as_ref()
        .context("actual nonempty town Water table")?;
    for time in [TimeOfDay::Morning, TimeOfDay::Day, TimeOfDay::Night] {
        let slots = water.slots(time);
        ensure!(
            slots.len() == 3,
            "three actual Water slots required at {time:?}"
        );
        for slot in slots {
            ensure!(
                pack.data.pokemon.contains_key(&slot.species) && slot.level > 0,
                "invalid native Water species/level"
            );
        }
        ensure!(
            base_encounter_rate(data, EncounterSurface::Water, time)? == 4,
            "native Water rate lookup differs"
        );
        let mut counts = [0_u16; 3];
        for roll in 1..=100 {
            let chosen = select_wild_encounter(
                data,
                &pack.data.encounter_slot_tables,
                EncounterSurface::Water,
                time,
                roll,
                0,
            )?
            .context("native Water slot roll did not resolve")?;
            *counts
                .get_mut(chosen.slot)
                .context("unexpected Water slot")? += 1;
        }
        ensure!(
            counts == [60, 30, 10],
            "native Water weights changed at {time:?}"
        );
    }
    Ok(())
}

fn validate_preservation(base: &CompiledGamePack, result: &CompiledGamePack) -> Result<()> {
    let old = base
        .data
        .wild_encounters
        .get(MAP)
        .context("old town encounters")?;
    let new = result
        .data
        .wild_encounters
        .get(MAP)
        .context("repaired town encounters")?;
    ensure!(
        encounter_hash(new)? == AFTER,
        "unreviewed repaired encounter record"
    );
    validate_water(result, new)?;
    let mut normalized = result.clone();
    let target = normalized
        .data
        .wild_encounters
        .get_mut(MAP)
        .context("retained encounter record")?;
    target.water_rate = old.water_rate;
    target.water = old.water.clone();
    normalized.identity = base.identity.clone();
    ensure!(
        &normalized == base,
        "change beyond two Water fields and derived identity"
    );
    Ok(())
}

#[cfg(test)]
#[path = "grand_marais_water_repair_tests.rs"]
mod tests;
