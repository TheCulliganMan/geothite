//! Native Duluth exterior, ordinary rooms and canonical Gym8 city rebinding.
#[path = "duluth_city_design.rs"]
mod design;
#[path = "duluth_city_events.rs"]
mod events;
#[path = "duluth_city_geometry.rs"]
mod geometry;
#[path = "duluth_city_gym.rs"]
mod gym;
#[path = "duluth_city_publication.rs"]
mod publication;

use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Adds the complete authored Duluth exterior and six ordinary interiors.
///
/// The supplied pack must already contain the exact canonical native Gym8.
/// Its existing module, scripts, puzzle, parties, reward and art are retained;
/// only warp destinations 1/2/3 are rebound to Duluth's passive exterior warp4.
/// The canonical JSON is used to verify the single referenced Gym8 object;
/// unrelated gym changes and collection-file hashes are outside this contract.
///
/// Native HealParty, the ordinary PC collision/PCScript and a dedicated standard
/// mart serve the clinic/shop. The real Q07 receiver checks all eight Johto
/// badges and the Grand Rapids depot event before setting its one signal flag.
/// No prerequisite is awarded during installation. HOME/recovery, source art,
/// audio and unrelated maps/catalog entries remain unchanged. Group68 is reserved
/// for these seven maps. New clinic blackout registration and reciprocal R10/R11
/// travel/checkpoints remain explicit caller-owned integration work.
///
/// # Errors
/// Returns an error for invalid/stale authoring, an occupied identity/group,
/// missing or mismatching canonical Gym8/art/party, unsafe native geometry,
/// missing service inputs, malformed scripts or failed runtime verification.
/// Borrowed inputs remain unchanged on success and failure.
pub fn build_duluth_city_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    canonical_gyms_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let (town, blocks) = design::parse(town_json, layout_json)?;
    publication::available(base)?;
    let canonical = town
        .interiors
        .iter()
        .find(|r| r.id == "MnDuluthGym")
        .context("canonical Gym8 source reference")?;
    gym::validate(base, canonical, canonical_gyms_json)?;
    let mut pack = base.clone();
    events::prepare_catalogs(&mut pack, &town.interiors)?;
    let mut exterior = authoring::blank(&pack, "OlivineCity", &town.id, &town.native_constant)?;
    exterior.attributes.width = 32;
    exterior.attributes.height = 24;
    // mn_duluth is the editor's logical alias; every pixel/collision already
    // exists in the original johto_modern. Do not duplicate an art catalog.
    exterior.attributes.tileset_name = "johto_modern".into();
    exterior.attributes.border_block = 5;
    exterior.blocks = blocks;
    for door in &town.doors {
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("authored exterior door target")?;
        ensure!(
            door.warp_index == u16::try_from(exterior.events.warps.len() + 1)?
                && door.return_warp_index == Some(door.warp_index + 1),
            "Duluth active/passive exterior warp order changed"
        );
        authoring::warp(
            &mut exterior,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            door.target_warp,
        );
        authoring::warp(
            &mut exterior,
            (door.return_tile[0], door.return_tile[1]),
            &town.native_constant,
            i16::try_from(door.return_warp_index.context("Duluth passive return")?)?,
        );
    }
    events::install(&pack, &mut exterior, &town.residents, &town.signs)?;
    events::finish(&pack, &mut exterior)?;
    let mut modules = vec![(exterior, [6, 24])];
    for room in town.interiors.iter().filter(|r| r.id != "MnDuluthGym") {
        ensure!(
            room.source_reference.is_none(),
            "unexpected canonical reference in ordinary room"
        );
        let (template, border) = match room.tileset.as_str() {
            "pokecenter" => ("GoldenrodPokecenter1F", 0),
            "mart" => ("OlivineMart", 0),
            "house" => ("OlivineGoodRodHouse", 0x0a),
            "traditional_house" => ("VioletNicknameSpeechHouse", 0),
            _ => anyhow::bail!("unsupported Duluth room art"),
        };
        let mut module = authoring::blank(&pack, template, &room.id, &room.native_constant)?;
        ensure!(
            room.rows.len() == usize::from(room.dimensions_metatiles[1])
                && room
                    .rows
                    .iter()
                    .all(|r| r.split_whitespace().count()
                        == usize::from(room.dimensions_metatiles[0])),
            "Duluth room rows/dimensions differ"
        );
        authoring::blocks(
            &mut module,
            room.dimensions_metatiles[0],
            room.dimensions_metatiles[1],
            &room.rows.join(" "),
        )?;
        module.attributes.tileset_name = room.tileset.clone();
        module.attributes.border_block = border;
        let door = town
            .doors
            .iter()
            .find(|d| d.target == room.id)
            .context("ordinary room exterior door")?;
        ensure!(
            room.exits.len() == 2 && room.passive_arrival.index == 3,
            "ordinary room requires carpets1/2 and passive3"
        );
        for (index, exit) in room.exits.iter().enumerate() {
            ensure!(
                exit.index == u16::try_from(index + 1)?
                    && exit.target == town.id
                    && Some(u16::try_from(exit.target_warp)?) == door.return_warp_index,
                "ordinary room carpet must return to its own exterior FLOOR"
            );
            authoring::warp(
                &mut module,
                (exit.tile[0], exit.tile[1]),
                &town.native_constant,
                exit.target_warp,
            );
        }
        authoring::warp(
            &mut module,
            (room.passive_arrival.tile[0], room.passive_arrival.tile[1]),
            &town.native_constant,
            i16::try_from(
                door.return_warp_index
                    .context("ordinary room passive exterior")?,
            )?,
        );
        events::install(&pack, &mut module, &room.residents, &room.signs)?;
        events::finish(&pack, &mut module)?;
        modules.push((module, room.passive_arrival.tile));
    }
    let geometry_modules = modules
        .iter()
        .map(|(m, _)| m.clone())
        .collect::<Vec<MapModule>>();
    geometry::validate(&pack, &town, &geometry_modules)?;
    gym::rebind(&mut pack)?;
    publication::publish(pack, modules)
}
