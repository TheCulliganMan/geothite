//! Renumber the public regional Pokédex without changing species or gameplay.

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::{CompiledGamePack, LoadedCompiledGamePack};

const POLICY: &str = "data/northstar-save-compatibility.json";
const RELEASE: &str = "northstar-dex-order-v1";
const PARENT_HASHES: [&str; 2] = [
    "c28b37a15c0ae59151da94d4d70dde6ac3cc066b47afe993665cb071e7a4b3d4",
    "f123f566ed0b70cd9e551a2c88673faac132555e610c3eb5e3ecc95816d1bc57",
];

/// Builds raw and hosted Pokédex-order editions from the published Fair
/// presentation archives. Existing species identities and save membership stay
/// fixed; only displayed regional numbers and release metadata change.
///
/// # Errors
/// Rejects different parent archives, divergent inherited save policies,
/// incomplete numbering, unrelated content changes, or an invalid output pack.
pub fn build_northstar_dex_order(
    raw_parent: &LoadedCompiledGamePack,
    hosted_parent: &LoadedCompiledGamePack,
) -> Result<[CompiledGamePack; 2]> {
    let sources = [raw_parent, hosted_parent];
    let mut identities = Vec::new();
    for (source, expected) in sources.iter().zip(PARENT_HASHES) {
        let identity = source.save_modpack_identity()?;
        ensure!(
            identity.hash() == expected,
            "expected public Fair presentation archive"
        );
        identities.push(json!({
            "modpack_id": identity.id(),
            "modpack_hash": identity.hash(),
            "content_hash": source.pack_content_hash()?,
        }));
    }
    let inherited = raw_parent
        .pack()
        .runtime_files
        .get(POLICY)
        .context("save policy")?;
    ensure!(
        hosted_parent.pack().runtime_files.get(POLICY) == Some(inherited),
        "public save policies differ"
    );
    let inherited: Value = serde_json::from_slice(inherited)?;
    ensure!(
        inherited["schema_version"] == 10,
        "expected Fair presentation policy"
    );
    let policy = serde_json::to_vec(&json!({
        "schema_version": 11,
        "sources": identities,
        "inherited": inherited,
    }))?;
    let mut raw = raw_parent.pack().clone();
    crate::minnesota_dex_roster_sync::install_regional_numbers(&mut raw)?;
    raw.report.manifests.push(RELEASE.into());
    raw.runtime_files.insert(POLICY.into(), policy);
    raw.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        raw.format_version,
        &raw.data,
        &raw.audio_manifest,
        &raw.runtime_files,
        &raw.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&raw)?;
    let all = crate::build_all_pokemon_modpack(&raw)?;
    let battles = crate::build_default_battle_modpack(&all)?;
    let clock = crate::build_realtime_clock_modpack(&battles)?;
    let hosted = crate::build_player_customization_modpack(&clock)?;
    for (parent, child) in sources.iter().zip([&raw, &hosted]) {
        verify_preservation(parent.pack(), child)?;
    }
    Ok([raw, hosted])
}

fn verify_preservation(parent: &CompiledGamePack, child: &CompiledGamePack) -> Result<()> {
    let before = parent.data.minnesota_dex_numbers();
    let after = child.data.minnesota_dex_numbers();
    ensure!(
        before.keys().eq(after.keys()),
        "regional membership changed"
    );
    ensure!(after.len() == 225, "expected 225 active species");
    for (name, number) in [
        ("POUCHID", 1),
        ("KINDLIT", 4),
        ("BIRCHIT", 7),
        ("WRIGGLET", 218),
        ("PUPUMP", 219),
        ("MOSQUITAN", 220),
        ("BABEHEMOTH", 221),
        ("KETTLURK", 222),
        ("VERDIGRIM", 223),
        ("BUNJANT", 224),
        ("VERGLOOM", 225),
    ] {
        ensure!(
            after.get(name) == Some(&number),
            "unexpected number for {name}"
        );
    }
    let mut restored = child.clone();
    for (name, value) in &mut restored.data.story_event_script_constants.global {
        if name.starts_with("MN_DEX_MEMBER_") {
            *value = *parent
                .data
                .story_event_script_constants
                .global
                .get(name)
                .context("new regional species")?;
        }
    }
    restored.runtime_files.insert(
        POLICY.into(),
        parent
            .runtime_files
            .get(POLICY)
            .context("parent policy")?
            .clone(),
    );
    ensure!(
        restored
            .report
            .manifests
            .iter()
            .filter(|m| m.as_str() == RELEASE)
            .count()
            == 1,
        "expected one ordering manifest"
    );
    restored.report.manifests.retain(|m| m != RELEASE);
    restored.identity = parent.identity.clone();
    ensure!(
        &restored == parent,
        "ordering revision changed unrelated content"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};

    #[test]
    #[ignore = "requires CRYSTAL_DEX_ORDER_RAW_PARENT and CRYSTAL_DEX_ORDER_HOSTED_PARENT archives"]
    fn dex_order_preserves_all_other_pack_content() -> Result<()> {
        let read = |key| -> Result<_> {
            crate::read_loaded_verified_compiled_game_pack(
                std::env::var_os(key).with_context(|| format!("missing {key}"))?,
            )
        };
        let raw = read("CRYSTAL_DEX_ORDER_RAW_PARENT")?;
        let hosted = read("CRYSTAL_DEX_ORDER_HOSTED_PARENT")?;
        let editions = super::build_northstar_dex_order(&raw, &hosted)?;
        for (parent, child) in [raw.pack(), hosted.pack()].into_iter().zip(&editions) {
            super::verify_preservation(parent, child)?;
            let numbers = child.data.minnesota_dex_numbers();
            let mut dense: Vec<_> = numbers.values().copied().collect();
            dense.sort_unstable();
            assert_eq!(dense, (1..=225).collect::<Vec<_>>());
            assert!(!numbers.contains_key("ICEMYTH"));
            assert!(!numbers.contains_key("CHIKORITA"));
            assert_ne!(child.identity, parent.identity);
        }
        Ok(())
    }
}
