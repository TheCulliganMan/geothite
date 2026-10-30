//! Outdoor landing rows above Minnesota's existing imported stone stairs.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-outdoor-stair-landings-v1";
const RECIPES: [(&str, usize, usize, usize); 8] = [
    ("mn_boundary_safari", 0x84, 0x12, 0x72),
    ("mn_iron_range", 0x85, 0x12, 0x72),
    ("mn_iron_range", 0x86, 0x30, 0x72),
    ("mn_iron_range", 0x87, 0x36, 0x69),
    ("mn_r10_rockstairs", 0x80, 0x12, 0x72),
    ("mn_north_shore", 0x81, 0x12, 0x72),
    ("mn_north_shore", 0x82, 0x30, 0x72),
    ("mn_north_shore", 0x83, 0x36, 0x69),
];

/// Replaces cave-floor landing rows above existing Minnesota exterior stairs.
///
/// Reuses the destination's outdoor front72 or right-wall69 upper glyph rows.
/// Native lower steps, their palettes, collision, maps, events and audio remain
/// unchanged. Duluth's separately authored terrace revision supplies its own
/// matching landing rows. This operation writes no files and borrows the parent.
///
/// # Errors
/// Rejects invalid parents, repeated installation, missing source art, changed
/// stair collisions or lower glyphs, tilesets shared outside Minnesota outdoors,
/// and invalid resulting content.
pub fn revise_minnesota_outdoor_stair_landings(
    base: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    crate::validate_compact_superior_geography(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "outdoor stair landings already installed"
    );
    let duluth = base.data.maps.get("MnDuluth").context("Duluth exterior")?;
    ensure!(
        duluth
            .blocks
            .iter()
            .filter(|&&block| matches!(block, 0x8d..=0x90))
            .count()
            == 11
            && !duluth
                .blocks
                .iter()
                .any(|block| [0x80, 0x81, 0x8c].contains(block)),
        "install Duluth's corrected terrace flights before the regional landing pass"
    );
    let duluth_art = base
        .runtime_files
        .get("data/tilesets/mn_duluth_expanded_metatiles.bin")
        .context("Duluth terrace art")?;
    for (stair, landing) in [(0x8d, 0x72), (0x8e, 0x72), (0x8f, 0x69)] {
        ensure!(
            duluth_art.get(stair * 16..stair * 16 + 8)
                == duluth_art.get(landing * 16..landing * 16 + 8)
                && duluth_art.len() >= (stair + 1) * 16,
            "Duluth still carries cave-floor landing rows"
        );
    }
    ensure!(
        duluth_art.get(0x90 * 16..0x90 * 16 + 8) == duluth_art.get(0x8d * 16 + 8..0x8e * 16)
            && duluth_art.get(0x90 * 16 + 8..0x91 * 16) == duluth_art.get(0x70 * 16 + 8..0x71 * 16),
        "Duluth north-facing plateau stair rows differ"
    );
    let north = base
        .data
        .tilesets
        .get("mn_north_shore")
        .context("North Shore art")?;
    let authored = if north.collision.contains_key("83") {
        base.clone()
    } else {
        crate::north_shore_art::append_native_cave_stairs(
            base,
            "mn_north_shore",
            0x83,
            &[0x36],
            "minnesota-north-shore-corner-stairs-v1",
        )?
    };
    let cave = authored
        .data
        .tilesets
        .get("cave")
        .context("native cave art")?;
    let cave_blocks = authored
        .runtime_files
        .get("data/tilesets/cave_metatiles.bin")
        .context("native cave blocks")?;
    let mut result = authored.clone();
    for (tileset, stair, source, landing) in RECIPES {
        ensure!(
            base.data.maps.iter().all(|(name, map)| {
                map.attributes.tileset_name != tileset
                    || (name.starts_with("Mn")
                        && matches!(
                            map.attributes.environment.as_deref(),
                            Some("TOWN" | "ROUTE")
                        ))
            }),
            "outdoor stair tileset is shared outside Minnesota exteriors: {tileset}"
        );
        let definition = authored.data.tilesets.get(tileset).context("outdoor art")?;
        ensure!(
            definition.collision.get(&format!("{stair:02x}"))
                == cave.collision.get(&format!("{source:02x}")),
            "outdoor stair collision differs: {tileset}/{stair:02x}"
        );
        let path = format!("data/tilesets/{tileset}_metatiles.bin");
        let original = authored
            .runtime_files
            .get(&path)
            .context("outdoor blocks")?;
        let lower = original
            .get(stair * 16 + 8..stair * 16 + 16)
            .context("outdoor lower stair rows")?;
        let native = cave_blocks
            .get(source * 16 + 8..source * 16 + 16)
            .context("native lower stair rows")?;
        for (&actual, &expected) in lower.iter().zip(native) {
            ensure!(
                glyph_bytes(&authored, tileset, actual)?
                    == glyph_bytes(&authored, "cave", expected)?,
                "outdoor stair lower glyph differs: {tileset}/{stair:02x}"
            );
        }
        let upper = original
            .get(landing * 16..landing * 16 + 8)
            .context("existing outdoor landing rows")?;
        result
            .runtime_files
            .get_mut(&path)
            .and_then(|blocks| blocks.get_mut(stair * 16..stair * 16 + 8))
            .context("outdoor stair upper rows")?
            .copy_from_slice(upper);
    }
    ensure!(
        result.data == authored.data,
        "outdoor landing patch changed gameplay"
    );
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn glyph_bytes<'a>(pack: &'a CompiledGamePack, tileset: &str, glyph: u8) -> Result<&'a [u8]> {
    let graphics = pack
        .runtime_files
        .get(&format!("gfx/tilesets/{tileset}.2bpp"))
        .context("native glyph graphics")?;
    let palette = pack
        .data
        .tilesets
        .get(tileset)
        .and_then(|definition| definition.palette_map.get(usize::from(glyph)))
        .context("native glyph palette")?;
    let slot = if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 0x7f) + graphics.len() / 32
    };
    graphics
        .get(slot * 16..slot * 16 + 16)
        .context("native glyph address")
}
