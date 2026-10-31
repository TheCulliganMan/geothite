//! Optional native field discoveries for Flash and Whirlpool.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::minnesota_field_use_art as art;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;
use crate::{CompiledGamePack, MapModule};

const MARKER: &str = "minnesota-native-field-uses-v1";
const HOLLOW: &str = "MnR01Cave1F";
const GALLERY: &str = "MnDriftlessLampGallery";
const GALLERY_CONSTANT: &str = "MN_DRIFTLESS_LAMP_GALLERY";
const READ: &str = "EVENT_MN_LAMP_GALLERY_READ";
const LAMP_CACHE: &str = "EVENT_MN_LAMP_GALLERY_CACHE";
const LAMP_HIDDEN: &str = "EVENT_MN_LAMP_GALLERY_HIDDEN";
const EDDY_CACHE: &str = "EVENT_MN_LEECH_EDDY_CACHE";

/// Adds an optional unlit fossil gallery and a Whirlpool-gated hidden lake cache.
///
/// The existing hollow stays lit. Its added stair uses separate active and
/// passive arrivals. Fossil evidence requires actual native Flash illumination
/// and acknowledgment; the rock cache has one Whirlpool-controlled approach.
/// Existing map edges, story gates, original art and audio are retained. The
/// pinned local native pushback source fills the missing current script; existing
/// source programs must agree exactly.
/// The borrowed input is unchanged and no save identity is migrated.
///
/// # Errors
/// Rejects changed site rows or anchors, duplicate identities, incompatible
/// native art, occupied stairs/cache sites, foreign gallery encounters, altered
/// unrelated maps or original assets, and invalid resulting runtime content.
pub fn build_minnesota_field_use_pack(
    base: &CompiledGamePack,
    source: &str,
    forced_movement_source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let site: Value = serde_json::from_str(source)?;
    ensure!(
        site["schema_version"] == 1 && site["id"] == MARKER,
        "reviewed field-use source"
    );
    for (section, expected) in [
        (
            "gallery",
            json!({"map":GALLERY,"constant":GALLERY_CONSTANT,"parent":HOLLOW,
            "parent_stair_metatile":[5,4],"parent_before":"02","parent_after":"0b",
            "parent_active":[11,9],"parent_arrival":[10,9],"dimensions_metatiles":[8,8],
            "palette":"dark","time_of_day":"dark","visibility_flag":LAMP_HIDDEN,"active_exit":[9,13],"passive_arrival":[8,13],
            "cache":[8,4],"reward":"RARE_CANDY","quantity":1,"flag":LAMP_CACHE,
            "read":[6,6],"read_approach":[5,6],"read_flag":READ,"read_requires_native_flash":true,
            "read_pages":[["Roots once grew","through this seam."],["FLASH reveals the","fine fossil rings."]]}),
        ),
        (
            "eddy",
            json!({"map":"MnLeechLake","source_tileset":art::TILESET,
            "source_metatile_count":134,"cap_metatile":[30,9],"eddy_metatile":[30,10],
            "before":"59","cap_after":"36","closed_block":"07","open_block":"36",
            "closed_collision":["WHIRLPOOL","BUOY","WATER","BUOY"],
            "open_collision":["WATER","BUOY","WATER","BUOY"],
            "approach":[59,20],"current":[60,20],"cache_approach":[60,20],"cache":[61,20],
            "cache_kind":"hiddenitem","reward":"PP_UP","quantity":1,"flag":EDDY_CACHE,
            "native_whirlpool_frame_ticks":11}),
        ),
    ] {
        for (key, value) in expected.as_object().context("field-site contract")? {
            ensure!(
                site[section][key] == *value,
                "reviewed field site changed: {section}/{key}"
            );
        }
    }
    ensure!(
        !base.data.maps.contains_key(GALLERY) && !base.report.manifests.iter().any(|m| m == MARKER),
        "field uses already installed"
    );
    let rows = [
        "2e 2e 2e 2e 2e 2e 2e 2e",
        "2e 2e 04 05 06 2e 2e 2e",
        "2e 04 02 02 02 06 2e 2e",
        "2e 08 02 1d 02 02 06 2e",
        "2e 20 02 02 02 02 0a 2e",
        "2e 2e 20 21 02 02 0a 2e",
        "2e 2e 2e 2e 0b 02 0a 2e",
        "2e 2e 2e 2e 2e 2e 2e 2e",
    ];
    ensure!(
        site["gallery"]["rows"] == json!(rows),
        "reviewed native gallery rows"
    );
    let mut hollow = arc::map(base, HOLLOW)?;
    ensure!(
        hollow.attributes.width == 8
            && hollow.attributes.height == 7
            && hollow.attributes.palette.as_deref() == Some("PALETTE_NITE")
            && hollow.blocks[4 * 8 + 5] == 2
            && hollow.events.warps.len() == 1
            && hollow.events.warps[0].x == 9
            && hollow.events.warps[0].y == 11
            && hollow.events.warps[0].target_map_constant == "MnR01C00R00"
            && hollow.events.warps[0].target_warp_id == 2,
        "reviewed lit hollow stair preimage"
    );
    let mut lake = arc::map(base, "MnLeechLake")?;
    ensure!(
        lake.attributes.width == 32
            && lake.attributes.height == 24
            && lake.attributes.tileset_name == "mn_headwaters"
            && lake.blocks[9 * 32 + 30] == 0x59
            && lake.blocks[10 * 32 + 30] == 0x59,
        "reviewed Leech Lake cove preimage"
    );
    for (map, point) in [(&hollow, [11, 9]), (&hollow, [10, 9]), (&lake, [61, 20])] {
        ensure!(
            !map.objects.iter().any(|o| [o.x, o.y] == point)
                && !map.events.warps.iter().any(|w| [w.x, w.y] == point)
                && !map.events.bg_events.iter().any(|e| [e.x, e.y] == point),
            "occupied field anchor"
        );
    }
    let mut pack = base.clone();
    for flag in [READ, LAMP_CACHE, LAMP_HIDDEN, EDDY_CACHE] {
        a::flag(&mut pack, flag)?;
    }
    art::install(&mut pack)?;
    crate::minnesota_forced_movement_source::install(&mut pack, forced_movement_source)?;
    // This native menu entry has no pointer-table caller. Retain its existing
    // source body as an executable root when adding the region's first current.
    let standard = pack
        .data
        .story_events
        .iter_mut()
        .find_map(|catalog| catalog.get_mut("StandardScripts"))
        .context("native standard-script catalog")?;
    let roots = standard["GlobalScriptRoots"]
        .as_array_mut()
        .context("native field-script roots")?;
    for label in [
        "Script_UsedWhirlpool",
        "Script_ForcedMovement",
        "Script_UseFlash",
    ] {
        if !roots.iter().any(|root| root == label) {
            roots.push(Value::String(label.into()));
        }
    }
    pack.data.materialize_global_scripts()?;
    ensure!(
        pack.data
            .global_scripts
            .as_ref()
            .is_some_and(|module| module.scripts.contains_key("Script_UsedWhirlpool")),
        "retained native Whirlpool menu entry"
    );
    let mut gallery = authoring::blank(base, HOLLOW, GALLERY, GALLERY_CONSTANT)?;
    authoring::blocks(&mut gallery, 8, 8, &rows.join("\n"))?;
    gallery.attributes.border_block = 0x2e;
    gallery.attributes.palette = Some("dark".into());
    gallery.attributes.time_of_day = Some("dark".into());
    authoring::warp(
        &mut gallery,
        (9, 13),
        hollow
            .attributes
            .map_constant
            .as_deref()
            .context("hollow constant")?,
        3,
    );
    authoring::warp(&mut gallery, (8, 13), GALLERY_CONSTANT, 2);
    hollow.blocks[4 * 8 + 5] = 0x0b;
    authoring::warp(&mut hollow, (11, 9), GALLERY_CONSTANT, 2);
    let hollow_constant = hollow
        .attributes
        .map_constant
        .clone()
        .context("hollow constant")?;
    authoring::warp(&mut hollow, (10, 9), &hollow_constant, 3);
    cache(
        &pack,
        &mut gallery,
        "MN_LAMP_GALLERY_CACHE",
        [8, 4],
        "RARE_CANDY",
        LAMP_CACHE,
        LAMP_HIDDEN,
    )?;
    a::read(&mut gallery, [6, 6], "MnLampFossils");
    gallery.scripts.insert(
        "MnLampFossils".into(),
        json!([
            c("checkflag", &["STATUSFLAGS_FLASH"]),
            c("iffalse", &["MnLampTooDark"]),
            c("opentext", &[]),
            c("writetext", &["MnLampFossilsText"]),
            c("waitbutton", &[]),
            c("setevent", &[READ]),
            c("checkevent", &[LAMP_CACHE]),
            c("iftrue", &["MnLampReadDone"]),
            c("appear", &["MN_LAMP_GALLERY_CACHE"]),
            c("sjump", &["MnLampReadDone"])
        ]),
    );
    gallery.scripts.insert(
        "MnLampReadDone".into(),
        json!([c("closetext", &[]), c("end", &[])]),
    );
    gallery.scripts.insert(
        "MnLampTooDark".into(),
        json!([c("jumptext", &["MnLampTooDarkText"])]),
    );
    authoring::text(
        &mut gallery,
        "MnLampTooDarkText",
        &["Too dark to make", "out the markings."],
    );
    authoring::text(
        &mut gallery,
        "MnLampFossilsText",
        &[
            "Roots once grew",
            "through this seam.",
            "FLASH reveals the",
            "fine fossil rings.",
        ],
    );
    gallery.scripts.insert(
        "MnLampObjects".into(),
        json!([
            c("checkevent", &[LAMP_CACHE]),
            c("iftrue", &["MnLampHide"]),
            c("checkevent", &[READ]),
            c("iffalse", &["MnLampHide"]),
            c("appear", &["MN_LAMP_GALLERY_CACHE"]),
            c("endcallback", &[])
        ]),
    );
    gallery.scripts.insert(
        "MnLampHide".into(),
        json!([
            c("disappear", &["MN_LAMP_GALLERY_CACHE"]),
            c("endcallback", &[])
        ]),
    );
    a::callback(&mut gallery, "MAPCALLBACK_OBJECTS", "MnLampObjects");
    lake.blocks[9 * 32 + 30] = 0x36;
    lake.blocks[10 * 32 + 30] = art::CLOSED;
    lake.scripts.insert(
        "MN_LEECH_EDDY_CACHE".into(),
        json!([c("hiddenitem", &["PP_UP", EDDY_CACHE])]),
    );
    lake.events
        .bg_events
        .push(crystal_core::map::BackgroundEvent {
            x: 61,
            y: 20,
            event_type: "BGEVENT_ITEM".into(),
            script: "MN_LEECH_EDDY_CACHE".into(),
        });
    lake.script_field_pickups =
        crate::parse_script_field_pickups(&lake.id, &lake.scripts, &lake.objects)?;
    hint(
        &mut hollow,
        "MN_R01_HOLLOW_GUIDEFieldHMGivenText",
        &[
            "A lower gallery is",
            "unlit. FLASH shows",
            "the fossil rings.",
        ],
    )?;
    hint(
        &mut lake,
        "MN_LEECH_SHORE_FISHERText",
        &[
            "An eddy hides a",
            "tiny eastern cove.",
            "WHIRLPOOL clears",
            "the current.",
            "Try ITEMFINDER at",
            "the eastern rocks.",
        ],
    )?;
    let mut bemidji = arc::map(base, "MnBemidji")?;
    hint(
        &mut bemidji,
        "MN_BEMIDJI_PIER_FISHERFieldHMGivenText",
        &[
            "Try the eddy on",
            "LEECH LAKE's east",
            "shore. A rock cove",
            "lies behind it.",
        ],
    )?;
    verify_geometry(&pack, &hollow, &gallery, &lake)?;
    arc::publish(&mut pack, hollow)?;
    arc::publish(&mut pack, lake)?;
    arc::publish(&mut pack, bemidji)?;
    for (name, label) in [
        (HOLLOW, "MN_R01_HOLLOW_GUIDEFieldHMGivenText"),
        ("MnLeechLake", "MN_LEECH_SHORE_FISHERText"),
        ("MnBemidji", "MN_BEMIDJI_PIER_FISHERFieldHMGivenText"),
    ] {
        let mut changed = BTreeSet::from([label.to_owned()]);
        if name == "MnLeechLake" {
            changed.insert("MN_LEECH_EDDY_CACHE".to_owned());
        }
        let original = &base.data.maps[name];
        let revised = pack
            .data
            .maps
            .get_mut(name)
            .context("published field hint map")?;
        crate::northstar_story_refresh::preserve_unrevised_rows(
            &mut revised.script_control_commands,
            &original.script_control_commands,
            &changed,
            |row| &row.source_script,
        );
        crate::northstar_story_refresh::preserve_unrevised_rows(
            &mut revised.script_text_commands,
            &original.script_text_commands,
            &changed,
            |row| &row.source_script,
        );
        if name == "MnBemidji" {
            // This map only gains directions. Its event wire and cached
            // stair aliases must not become unrelated source edits.
            revised.map_event_section_commands = original.map_event_section_commands.clone();
            revised.map_script_section_commands = original.map_script_section_commands.clone();
            for label in [
                original.attributes.map_scripts_label.as_ref(),
                original.attributes.map_events_label.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                if let Some(body) = base.data.map_scripts.get(label) {
                    pack.data.map_scripts.insert(label.clone(), body.clone());
                } else {
                    pack.data.map_scripts.remove(label);
                }
            }
        }
        pack.data.map_attributes.insert(
            name.to_owned(),
            base.data
                .map_attributes
                .get(name)
                .context("original field map attributes")?
                .clone(),
        );
        if let Some(actors) = base.data.npcs.get(name) {
            pack.data.npcs.insert(name.to_owned(), actors.clone());
        } else {
            pack.data.npcs.remove(name);
        }
    }
    a::publish(&mut pack, gallery, HOLLOW, [8, 13])?;
    if let Some(original) = base.data.wild_encounters.get(HOLLOW) {
        let value = serde_json::to_value(original)?;
        validate_regional_encounters(base, &value)?;
        let mut encounters = original.clone();
        encounters.map_name = GALLERY.into();
        pack.data.wild_encounters.insert(GALLERY.into(), encounters);
    }
    for (name, original) in &base.data.maps {
        if !matches!(name.as_str(), HOLLOW | "MnLeechLake" | "MnBemidji") {
            ensure!(
                pack.data.maps.get(name) == Some(original),
                "unrelated field-use map {name}"
            );
        }
    }
    for (path, bytes) in &base.runtime_files {
        ensure!(
            pack.runtime_files.get(path) == Some(bytes),
            "original field-use asset {path}"
        );
    }
    ensure!(
        pack.data.pokemon == base.data.pokemon
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio
            && pack.audio_compression == base.audio_compression,
        "field uses must preserve creatures and audio"
    );
    pack.report.manifests.push(MARKER.into());
    pack.report.maps = pack.data.maps.len();
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn verify_geometry(
    pack: &CompiledGamePack,
    hollow: &MapModule,
    gallery: &MapModule,
    lake: &MapModule,
) -> Result<()> {
    use crate::authored_town_geometry::{kind, reachable};
    let entry = reachable(pack, hollow, [9, 10], false)?;
    ensure!(
        entry.contains(&[5, 6]) && entry.contains(&[10, 9]) && entry.contains(&[11, 9]),
        "safe hollow stair and passive return"
    );
    let gallery_walk = reachable(pack, gallery, [8, 13], false)?;
    ensure!(
        [[5, 6], [8, 5], [9, 13]]
            .iter()
            .all(|p| gallery_walk.contains(p)),
        "gallery read, cache approach and exit must connect"
    );
    for y in 0..16 {
        for x in 0..16 {
            if x == 0 || x == 15 || y == 0 || y == 15 {
                ensure!(
                    kind(pack, gallery, [x, y])? == "WALL",
                    "closed optional gallery border"
                );
            }
        }
    }
    ensure!(
        kind(pack, gallery, [6, 6])? == "WALL"
            && kind(pack, gallery, [8, 4])? == "FLOOR"
            && kind(pack, lake, [59, 20])? == "WATER"
            && kind(pack, lake, [60, 20])? == "WHIRLPOOL"
            && kind(pack, lake, [61, 20])? == "BUOY",
        "native field interaction anchors"
    );
    for point in [[61, 19], [61, 21], [62, 20]] {
        ensure!(
            matches!(kind(pack, lake, point)?, "WALL" | "BUOY"),
            "cove cache has a free bypass at {point:?}"
        );
    }
    let closed = reachable(pack, lake, [59, 20], true)?;
    ensure!(
        !closed.contains(&[60, 20]),
        "Whirlpool current must block its cache approach"
    );
    let mut opened = lake.clone();
    opened.blocks[10 * 32 + 30] = art::OPEN;
    let open = reachable(pack, &opened, [59, 20], true)?;
    ensure!(
        open.contains(&[60, 20]),
        "cleared native current must reach the cache approach"
    );
    let collision = &pack.data.tilesets[art::TILESET].collision;
    ensure!(
        collision[&format!("{:02x}", art::CLOSED)][1..]
            == collision[&format!("{:02x}", art::OPEN)][1..],
        "Whirlpool may only change its current quadrant"
    );
    Ok(())
}

pub(crate) fn cache(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    id: &str,
    at: [u16; 2],
    item: &str,
    flag: &str,
    visibility: &str,
) -> Result<()> {
    ensure!(pack.data.items.contains_key(item), "native cache item");
    a::actor(pack, map, id, "SPRITE_POKE_BALL", at)?;
    let object = map.objects.last_mut().context("native cache object")?;
    // Native disappear/appear updates an object's event flag. Keep temporary
    // clue visibility separate from the successful-delivery claim, so hiding
    // the ball never consumes it and a full pocket leaves both retry paths.
    object.object_type = "OBJECTTYPE_SCRIPT".into();
    object.event_flag = visibility.into();
    map.scripts.insert(
        id.into(),
        json!([
            c("checkevent", &[flag]),
            c("iftrue", &["MnLampCacheEnd"]),
            c("checkevent", &[READ]),
            c("iffalse", &["MnLampTooDark"]),
            c("opentext", &[]),
            c("verbosegiveitem", &[item, "1"]),
            c("iffalse", &["MnLampCacheClose"]),
            c("setevent", &[flag]),
            c("disappear", &[id]),
            c("sjump", &["MnLampCacheClose"])
        ]),
    );
    map.scripts.insert(
        "MnLampCacheClose".into(),
        json!([c("closetext", &[]), c("end", &[])]),
    );
    map.scripts
        .insert("MnLampCacheEnd".into(), json!([c("end", &[])]));
    Ok(())
}

pub(crate) fn hint(map: &mut MapModule, label: &str, lines: &[&str]) -> Result<()> {
    ensure!(
        lines
            .iter()
            .all(|line| !line.is_empty() && line.is_ascii() && line.len() <= 18),
        "native field hint lines"
    );
    let body = map
        .scripts
        .get_mut(label)
        .and_then(Value::as_array_mut)
        .context("existing field directions")?;
    ensure!(body.pop() == Some(c("done", &[])), "native hint terminator");
    for (index, line) in lines.iter().enumerate() {
        body.push(c(
            if index % 2 == 0 { "para" } else { "line" },
            &[&format!("\"{line}\"")],
        ));
    }
    body.push(c("done", &[]));
    Ok(())
}

fn validate_regional_encounters(pack: &CompiledGamePack, value: &Value) -> Result<()> {
    match value {
        Value::String(species) if pack.data.pokemon.contains_key(species) => ensure!(
            pack.data
                .story_event_script_constants
                .global
                .contains_key(&format!("MN_DEX_MEMBER_{species}")),
            "gallery encounter outside the active regional roster: {species}"
        ),
        Value::Array(values) => {
            for value in values {
                validate_regional_encounters(pack, value)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                validate_regional_encounters(pack, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
