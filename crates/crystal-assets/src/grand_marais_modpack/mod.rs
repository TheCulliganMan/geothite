//! Hand-authored Grand Marais playable slice. Source art stays in the supplied pack.
pub(crate) mod authoring;
mod design;
mod dialogue;
mod interiors;
mod residents;
mod story_dialogue;
pub use story_dialogue::refresh_grand_marais_story_dialogue;

use anyhow::{Context, Result, ensure};
use crystal_core::map::{BackgroundEvent, CoordEvent, MapScene};
use design::TownDesign;
use serde_json::json;

use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, RuntimeMapMetadata, RuntimeSpawnPoint,
    derive_compiled_game_pack_identity_from_manifest, verify_compiled_game_pack_for_runtime,
};
use authoring::{blank, blocks, command, finish, npc, scripts, speech, text, warp};

const TOWN: &str = "MN_GRAND_MARAIS";
const STATION: &str = "MN_GRAND_MARAIS_FIELD_STATION";
const CENTER: &str = "MN_GRAND_MARAIS_POKECENTER";
const MART: &str = "MN_GRAND_MARAIS_MART";
const ARTIST: &str = "MN_GRAND_MARAIS_ARTIST_HOUSE";
const FISHER: &str = "MN_GRAND_MARAIS_FISHER_HOUSE";
const LIGHTHOUSE: &str = "MN_GRAND_MARAIS_LIGHTHOUSE";
const SIGNAL: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL";
const STARTER: &str = "EVENT_MN_GRAND_MARAIS_STARTER";

/// Adds an isolated new-game Grand Marais town with working interiors and services.
///
/// The HOME spawn is redirected to its field station. Existing maps, artwork and
/// audio are retained. Returns an error if
/// templates are missing, the slice is already present, or pack validation fails.
pub fn build_grand_marais_modpack(
    base: &CompiledGamePack,
    town_layout: &str,
) -> Result<CompiledGamePack> {
    build_grand_marais(base, town_layout, true)
}

/// Adds the authored town and services while preserving HOME and recovery.
///
/// Use this extension when composing the town with other authored routes.
/// Regional gates and earned access remain separate implementation work.
///
/// # Errors
/// Returns an error for invalid authoring, missing templates, identifier
/// collisions, an existing town installation or failed pack validation.
pub fn build_grand_marais_region_extension(
    base: &CompiledGamePack,
    town_layout: &str,
) -> Result<CompiledGamePack> {
    build_grand_marais(base, town_layout, false)
}

fn build_grand_marais(
    base: &CompiledGamePack,
    town_layout: &str,
    new_game_in_station: bool,
) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.data.maps.contains_key("MnGrandMarais"),
        "Grand Marais is already installed"
    );
    let design = TownDesign::parse(town_layout)?;
    let mut modules = vec![
        town(base, &design)?,
        station(base)?,
        center(base)?,
        mart(base)?,
        lighthouse(base)?,
    ];
    for (name, constant, door, suffix, sprite) in [
        (
            "MnGrandMaraisArtistHouse",
            ARTIST,
            4,
            "Artist",
            "SPRITE_TEACHER",
        ),
        (
            "MnGrandMaraisFisherHouse",
            FISHER,
            5,
            "Fisher",
            "SPRITE_FISHER",
        ),
    ] {
        let mut m = blank(base, "GuideGentsHouse", name, constant)?;
        warp(&mut m, (2, 7), TOWN, door);
        warp(&mut m, (3, 7), TOWN, door);
        speech(base, &mut m, suffix, sprite, (5, 3), &[])?;
        modules.push(m);
    }
    interiors::apply(base, &design.interiors, &mut modules)?;
    dialogue::apply(&design.dialogue, &mut modules)?;
    let mut pack = base.clone();
    let mut home_spawn = None;
    let landmark_id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|entry| entry.id)
        .max()
        .context("source landmarks")?
        + 1;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(crate::PokegearLandmark {
            id: landmark_id,
            constant: "LANDMARK_MN_GRAND_MARAIS".into(),
            label: "MN_GRAND_MARAIS".into(),
            name: "GRAND MARAIS".into(),
            x: 140,
            y: 16,
            region: "JOHTO".into(),
        });
    for (index, mut module) in modules.into_iter().enumerate() {
        module.attributes.location = Some("LANDMARK_MN_GRAND_MARAIS".into());
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(module.id.clone(), "LANDMARK_MN_GRAND_MARAIS".into());
        finish(base, &mut module)?;
        let identifier = 100 + u16::try_from(index)?;
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("map constant")?;
        let name = module.id.clone();
        let start = name == "MnGrandMaraisFieldStation";
        let (x, y) = if start {
            (3, 5)
        } else if name == "MnGrandMarais" {
            (9, 10)
        } else {
            (3, 5)
        };
        let extension = CompiledMapExtension {
            manifest_id: format!("minnesota-grand-marais-{index}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                constant: constant.clone(),
                name: name.clone(),
                group_name: "MINNESOTA".into(),
                group_id: 40,
                map_id: index as u16 + 1,
                width: module.attributes.width,
                height: module.attributes.height,
                environment: module
                    .attributes
                    .environment
                    .clone()
                    .context("environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: identifier.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier,
                map_constant: constant,
                map_name: name,
                group_id: 40,
                map_id: index as i16 + 1,
                tile_x: x,
                tile_y: y,
                group_name: "MINNESOTA".into(),
                metatile_x: x / 2,
                metatile_y: y / 2,
                subtile_x: x % 2,
                subtile_y: y % 2,
            },
            module,
            wild_encounters: None,
            start_new_game_here: false,
        };
        crate::insert_compiled_map_extension(&mut pack, extension)?;
        let mut spawn = pack
            .data
            .runtime_spawn_points
            .remove(&identifier.to_string())
            .context("inserted spawn")?;
        if start {
            spawn.identifier = 0;
            home_spawn = Some(spawn);
        }
    }
    if new_game_in_station {
        // The isolated demo starts in town; regional composition keeps HOME.
        pack.data
            .runtime_spawn_points
            .insert("0".into(), home_spawn.context("town home spawn")?);
    }
    pack.report.maps = pack.data.maps.len();
    pack.report.reachable_maps.sort();
    pack.report.solvable_maps.sort();
    let next_flag = pack
        .data
        .story_event_script_constants
        .global
        .iter()
        .filter(|(key, _)| key.starts_with("EVENT_"))
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(0)
        + 1;
    pack.data
        .story_event_script_constants
        .global
        .insert(STARTER.into(), next_flag);
    pack.data
        .story_event_script_constants
        .global
        .insert(SIGNAL.into(), next_flag + 1);
    let mut encounters = base
        .data
        .wild_encounters
        .get("Route29")
        .context("grass encounter template")?
        .clone();
    encounters.map_name = "MnGrandMarais".into();
    crate::insert_wild_encounter_data(&mut pack.data.wild_encounters, encounters)?;
    pack.data.story_event_script_constants.maps.insert(
        "MnGrandMaraisFieldStation".into(),
        std::collections::BTreeMap::from([
            ("SCENE_MN_FIELD_STATION".into(), 0),
            ("SCENE_MN_FIELD_READY".into(), 1),
        ]),
    );
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn town(base: &CompiledGamePack, design: &TownDesign) -> Result<MapModule> {
    let mut m = blank(base, "CherrygroveCity", "MnGrandMarais", TOWN)?;
    blocks(&mut m, design.width, design.height, &design.rows.join(" "))?;
    ensure!(
        design.doors.len() == 6,
        "Grand Marais needs all six authored interiors"
    );
    for door in &design.doors {
        warp(&mut m, (door.tile[0], door.tile[1]), &door.target, 1);
    }
    for actor in &design.actors {
        residents::add(base, &mut m, actor)?;
    }
    for (index, sign) in design.signs.iter().enumerate() {
        ensure!(!sign.label.is_empty(), "sign requires a label");
        let script = format!("MnGrandMaraisSign{index}");
        let label = format!("{script}Text");
        m.scripts
            .insert(script.clone(), json!([command("jumptext", &[&label])]));
        text(
            &mut m,
            &label,
            &sign.lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        m.events.bg_events.push(BackgroundEvent {
            x: sign.tile[0],
            y: sign.tile[1],
            event_type: "BGEVENT_READ".into(),
            script,
        });
    }
    Ok(m)
}

fn station(base: &CompiledGamePack) -> Result<MapModule> {
    let mut m = blank(
        base,
        "PlayersNeighborsHouse",
        "MnGrandMaraisFieldStation",
        STATION,
    )?;
    warp(&mut m, (2, 7), TOWN, 1);
    warp(&mut m, (3, 7), TOWN, 1);
    let welcome = "MnGrandMaraisWelcome";
    let after = "MnGrandMaraisWelcomeAfter";
    m.scripts.insert(
        welcome.into(),
        json!([
            command("checkevent", &[STARTER]),
            command("iftrue", &[after]),
            command("opentext", &[]),
            command("writetext", &["MnGrandMaraisWelcomeText"]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("givepoke", &["TOTODILE", "8", "BERRY"]),
            command("giveitem", &["POKE_BALL", "5"]),
            command("giveitem", &["POTION", "3"]),
            command("giveitem", &["MACHINE_PART", "1"]),
            command("setevent", &[STARTER]),
            command("setscene", &["SCENE_MN_FIELD_READY"]),
            command("opentext", &[]),
            command("writetext", &["MnGrandMaraisPartnerText"]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[])
        ]),
    );
    m.scripts.insert(
        after.into(),
        json!([
            command("checkevent", &[SIGNAL]),
            command("iftrue", &["MnGrandMaraisTamarackThanks"]),
            command("jumptextfaceplayer", &["MnGrandMaraisReminderText"])
        ]),
    );
    m.scripts.insert(
        "MnGrandMaraisTamarackThanks".into(),
        json!([command(
            "jumptextfaceplayer",
            &["MnGrandMaraisTamarackThanksText"]
        )]),
    );
    npc(base, &mut m, welcome, "SPRITE_SCIENTIST", 5, 3)?;
    m.scripts.insert(
        "MnGrandMaraisStationNoop".into(),
        json!([command("end", &[])]),
    );
    m.scenes.scenes = vec![
        MapScene {
            scene_id: "SCENE_MN_FIELD_STATION".into(),
            script_name: Some("MnGrandMaraisStationNoop".into()),
        },
        MapScene {
            scene_id: "SCENE_MN_FIELD_READY".into(),
            script_name: Some("MnGrandMaraisStationNoop".into()),
        },
    ];
    m.map_script_section_commands = serde_json::from_value(json!([
        {"command":"def_scene_scripts","args":[],"command_index":0},
        {"command":"scene_script","args":["MnGrandMaraisStationNoop","SCENE_MN_FIELD_STATION"],"command_index":1},
        {"command":"scene_script","args":["MnGrandMaraisStationNoop","SCENE_MN_FIELD_READY"],"command_index":2},
        {"command":"def_callbacks","args":[],"command_index":3}
    ]))?;
    // Both exit columns intercept an unprepared player; the flag makes the gift idempotent.
    for x in [2, 3] {
        m.events.coord_events.push(CoordEvent {
            x,
            y: 6,
            scene_id: "SCENE_MN_FIELD_STATION".into(),
            script_name: welcome.into(),
        });
    }
    Ok(m)
}

fn center(base: &CompiledGamePack) -> Result<MapModule> {
    let mut m = blank(
        base,
        "MahoganyPokecenter1F",
        "MnGrandMaraisPokecenter",
        CENTER,
    )?;
    // Replace the upstairs stair block with the adjacent floor block. There is no orphaned 2F warp.
    m.blocks[15] = m.blocks[10];
    warp(&mut m, (3, 7), TOWN, 2);
    warp(&mut m, (4, 7), TOWN, 2);
    m.scripts.insert(
        "MnGrandMaraisNurse".into(),
        json!([command("jumpstd", &["PokecenterNurseScript"])]),
    );
    npc(base, &mut m, "MnGrandMaraisNurse", "SPRITE_NURSE", 3, 1)?;
    speech(base, &mut m, "Hiker", "SPRITE_POKEFAN_M", (7, 4), &[])?;
    Ok(m)
}

fn mart(base: &CompiledGamePack) -> Result<MapModule> {
    let mut m = blank(base, "VioletMart", "MnGrandMaraisMart", MART)?;
    warp(&mut m, (2, 7), TOWN, 3);
    warp(&mut m, (3, 7), TOWN, 3);
    scripts(
        &mut m,
        [(
            "MnGrandMaraisClerk".into(),
            json!([
                command("opentext", &[]),
                command("pokemart", &["MARTTYPE_STANDARD", "MART_VIOLET"]),
                command("closetext", &[]),
                command("end", &[])
            ]),
        )],
    );
    let mut clerk = base.data.maps["VioletMart"].objects[0].clone();
    clerk.script = "MnGrandMaraisClerk".into();
    clerk.object_identifier = Some("MNGRANDMARAISMART_CLERK".into());
    m.objects.push(clerk);
    Ok(m)
}

fn lighthouse(base: &CompiledGamePack) -> Result<MapModule> {
    let mut m = blank(
        base,
        "GuideGentsHouse",
        "MnGrandMaraisLighthouse",
        LIGHTHOUSE,
    )?;
    warp(&mut m, (2, 7), TOWN, 6);
    warp(&mut m, (3, 7), TOWN, 6);
    let script = "MnGrandMaraisKeeper";
    m.scripts.insert(
        script.into(),
        json!([
            command("checkevent", &[SIGNAL]),
            command("iftrue", &["MnGrandMaraisSignalDone"]),
            command("checkitem", &["MACHINE_PART"]),
            command("iffalse", &["MnGrandMaraisSignalMissing"]),
            command("giveitem", &["SUPER_POTION", "1"]),
            command("iffalse", &["MnGrandMaraisSignalBagFull"]),
            command("takeitem", &["MACHINE_PART"]),
            command("setevent", &[SIGNAL]),
            command("jumptextfaceplayer", &["MnGrandMaraisSignalFixedText"])
        ]),
    );
    m.scripts.insert(
        "MnGrandMaraisSignalDone".into(),
        json!([command(
            "jumptextfaceplayer",
            &["MnGrandMaraisSignalDoneText"]
        )]),
    );
    m.scripts.insert(
        "MnGrandMaraisSignalBagFull".into(),
        json!([command(
            "jumptextfaceplayer",
            &["MnGrandMaraisSignalBagFullText"]
        )]),
    );
    m.scripts.insert(
        "MnGrandMaraisSignalMissing".into(),
        json!([command(
            "jumptextfaceplayer",
            &["MnGrandMaraisSignalMissingText"]
        )]),
    );
    npc(base, &mut m, script, "SPRITE_SAILOR", 5, 3)?;
    Ok(m)
}
