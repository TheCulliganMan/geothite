//! Add Bemidji's authored town around the already installed canonical Grass Gym.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::authored_route_design::Facing;
use crate::authored_route_layout::Layout;
use crate::grand_marais_modpack::authoring;
use crate::gym_design::{Gym, GymDesign};
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint, bemidji_city_art,
};

/// Reserved native group for Bemidji and its five non-gym rooms.
pub const BEMIDJI_CITY_GROUP: u16 = 70;
/// Native exterior map identity.
pub const BEMIDJI_CITY_MAP: &str = "MnBemidji";
const CONSTANT: &str = "MN_BEMIDJI";
const GROUP: &str = "MINNESOTA_BEMIDJI";
const LANDMARK: &str = "LANDMARK_MN_BEMIDJI";
const MANIFEST: &str = "minnesota-bemidji-city-v1";
const MART: &str = "MART_MN_BEMIDJI";
const GYM: &str = "MnBemidjiGym";

#[derive(Deserialize)]
struct Town {
    id: String,
    native_constant: String,
    compiled_tileset: String,
    dimensions_metatiles: [u16; 2],
    ports: Vec<Port>,
    residents: Vec<Resident>,
    signs: Vec<Interaction>,
    doors: Vec<Door>,
    interiors: Vec<Room>,
}
#[derive(Deserialize)]
struct Port {
    route: String,
    side: String,
    gate_metatile: [u16; 2],
    landing_metatile: [u16; 2],
}
#[derive(Deserialize)]
struct Door {
    tile: [u16; 2],
    return_tile: [u16; 2],
    arrival_tile: [u16; 2],
    target: String,
    warp_index: u16,
    return_warp_index: u16,
    target_warp: i16,
}
#[derive(Deserialize)]
struct Resident {
    id: String,
    sprite: String,
    tile: [u16; 2],
    facing: Facing,
    lines: Vec<String>,
    #[serde(default)]
    service: Option<Service>,
    #[serde(default)]
    stock: Vec<String>,
    #[serde(default)]
    counter_tile: Option<[u16; 2]>,
    #[serde(default)]
    approach: Option<[u16; 2]>,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Service {
    HealParty,
    CanonicalPokemartPending,
}
#[derive(Deserialize)]
struct Interaction {
    id: String,
    tile: [u16; 2],
    lines: Vec<String>,
}
#[derive(Deserialize)]
struct Arrival {
    index: u16,
    tile: [u16; 2],
}
#[derive(Deserialize)]
struct Exit {
    index: u16,
    tile: [u16; 2],
    target: String,
    target_warp: i16,
}
#[derive(Deserialize)]
struct Room {
    id: String,
    native_constant: String,
    tileset: String,
    dimensions_metatiles: [u16; 2],
    rows: Vec<String>,
    passive_arrival: Arrival,
    exits: Vec<Exit>,
    residents: Vec<Resident>,
    #[serde(default)]
    pc: Option<Pc>,
    #[serde(default)]
    exhibits: Vec<Interaction>,
    #[serde(default)]
    canonical_gym_sha256: Option<String>,
}
#[derive(Deserialize)]
struct Pc {
    tile: [u16; 2],
    approach: [u16; 2],
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Recipe {
    id: String,
    source_tileset: String,
    source_block: String,
    collision_source_tileset: String,
    collision_source_block: String,
}
#[derive(Deserialize)]
struct ArtSource {
    tileset: String,
    shore_fitting: String,
    source_pixels_modified: bool,
    derived_blocks: Vec<Recipe>,
}

/// Adds the hand-authored exterior, five rooms and production service scripts.
///
/// Canonical Gym7 must already be installed and match the supplied authoritative
/// gym JSON, including its complete module, source art, trainers and rewards.
/// Its three warp records are rebound to the city's passive exterior return;
/// their coordinates and all terrain, puzzle and battle scripts remain unchanged.
/// Original maps/assets/audio/HOME are preserved. Two new recovery
/// records support ordinary clinic entry and the authored healed indoor return.
///
/// Four regional ports remain unbound walking reservations. The existing-anchor
/// Q05 officer speaks the supplied ordinary text, but this builder deliberately
/// does not bind or replace CompareHeadwaterReport or grant its report flag.
/// Bespoke Paul/Babe statues are not fabricated. These missing story/connection
/// bindings and controller verification must be completed before playability.
///
/// # Errors
/// Returns an error for absent/stale canonical Gym7, changed source identities,
/// occupied map/group/art/mart identities, malformed strokes/rows, unsafe native
/// doors/services/returns, disconnected paths, open lake boundaries, invalid
/// native art or script commands, or failed runtime pack verification. The
/// borrowed input pack and JSON strings remain unchanged on success and failure.
pub fn build_bemidji_city_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    gyms_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town: Town = serde_json::from_str(town_json).context("Bemidji city source")?;
    let layout: Layout = serde_json::from_str(layout_json).context("Bemidji explicit strokes")?;
    let art: ArtSource = serde_json::from_str(layout_json)?;
    let gyms: GymDesign = serde_json::from_str(gyms_json).context("canonical gym source")?;
    gyms.validate()?;
    let gym = gyms
        .gyms
        .iter()
        .find(|gym| gym.id == GYM)
        .context("canonical Gym7 source")?;
    ensure!(
        town.id == BEMIDJI_CITY_MAP
            && town.native_constant == CONSTANT
            && town.dimensions_metatiles == [24, 24]
            && town.compiled_tileset == bemidji_city_art::TILESET
            && layout.id == town.id
            && layout.schema_version == 2
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && layout.compiled_tileset == town.compiled_tileset,
        "retain complete 24x24 Bemidji identity"
    );
    let expected_recipes = ["22", "23", "28", "29"]
        .into_iter()
        .enumerate()
        .map(|(i, b)| Recipe {
            id: format!("{:02x}", 0x80 + i),
            source_tileset: "johto".into(),
            source_block: b.into(),
            collision_source_tileset: "johto".into(),
            collision_source_block: b.into(),
        })
        .collect::<Vec<_>>();
    ensure!(
        art.tileset == "johto_modern"
            && art.shore_fitting == "johto_8_neighbors"
            && !art.source_pixels_modified
            && art.derived_blocks == expected_recipes,
        "exact native Bemidji timber recipes required"
    );
    available(base, &town)?;
    let gym_room = town
        .interiors
        .iter()
        .find(|r| r.id == GYM)
        .context("city canonical gym reference")?;
    let raw: Value = serde_json::from_str(gyms_json)?;
    let raw_gym = raw["gyms"]
        .as_array()
        .context("canonical gym array")?
        .iter()
        .find(|g| g["id"] == GYM)
        .context("canonical Gym7 object")?;
    let hash = format!("{:x}", Sha256::digest(serde_json::to_vec(raw_gym)?));
    ensure!(
        gym_room.canonical_gym_sha256.as_deref() == Some(hash.as_str())
            && gym_room.dimensions_metatiles == gym.dimensions_metatiles
            && gym_room.passive_arrival.tile == gym.arrival_core
            && gym_room.native_constant == gym.constant
            && gym_room.exits.iter().map(|e| e.tile).collect::<Vec<_>>() == gym.exit_tiles_core,
        "stale city canonical Gym7 reference; refresh source preview"
    );
    let needed = gym
        .rows
        .iter()
        .flat_map(|row| row.chars())
        .chain(gym.gates.iter().map(|gate| gate.open_symbol))
        .chain(gym.cut_gates.iter().map(|gate| gate.replacement_symbol))
        .collect::<BTreeSet<_>>();
    let symbols = needed
        .into_iter()
        .enumerate()
        .map(|(index, char)| Ok((char, u16::try_from(index)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let expected_rows = gym
        .rows
        .iter()
        .map(|row| {
            row.chars()
                .map(|char| format!("{:02x}", symbols[&char]))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>();
    ensure!(
        gym_room.tileset == gym.tileset()
            && gym_room.rows == expected_rows
            && gym_room.passive_arrival.index == 3,
        "city gym preview terrain differs from canonical source"
    );
    let mut expected_actors = vec![
        (
            format!("{GYM}Leader"),
            gym.leader_sprite.as_str(),
            gym.leader_at,
        ),
        (format!("{GYM}Guide"), "SPRITE_GYM_GUIDE", gym.guide_at),
    ];
    expected_actors.extend(
        gym.trainers
            .iter()
            .map(|t| (t.id.clone(), t.sprite.as_str(), t.at)),
    );
    let actual_actors = gym_room
        .residents
        .iter()
        .map(|r| (r.id.clone(), r.sprite.as_str(), r.tile))
        .collect::<BTreeSet<_>>();
    ensure!(
        actual_actors == expected_actors.into_iter().collect()
            && actual_actors.len() == gym_room.residents.len(),
        "city gym preview actors differ from canonical source"
    );
    validate_installed_gym(base, gym)?;
    let mut pack = base.clone();
    bemidji_city_art::install(&mut pack)?;
    let stock = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .find(|r| matches!(r.service, Some(Service::CanonicalPokemartPending)))
        .context("actual mart clerk")?
        .stock
        .clone();
    ensure!(
        !stock.is_empty() && stock.iter().all(|item| pack.data.items.contains_key(item)),
        "legal authored Bemidji mart stock"
    );
    pack.data.marts.0.insert(MART.into(), stock);
    let mut exterior = authoring::blank(&pack, "CherrygroveCity", BEMIDJI_CITY_MAP, CONSTANT)?;
    exterior.attributes.width = 24;
    exterior.attributes.height = 24;
    exterior.attributes.tileset_name = town.compiled_tileset.clone();
    exterior.attributes.border_block = 5;
    exterior.attributes.music = pack
        .data
        .maps
        .get("EcruteakCity")
        .context("native Ecruteak music")?
        .attributes
        .music
        .clone();
    exterior.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
    exterior.attributes.location = Some(LANDMARK.into());
    exterior.blocks = layout.compile()?;
    people(&pack, &mut exterior, &town.residents, &town.signs, None)?;
    let mut modules = Vec::new();
    for room in town.interiors.iter().filter(|r| r.id != GYM) {
        let template = match room.tileset.as_str() {
            "pokecenter" => "MahoganyPokecenter1F",
            "mart" => "VioletMart",
            "house" => "GuideGentsHouse",
            _ => anyhow::bail!("unsupported authored room art {}", room.id),
        };
        let mut module = authoring::blank(&pack, template, &room.id, &room.native_constant)?;
        authoring::blocks(
            &mut module,
            room.dimensions_metatiles[0],
            room.dimensions_metatiles[1],
            &room.rows.join(" "),
        )?;
        ensure!(
            module.attributes.tileset_name == room.tileset,
            "source room tileset mismatch"
        );
        module.attributes.border_block = if room.tileset == "house" { 0x0a } else { 0 };
        module.attributes.map_group_constant = exterior.attributes.map_group_constant.clone();
        module.attributes.location = exterior.attributes.location.clone();
        people(
            &pack,
            &mut module,
            &room.residents,
            &room.exhibits,
            room.pc.as_ref(),
        )?;
        modules.push(module);
    }
    validate_geometry(&pack, &town, &exterior, &modules)?;
    for (index, door) in town.doors.iter().enumerate() {
        ensure!(
            door.warp_index == u16::try_from(index * 2 + 1)?
                && door.return_warp_index == u16::try_from(index * 2 + 2)?
                && door.target_warp == 3
                && door.return_tile == door.arrival_tile
                && door.return_tile == [door.tile[0], door.tile[1] + 1],
            "paired native exterior warp order"
        );
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("door target room")?;
        authoring::warp(
            &mut exterior,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            3,
        );
        authoring::warp(
            &mut exterior,
            (door.return_tile[0], door.return_tile[1]),
            &room.native_constant,
            3,
        );
        if room.id == GYM {
            continue;
        }
        let module = modules
            .iter_mut()
            .find(|m| m.id == room.id)
            .context("new room module")?;
        ensure!(
            room.passive_arrival.index == 3 && room.exits.len() == 2,
            "two carpet exits and passive FLOOR arrival3"
        );
        for (i, exit) in room.exits.iter().enumerate() {
            ensure!(
                exit.index == u16::try_from(i + 1)?
                    && exit.target == BEMIDJI_CITY_MAP
                    && exit.target_warp == i16::try_from(door.return_warp_index)?,
                "source room reciprocal passive return"
            );
            authoring::warp(
                module,
                (exit.tile[0], exit.tile[1]),
                CONSTANT,
                exit.target_warp,
            );
        }
        let [x, y] = room.passive_arrival.tile;
        authoring::warp(
            module,
            (x, y),
            CONSTANT,
            i16::try_from(door.return_warp_index)?,
        );
        finish(&pack, module)?;
    }
    finish(&pack, &mut exterior)?;
    let gym_door = town
        .doors
        .iter()
        .find(|d| d.target == GYM)
        .context("real gym facade")?;
    let mut rebound = pack.data.maps.get(GYM).context("installed gym")?.clone();
    for exit in rebound
        .events
        .warps
        .iter_mut()
        .filter(|w| [1, 2, 3].contains(&w.index))
    {
        exit.target_map_constant = CONSTANT.into();
        exit.target_map = CONSTANT.into();
        exit.target_warp_id = i16::try_from(gym_door.return_warp_index)?;
    }
    // Rebuild only the native event wire representation; all canonical script
    // source and parser tables must remain equal to the validated installed gym.
    authoring::finish(&pack, &mut rebound)?;
    let old = pack.data.maps.get(GYM).context("original gym")?;
    let mut allowed = serde_json::to_value(old)?;
    let actual = serde_json::to_value(&rebound)?;
    allowed["events"]["warps"] = actual["events"]["warps"].clone();
    allowed["map_event_section_commands"] = actual["map_event_section_commands"].clone();
    ensure!(
        allowed == actual,
        "gym rebinding changed canonical behavior beyond warp wire data"
    );
    // Keep the source-shaped split catalog in agreement with the validated
    // module. Split-payload reloads must not retain the review-hall exits.
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &rebound.id,
        rebound
            .attributes
            .map_scripts_label
            .as_deref()
            .context("Gym7 scripts label")?,
        rebound
            .attributes
            .map_events_label
            .as_deref()
            .context("Gym7 events label")?,
        &rebound,
    )?;
    pack.data.maps.insert(GYM.into(), rebound);
    modules.insert(0, exterior);
    publish(pack, &town, modules)
}

fn available(base: &CompiledGamePack, town: &Town) -> Result<()> {
    ensure!(
        town.doors.len() == 6 && town.interiors.len() == 6,
        "six authentic facades and rooms"
    );
    ensure!(
        !base
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == BEMIDJI_CITY_GROUP)
            && !base
                .data
                .story_event_script_constants
                .global
                .contains_key(&format!("MAPGROUP_{GROUP}"))
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Bemidji group70 occupied"
    );
    ensure!(
        !base.data.marts.0.contains_key(MART)
            && !base
                .data
                .pokegear_landmarks
                .landmarks
                .iter()
                .any(|l| l.constant == LANDMARK),
        "Bemidji service/landmark occupied"
    );
    let mut names = BTreeSet::new();
    let mut constants = BTreeSet::new();
    for (name, constant) in std::iter::once((town.id.as_str(), town.native_constant.as_str()))
        .chain(
            town.interiors
                .iter()
                .filter(|r| r.id != GYM)
                .map(|r| (r.id.as_str(), r.native_constant.as_str())),
        )
    {
        ensure!(
            names.insert(name)
                && constants.insert(constant)
                && !base.data.maps.contains_key(name)
                && !base.data.runtime_map_metadata.contains_key(constant)
                && !base
                    .data
                    .runtime_map_metadata
                    .values()
                    .any(|m| m.name == name),
            "Bemidji map/constant occupied: {name}"
        );
    }
    ensure!(
        base.data.maps.contains_key(GYM),
        "canonical Gym7 must already be installed"
    );
    Ok(())
}

fn validate_installed_gym(base: &CompiledGamePack, gym: &Gym) -> Result<()> {
    let installed = base
        .data
        .maps
        .get(GYM)
        .context("installed canonical Gym7")?;
    let mut scratch = base.clone();
    scratch.data.tilesets.remove(&gym.tileset());
    scratch
        .data
        .field_moves
        .cut
        .replacements
        .remove(&gym.tileset());
    for path in [
        format!("data/tilesets/{}_metatiles.bin", gym.tileset()),
        format!("data/tilesets/{}.json", gym.tileset()),
        format!("data/tilesets/{}_palette_map.json", gym.tileset()),
        format!("gfx/tilesets/{}.2bpp", gym.tileset()),
        format!("gfx/tilesets/{}.png", gym.tileset()),
    ] {
        scratch.runtime_files.remove(&path);
    }
    let symbols = crate::gym_art::install(&mut scratch, gym)?;
    ensure!(
        scratch.data.field_moves == base.data.field_moves,
        "installed Gym7 field-move behavior differs from canonical source"
    );
    ensure!(
        scratch.data.tilesets.get(&gym.tileset()) == base.data.tilesets.get(&gym.tileset()),
        "installed Gym7 art definition differs from canonical source"
    );
    for (path, bytes) in scratch
        .runtime_files
        .iter()
        .filter(|(path, _)| path.contains(&gym.tileset()))
    {
        ensure!(
            base.runtime_files.get(path) == Some(bytes),
            "installed Gym7 native art differs: {path}"
        );
    }
    let ids = gym
        .trainers
        .iter()
        .map(|t| t.id.clone())
        .chain([gym.battle_id()])
        .collect::<Vec<_>>();
    for id in &ids {
        scratch.data.trainers.trainers.remove(id);
        scratch.data.story_event_script_constants.global.remove(id);
    }
    let hall_index = match installed
        .events
        .warps
        .first()
        .map(|warp| warp.target_warp_id)
    {
        Some(3) => 0,
        Some(21) => 6,
        _ => anyhow::bail!("installed Gym7 has no canonical review-hall return"),
    };
    let mut expected = crate::gym_publication::room(&scratch, gym, &symbols, hall_index)?;
    crate::gym_scripts::install(&mut scratch, &mut expected, gym, &symbols)?;
    for id in &ids {
        ensure!(
            scratch.data.trainers.trainers.get(id) == base.data.trainers.trainers.get(id),
            "installed Gym7 trainer differs from canonical source: {id}"
        );
    }
    scratch.data.story_event_script_constants.global =
        base.data.story_event_script_constants.global.clone();
    crate::gym_scripts::finish(&scratch, &mut expected)?;
    ensure!(
        &expected == installed,
        "installed Gym7 module differs from supplied canonical source; rebuild gyms before city publication"
    );
    Ok(())
}

fn people(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    residents: &[Resident],
    interactions: &[Interaction],
    pc: Option<&Pc>,
) -> Result<()> {
    for resident in residents {
        ensure!(
            !module.objects.iter().any(|o| [o.x, o.y] == resident.tile)
                && !module.scripts.contains_key(&resident.id),
            "duplicate city actor"
        );
        ensure!(
            pack.runtime_files.contains_key(&format!(
                "gfx/sprites/{}.png",
                resident
                    .sprite
                    .trim_start_matches("SPRITE_")
                    .to_ascii_lowercase()
            )),
            "missing original native resident sprite"
        );
        let label = format!("{}Text", resident.id);
        authored_text(module, &label, &resident.lines)?;
        let body = match resident.service {
            None => json!([authoring::command("jumptextfaceplayer", &[&label])]),
            Some(Service::HealParty) => json!([
                authoring::command("faceplayer", &[]),
                authoring::command("opentext", &[]),
                authoring::command("writetext", &[&label]),
                authoring::command("waitbutton", &[]),
                authoring::command("special", &["HealParty"]),
                authoring::command("blackoutmod", &["MN_BEMIDJI_POKECENTER_1F"]),
                authoring::command("closetext", &[]),
                authoring::command("end", &[])
            ]),
            Some(Service::CanonicalPokemartPending) => json!([
                authoring::command("faceplayer", &[]),
                authoring::command("opentext", &[]),
                authoring::command("pokemart", &["MARTTYPE_STANDARD", MART]),
                authoring::command("closetext", &[]),
                authoring::command("end", &[])
            ]),
        };
        module.scripts.insert(resident.id.clone(), body);
        authoring::npc(
            pack,
            module,
            &resident.id,
            &resident.sprite,
            i16::try_from(resident.tile[0])?,
            i16::try_from(resident.tile[1])?,
        )?;
        let object = module.objects.last_mut().context("city native resident")?;
        object.object_identifier = Some(resident.id.clone());
        object.spritemovedata = resident.facing.movement().into();
        object.object_type = "OBJECTTYPE_SCRIPT".into();
        object.radius = 0;
    }
    for interaction in interactions {
        interaction_script(module, interaction)?;
    }
    if let Some(pc) = pc {
        let script = "MnBemidjiClinicPc";
        module.scripts.insert(
            script.into(),
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
            script: script.into(),
        });
    }
    Ok(())
}
fn authored_text(module: &mut MapModule, label: &str, lines: &[String]) -> Result<()> {
    ensure!(
        !lines.is_empty()
            && lines
                .iter()
                .all(|s| !s.is_empty() && s.chars().count() <= 18),
        "city dialogue must fit18columns"
    );
    authoring::text(
        module,
        label,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    Ok(())
}
fn interaction_script(module: &mut MapModule, interaction: &Interaction) -> Result<()> {
    ensure!(
        !module.scripts.contains_key(&interaction.id),
        "duplicate city interaction"
    );
    let label = format!("{}Text", interaction.id);
    authored_text(module, &label, &interaction.lines)?;
    module.scripts.insert(
        interaction.id.clone(),
        json!([authoring::command("jumptext", &[&label])]),
    );
    module.events.bg_events.push(BackgroundEvent {
        x: interaction.tile[0],
        y: interaction.tile[1],
        event_type: "BGEVENT_READ".into(),
        script: interaction.id.clone(),
    });
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
fn kind<'a>(pack: &'a CompiledGamePack, module: &MapModule, [x, y]: [u16; 2]) -> Result<&'a str> {
    ensure!(
        x < module.attributes.width * 2 && y < module.attributes.height * 2,
        "city event outside authored extent"
    );
    let block = module.blocks
        [usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2)];
    let q = pack
        .data
        .tilesets
        .get(&module.attributes.tileset_name)
        .context("city tileset")?
        .collision
        .get(&format!("{block:02x}"))
        .context("city metatile collision")?;
    Ok(&q[usize::from(y % 2 * 2 + x % 2)])
}
fn neighbors([x, y]: [u16; 2], width: u16, height: u16) -> Vec<[u16; 2]> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        (x + 1 < width).then_some([x + 1, y]),
        (y + 1 < height).then_some([x, y + 1]),
    ]
    .into_iter()
    .flatten()
    .collect()
}
fn reachable(
    pack: &CompiledGamePack,
    module: &MapModule,
    start: [u16; 2],
    surf: bool,
) -> Result<BTreeSet<[u16; 2]>> {
    let occupied = module
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        !occupied.contains(&start) && matches!(kind(pack, module, start)?, "FLOOR" | "DOOR"),
        "safe native city arrival"
    );
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        if kind(pack, module, at)? == "DOOR" {
            continue;
        }
        for q in neighbors(
            at,
            module.attributes.width * 2,
            module.attributes.height * 2,
        ) {
            let collision = kind(pack, module, q)?;
            if !occupied.contains(&q)
                && (matches!(collision, "FLOOR" | "DOOR" | "WARP_CARPET_DOWN")
                    || surf && collision == "WATER")
                && seen.insert(q)
            {
                queue.push_back(q);
            }
        }
    }
    Ok(seen)
}
fn validate_geometry(
    pack: &CompiledGamePack,
    town: &Town,
    exterior: &MapModule,
    rooms: &[MapModule],
) -> Result<()> {
    let expected = [
        ("MN_R08", "S", [12, 22], [12, 20]),
        ("MN_R09", "E", [22, 12], [20, 12]),
        ("MN_R14", "W", [1, 12], [3, 12]),
        ("MN_R17", "N", [12, 1], [12, 3]),
    ];
    ensure!(town.ports.len() == 4, "four original Bemidji ports");
    let seen = reachable(pack, exterior, [24, 40], false)?;
    let surf = reachable(pack, exterior, [24, 40], true)?;
    for (port, (route, side, gate, landing)) in town.ports.iter().zip(expected) {
        ensure!(
            port.route == route
                && port.side == side
                && port.gate_metatile == gate
                && port.landing_metatile == landing
                && [gate, landing]
                    .into_iter()
                    .all(|p| seen.contains(&p.map(|v| v * 2))),
            "original town port identity/connectivity"
        );
    }
    let doors = town.doors.iter().map(|d| d.tile).collect::<BTreeSet<_>>();
    let mut native_doors = BTreeSet::new();
    let mut launches = BTreeSet::new();
    let expected_outer = BTreeSet::from([
        [0, 24],
        [0, 25],
        [47, 24],
        [47, 25],
        [24, 0],
        [25, 0],
        [24, 47],
        [25, 47],
    ]);
    for y in 0..48 {
        for x in 0..48 {
            let at = [x, y];
            let k = kind(pack, exterior, at)?;
            ensure!(k != "CUT_TREE", "no incidental city Cut gate");
            if k == "DOOR" {
                native_doors.insert(at);
            }
            if k == "FLOOR" {
                for q in neighbors(at, 48, 48) {
                    if kind(pack, exterior, q)? == "WATER" {
                        launches.insert((at, q));
                    }
                }
            }
            if x == 0 || y == 0 || x == 47 || y == 47 {
                ensure!(
                    !(k == "WATER" && surf.contains(&at)),
                    "unconnected Surf outer boundary"
                );
                ensure!(
                    !seen.contains(&at) || expected_outer.contains(&at),
                    "unbound outer walking exit"
                );
            }
            if matches!(k, "FLOOR" | "DOOR") && !exterior.objects.iter().any(|o| [o.x, o.y] == at) {
                ensure!(seen.contains(&at), "isolated public exterior floor");
            }
        }
    }
    ensure!(
        doors == native_doors,
        "unbound or false exterior native door"
    );
    ensure!(
        expected_outer.iter().all(|at| seen.contains(at)),
        "original outer port lane disconnected"
    );
    ensure!(
        launches == BTreeSet::from([([29, 40], [30, 40]), ([29, 41], [30, 41])]),
        "only the marked low lake launch may meet Surf water"
    );
    for door in &town.doors {
        ensure!(
            seen.contains(&door.tile)
                && seen.contains(&door.return_tile)
                && kind(pack, exterior, door.return_tile)? == "FLOOR",
            "unsafe actual door return"
        );
    }
    for actor in &town.residents {
        ensure!(
            kind(pack, exterior, actor.tile)? == "FLOOR"
                && neighbors(actor.tile, 48, 48)
                    .iter()
                    .any(|p| seen.contains(p)),
            "inaccessible city actor"
        );
    }
    for sign in &town.signs {
        ensure!(
            kind(pack, exterior, sign.tile)? == "WALL"
                && neighbors(sign.tile, 48, 48)
                    .iter()
                    .any(|p| seen.contains(p)),
            "inaccessible native city sign"
        );
    }
    for at in [[11, 19], [11, 22], [27, 35], [37, 35], [29, 40]] {
        ensure!(seen.contains(&at), "ledge/pier approach disconnected");
    }
    ensure!(
        kind(pack, exterior, [11, 20])? == "HOP_DOWN" && kind(pack, exterior, [11, 21])? == "WALL",
        "native downhill ledge/reverse barrier"
    );
    let officer = exterior
        .objects
        .iter()
        .find(|o| o.object_identifier.as_deref() == Some("MN_BEMIDJI_OFFICER"))
        .context("existing Q05 officer anchor")?;
    ensure!(
        [officer.x, officer.y] == [21, 19],
        "preserve Q05 officer; scene binding remains separate"
    );
    for room in town.interiors.iter().filter(|r| r.id != GYM) {
        let module = rooms
            .iter()
            .find(|m| m.id == room.id)
            .context("authored native room")?;
        ensure!(
            room.rows.len() == usize::from(room.dimensions_metatiles[1])
                && room
                    .rows
                    .iter()
                    .all(|r| r.split_whitespace().count()
                        == usize::from(room.dimensions_metatiles[0])),
            "actual room row extent"
        );
        let public = reachable(pack, module, room.passive_arrival.tile, false)?;
        ensure!(
            kind(pack, module, room.passive_arrival.tile)? == "FLOOR",
            "room passive arrival must be FLOOR"
        );
        let exits = room.exits.iter().map(|e| e.tile).collect::<BTreeSet<_>>();
        let mut carpets = BTreeSet::new();
        for y in 0..module.attributes.height * 2 {
            for x in 0..module.attributes.width * 2 {
                let at = [x, y];
                if kind(pack, module, at)? == "WARP_CARPET_DOWN" {
                    carpets.insert(at);
                }
                if public.contains(&at)
                    && (x == 0
                        || y == 0
                        || x == module.attributes.width * 2 - 1
                        || y == module.attributes.height * 2 - 1)
                {
                    ensure!(exits.contains(&at), "room exterior floor leak");
                }
            }
        }
        ensure!(
            exits.len() == 2 && carpets == exits && exits.iter().all(|p| public.contains(p)),
            "two actual reachable room carpet exits"
        );
        for actor in &room.residents {
            ensure!(
                kind(pack, module, actor.tile)? == "FLOOR",
                "native service actor off FLOOR"
            );
            if let (Some(counter), Some(approach)) = (actor.counter_tile, actor.approach) {
                ensure!(
                    kind(pack, module, counter)? == "COUNTER" && public.contains(&approach),
                    "native counter inaccessible"
                );
            } else {
                ensure!(
                    neighbors(
                        actor.tile,
                        module.attributes.width * 2,
                        module.attributes.height * 2
                    )
                    .iter()
                    .any(|p| public.contains(p)),
                    "room actor inaccessible"
                );
            }
        }
        for exhibit in &room.exhibits {
            ensure!(
                kind(pack, module, exhibit.tile)? == "BOOKSHELF"
                    && neighbors(
                        exhibit.tile,
                        module.attributes.width * 2,
                        module.attributes.height * 2
                    )
                    .iter()
                    .any(|p| public.contains(p)),
                "native heritage exhibit inaccessible"
            );
        }
        if let Some(pc) = &room.pc {
            ensure!(
                kind(pack, module, pc.tile)? == "PC" && public.contains(&pc.approach),
                "native PC terminal inaccessible"
            );
        }
    }
    Ok(())
}

fn publish(
    mut pack: CompiledGamePack,
    town: &Town,
    modules: Vec<MapModule>,
) -> Result<CompiledGamePack> {
    pack.data
        .story_event_script_constants
        .global
        .insert(format!("MAPGROUP_{GROUP}"), i64::from(BEMIDJI_CITY_GROUP));
    let id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("native landmarks")?
        .checked_add(1)
        .context("Bemidji landmark id")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id,
            constant: LANDMARK.into(),
            label: "MN_BEMIDJI".into(),
            name: "BEMIDJI".into(),
            x: 88,
            y: 32,
            region: "JOHTO".into(),
        });
    let first_spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|p| p.identifier)
        .max()
        .context("source recovery records")?
        .checked_add(1)
        .context("new recovery id")?
        .max(crystal_core::systems::special_routines::FIRST_ADDON_RECOVERY_SPAWN);
    for (index, module) in modules.into_iter().enumerate() {
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("authored constant")?;
        let arrival = if name == BEMIDJI_CITY_MAP {
            [31, 10]
        } else {
            town.interiors
                .iter()
                .find(|r| r.id == name)
                .context("room arrival")?
                .passive_arrival
                .tile
        };
        let identifier = first_spawn
            .checked_add(u16::try_from(index)?)
            .context("new recovery id")?;
        let map_id = u16::try_from(index + 1)?;
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("{MANIFEST}-{index}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                name: name.clone(),
                constant: constant.clone(),
                group_name: GROUP.into(),
                group_id: BEMIDJI_CITY_GROUP,
                map_id,
                width: module.attributes.width,
                height: module.attributes.height,
                environment: module
                    .attributes
                    .environment
                    .clone()
                    .context("authored environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: identifier.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier,
                map_constant: constant,
                map_name: name.clone(),
                group_name: GROUP.into(),
                group_id: i16::try_from(BEMIDJI_CITY_GROUP)?,
                map_id: i16::try_from(map_id)?,
                tile_x: i16::try_from(arrival[0])?,
                tile_y: i16::try_from(arrival[1])?,
                metatile_x: i16::try_from(arrival[0] / 2)?,
                metatile_y: i16::try_from(arrival[1] / 2)?,
                subtile_x: i16::try_from(arrival[0] % 2)?,
                subtile_y: i16::try_from(arrival[1] % 2)?,
            },
            module,
            wild_encounters: None,
            start_new_game_here: false,
        };
        crate::validate_compiled_map_extension(&extension)?;
        crate::insert_compiled_map_extension(&mut pack, extension)?;
        if !matches!(name.as_str(), BEMIDJI_CITY_MAP | "MnBemidjiPokecenter1F") {
            pack.data
                .runtime_spawn_points
                .remove(&identifier.to_string());
        }
    }
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
    pack.report.reachable_maps.sort();
    pack.report.solvable_maps.sort();
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
