//! Numbering-only save checks use isolated generated checkpoints, never player saves.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::save;
use crystal_core::world::collision::PlayerTraversalState;
use crystal_core::world::map::TilePosition;
use serde_json::json;

use crate::{CrystalRuntime, RuntimeGameShell};

fn runtime(path: &Path) -> Result<CrystalRuntime> {
    CrystalRuntime::from_loaded_compiled_pack(
        &AssetRoot::new(path.parent().context("pack directory")?),
        read_loaded_verified_compiled_game_pack(path)?,
    )
}

#[test]
fn dex_order_policy_rejects_unapproved_sources_and_targets() -> Result<()> {
    let raw = "test+northstar-fair-presentation-v1";
    let hosted = format!("{raw}{}", super::SUFFIX);
    let target = format!("{raw}{}", super::MARKER);
    let policy = json!({
        "schema_version": 11,
        "sources": [
            { "modpack_id": raw, "modpack_hash": super::PARENT_HASHES[0], "content_hash": super::PARENT_CONTENT[0] },
            { "modpack_id": hosted, "modpack_hash": super::PARENT_HASHES[1], "content_hash": super::PARENT_CONTENT[1] },
        ],
        "inherited": {},
    });
    for (pointer, replacement) in [
        ("/schema_version", json!(10)),
        ("/sources/0/modpack_hash", json!("0".repeat(64))),
        ("/sources/1/content_hash", json!("0".repeat(64))),
        ("/sources/1/modpack_id", json!(raw)),
    ] {
        let mut invalid = policy.clone();
        *invalid.pointer_mut(pointer).context("test field")? = replacement;
        let error = super::parse(&serde_json::to_vec(&invalid)?, &target)
            .expect_err("unapproved release must be rejected");
        assert!(
            error
                .to_string()
                .contains("expected exact Fair presentation sources")
        );
    }
    let error =
        super::parse(&serde_json::to_vec(&policy)?, raw).expect_err("target marker required");
    assert!(error.to_string().contains("invalid Pokédex-order target"));
    Ok(())
}

#[test]
#[ignore = "requires CRYSTAL_DEX_ORDER_PARENT and CRYSTAL_DEX_ORDER_TARGET external archives"]
fn dex_order_copies_exact_idle_checkpoints_and_preserves_research() -> Result<()> {
    let parent_path =
        PathBuf::from(std::env::var_os("CRYSTAL_DEX_ORDER_PARENT").context("parent")?);
    let target_path =
        PathBuf::from(std::env::var_os("CRYSTAL_DEX_ORDER_TARGET").context("target")?);
    let parent = runtime(&parent_path)?;
    let target = runtime(&target_path)?;
    let sources = crate::compatible_save_sources(&target)?;
    ensure!(
        sources.len() == 24,
        "expected inherited and direct source identities"
    );
    ensure!(
        sources[..2]
            .iter()
            .any(|source| source.modpack_hash == parent.modpack().hash()),
        "expected exact presentation parent"
    );
    let inherited = crate::compatible_save_sources(&parent)?;
    assert_eq!(
        sources[2..]
            .iter()
            .map(|s| (&s.modpack_id, &s.modpack_hash, &s.content_hash))
            .collect::<Vec<_>>(),
        inherited
            .iter()
            .map(|s| (&s.modpack_id, &s.modpack_hash, &s.content_hash))
            .collect::<Vec<_>>(),
        "older save identities must remain supported"
    );
    let root = std::env::temp_dir().join(format!(
        "dex-order-save-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&root)?;
    let result = (|| -> Result<()> {
        for (index, (map, x, y)) in [("MnR10C00R00", 24, 20), ("MnStateFairgrounds", 3, 2)]
            .into_iter()
            .enumerate()
        {
            let mut shell = RuntimeGameShell::new_game_at_runtime_tile(
                AssetRoot::new(parent_path.parent().context("parent directory")?),
                parent.clone(),
                parent.title_new_game_spawn_identifier()?,
                map,
                x,
                y,
            )?;
            let initial = root.join(format!("initial-{index}.crystalsave"));
            shell.save(&initial)?;
            let mut fixture = parent.load_save(&initial)?;
            fixture.script_runtime = Default::default();
            fixture.money = 29870;
            fixture.coins = 672;
            fixture
                .flags
                .set_event_flag("EVENT_MN_GYM_2_TRAINER_2_DEFEATED", true)?;
            let pokemon = parent
                .data()
                .create_pokemon("POUCHID", 17, Default::default())?;
            fixture.pokedex.record_caught_pokemon(&pokemon);
            fixture.storage.party.add_pokemon(pokemon);
            fixture.sync_party_from_storage();
            for species in ["WRIGGLET", "MOSQUITAN", "BABEHEMOTH", "VERGLOOM"] {
                fixture.pokedex.seen_species.insert(species.into());
                fixture.pokedex.caught_species.insert(species.into());
            }
            ensure!(
                fixture
                    .bag
                    .add_item(parent.data().item("POTION")?, 4)
                    .map_err(anyhow::Error::msg)?
            );
            let room = parent.data().overworld_session_for_traversal_with_state(
                map,
                TilePosition::new(x, y),
                fixture.frame_counter,
                PlayerTraversalState::Walk,
                Some(&fixture),
            )?;
            fixture
                .map_object_overrides
                .entry(map.into())
                .or_default()
                .object_structs = room.object_struct_roster_memory()?;
            parent.validate_save_state_for_runtime_pack(&fixture)?;
            super::super::validate_upgrade_checkpoint(&fixture, parent.data())?;
            let source = root.join(format!("source-{index}.crystalsave"));
            let destination = root.join(format!("target-{index}.crystalsave"));
            parent.save_game(&source, fixture.clone())?;
            let original = std::fs::read(&source)?;
            let backup = save::save_backup_path(&source);
            let original_backup = std::fs::read(&backup)?;
            // The second case exercises fallback to the untouched recovery copy.
            if index == 1 {
                std::fs::write(&source, b"invalid primary")?;
            }
            let source_bytes = std::fs::read(&source)?;
            let absent = root.join("absent.crystalsave");
            assert!(crate::prepare_runtime_save_slot(
                &target,
                &destination,
                |candidate| {
                    if candidate.modpack_hash == parent.modpack().hash() {
                        source.clone()
                    } else {
                        absent.clone()
                    }
                }
            )?);
            assert_eq!(std::fs::read(&source)?, source_bytes);
            assert_eq!(std::fs::read(&backup)?, original_backup);
            assert_eq!(
                target.load_save(&destination)?,
                fixture,
                "entire checkpoint preserved on {map}"
            );
            assert!(crate::prepare_runtime_save_slot(
                &target,
                &destination,
                |_| absent.clone()
            )?);
            assert_eq!(
                target.load_save(&destination)?,
                fixture,
                "resume keeps the migrated state"
            );
            let numbers = target.data().minnesota_dex_numbers();
            assert_eq!(numbers["MOSQUITAN"], 220);
            assert_eq!(numbers["BABEHEMOTH"], 221);
            assert_eq!(numbers["VERGLOOM"], 225);
            assert_eq!(original, original_backup);
        }
        if let Some(proof) = std::env::var_os("CRYSTAL_DEX_ORDER_PROOF") {
            let proof = PathBuf::from(proof);
            ensure!(!proof.exists(), "use a fresh proof path");
            std::fs::write(
                proof,
                serde_json::to_vec_pretty(&json!({
                    "parent_pack_sha256": parent.modpack().hash(),
                    "parent_content_identity": parent.pack_identity().content_hash,
                    "target_pack_sha256": target.modpack().hash(),
                    "target_content_identity": target.pack_identity().content_hash,
                    "save_sources": sources.len(),
                    "direct_copies": 2,
                    "complete_checkpoint_preserved": true,
                    "source_primary_and_backup_preserved": true,
                    "recovery_copy_exercised": true,
                }))?,
            )?;
        }
        Ok(())
    })();
    std::fs::remove_dir_all(root)?;
    result
}
