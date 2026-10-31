//! Local biome corrections for the installed Minnesota encounter surfaces.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{TimeOfDay, WildEncounterTable};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

const MANIFEST: &str = "minnesota-biome-ecology-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema_version: u8,
    id: String,
    parent_content_hash: String,
    reference_structure: String,
    grass: Vec<Site>,
    surf: Vec<Site>,
    fishing: Vec<crate::minnesota_ecology_fishing::Site>,
    rod_recipes: BTreeMap<String, crate::minnesota_ecology_fishing::Recipe>,
    headbutt: Headbutt,
    hints: Vec<Hint>,
    retired_water: Vec<RetiredWater>,
    habitat_review: crate::minnesota_habitat_review::Review,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RetiredWater {
    map: String,
    reason: String,
    before_group: String,
    before_rate: u8,
    before_table: WildEncounterTable,
    water_cells: usize,
    outer_water: BTreeSet<[u16; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Hint {
    map: String,
    label: String,
    before: Vec<String>,
    after: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Site {
    map: String,
    theme: String,
    sample: [u16; 2],
    #[serde(default)]
    shore: Option<[u16; 2]>,
    #[serde(rename = "loop")]
    next: [u16; 2],
    edits: Vec<Edit>,
    replay: Replay,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    time: String,
    slot: usize,
    before: String,
    after: String,
    level: Option<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replay {
    time: String,
    species: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Headbutt {
    map: String,
    replacements: BTreeMap<String, String>,
    sleep_turns_by_time: BTreeMap<String, BTreeMap<TimeOfDay, u8>>,
}

/// Corrects misplaced ordinary creatures in base tables and authored local zones.
///
/// Walking and Surf slots use explicit species preimages. Native rates, weights
/// and levels remain unchanged except the declared relocated shipwreck level.
/// Upgraded rods receive private groups; their original Old Rods, thresholds,
/// levels, catalogs and swarm flag semantics remain intact. The original oak
/// Headbutt weights survive with forest bird/squirrel identities and sleep times.
/// Explicit habitat relocations correct inherited local-zone residents and
/// enforce forest, mine, lighthouse, smokehouse and State Fair restrictions.
/// Two informational wildlife hints follow their local replacements. Terrain,
/// story progression, trainers, native art and audio are unchanged. The clinic
/// rhythm companion moves from a river zone to a free Rochester resting-house
/// offer after the first parcel; refusal/capacity failure preserve its claim.
/// The sealed Route 2 drainage has no reachable bank; its inert water table
/// and fishing header are retired after checking the entire native boundary.
///
/// # Errors
/// Rejects stale or incomplete plans, duplicate edits, invalid native surfaces,
/// occupied namespaces, inactive/nonregional species, illegal evolution levels,
/// lost family coverage or invalid compiled output. The parent is unchanged.
pub fn build_minnesota_biome_ecology(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let source: Source = serde_json::from_str(source)?;
    ensure!(
        source.schema_version == 1
            && source.id == MANIFEST
            && source.parent_content_hash == base.identity()?.content_hash
            && !source.reference_structure.is_empty()
            && source.grass.len() == 66
            && source.surf.len() == 67
            && source.fishing.len() == 45
            && source.rod_recipes.len() == 6
            && source.hints.len() == 2
            && source.retired_water.len() == 1
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "complete current-parent biome plan required"
    );
    let policy = crate::authored_regional_approach::policy()?;
    let mut pack = base.clone();
    install_sites(&mut pack, &source.grass, false, &policy)?;
    install_sites(&mut pack, &source.surf, true, &policy)?;
    crate::minnesota_ecology_fishing::install(
        &mut pack,
        &source.fishing,
        &source.rod_recipes,
        &policy,
    )?;
    install_headbutt(&mut pack, &source.headbutt, &policy)?;
    install_hints(&mut pack, &source.hints)?;
    retire_water(&mut pack, &source.retired_water)?;
    crate::minnesota_clinic_companion::install(&mut pack)?;
    crate::minnesota_habitat_review::install(&mut pack, &source.habitat_review)?;
    crate::minnesota_ecology_coverage::preserve(base, &pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    Ok(pack)
}

fn retire_water(pack: &mut CompiledGamePack, sites: &[RetiredWater]) -> Result<()> {
    for site in sites {
        ensure!(
            site.map == "MnR02C00R00" && !site.reason.is_empty(),
            "reviewed drainage retirement required"
        );
        let mut map = pack
            .data
            .maps
            .get(&site.map)
            .context("drainage map")?
            .clone();
        let mut data = pack.data.wild_encounters[&site.map].clone();
        ensure!(
            map.attributes.fishing_group.as_deref() == Some(site.before_group.as_str())
                && data.water_rate == Some(site.before_rate)
                && data.water.as_ref() == Some(&site.before_table),
            "drainage encounter preimage differs"
        );
        let width = map.attributes.width * 2;
        let height = map.attributes.height * 2;
        let mut count = 0;
        let mut outer = BTreeSet::new();
        for y in 0..height {
            for x in 0..width {
                if kind(pack, &map, [x, y])? != "WATER" {
                    continue;
                }
                count += 1;
                if x == 0 || y == 0 || x + 1 == width || y + 1 == height {
                    outer.insert([x, y]);
                }
                for [nx, ny] in [
                    [x.checked_sub(1), Some(y)],
                    [x.checked_add(1), Some(y)],
                    [Some(x), y.checked_sub(1)],
                    [Some(x), y.checked_add(1)],
                ] {
                    let (Some(nx), Some(ny)) = (nx, ny) else {
                        continue;
                    };
                    if nx >= width || ny >= height {
                        continue;
                    }
                    let terrain = kind(pack, &map, [nx, ny])?;
                    ensure!(
                        matches!(terrain, "WALL" | "WATER"),
                        "drainage has a native land or Surf access at {nx},{ny}"
                    );
                }
            }
        }
        ensure!(
            count == site.water_cells
                && count == 152
                && outer == site.outer_water
                && outer.len() == 8,
            "reviewed sealed drainage geometry required"
        );
        data.water = None;
        data.water_rate = None;
        map.attributes.fishing_group = Some("FISHGROUP_NONE".into());
        pack.data
            .map_attributes
            .get_mut(&site.map)
            .context("drainage header mirror")?
            .fishing_group = Some("FISHGROUP_NONE".into());
        pack.data.maps.insert(site.map.clone(), map);
        pack.data.wild_encounters.insert(site.map.clone(), data);
    }
    Ok(())
}

fn install_hints(pack: &mut CompiledGamePack, hints: &[Hint]) -> Result<()> {
    let expected = BTreeSet::from([
        ("MnR13C00R01", "MN_R13_BIRD_WATCHERText"),
        ("MnR17C00R00", "MN_R17_OLD_PINES_WATCHERText"),
    ]);
    ensure!(
        hints
            .iter()
            .map(|h| (h.map.as_str(), h.label.as_str()))
            .collect::<BTreeSet<_>>()
            == expected,
        "only the two reviewed wildlife hints may change"
    );
    for hint in hints {
        ensure!(
            !hint.after.is_empty() && hint.after.iter().all(|s| s.chars().count() <= 18),
            "native hint width"
        );
        let mut map = pack
            .data
            .maps
            .get(&hint.map)
            .context("wildlife hint map")?
            .clone();
        let original = map
            .script_text_bodies
            .get(&hint.label)
            .context("wildlife hint body")?;
        let lines = original
            .commands
            .iter()
            .filter(|c| matches!(c.command.as_str(), "text" | "line" | "para"))
            .filter_map(|c| c.args.first())
            .map(|s| s.trim_matches('"'))
            .collect::<Vec<_>>();
        ensure!(lines == hint.before, "stale wildlife hint preimage");
        crate::grand_marais_modpack::authoring::text(
            &mut map,
            &hint.label,
            &hint.after.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        let payload = map.scripts[&hint.label].clone();
        let mut parsed = crate::parse_script_text_bodies(
            &hint.map,
            &BTreeMap::from([(hint.label.clone(), payload.clone())]),
        )?;
        map.script_text_bodies.insert(
            hint.label.clone(),
            parsed
                .remove(&hint.label)
                .context("compiled wildlife hint")?,
        );
        pack.data.map_scripts.insert(hint.label.clone(), payload);
        pack.data.maps.insert(hint.map.clone(), map);
    }
    Ok(())
}

fn slots_mut<'a>(
    table: &'a mut WildEncounterTable,
    time: &str,
) -> Result<&'a mut Vec<crystal_core::world::encounters::WildEncounter>> {
    match time {
        "morning" => Ok(&mut table.morning),
        "day" => Ok(&mut table.day),
        "night" => Ok(&mut table.night),
        _ => anyhow::bail!("unknown native encounter time"),
    }
}

fn install_sites(
    pack: &mut CompiledGamePack,
    sites: &[Site],
    water: bool,
    policy: &crate::northstar_regional_roster::RegionalRoster,
) -> Result<()> {
    let mut maps = BTreeSet::new();
    for site in sites {
        ensure!(
            site.map.starts_with("Mn")
                && maps.insert(&site.map)
                && !site.theme.is_empty()
                && !site.edits.is_empty()
                && site.sample[0].abs_diff(site.next[0]) + site.sample[1].abs_diff(site.next[1])
                    == 1,
            "unique complete regional site required"
        );
        let map = pack.data.maps.get(&site.map).context("biome map")?;
        if let Some(shore) = site.shore {
            ensure!(
                water
                    && kind(pack, map, shore)? == "FLOOR"
                    && shore[0].abs_diff(site.sample[0]) + shore[1].abs_diff(site.sample[1]) == 1
                    && !map.objects.iter().any(|o| [o.x, o.y] == shore),
                "declared native Surf shore must be adjacent and unoccupied"
            );
        }
        for at in [site.sample, site.next] {
            ensure!(
                kind(pack, map, at)? == if water { "WATER" } else { "TALL_GRASS" }
                    && !map.objects.iter().any(|o| [o.x, o.y] == at),
                "unoccupied native encounter loop required: {} {at:?}",
                site.map
            );
        }
        let mut data = pack.data.wild_encounters[&site.map].clone();
        if !water {
            for at in [site.sample, site.next] {
                ensure!(
                    data.zone_at(at[0].try_into()?, at[1].try_into()?).is_none(),
                    "base-grass review cannot overwrite an authored zone"
                );
            }
        }
        let original = data.clone();
        let table = if water {
            ensure!(data.water_rate.is_some_and(|r| r > 0), "active Surf rate");
            data.water.as_mut().context("Surf table")?
        } else {
            ensure!(
                data.grass_rates.as_ref().is_some_and(|rates| {
                    ["morning", "day", "night"]
                        .iter()
                        .all(|t| rates.get(*t).is_some_and(|r| *r > 0))
                }),
                "active native walking rates"
            );
            data.grass.as_mut().context("grass table")?
        };
        let mut edited = BTreeSet::new();
        for edit in &site.edits {
            ensure!(
                edited.insert((&edit.time, edit.slot)),
                "duplicate encounter edit"
            );
            let slots = slots_mut(table, &edit.time)?;
            ensure!(
                slots.len() == if water { 3 } else { 7 },
                "native slot extent"
            );
            let slot = slots.get_mut(edit.slot).context("native slot")?;
            ensure!(
                slot.species == edit.before && edit.before != edit.after,
                "slot species preimage"
            );
            let level = edit.level.unwrap_or(slot.level);
            validate_species(pack, policy, &edit.after, level)?;
            ensure!(
                edit.level.is_none() || (water && edit.after == "WRECKQUIEM" && level == 41),
                "only relocated shipwreck changes level"
            );
            slot.species.clone_from(&edit.after);
            slot.level = level;
        }
        ensure!(
            slots_mut(table, &site.replay.time)?
                .iter()
                .any(|s| s.species == site.replay.species)
                && site
                    .edits
                    .iter()
                    .any(|e| e.time == site.replay.time && e.after == site.replay.species),
            "declared replay must select an edited local resident"
        );
        ensure!(
            data.grass_rates == original.grass_rates
                && data.water_rate == original.water_rate
                && data.zones == original.zones
                && data.swarm_overrides == original.swarm_overrides
                && (water || data.water == original.water)
                && (!water || data.grass == original.grass),
            "unowned encounter components changed"
        );
        pack.data.wild_encounters.insert(site.map.clone(), data);
    }
    Ok(())
}

pub(super) fn validate_species(
    pack: &CompiledGamePack,
    policy: &crate::northstar_regional_roster::RegionalRoster,
    species: &str,
    level: u8,
) -> Result<()> {
    ensure!(
        (1..=100).contains(&level)
            && policy.target(pack, species, Some(level))? == species
            && pack
                .data
                .story_event_script_constants
                .global
                .contains_key(&format!("MN_DEX_MEMBER_{species}")),
        "active regional species at a legal level required: {species}@{level}"
    );
    Ok(())
}

fn install_headbutt(
    pack: &mut CompiledGamePack,
    source: &Headbutt,
    policy: &crate::northstar_regional_roster::RegionalRoster,
) -> Result<()> {
    ensure!(
        source.map == "MnR19C00R00" && source.replacements.len() == 2,
        "reviewed oak habitat"
    );
    let mut data = pack.data.field_encounters[&source.map].clone();
    let table = data
        .tables
        .get_mut("headbutt")
        .context("oak Headbutt table")?;
    let mut count = 0;
    for slot in table.common.iter_mut().chain(&mut table.rare) {
        if let Some(target) = source.replacements.get(&slot.species) {
            validate_species(pack, policy, target, slot.level)?;
            slot.species.clone_from(target);
            slot.sleep_turns_by_time.clone_from(
                source
                    .sleep_turns_by_time
                    .get(target)
                    .context("native sleep schedule")?,
            );
            count += 1;
        }
    }
    ensure!(count == 4, "four forest tree identities required");
    pack.data.field_encounters.insert(source.map.clone(), data);
    Ok(())
}

#[cfg(test)]
#[path = "minnesota_biome_ecology_tests.rs"]
mod tests;
