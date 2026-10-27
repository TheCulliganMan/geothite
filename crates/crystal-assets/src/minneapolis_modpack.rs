//! Authored Minneapolis and the native ICE dispatch-office chapter.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::{Value, json};

use crate::authored_route_layout::Layout;
use crate::authored_town_design::{Actor, Town};
use crate::grand_marais_modpack::authoring;
use crate::minneapolis_gym as gym_content;
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

#[path = "minneapolis_ice.rs"]
mod ice;

const TOWN: &str = "MnMinneapolis";
const CONSTANT: &str = "MN_MINNEAPOLIS";
const GYM: &str = "MnMinneapolisGym";
const CLINIC: &str = "MnMinneapolisPokecenter1F";
const OFFICE: &str = "MnMinneapolisDispatchOffice";
const GROUP: &str = "MINNESOTA_MINNEAPOLIS";
const LANDMARK: &str = "LANDMARK_MN_MINNEAPOLIS";
const MANIFEST: &str = "minnesota-minneapolis-v1";
const MART: &str = "MART_MN_MINNEAPOLIS";

/// Publishes the explicitly authored river city and the ICE battle/rescue/note.
///
/// The installed Gym3 retains its logic/art; returns and geographic landmark
/// are rebound to the city.
/// New ordinary healing/PC/shop services use production routines. ICE victory,
/// captive release and reading the dispatch copy are separate persistent steps;
/// none grants a badge or any later Northstar milestone. Original content, audio,
/// HOME and recovery destinations are preserved; town/clinic recovery is additive.
/// Regional route reservations remain unbound until separately verified.
///
/// # Errors
/// Returns an error for malformed/stale sources, occupied identities, changed
/// gym epoch, unsafe geometry, incompatible story contracts, missing native
/// services/assets or failed parsing/runtime validation. Inputs are unchanged.
pub fn build_minneapolis_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    story_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town: Town = serde_json::from_str(town_json).context("Minneapolis town")?;
    let raw: Value = serde_json::from_str(town_json)?;
    let layout: Layout = serde_json::from_str(layout_json).context("Minneapolis layout")?;
    ensure!(
        town.id == TOWN
            && town.native_constant == CONSTANT
            && town.dimensions_metatiles == [32, 24]
            && town.compiled_tileset == "johto_modern"
            && layout.id == TOWN
            && layout.compiled_tileset == "johto_modern"
            && layout.dimensions_metatiles == [32, 24],
        "retain complete native Minneapolis identity"
    );
    ensure!(
        !base.data.maps.contains_key(TOWN)
            && !base
                .data
                .runtime_map_metadata
                .values()
                .any(|m| m.group_id == 72)
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Minneapolis already installed or group72 occupied"
    );
    let expected = BTreeSet::from([
        CLINIC,
        GYM,
        "MnMinneapolisMart",
        OFFICE,
        "MnMinneapolisDispatcherHouse",
    ]);
    ensure!(
        town.interiors
            .iter()
            .map(|r| r.id.as_str())
            .collect::<BTreeSet<_>>()
            == expected
            && town.interiors.len() == 5
            && town.doors.len() == 5,
        "Minneapolis requires its five real room pairings"
    );
    let native = base.data.maps.get(GYM).context("installed native Gym3")?;
    let reference = raw["interiors"]
        .as_array()
        .context("rooms")?
        .iter()
        .find(|r| r["id"] == GYM)
        .context("Gym3 reference")?;
    let hash = gym_content::content_hash(native)?;
    ensure!(
        reference["source_reference"]["native_content_sha256"] == hash
            && native.attributes.width == 5
            && native.attributes.height == 9
            && native.events.warps.len() == 3,
        "installed Gym3 content differs from the canonical switch-pair room: {hash}"
    );
    let mut pack = base.clone();
    for service in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(service)?;
    }
    ensure!(
        !pack.data.marts.0.contains_key(MART),
        "Minneapolis shop occupied"
    );
    let clerk = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .find(|a| a.service.as_deref() == Some("pokemart"))
        .context("native clerk")?;
    ensure!(
        clerk.stock
            == [
                "GREAT_BALL",
                "SUPER_POTION",
                "ANTIDOTE",
                "PARLYZ_HEAL",
                "ESCAPE_ROPE",
                "REPEL"
            ]
            && clerk.stock.iter().all(|s| pack.data.items.contains_key(s)),
        "Minneapolis early-game shop stock"
    );
    pack.data.marts.0.insert(MART.into(), clerk.stock.clone());
    ice::prepare(&mut pack, story_json)?;
    let mut exterior = authoring::blank(&pack, "GoldenrodCity", TOWN, CONSTANT)?;
    exterior.attributes.width = 32;
    exterior.attributes.height = 24;
    exterior.attributes.border_block = 5;
    exterior.attributes.tileset_name = "johto_modern".into();
    exterior.blocks = layout.compile()?;
    ordinary(&pack, &mut exterior, &town.residents, &town.signs)?;
    let mut modules = vec![(exterior, [32, 40])];
    let mut gym = native.clone();
    for room in &town.interiors {
        let door = town
            .doors
            .iter()
            .find(|d| d.target == room.id)
            .context("paired city door")?;
        ensure!(
            door.target_warp == 3
                && door.return_warp_index == Some(door.warp_index + 1)
                && room.passive_arrival.index == 3
                && room.exits.len() == 2,
            "native passive room pairing"
        );
        let mut module = if room.id == GYM {
            ensure!(
                room.rows.is_empty()
                    && room.residents.is_empty()
                    && room.signs.is_empty()
                    && room.dimensions_metatiles
                        == [native.attributes.width, native.attributes.height]
                    && room
                        .exits
                        .iter()
                        .zip(&native.events.warps[..2])
                        .all(|(e, w)| e.index == w.index && e.tile == [w.x, w.y])
                    && room.passive_arrival.tile
                        == [native.events.warps[2].x, native.events.warps[2].y],
                "city cannot change the installed gym's carpets, arrival, actors or terrain"
            );
            gym.clone()
        } else {
            ensure!(
                !pack.data.maps.contains_key(&room.id),
                "new Minneapolis room already exists"
            );
            let mut m = authoring::blank(&pack, &room.template, &room.id, &room.native_constant)?;
            authoring::blocks(
                &mut m,
                room.dimensions_metatiles[0],
                room.dimensions_metatiles[1],
                &room.rows.join(" "),
            )?;
            m.attributes.tileset_name = room.tileset.clone();
            m.attributes.border_block = room.border_block;
            ordinary(&pack, &mut m, &room.residents, &room.signs)?;
            m
        };
        module.events.warps.clear();
        for (index, exit) in room.exits.iter().enumerate() {
            ensure!(
                exit.index == u16::try_from(index + 1)?
                    && exit.target == TOWN
                    && Some(u16::try_from(exit.target_warp)?) == door.return_warp_index,
                "room returns to its own city facade"
            );
            authoring::warp(
                &mut module,
                (exit.tile[0], exit.tile[1]),
                CONSTANT,
                exit.target_warp,
            );
        }
        authoring::warp(
            &mut module,
            (room.passive_arrival.tile[0], room.passive_arrival.tile[1]),
            CONSTANT,
            i16::try_from(door.return_warp_index.context("return record")?)?,
        );
        if let Some(pc) = &room.pc {
            let label = "MnMPClinicPC";
            module.scripts.insert(
                label.into(),
                json!([
                    authoring::command("opentext", &[]),
                    authoring::command("special", &["PokemonCenterPC"]),
                    authoring::command("closetext", &[]),
                    authoring::command("end", &[])
                ]),
            );
            module.events.bg_events.push(BackgroundEvent {
                x: pc.tile[0],
                y: pc.tile[1],
                event_type: "BGEVENT_READ".into(),
                script: label.into(),
            });
        }
        if room.id == GYM {
            authoring::finish(&pack, &mut module)?;
            gym = module;
        } else {
            finish(&pack, &mut module)?;
            modules.push((module, room.passive_arrival.tile));
        }
    }
    for door in &town.doors {
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("door room")?;
        let outside = &mut modules[0].0;
        ensure!(
            usize::from(door.warp_index) == outside.events.warps.len() + 1,
            "city exterior warp order"
        );
        authoring::warp(
            outside,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            3,
        );
        authoring::warp(
            outside,
            (door.return_tile[0], door.return_tile[1]),
            CONSTANT,
            i16::try_from(door.return_warp_index.context("safe exterior return")?)?,
        );
    }
    for (module, _) in &mut modules {
        finish(&pack, module)?;
    }
    crate::authored_town_geometry::validate(&pack, &town, &modules)?;
    let office = modules
        .iter_mut()
        .find(|(m, _)| m.id == OFFICE)
        .context("actual office")?;
    ice::install(&pack, &mut office.0, &raw, story_json)?;
    finish(&pack, &mut office.0)?;
    let home = modules
        .iter_mut()
        .find(|(m, _)| m.id == "MnMinneapolisDispatcherHouse")
        .context("dispatcher home")?;
    ice::install_home(&mut home.0, &raw)?;
    finish(&pack, &mut home.0)?;
    gym.attributes.location = Some(LANDMARK.into());
    pack.data
        .map_attributes
        .insert(GYM.into(), gym.attributes.clone());
    pack.data
        .pokegear_landmarks
        .map_to_landmark
        .insert(GYM.into(), LANDMARK.into());
    mirror(&mut pack, &gym)?;
    pack.data.maps.insert(GYM.into(), gym);
    publish(pack, modules)
}

fn ordinary(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    actors: &[Actor],
    signs: &[crate::authored_route_design::Sign],
) -> Result<()> {
    let c = authoring::command;
    for actor in actors {
        let label = format!("{}Text", actor.id);
        // The native mart owns its greeting; an unused actor text body would
        // survive only in the cached module, not an uncached split reload.
        if actor.service.as_deref() != Some("pokemart") {
            authoring::text(
                map,
                &label,
                &actor.lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
        }
        let commands = match actor.service.as_deref() {
            None => json!([c("jumptextfaceplayer", &[&label])]),
            Some("heal_party") => json!([
                c("faceplayer", &[]),
                c("opentext", &[]),
                c("writetext", &[&label]),
                c("waitbutton", &[]),
                c("special", &["HealParty"]),
                c("blackoutmod", &["MN_MINNEAPOLIS_POKECENTER_1F"]),
                c("closetext", &[]),
                c("end", &[])
            ]),
            Some("pokemart") => json!([
                c("faceplayer", &[]),
                c("opentext", &[]),
                c("pokemart", &["MARTTYPE_STANDARD", MART]),
                c("closetext", &[]),
                c("end", &[])
            ]),
            Some("ice_family") => {
                let after = format!("{}After", actor.id);
                authoring::text(
                    map,
                    &format!("{after}Text"),
                    &actor
                        .repeat_lines
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                );
                map.scripts.insert(
                    after.clone(),
                    json!([c("jumptextfaceplayer", &[&format!("{after}Text")])]),
                );
                json!([
                    c("checkevent", &["EVENT_MN_ICE_CAPTIVE_RESCUED"]),
                    c("iftrue", &[&after]),
                    c("jumptextfaceplayer", &[&label])
                ])
            }
            Some(service) => anyhow::bail!("unsupported Minneapolis ordinary service {service}"),
        };
        ensure!(
            !map.scripts.contains_key(&actor.id),
            "actor script collision"
        );
        map.scripts.insert(actor.id.clone(), commands);
        authoring::npc(
            pack,
            map,
            &actor.id,
            &actor.sprite,
            i16::try_from(actor.tile[0])?,
            i16::try_from(actor.tile[1])?,
        )?;
        let object = map.objects.last_mut().context("new native actor")?;
        object.object_identifier = Some(actor.id.clone());
        object.spritemovedata = actor.facing.movement().into();
    }
    for sign in signs {
        let label = format!("{}Text", sign.id);
        authoring::text(
            map,
            &label,
            &sign.lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        map.scripts
            .insert(sign.id.clone(), json!([c("jumptext", &[&label])]));
        map.events.bg_events.push(BackgroundEvent {
            x: sign.tile[0],
            y: sign.tile[1],
            event_type: "BGEVENT_READ".into(),
            script: sign.id.clone(),
        });
    }
    Ok(())
}

fn finish(pack: &CompiledGamePack, map: &mut MapModule) -> Result<()> {
    authoring::finish(pack, map)?;
    let name = &map.id;
    let scripts = &map.scripts;
    map.script_runtime_commands = crate::parse_script_runtime_commands(name, scripts)?;
    map.script_object_commands = crate::parse_script_object_commands(name, scripts)?;
    map.script_variable_commands = crate::parse_script_variable_commands(name, scripts)?;
    map.script_movements =
        crate::parse_script_movements(name, scripts, &map.script_object_commands)?;
    map.script_block_changes = crate::parse_script_block_changes(name, scripts)?;
    map.trainer_scripts = crate::parse_trainer_scripts(name, scripts)?;
    map.scripted_trainer_battles = crate::parse_scripted_trainer_battles(name, scripts)?;
    let mut names = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    names.insert(
        map.attributes
            .map_constant
            .clone()
            .context("native map identity")?,
        name.clone(),
    );
    names.insert(CONSTANT.into(), TOWN.into());
    map.script_map_commands = crate::parse_script_map_commands(name, scripts, &names)?;

    Ok(())
}
fn mirror(pack: &mut CompiledGamePack, map: &MapModule) -> Result<()> {
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &map.id,
        map.attributes
            .map_scripts_label
            .as_deref()
            .context("city script label")?,
        map.attributes
            .map_events_label
            .as_deref()
            .context("city event label")?,
        map,
    )
}
fn publish(
    mut pack: CompiledGamePack,
    modules: Vec<(MapModule, [u16; 2])>,
) -> Result<CompiledGamePack> {
    ensure!(
        !pack
            .data
            .story_event_script_constants
            .global
            .iter()
            .any(|(k, v)| k.starts_with("MAPGROUP_") && *v == 72),
        "group72 numeric identity occupied"
    );
    pack.data
        .story_event_script_constants
        .global
        .insert(format!("MAPGROUP_{GROUP}"), 72);
    let id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("landmarks")?
        .checked_add(1)
        .context("landmark overflow")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id,
            constant: LANDMARK.into(),
            label: CONSTANT.into(),
            name: "MINNEAPOLIS".into(),
            x: 88,
            y: 112,
            region: "JOHTO".into(),
        });
    let first = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("spawns")?
        .checked_add(1)
        .context("spawn overflow")?
        .max(crystal_core::systems::special_routines::FIRST_ADDON_RECOVERY_SPAWN);
    for (index, (mut module, [x, y])) in modules.into_iter().enumerate() {
        module.attributes.location = Some(LANDMARK.into());
        module.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("map constant")?;
        let map_id = u16::try_from(index + 1)?;
        let identifier = first
            .checked_add(u16::try_from(index)?)
            .context("spawn identity overflow")?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("{MANIFEST}-{map_id}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                constant: constant.clone(),
                name: name.clone(),
                group_name: GROUP.into(),
                group_id: 72,
                map_id,
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
                map_name: name.clone(),
                group_name: GROUP.into(),
                group_id: 72,
                map_id: i16::try_from(map_id)?,
                tile_x: i16::try_from(x)?,
                tile_y: i16::try_from(y)?,
                metatile_x: i16::try_from(x / 2)?,
                metatile_y: i16::try_from(y / 2)?,
                subtile_x: i16::try_from(x % 2)?,
                subtile_y: i16::try_from(y % 2)?,
            },
            module,
            wild_encounters: None,
            start_new_game_here: false,
        };
        crate::validate_compiled_map_extension(&extension)?;
        crate::insert_compiled_map_extension(&mut pack, extension)?;
        if name != TOWN && name != CLINIC {
            pack.data
                .runtime_spawn_points
                .remove(&identifier.to_string());
        }
    }
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
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
