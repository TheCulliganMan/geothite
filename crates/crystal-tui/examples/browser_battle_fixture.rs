//! Verification-only save fixture. Never used by production browser gameplay.
//! The browser test imports this local save, then uses only Game Boy inputs.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::models::pokemon::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: browser_battle_fixture PACK OUTPUT_SAVE")?,
    );
    let output = PathBuf::from(args.next().context("missing output save path")?);
    let level = args
        .next()
        .map(|level| level.parse::<u8>())
        .transpose()?
        .unwrap_or(50);
    let trainer = args.next();
    let level_up = trainer.as_deref() == Some("levelup");
    let trainer = trainer.filter(|_| !level_up);
    anyhow::ensure!((1..=100).contains(&level), "fixture level must be 1..=100");
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let loaded = read_loaded_verified_compiled_game_pack(&pack)?;
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let spawn = runtime.title_new_game_spawn_identifier()?;
    let trainer_key = trainer
        .map(|trainer| {
            runtime
                .scripted_trainer_battle_keys()
                .into_iter()
                .find(|key| key.trainer_class == trainer)
                .context("trainer fixture class is missing from the supplied pack")
        })
        .transpose()?;
    let (map, x, y) = trainer_key
        .as_ref()
        .map_or(("Route29", 46, 12), |key| (key.map_name.as_str(), 4, 13));
    let mut fixture = RuntimeGameShell::new_game_at_runtime_tile(root, runtime, spawn, map, x, y)?;
    let species = std::env::var("TUI_FIXTURE_SPECIES").unwrap_or_else(|_| "TOTODILE".into());
    fixture.add_party_pokemon(
        &species,
        level,
        None,
        None,
        "TUI_TEST",
        1,
        Dv::from_non_hp(10, 10, 10, 10),
    )?;
    if level_up {
        let next = fixture.runtime().data().create_pokemon(
            &species,
            level.saturating_add(1).min(100),
            Dv::from_non_hp(10, 10, 10, 10),
        )?;
        let state = fixture.session_mut().state_mut();
        state.storage.party.pokemon[0]
            .as_mut()
            .context("fixture party")?
            .experience = next.experience - 1;
        state.sync_party_from_storage();
    }
    if let Some(key) = trainer_key {
        fixture.start_scripted_trainer_battle(
            &key.map_name,
            &key.source_script,
            key.startbattle_command_index,
        )?;
    }
    fixture.save(output)?;
    Ok(())
}
