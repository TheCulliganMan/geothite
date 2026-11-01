//! Solo completion overlays for every client. Content remains in the supplied pack.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::systems::evolution::{EvolutionEntry, METHOD_TRADE, TRADE_ANY_ITEM};
use serde::Deserialize;

use crate::{
    CompiledGamePack, derive_compiled_game_pack_identity_from_manifest,
    verify_compiled_game_pack_for_runtime,
};

pub const ALL_251_MANIFEST_ID: &str = "all-251-catchable-v1";
pub const SOLO_EVOLUTION_MANIFEST_ID: &str = "solo-evolutions-v1";
pub const LINK_CABLE_ITEM_ID: &str = "LINK_CABLE";
pub const STONE_MART_ID: &str = "MART_CELADON_4F";
pub const EVOLUTION_STONES: &[&str] = &[
    "FIRE_STONE",
    "WATER_STONE",
    "THUNDERSTONE",
    "LEAF_STONE",
    "MOON_STONE",
    "SUN_STONE",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    map: String,
    surface: String,
    slot: usize,
    species: String,
    level: u8,
}

pub fn build_all_251_catchable_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let placements: Vec<Placement> = serde_json::from_str(include_str!(
        "../../../modpacks/all-pokemon/encounters.json"
    ))?;
    let mut encounters = base.data().wild_encounters.clone();
    let mut occupied = BTreeSet::new();
    for placement in placements {
        ensure!(
            occupied.insert((
                placement.map.clone(),
                placement.surface.clone(),
                placement.slot
            )),
            "duplicate encounter slot"
        );
        ensure!(
            (1..=100).contains(&placement.level),
            "invalid encounter level"
        );
        ensure!(
            base.data().pokemon.contains_key(&placement.species),
            "unknown species {}",
            placement.species
        );
        let table = encounters
            .get_mut(&placement.map)
            .with_context(|| format!("missing encounter map {}", placement.map))?;
        let encounter_table = match placement.surface.as_str() {
            "grass" => table.grass.as_mut(),
            "water" => table.water.as_mut(),
            _ => anyhow::bail!("unknown encounter surface"),
        }
        .context("placement requires an existing encounter table")?;
        for slots in [
            &mut encounter_table.morning,
            &mut encounter_table.day,
            &mut encounter_table.night,
        ] {
            let slot = slots
                .get_mut(placement.slot)
                .context("missing encounter slot")?;
            slot.species = placement.species.clone();
            slot.level = placement.level;
        }
    }
    let pack = base.with_wild_encounter_overlay(ALL_251_MANIFEST_ID, encounters)?;
    verify_all_251_coverage(&pack)?;
    Ok(pack)
}

/// Every Gen 2 species must have a renewable, nonzero-rate grass or Surf encounter.
/// Vanilla day/night differences remain meaningful; additions appear in all three periods.
pub fn verify_all_251_coverage(pack: &CompiledGamePack) -> Result<()> {
    let expected = pack
        .data()
        .pokemon
        .values()
        .filter(|p| (1..=251).contains(&p.int_id))
        .collect::<Vec<_>>();
    ensure!(
        expected.len() == 251,
        "all-251 requires the complete Gen 2 catalog"
    );
    ensure!(
        expected
            .iter()
            .map(|species| species.int_id)
            .collect::<BTreeSet<_>>()
            == (1..=251).collect(),
        "all-251 requires each Gen 2 Pokédex number exactly once"
    );
    ensure!(
        expected.iter().all(|species| species.catch_rate > 0),
        "all-251 requires nonzero capture rates"
    );
    let mut found = BTreeSet::new();
    for (map, table) in &pack.data().wild_encounters {
        ensure!(pack.data().maps.contains_key(map), "unknown map {map}");
        for (surface, encounters) in [("grass", &table.grass), ("water", &table.water)] {
            let Some(encounters) = encounters else {
                continue;
            };
            for (period, slots) in [
                ("morning", &encounters.morning),
                ("day", &encounters.day),
                ("night", &encounters.night),
            ] {
                let rate = if surface == "grass" {
                    table
                        .grass_rates
                        .as_ref()
                        .and_then(|rates| rates.get(period))
                        .copied()
                } else {
                    table.water_rate
                };
                if !rate.is_some_and(|rate| rate > 0) {
                    continue;
                }
                for slot in slots {
                    found.insert(slot.species.as_str());
                }
            }
        }
    }
    let missing = expected
        .iter()
        .filter(|p| !found.contains(p.id.as_str()))
        .map(|p| p.id.as_str())
        .collect::<Vec<_>>();
    ensure!(
        missing.is_empty(),
        "uncatchable species: {}",
        missing.join(", ")
    );
    Ok(())
}

/// Add a consumable Link Cable beside the evolution stones at Celadon's 4F
/// counter. Real trade rules remain available; solo item routes retain and
/// consume any required held item exactly as a trade would.
pub fn build_solo_evolution_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == SOLO_EVOLUTION_MANIFEST_ID),
        "solo evolution overlay already applied"
    );
    ensure!(
        !base.data.items.contains_key(LINK_CABLE_ITEM_ID),
        "Link Cable item already exists"
    );
    let mut pack = base.clone();
    let mut cable = pack
        .data
        .items
        .get("FIRE_STONE")
        .context("missing evolution stone template")?
        .clone();
    cable.name = "LINK CABLE".into();
    cable.description = "Evolves trade Pokémon. Hold any required trade item.".into();
    cable.script_name = LINK_CABLE_ITEM_ID.into();
    cable.price = 2100;
    pack.data.items.insert(LINK_CABLE_ITEM_ID.into(), cable);
    pack.report.items = pack.data.items.len();
    // The source Moon Stone has price zero, which disables the production
    // shop quantity selector. Give the newly sold stone the other stones' price.
    pack.data
        .items
        .get_mut("MOON_STONE")
        .context("missing Moon Stone")?
        .price = 2100;
    let stock = pack
        .data
        .marts
        .0
        .get_mut(STONE_MART_ID)
        .context("missing Celadon stone counter")?;
    for id in EVOLUTION_STONES.iter().copied().chain([LINK_CABLE_ITEM_ID]) {
        ensure!(
            pack.data.items.contains_key(id),
            "missing stone seller item {id}"
        );
        if !stock.iter().any(|item| item == id) {
            stock.push(id.into());
        }
    }
    let mut added = 0;
    for entries in pack.data.evolutions.0.values_mut() {
        let alternatives = entries
            .iter()
            .filter(|entry| entry.method == METHOD_TRADE)
            .map(|trade| {
                let mut entry = EvolutionEntry::item(&trade.species, LINK_CABLE_ITEM_ID);
                entry.held_item = trade.held_item.clone().filter(|id| id != TRADE_ANY_ITEM);
                entry
            })
            .collect::<Vec<_>>();
        added += alternatives.len();
        entries.extend(alternatives);
    }
    ensure!(added > 0, "pack contains no trade evolutions");
    pack.report
        .manifests
        .push(SOLO_EVOLUTION_MANIFEST_ID.into());
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    verify_solo_evolutions(&pack)?;
    Ok(pack)
}

fn verify_solo_evolutions(pack: &CompiledGamePack) -> Result<()> {
    let cable = pack
        .data
        .items
        .get(LINK_CABLE_ITEM_ID)
        .context("missing Link Cable")?;
    ensure!(
        cable.field_usable && cable.consumable && !cable.battle_usable && cable.price > 0,
        "Link Cable must be a purchasable, consumable field item"
    );
    let stock = pack.data.marts.inventory_ids(STONE_MART_ID)?;
    for id in EVOLUTION_STONES.iter().copied().chain([LINK_CABLE_ITEM_ID]) {
        ensure!(
            stock.iter().any(|item| item == id),
            "stone seller does not stock {id}"
        );
        ensure!(
            pack.data.items.get(id).is_some_and(|item| item.price > 0),
            "stone seller item {id} cannot use the production purchase selector"
        );
    }
    for (source, entries) in &pack.data.evolutions.0 {
        for trade in entries.iter().filter(|entry| entry.method == METHOD_TRADE) {
            let mut alternative = EvolutionEntry::item(&trade.species, LINK_CABLE_ITEM_ID);
            alternative.held_item = trade.held_item.clone().filter(|id| id != TRADE_ANY_ITEM);
            ensure!(
                entries.contains(&alternative),
                "missing solo evolution for {source}"
            );
        }
    }
    Ok(())
}

/// Default content policy used by native, browser, TUI and hosted builds.
/// Reusing an enhanced pack is harmless and preserves its identity.
pub fn build_all_pokemon_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let encounters = if base
        .report
        .manifests
        .iter()
        .any(|id| id == ALL_251_MANIFEST_ID)
    {
        verify_all_251_coverage(base)?;
        base.clone()
    } else {
        build_all_251_catchable_modpack(base)?
    };
    if encounters
        .report
        .manifests
        .iter()
        .any(|id| id == SOLO_EVOLUTION_MANIFEST_ID)
    {
        verify_compiled_game_pack_for_runtime(&encounters)?;
        verify_solo_evolutions(&encounters)?;
        Ok(encounters)
    } else {
        build_solo_evolution_modpack(&encounters)
    }
}
