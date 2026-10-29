//! A hand-authored river-bluff discovery behind a native Strength boulder.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE;
use crystal_core::world::encounters::WildEncounterTable;
use serde::Deserialize;

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc::{map, publish, put};
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-castorock-incisor-gallery-v1";
const ACTOR: &str = "MN_R03_JAW_CASTOROCK";
const BOULDER: &str = "MN_R03_JAW_BOULDER";
const READY: &str = "EVENT_MN_CASTOROCK_TRACKS_READ";
const CAUGHT: &str = "EVENT_MN_CASTOROCK_CAUGHT";
const CLUES: [&str; 3] = [
    "EVENT_MN_CASTOROCK_POOL",
    "EVENT_MN_CASTOROCK_PILLAR",
    "EVENT_MN_CASTOROCK_BED",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Site {
    id: String,
    map: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
    changed_original_blocks: Vec<(u16, u16, String, String)>,
    entrance_preserved: [u16; 2],
    initial_arrival: [u16; 2],
    boulder: [u16; 2],
    boulder_approach: [u16; 2],
    boulder_pushes_to_junction: u16,
    creature: [u16; 2],
    creature_approach: [u16; 2],
    level: u8,
    clues: [[u16; 2]; 3],
    clue_approaches: [[u16; 2]; 3],
    contract: String,
}

/// Extends the existing R03 cave with an optional incisor gallery. The original
/// mouth, pool, walker and pickup remain. Native Strength opens the only throat;
/// three acknowledged observations reveal Castorock. Only actual capture records
/// the permanent claim. Refusal, escape and knockout retain another visit.
///
/// # Errors
/// Rejects incompatible parents, occupied identities, missing reviewed art,
/// foreign/retired wildlife, invalid hand-authored rows, broken gate/return
/// geometry, overlong dialogue, duplicate installation or invalid native output.
pub fn build_minnesota_castorock(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "incisor gallery already installed"
    );
    let site: Site = serde_json::from_str(source)?;
    ensure!(
        site.id == MANIFEST && site.map == "MnR03Cave1F" && site.tileset == "cave",
        "reviewed river-bluff site required"
    );
    ensure!(
        site.level == 24 && !site.contract.is_empty(),
        "reviewed discovery contract required"
    );
    ensure!(
        native(base, "CASTOROCK") && native(base, "CASTOROAR"),
        "active original beaver family required"
    );
    ensure!(
        base.runtime_files
            .contains_key("gfx/pokemon/castorock/overworld.2bpp"),
        "install the reviewed native Castorock walking bank first"
    );
    if let Some(area) = base.data.wild_encounters.get(&site.map) {
        for table in [&area.grass, &area.water].into_iter().flatten() {
            check_wildlife(base, table)?;
        }
        for zone in &area.zones {
            check_wildlife(base, &zone.grass)?;
        }
        for swarm in area.swarm_overrides.values() {
            check_wildlife(base, &swarm.grass)?;
        }
    }
    let mut pack = base.clone();
    let mut room = map(&pack, &site.map)?;
    ensure!(
        room.attributes.tileset_name == site.tileset
            && (room.attributes.width, room.attributes.height) == (8, 6),
        "canonical original cave required"
    );
    ensure!(
        room.events
            .warps
            .iter()
            .any(|w| [w.x, w.y] == site.entrance_preserved),
        "preserve the original cave mouth"
    );
    let [width, height] = site.dimensions_metatiles;
    ensure!(
        (width, height) == (14, 8) && site.rows.len() == usize::from(height),
        "reviewed whole cave dimensions required"
    );
    let rows = site
        .rows
        .iter()
        .map(|row| {
            row.split_whitespace()
                .map(|v| u16::from_str_radix(v, 16).context("native cave block"))
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        rows.iter().all(|row| row.len() == usize::from(width)),
        "native row width"
    );
    let changes = site
        .changed_original_blocks
        .iter()
        .map(|(x, y, old, new)| {
            Ok((
                (*x, *y),
                (u16::from_str_radix(old, 16)?, u16::from_str_radix(new, 16)?),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    ensure!(
        changes.len() == 3,
        "three reviewed throat/return changes required"
    );
    for y in 0..6_u16 {
        for x in 0..8_u16 {
            let old = room.blocks[usize::from(y) * 8 + usize::from(x)];
            let new = rows[usize::from(y)][usize::from(x)];
            if let Some(expected) = changes.get(&(x, y)) {
                ensure!(
                    *expected == (old, new),
                    "reviewed old-cell change differs at {x},{y}"
                );
            } else {
                ensure!(old == new, "unreviewed original cave change at {x},{y}");
            }
        }
    }
    room.attributes.width = width;
    room.attributes.height = height;
    room.blocks = rows.into_iter().flatten().collect();
    for flag in CLUES.into_iter().chain([READY, CAUGHT]) {
        a::flag(&mut pack, flag)?;
    }
    for at in [
        site.initial_arrival,
        site.boulder,
        site.boulder_approach,
        site.creature,
        site.creature_approach,
    ]
    .into_iter()
    .chain(site.clue_approaches)
    {
        ensure!(
            crate::authored_town_geometry::kind(&pack, &room, at)? == "FLOOR",
            "native dry approach at {at:?}"
        );
    }
    ensure!(
        site.boulder == [site.boulder_approach[0] + 1, site.boulder_approach[1]]
            && site.boulder_pushes_to_junction == 2,
        "native eastward two-step throat"
    );
    for distance in 1..=site.boulder_pushes_to_junction {
        ensure!(
            crate::authored_town_geometry::kind(
                &pack,
                &room,
                [site.boulder[0] + distance, site.boulder[1]]
            )? == "FLOOR",
            "clear boulder destination"
        );
    }
    a::actor(&pack, &mut room, BOULDER, "SPRITE_BOULDER", site.boulder)?;
    room.objects
        .last_mut()
        .context("new boulder")?
        .spritemovedata = "SPRITEMOVEDATA_STRENGTH_BOULDER".into();
    put(
        &mut room,
        BOULDER,
        vec![c("jumpstd", &["StrengthBoulderScript"])],
    );
    a::actor(&pack, &mut room, ACTOR, "SPRITE_CASTOROCK", site.creature)?;
    let before =
        crate::authored_town_geometry::reachable(&pack, &room, site.initial_arrival, true)?;
    ensure!(
        before.contains(&site.boulder_approach) && !before.contains(&site.creature_approach),
        "boulder must seal the optional gallery, including Surf bypasses"
    );
    let mut opened = room.clone();
    opened
        .objects
        .retain(|o| o.object_identifier.as_deref() != Some(BOULDER));
    let after =
        crate::authored_town_geometry::reachable(&pack, &opened, site.initial_arrival, false)?;
    ensure!(
        after.contains(&site.creature_approach)
            && site.clue_approaches.iter().all(|at| after.contains(at)),
        "all observations and encounter need a dry return"
    );
    for y in 0..height * 2 {
        for x in 0..width * 2 {
            if x == 0 || y == 0 || x == width * 2 - 1 || y == height * 2 - 1 {
                ensure!(
                    crate::authored_town_geometry::kind(&pack, &room, [x, y])? == "WALL",
                    "visible closed cave border at {x},{y}"
                );
            }
        }
    }
    text(
        &mut room,
        "MN_R03_CAVE_WALKERText",
        &[
            "Three marks: pool,",
            "split pillar, bed.",
            "Read each closely.",
            "The mill foreman",
            "in MINNEAPOLIS",
            "loans HM STRENGTH",
            "after the CURRENT",
            "BADGE. Move the",
            "rock to explore.",
        ],
    )?;
    for (index, label) in ["MnJawPool", "MnJawPillar", "MnJawBed"].iter().enumerate() {
        let at = site.clues[index];
        ensure!(
            crate::authored_town_geometry::kind(&pack, &room, at)? == "WALL",
            "readable rock marker {at:?}"
        );
        let text_label = format!("{label}Text");
        put(
            &mut room,
            label,
            vec![
                c("opentext", &[]),
                c("writetext", &[&text_label]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("setevent", &[CLUES[index]]),
                c("scall", &["MnJawCheck"]),
                c("end", &[]),
            ],
        );
        a::read(&mut room, at, label);
    }
    text(
        &mut room,
        "MnJawPoolText",
        &[
            "An old waterline.",
            "Paired grooves cut",
            "across the stone.",
            "You sketch them.",
        ],
    )?;
    text(
        &mut room,
        "MnJawPillarText",
        &[
            "The pillar carries",
            "matching grooves.",
            "They follow a wide",
            "curve, not a tool.",
            "You compare marks.",
        ],
    )?;
    text(
        &mut room,
        "MnJawBedText",
        &[
            "Fresh wood lies in",
            "an ancient bed.",
            "The same broad",
            "toothmarks remain.",
            "Life still stirs.",
        ],
    )?;
    let mut check = vec![
        c("checkevent", &[CAUGHT]),
        c("iftrue", &["MnJawReturn"]),
        c("checkevent", &[READY]),
        c("iftrue", &["MnJawReturn"]),
    ];
    for flag in CLUES {
        check.extend([c("checkevent", &[flag]), c("iffalse", &["MnJawReturn"])]);
    }
    check.extend([
        c("cry", &["CASTOROCK"]),
        c("appear", &[ACTOR]),
        c("opentext", &[]),
        c("writetext", &["MnJawRevealText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("setevent", &[READY]),
        c("return", &[]),
    ]);
    put(&mut room, "MnJawCheck", check);
    put(&mut room, "MnJawReturn", vec![c("return", &[])]);
    text(
        &mut room,
        "MnJawRevealText",
        &[
            "Stone shifts near",
            "the old river bed.",
            "CASTOROCK emerges!",
        ],
    )?;
    put(
        &mut room,
        ACTOR,
        vec![
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("writetext", &["MnJawOfferText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnJawDecline"]),
            c("closetext", &[]),
            c("cry", &["CASTOROCK"]),
            c(
                "loadvar",
                &["VAR_BATTLETYPE", OPTIONAL_LEGENDARY_BATTLE_TYPE],
            ),
            c("loadwildmon", &["CASTOROCK", &site.level.to_string()]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("special", &["CheckCaughtCelebi"]),
            c("iftrue", &["MnJawCaught"]),
            c("disappear", &[ACTOR]),
            c("jumptext", &["MnJawRetreatText"]),
        ],
    );
    put(
        &mut room,
        "MnJawCaught",
        vec![
            c("setevent", &[CAUGHT]),
            c("disappear", &[ACTOR]),
            c("jumptext", &["MnJawCaughtText"]),
        ],
    );
    put(
        &mut room,
        "MnJawDecline",
        vec![
            c("writetext", &["MnJawDeclineText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    text(
        &mut room,
        "MnJawOfferText",
        &["CASTOROCK waits by", "its old shelter.", "Approach it?"],
    )?;
    text(&mut room, "MnJawDeclineText", &["You let it rest."])?;
    text(
        &mut room,
        "MnJawRetreatText",
        &["It retreats.", "Leave the cave and", "return for a try."],
    )?;
    text(
        &mut room,
        "MnJawCaughtText",
        &["The river's old", "friend joins you."],
    )?;
    put(
        &mut room,
        "MnJawRestore",
        vec![
            c("checkevent", &[CAUGHT]),
            c("iftrue", &["MnJawHide"]),
            c("checkevent", &[READY]),
            c("iffalse", &["MnJawHide"]),
            c("appear", &[ACTOR]),
            c("endcallback", &[]),
        ],
    );
    put(
        &mut room,
        "MnJawHide",
        vec![c("disappear", &[ACTOR]), c("endcallback", &[])],
    );
    a::callback(&mut room, "MAPCALLBACK_OBJECTS", "MnJawRestore");
    room.scripted_wild_battles = crate::parse_scripted_wild_battles(&room.id, &room.scripts)?;
    publish(&mut pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn native(pack: &CompiledGamePack, species: &str) -> bool {
    pack.data.pokemon.contains_key(species)
        && pack
            .data
            .story_event_script_constants
            .global
            .contains_key(&format!("MN_DEX_MEMBER_{species}"))
}

fn check_wildlife(pack: &CompiledGamePack, table: &WildEncounterTable) -> Result<()> {
    for slot in table.morning.iter().chain(&table.day).chain(&table.night) {
        ensure!(
            native(pack, &slot.species),
            "foreign or retired cave wildlife: {}",
            slot.species
        );
    }
    Ok(())
}

fn text(room: &mut MapModule, label: &str, lines: &[&str]) -> Result<()> {
    ensure!(
        lines.iter().all(|line| line.chars().count() <= 18),
        "native text row exceeds 18 columns: {label}"
    );
    authoring::text(room, label, lines);
    Ok(())
}
