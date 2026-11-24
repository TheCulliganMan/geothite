//! Give Minnesota cave boundaries quiet native art with solid collision.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const GLYPHS_PER_METATILE: usize = 16;
const STIPPLE: usize = 0x09;
const ROCK_STIPPLE_SHA256: &str =
    "2090570ea02671ffcdc2d73efabbb49d341b40acebba5ba1e521a52c2033c5a1";
const ICE_STIPPLE_SHA256: &str = "6923fb295352324cffdf24d37e3a3542d97431e312cc665b67c6ee7ea4c842e7";
const CAVE_FIELDS: [(&str, u8); 7] = [
    ("cave", 0x50),
    ("ice_path", 0x40),
    ("mn_r05_rock_cut", 0x62),
    ("mn_iron_haulage_rock", 0x51),
    ("mn_iron_sump_rock", 0x4c),
    ("mn_iron_vent_rock", 0x4c),
    ("mn_iron_core_rock", 0x48),
];

/// Appends a solid alias of each tileset’s quiet native stipple and returns its
/// block ID by tileset. Existing glyph sheets, palettes, recipes and collision
/// entries are preserved. This lets cave rooms use quiet space beyond a visible
/// cliff face while retaining the original walkable stipple elsewhere.
///
/// # Errors
/// Rejects a missing bank, drifted donor, or a block ID that is already occupied.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<BTreeMap<String, u8>> {
    let mut aliases = BTreeMap::new();
    for (tileset, id) in CAVE_FIELDS {
        let layout_path = format!("data/tilesets/{tileset}_metatiles.bin");
        let layout = pack
            .runtime_files
            .get_mut(&layout_path)
            .with_context(|| format!("native metatile bank {tileset}"))?;
        ensure!(
            layout.len() == usize::from(id) * GLYPHS_PER_METATILE,
            "expected an available block {id:02x} at the end of {tileset}"
        );
        let start = STIPPLE * GLYPHS_PER_METATILE;
        let glyphs: [u8; GLYPHS_PER_METATILE] = layout
            .get(start..start + GLYPHS_PER_METATILE)
            .context("native stipple recipe")?
            .try_into()
            .context("complete native cave stipple")?;
        let expected = if tileset == "ice_path" {
            ICE_STIPPLE_SHA256
        } else {
            ROCK_STIPPLE_SHA256
        };
        ensure!(
            format!("{:x}", Sha256::digest(glyphs)) == expected,
            "native stipple art changed in {tileset}"
        );
        let definition = pack
            .data
            .tilesets
            .get_mut(tileset)
            .with_context(|| format!("native collision bank {tileset}"))?;
        let donor = definition
            .collision
            .get("09")
            .with_context(|| format!("native stipple permission in {tileset}"))?;
        ensure!(
            donor == &["FLOOR"; 4],
            "native stipple changed in {tileset}"
        );
        let collision_key = format!("{id:02x}");
        ensure!(
            !definition.collision.contains_key(&collision_key),
            "occupied cave field permission {id:02x} in {tileset}"
        );
        definition
            .collision
            .insert(collision_key, vec![String::from("WALL"); 4]);
        layout.extend_from_slice(&glyphs);
        pack.runtime_files.insert(
            format!("data/tilesets/{tileset}.json"),
            serde_json::to_vec(&definition.collision)?,
        );
        aliases.insert(tileset.to_owned(), id);
    }
    Ok(aliases)
}
