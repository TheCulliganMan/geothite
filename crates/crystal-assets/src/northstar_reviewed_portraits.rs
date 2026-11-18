//! Reconcile battle and walking sprites with the accepted Minnesota family sheets.
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};

use crate::minnesota_pokemon_profiles::{self, Roster};
use crate::minnesota_reviewed_overworld::{self, Coverage};
use crate::{CompiledGamePack, minnesota_pokemon_curation, minnesota_pokemon_graphics};

pub(super) fn install(pack: &mut CompiledGamePack, root: &Path, native: &Path) -> Result<()> {
    let mut roster: Roster = serde_json::from_slice(&fs::read(root.join("roster.json"))?)?;
    let mut profiles =
        minnesota_pokemon_profiles::load_profiles(&root.join("species-profiles.json"))?;
    minnesota_pokemon_curation::apply(root, &mut roster, &mut profiles)?;
    for family in &roster.families {
        let files =
            minnesota_pokemon_graphics::load_family_graphics(root, native, family, &profiles)
                .with_context(|| format!("reviewed portraits for {}", family.id))?;
        for member in &family.members {
            let name = member.name.to_ascii_uppercase();
            let number: u16 = member
                .id
                .strip_prefix("MN_")
                .context("Minnesota species ID")?
                .parse()?;
            let species = pack
                .data
                .pokemon
                .get(&name)
                .with_context(|| format!("installed {name}"))?;
            ensure!(
                species.int_id.checked_sub(251) == Some(number),
                "unexpected species number for {name}"
            );
        }
        for (key, reviewed) in files {
            let installed = pack
                .runtime_files
                .get_mut(&key)
                .with_context(|| format!("installed sprite resource {key}"))?;
            if *installed != reviewed {
                *installed = reviewed;
            }
        }
    }
    let overworld = minnesota_reviewed_overworld::load(
        root,
        &root.join("overworld/20261027-v1"),
        pack,
        Coverage::Complete,
    )?;
    pack.runtime_files.extend(overworld.runtime_files);
    Ok(())
}
