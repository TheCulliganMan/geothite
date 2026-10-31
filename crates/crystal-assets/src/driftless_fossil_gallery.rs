//! A connected optional fossil gallery below the lit Driftless tutorial hollow.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::authored_town_geometry;
use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command;
use crate::minnesota_field_use;
use crate::northstar_lair_authoring;
use crate::{CompiledGamePack, MapModule};

const HOLLOW: &str = "MnR01Cave1F";
const GALLERY: &str = "MnDriftlessLampGallery";
const CONSTANT: &str = "MN_DRIFTLESS_LAMP_GALLERY";
const READ: &str = "EVENT_MN_LAMP_GALLERY_READ";
const CLAIMED: &str = "EVENT_MN_LAMP_GALLERY_CACHE";
const HIDDEN: &str = "EVENT_MN_LAMP_GALLERY_HIDDEN";
const GUIDE_TEXT: &str = "MN_R01_HOLLOW_GUIDEText";

#[derive(Deserialize)]
struct Gallery {
    schema_version: u8,
    id: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    border_block: u8,
    palette_group: String,
    rows: Vec<String>,
    parent: String,
    parent_stair_metatile: [u16; 2],
    parent_before: String,
    parent_after: String,
    parent_active: [u16; 2],
    parent_arrival: [u16; 2],
    active_exit: [u16; 2],
    passive_arrival: [u16; 2],
    cache: [u16; 2],
    cache_approach: [u16; 2],
    cache_facing: String,
    reward: String,
    quantity: u16,
    flag: String,
    read: [u16; 2],
    read_approach: [u16; 2],
    read_facing: String,
    read_flag: String,
    visibility_flag: String,
    read_requires_native_flash: bool,
    read_pages: Vec<Vec<String>>,
    guide_hint: Vec<String>,
}

/// Adds the dry, unlit fossil gallery and its paired stair below Driftless Hollow.
/// Native Flash and acknowledged fossil text reveal a once-only sample cache.
/// A full item pocket leaves the claim available for retry. The lit hollow,
/// original HM gift and Antidote, world exit, all other maps and all art/audio
/// remain intact. No blackout or Fly destination is added.
///
/// # Errors
/// Rejects stale whole-parent geometry/programs, occupied identities, unsupported
/// native terrain, disconnected approaches, unsafe stair pairings, missing native
/// Flash source, changed unrelated content, or invalid compiled output.
pub fn build_driftless_fossil_gallery(
    base: &CompiledGamePack,
    source_json: &str,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let source: Gallery = serde_json::from_str(source_json)?;
    let contract: Value = serde_json::from_str(contract_json)?;
    let original = base.data.maps.get(HOLLOW).context("current lit hollow")?;
    ensure!(
        contract["schema_version"] == 1
            && contract["native_parent_map"] == HOLLOW
            && contract["native_module_sha256"]
                == format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                )
            && original.attributes.width == 8
            && original.attributes.height == 7
            && original.attributes.time_of_day.as_deref() == Some("nite")
            && original.blocks[4 * 8 + 5] == 0x02
            && original.events.warps.len() == 1
            && contract["preserved_exterior_warp"]
                == serde_json::to_value(&original.events.warps[0])?
            && !base.data.maps.contains_key(GALLERY),
        "current complete hollow preimage differs"
    );
    let [width, height] = source.dimensions_metatiles;
    ensure!(
        source.schema_version == 1
            && source.id == GALLERY
            && source.tileset == "cave"
            && (8..=24).contains(&width)
            && (8..=24).contains(&height)
            && source.border_block == 0x2e
            && source.palette_group == "dark"
            && source.parent == HOLLOW
            && source.parent_stair_metatile == [5, 4]
            && source.parent_before == "02"
            && source.parent_after == "0b"
            && source.parent_active == [11, 9]
            && source.parent_arrival == [10, 9]
            && source.flag == CLAIMED
            && source.read_flag == READ
            && source.visibility_flag == HIDDEN
            && source.read_requires_native_flash
            && source.reward == "RARE_CANDY"
            && source.quantity == 1
            && source.rows.len() == usize::from(height)
            && source
                .rows
                .iter()
                .all(|row| row.split_whitespace().count() == usize::from(width))
            && source.read_pages.len() >= 2
            && source.read_pages.iter().all(|page| page.len() == 2),
        "complete native fossil gallery and acknowledged Flash discovery required"
    );
    for line in source.read_pages.iter().flatten().chain(&source.guide_hint) {
        ensure!(
            !line.is_empty() && line.is_ascii() && line.len() <= 18,
            "native fossil text must fit an 18-character line"
        );
    }
    let mut pack = base.clone();
    for flag in [READ, CLAIMED, HIDDEN] {
        northstar_lair_authoring::flag(&mut pack, flag)?;
    }
    install_native_flash(&mut pack)?;
    let mut gallery = authoring::blank(base, HOLLOW, GALLERY, CONSTANT)?;
    authoring::blocks(&mut gallery, width, height, &source.rows.join("\n"))?;
    ensure!(
        gallery.blocks.iter().all(|b| *b < 0x40),
        "native cave art only"
    );
    gallery.attributes.border_block = source.border_block;
    gallery.attributes.palette = Some("dark".into());
    gallery.attributes.time_of_day = Some("dark".into());
    gallery.attributes.phone_service = (gallery.attributes.phone_service & 0xf0) | 4;
    gallery.attributes.fishing_group = Some("FISHGROUP_NONE".into());
    let hollow_constant = original
        .attributes
        .map_constant
        .as_deref()
        .context("hollow constant")?;
    let [x, y] = source.active_exit;
    authoring::warp(&mut gallery, (x, y), hollow_constant, 3);
    let [x, y] = source.passive_arrival;
    authoring::warp(&mut gallery, (x, y), CONSTANT, 2);
    minnesota_field_use::cache(
        &pack,
        &mut gallery,
        "MN_LAMP_GALLERY_CACHE",
        source.cache,
        &source.reward,
        CLAIMED,
        HIDDEN,
    )?;
    author_fossils(&mut gallery, &source);
    validate_geometry(&pack, &gallery, &source)?;

    // Refresh only the new warp wire and the appended hint's text body. Existing
    // HM dispatch, item and runtime command tables are not re-lowered.
    let mut hollow = original.clone();
    hollow.blocks[4 * 8 + 5] = 0x0b;
    authoring::warp(&mut hollow, (11, 9), CONSTANT, 2);
    authoring::warp(&mut hollow, (10, 9), hollow_constant, 3);
    minnesota_field_use::hint(
        &mut hollow,
        GUIDE_TEXT,
        &source
            .guide_hint
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )?;
    let mut text = crate::parse_script_text_bodies(HOLLOW, &hollow.scripts)?;
    hollow.script_text_bodies.insert(
        GUIDE_TEXT.into(),
        text.remove(GUIDE_TEXT).context("appended guide text")?,
    );
    authoring::refresh_event_section(&mut hollow);
    let reached = authored_town_geometry::reachable(&pack, &hollow, [9, 10], false)?;
    ensure!(
        reached.contains(&[10, 9]) && reached.contains(&[11, 9]) && reached.contains(&[5, 6]),
        "hollow stair, passive return and original guide must connect"
    );
    crate::authored_town_publication::replace(&mut pack, hollow)?;
    northstar_lair_authoring::publish(&mut pack, gallery, HOLLOW, source.passive_arrival)?;
    // The generic room allocator inherits the anchor's metadata. This deeper
    // room has its own native DARK selector, including for menu OBJ palettes.
    pack.data
        .runtime_map_metadata
        .get_mut(CONSTANT)
        .context("new gallery metadata")?
        .phone_service = (original.attributes.phone_service & 0xf0) | 4;
    for (name, map) in &base.data.maps {
        if name != HOLLOW {
            ensure!(
                pack.data.maps.get(name) == Some(map),
                "unrelated map changed: {name}"
            );
        }
    }
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio,
        "native art and audio must remain exact"
    );
    pack.report
        .manifests
        .push("minnesota-driftless-fossil-gallery-v1".into());
    pack.report.maps = pack.data.maps.len();
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

fn install_native_flash(pack: &mut CompiledGamePack) -> Result<()> {
    let standard = pack
        .data
        .story_events
        .iter_mut()
        .find_map(|catalog| catalog.get_mut("StandardScripts"))
        .context("retained standard scripts")?;
    let roots = standard["GlobalScriptRoots"]
        .as_array_mut()
        .context("native global roots")?;
    if !roots.iter().any(|root| root == "Script_UseFlash") {
        roots.push(Value::String("Script_UseFlash".into()));
    }
    pack.data.materialize_global_scripts()?;
    ensure!(
        pack.data
            .global_scripts
            .as_ref()
            .is_some_and(|m| m.scripts.contains_key("Script_UseFlash")),
        "retained native Flash executable source required"
    );
    Ok(())
}

fn author_fossils(gallery: &mut MapModule, source: &Gallery) {
    northstar_lair_authoring::read(gallery, source.read, "MnLampFossils");
    gallery.scripts.insert(
        "MnLampFossils".into(),
        json!([
            command("checkflag", &["STATUSFLAGS_FLASH"]),
            command("iffalse", &["MnLampTooDark"]),
            command("opentext", &[]),
            command("writetext", &["MnLampFossilsText"]),
            command("waitbutton", &[]),
            command("setevent", &[READ]),
            command("checkevent", &[CLAIMED]),
            command("iftrue", &["MnLampReadDone"]),
            command("appear", &["MN_LAMP_GALLERY_CACHE"]),
            command("sjump", &["MnLampReadDone"])
        ]),
    );
    gallery.scripts.insert(
        "MnLampReadDone".into(),
        json!([command("closetext", &[]), command("end", &[])]),
    );
    gallery.scripts.insert(
        "MnLampTooDark".into(),
        json!([command("jumptext", &["MnLampTooDarkText"])]),
    );
    authoring::text(
        gallery,
        "MnLampTooDarkText",
        &["Too dark to make", "out the markings."],
    );
    let lines = source
        .read_pages
        .iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>();
    authoring::text(gallery, "MnLampFossilsText", &lines);
    gallery.scripts.insert(
        "MnLampObjects".into(),
        json!([
            command("checkevent", &[CLAIMED]),
            command("iftrue", &["MnLampHide"]),
            command("checkevent", &[READ]),
            command("iffalse", &["MnLampHide"]),
            command("appear", &["MN_LAMP_GALLERY_CACHE"]),
            command("endcallback", &[])
        ]),
    );
    gallery.scripts.insert(
        "MnLampHide".into(),
        json!([
            command("disappear", &["MN_LAMP_GALLERY_CACHE"]),
            command("endcallback", &[])
        ]),
    );
    northstar_lair_authoring::callback(gallery, "MAPCALLBACK_OBJECTS", "MnLampObjects");
}

fn validate_geometry(pack: &CompiledGamePack, gallery: &MapModule, source: &Gallery) -> Result<()> {
    let kind = |at| authored_town_geometry::kind(pack, gallery, at);
    let reached = authored_town_geometry::reachable(pack, gallery, source.passive_arrival, false)?;
    ensure!(
        kind(source.passive_arrival)? == "FLOOR"
            && kind(source.active_exit)? == "LADDER"
            && source.passive_arrival[0].abs_diff(source.active_exit[0])
                + source.passive_arrival[1].abs_diff(source.active_exit[1])
                == 1,
        "adjacent active ladder and passive Floor required"
    );
    ensure!(
        kind(source.read)? == "WALL"
            && reached.contains(&source.read_approach)
            && facing_matches(source.read_approach, source.read, &source.read_facing)
            && source.read[0].abs_diff(source.read_approach[0])
                + source.read[1].abs_diff(source.read_approach[1])
                == 1,
        "readable fossil wall required"
    );
    ensure!(
        kind(source.cache)? == "FLOOR"
            && reached.contains(&source.cache_approach)
            && facing_matches(source.cache_approach, source.cache, &source.cache_facing)
            && source.cache[0].abs_diff(source.cache_approach[0])
                + source.cache[1].abs_diff(source.cache_approach[1])
                == 1,
        "reachable sample cache required"
    );
    for y in 0..gallery.attributes.height * 2 {
        for x in 0..gallery.attributes.width * 2 {
            let at = [x, y];
            let k = kind(at)?;
            if x == 0
                || y == 0
                || x == gallery.attributes.width * 2 - 1
                || y == gallery.attributes.height * 2 - 1
            {
                ensure!(k == "WALL", "closed gallery border required");
            }
            if k == "FLOOR" && at != source.cache {
                ensure!(reached.contains(&at), "stranded gallery floor: {at:?}");
            }
            ensure!(
                matches!(k, "FLOOR" | "WALL" | "UP_WALL" | "LADDER"),
                "dry ordinary native gallery terrain required"
            );
            if k == "LADDER" {
                ensure!(at == source.active_exit, "unpaired gallery ladder");
            }
        }
    }
    Ok(())
}

fn facing_matches(from: [u16; 2], to: [u16; 2], facing: &str) -> bool {
    matches!(
        (
            i32::from(to[0]) - i32::from(from[0]),
            i32::from(to[1]) - i32::from(from[1]),
            facing
        ),
        (1, 0, "right") | (-1, 0, "left") | (0, 1, "down") | (0, -1, "up")
    )
}
