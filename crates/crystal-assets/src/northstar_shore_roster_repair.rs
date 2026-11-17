//! Replace the archived shore partner without changing saved actor identities.

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, MapModule};

pub(crate) const PARTNER_SPRITE: &str = "SPRITE_CHORIP";

pub(crate) fn validate_partner(pack: &CompiledGamePack) -> Result<()> {
    ensure!(
        RegionalRoster::current()?.allowed.contains("CHORIP")
            && pack
                .runtime_files
                .contains_key("gfx/pokemon/chorip/overworld.2bpp"),
        "shore partner requires active Chorip and its native overworld art"
    );
    Ok(())
}

/// Check named creature actors against the same curated policy as parties.
pub(crate) fn validate_visible_species(pack: &CompiledGamePack, map: &MapModule) -> Result<()> {
    let roster = RegionalRoster::current()?;
    for actor in &map.objects {
        let Some(species) = actor.sprite.strip_prefix("SPRITE_") else {
            continue;
        };
        if pack.data.pokemon.contains_key(species) {
            ensure!(
                roster.allowed.contains(species),
                "{} has an inactive regional creature actor: {species}",
                map.id
            );
        }
    }
    Ok(())
}

pub(crate) fn repair_existing(pack: &mut CompiledGamePack, story: &str) -> Result<()> {
    validate_partner(pack)?;
    let source: Value = serde_json::from_str(story)?;
    for (map_id, actor_id) in [
        ("MnWinona", "MN_RESTORE_WINONA_GILLPUP"),
        ("MnLeechLake", "MN_RESTORE_LEECH_GILLPUP"),
    ] {
        let mut map = pack.data.maps.get(map_id).context("shore map")?.clone();
        let actor = map
            .objects
            .iter_mut()
            .find(|actor| actor.object_identifier.as_deref() == Some(actor_id))
            .context("saved shore partner identity")?;
        ensure!(actor.sprite == "SPRITE_GILLPUP", "shore actor changed");
        // The identifier is retained for existing visibility and talk records.
        actor.sprite = PARTNER_SPRITE.into();
        for (label, pages) in source["refinement_aftermath"]["dialogue"][map_id]
            .as_object()
            .context("canonical shore pages")?
        {
            ensure!(
                map.scripts.contains_key(label),
                "missing shore text {label}"
            );
            let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(pages.clone())?;
            map.scripts
                .insert(label.clone(), authored_dialogue::compile(&pages)?);
        }
        validate_visible_species(pack, &map)?;
        crate::northstar_story_revision::publish(pack, map)?;
    }
    pack.report
        .manifests
        .push("northstar-shore-roster-repair-v1".into());
    Ok(())
}
