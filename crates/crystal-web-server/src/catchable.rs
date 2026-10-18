//! Serve the shared default solo-completion content with hosted features.
use anyhow::Result;
use crystal_assets::read_verified_compiled_game_pack;
use flate2::{Compression, write::GzEncoder};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

pub use crystal_assets::{
    ALL_251_MANIFEST_ID as MANIFEST_ID, verify_all_251_coverage as verify_coverage,
};
pub const BROWSER_PACK_FILENAME: &str = "core-modular.browser.crystalpack";

/// Every hosted entry point gets the same trainer and wild battle defaults.
pub fn build(base: &crystal_assets::CompiledGamePack) -> Result<crystal_assets::CompiledGamePack> {
    crystal_assets::build_default_battle_modpack(&crystal_assets::build_all_pokemon_modpack(base)?)
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
        &crystal_assets::build_realtime_clock_modpack(&build(&base)?)?,
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
    fn real_browser_pack_has_all_251_renewable_and_default_battle_rules() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs/core-modular.browser.crystalpack");
        let base =
            read_verified_compiled_game_pack(std::fs::canonicalize(source).unwrap()).unwrap();

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

        assert!(pack.runtime_modpack_id().unwrap().contains(MANIFEST_ID));
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
        unchanged.items = base.data().items.clone();
        unchanged.marts = base.data().marts.clone();
        unchanged.evolutions = base.data().evolutions.clone();
        unchanged.competitive_battles = base.data().competitive_battles;
        unchanged.type_categories = base.data().type_categories.clone();
        unchanged.wild_challenge = base.data().wild_challenge;
        assert_eq!(&unchanged, base.data());
        assert_eq!(pack.runtime_files(), base.runtime_files());
        assert_eq!(pack.compiled_audio(), base.compiled_audio());
        assert_eq!(
            pack.audio_manifest().unwrap(),
            base.audio_manifest().unwrap()
        );
        assert_eq!(
            build(&pack).unwrap().identity().unwrap(),
            pack.identity().unwrap()
        );
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
    assert!(pack.data().competitive_battles);
    assert!(pack.data().wild_challenge.is_some());
    assert!(
        identity
            .runtime_modpack_id
            .contains(crystal_assets::WILD_CHALLENGE_MANIFEST_ID)
    );
    assert_eq!(pack.data().type_categories.moves.len(), 251);
    assert!(
        identity
            .runtime_modpack_id
            .contains(crystal_assets::COMPETITIVE_BATTLES_MANIFEST_ID)
    );
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
