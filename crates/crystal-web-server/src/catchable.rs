//! Server-owned Gen 2 encounter expansion. No gameplay rules live here.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack};
use flate2::{Compression, write::GzEncoder};
use serde::Deserialize;
use std::io::{self, BufReader, BufWriter, Write};

pub const MANIFEST_ID: &str = "all-251-catchable-v1";
pub const BROWSER_PACK_FILENAME: &str = "core-modular.browser.crystalpack";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    map: String,
    surface: String,
    slot: usize,
    species: String,
    level: u8,
}

pub fn build(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let placements: Vec<Placement> =
        serde_json::from_str(include_str!("../modpacks/all-251/encounters.json"))?;
    let mut encounters = base.data().wild_encounters.clone();
    let mut occupied = BTreeSet::new();
    for placement in placements {
        ensure!(
            occupied.insert((
                placement.map.clone(),
                placement.surface.clone(),
                placement.slot
            )),
            "duplicate encounter slot"
        );
        ensure!(
            (1..=100).contains(&placement.level),
            "invalid encounter level"
        );
        ensure!(
            base.data().pokemon.contains_key(&placement.species),
            "unknown species {}",
            placement.species
        );
        let table = encounters
            .get_mut(&placement.map)
            .with_context(|| format!("missing encounter map {}", placement.map))?;
        let encounter_table = match placement.surface.as_str() {
            "grass" => table.grass.as_mut(),
            "water" => table.water.as_mut(),
            _ => anyhow::bail!("unknown encounter surface"),
        }
        .context("placement requires an existing encounter table")?;
        for slots in [
            &mut encounter_table.morning,
            &mut encounter_table.day,
            &mut encounter_table.night,
        ] {
            let slot = slots
                .get_mut(placement.slot)
                .context("missing encounter slot")?;
            slot.species = placement.species.clone();
            slot.level = placement.level;
        }
    }
    let pack = base.with_wild_encounter_overlay(MANIFEST_ID, encounters)?;
    verify_coverage(&pack)?;
    Ok(pack)
}

/// Every Gen 2 species must have a renewable, nonzero-rate grass or Surf encounter.
/// Vanilla day/night differences remain meaningful; additions appear in all three periods.
pub fn verify_coverage(pack: &CompiledGamePack) -> Result<()> {
    let expected = pack
        .data()
        .pokemon
        .values()
        .filter(|p| (1..=251).contains(&p.int_id))
        .collect::<Vec<_>>();
    ensure!(
        expected.len() == 251,
        "all-251 requires the complete Gen 2 catalog"
    );
    ensure!(
        expected
            .iter()
            .map(|species| species.int_id)
            .collect::<BTreeSet<_>>()
            == (1..=251).collect(),
        "all-251 requires each Gen 2 Pokédex number exactly once"
    );
    ensure!(
        expected.iter().all(|species| species.catch_rate > 0),
        "all-251 requires nonzero capture rates"
    );
    let mut found = BTreeSet::new();
    for (map, table) in &pack.data().wild_encounters {
        ensure!(pack.data().maps.contains_key(map), "unknown map {map}");
        for (surface, encounters) in [("grass", &table.grass), ("water", &table.water)] {
            let Some(encounters) = encounters else {
                continue;
            };
            for (period, slots) in [
                ("morning", &encounters.morning),
                ("day", &encounters.day),
                ("night", &encounters.night),
            ] {
                let rate = if surface == "grass" {
                    table
                        .grass_rates
                        .as_ref()
                        .and_then(|rates| rates.get(period))
                        .copied()
                } else {
                    table.water_rate
                };
                if !rate.is_some_and(|rate| rate > 0) {
                    continue;
                }
                for slot in slots {
                    found.insert(slot.species.as_str());
                }
            }
        }
    }
    let missing = expected
        .iter()
        .filter(|p| !found.contains(p.id.as_str()))
        .map(|p| p.id.as_str())
        .collect::<Vec<_>>();
    ensure!(
        missing.is_empty(),
        "uncatchable species: {}",
        missing.join(", ")
    );
    Ok(())
}

/// Materialize into writable server data, preserving the source and audio storage.
pub fn prepare(
    web_root: &Path,
    data_dir: &Path,
) -> Result<(PathBuf, crystal_assets::CompiledGamePackIdentity)> {
    let base = read_verified_compiled_game_pack(std::fs::canonicalize(
        web_root.join(BROWSER_PACK_FILENAME),
    )?)?;
    let pack = crystal_assets::build_player_customization_modpack(
        &crystal_assets::build_realtime_clock_modpack(&build(&base)?)?
    )?;
    let directory = data_dir.join("modpacks");
    std::fs::create_dir_all(&directory)?;
    let output = directory.join("all-251.browser.crystalpack");
    let temporary = directory.join("all-251.pending.crystalpack");
    pack.write_preserving_storage(&temporary)?;
    std::fs::rename(&temporary, &output)?;
    let gzip_output = output.with_extension("crystalpack.gz");
    let gzip_temporary = directory.join("all-251.pending.crystalpack.gz");
    let mut source = BufReader::new(std::fs::File::open(&output)?);
    let target = BufWriter::new(std::fs::File::create(&gzip_temporary)?);
    let mut encoder = GzEncoder::new(target, Compression::best());
    io::copy(&mut source, &mut encoder)?;
    encoder.finish()?.flush()?;
    std::fs::rename(gzip_temporary, gzip_output)?;
    Ok((output, pack.identity()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_browser_pack_has_all_251_renewable_without_changing_gameplay_rules() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs/core-modular.browser.crystalpack");
        let base =
            read_verified_compiled_game_pack(std::fs::canonicalize(source).unwrap()).unwrap();
        assert!(
            verify_coverage(&base).is_err(),
            "vanilla should expose the completion barrier"
        );
        let pack = build(&base).unwrap();
        verify_coverage(&pack).unwrap();
        assert!(
            base.with_wild_encounter_overlay("empty", Default::default())
                .is_err()
        );
        let mut invalid = base.data().wild_encounters["Route29"].clone();
        invalid.grass.as_mut().unwrap().day[0].species = "MISSING_SPECIES".into();
        assert!(
            base.with_wild_encounter_overlay(
                "invalid-species",
                [("Route29".into(), invalid)].into()
            )
            .is_err()
        );
        let mut uncatchable = pack.data().wild_encounters.clone();
        for table in uncatchable.values_mut() {
            for encounters in [&mut table.grass, &mut table.water].into_iter().flatten() {
                for slots in [
                    &mut encounters.morning,
                    &mut encounters.day,
                    &mut encounters.night,
                ] {
                    for slot in slots {
                        if slot.species == "CELEBI" {
                            slot.species = "RATTATA".into();
                        }
                    }
                }
            }
        }
        let incomplete = pack
            .with_wild_encounter_overlay("missing-celebi", uncatchable)
            .unwrap();
        assert!(
            verify_coverage(&incomplete)
                .unwrap_err()
                .to_string()
                .contains("CELEBI")
        );
        assert_ne!(
            base.identity().unwrap().content_hash,
            pack.identity().unwrap().content_hash
        );
        assert!(pack.runtime_modpack_id().unwrap().ends_with(MANIFEST_ID));
        let identity = pack.identity().unwrap();
        let mut hub = crate::hub::Hub::default();
        hub.connect(
            uuid::Uuid::new_v4(),
            crate::hub::ClientIdentity {
                user_id: "catchable-test".into(),
                display_name: "Collector".into(),
                world: crate::hub::WorldIdentity {
                    world_id: "main".into(),
                    modpack: crate::hub::ModpackIdentity {
                        id: identity.runtime_modpack_id,
                        content_hash: identity.content_hash,
                    },
                },
            },
        )
        .unwrap();
        let mut unchanged = pack.data().clone();
        unchanged.wild_encounters = base.data().wild_encounters.clone();
        assert_eq!(&unchanged, base.data());
        assert_eq!(pack.runtime_files(), base.runtime_files());
        assert_eq!(pack.compiled_audio(), base.compiled_audio());
        assert_eq!(
            pack.audio_manifest().unwrap(),
            base.audio_manifest().unwrap()
        );
        assert!(build(&pack).is_err(), "duplicate overlay must fail");
        assert_eq!(
            build(&base).unwrap().identity().unwrap(),
            pack.identity().unwrap()
        );
        let path = std::env::temp_dir().join(format!(
            "catchable-test-{}.crystalpack",
            uuid::Uuid::new_v4()
        ));
        pack.write_preserving_storage(&path).unwrap();
        let loaded = read_verified_compiled_game_pack(&path).unwrap();
        assert_eq!(loaded.identity().unwrap(), pack.identity().unwrap());
        verify_coverage(&loaded).unwrap();
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(test)]
#[test]
fn hosted_pack_composes_server_clock_and_encounters() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content-packs");
    let directory = std::env::temp_dir().join(format!("hosted-clock-{}", uuid::Uuid::new_v4()));
    let (path, identity) = prepare(&source, &directory).unwrap();
    let pack = read_verified_compiled_game_pack(path).unwrap();
    assert!(pack.data().server_clock);
    assert!(pack.data().player_customization);
    assert!(identity.runtime_modpack_id.contains(MANIFEST_ID));
    assert!(
        identity
            .runtime_modpack_id
            .contains(crystal_assets::REALTIME_CLOCK_MANIFEST_ID)
    );
    assert_eq!(pack.identity().unwrap(), identity);
    verify_coverage(&pack).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
}
