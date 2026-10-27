//! Native preserve terrain and hospitality; expedition admission remains closed.
use std::collections::{BTreeSet, VecDeque};

mod expansion;
mod expansion_geometry;
pub use expansion::expand_boundary_safari_modpack;

use anyhow::{Context, Result, ensure};
use crystal_core::map::{BackgroundEvent, MapConnection};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_route_layout::Layout;
use crate::authored_route_modpack::collision;
use crate::grand_marais_modpack::authoring;
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RootBeerRestPolicy,
    RuntimeMapMetadata, RuntimeSpawnPoint, prepare_root_beer_cabin,
};

/// Native group reserved for four preserve chunks and the actual cabin.
pub const BOUNDARY_SAFARI_GROUP: u16 = 61;
/// This overlay publishes terrain, not an active Safari expedition.
pub const BOUNDARY_SAFARI_MANIFEST: &str = "minnesota-boundary-waters-terrain-v1";
/// Two always-present native guards close the actual arrival-to-launch throat.
pub const BOUNDARY_SAFARI_PENDING_GUARDS: [(&str, [u16; 2]); 2] = [
    ("BWS_PENDING_GATE_WEST", [11, 69]),
    ("BWS_PENDING_GATE_EAST", [12, 69]),
];
const GROUP: &str = "MINNESOTA_BOUNDARY_SAFARI";
const LANDMARK: &str = "LANDMARK_MN_BOUNDARY_SAFARI";
const CLOSED_LINES: &[&str] = &[
    "EXPEDITIONS CLOSED",
    "Safari permits are",
    "not available yet.",
    "No fee is charged.",
];

#[derive(Deserialize)]
struct Source {
    retire_anchors: Vec<Actor>,
    landmark_actors: Vec<Actor>,
    decorations: Vec<Actor>,
    signs: Vec<Sign>,
}
#[derive(Deserialize)]
struct Actor {
    id: String,
    tile: [u16; 2],
    sprite: String,
    facing: String,
    #[serde(default)]
    lines: Vec<String>,
}
#[derive(Deserialize)]
struct Sign {
    id: String,
    tile: [u16; 2],
    lines: Vec<String>,
}

/// Compiles the frozen native preserve and actual empty Root Beer cabin.
///
/// Publishes four clipped48×40-metatile preserve chunks and the5×4 authored
/// cabin in group61. Exact source actors, campsites, sign anchors and native
/// terrain remain; the existing outside Dorothy gets real gift/shop/rest.
/// Reciprocal internal walking/Surf neighbors and the closed outer water edge
/// are validated. The two native gate guards and truthful ranger text keep the
/// ordinary arrival closed while the expedition engine is absent.
///
/// No fee, permit, balls, steps, ordinary wild encounter table, capture mode,
/// indoor Dorothy relocation, regional hub connection, HOME change or recovery
/// destination is introduced. Explicit native review placement may inspect the
/// terrain. This function does not claim a playable Safari expedition.
///
/// # Errors
/// Returns an error for invalid source/cabin contracts, changed source collision,
/// unsafe actor/arrival/boundary/gate, missing native assets, occupied group/map
/// identifiers, service/parser failures or failed final production verification.
/// The supplied base remains unchanged on success and failure.
pub fn build_boundary_safari_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    zone_json: &str,
    cabin_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base
            .data()
            .runtime_map_metadata
            .values()
            .any(|map| map.group_id == BOUNDARY_SAFARI_GROUP)
            && !base.data().maps.contains_key("MnBoundarySafariC00R00")
            && !base
                .report()
                .manifests
                .iter()
                .any(|id| id == BOUNDARY_SAFARI_MANIFEST),
        "Boundary Waters preserve or native group61 already exists"
    );
    let zone: Value = serde_json::from_str(zone_json).context("Safari zone contract")?;
    ensure!(
        zone["id"] == "MN_BOUNDARY_WATERS_SAFARI"
            && zone["candidate_extent_metatiles"] == json!([48, 40])
            && zone["story_contract"]["paid_expedition_required_for_story"] == false,
        "retain optional preserve extent and free story access"
    );
    let layout = Layout::parse(layout_json)?;
    ensure!(
        layout.id == "MN_BOUNDARY_WATERS_SAFARI"
            && layout.dimensions_metatiles == [48, 40]
            && layout.compiled_tileset == "mn_boundary_safari",
        "actual preserve layout required"
    );
    let blocks = layout.compile()?;
    let source: Source = serde_json::from_str(layout_json).context("preserve source anchors")?;
    let anchors: Value = serde_json::from_str(layout_json)?;
    let mut pack = if base.data().tilesets.contains_key("mn_boundary_safari") {
        base.clone()
    } else {
        crate::build_boundary_safari_art_pack(base)?
    };
    pack.data
        .materialize_global_scripts()
        .context("native field scripts")?;
    // Materializing the supplied native script roots changes compiled data.
    // The cabin/service preparer verifies its input, so refresh this legitimate
    // intermediate identity before crossing that production boundary.
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    validate_source(&pack, &blocks, &source, &anchors)?;
    let mut modules = Vec::new();
    for row in 0..2 {
        for column in 0..2 {
            let name = chunk([column * 64, row * 48]);
            let mut module = authoring::blank(&pack, "Route32", &name, &map_constant(column, row))?;
            module.attributes.width = if column == 0 { 32 } else { 16 };
            module.attributes.height = if row == 0 { 24 } else { 16 };
            module.attributes.tileset_name = "mn_boundary_safari".into();
            module.attributes.border_block = 5;
            module.attributes.music = pack.data().maps["Route42"].attributes.music.clone();
            module.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
            module.attributes.location = Some(LANDMARK.into());
            module.blocks.clear();
            for y in row * 24..row * 24 + module.attributes.height {
                let offset = usize::from(y) * 48 + usize::from(column) * 32;
                module.blocks.extend_from_slice(
                    &blocks[offset..offset + usize::from(module.attributes.width)],
                );
            }
            for (direction, target_column, target_row) in [
                ("west", i32::from(column) - 1, i32::from(row)),
                ("east", i32::from(column) + 1, i32::from(row)),
                ("north", i32::from(column), i32::from(row) - 1),
                ("south", i32::from(column), i32::from(row) + 1),
            ] {
                if (0..2).contains(&target_column) && (0..2).contains(&target_row) {
                    module.attributes.connections.push(MapConnection {
                        direction: direction.into(),
                        target_map: chunk([target_column as u16 * 64, target_row as u16 * 48]),
                        offset: 0,
                    });
                }
            }
            module.attributes.connection_flags = Some(
                module
                    .attributes
                    .connections
                    .iter()
                    .map(|edge| edge.direction.to_ascii_uppercase())
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
            modules.push(module);
        }
    }
    for actor in source
        .retire_anchors
        .iter()
        .chain(&source.landmark_actors)
        .chain(&source.decorations)
    {
        let module = module_at(&mut modules, actor.tile)?;
        let local = [actor.tile[0] % 64, actor.tile[1] % 48];
        let text = if source
            .retire_anchors
            .iter()
            .any(|ranger| ranger.id == actor.id)
        {
            CLOSED_LINES.to_vec()
        } else {
            actor.lines.iter().map(String::as_str).collect()
        };
        author_actor(
            &pack,
            module,
            &actor.id,
            &actor.sprite,
            local,
            &actor.facing,
            &text,
        )?;
    }
    for (id, tile) in BOUNDARY_SAFARI_PENDING_GUARDS {
        author_actor(
            &pack,
            module_at(&mut modules, tile)?,
            id,
            "SPRITE_OFFICER",
            [tile[0] % 64, tile[1] % 48],
            "south",
            CLOSED_LINES,
        )?;
    }
    for sign in &source.signs {
        let module = module_at(&mut modules, sign.tile)?;
        let label = format!("{}Text", sign.id);
        let lines = if sign.id == "BWS_SIGN_01" || sign.id == "BWS_SIGN_06" {
            CLOSED_LINES.to_vec()
        } else {
            sign.lines.iter().map(String::as_str).collect()
        };
        module.scripts.insert(
            sign.id.clone(),
            json!([authoring::command("jumptext", &[&label])]),
        );
        authoring::text(module, &label, &lines);
        module.events.bg_events.push(BackgroundEvent {
            x: sign.tile[0] % 64,
            y: sign.tile[1] % 48,
            event_type: "BGEVENT_READ".into(),
            script: sign.id.clone(),
        });
    }
    for module in &mut modules {
        authoring::finish(&pack, module)?;
    }
    let exterior = modules
        .iter()
        .find(|module| module.id == "MnBoundarySafariC00R00")
        .context("actual cabin exterior chunk")?;
    let prepared = prepare_root_beer_cabin(
        &pack,
        cabin_json,
        layout_json,
        exterior,
        RootBeerRestPolicy::Offer,
    )?;
    pack = prepared.pack;
    let exterior_index = modules
        .iter()
        .position(|module| module.id == prepared.exterior.id)
        .context("prepared cabin exterior")?;
    modules[exterior_index] = prepared.exterior;
    modules.push(prepared.cabin);
    register_identity(&mut pack)?;
    let temporary_spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|spawn| spawn.identifier)
        .max()
        .context("original recovery table")?
        .checked_add(1)
        .context("temporary spawn identifier")?;
    let arrivals = [[50, 43], [18, 27], [15, 25], [6, 19], [5, 6]];
    for (index, module) in modules.into_iter().enumerate() {
        let map_id = u16::try_from(index + 1)?;
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("authored native map constant")?;
        let [x, y] = arrivals[index];
        ensure!(
            collision(
                &pack,
                &module.blocks,
                module.attributes.width,
                [x, y],
                &module.attributes.tileset_name
            )? == "FLOOR"
                && !module
                    .objects
                    .iter()
                    .any(|actor| actor.x == x && actor.y == y),
            "unsafe temporary publication arrival"
        );
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("{BOUNDARY_SAFARI_MANIFEST}-{index}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                name: name.clone(),
                constant: constant.clone(),
                group_name: GROUP.into(),
                group_id: BOUNDARY_SAFARI_GROUP,
                map_id,
                width: module.attributes.width,
                height: module.attributes.height,
                environment: module
                    .attributes
                    .environment
                    .clone()
                    .context("native map environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: temporary_spawn.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier: temporary_spawn,
                map_constant: constant,
                map_name: name,
                group_name: GROUP.into(),
                group_id: i16::try_from(BOUNDARY_SAFARI_GROUP)?,
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
        pack.data
            .runtime_spawn_points
            .remove(&temporary_spawn.to_string());
    }
    pack.report.manifests.push(BOUNDARY_SAFARI_MANIFEST.into());
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

fn chunk([x, y]: [u16; 2]) -> String {
    format!("MnBoundarySafariC{:02}R{:02}", x / 64, y / 48)
}
fn map_constant(column: u16, row: u16) -> String {
    format!("MN_BOUNDARY_SAFARI_C{column:02}R{row:02}")
}
fn module_at(modules: &mut [MapModule], tile: [u16; 2]) -> Result<&mut MapModule> {
    modules
        .iter_mut()
        .find(|module| module.id == chunk(tile))
        .context("preserve source anchor outside chunks")
}
fn author_actor(
    pack: &CompiledGamePack,
    module: &mut MapModule,
    id: &str,
    sprite: &str,
    tile: [u16; 2],
    facing: &str,
    lines: &[&str],
) -> Result<()> {
    let label = format!("{id}Text");
    module.scripts.insert(
        id.into(),
        if lines.is_empty() {
            json!([authoring::command("end", &[])])
        } else {
            json!([authoring::command("jumptextfaceplayer", &[&label])])
        },
    );
    if !lines.is_empty() {
        authoring::text(module, &label, lines);
    }
    authoring::npc(
        pack,
        module,
        id,
        sprite,
        i16::try_from(tile[0])?,
        i16::try_from(tile[1])?,
    )?;
    let actor = module
        .objects
        .last_mut()
        .context("native authored object")?;
    actor.object_identifier = Some(id.into());
    actor.spritemovedata = match facing {
        "north" => "SPRITEMOVEDATA_STANDING_UP",
        "east" => "SPRITEMOVEDATA_STANDING_RIGHT",
        "west" => "SPRITEMOVEDATA_STANDING_LEFT",
        "south" => "SPRITEMOVEDATA_STANDING_DOWN",
        _ => anyhow::bail!("invalid actor facing"),
    }
    .into();
    actor.object_type = "OBJECTTYPE_SCRIPT".into();
    actor.radius = 0;
    Ok(())
}
fn register_identity(pack: &mut CompiledGamePack) -> Result<()> {
    let group = format!("MAPGROUP_{GROUP}");
    ensure!(
        !pack
            .data
            .story_event_script_constants
            .global
            .contains_key(&group),
        "preserve group constant occupied"
    );
    pack.data
        .story_event_script_constants
        .global
        .insert(group, i64::from(BOUNDARY_SAFARI_GROUP));
    ensure!(
        !pack
            .data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|entry| entry.constant == LANDMARK),
        "preserve landmark occupied"
    );
    let id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|entry| entry.id)
        .max()
        .context("source landmark table")?
        .checked_add(1)
        .context("preserve landmark identifier")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id,
            constant: LANDMARK.into(),
            label: "MN_BOUNDARY_SAFARI".into(),
            name: "BOUNDARY PRESERVE".into(),
            x: 144,
            y: 8,
            region: "JOHTO".into(),
        });
    Ok(())
}
fn point(value: &Value) -> Result<[u16; 2]> {
    Ok([
        u16::try_from(value[0].as_u64().context("source x")?)?,
        u16::try_from(value[1].as_u64().context("source y")?)?,
    ])
}
fn reached(start: [u16; 2], allowed: &BTreeSet<[u16; 2]>) -> BTreeSet<[u16; 2]> {
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some([x, y]) = queue.pop_front() {
        for tile in [
            [x.saturating_sub(1), y],
            [x + 1, y],
            [x, y.saturating_sub(1)],
            [x, y + 1],
        ] {
            if allowed.contains(&tile) && seen.insert(tile) {
                queue.push_back(tile);
            }
        }
    }
    seen
}
fn validate_source(
    pack: &CompiledGamePack,
    blocks: &[u16],
    source: &Source,
    anchors: &Value,
) -> Result<()> {
    ensure!(
        source.retire_anchors.len() == 3
            && source.landmark_actors.len() == 1
            && source.decorations.len() == 16
            && source.signs.len()
                == if anchors["dimensions_metatiles"] == json!([64, 56]) {
                    18
                } else {
                    13
                },
        "retain source rangers, Dorothy, four native stone-ring camps and signs"
    );
    let [width, height] = point(&anchors["dimensions_metatiles"])?;
    ensure!(
        matches!([width, height], [48, 40] | [64, 56]),
        "unsupported preserve extent"
    );
    let kind = |tile| collision(pack, blocks, width, tile, "mn_boundary_safari");
    let mut bodies = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for actor in source
        .retire_anchors
        .iter()
        .chain(&source.landmark_actors)
        .chain(&source.decorations)
    {
        ensure!(
            ids.insert(actor.id.as_str())
                && bodies.insert(actor.tile)
                && kind(actor.tile)? == "FLOOR",
            "unsafe or duplicate source actor"
        );
        ensure!(
            actor.tile[0] % 64 > 0 && actor.tile[1] % 48 > 0,
            "actor on native map seam"
        );
    }
    let mut walking = BTreeSet::new();
    let mut surf = BTreeSet::new();
    let mut cuts = BTreeSet::new();
    for y in 0..height * 2 {
        for x in 0..width * 2 {
            let tile = [x, y];
            let terrain = kind(tile)?;
            if terrain == "CUT_TREE" {
                cuts.insert(tile);
            }
            if x == 0 || x == width * 2 - 1 || y == 0 || y == height * 2 - 1 {
                ensure!(
                    !matches!(
                        terrain,
                        "WATER" | "WHIRLPOOL" | "WATERFALL" | "FLOOR" | "TALL_GRASS"
                    ),
                    "unconnected preserve outer water/walking edge"
                );
            }
            if matches!(terrain, "FLOOR" | "TALL_GRASS" | "DOOR") && !bodies.contains(&tile) {
                walking.insert(tile);
                surf.insert(tile);
            }
            if terrain == "WATER" {
                surf.insert(tile);
            }
        }
    }
    for sign in &source.signs {
        ensure!(
            ids.insert(sign.id.as_str()) && kind(sign.tile)? == "WALL",
            "source sign must keep native wall anchor"
        );
    }
    let gate = &anchors["entry_gate"];
    let arrival = point(&anchors["entry_contract"]["arrival_tile"])?;
    let launch = point(&gate["launch_target"])?;
    ensure!(
        kind(arrival)? == "FLOOR" && reached(arrival, &walking).contains(&launch),
        "source arrival-to-launch path disconnected"
    );
    for (_, tile) in expansion::pending_guards(anchors)? {
        ensure!(
            kind(tile)? == "FLOOR" && !bodies.contains(&tile),
            "pending gate guard anchor obstructed"
        );
        surf.remove(&tile);
    }
    ensure!(
        !reached(arrival, &surf).contains(&launch),
        "closed pending expedition gate has a walking or Surf bypass"
    );
    ensure!(
        cuts == BTreeSet::from([[10, 31]]),
        "retain the sole purposeful native Cut gate"
    );
    let gates = anchors["cut_gates"]
        .as_array()
        .context("native Cut source")?;
    ensure!(
        gates.len() == 1 && point(&gates[0]["tile"])? == [10, 31],
        "Cut gate binding changed"
    );
    let index = 15 * usize::from(width) + 5;
    let original = blocks[index];
    let replacement = &pack.data.field_moves.cut.replacements["mn_boundary_safari"][&original];
    ensure!(
        replacement.variant == "tree" && replacement.replacement_block_id == 0x3f,
        "Cut must use actual native63-to3f rule"
    );
    let mut opened = blocks.to_vec();
    opened[index] = replacement.replacement_block_id;
    let target = point(&gates[0]["targets"][0])?;
    let before = reached(launch, &surf);
    ensure!(
        !before.contains(&target),
        "rare thicket has a Surf bypass before Cut"
    );
    let mut after = surf.clone();
    for y in 30..32 {
        for x in 10..12 {
            if matches!(
                collision(pack, &opened, width, [x, y], "mn_boundary_safari")?,
                "FLOOR" | "TALL_GRASS"
            ) && !bodies.contains(&[x, y])
            {
                after.insert([x, y]);
            }
        }
    }
    ensure!(
        reached(launch, &after).contains(&target),
        "native Cut replacement does not open declared thicket"
    );
    for portage in anchors["portages"].as_array().context("source portages")? {
        let west = point(&portage["west_land"])?;
        let east = point(&portage["east_land"])?;
        ensure!(
            kind(west)? == "FLOOR"
                && kind(east)? == "FLOOR"
                && kind(point(&portage["west_water"])?)? == "WATER"
                && kind(point(&portage["east_water"])?)? == "WATER",
            "unsafe native portage landing"
        );
        ensure!(
            reached(west, &walking).contains(&east),
            "native portage land path disconnected"
        );
        let b = portage["grass_bounds_core"]
            .as_array()
            .context("portage grass bounds")?;
        let bounds = b
            .iter()
            .map(|v| {
                v.as_u64()
                    .context("grass bound")
                    .and_then(|v| Ok(u16::try_from(v)?))
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(bounds.len() == 4, "portage grass bounds");
        let mut dry = walking.clone();
        for y in bounds[1]..bounds[3] {
            for x in bounds[0]..bounds[2] {
                ensure!(kind([x, y])? == "TALL_GRASS", "source portage lost grass");
                dry.remove(&[x, y]);
            }
        }
        ensure!(
            !reached(west, &dry).contains(&east),
            "native portage has a free dry bypass"
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "boundary_safari_modpack_tests.rs"]
mod tests;
