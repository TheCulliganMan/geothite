//! Authored finishing pass for the two Fen pools, their margins and the woodland hollow.
use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{WildEncounter, WildEncounterTable};

use crate::CompiledGamePack;
use crate::northstar_regional_roster::RegionalRoster;

const ROOM: &str = "MnLayeredFen";

fn wetland_slots(partners: [&str; 3]) -> Vec<WildEncounter> {
    [32, 34, 36]
        .into_iter()
        .zip(partners)
        .map(|(level, species)| WildEncounter {
            level,
            species: species.to_owned(),
        })
        .collect()
}

fn populate_pools(pack: &mut CompiledGamePack) -> Result<()> {
    let roster = RegionalRoster::current()?;
    // The Fen is tagged peat-water in the regional ecology. Its badge band
    // already specifies these three Surf levels, but the original map had no table.
    let water = WildEncounterTable {
        morning: wetland_slots(["RONDIVE", "REEDRAFT", "CROSSPOUT"]),
        day: wetland_slots(["REEDRAFT", "RONDIVE", "CROSSPOUT"]),
        night: wetland_slots(["LEECHURGE", "CROSSPOUT", "RONDIVE"]),
    };
    ensure!(
        water
            .morning
            .iter()
            .chain(&water.day)
            .chain(&water.night)
            .all(|slot| roster.allowed.contains(&slot.species)
                && pack.data.pokemon.contains_key(&slot.species)),
        "inactive Fen water partner"
    );
    let encounters = pack.data.wild_encounters.get_mut(ROOM).context(ROOM)?;
    ensure!(
        encounters.map_name == ROOM
            && encounters.water.is_none()
            && encounters.water_rate.is_none(),
        "unexpected existing Fen water encounters"
    );
    encounters.water = Some(water);
    encounters.water_rate = Some(3);
    Ok(())
}

/// Adds the Fen discoveries and terrain to either completed Trail Battles edition.
///
/// # Errors
/// Returns an error if the source edition, existing terrain, or authored wildlife differs
/// from the checked parent, or if the resulting archive fails runtime validation.
pub fn revise_fen_clearings(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        base.report.manifests.last().map(String::as_str) == Some("northstar-trail-battles-v1"),
        "Fen clearings require the two-lake Trail Battles edition"
    );
    let mut pack = base.clone();
    crate::northstar_fen_thaw::apply(&mut pack)?;
    crate::northstar_fen_cache::apply(&mut pack)?;
    crate::northstar_fen_hollow::apply(&mut pack)?;
    populate_pools(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-fen-clearings-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use anyhow::{Context, Result, ensure};
    use crystal_core::world::encounters::{EncounterSurface, TimeOfDay, select_wild_encounter};

    #[test]
    #[ignore = "requires an external Fen Clearings pack in MN_FEN_CLEARINGS_PACK"]
    fn fen_pools_resolve_all_three_surf_slots_at_each_clock() -> Result<()> {
        let path = std::env::var_os("MN_FEN_CLEARINGS_PACK").context("MN_FEN_CLEARINGS_PACK")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let table = data.require_wild_encounters_for_map(super::ROOM)?;
        ensure!(table.water_rate == Some(3));
        for (time, common) in [
            (TimeOfDay::Morning, "RONDIVE"),
            (TimeOfDay::Day, "REEDRAFT"),
            (TimeOfDay::Night, "LEECHURGE"),
        ] {
            let mut slots = BTreeSet::new();
            for roll in 1..=100 {
                let found = select_wild_encounter(
                    table,
                    &data.encounter_slot_tables,
                    EncounterSurface::Water,
                    time,
                    roll,
                    255,
                )?
                .context("Fen Surf encounter")?;
                ensure!(found.encounter.level == [32, 34, 36][found.slot]);
                ensure!(data.saved_wild_encounter_exists(
                    super::ROOM,
                    &found.encounter.species,
                    found.level
                ));
                if found.slot == 0 {
                    ensure!(found.encounter.species == common);
                }
                slots.insert(found.slot);
            }
            ensure!(slots == BTreeSet::from([0, 1, 2]));
        }
        Ok(())
    }
}
