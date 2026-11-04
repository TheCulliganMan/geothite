//! Apply authored regional typings to species already present in a pack.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-typing-continuity-v1";

/// Updates existing Minnesota species types without modifying their other data.
///
/// # Errors
/// Returns an error for invalid roster data, species identity collisions, invalid
/// type tokens, an already applied revision, or an invalid resulting pack.
pub fn build_minnesota_typing_revision(
    base: &CompiledGamePack,
    roster_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|item| item == MANIFEST),
        "typing revision already installed"
    );
    let roster: Value = serde_json::from_str(roster_json)?;
    let families = roster["families"].as_array().context("roster families")?;
    let mut pack = base.clone();
    let mut seen = BTreeSet::new();
    let mut updated = 0;
    for family in families {
        for member in family["members"].as_array().context("family members")? {
            let id = member["id"].as_str().context("species ID")?;
            let number: u16 = id
                .strip_prefix("MN_")
                .context("regional species ID")?
                .parse()?;
            let name = member["name"]
                .as_str()
                .context("species name")?
                .to_ascii_uppercase();
            ensure!(seen.insert(name.clone()), "duplicate roster species {name}");
            let Some(species) = pack.data.pokemon.get_mut(&name) else {
                continue;
            };
            ensure!(
                species.int_id == number.checked_add(251).context("regional ID overflow")?,
                "regional ID mismatch for {name}"
            );
            let kinds = member["types"].as_array().context("species types")?;
            ensure!(
                (1..=2).contains(&kinds.len()),
                "invalid type count for {name}"
            );
            let resolve = |value: &Value| -> Result<String> {
                let kind = value.as_str().context("type token")?;
                ensure!(
                    [
                        "NORMAL", "FIRE", "WATER", "ELECTRIC", "GRASS", "ICE", "FIGHTING",
                        "POISON", "GROUND", "FLYING", "PSYCHIC", "BUG", "ROCK", "GHOST", "DRAGON",
                        "DARK", "STEEL", "FAIRY"
                    ]
                    .contains(&kind),
                    "invalid type {kind}"
                );
                Ok(if kind == "PSYCHIC" {
                    "PSYCHIC_TYPE"
                } else {
                    kind
                }
                .to_owned())
            };
            let first = resolve(&kinds[0])?;
            let second = if kinds.len() == 2 {
                resolve(&kinds[1])?
            } else {
                first.clone()
            };
            if species.type1 != first || species.type2 != second {
                species.type1 = first;
                species.type2 = second;
                updated += 1;
            }
        }
    }
    ensure!(updated > 0, "no installed species need typing updates");
    pack.report.manifests.push(MANIFEST.to_owned());
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
    use std::collections::BTreeMap;
    use std::path::Path;

    use anyhow::{Context, Result, ensure};
    use serde_json::Value;

    #[test]
    #[ignore = "requires the local selected regional pack and its typing child"]
    fn published_typing_matches_roster_and_preserves_game_data() -> Result<()> {
        let base_path = std::env::var("MINNESOTA_TYPING_BASE")?;
        let child_path = std::env::var("MINNESOTA_TYPING_CHILD")?;
        let base = crate::read_verified_compiled_game_pack(Path::new(&base_path))?;
        let child = crate::read_verified_compiled_game_pack(Path::new(&child_path))?;
        let roster: Value = serde_json::from_str(&std::fs::read_to_string(
            "../../planning/minnesota/pokemon/roster.json",
        )?)?;
        let mut expected = BTreeMap::new();
        for family in roster["families"].as_array().context("families")? {
            for member in family["members"].as_array().context("members")? {
                let name = member["name"]
                    .as_str()
                    .context("name")?
                    .to_ascii_uppercase();
                let types = member["types"].as_array().context("types")?;
                let normalize =
                    |s: &str| if s == "PSYCHIC" { "PSYCHIC_TYPE" } else { s }.to_owned();
                let first = normalize(types[0].as_str().context("first type")?);
                let second = if types.len() == 2 {
                    normalize(types[1].as_str().context("second type")?)
                } else {
                    first.clone()
                };
                expected.insert(name, (first, second));
            }
        }
        let mut before = serde_json::to_value(base.data())?;
        let mut after = serde_json::to_value(child.data())?;
        let base_species = before["pokemon"].as_object_mut().context("base Pokémon")?;
        let child_species = after["pokemon"].as_object_mut().context("child Pokémon")?;
        let mut changed = 0;
        for (name, value) in child_species.iter_mut() {
            let Some((first, second)) = expected.get(name) else {
                continue;
            };
            let old = base_species
                .get_mut(name)
                .context("matching base species")?;
            ensure!(
                value["type1"] == *first && value["type2"] == *second,
                "{name} types differ"
            );
            if old["type1"] != value["type1"] || old["type2"] != value["type2"] {
                changed += 1;
            }
            old["type1"] = value["type1"].clone();
            old["type2"] = value["type2"].clone();
        }
        ensure!(changed > 0, "no typing changes found");
        ensure!(before == after, "unrelated game data changed");
        println!("Verified {changed} species typing updates and all remaining game data");
        Ok(())
    }
}
