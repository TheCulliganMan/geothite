//! Native free Boundary Waters basecamp and its actual authored survey shelter.
mod geometry;
mod publication;
mod story_contract;
#[cfg(test)]
mod tests;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_route_design::{Door, Interior, Resident, ResidentService, Sign};
use crate::authored_route_layout::Layout;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Reserved native group for the free town and its survey shelter.
pub const BOUNDARY_BASECAMP_GROUP: u16 = 62;
/// Additive publication identity; expedition admission remains closed.
pub const BOUNDARY_BASECAMP_MANIFEST: &str = "minnesota-boundary-basecamp-v1";
/// Real town identity, separate from the optional capture preserve.
pub const BOUNDARY_BASECAMP_MAP: &str = "MnBoundaryWaters";
/// Native constant used in warp records and metadata lookup.
pub const BOUNDARY_BASECAMP_CONSTANT: &str = "MN_BOUNDARY_WATERS";
const GROUP: &str = "MINNESOTA_BOUNDARY_BASECAMP";
const LANDMARK: &str = "LANDMARK_MN_BOUNDARY_BASECAMP";

#[derive(Deserialize)]
struct Town {
    id: String,
    native_constant: String,
    compiled_tileset: String,
    dimensions_metatiles: [u16; 2],
    ports: Vec<Port>,
    residents: Vec<Resident>,
    signs: Vec<Sign>,
    doors: Vec<Door>,
    interiors: Vec<Interior>,
}
#[derive(Deserialize)]
struct Port {
    route: String,
    side: String,
    gate_metatile: [u16; 2],
    landing_metatile: [u16; 2],
}

/// Publishes the supplied hand-authored free town and actual survey shelter.
///
/// The 32×24 town uses the existing `mn_boundary_safari` native art; every block
/// is compiled from the supplied layout. The 8×6 lab room replaces all template
/// geometry with the supplied rows, has two real carpet exits and a passive
/// FLOOR arrival 3. Its actual medic receives native `HealParty`; Q07 chief and
/// receiver use the focused native story binder, independent of paid admission.
///
/// The three source route ports remain honest unbound walking reservations.
/// This installer adds no regional connections, port teleports, Safari fee,
/// permit, encounter table, capture mode, session renewal, SQ03 reward, HOME or
/// recovery spawn. A separate integration step must bind and prove the actual
/// reciprocal regional connections. Existing maps, files, art and audio remain
/// unchanged; no current story prerequisite is granted to a saved player.
///
/// # Errors
/// Returns an error for an occupied identity/group, malformed source/layout,
/// missing existing native art, unsafe doors/arrival/actors/ports, inaccessible
/// free services, admission bypass, open outer water, conflicting Q07 metadata,
/// native parser failures or failed final pack verification. Both input strings
/// and the base pack remain unchanged on success and failure.
pub fn build_boundary_basecamp_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town: Town = serde_json::from_str(town_json).context("Boundary basecamp source")?;
    let contract: Value = serde_json::from_str(town_json)?;
    let layout = Layout::parse(layout_json).context("Boundary basecamp native layout")?;
    ensure!(
        town.id == BOUNDARY_BASECAMP_MAP
            && town.native_constant == BOUNDARY_BASECAMP_CONSTANT
            && town.dimensions_metatiles == [32, 24]
            && town.compiled_tileset == "mn_boundary_safari"
            && layout.id == town.id
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && layout.compiled_tileset == town.compiled_tileset,
        "retain actual 32×24 Boundary basecamp source identity and existing native art"
    );
    ensure!(
        town.doors.len() == 1 && town.interiors.len() == 1,
        "basecamp needs one actual shelter and one honest native facade door"
    );
    let room = &town.interiors[0];
    let room_constant = contract["interiors"][0]["native_constant"]
        .as_str()
        .context("actual shelter native constant")?;
    ensure!(
        room.id == crate::boundary_basecamp_story::BOUNDARY_SURVEY_SHELTER
            && room_constant == crate::boundary_basecamp_story::BOUNDARY_SURVEY_SHELTER_CONSTANT
            && room.template == "ElmsLab"
            && room.dimensions_metatiles == [8, 6]
            && contract["interiors"][0]["tileset"] == "lab",
        "retain actual 8×6 lab survey shelter"
    );
    publication::validate_available(base, &town, room_constant)?;
    ensure!(
        base.data().tilesets.contains_key(&town.compiled_tileset),
        "basecamp requires the already compiled native mn_boundary_safari source art"
    );
    let mut pack = base.clone();
    pack.data.materialize_global_scripts()?;
    publication::refresh_identity(&mut pack)?;
    let mut exterior = authoring::blank(&pack, "CherrygroveCity", &town.id, &town.native_constant)?;
    exterior.attributes.width = town.dimensions_metatiles[0];
    exterior.attributes.height = town.dimensions_metatiles[1];
    exterior.attributes.tileset_name = town.compiled_tileset.clone();
    exterior.attributes.border_block = 5;
    exterior.attributes.music = pack
        .data()
        .maps
        .get("LakeOfRage")
        .context("native LakeOfRage music")?
        .attributes
        .music
        .clone();
    exterior.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
    exterior.attributes.location = Some(LANDMARK.into());
    exterior.blocks = layout.compile()?;
    author_people(&pack, &mut exterior, &town.residents, &town.signs)?;
    let mut shelter = authoring::blank(&pack, &room.template, &room.id, room_constant)?;
    authoring::blocks(
        &mut shelter,
        room.dimensions_metatiles[0],
        room.dimensions_metatiles[1],
        &room.rows.join(" "),
    )?;
    ensure!(shelter.attributes.tileset_name == "lab", "native lab art");
    shelter.attributes.border_block = room.border_block.context("shelter border block")?;
    shelter.attributes.map_group_constant = exterior.attributes.map_group_constant.clone();
    shelter.attributes.location = exterior.attributes.location.clone();
    author_people(&pack, &mut shelter, &room.residents, &room.signs)?;
    geometry::validate(&pack, &town, &contract, &exterior, &shelter)?;
    let door = &town.doors[0];
    let passive = room
        .passive_arrival
        .as_ref()
        .context("shelter passive arrival")?;
    for at in [door.tile, door.return_tile] {
        authoring::warp(
            &mut exterior,
            (at[0], at[1]),
            room_constant,
            door.target_warp,
        );
    }
    for at in room.exits.iter().copied().chain([passive.tile]) {
        authoring::warp(
            &mut shelter,
            (at[0], at[1]),
            &town.native_constant,
            i16::try_from(door.return_warp_index.context("exterior passive return")?)?,
        );
    }
    finish(&pack, &mut exterior)?;
    finish(&pack, &mut shelter)?;
    let prepared = crate::boundary_basecamp_story::prepare_boundary_basecamp_story(
        &pack,
        &shelter,
        crate::boundary_basecamp_story::BoundaryBasecampStoryAnchors {
            chief_tile: [9, 5],
            chief_approaches: [[8, 5], [10, 5], [9, 6]],
            receiver_tile: [12, 3],
            receiver_approach: [12, 4],
            clear_arrival: passive.tile,
        },
    )?;
    story_contract::validate(&contract, &prepared.pack, &prepared.shelter)?;
    publication::publish(
        prepared.pack,
        [exterior, prepared.shelter],
        [door.return_tile, passive.tile],
    )
}

fn author_people(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    residents: &[Resident],
    signs: &[Sign],
) -> Result<()> {
    for resident in residents {
        ensure!(
            !module
                .objects
                .iter()
                .any(|object| [object.x, object.y] == resident.tile),
            "overlapping authored resident {}",
            resident.id
        );
        ensure!(
            !module.scripts.contains_key(&resident.id),
            "duplicate authored resident {}",
            resident.id
        );
        ensure!(
            pack.runtime_files().contains_key(&format!(
                "gfx/sprites/{}.png",
                resident
                    .sprite
                    .trim_start_matches("SPRITE_")
                    .to_ascii_lowercase()
            )),
            "missing native sprite {}",
            resident.sprite
        );
        let label = format!("{}Text", resident.id);
        text(module, &label, &resident.lines)?;
        module.scripts.insert(
            resident.id.clone(),
            match resident.service {
                None => json!([authoring::command("jumptextfaceplayer", &[&label])]),
                Some(ResidentService::HealParty) => json!([
                    authoring::command("faceplayer", &[]),
                    authoring::command("opentext", &[]),
                    authoring::command("writetext", &[&label]),
                    authoring::command("waitbutton", &[]),
                    authoring::command("special", &["HealParty"]),
                    authoring::command("closetext", &[]),
                    authoring::command("end", &[])
                ]),
            },
        );
        authoring::npc(
            pack,
            module,
            &resident.id,
            &resident.sprite,
            i16::try_from(resident.tile[0])?,
            i16::try_from(resident.tile[1])?,
        )?;
        let native = module.objects.last_mut().context("authored resident")?;
        native.object_identifier = Some(resident.id.clone());
        native.spritemovedata = resident.facing.movement().into();
        native.object_type = "OBJECTTYPE_SCRIPT".into();
        native.radius = 0;
    }
    for sign in signs {
        ensure!(
            !module.scripts.contains_key(&sign.id),
            "duplicate source interaction"
        );
        let label = format!("{}Text", sign.id);
        text(module, &label, &sign.lines)?;
        module.scripts.insert(
            sign.id.clone(),
            json!([authoring::command("jumptext", &[&label])]),
        );
        module.events.bg_events.push(BackgroundEvent {
            x: sign.tile[0],
            y: sign.tile[1],
            event_type: "BGEVENT_READ".into(),
            script: sign.id.clone(),
        });
    }
    Ok(())
}
fn text(module: &mut MapModule, label: &str, lines: &[String]) -> Result<()> {
    ensure!(
        !lines.is_empty()
            && lines
                .iter()
                .all(|line| !line.is_empty() && line.chars().count() <= 18),
        "authored dialogue must fit 18 columns: {label}"
    );
    authoring::text(
        module,
        label,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    Ok(())
}
fn finish(pack: &CompiledGamePack, module: &mut MapModule) -> Result<()> {
    authoring::finish(pack, module)?;
    module.script_runtime_commands =
        crate::parse_script_runtime_commands(&module.id, &module.scripts)?;
    module.script_object_commands =
        crate::parse_script_object_commands(&module.id, &module.scripts)?;
    Ok(())
}
fn point(value: &Value) -> Result<[u16; 2]> {
    Ok([
        u16::try_from(value[0].as_u64().context("source x")?)?,
        u16::try_from(value[1].as_u64().context("source y")?)?,
    ])
}
