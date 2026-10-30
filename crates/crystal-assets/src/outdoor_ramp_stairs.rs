//! Minnesota-only native steps at the authored outdoor ramp crossings.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

#[derive(Deserialize)]
struct Plan {
    modules: BTreeMap<String, String>,
    aliases: Vec<Alias>,
    placements: Vec<Placement>,
    retained_north_ramps: Vec<NorthRamp>,
}

#[derive(Deserialize)]
struct Alias {
    source: String,
    alias: String,
    native_metatile_count: usize,
}

#[derive(Deserialize)]
struct Placement {
    map: String,
    tileset: String,
    metatile: [usize; 2],
    before: String,
}

#[derive(Deserialize)]
struct NorthRamp {
    map: String,
    tileset: String,
    metatile: [usize; 2],
}

#[derive(Deserialize)]
struct NorthStairPlan {
    modules: BTreeMap<String, String>,
    placements: Vec<NorthStairPlacement>,
}

#[derive(Deserialize)]
struct NorthStairPlacement {
    map: String,
    tileset: String,
    metatile: [usize; 2],
    inside_landing: String,
}

/// Gives the four north-facing Headwaters ramps visible native gray treads.
///
/// Reuses the installed private south stair's gray glyphs in the upper half and
/// native74's paved inside landing or native70's rock landing in the lower half.
/// Appends private87/88 without
/// allocating glyphs or changing any existing pixels, palettes or animations.
/// Retains native74's exact collision. R07 adopts the existing Minnesota-only
/// alias; shared Headwaters/Johto art and field-move rules remain unchanged.
/// Borrows the parent and writes no files.
///
/// # Errors
/// Rejects stale map guards, different ramp positions/tilesets, missing installed
/// stair art, changed source collision, reused output IDs, shared block payloads
/// and invalid resulting packs.
pub fn revise_minnesota_outdoor_north_stairs(
    base: &CompiledGamePack,
    plan_json: &str,
) -> Result<CompiledGamePack> {
    const ALIAS: &str = "mn_outdoor_headwaters_stairs";
    const MANIFEST: &str = "minnesota-outdoor-north-stairs-v1";
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "north stairs already installed"
    );
    let plan: NorthStairPlan = serde_json::from_str(plan_json).context("north stair plan")?;
    let expected: BTreeSet<_> = [
        ("MnR05C00R00", [12, 7]),
        ("MnR05C00R00", [18, 13]),
        ("MnR05C00R01", [14, 2]),
        ("MnR07C00R00", [8, 21]),
    ]
    .into();
    let cells: BTreeSet<_> = plan
        .placements
        .iter()
        .map(|p| (p.map.as_str(), p.metatile))
        .collect();
    ensure!(
        plan.placements.len() == 4 && cells == expected,
        "exact four native northern ramps required"
    );
    let names: BTreeSet<_> = plan.placements.iter().map(|p| &p.map).collect();
    ensure!(
        names.iter().copied().eq(plan.modules.keys()),
        "north stair map guards differ"
    );
    for (name, guard) in &plan.modules {
        let map = base.data.maps.get(name).context("north stair exterior")?;
        ensure!(
            format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&serde_json::to_value(map)?)?)
            ) == *guard,
            "north stair map preimage differs: {name}"
        );
    }
    let path = format!("data/tilesets/{ALIAS}_metatiles.bin");
    let raw = base
        .runtime_files
        .get(&path)
        .context("installed Headwaters stairs")?;
    let definition = base
        .data
        .tilesets
        .get(ALIAS)
        .context("private Headwaters stair collision")?;
    let native = base
        .data
        .tilesets
        .get("mn_headwaters")
        .context("shared Headwaters collision")?;
    let kinds = ["WALL", "FLOOR", "FLOOR", "FLOOR"]
        .map(String::from)
        .to_vec();
    ensure!(
        raw.len() == 0x87 * 16
            && definition.collision.get("74") == Some(&kinds)
            && !definition.collision.contains_key("87")
            && !definition.collision.contains_key("88")
            && native
                .collision
                .iter()
                .all(|(id, collision)| definition.collision.get(id) == Some(collision)),
        "north stair source art extent or native collision differs"
    );
    let mut joined = Vec::with_capacity(32);
    for landing in [0x74, 0x70] {
        joined.extend_from_slice(&raw[0x86 * 16 + 8..0x87 * 16]);
        joined.extend_from_slice(&raw[landing * 16 + 8..(landing + 1) * 16]);
    }
    let mut result = base.clone();
    result
        .runtime_files
        .get_mut(&path)
        .context("private stair metatiles")?
        .extend(joined);
    let definition = result
        .data
        .tilesets
        .get_mut(ALIAS)
        .context("private stair definition")?;
    definition.collision.insert("87".into(), kinds.clone());
    definition.collision.insert("88".into(), kinds);
    result.runtime_files.insert(
        format!("data/tilesets/{ALIAS}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    for placement in &plan.placements {
        let map = result
            .data
            .maps
            .get_mut(&placement.map)
            .context("north stair map")?;
        let [x, y] = placement.metatile;
        let width = usize::from(map.attributes.width);
        let (block, inside) = match placement.inside_landing.as_str() {
            "paved" => (0x87, 0x01),
            "rock" => (0x88, 0x71),
            _ => anyhow::bail!("north stair requires an authored paved or rock landing"),
        };
        ensure!(
            map.attributes.tileset_name == placement.tileset
                && matches!(placement.tileset.as_str(), "mn_headwaters" | ALIAS)
                && x > 0
                && x + 1 < width
                && y + 1 < usize::from(map.attributes.height)
                && map.blocks[y * width + x] == 0x74
                && map.blocks[(y + 1) * width + x] == inside
                && (map.blocks[y * width + x - 1] == 0x70 || map.blocks[y * width + x + 1] == 0x70),
            "north stair must retain its exact native ramp and back-rim join"
        );
        map.blocks[y * width + x] = block;
        map.attributes.tileset_name = ALIAS.into();
    }
    for name in names {
        let map = &result.data.maps[name];
        let label = map
            .attributes
            .blocks_label
            .as_ref()
            .context("north stair terrain label")?;
        ensure!(
            base.data
                .maps
                .values()
                .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
                .count()
                == 1,
            "north stair payload is shared"
        );
        let bytes = map
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        result
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    }
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::validate_compact_superior_geography(&result)?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

/// Replaces explicitly authored outdoor ramps with native gray stone steps.
///
/// Creates Minnesota-only art aliases, retaining every original metatile and
/// foreground palette. Each new crossing keeps outdoor72's upper landing and
/// front face, and uses cave12's lower stair rows. Its lower left quadrant is
/// Wall and the right lane is Floor. Complete map preimages bind exact placements.
/// Existing shared Johto/Kanto art is unchanged. Writes no files.
///
/// # Errors
/// Rejects invalid parents, stale modules, non-Minnesota exterior targets,
/// unexpected native ramp collision, occupied aliases, missing native art,
/// insufficient unused glyph storage, invalid placements or final verification.
pub fn revise_minnesota_outdoor_ramp_stairs(
    base: &CompiledGamePack,
    plan_json: &str,
) -> Result<CompiledGamePack> {
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan: Plan = serde_json::from_str(plan_json).context("outdoor stair placements")?;
    let names: BTreeSet<_> = plan.placements.iter().map(|p| &p.map).collect();
    ensure!(
        !names.is_empty() && names.iter().copied().eq(plan.modules.keys()),
        "stair map guards differ"
    );
    for (name, expected) in &plan.modules {
        let map = base.data.maps.get(name).context("stair exterior")?;
        ensure!(
            name.starts_with("Mn")
                && matches!(
                    map.attributes.environment.as_deref(),
                    Some("TOWN" | "ROUTE")
                ),
            "outdoor stairs require Minnesota exteriors"
        );
        ensure!(
            format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&serde_json::to_value(map)?)?)
            ) == *expected,
            "outdoor stair map preimage differs: {name}"
        );
    }
    let mut result = base.clone();
    let mut retained = BTreeSet::new();
    for ramp in &plan.retained_north_ramps {
        let map = base
            .data
            .maps
            .get(&ramp.map)
            .context("retained north ramp")?;
        let [x, y] = ramp.metatile;
        let width = usize::from(map.attributes.width);
        ensure!(
            ramp.map.starts_with("Mn")
                && map.attributes.tileset_name == ramp.tileset
                && x < width
                && y < usize::from(map.attributes.height)
                && map.blocks.get(y * width + x) == Some(&0x74)
                && ((x > 0 && map.blocks[y * width + x - 1] == 0x70)
                    || (x + 1 < width && map.blocks[y * width + x + 1] == 0x70))
                && retained.insert((&ramp.map, ramp.metatile)),
            "retained crossing is not a unique native north-facing ramp"
        );
    }
    let mut aliases = BTreeMap::new();
    for alias in &plan.aliases {
        ensure!(
            alias.alias.starts_with("mn_outdoor_")
                && aliases.insert(alias.source.as_str(), alias).is_none(),
            "stair aliases must be unique and Minnesota-only"
        );
        let path = format!("data/tilesets/{}_metatiles.bin", alias.source);
        let raw = base
            .runtime_files
            .get(&path)
            .context("outdoor source blocks")?;
        ensure!(
            raw.len() == alias.native_metatile_count * 16 && alias.native_metatile_count < 256,
            "outdoor stair source extent differs"
        );
        let collision = base
            .data
            .tilesets
            .get(&alias.source)
            .and_then(|t| t.collision.get("74"))
            .context("native outdoor ramp")?;
        ensure!(
            collision
                .iter()
                .map(String::as_str)
                .eq(["WALL", "FLOOR", "FLOOR", "FLOOR"]),
            "native ramp quadrants differ"
        );
        let art = serde_json::json!({
            "tileset": alias.source, "source_pixels_modified": false,
            "derived_blocks": [{"id": format!("{:02x}", alias.native_metatile_count), "source_tileset": "cave", "source_block": "12", "collision": base.data.tilesets.get("cave").and_then(|tileset| tileset.collision.get("12")).context("native cave12 collision")?}]
        });
        result = crate::authored_city_expansion_art::install(
            &result,
            &serde_json::to_string(&art)?,
            &alias.alias,
            &alias.source,
            &alias.alias.replace('_', "-"),
        )?;
        let target = format!("data/tilesets/{}_metatiles.bin", alias.alias);
        let blocks = result
            .runtime_files
            .get_mut(&target)
            .context("new outdoor stair art")?;
        blocks
            .get_mut(alias.native_metatile_count * 16..alias.native_metatile_count * 16 + 8)
            .context("new outdoor stair upper rows")?
            .copy_from_slice(
                raw.get(0x72 * 16..0x72 * 16 + 8)
                    .context("outdoor upper landing")?,
            );
        let definition = result
            .data
            .tilesets
            .get_mut(&alias.alias)
            .context("new stair collision")?;
        definition.collision.insert(
            format!("{:02x}", alias.native_metatile_count),
            ["FLOOR", "FLOOR", "WALL", "FLOOR"].map(String::from).into(),
        );
        result.runtime_files.insert(
            format!("data/tilesets/{}.json", alias.alias),
            serde_json::to_vec(&definition.collision)?,
        );
        result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            result.format_version,
            &result.data,
            &result.audio_manifest,
            &result.runtime_files,
            &result.report,
        )?;
    }
    let mut cells = BTreeSet::new();
    for placement in &plan.placements {
        let alias = aliases
            .get(placement.tileset.as_str())
            .context("stair alias for map")?;
        let map = result
            .data
            .maps
            .get_mut(&placement.map)
            .context("stair exterior")?;
        let original = base
            .data
            .maps
            .get(&placement.map)
            .context("stair map preimage")?;
        ensure!(
            placement.before == "74"
                && original.attributes.tileset_name == placement.tileset
                && cells.insert((&placement.map, placement.metatile)),
            "stair placement source differs"
        );
        let [x, y] = placement.metatile;
        ensure!(
            x < usize::from(map.attributes.width) && y < usize::from(map.attributes.height),
            "stair outside map"
        );
        let cell = map
            .blocks
            .get_mut(y * usize::from(map.attributes.width) + x)
            .context("outdoor ramp cell")?;
        ensure!(*cell == 0x74, "outdoor ramp cell differs");
        *cell = u16::try_from(alias.native_metatile_count)?;
        map.attributes.tileset_name = alias.alias.clone();
    }
    for name in names {
        let map = result
            .data
            .maps
            .get(name)
            .context("revised outdoor stairs")?;
        for (index, block) in map.blocks.iter().enumerate() {
            let width = usize::from(map.attributes.width);
            ensure!(
                *block != 0x74 || retained.contains(&(name, [index % width, index / width])),
                "a south-facing ramp is missing from the stair plan"
            );
        }
        let label = map
            .attributes
            .blocks_label
            .as_ref()
            .context("stair terrain label")?;
        ensure!(
            base.data
                .maps
                .values()
                .filter(|m| m.attributes.blocks_label.as_ref() == Some(label))
                .count()
                == 1,
            "stair terrain payload is shared"
        );
        let bytes = map
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        result
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    }
    result
        .report
        .manifests
        .push("minnesota-outdoor-ramp-stairs-v1".into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::validate_compact_superior_geography(&result)?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}
