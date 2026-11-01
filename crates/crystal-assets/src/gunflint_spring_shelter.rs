//! Hand-authored Gunflint spring paths and an optional native camp rest.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::MapModule;
use crate::authored_town_geometry;
use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;

const ROOM: &str = "MnR12Cave1F";
const OTTA: &str = "MN_R12_OTTA";

#[derive(Deserialize)]
struct Terrain {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    border_block: u16,
    palette_group: String,
    rows: Vec<String>,
    rest: Option<Rest>,
}

#[derive(Deserialize)]
struct Rest {
    actor: String,
    approach: [u16; 2],
    facing: String,
    offer_lines: Vec<String>,
    accepted_lines: Vec<String>,
    done_lines: Vec<String>,
    declined_lines: Vec<String>,
}

impl Terrain {
    fn blocks(&self) -> Result<Vec<u16>> {
        ensure!(self.rows.len() == 12, "complete shelter rows required");
        let mut blocks = Vec::new();
        for row in &self.rows {
            let words = row.split_whitespace().collect::<Vec<_>>();
            ensure!(words.len() == 16, "complete shelter columns required");
            for word in words {
                blocks.push(u16::from_str_radix(word, 16).context("native shelter block")?);
            }
        }
        Ok(blocks)
    }
}

/// Rebuilds the Gunflint shelter around two dry paths and a contained spring.
/// Retains its exterior warp, original actors, Otta's original conversation and
/// the once-only Max Ether. Otta additionally offers a free rest; declining
/// changes no party resources, and accepting heals only after acknowledgement.
/// All other maps, native graphics, encounters and audio remain unchanged.
///
/// # Errors
/// Rejects stale complete-map inputs, changed original anchors or scripts,
/// unsupported native terrain, disconnected paths, unsafe spring boundaries,
/// occupied rest labels, or an invalid resulting pack.
pub fn rebuild_gunflint_spring_shelter(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let before: Terrain = serde_json::from_str(before_json)?;
    let after: Terrain = serde_json::from_str(after_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let original = base
        .data
        .maps
        .get(ROOM)
        .context("current Gunflint shelter")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["map"] == ROOM
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && contract["preserved_warps"] == serde_json::to_value(&original.events.warps)?
            && original.attributes.tileset_name == "cave"
            && original.attributes.width == 16
            && original.attributes.height == 12
            && original.attributes.time_of_day.as_deref() == Some("nite")
            && original.attributes.border_block == 46
            && before.blocks()? == original.blocks,
        "complete Gunflint shelter preimage differs"
    );
    for source in [&before, &after] {
        ensure!(
            source.schema_version == 1
                && source.id == ROOM
                && source.tileset == "cave"
                && source.dimensions_metatiles == [16, 12]
                && source.border_block == 46
                && source.palette_group == "nite",
            "native shelter identity, size and palette required"
        );
    }
    ensure!(
        original
            .objects
            .iter()
            .map(|o| (o.script.as_str(), o.x, o.y))
            .eq([(OTTA, 5, 5), ("MN_R12_CAVE_CACHE", 27, 5)])
            && original.events.warps.len() == 1
            && original.events.warps[0].x == 17
            && original.events.warps[0].y == 23
            && original.events.coord_events.is_empty()
            && original.events.bg_events.is_empty()
            && original.scripts[OTTA] == json!([c("jumptextfaceplayer", &["MN_R12_OTTAText"])])
            && original.scripts["MN_R12_CAVE_CACHE"] == json!([c("itemball", &["MAX_ETHER", "1"])]),
        "original camper, cache, once-only award and exterior warp required"
    );
    let rest = after.rest.as_ref().context("authored camp rest")?;
    ensure!(
        rest.actor == OTTA && rest.approach == [6, 5] && rest.facing == "left",
        "original camper approach required"
    );
    let mut room = original.clone();
    room.blocks = after.blocks()?;
    validate_geometry(base, &room)?;
    for (label, lines) in [
        ("MnGunflintRestOfferText", &rest.offer_lines),
        ("MnGunflintRestAcceptedText", &rest.accepted_lines),
        ("MnGunflintRestDoneText", &rest.done_lines),
        ("MnGunflintRestDeclinedText", &rest.declined_lines),
    ] {
        ensure!(
            !base.data.map_scripts.contains_key(label) && !lines.is_empty(),
            "new nonempty camp rest text required: {label}"
        );
        authoring::text(
            &mut room,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    ensure!(
        !base.data.map_scripts.contains_key("MnGunflintRestDeclined"),
        "rest label occupied"
    );
    room.scripts.insert(
        OTTA.into(),
        json!([
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("writetext", &["MN_R12_OTTAText"]),
            c("waitbutton", &[]),
            c("writetext", &["MnGunflintRestOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnGunflintRestDeclined"]),
            c("writetext", &["MnGunflintRestAcceptedText"]),
            c("waitbutton", &[]),
            c("special", &["HealParty"]),
            c("writetext", &["MnGunflintRestDoneText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    room.scripts.insert(
        "MnGunflintRestDeclined".into(),
        json!([
            c("writetext", &["MnGunflintRestDeclinedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    let names = base.data.map_name_by_constant_from_attributes()?;
    crate::authored_city_expansion_events::finish(base, &mut room, &names)?;
    ensure!(
        room.events == original.events && room.objects == original.objects,
        "original shelter ports and bodies changed"
    );
    for (label, body) in &original.scripts {
        if label != OTTA {
            ensure!(
                room.scripts.get(label) == Some(body),
                "original shelter program changed"
            );
        }
    }
    let mut pack = base.clone();
    crate::authored_town_publication::replace(&mut pack, room)?;
    pack.report
        .manifests
        .push("minnesota-gunflint-spring-shelter-v1".into());
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

fn validate_geometry(pack: &CompiledGamePack, room: &MapModule) -> Result<()> {
    let kind = |at| authored_town_geometry::kind(pack, room, at);
    let bodies = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let seen = authored_town_geometry::reachable(pack, room, [17, 22], false)?;
    let mut water = 0;
    for y in 0..24 {
        for x in 0..32 {
            let at = [x, y];
            let k = kind(at)?;
            ensure!(
                matches!(
                    k,
                    "FLOOR" | "WALL" | "UP_WALL" | "WATER" | "WARP_CARPET_DOWN"
                ),
                "unsupported shelter collision {k}"
            );
            if x == 0 || x == 31 || y == 0 || y == 23 {
                ensure!(
                    k == "WALL" || (y == 23 && [16, 17].contains(&x)),
                    "unconnected shelter perimeter"
                );
            }
            if k == "FLOOR" && !bodies.contains(&at) {
                ensure!(seen.contains(&at), "orphan shelter floor {at:?}");
            }
            if k == "WARP_CARPET_DOWN" {
                ensure!(at == [17, 23], "unpaired shelter exit");
            }
            if k == "WATER" {
                water += 1;
            }
        }
    }
    ensure!(
        (24..=64).contains(&water) && kind([17, 23])? == "WARP_CARPET_DOWN",
        "contained spring and original mouth required"
    );
    for at in [
        [6, 5],
        [26, 5],
        [6, 8],
        [6, 9],
        [26, 8],
        [26, 9],
        [8, 14],
        [20, 16],
        [13, 10],
        [13, 18],
    ] {
        ensure!(
            seen.contains(&at) && kind(at)? == "FLOOR",
            "shelter destination unreachable {at:?}"
        );
    }
    for actor in &bodies {
        ensure!(kind(*actor)? == "FLOOR", "shelter actor embedded in rock");
    }
    ensure!(
        room.blocks[4 * 16 + 3] == 0x30 && room.blocks[4 * 16 + 13] == 0x36,
        "ordinary stair and native right-corner stair required"
    );
    // The central fin must keep distinct west/east dry passages. Remove each
    // side from a geometry projection, then prove the other still connects.
    for east in [false, true] {
        let mut projection = room.clone();
        for y in 6..9 {
            for x in if east { 1..5 } else { 8..11 } {
                projection.blocks[y * 16 + x] = 0x2e;
            }
        }
        ensure!(
            authored_town_geometry::reachable(pack, &projection, [13, 18], false)?
                .contains(&[13, 10]),
            "both dry passages must connect around the stone"
        );
    }
    Ok(())
}
