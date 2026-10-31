//! Sheltered winter-town circuits with one optional original Minnesota Skier.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::MapModule;
use crate::authored_town_geometry;

const TOWN: &str = "MnInternationalFalls";
const SKIER: &str = "MN_EXTRA_FALLS_SKIER";

#[derive(Deserialize)]
struct Terrain {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    border_block: u16,
    palette_group: String,
    rows: Vec<String>,
}

impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        ensure!(
            self.schema_version == 1
                && self.id == TOWN
                && self.tileset == "mn_international_falls"
                && self.dimensions_metatiles == [16, 14]
                && self.border_block == 129
                && self.palette_group == "day"
                && self.rows.len() == 14,
            "complete native winter-town source required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let values = row.split_whitespace().collect::<Vec<_>>();
            ensure!(values.len() == 16, "complete winter-town columns required");
            for value in values {
                blocks.push(u16::from_str_radix(value, 16).context("native winter-town block")?);
            }
        }
        Ok(blocks)
    }
}

/// Replaces the broad snow court with sheltered service and library circuits,
/// preserving native snow/pines, all original actors, doors, rooms, boards and
/// the actual southern road. Adds one optional native Skier with original
/// Minnesota Lutebrisk and Fencicle, retaining inherited trainer scaling.
/// The eastern street remains a visibly closed local stop; the existing long
/// icefields/Voyageurs journey and ranger quest are unchanged. No art or audio
/// is changed and no field gate, ice slide or additional exit is introduced.
///
/// # Errors
/// Rejects stale native/source preimages, changed facades or boundary buffers,
/// misplaced original events/approaches, disconnected terrain, invalid regional
/// opponents or resulting pack data.
pub fn rebuild_international_falls_windbreak(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    contract_json: &str,
    trainers_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before: Terrain = serde_json::from_str(before_json)?;
    let after: Terrain = serde_json::from_str(after_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let old = base.data.maps.get(TOWN).context("installed winter town")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == TOWN
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(old)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&old.events.warps)?
            && contract["preserved_readings"] == serde_json::to_value(&old.events.bg_events)?
            && contract["preserved_coordinate_events"]
                == serde_json::to_value(&old.events.coord_events)?
            && old.attributes.width == 16
            && old.attributes.height == 14
            && old.attributes.tileset_name == "mn_international_falls"
            && old.blocks == before.blocks()?,
        "complete winter-town preimage differs"
    );
    ensure!(
        old.objects.len() == 4,
        "four original winter-town residents required"
    );
    let placements: Value = serde_json::from_str(trainers_json)?;
    let trainers = placements["trainers"]
        .as_array()
        .context("one optional winter Skier")?;
    ensure!(
        placements["schema_version"] == 1
            && trainers.len() == 1
            && trainers[0]["map"] == TOWN
            && trainers[0]["id"] == SKIER
            && trainers[0]["tile"] == json!([17, 12])
            && trainers[0]["approach"] == json!([17, 13])
            && trainers[0]["sprite"] == "SPRITE_COOLTRAINER_F"
            && trainers[0]["trainer_class"] == "SKIER"
            && trainers[0]["optional_battle"] == true
            && trainers[0]["party"]
                == json!([
                    {"species":"LUTEBRISK","level":39},
                    {"species":"FENCICLE","level":40}
                ]),
        "declared optional original winter Skier required"
    );
    let mut town = old.clone();
    town.blocks = after.blocks()?;
    for y in 0..14 {
        for x in 0..16 {
            if !(2..14).contains(&x) || !(2..11).contains(&y) {
                ensure!(
                    town.blocks[y * 16 + x] == old.blocks[y * 16 + x],
                    "original winter boundary and southern arrival must remain"
                );
            }
        }
    }
    for (x0, y0) in [(4, 2), (11, 3)] {
        for y in y0..y0 + 2 {
            for x in x0..x0 + 2 {
                ensure!(
                    town.blocks[y * 16 + x] == old.blocks[y * 16 + x],
                    "complete original winter service facade required"
                );
            }
        }
    }
    for (x, y) in [(3, 6), (13, 6), (9, 11)] {
        ensure!(
            town.blocks[y * 16 + x] == old.blocks[y * 16 + x],
            "original winter placard pixels required"
        );
    }
    validate_geometry(base, &town)?;
    let mut pack = base.clone();
    crate::authored_town_publication::replace(&mut pack, town)?;
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    let pack = crate::minnesota_trainers::build_trainers(
        &pack,
        trainers_json,
        "minnesota-international-falls-windbreak-v1",
        base.data.minnesota_trainer_scaling,
    )?;
    let town = &pack.data.maps[TOWN];
    ensure!(
        town.attributes == old.attributes
            && town.events == old.events
            && town.objects.starts_with(&old.objects),
        "original winter identities changed"
    );
    for (label, body) in &old.scripts {
        ensure!(
            town.scripts.get(label) == Some(body),
            "original winter program changed: {label}"
        );
    }
    validate_geometry(&pack, town)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn validate_geometry(pack: &CompiledGamePack, town: &MapModule) -> Result<()> {
    let reached = authored_town_geometry::reachable(pack, town, [20, 25], false)?;
    for y in 0..28 {
        for x in 0..32 {
            let kind = authored_town_geometry::kind(pack, town, [x, y])?;
            ensure!(
                !matches!(
                    kind,
                    "WATER"
                        | "WATERFALL"
                        | "ICE"
                        | "CUT_TREE"
                        | "HOP_DOWN"
                        | "HOP_LEFT"
                        | "HOP_RIGHT"
                        | "LADDER"
                        | "TALL_GRASS"
                ),
                "unexpected winter-town field gate or water"
            );
            if kind == "FLOOR" && !town.objects.iter().any(|o| o.x == x && o.y == y) {
                ensure!(
                    reached.contains(&[x, y]),
                    "orphan winter-town floor [{x},{y}]"
                );
            }
        }
    }
    for actor in &town.objects {
        ensure!(
            authored_town_geometry::kind(pack, town, [actor.x, actor.y])? == "FLOOR",
            "winter-town actor needs ordinary ground"
        );
    }
    for at in [
        [20, 27],
        [21, 27],
        [9, 8],
        [23, 10],
        [15, 14],
        [13, 20],
        [8, 17],
        [23, 17],
        [7, 12],
        [19, 22],
        [27, 12],
        [17, 13],
        [14, 11],
        [27, 14],
    ] {
        ensure!(
            reached.contains(&at),
            "winter service, resident, board or road approach disconnected {at:?}"
        );
    }
    ensure!(
        authored_town_geometry::kind(pack, town, [28, 14])? == "WALL"
            && town.attributes.connections.len() == 1
            && town.attributes.connections[0].direction == "south"
            && town.attributes.connections[0].target_map == "MnR17C00R00"
            && town.attributes.connections[0].offset == -8,
        "retain the real southern road and closed eastern local stop"
    );
    Ok(())
}
