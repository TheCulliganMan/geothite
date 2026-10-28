//! Revise a simple installed cave's terrain without reinstalling its programs.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};
use serde_json::Value;

use crate::authored_route_design::Interior;
use crate::authored_route_modpack::{collision, validate_room};
use crate::{CompiledGamePack, MapModule};

/// Replaces terrain and its solid-wall border in a same-size installed cave.
///
/// Both sources must retain every field except rows, the solid-wall border and
/// the editorial fields `purpose`, `candidate_status` and `review_contract`.
/// This operation supports
/// ordinary residents, item balls and an already installed `HealParty` resident
/// whose exact source and native programs are certified before revision. No
/// service is installed or changed. Scripted trainers, keepers and passive
/// interior arrivals require a separate implementation. Native numeric identities,
/// all events/programs, connections, palettes, art, audio, catalogs, manifests,
/// HOME and recovery data remain unchanged. Only the map's blocks, solid-wall
/// border and attribute mirror, its uniquely owned encoded block payload and
/// the derived pack identity change.
///
/// Revised floor must be fully reachable with actors present. All ladders must
/// be the retained exits; borders retain only the original mouth aperture.
/// Every short native downward or lateral ledge must have a clear landing,
/// blocked reverse/sides and an ordinary return. This validates native geometry,
/// not visual quality, production
/// input, earned progression or compatibility with previously saved overrides.
///
/// # Errors
/// Returns an error for source drift, unsupported content, native preimage or
/// split-payload disagreement, shared block labels, unsafe revised geometry or
/// failed pack verification. Borrowed inputs are unchanged on failure.
pub fn revise_authored_cave_terrain(
    base: &CompiledGamePack,
    previous_room_json: &str,
    revised_room_json: &str,
) -> Result<CompiledGamePack> {
    let mut old_value: Value = serde_json::from_str(previous_room_json)?;
    let mut new_value: Value = serde_json::from_str(revised_room_json)?;
    let old: Interior = serde_json::from_value(old_value.clone())?;
    let new: Interior = serde_json::from_value(new_value.clone())?;
    for value in [&mut old_value, &mut new_value] {
        let fields = value.as_object_mut().context("cave room object")?;
        for field in [
            "rows",
            "border_block",
            "purpose",
            "candidate_status",
            "review_contract",
        ] {
            fields.remove(field);
        }
    }
    ensure!(
        old_value == new_value,
        "cave revision changes a retained source field"
    );
    ensure!(
        old.id.starts_with("MnR")
            && old
                .dimensions_metatiles
                .iter()
                .all(|&size| size > 0 && size <= 32)
            && old.passive_arrival.is_none()
            && old.keeper_tile.is_none()
            && old.signs.is_empty()
            && ["story_trainers", "story_objects"].iter().all(|key| {
                old_value
                    .get(key)
                    .is_none_or(|v| v.as_array().is_some_and(Vec::is_empty))
            }),
        "requires a bounded simple authored cave"
    );
    let previous = blocks(&old)?;
    let replacement = blocks(&new)?;
    ensure!(previous != replacement, "cave terrain is unchanged");
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let original = base.data.maps.get(&old.id).context("installed cave")?;
    let attributes = &original.attributes;
    let border = revised_border(base, original, &old, &new)?;
    ensure!(
        original.id == old.id
            && attributes.tileset_name == "cave"
            && attributes.environment.as_deref() == Some("CAVE")
            && [attributes.width, attributes.height] == old.dimensions_metatiles
            && attributes.connections.is_empty()
            && original.blocks == previous
            && base.data.map_attributes.get(&old.id) == Some(attributes),
        "installed cave identity, geometry or attributes differ from the preimage"
    );
    let label = attributes
        .blocks_label
        .as_deref()
        .context("native cave block label")?;
    ensure!(
        base.data.maps.iter().all(|(name, module)| name == &old.id
            || module.attributes.blocks_label.as_deref() != Some(label)),
        "cave block payload is shared"
    );
    let encoded = base
        .data
        .map_blocks
        .get(label)
        .context("encoded cave blocks")?;
    ensure!(
        crate::decode_base64_bytes(encoded)?
            .into_iter()
            .map(u16::from)
            .eq(previous),
        "encoded cave blocks disagree with the native preimage"
    );
    validate_native_source(base, original, &old)?;
    let mut revised = original.clone();
    revised.blocks = replacement;
    revised.attributes.border_block = border;
    validate_room(base, &revised, &new).context("revised cave event access")?;
    validate_terrain(base, &revised, &new)?;
    let bytes = revised
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut result = base.clone();
    result
        .data
        .map_blocks
        .insert(label.into(), crate::encode_base64_bytes(&bytes));
    result
        .data
        .map_attributes
        .insert(old.id.clone(), revised.attributes.clone());
    result.data.maps.insert(old.id, revised);
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

/// Keeps border repetition blocking while allowing the native solid-rock finish.
pub(crate) fn revised_border(
    base: &CompiledGamePack,
    module: &MapModule,
    old: &Interior,
    new: &Interior,
) -> Result<u8> {
    ensure!(
        old.border_block
            .is_none_or(|border| border == module.attributes.border_block),
        "cave border differs from its source preimage"
    );
    let border = new.border_block.unwrap_or(module.attributes.border_block);
    for tile in [[0, 0], [1, 0], [0, 1], [1, 1]] {
        ensure!(
            collision(base, &[u16::from(border)], 1, tile, "cave")? == "WALL",
            "cave border must be solid native rock in every quadrant"
        );
    }
    Ok(border)
}

pub(crate) fn blocks(room: &Interior) -> Result<Vec<u16>> {
    let [width, height] = room.dimensions_metatiles;
    ensure!(
        room.rows.len() == usize::from(height),
        "complete cave rows required"
    );
    let mut blocks = Vec::new();
    for row in &room.rows {
        let values = row
            .split_whitespace()
            .map(|word| u16::from_str_radix(word, 16).context("native cave block"))
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            values.len() == usize::from(width) && values.iter().all(|&value| value < 0x40),
            "native cave row or block extent"
        );
        blocks.extend(values);
    }
    Ok(blocks)
}

fn validate_native_source(
    base: &CompiledGamePack,
    module: &MapModule,
    room: &Interior,
) -> Result<()> {
    ensure!(
        module.events.warps.len() == room.exits.len()
            && module
                .events
                .warps
                .iter()
                .zip(&room.exits)
                .all(|(warp, &tile)| [warp.x, warp.y] == tile)
            && module.events.bg_events.is_empty()
            && module.events.coord_events.is_empty(),
        "native cave exits/events differ from the retained source"
    );
    // Use the established authoring code only to certify the source preimage.
    // None of its generated commands or indexes enter the revised pack.
    let mut expected = module.clone();
    expected.objects.clear();
    expected.scripts.clear();
    expected.script_text_bodies.clear();
    crate::authored_route_events::room_events(base, &mut expected, room)?;
    ensure!(
        expected.objects == module.objects,
        "native cave actor records differ from source"
    );
    let mut native_programs = module.scripts.clone();
    for label in [
        module.attributes.map_scripts_label.as_ref(),
        module.attributes.map_events_label.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        native_programs.remove(label);
    }
    ensure!(
        expected.scripts == native_programs,
        "native cave programs differ from source"
    );
    let mut split = base.data.clone();
    split.maps.remove(&room.id);
    let assembled = split.assemble_map_module_from_compiled_payloads(&room.id)?;
    let mut certified = module.clone();
    for label in [
        module.attributes.map_scripts_label.as_ref(),
        module.attributes.map_events_label.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        let payload = base
            .data
            .map_scripts
            .get(label)
            .context("native cave transport payload")?;
        certified
            .scripts
            .entry(label.clone())
            .or_insert_with(|| payload.clone());
    }
    ensure!(
        assembled == certified,
        "native cave split payloads/indexes drifted"
    );
    validate_room(base, module, room)
}

pub(crate) fn validate_terrain(
    base: &CompiledGamePack,
    module: &MapModule,
    room: &Interior,
) -> Result<()> {
    let [width, height] = room.dimensions_metatiles.map(|size| size * 2);
    let kind = |p| collision(base, &module.blocks, module.attributes.width, p, "cave");
    let bodies = module
        .objects
        .iter()
        .map(|object| [object.x, object.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        bodies.len() == module.objects.len(),
        "overlapping cave actors"
    );
    let mut floor = BTreeSet::new();
    let mut ladders = BTreeSet::new();
    let mut hops = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let at = [x, y];
            let permission = kind(at)?;
            if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                // Native cave07 has a two-tile mouth but one active warp. Its
                // adjacent Floor quadrant is retained exactly, not generalized
                // into permission to open other outer Floor or Water cells.
                let index =
                    usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2);
                let retained_mouth_floor = permission == "FLOOR"
                    && room
                        .exits
                        .iter()
                        .any(|exit| [exit[0] / 2, exit[1] / 2] == [x / 2, y / 2])
                    && base.data.maps[&room.id].blocks[index] == module.blocks[index];
                ensure!(
                    permission == "WALL"
                        || (permission == "LADDER" && room.exits.contains(&at))
                        || retained_mouth_floor,
                    "unconnected cave border is open at {at:?}: {permission}"
                );
            }
            match permission {
                "FLOOR" if !bodies.contains(&at) => {
                    floor.insert(at);
                }
                "LADDER" => {
                    ladders.insert(at);
                }
                "HOP_DOWN" => hops.push((at, Direction::Down, [0, 1])),
                "HOP_LEFT" => hops.push((at, Direction::Left, [-1, 0])),
                "HOP_RIGHT" => hops.push((at, Direction::Right, [1, 0])),
                "FLOOR" | "WALL" | "UP_WALL" | "WATER" => {}
                other => anyhow::bail!("unsupported cave collision {other}"),
            }
        }
    }
    ensure!(
        ladders == room.exits.iter().copied().collect(),
        "unpaired cave ladder"
    );
    let entry = room.clear_arrival.context("ordinary cave arrival")?;
    let seen = flood(entry, &floor);
    ensure!(seen == floor, "unreachable cave floor with actors present");
    let map =
        OverworldMapData::from_attributes(&module.id, &module.attributes, module.blocks.clone());
    let tileset = base.data.tileset_collision("cave")?;
    let occupied = module
        .objects
        .iter()
        .map(|object| {
            Ok(OccupiedTile {
                tile: TilePosition::new(i16::try_from(object.x)?, i16::try_from(object.y)?),
                object_identifier: object.object_identifier.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    for ([x, y], direction, [dx, dy]) in hops {
        let offset = |ox: i32, oy: i32| -> Result<[u16; 2]> {
            let nx = i32::from(x) + ox;
            let ny = i32::from(y) + oy;
            ensure!(
                nx > 0 && nx < i32::from(width) - 1 && ny > 0 && ny < i32::from(height) - 1,
                "cave hop touches the border"
            );
            Ok([u16::try_from(nx)?, u16::try_from(ny)?])
        };
        let approach = offset(-dx, -dy)?;
        let course = offset(dx, dy)?;
        let landing = offset(dx * 2, dy * 2)?;
        // Ordinary Floor may border the lip, as in native cave21 beside39.
        // The two perpendicular neighbors of the jumped-over Wall course must
        // stay closed: those are the cells that could bypass the drop sideways.
        let course_sides = [offset(dx - dy, dy + dx)?, offset(dx + dy, dy - dx)?];
        ensure!(
            floor.contains(&approach)
                && floor.contains(&landing)
                && kind(course)? == "WALL"
                && kind(course_sides[0])? == "WALL"
                && kind(course_sides[1])? == "WALL"
                && flood(landing, &floor).contains(&approach),
            "cave drop needs a clear ordinary return and blocked reverse/sides"
        );
        let mut player =
            PlayerMovementState::new(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?));
        ensure!(
            matches!(attempt_ledge_jump_with_occupied_tiles(&mut player, direction,
            &map, &tileset, StepOptions::default(), &occupied),
            LedgeJumpOutcome::Jumped { to, .. } if to == TilePosition::new(i16::try_from(landing[0])?, i16::try_from(landing[1])?)),
            "native cave ledge does not reach its landing"
        );
        let reverse = match direction {
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => {
                unreachable!("only native downward and lateral cave ledges are collected")
            }
        };
        player.facing = reverse;
        ensure!(
            matches!(
                attempt_step_with_occupied_tiles(
                    &mut player,
                    reverse,
                    &map,
                    &tileset,
                    StepOptions::default(),
                    &occupied
                ),
                StepOutcome::Blocked { .. }
            ),
            "native cave ledge permits reverse movement"
        );
    }
    Ok(())
}

fn flood(start: [u16; 2], floor: &BTreeSet<[u16; 2]>) -> BTreeSet<[u16; 2]> {
    if !floor.contains(&start) {
        return BTreeSet::new();
    }
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([start]);
    while let Some([x, y]) = todo.pop_front() {
        for next in [
            [x.saturating_sub(1), y],
            [x + 1, y],
            [x, y.saturating_sub(1)],
            [x, y + 1],
        ] {
            if floor.contains(&next) && seen.insert(next) {
                todo.push_back(next);
            }
        }
    }
    seen
}
