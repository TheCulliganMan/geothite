//! Optional city paper with stable, seed-selected native wild disguises.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-city-note-mimics-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Notes {
    schema_version: u8,
    seed: u64,
    mimic_numerator: usize,
    mimic_denominator: usize,
    cities: Vec<City>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct City {
    map: String,
    code: String,
    entry: [u16; 2],
    notes: Vec<[u16; 2]>,
    encounters: Vec<Encounter>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Encounter {
    species: String,
    level: u8,
    scale_level: bool,
}

// Native slots are deliberately separate from challenge slots. This exact
// namespace is authored only by this overlay, never by a runtime random roll.
pub(crate) fn preserves_authored_level(map: &str, script: &str) -> bool {
    matches!(map, "MnMinneapolis" | "MnStPaul" | "MnDuluth")
        && script.starts_with("MnCityNote")
        && script.ends_with("Native")
}

fn selected_notes(source: &Notes, city: &City) -> BTreeSet<usize> {
    let mut ranked = (0..city.notes.len())
        .map(|index| {
            let digest = Sha256::digest(format!(
                "city-note-v1/{}/{}/{}",
                source.seed, city.map, index
            ));
            (digest, index)
        })
        .collect::<Vec<_>>();
    ranked.sort_unstable();
    ranked
        .into_iter()
        .take(city.notes.len() * source.mimic_numerator / source.mimic_denominator)
        .map(|(_, index)| index)
        .collect()
}

fn validate(source: &Notes) -> Result<()> {
    ensure!(
        source.schema_version == 1
            && source.mimic_denominator > 0
            && source.mimic_denominator <= 100
            && source.mimic_numerator > 0
            && source.mimic_numerator <= 100
            && source.mimic_numerator * 2 < source.mimic_denominator,
        "city notes require a strict minority of mimics"
    );
    let mut maps = BTreeSet::new();
    let mut codes = BTreeSet::new();
    for city in &source.cities {
        ensure!(
            matches!(city.map.as_str(), "MnMinneapolis" | "MnStPaul" | "MnDuluth")
                && maps.insert(&city.map)
                && codes.insert(&city.code)
                && !city.code.is_empty()
                && city.code.bytes().all(|b| b.is_ascii_uppercase()),
            "unique supported city/code required"
        );
        let count = city.notes.len() * source.mimic_numerator / source.mimic_denominator;
        ensure!(
            city.notes.len() <= 12
                && !city.encounters.is_empty()
                && count >= city.encounters.len()
                && count > 0,
            "each city must retain its encounter choices and ordinary notes"
        );
        ensure!(
            city.encounters
                .iter()
                .any(|e| e.species == "JOTLET" && !e.scale_level),
            "every city needs a genuinely low-level basic-form slot"
        );
        for encounter in &city.encounters {
            let minimum = match encounter.species.as_str() {
                "JOTLET" => 1,
                "MEMOFOLD" => 24,
                "RECALLISK" => 44,
                _ => anyhow::bail!("only the existing Jotlet family may mimic notes"),
            };
            ensure!(
                (minimum..=60).contains(&encounter.level),
                "invalid family level"
            );
            ensure!(
                encounter.scale_level || (encounter.species == "JOTLET" && encounter.level <= 12),
                "native-level refuge must remain a low-level Jotlet"
            );
        }
    }
    ensure!(!source.cities.is_empty(), "at least one city required");
    Ok(())
}

/// Adds optional note discoveries without replacing any existing map interaction.
///
/// A configured minority of new papers are wild Jotlet-family encounters. A
/// stable SHA-256 ordering selects the papers at pack build time; reloads,
/// repeated reads and map visits never reroll them or consume gameplay RNG.
/// Acknowledging the reveal consumes the mimic before battle starts. Catch,
/// defeat, flee and blackout all leave it gone. In-progress saves use the
/// ordinary script/battle cursor; older saves before discovery see the same
/// species again. No items or story flags are awarded. Native Jotlet slots
/// retain their low authored level even with a high-level party.
///
/// # Errors
/// Rejects unsupported/occupied cities or identities, unsafe placements,
/// non-active family species, duplicate publication, invalid native scripts,
/// or invalid resulting pack data. The borrowed parent is unchanged on error.
pub fn build_city_note_mimics(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "city notes already installed"
    );
    let source: Notes = serde_json::from_str(source)?;
    validate(&source)?;
    ensure!(
        base.data
            .maps
            .get("NewBarkTown")
            .is_some_and(|map| !map.objects.is_empty()),
        "native NPC template required"
    );
    let policy = crate::authored_regional_approach::policy()?;
    let mut pack = base.clone();
    for city in &source.cities {
        let mut map = pack
            .data
            .maps
            .get(&city.map)
            .context("installed city")?
            .clone();
        ensure!(
            map.attributes.environment.as_deref() == Some("TOWN"),
            "outdoor city required"
        );
        let before = city_reachable(&pack, &map, city.entry)?;
        let mut tiles = BTreeSet::new();
        let selected = selected_notes(&source, city);
        let mut encounter_index = 0;
        for (index, &at) in city.notes.iter().enumerate() {
            ensure!(
                at != city.entry
                    && tiles.insert(at)
                    && before.contains(&at)
                    && kind(&pack, &map, at)? == "FLOOR"
                    && !map.events.warps.iter().any(|w| [w.x, w.y] == at)
                    && !map.events.coord_events.iter().any(|e| [e.x, e.y] == at)
                    && !map.events.bg_events.iter().any(|e| [e.x, e.y] == at),
                "note must occupy a unique reachable optional floor: {} {at:?}",
                city.map
            );
            let id = format!("MN_CITY_NOTE_{}_{:02}", city.code, index + 1);
            let label = format!("MnCityNote{}{:02}", city.code, index + 1);
            ensure!(
                !map.objects
                    .iter()
                    .any(|o| o.object_identifier.as_deref() == Some(&id))
                    && !pack.data.map_scripts.keys().any(|s| s.starts_with(&label)),
                "occupied note identity {id}"
            );
            authoring::npc(
                &pack,
                &mut map,
                &label,
                "SPRITE_PAPER",
                at[0].try_into()?,
                at[1].try_into()?,
            )?;
            let object = map.objects.last_mut().context("new paper")?;
            object.object_identifier = Some(id.clone());
            object.sprite_has_facings = false;
            object.spritemovedata = "SPRITEMOVEDATA_STILL".into();
            if selected.contains(&index) {
                let encounter = &city.encounters[encounter_index % city.encounters.len()];
                encounter_index += 1;
                ensure!(
                    policy.target(&pack, &encounter.species, Some(encounter.level))?
                        == encounter.species,
                    "active regional family member required"
                );
                install_mimic(&mut pack, &mut map, &label, &id, encounter)?;
            } else {
                let reminder = [
                    "Call home tonight.",
                    "Return that book.",
                    "Water the fern.",
                    "Bring spare mitts.",
                ][index % 4];
                authoring::text(
                    &mut map,
                    &format!("{label}Text"),
                    &["A loose reminder:", reminder],
                );
                map.scripts.insert(
                    label.clone(),
                    json!([c("jumptext", &[&format!("{label}Text")])]),
                );
            }
        }
        let after = city_reachable(&pack, &map, city.entry)?;
        ensure!(
            before.difference(&after).all(|tile| tiles.contains(tile)),
            "notes must not block city travel: {}",
            city.map
        );
        crate::ely_league_rooms::finish(&pack, &mut map)?;
        map.scripted_wild_battles = crate::parse_scripted_wild_battles(&map.id, &map.scripts)?;
        map.script_audio_commands = crate::parse_script_audio_commands(&map.id, &map.scripts)?;
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    pack.report.manifests.push(MANIFEST.into());
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

// Use native numeric bridge/ledge permissions rather than a FLOOR-only flood.
fn city_reachable(
    pack: &CompiledGamePack,
    map: &MapModule,
    entry: [u16; 2],
) -> Result<BTreeSet<[u16; 2]>> {
    let occupied = map.objects.iter().map(|o| [o.x, o.y]).collect();
    Ok(
        crate::authored_route_surface::walking_map_distances(pack, map, entry, &occupied)?
            .into_keys()
            .collect(),
    )
}

fn install_mimic(
    pack: &mut CompiledGamePack,
    map: &mut MapModule,
    label: &str,
    id: &str,
    encounter: &Encounter,
) -> Result<()> {
    let flag = format!("EVENT_{id}_REVEALED");
    crate::northstar_lair_authoring::flag(pack, &flag)?;
    map.objects.last_mut().context("paper object")?.event_flag = flag.clone();
    let battle = format!(
        "{label}{}",
        if encounter.scale_level {
            "Challenge"
        } else {
            "Native"
        }
    );
    let text = format!("{label}RevealText");
    let empty = format!("{label}NoParty");
    let end = format!("{label}End");
    authoring::text(map, &text, &["The note moves!", "It's a POKEMON!"]);
    authoring::text(
        map,
        &format!("{empty}Text"),
        &["The paper quivers.", "Bring a POKEMON."],
    );
    map.scripts.insert(
        empty.clone(),
        json!([c("jumptext", &[&format!("{empty}Text")])]),
    );
    map.scripts.insert(end.clone(), json!([c("end", &[])]));
    map.scripts.insert(
        label.into(),
        json!([
            c("checkevent", &[&flag]),
            c("iftrue", &[&end]),
            c("readvar", &["VAR_PARTYCOUNT"]),
            c("ifequal", &["0", &empty]),
            c("opentext", &[]),
            c("writetext", &[&text]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("setevent", &[&flag]),
            c("disappear", &[id]),
            c("sjump", &[&battle]),
        ]),
    );
    map.scripts.insert(
        battle,
        json!([
            c("loadvar", &["VAR_BATTLETYPE", "BATTLETYPE_NORMAL"]),
            c(
                "loadwildmon",
                &[&encounter.species, &encounter.level.to_string()]
            ),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("end", &[]),
        ]),
    );
    Ok(())
}

#[cfg(test)]
#[path = "city_note_mimics_tests.rs"]
mod tests;
