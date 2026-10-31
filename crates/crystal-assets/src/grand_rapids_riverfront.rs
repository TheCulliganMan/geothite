//! Pine-bounded Grand Rapids streets and an optional native fishing pier.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::grand_rapids_riverfront_art;
use crate::{CompiledGamePack, MapModule};

const ROOM: &str = "MnGrandRapids";
const FISHER: &str = "MN_EXTRA_GR_BANK_FISHER";

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
                && self.id == ROOM
                && self.tileset == "johto_modern"
                && self.dimensions_metatiles == [24, 24]
                && self.border_block == 5
                && self.palette_group == "day"
                && self.rows.len() == 24,
            "complete native town source required"
        );
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 24, "complete town columns required");
            for word in words {
                blocks.push(u16::from_str_radix(word, 16).context("native town metatile")?);
            }
        }
        Ok(blocks)
    }
}

/// Rebuilds the complete Grand Rapids exterior with connected, narrower service
/// streets and a riverbank loop ending at a native stone fishing pier. Adds one
/// optional original Minnesota fisher encounter using existing regional scaling.
/// The three complete facades, all original anchors, gate programs, rooms,
/// recovery spawns, seams, outer buffers and river outline remain intact.
/// Original graphics and audio remain unchanged; port paving is imported into
/// a private alias, with its water bound to the native outdoor animation.
///
/// # Errors
/// Rejects stale complete-map inputs, changed facades/ports, obstructed original
/// approaches, disconnected streets, unsafe pier/water boundaries, altered
/// existing programs, invalid regional trainers or an invalid resulting pack.
pub fn rebuild_grand_rapids_riverfront(
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
    let source: Value = serde_json::from_str(after_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let old = base.data.maps.get(ROOM).context("current Grand Rapids")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == ROOM
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(old)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&old.events.warps)?
            && contract["preserved_readings"] == serde_json::to_value(&old.events.bg_events)?
            && contract["preserved_coordinate_events"]
                == serde_json::to_value(&old.events.coord_events)?
            && old.attributes.width == 24
            && old.attributes.height == 24
            && old.attributes.tileset_name == "johto_modern"
            && old.blocks == before.blocks()?,
        "complete Grand Rapids preimage differs"
    );
    let placements: Value = serde_json::from_str(trainers_json)?;
    let trainers = placements["trainers"]
        .as_array()
        .context("riverbank fisher")?;
    ensure!(
        placements["schema_version"] == 1
            && trainers.len() == 1
            && trainers[0]["id"] == FISHER
            && trainers[0]["map"] == ROOM
            && trainers[0]["tile"] == json!([25, 36])
            && trainers[0]["approach"] == json!([26, 36])
            && trainers[0]["optional_battle"] == true
            && trainers[0]["sprite"] == "SPRITE_FISHER",
        "one optional native riverbank fisher required"
    );
    let mut pack = grand_rapids_riverfront_art::install(base, &source)?;
    let mut town = old.clone();
    town.blocks = after.blocks()?;
    town.attributes.tileset_name = grand_rapids_riverfront_art::ALIAS.into();
    for y in 0..24 {
        for x in 0..24 {
            if !(3..21).contains(&x) || !(2..22).contains(&y) || (17..=21).contains(&y) {
                let dock = (12..=13).contains(&x) && (17..=19).contains(&y);
                ensure!(
                    dock || town.blocks[y * 24 + x] == old.blocks[y * 24 + x],
                    "native gate buffers, boundary and river outline must remain"
                );
            }
        }
    }
    for (x0, y0, width, height) in [(4, 4, 2, 2), (13, 5, 3, 2), (19, 6, 2, 2)] {
        for y in y0..y0 + height {
            for x in x0..x0 + width {
                ensure!(
                    town.blocks[y * 24 + x] == old.blocks[y * 24 + x],
                    "complete native facade must remain intact"
                );
            }
        }
    }
    for (x, y) in [(7, 11), (19, 11), (11, 6)] {
        ensure!(
            town.blocks[y * 24 + x] == old.blocks[y * 24 + x],
            "original readable board pixels required"
        );
    }
    validate_geometry(&pack, &town)?;
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
        "minnesota-grand-rapids-riverfront-v1",
        base.data.minnesota_trainer_scaling,
    )?;
    let town = &pack.data.maps[ROOM];
    ensure!(
        town.events.warps == old.events.warps
            && town.events.coord_events == old.events.coord_events
            && town.events.bg_events == old.events.bg_events
            && town.objects.starts_with(&old.objects),
        "original town identities and anchors changed"
    );
    for (label, body) in &old.scripts {
        ensure!(
            town.scripts.get(label) == Some(body),
            "original town program changed: {label}"
        );
    }
    validate_geometry(&pack, town)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn validate_geometry(pack: &CompiledGamePack, town: &MapModule) -> Result<()> {
    let kind = |at| authored_town_geometry::kind(pack, town, at);
    let bodies = town
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let reached = authored_town_geometry::reachable(pack, town, [6, 24], false)?;
    for y in 0..48 {
        for x in 0..48 {
            let at = [x, y];
            let k = kind(at)?;
            ensure!(
                !matches!(
                    k,
                    "CUT_TREE"
                        | "HOP_DOWN"
                        | "HOP_LEFT"
                        | "HOP_RIGHT"
                        | "PIT"
                        | "LADDER"
                        | "CAVE"
                        | "TALL_GRASS"
                ),
                "unexpected town field gate {k}"
            );
            if k == "FLOOR" && !bodies.contains(&at) {
                let inbound_pocket = [0, 1, 46, 47].contains(&x) && [24, 25].contains(&y);
                ensure!(
                    reached.contains(&at) || inbound_pocket,
                    "orphan town floor {at:?}"
                );
            }
            if x == 0 || x == 47 || y == 0 || y == 47 {
                ensure!(
                    !matches!(k, "WATER" | "WATERFALL"),
                    "unconnected outer water must stay closed"
                );
            }
        }
    }
    // The original four officers intentionally separate the eight inbound
    // arrival cells from the city until their native checkpoint script runs.
    // Prove those exact pockets reconnect when only those officers are absent;
    // actor bodies are not treated as permanent terrain barriers.
    let mut open = town.clone();
    open.objects.retain(|o| {
        !matches!(
            o.script.as_str(),
            "MnGrandRapidsWestCheck" | "MnGrandRapidsEastCheck"
        )
    });
    let open_reached = authored_town_geometry::reachable(pack, &open, [6, 24], false)?;
    for x in [0, 1, 46, 47] {
        for y in [24, 25] {
            ensure!(
                open_reached.contains(&[x, y]),
                "original inbound arrival disconnected"
            );
        }
    }
    for actor in &town.objects {
        ensure!(
            kind([actor.x, actor.y])? == "FLOOR",
            "town actor needs ordinary ground"
        );
    }
    for at in [
        [9, 12],
        [39, 16],
        [27, 14],
        [14, 23],
        [38, 23],
        [23, 12],
        [16, 25],
        [31, 33],
        [20, 31],
        [26, 33],
        [26, 36],
        [26, 38],
        [43, 24],
        [43, 25],
    ] {
        ensure!(
            reached.contains(&at),
            "original service/reading/gate or pier approach disconnected {at:?}"
        );
    }
    for y in 17..=18 {
        ensure!(
            town.blocks[y * 24 + 12] == 0x80 && town.blocks[y * 24 + 13] == 0x81,
            "native narrow pier sides required"
        );
    }
    ensure!(
        town.blocks[19 * 24 + 12] == 0x82
            && town.blocks[19 * 24 + 13] == 0x82
            && kind([26, 38])? == "FLOOR"
            && kind([26, 39])? == "WATER",
        "native capped pier launch required"
    );
    Ok(())
}
