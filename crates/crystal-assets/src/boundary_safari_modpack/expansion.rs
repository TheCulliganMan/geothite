//! Transactional expansion of the installed preserve; existing services migrate.
use std::collections::BTreeMap;

use super::*;

const MANIFEST: &str = "minnesota-boundary-waters-expanded-terrain-v1";
const OLD_CHUNKS: [&str; 4] = [
    "MnBoundarySafariC00R00",
    "MnBoundarySafariC01R00",
    "MnBoundarySafariC00R01",
    "MnBoundarySafariC01R01",
];

/// Expands the installed48×40 preserve to its authored64×56 terrain.
///
/// Matches the previous source against installed terrain, actors and split
/// payloads before cloning. Existing map IDs, service programs, item identities,
/// recovery records and unrelated content are retained. Actor/sign coordinates
/// and the cabin's paired return are relocated by the new source. The two new
/// chunks use IDs6/7; cabin5 remains unchanged except its exterior warp target.
/// Expedition admission remains closed with real blocking guards. This does
/// not install Safari capture/session mechanics or regional arrival links.
///
/// # Errors
/// Returns an error on stale source/native data, changed owned identities,
/// occupied new IDs, unsafe geometry/gates/anchors or final pack verification.
/// The borrowed base is never modified, including on failure.
pub fn expand_boundary_safari_modpack(
    base: &CompiledGamePack,
    previous_layout_json: &str,
    expanded_layout_json: &str,
    expanded_zone_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let old = Layout::parse(previous_layout_json)?;
    let next = Layout::parse(expanded_layout_json)?;
    ensure!(
        old.id == "MN_BOUNDARY_WATERS_SAFARI"
            && old.dimensions_metatiles == [48, 40]
            && next.id == old.id
            && next.dimensions_metatiles == [64, 56]
            && old.compiled_tileset == "mn_boundary_safari"
            && next.compiled_tileset == old.compiled_tileset,
        "preserve expansion requires exact previous and next generations"
    );
    let old_anchors: Value = serde_json::from_str(previous_layout_json)?;
    let anchors: Value = serde_json::from_str(expanded_layout_json)?;
    let old_source: Source = serde_json::from_str(previous_layout_json)?;
    let source: Source = serde_json::from_str(expanded_layout_json)?;
    let zone: Value = serde_json::from_str(expanded_zone_json)?;
    ensure!(
        zone["id"] == old.id
            && zone["candidate_extent_metatiles"] == json!([64, 56])
            && zone["story_contract"]["paid_expedition_required_for_story"] == false,
        "retain optional preserve and free story access"
    );
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "expansion already installed"
    );
    let old_blocks = old.compile()?;
    validate_source(base, &old_blocks, &old_source, &old_anchors)?;
    validate_preimage(base, &old_blocks, &old_source, &old_anchors)?;
    // The exact cave12 stair extension is appended separately from source
    // geometry; the helper preserves every original imported glyph and block.
    let mut pack = crate::north_shore_art::append_native_cave_stairs(
        base,
        "mn_boundary_safari",
        0x84,
        &[0x12],
        "minnesota-boundary-waters-stone-stairs-v1",
    )?;
    let old_art = old_anchors["derived_blocks"]
        .as_array()
        .context("previous native art")?;
    let new_art = anchors["derived_blocks"]
        .as_array()
        .context("expanded native art")?;
    ensure!(
        new_art.len() == old_art.len() + 1
            && new_art[..old_art.len()] == old_art[..]
            && new_art.last().is_some_and(|stair| stair["id"] == "84"
                && stair["source_tileset"] == "cave"
                && stair["source_block"] == "12"
                && stair.get("palette_override").is_none()
                && stair.get("tile_row_order").is_none()
                && stair.get("tile_row_sources").is_none()
                && stair.get("collision_source_block").is_none())
            && anchors["tileset"] == old_anchors["tileset"],
        "preserve expansion may only append exact native cave12 stairs"
    );
    let blocks = next.compile()?;
    validate_source(&pack, &blocks, &source, &anchors)?;
    super::expansion_geometry::validate(&pack, &blocks, &anchors)?;
    let old_actors = actors_by_id(&old_source)?;
    let new_actors = actors_by_id(&source)?;
    ensure!(
        old_actors.keys().eq(new_actors.keys()),
        "preserve actor identities changed"
    );
    let old_signs = old_source
        .signs
        .iter()
        .map(|sign| (&sign.id, sign))
        .collect::<BTreeMap<_, _>>();
    let new_signs = source
        .signs
        .iter()
        .map(|sign| (&sign.id, sign))
        .collect::<BTreeMap<_, _>>();
    let added_signs = anchors["expansion_contract"]["added_sign_ids"]
        .as_array()
        .context("explicit added sign identities")?
        .iter()
        .map(|id| id.as_str().context("added sign identity"))
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(
        added_signs.len() == 5
            && added_signs
                .iter()
                .all(|id| !old_signs.keys().any(|old| old.as_str() == *id))
            && new_signs
                .keys()
                .map(|id| id.as_str())
                .collect::<BTreeSet<_>>()
                == old_signs
                    .keys()
                    .map(|id| id.as_str())
                    .chain(added_signs.iter().copied())
                    .collect(),
        "preserve sign identities changed beyond declared additions"
    );
    for id in &added_signs {
        let text = format!("{id}Text");
        ensure!(
            !base.data.map_scripts.contains_key(*id) && !base.data.map_scripts.contains_key(&text),
            "new preserve sign label occupied: {id}"
        );
    }
    for (id, before) in &old_actors {
        let after = new_actors[id];
        ensure!(
            before.sprite == after.sprite
                && before.lines == after.lines
                && before.facing == after.facing,
            "preserve actor service/text changed: {id}"
        );
    }
    for (id, before) in &old_signs {
        ensure!(
            before.lines == new_signs[id].lines,
            "preserve sign text changed: {id}"
        );
    }
    let mut modules = Vec::new();
    for row in 0..3u16 {
        for column in 0..2u16 {
            let name = chunk([column * 64, row * 48]);
            let mut module =
                authoring::blank(&pack, OLD_CHUNKS[0], &name, &map_constant(column, row))?;
            module.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
            module.attributes.width = 32;
            module.attributes.height = if row == 2 { 8 } else { 24 };
            module.blocks.clear();
            for y in row * 24..row * 24 + module.attributes.height {
                let offset = usize::from(y) * 64 + usize::from(column) * 32;
                module
                    .blocks
                    .extend_from_slice(&blocks[offset..offset + 32]);
            }
            for (direction, c, r) in [
                ("west", i32::from(column) - 1, i32::from(row)),
                ("east", i32::from(column) + 1, i32::from(row)),
                ("north", i32::from(column), i32::from(row) - 1),
                ("south", i32::from(column), i32::from(row) + 1),
            ] {
                if (0..2).contains(&c) && (0..3).contains(&r) {
                    module.attributes.connections.push(MapConnection {
                        direction: direction.into(),
                        target_map: chunk([u16::try_from(c)? * 64, u16::try_from(r)? * 48]),
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
    for (id, actor) in new_actors {
        let before = old_actors[&id];
        let old_module = &base.data.maps[&chunk(before.tile)];
        let mut object = old_module
            .objects
            .iter()
            .find(|object| object.object_identifier.as_deref() == Some(id.as_str()))
            .context("installed preserve actor")?
            .clone();
        object.x = actor.tile[0] % 64;
        object.y = actor.tile[1] % 48;
        let target = module_at(&mut modules, actor.tile)?;
        target.objects.push(object);
        migrate_scripts(old_module, target, &id)?;
    }
    for (id, sign) in new_signs {
        let target = module_at(&mut modules, sign.tile)?;
        if let Some(before) = old_signs.get(id) {
            let old_module = &base.data.maps[&chunk(before.tile)];
            let mut event = old_module
                .events
                .bg_events
                .iter()
                .find(|event| event.script == *id)
                .context("installed preserve sign")?
                .clone();
            event.x = sign.tile[0] % 64;
            event.y = sign.tile[1] % 48;
            target.events.bg_events.push(event);
            migrate_scripts(old_module, target, id)?;
        } else {
            let label = format!("{id}Text");
            target.scripts.insert(
                id.clone(),
                json!([authoring::command("jumptext", &[&label])]),
            );
            let lines = sign.lines.iter().map(String::as_str).collect::<Vec<_>>();
            authoring::text(target, &label, &lines);
            target.events.bg_events.push(BackgroundEvent {
                x: sign.tile[0] % 64,
                y: sign.tile[1] % 48,
                event_type: "BGEVENT_READ".into(),
                script: id.clone(),
            });
        }
    }
    for ((id, before), (_, after)) in pending_guards(&old_anchors)?
        .into_iter()
        .zip(pending_guards(&anchors)?)
    {
        let old_module = &base.data.maps[&chunk(before)];
        let mut object = old_module
            .objects
            .iter()
            .find(|object| object.object_identifier.as_deref() == Some(id))
            .context("installed closed-admission guard")?
            .clone();
        object.x = after[0] % 64;
        object.y = after[1] % 48;
        let target = module_at(&mut modules, after)?;
        target.objects.push(object);
        migrate_scripts(old_module, target, id)?;
    }
    relocate_cabin(base, &anchors, &mut modules)?;
    for module in &mut modules {
        authoring::finish(&pack, module)?;
    }
    for (column, map_id) in [(0, 6), (1, 7)] {
        let name = chunk([column * 64, 96]);
        ensure!(
            !pack.data.maps.contains_key(&name)
                && !pack
                    .data
                    .runtime_map_metadata
                    .values()
                    .any(|entry| entry.group_id == BOUNDARY_SAFARI_GROUP && entry.map_id == map_id),
            "new preserve identity occupied"
        );
        let module = modules
            .iter()
            .find(|module| module.id == name)
            .context("new preserve chunk")?;
        insert_empty_chunk(&mut pack, module, map_id)?;
    }
    for module in modules {
        replace_chunk(&mut pack, module)?;
    }
    let mut cabin = base
        .data
        .maps
        .get("MnBoundaryRootBeerCabin")
        .context("installed cabin")?
        .clone();
    let exterior = point(&anchors["root_beer_lady"]["door"]["tile"])?;
    let constant = map_constant(exterior[0] / 64, exterior[1] / 48);
    for warp in &mut cabin.events.warps {
        warp.target_map = constant.clone();
        warp.target_map_constant = constant.clone();
    }
    authoring::finish(&pack, &mut cabin)?;
    replace_chunk(&mut pack, cabin)?;
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

pub(super) fn pending_guards(anchors: &Value) -> Result<[(&'static str, [u16; 2]); 2]> {
    let tiles = anchors["entry_gate"]["gate_tiles"]
        .as_array()
        .context("entry gate tiles")?
        .iter()
        .map(point)
        .collect::<Result<Vec<_>>>()?;
    let bottom = tiles
        .iter()
        .map(|tile| tile[1])
        .max()
        .context("entry gate row")?;
    let mut guards = tiles
        .into_iter()
        .filter(|tile| tile[1] == bottom)
        .collect::<Vec<_>>();
    guards.sort();
    ensure!(
        guards.len() == 2 && guards[1][0] == guards[0][0] + 1,
        "two adjacent closed gate guards required"
    );
    Ok([
        ("BWS_PENDING_GATE_WEST", guards[0]),
        ("BWS_PENDING_GATE_EAST", guards[1]),
    ])
}

fn actors_by_id(source: &Source) -> Result<BTreeMap<String, &Actor>> {
    let mut actors = BTreeMap::new();
    for actor in source
        .retire_anchors
        .iter()
        .chain(&source.landmark_actors)
        .chain(&source.decorations)
    {
        ensure!(
            actors.insert(actor.id.clone(), actor).is_none(),
            "duplicate preserve actor identity"
        );
    }
    Ok(actors)
}

fn validate_preimage(
    base: &CompiledGamePack,
    blocks: &[u16],
    source: &Source,
    anchors: &Value,
) -> Result<()> {
    for (index, name) in OLD_CHUNKS.into_iter().enumerate() {
        let column = u16::try_from(index % 2)?;
        let row = u16::try_from(index / 2)?;
        let module = base
            .data
            .maps
            .get(name)
            .context("missing installed preserve chunk")?;
        ensure!(
            module.attributes.width == if column == 0 { 32 } else { 16 }
                && module.attributes.height == if row == 0 { 24 } else { 16 }
                && module.attributes.tileset_name == "mn_boundary_safari"
                && module.events.coord_events.is_empty(),
            "installed preserve attributes drifted"
        );
        let metadata = base.data.runtime_map_metadata_for_name(name)?;
        ensure!(
            metadata.group_id == BOUNDARY_SAFARI_GROUP && usize::from(metadata.map_id) == index + 1,
            "installed preserve numeric identity drifted"
        );
        let mut expected = Vec::new();
        for y in row * 24..row * 24 + module.attributes.height {
            let offset = usize::from(y) * 48 + usize::from(column) * 32;
            expected
                .extend_from_slice(&blocks[offset..offset + usize::from(module.attributes.width)]);
        }
        ensure!(
            module.blocks == expected,
            "stale preserve previous terrain: {name}"
        );
        let label = module
            .attributes
            .blocks_label
            .as_ref()
            .context("preserve blocks label")?;
        let bytes = expected
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(
            base.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&bytes)),
            "preserve encoded terrain differs: {name}"
        );
        validate_split(base, module)?;
    }
    let mut expected_ids = BTreeSet::new();
    for actor in actors_by_id(source)?.values() {
        let module = &base.data.maps[&chunk(actor.tile)];
        let object = module
            .objects
            .iter()
            .find(|object| object.object_identifier.as_deref() == Some(&actor.id))
            .context("previous source actor missing")?;
        ensure!(
            object.x == actor.tile[0] % 64
                && object.y == actor.tile[1] % 48
                && object.sprite == actor.sprite,
            "previous source actor anchor drifted: {}",
            actor.id
        );
        expected_ids.insert(actor.id.as_str());
    }
    for (id, tile) in pending_guards(anchors)? {
        let object = base.data.maps[&chunk(tile)]
            .objects
            .iter()
            .find(|object| object.object_identifier.as_deref() == Some(id))
            .context("previous native gate guard")?;
        ensure!(
            [object.x, object.y] == [tile[0] % 64, tile[1] % 48],
            "previous gate guard drifted"
        );
        expected_ids.insert(id);
    }
    let actual_ids = OLD_CHUNKS
        .iter()
        .flat_map(|name| &base.data.maps[*name].objects)
        .map(|object| {
            object
                .object_identifier
                .as_deref()
                .context("unowned preserve object")
        })
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(
        actual_ids == expected_ids,
        "unexpected installed preserve object"
    );
    for sign in &source.signs {
        let module = &base.data.maps[&chunk(sign.tile)];
        ensure!(
            module
                .events
                .bg_events
                .iter()
                .any(|event| event.script == sign.id
                    && [event.x, event.y] == [sign.tile[0] % 64, sign.tile[1] % 48]),
            "previous sign drifted: {}",
            sign.id
        );
    }
    let door = point(&anchors["root_beer_lady"]["door"]["tile"])?;
    let module = &base.data.maps[&chunk(door)];
    ensure!(
        module.events.warps.len() == 2
            && module.events.warps[0].index == 1
            && [module.events.warps[0].x, module.events.warps[0].y] == [door[0] % 64, door[1] % 48]
            && module
                .events
                .warps
                .iter()
                .all(
                    |warp| warp.target_map_constant == "MN_BOUNDARY_ROOT_BEER_CABIN"
                        && warp.target_warp_id == 3
                ),
        "previous cabin warp drifted"
    );
    let cabin = base
        .data
        .maps
        .get("MnBoundaryRootBeerCabin")
        .context("installed cabin missing")?;
    let metadata = base.data.runtime_map_metadata_for_name(&cabin.id)?;
    ensure!(
        metadata.group_id == BOUNDARY_SAFARI_GROUP && metadata.map_id == 5,
        "cabin identity drifted"
    );
    validate_split(base, cabin)?;
    Ok(())
}

fn validate_split(base: &CompiledGamePack, module: &MapModule) -> Result<()> {
    let mut scripts = BTreeMap::new();
    let mut npcs = BTreeMap::new();
    crate::insert_compiled_map_split_payloads(
        &mut scripts,
        &mut npcs,
        &module.id,
        module
            .attributes
            .map_scripts_label
            .as_deref()
            .context("map scripts label")?,
        module
            .attributes
            .map_events_label
            .as_deref()
            .context("map events label")?,
        module,
    )?;
    ensure!(
        scripts
            .iter()
            .all(|(key, value)| base.data.map_scripts.get(key) == Some(value))
            && npcs
                .iter()
                .all(|(key, value)| base.data.npcs.get(key) == Some(value)),
        "installed preserve typed/split payload disagreement: {}",
        module.id
    );
    Ok(())
}

fn migrate_scripts(before: &MapModule, after: &mut MapModule, id: &str) -> Result<()> {
    let mut count = 0;
    for (label, value) in &before.scripts {
        if label.starts_with(id) {
            ensure!(
                after.scripts.insert(label.clone(), value.clone()).is_none(),
                "duplicate migrated script {label}"
            );
            count += 1;
        }
    }
    ensure!(count > 0, "no owned service scripts for {id}");
    Ok(())
}

fn relocate_cabin(
    base: &CompiledGamePack,
    anchors: &Value,
    modules: &mut [MapModule],
) -> Result<()> {
    let door = &anchors["root_beer_lady"]["door"];
    let tile = point(&door["tile"])?;
    let passive = point(&door["return_tile"])?;
    ensure!(
        chunk(tile) == chunk(passive),
        "cabin mouth/passive arrival cross chunk"
    );
    let old_module = &base.data.maps[OLD_CHUNKS[0]];
    let target = module_at(modules, tile)?;
    target.events.warps = old_module.events.warps.clone();
    ensure!(
        target.events.warps.len() == 2,
        "existing paired cabin mouth"
    );
    for (warp, tile) in target.events.warps.iter_mut().zip([tile, passive]) {
        warp.x = tile[0] % 64;
        warp.y = tile[1] % 48;
    }
    Ok(())
}

fn insert_empty_chunk(pack: &mut CompiledGamePack, module: &MapModule, map_id: u16) -> Result<()> {
    let constant = module
        .attributes
        .map_constant
        .clone()
        .context("new preserve constant")?;
    let mut blank = authoring::blank(pack, OLD_CHUNKS[0], &module.id, &constant)?;
    blank.attributes = module.attributes.clone();
    blank.blocks = module.blocks.clone();
    authoring::finish(pack, &mut blank)?;
    let arrival = (0..blank.attributes.height * 2)
        .flat_map(|y| (0..64).map(move |x| [x, y]))
        .find(|tile| {
            collision(pack, &blank.blocks, 32, *tile, "mn_boundary_safari")
                .is_ok_and(|kind| kind == "FLOOR")
                && !module
                    .objects
                    .iter()
                    .any(|object| [object.x, object.y] == *tile)
        })
        .context("new preserve chunk needs a safe temporary validation tile")?;
    let temporary = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|spawn| spawn.identifier)
        .max()
        .context("recovery table")?
        .checked_add(1)
        .context("temporary spawn overflow")?;
    let extension = CompiledMapExtension {
        manifest_id: format!("{MANIFEST}-{map_id}"),
        map_name: module.id.clone(),
        map_constant: constant.clone(),
        metadata: RuntimeMapMetadata {
            name: module.id.clone(),
            constant: constant.clone(),
            group_name: GROUP.into(),
            group_id: BOUNDARY_SAFARI_GROUP,
            map_id,
            width: 32,
            height: module.attributes.height,
            environment: module
                .attributes
                .environment
                .clone()
                .context("preserve environment")?,
            phone_service: module.attributes.phone_service,
        },
        spawn_key: temporary.to_string(),
        spawn: RuntimeSpawnPoint {
            identifier: temporary,
            map_constant: constant,
            map_name: module.id.clone(),
            group_name: GROUP.into(),
            group_id: i16::try_from(BOUNDARY_SAFARI_GROUP)?,
            map_id: i16::try_from(map_id)?,
            tile_x: i16::try_from(arrival[0])?,
            tile_y: i16::try_from(arrival[1])?,
            metatile_x: i16::try_from(arrival[0] / 2)?,
            metatile_y: i16::try_from(arrival[1] / 2)?,
            subtile_x: i16::try_from(arrival[0] % 2)?,
            subtile_y: i16::try_from(arrival[1] % 2)?,
        },
        module: blank,
        wild_encounters: None,
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(pack, extension)?;
    pack.data
        .runtime_spawn_points
        .remove(&temporary.to_string());
    pack.data
        .pokegear_landmarks
        .map_to_landmark
        .insert(module.id.clone(), LANDMARK.into());
    Ok(())
}

fn replace_chunk(pack: &mut CompiledGamePack, module: MapModule) -> Result<()> {
    let constant = module
        .attributes
        .map_constant
        .as_ref()
        .context("preserve map constant")?;
    let metadata = pack
        .data
        .runtime_map_metadata
        .get_mut(constant)
        .context("preserved map metadata")?;
    metadata.width = module.attributes.width;
    metadata.height = module.attributes.height;
    pack.data.map_dimensions.insert(
        constant.clone(),
        json!({"width":metadata.width,"height":metadata.height}),
    );
    pack.data
        .map_attributes
        .insert(module.id.clone(), module.attributes.clone());
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    pack.data.map_blocks.insert(
        module
            .attributes
            .blocks_label
            .clone()
            .context("preserve block label")?,
        crate::encode_base64_bytes(&bytes),
    );
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &module.id,
        module
            .attributes
            .map_scripts_label
            .as_deref()
            .context("preserve scripts label")?,
        module
            .attributes
            .map_events_label
            .as_deref()
            .context("preserve events label")?,
        &module,
    )?;
    // Finish handles ordinary authoring events, but Dorothy also owns menus,
    // variables and native runtime service commands. Recompile every index
    // through the production split assembler after moving her script closure.
    pack.data.maps.remove(&module.id);
    let mut canonical = pack
        .data
        .assemble_map_module_from_compiled_payloads(&module.id)?;
    for label in [
        &module.attributes.map_scripts_label,
        &module.attributes.map_events_label,
    ] {
        canonical
            .scripts
            .remove(label.as_ref().context("owned transport section")?);
    }
    ensure!(
        canonical.objects == module.objects
            && canonical.events == module.events
            && canonical.blocks == module.blocks,
        "preserve reassembly changed authored geometry/events"
    );
    pack.data.maps.insert(module.id.clone(), canonical);
    Ok(())
}
