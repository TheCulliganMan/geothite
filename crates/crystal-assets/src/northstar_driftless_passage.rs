//! Add an eastward, lit passage to the existing Driftless hollow.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::authored_town_geometry;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring;

const ROUTE: &str = "MnR01C00R00";
const CAVE: &str = "MnR01Cave1F";
const CLEFT: &str = "MnR20Cave1F";
const MANIFEST: &str = "northstar-driftless-connecting-passage-v1";

#[derive(Deserialize)]
struct Passage {
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
    exits: Vec<[u16; 2]>,
}

/// Extend the hollow east and align the Voyageurs cleft's eastern exit.
///
/// Existing hollow coordinates and the gallery stair remain unchanged. The
/// new mouth is an optional, lit alternative to the outdoor road.
///
/// # Errors
/// Rejects incompatible input geometry, occupied warp locations, disconnected
/// passages, or an invalid resulting pack.
pub fn add_northstar_driftless_passage(
    base: &CompiledGamePack,
    source_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "passage already installed"
    );
    let source: Passage = serde_json::from_str(source_json)?;
    ensure!(
        source.dimensions_metatiles == [20, 8] && source.exits == [[9, 11], [31, 13]],
        "unexpected passage contract"
    );
    let mut route = base.data.maps.get(ROUTE).context("Route 1")?.clone();
    let mut cave = base
        .data
        .maps
        .get(CAVE)
        .context("Driftless hollow")?
        .clone();
    ensure!(
        route.attributes.width == 32
            && route.attributes.height == 20
            && matches!(route.blocks[9 * 32 + 20], 0x72 | 0x80)
            && route.events.warps.len() == 6
            && cave.attributes.width == 8
            && cave.attributes.height == 7
            && cave.events.warps.len() == 3
            && cave.attributes.time_of_day.as_deref() == Some("nite")
            && cave.events.warps[0].x == 9
            && cave.events.warps[0].y == 11
            && cave.events.warps[0].target_map_constant == ROUTE
            && cave.events.warps[0].target_warp_id == 2
            && cave.events.warps[1].x == 11
            && cave.events.warps[1].y == 9
            && cave.events.warps[1].target_map_constant == "MN_DRIFTLESS_LAMP_GALLERY"
            && route.events.warps[0].x == 18
            && route.events.warps[0].y == 17
            && route.events.warps[0].target_map_constant == CAVE,
        "existing route or hollow preimage differs: route {}x{}, block {:?}, warps {:?}; cave {}x{}, time {:?}, warps {:?}",
        route.attributes.width,
        route.attributes.height,
        route.blocks.get(9 * 32 + 20),
        route.events.warps,
        cave.attributes.width,
        cave.attributes.height,
        cave.attributes.time_of_day,
        cave.events.warps
    );
    ensure!(
        !route
            .objects
            .iter()
            .any(|o| [o.x, o.y] == [40, 19] || [o.x, o.y] == [40, 20]),
        "new mouth is occupied"
    );
    ensure!(
        authored_town_geometry::kind(base, &route, [40, 20])? == "TALL_GRASS"
            || authored_town_geometry::kind(base, &route, [40, 20])? == "FLOOR",
        "new exterior arrival is blocked"
    );
    let old_blocks = cave.blocks.clone();
    ensure!(
        [
            old_blocks[30],
            old_blocks[31],
            old_blocks[38],
            old_blocks[39]
        ] == [0x22, 0x2e, 0x0a, 0x2e],
        "original hollow opening differs"
    );
    authoring::blocks(&mut cave, 20, 8, &source.rows.join("\n"))?;
    for y in 0..7 {
        for x in 0..8 {
            if !matches!((x, y), (6 | 7, 3 | 4)) {
                ensure!(
                    cave.blocks[y * 20 + x] == old_blocks[y * 8 + x],
                    "original hollow geometry changed at {x},{y}"
                );
            }
        }
    }
    ensure!(
        authored_town_geometry::kind(base, &cave, [31, 13])? == "LADDER",
        "east exit must be a ladder"
    );
    let reachable = authored_town_geometry::reachable(base, &cave, [9, 10], false)?;
    ensure!(
        reachable.contains(&[31, 13])
            && reachable.contains(&[11, 9])
            && reachable.contains(&[5, 6]),
        "passage, gallery and guide must connect"
    );
    for y in 0..16 {
        for x in 0..40 {
            let kind = authored_town_geometry::kind(base, &cave, [x, y])?;
            if x == 0 || y == 0 || x == 39 || y == 15 {
                ensure!(kind == "WALL", "passage border is open at {x},{y}");
            }
            if kind == "LADDER" {
                ensure!(
                    [[9, 11], [11, 9], [31, 13]].contains(&[x, y]),
                    "unpaired ladder at {x},{y}"
                );
            }
            if kind == "FLOOR" && !cave.objects.iter().any(|o| [o.x, o.y] == [x, y]) {
                ensure!(reachable.contains(&[x, y]), "stranded floor at {x},{y}");
            }
        }
    }
    // The eastern mouth occupies the old uphill stair. Move the same native
    // stair one metatile west so the fossil terrace still has a two-way climb.
    ensure!(
        route.blocks[9 * 32 + 19] == 0x72 && route.blocks[9 * 32 + 20] == 0x80,
        "expected the native stair and adjacent cliff face"
    );
    route.blocks[9 * 32 + 19] = 0x80;
    route.blocks[9 * 32 + 20] = 0x73;
    ensure!(
        authored_town_geometry::kind(base, &route, [39, 19])? == "FLOOR"
            && authored_town_geometry::kind(base, &route, [39, 18])? == "FLOOR"
            && authored_town_geometry::kind(base, &route, [39, 20])? == "TALL_GRASS",
        "relocated stair must join the road and terrace"
    );
    ensure!(
        authored_town_geometry::kind(base, &route, [40, 19])? == "CAVE",
        "new exterior mouth must use native cave collision"
    );
    authoring::warp(&mut route, (40, 19), CAVE, 4);
    authoring::warp(&mut route, (40, 20), CAVE, 4);
    authoring::warp(&mut cave, (31, 13), ROUTE, 8);
    authoring::text(
        &mut route,
        "MN_R01_SIGN_03Text",
        &["LIMESTONE PASSAGE", "WEST: ROCHESTER", "EAST: WINONA"],
    );
    authoring::text(
        &mut cave,
        "MN_R01_HOLLOW_GUIDEText",
        &[
            "The passage runs",
            "east to WINONA.",
            "That ball holds",
            "an ANTIDOTE.",
            "The lower gallery",
            "is dark. FLASH",
            "shows its fossils.",
        ],
    );
    for (map, label) in [
        (&mut route, "MN_R01_SIGN_03Text"),
        (&mut cave, "MN_R01_HOLLOW_GUIDEText"),
    ] {
        let mut bodies = crate::parse_script_text_bodies(&map.id, &map.scripts)?;
        map.script_text_bodies
            .insert(label.into(), bodies.remove(label).context("revised text")?);
        authoring::refresh_event_section(map);
    }
    let mut cleft = base
        .data
        .maps
        .get(CLEFT)
        .context("Voyageurs cleft")?
        .clone();
    ensure!(
        cleft.attributes.width == 14
            && cleft.attributes.height == 12
            && cleft.events.warps.len() == 2
            && (cleft.events.warps[0].x, cleft.events.warps[0].y) == (5, 21)
            && (cleft.events.warps[1].x, cleft.events.warps[1].y) == (23, 21)
            && cleft.blocks[9 * 14 + 11] == 0x02
            && cleft.blocks[10 * 14 + 11] == 0x07,
        "Voyageurs cleft preimage differs"
    );
    cleft.blocks[9 * 14 + 11] = 0x07;
    // Keep the former exit walkable for a save made at that position.
    cleft.blocks[10 * 14 + 11] = 0x02;
    cleft.events.warps[1].y = 19;
    ensure!(
        authored_town_geometry::kind(base, &cleft, [23, 19])? == "LADDER"
            && authored_town_geometry::kind(base, &cleft, [23, 21])? == "FLOOR"
            && authored_town_geometry::reachable(base, &cleft, [5, 20], false)?.contains(&[23, 19]),
        "aligned cleft exit is disconnected"
    );
    authoring::refresh_event_section(&mut cleft);
    let mut pack = base.clone();
    authored_town_publication::replace(&mut pack, route)?;
    authored_town_publication::replace(&mut pack, cave)?;
    authored_town_publication::replace(&mut pack, cleft)?;
    ensure!(
        pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio
            && pack.runtime_files == base.runtime_files,
        "art or audio changed"
    );
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

#[cfg(test)]
mod tests {
    use super::add_northstar_driftless_passage;
    use crate::read_verified_compiled_game_pack;

    #[test]
    #[ignore = "requires the external Northstar review pack"]
    fn driftless_passage_preserves_other_maps_and_pairs_both_mouths() -> anyhow::Result<()> {
        let path = std::env::var("CRYSTAL_DRIFTLESS_PARENT")?;
        let base = read_verified_compiled_game_pack(path)?;
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../planning/minnesota/caves/geographic-audit-v01/r01-passage-draft.json"
        ))?;
        let pack = add_northstar_driftless_passage(&base, &source)?;
        assert_eq!(pack.data.maps.len(), base.data.maps.len());
        for (name, before) in &base.data.maps {
            if name != "MnR01C00R00" && name != "MnR01Cave1F" && name != "MnR20Cave1F" {
                assert_eq!(pack.data.maps.get(name), Some(before), "changed {name}");
            }
        }
        let route = &pack.data.maps["MnR01C00R00"];
        assert_eq!(route.blocks[9 * 32 + 19], 0x80);
        let cave = &pack.data.maps["MnR01Cave1F"];
        assert_eq!((route.events.warps[6].x, route.events.warps[6].y), (40, 19));
        assert_eq!(route.events.warps[6].target_warp_id, 4);
        assert_eq!((route.events.warps[7].x, route.events.warps[7].y), (40, 20));
        assert_eq!((cave.events.warps[3].x, cave.events.warps[3].y), (31, 13));
        assert_eq!(cave.events.warps[3].target_warp_id, 8);
        let cleft = &pack.data.maps["MnR20Cave1F"];
        assert_eq!((cleft.events.warps[1].x, cleft.events.warps[1].y), (23, 19));
        assert_eq!(cleft.events.warps[1].target_warp_id, 2);
        assert_eq!(
            base.data.maps["MnDriftlessLampGallery"],
            pack.data.maps["MnDriftlessLampGallery"]
        );
        let mut invalid: serde_json::Value = serde_json::from_str(&source)?;
        invalid["exits"][1] = serde_json::json!([9, 11]);
        assert!(add_northstar_driftless_passage(&base, &invalid.to_string()).is_err());
        Ok(())
    }
}
