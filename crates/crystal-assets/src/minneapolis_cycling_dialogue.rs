//! Text-only revision of the installed Minneapolis cycling cast.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

const CITY: &str = "MnMinneapolis";

#[derive(Deserialize)]
struct Source {
    schema_version: u8,
    trainers: Vec<Rider>,
}

#[derive(Deserialize)]
struct Rider {
    id: String,
    map: String,
    sprite: String,
    optional_battle: bool,
    before_lines: Vec<String>,
    after_lines: Vec<String>,
    loss_lines: Vec<String>,
}

/// Refreshes only the native text of all 18 installed Minneapolis cyclists.
/// Existing teams, patrols, battle programs, city services and progression stay
/// unchanged. Applying the same source again is idempotent.
///
/// # Errors
/// Rejects invalid packs, incomplete or duplicate rider identities, incompatible
/// actors, non-text target programs and invalid native dialogue lines.
pub fn refresh_minneapolis_cycling_dialogue(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let source: Source = serde_json::from_str(source)?;
    let expected: BTreeSet<_> = (1..=18)
        .map(|index| format!("MN_EXTRA_MPLS_CYCLE_{index:02}"))
        .collect();
    ensure!(
        source.schema_version == 1
            && source.trainers.len() == expected.len()
            && source
                .trainers
                .iter()
                .map(|r| r.id.clone())
                .collect::<BTreeSet<_>>()
                == expected,
        "complete existing cycling cast required"
    );
    let mut pack = base.clone();
    let mut city = base.data.maps.get(CITY).context("installed city")?.clone();
    for rider in source.trainers {
        ensure!(
            rider.map == CITY
                && rider.optional_battle
                && matches!(
                    rider.sprite.as_str(),
                    "SPRITE_CHRIS_BIKE" | "SPRITE_KRIS_BIKE"
                ),
            "native optional city cyclist required"
        );
        let actors: Vec<_> = city
            .objects
            .iter()
            .filter(|o| o.script == rider.id)
            .collect();
        ensure!(
            actors.len() == 1
                && actors[0].sprite == rider.sprite
                && actors[0].object_type == "OBJECTTYPE_SCRIPT",
            "installed optional rider required: {}",
            rider.id
        );
        for (suffix, lines) in [
            ("Seen", rider.before_lines),
            ("Won", rider.after_lines),
            ("Lost", rider.loss_lines),
        ] {
            let label = format!("{}{suffix}", rider.id);
            ensure!(
                !lines.is_empty()
                    && lines.iter().all(|line| !line.is_empty()
                        && line.len() <= 18
                        && line.is_ascii()
                        && !line
                            .chars()
                            .any(|c| c.is_control() || matches!(c, '"' | '<' | '>'))),
                "invalid native cycling text: {label}"
            );
            let old = city
                .scripts
                .get(&label)
                .and_then(serde_json::Value::as_array)
                .context("installed cycling text")?;
            ensure!(
                old.iter().all(|row| matches!(
                    row["command"].as_str(),
                    Some("text" | "line" | "para" | "cont" | "done" | "prompt")
                )),
                "cycling revision must target text only"
            );
            authoring::text(
                &mut city,
                &label,
                &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
            pack.data
                .map_scripts
                .insert(label.clone(), city.scripts[&label].clone());
        }
    }
    city.script_text_commands = crate::parse_script_text_commands(CITY, &city.scripts)?;
    city.script_text_bodies = crate::parse_script_text_bodies(CITY, &city.scripts)?;
    pack.data.maps.insert(CITY.into(), city);
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
