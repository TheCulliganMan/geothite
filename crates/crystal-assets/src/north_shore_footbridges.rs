//! Bounded native footbridge repair on the current North Shore land route.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;

const TILESET: &str = "mn_north_shore";
const MANIFEST: &str = "minnesota-north-shore-footbridges-v1";

#[derive(Deserialize)]
struct Terrain {
    dimensions_metatiles: [u16; 2],
    tileset: String,
    rows: Vec<String>,
}

impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        let [width, height] = self.dimensions_metatiles;
        ensure!(
            self.rows.len() == usize::from(height),
            "complete route rows required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(
                words.len() == usize::from(width),
                "complete route columns required"
            );
            for word in words {
                blocks.push(u16::from_str_radix(word, 16).context("native route block")?);
            }
        }
        Ok(blocks)
    }
}

/// Replaces the eight brick crossing blocks with seven visibly railed native
/// footbridges. Appends one metatile assembled from existing rail/ground glyphs;
/// every original pixel, palette and collision quadrant remains unchanged.
/// Existing bridge movement remains ordinary walkable `LADDER` collision.
///
/// # Errors
/// Rejects stale complete route terrain, different crossing positions, occupied
/// art/manifest addresses, nonnative collision or an invalid resulting pack.
pub fn rebuild_north_shore_footbridges(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "footbridges already installed"
    );
    let before: BTreeMap<String, Terrain> = serde_json::from_str(before_json)?;
    let after: BTreeMap<String, Terrain> = serde_json::from_str(after_json)?;
    let names = (0..2)
        .flat_map(|column| (0..4).map(move |row| format!("MnR11C{column:02}R{row:02}")))
        .collect::<Vec<_>>();
    ensure!(
        before.len() == 8 && after.len() == 8,
        "all eight current route chunks required"
    );
    let definition = base
        .data
        .tilesets
        .get(TILESET)
        .context("current coastal tileset")?;
    ensure!(
        definition.collision["06"]
            .iter()
            .all(|kind| kind == "LADDER")
            && definition.collision["01"]
                .iter()
                .all(|kind| kind == "FLOOR")
            && !definition.collision.contains_key("84"),
        "native crossing/floor collision or append extent differs"
    );
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let original_art = base
        .runtime_files
        .get(&path)
        .context("current coast metatiles")?;
    ensure!(
        original_art.len() == 0x84 * 16,
        "current coastal art extent differs"
    );
    let mut art = original_art.clone();
    for row in 0..4 {
        for (block, column) in [(0x44, 0), (0x01, 1), (0x01, 2), (0x44, 0)] {
            art.push(original_art[block * 16 + row * 4 + column]);
        }
    }
    let mut pack = base.clone();
    let mut expected_cells = vec![
        (20, 20),
        (20, 38),
        (23, 48),
        (23, 49),
        (20, 55),
        (17, 61),
        (20, 81),
        (23, 93),
    ];
    let mut changed_cells = Vec::new();
    for name in names {
        let original = base.data.maps.get(&name).context("current route chunk")?;
        let old = before.get(&name).context("route preimage")?;
        let new = after.get(&name).context("route candidate")?;
        let width = original.attributes.width;
        ensure!(
            old.tileset == TILESET
                && new.tileset == TILESET
                && original.attributes.tileset_name == TILESET
                && old.dimensions_metatiles == [width, original.attributes.height]
                && new.dimensions_metatiles == old.dimensions_metatiles
                && old.blocks()? == original.blocks,
            "complete route preimage differs: {name}"
        );
        let mut expected = original.blocks.clone();
        let row = name[9..11].parse::<usize>()?;
        let column = name[6..8].parse::<usize>()?;
        for (index, block) in expected.iter_mut().enumerate() {
            if *block == 0x06 {
                changed_cells.push((
                    column * 32 + index % usize::from(width),
                    row * 24 + index / usize::from(width),
                ));
                *block = 0x84;
            }
        }
        ensure!(
            new.blocks()? == expected,
            "only existing crossing cells may change: {name}"
        );
        if expected != original.blocks {
            let mut module = original.clone();
            module.blocks = expected;
            crate::authored_town_publication::replace(&mut pack, module)?;
        }
    }
    changed_cells.sort_unstable();
    expected_cells.sort_unstable();
    ensure!(
        changed_cells == expected_cells,
        "exact seven bridge footprints required"
    );
    let mut updated = definition.clone();
    updated
        .collision
        .insert("84".into(), definition.collision["06"].clone());
    pack.runtime_files.insert(path, art);
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&updated.collision)?,
    );
    pack.data.tilesets.insert(TILESET.into(), updated);
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
