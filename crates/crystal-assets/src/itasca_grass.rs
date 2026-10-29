//! Encounter meadow and a native dry glade return at the headwaters park.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_layout::Layout;
use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnLakeItasca";
const MANIFEST: &str = "minnesota-itasca-grass-v1";
const PATCHES: [(usize, usize, u16, u16); 22] = [
    (16, 11, 0x02, 0x03),
    (16, 12, 0x01, 0x03),
    (16, 13, 0x01, 0x03),
    (16, 14, 0x01, 0x03),
    (16, 15, 0x01, 0x03),
    (16, 16, 0x01, 0x03),
    (16, 17, 0x02, 0x03),
    (15, 12, 0x01, 0x03),
    (15, 13, 0x02, 0x03),
    (15, 14, 0x02, 0x03),
    (14, 12, 0x01, 0x03),
    (14, 13, 0x02, 0x03),
    (5, 15, 0x01, 0x03),
    (7, 15, 0x01, 0x03),
    (8, 15, 0x02, 0x03),
    (9, 15, 0x02, 0x03),
    (10, 15, 0x02, 0x03),
    (5, 16, 0x01, 0x03),
    (5, 17, 0x01, 0x03),
    (5, 14, 0x01, 0x05),
    (6, 14, 0x02, 0x56),
    (7, 14, 0x01, 0x05),
];

/// Adds encounter meadows and a tree-framed native southbound glade return.
///
/// Requires exact installed before/after source cells for the 24×24 park.
/// Nineteen floor blocks become grass, two frame blocks become native pines,
/// and one block becomes a south-facing ledge. Original event/actor anchors,
/// plaque, bridge, portals and every remaining walking cell stay accessible.
/// Only this map's blocks/mirror, its new R14-derived wild table and provenance change.
/// No output is written, and native gameplay/visual acceptance remains required.
///
/// # Errors
/// Rejects stale source/payloads, another map/extent/alias, shared labels,
/// changed art, undeclared cells, affected event/actor anchors, dynamic block
/// programs, incompatible native collision, unsafe movement or invalid packs.
/// The borrowed parent remains unchanged on success and failure.
pub fn revise_itasca_grass(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|name| name == MANIFEST),
        "Itasca grass already installed"
    );
    let before_art: Value = serde_json::from_str(before_json)?;
    let after_art: Value = serde_json::from_str(after_json)?;
    for key in ["tileset", "derived_blocks", "source_pixels_modified"] {
        ensure!(
            before_art.get(key) == after_art.get(key),
            "Itasca cannot replace native art recipes"
        );
    }
    let before = Layout::parse(before_json)?;
    let after = Layout::parse(after_json)?;
    for source in [&before, &after] {
        ensure!(
            source.id == MAP
                && source.dimensions_metatiles == [24, 24]
                && source.compiled_tileset == "mn_headwaters",
            "current Itasca source required"
        );
    }
    let old = base.data.maps.get(MAP).context("installed Itasca map")?;
    ensure!(
        old.attributes.tileset_name == "mn_headwaters"
            && [old.attributes.width, old.attributes.height] == [24, 24]
            && old.attributes.border_block == 5,
        "current Itasca extent and art required"
    );
    ensure!(
        old.script_block_changes.is_empty(),
        "unreviewed Itasca block programs"
    );
    ensure!(
        before.compile()? == old.blocks,
        "Itasca source preimage differs"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("native Itasca block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(name, map)| name == MAP || map.attributes.blocks_label.as_ref() != Some(label)),
        "shared Itasca block label"
    );
    ensure!(
        crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(label)
                .context("native encoded blocks")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(old.blocks.iter().copied()),
        "Itasca native block mirror differs"
    );
    let mut protected = BTreeSet::new();
    protected.extend(
        old.objects
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
    );
    protected.extend(
        old.events
            .warps
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
    );
    protected.extend(
        old.events
            .bg_events
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
    );
    protected.extend(
        old.events
            .coord_events
            .iter()
            .map(|v| (usize::from(v.x / 2), usize::from(v.y / 2))),
    );
    let mut module = old.clone();
    for (x, y, expected, replacement) in PATCHES {
        ensure!(
            !protected.contains(&(x, y)) && module.blocks[y * 24 + x] == expected,
            "Itasca patch preimage/anchor differs at {x},{y}"
        );
        module.blocks[y * 24 + x] = replacement;
    }
    ensure!(
        after.compile()? == module.blocks,
        "only the declared reviewed Itasca cells may change"
    );
    validate_movement(base, old, &module)?;
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Itasca native block range")?;
    ensure!(
        !base.data.wild_encounters.contains_key(MAP),
        "Itasca already has a wild table"
    );
    let mut encounters = base
        .data
        .wild_encounters
        .get("MnR14C00R00")
        .context("current adjacent R14 encounter table")?
        .clone();
    ensure!(
        encounters.grass.is_some()
            && encounters.water.is_some()
            && encounters.grass_rates.is_some()
            && encounters.water_rate.is_some(),
        "complete adjacent regional encounter table required"
    );
    encounters.map_name = MAP.into();
    encounters.zones.clear();
    encounters.swarm_overrides.clear();
    let mut output = base.clone();
    output.data.wild_encounters.insert(MAP.into(), encounters);
    output.data.maps.insert(MAP.into(), module);
    output
        .data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    output.report.manifests.push(MANIFEST.into());
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    crate::validate_compact_superior_geography(&output)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn validate_movement(pack: &CompiledGamePack, old: &MapModule, revised: &MapModule) -> Result<()> {
    let definition = pack
        .data
        .tilesets
        .get("mn_headwaters")
        .context("native Johto collision")?;
    for (block, expected) in [(1, ["FLOOR"; 4]), (3, ["TALL_GRASS"; 4]), (2, ["FLOOR"; 4])] {
        let actual = definition
            .collision
            .get(&format!("{block:02x}"))
            .context("Itasca native block")?;
        ensure!(
            actual.iter().map(String::as_str).eq(expected),
            "Itasca native collision recipe changed"
        );
    }
    let occupied = old
        .objects
        .iter()
        .map(|v| [v.x, v.y])
        .collect::<BTreeSet<_>>();
    for start in [[42, 24], [42, 25], [4, 28], [4, 29]] {
        let before = walking_map_distances(pack, old, start, &occupied)?;
        let after = walking_map_distances(pack, revised, start, &occupied)?;
        ensure!(
            before.keys().all(|at| after.contains_key(at)
                || matches!(*at, [10 | 11 | 14 | 15, 28 | 29] | [12 | 13, 29])),
            "Itasca loses existing walking access"
        );
    }
    let contracts = serde_json::json!({
        "marker_contract":{"tile":[19,21],"approach":[19,22],"milestone":"MS06","requires":["BADGE_7"],"sets":["EVENT_MN_HEADWATERS_READING"]},
        "bridge_contract":{"walk_rows_core":[25,26],"walk_x_core":[20,25],"top_rail_core_y":24,"bottom_rail_core_y":27,"west_target_core":[19,22],"east_arrival_core":[42,24]}
    });
    crate::itasca_geometry::validate(pack, revised, &contracts)?;
    let down = walking_map_distances(pack, revised, [13, 27], &occupied)?;
    ensure!(
        down.get(&[13, 30]) == Some(&3),
        "native glade descent changed"
    );
    let mut no_lip = occupied;
    no_lip.extend([[12, 28], [13, 28]]);
    let up = walking_map_distances(pack, revised, [13, 30], &no_lip)?;
    ensure!(
        up.get(&[13, 27]) == Some(&9),
        "ordinary glade return changed"
    );
    Ok(())
}
