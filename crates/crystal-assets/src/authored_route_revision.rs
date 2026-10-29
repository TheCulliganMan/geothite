//! Replace only an installed authored route's terrain after exact source matching.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::ObjectEvent;

use crate::authored_route_design::{Facing, RouteDesign};
use crate::authored_route_events;
use crate::authored_route_layout::Layout;
use crate::{CompiledGamePack, MapModule};

/// Revises terrain on an already installed route without reinstalling its events.
///
/// Both layouts must describe the same native route, extent, tileset and clipped
/// chunk grid. Existing ragged edge chunks retain their installed dimensions;
/// omitted source cells must remain unchanged, fully blocking wall terrain.
/// R17's registered southern green-pine alias remains installed, with every old
/// and proposed block required to share the planning alias's native collision.
/// Raw `tileset`, `derived_blocks` and `source_pixels_modified` source
/// art fields must also remain equal; this operation cannot publish new art. Every installed chunk and its uniquely owned base64 block payload
/// must exactly match the compiled previous layout. The dossier certifies existing
/// actor/sign/door anchors and trainer/pickup identities; it cannot move those
/// events, change parties, install scripts or redesign regional story progression.
/// Revised terrain passes the same complete geometry checks as initial route
/// publication, including required grass, actor access, Cut/ledges and water edges.
///
/// Only existing exterior `MapModule::blocks`, their existing `map_blocks` entries
/// and the derived pack identity change. All map attributes/connections, rooms,
/// events, compiled scripts, encounters, catalogs, audio, presentation files,
/// report/manifests, HOME and recovery data are retained. No output is written.
/// This is a source/native revision operation, not controller or visual approval;
/// saved terrain overrides and earned regional progression need separate review.
///
/// # Errors
/// Returns an error for invalid input packs/source, layout or native chunk drift,
/// stale previous terrain, shared labels, encoded-block disagreement, stale native
/// actor/sign/door/room anchors, changed trainer/pickup identities, unsafe revised
/// geometry, block IDs outside Crystal's byte range or final verification failure.
/// The borrowed base pack and all source strings remain unchanged on failure.
pub fn revise_authored_route_terrain(
    base: &CompiledGamePack,
    previous_layout_json: &str,
    revised_layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    // Layout deliberately parses native geometry only. Retain the raw source-art
    // contract too, rather than silently ignoring a proposed renderer-only recipe.
    let previous_art: serde_json::Value =
        serde_json::from_str(previous_layout_json).context("previous source art contract")?;
    let revised_art: serde_json::Value =
        serde_json::from_str(revised_layout_json).context("revised source art contract")?;
    ensure!(
        ["tileset", "derived_blocks", "source_pixels_modified"]
            .iter()
            .all(|field| previous_art.get(field) == revised_art.get(field)),
        "terrain revision cannot change source art recipes"
    );
    let previous = Layout::parse(previous_layout_json).context("previous route layout")?;
    let revised = Layout::parse(revised_layout_json).context("revised route layout")?;
    let design = RouteDesign::parse(dossier_json).context("installed route dossier")?;
    validate_layout_contract(&previous, &revised, &design)?;
    let old_blocks = previous
        .compile()
        .context("compile previous route terrain")?;
    let new_blocks = revised.compile().context("compile revised route terrain")?;
    let chunks = installed_chunks(base, &design, &previous, &old_blocks)?;
    for chunk in &chunks {
        validate_chunk_collision_alias(base, &previous, &old_blocks, chunk)?;
        validate_chunk_collision_alias(base, &revised, &new_blocks, chunk)?;
        validate_clipped_padding(base, &previous, &old_blocks, &new_blocks, chunk)?;
    }
    validate_anchors(base, &design)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::authored_route_modpack::validate_geometry(
        base,
        &design,
        &new_blocks,
        &revised.compiled_tileset,
    )
    .context("revised complete route geometry")?;
    let mut pack = base.clone();
    for chunk in chunks {
        let replacement = clipped_blocks(&new_blocks, design.dimensions_metatiles[0], &chunk);
        let bytes = replacement
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("revised native block exceeds Crystal byte range")?;
        pack.data
            .maps
            .get_mut(&chunk.name)
            .context("validated existing route chunk")?
            .blocks = replacement;
        *pack
            .data
            .map_blocks
            .get_mut(&chunk.label)
            .context("validated existing encoded block payload")? =
            crate::encode_base64_bytes(&bytes);
    }
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

fn validate_layout_contract(old: &Layout, new: &Layout, design: &RouteDesign) -> Result<()> {
    ensure!(
        old.id == new.id
            && old.id == design.id
            && old.dimensions_metatiles == new.dimensions_metatiles
            && old.dimensions_metatiles == design.dimensions_metatiles
            && old.compiled_tileset == new.compiled_tileset
            && design.chunk_metatiles == [32, 24]
            && design.chunk_columns == old.dimensions_metatiles[0].div_ceil(32)
            && design.chunk_rows == old.dimensions_metatiles[1].div_ceil(24),
        "terrain revision cannot change route identity, extent, tileset or chunk grid"
    );
    Ok(())
}

#[derive(Debug)]
pub(crate) struct InstalledChunk {
    pub(crate) name: String,
    pub(crate) label: String,
    pub(crate) origin: [u16; 2],
    pub(crate) size: [u16; 2],
}

fn accepts_installed_tileset(layout: &Layout, chunk: &InstalledChunk, actual: &str) -> bool {
    actual == layout.compiled_tileset
        || (layout.id == "MN_R17"
            && layout.dimensions_metatiles == [32, 56]
            && layout.compiled_tileset == "mn_winter_outdoor"
            && chunk.name == "MnR17C00R02"
            && chunk.origin == [0, 48]
            && chunk.size == [32, 8]
            && actual == "mn_border_pines")
}

fn validate_chunk_collision_alias(
    pack: &CompiledGamePack,
    layout: &Layout,
    blocks: &[u16],
    chunk: &InstalledChunk,
) -> Result<()> {
    let actual = &pack
        .data
        .maps
        .get(&chunk.name)
        .context("installed seasonal chunk")?
        .attributes
        .tileset_name;
    if actual == &layout.compiled_tileset {
        return Ok(());
    }
    // Preserve R17's installed green southern transition. The complete-route
    // geometry check must agree with its actual native collision for every
    // old and proposed block; this cannot change an alias or install new art.
    let planned = &pack
        .data
        .tilesets
        .get(&layout.compiled_tileset)
        .context("planned seasonal tileset")?
        .collision;
    let native = &pack
        .data
        .tilesets
        .get(actual)
        .context("installed seasonal tileset")?
        .collision;
    for block in clipped_blocks(blocks, layout.dimensions_metatiles[0], chunk)
        .into_iter()
        .collect::<BTreeSet<_>>()
    {
        let key = format!("{block:02x}");
        ensure!(
            planned.get(&key).is_some() && planned.get(&key) == native.get(&key),
            "{}: planned/native seasonal collision differs for block {key}",
            chunk.name
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "border_pines_revision_tests.rs"]
mod border_pines_tests;

pub(crate) fn clipped_blocks(blocks: &[u16], width: u16, chunk: &InstalledChunk) -> Vec<u16> {
    let mut result = Vec::with_capacity(usize::from(chunk.size[0]) * usize::from(chunk.size[1]));
    for y in chunk.origin[1]..chunk.origin[1] + chunk.size[1] {
        let start = usize::from(y) * usize::from(width) + usize::from(chunk.origin[0]);
        result.extend_from_slice(&blocks[start..start + usize::from(chunk.size[0])]);
    }
    result
}

fn validate_clipped_padding(
    pack: &CompiledGamePack,
    layout: &Layout,
    before: &[u16],
    after: &[u16],
    chunk: &InstalledChunk,
) -> Result<()> {
    let [width, height] = layout.dimensions_metatiles;
    for y in chunk.origin[1]..(chunk.origin[1] + 24).min(height) {
        for x in chunk.origin[0]..(chunk.origin[0] + 32).min(width) {
            if x < chunk.origin[0] + chunk.size[0] && y < chunk.origin[1] + chunk.size[1] {
                continue;
            }
            let index = usize::from(y) * usize::from(width) + usize::from(x);
            ensure!(
                before[index] == after[index],
                "{}: terrain revision changes omitted ragged padding at {x},{y}",
                chunk.name
            );
            for dy in 0..2 {
                for dx in 0..2 {
                    ensure!(
                        crate::authored_route_modpack::collision(
                            pack,
                            before,
                            width,
                            [x * 2 + dx, y * 2 + dy],
                            &layout.compiled_tileset,
                        )? == "WALL",
                        "{}: omitted ragged padding is traversable at {x},{y}",
                        chunk.name
                    );
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn installed_chunks(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    layout: &Layout,
    old_blocks: &[u16],
) -> Result<Vec<InstalledChunk>> {
    let mut chunks = Vec::new();
    for row in 0..design.chunk_rows {
        for column in 0..design.chunk_columns {
            let name = authored_route_events::chunk(&design.map_prefix, [column * 64, row * 48]);
            let module = pack
                .data
                .maps
                .get(&name)
                .with_context(|| format!("missing installed {name}"))?;
            let label = module
                .attributes
                .blocks_label
                .as_deref()
                .context("installed block label")?;
            let chunk = InstalledChunk {
                name: name.clone(),
                label: label.into(),
                origin: [column * 32, row * 24],
                size: [module.attributes.width, module.attributes.height],
            };
            ensure!(
                module.id == name
                    && chunk.size[0] > 0
                    && chunk.size[1] > 0
                    && chunk.size[0] <= (layout.dimensions_metatiles[0] - column * 32).min(32)
                    && chunk.size[1] <= (layout.dimensions_metatiles[1] - row * 24).min(24)
                    && accepts_installed_tileset(layout, &chunk, &module.attributes.tileset_name)
                    && module.attributes.map_constant.as_deref()
                        == Some(authored_route_events::constant(&name).as_str())
                    && pack.data.map_attributes.get(&name) == Some(&module.attributes),
                "{name}: installed route identity/extent/tileset/attributes drift"
            );
            validate_clipped_padding(pack, layout, old_blocks, old_blocks, &chunk)?;
            let section_labels = [
                Some(label),
                module.attributes.map_scripts_label.as_deref(),
                module.attributes.map_events_label.as_deref(),
            ];
            ensure!(
                section_labels.into_iter().collect::<BTreeSet<_>>().len() == 3,
                "{name}: native section labels must be distinct"
            );
            for owned_label in section_labels {
                let owned_label = owned_label
                    .filter(|v| !v.is_empty())
                    .context("installed native section label")?;
                ensure!(
                    pack.data.maps.iter().all(|(other_name, other)| {
                        other_name == &name
                            || ![
                                other.attributes.blocks_label.as_deref(),
                                other.attributes.map_scripts_label.as_deref(),
                                other.attributes.map_events_label.as_deref(),
                            ]
                            .contains(&Some(owned_label))
                    }),
                    "{name}: native section label {owned_label} is shared by another map"
                );
            }
            for script in module
                .scripts
                .keys()
                .chain(module.script_text_bodies.keys())
            {
                ensure!(
                    pack.data.maps.iter().all(|(owner, other)| owner == &name
                        || (!other.scripts.contains_key(script)
                            && !other.script_text_bodies.contains_key(script))),
                    "{name}: native script label {script} is shared by another map"
                );
            }
            let expected = clipped_blocks(old_blocks, layout.dimensions_metatiles[0], &chunk);
            ensure!(
                module.blocks == expected,
                "{name}: installed blocks do not match previous layout"
            );
            let encoded = pack
                .data
                .map_blocks
                .get(label)
                .context("missing installed encoded blocks")?;
            let bytes =
                crate::decode_base64_bytes(encoded).context("decode installed route blocks")?;
            ensure!(
                bytes.into_iter().map(u16::from).eq(expected),
                "{name}: encoded blocks do not match previous layout"
            );
            chunks.push(chunk);
        }
    }
    let expected: BTreeSet<_> = chunks.iter().map(|c| c.name.as_str()).collect();
    let prefix = format!("{}C", design.map_prefix);
    for name in pack.data.maps.keys() {
        // Rooms such as MnR17Cave1F share the prefix but are not exterior chunks.
        let exterior = name.strip_prefix(&prefix).is_some_and(|suffix| {
            let bytes = suffix.as_bytes();
            bytes.len() == 5
                && bytes[2] == b'R'
                && bytes[..2].iter().chain(&bytes[3..]).all(u8::is_ascii_digit)
        });
        ensure!(
            !exterior || expected.contains(name.as_str()),
            "unexpected installed route chunk outside source grid"
        );
    }
    Ok(chunks)
}

fn native_actor<'a>(
    pack: &'a CompiledGamePack,
    design: &RouteDesign,
    id: &str,
    tile: [u16; 2],
) -> Result<(&'a MapModule, &'a ObjectEvent)> {
    let name = authored_route_events::chunk(&design.map_prefix, tile);
    let module = pack.data.maps.get(&name).context("source actor chunk")?;
    let matches = pack
        .data
        .maps
        .values()
        .flat_map(|m| m.objects.iter().map(move |o| (m, o)))
        .filter(|(_, o)| o.object_identifier.as_deref() == Some(id))
        .collect::<Vec<_>>();
    ensure!(
        matches.len() == 1,
        "{id}: native actor identity missing or ambiguous"
    );
    let (owner, object) = matches[0];
    ensure!(
        owner.id == name
            && [object.x, object.y] == authored_route_events::local(tile)
            && object.script == id
            && module.scripts.contains_key(id),
        "{id}: source/native actor anchor or script differs"
    );
    ensure!(
        pack.data
            .maps
            .values()
            .all(|m| m.id == name || !m.scripts.contains_key(id)),
        "{id}: native actor script label is shared by another map"
    );
    Ok((module, object))
}

fn facing(object: &ObjectEvent, sprite: &str, direction: Facing) -> Result<()> {
    ensure!(
        object.sprite == sprite && object.spritemovedata == direction.movement(),
        "{}: source/native actor sprite or facing differs",
        object.script
    );
    Ok(())
}

pub(crate) fn validate_anchors(pack: &CompiledGamePack, design: &RouteDesign) -> Result<()> {
    let actor_ids = design
        .trainers
        .iter()
        .map(|v| v.id.as_str())
        .chain(design.residents.iter().map(|v| v.id.as_str()))
        .chain(design.items.iter().map(|v| v.id.as_str()))
        .collect::<BTreeSet<_>>();
    ensure!(
        actor_ids.len() == design.trainers.len() + design.residents.len() + design.items.len(),
        "source route actor IDs must be unique"
    );
    for row in 0..design.chunk_rows {
        for column in 0..design.chunk_columns {
            let name = authored_route_events::chunk(&design.map_prefix, [column * 64, row * 48]);
            ensure!(
                pack.data.maps[&name].objects.iter().all(|object| object
                    .object_identifier
                    .as_deref()
                    .is_some_and(|id| actor_ids.contains(id))),
                "{name}: installed actor omitted from route dossier"
            );
        }
    }
    for trainer in &design.trainers {
        let (module, object) = native_actor(pack, design, &trainer.id, trainer.tile)?;
        facing(object, &trainer.sprite, trainer.facing)?;
        ensure!(
            object.object_type == "OBJECTTYPE_TRAINER"
                && object.radius == u16::from(trainer.sight_range),
            "{}: source/native trainer sight differs",
            trainer.id
        );
        let battle = module
            .trainer_scripts
            .get(&trainer.id)
            .context("existing native trainer request")?;
        let party = pack
            .data
            .trainers
            .trainers
            .get(&trainer.id)
            .context("existing native trainer party")?;
        ensure!(
            battle.trainer_id == trainer.id
                && battle.trainer_class == trainer.trainer_class
                && battle.event_flag == trainer.defeated_flag
                && party.trainer_class == trainer.trainer_class
                && party.name == trainer.display_name.to_ascii_uppercase()
                && party.party.len() == trainer.party.len()
                && party
                    .party
                    .iter()
                    .zip(&trainer.party)
                    .all(|(a, b)| a.species == b.species && a.level == b.level),
            "{}: source/native trainer identity, flag or party differs",
            trainer.id
        );
    }
    for resident in &design.residents {
        let (_, object) = native_actor(pack, design, &resident.id, resident.tile)?;
        facing(object, &resident.sprite, resident.facing)?;
    }
    for item in &design.items {
        let (module, object) = native_actor(pack, design, &item.id, item.tile)?;
        ensure!(
            object.object_type == "OBJECTTYPE_ITEMBALL" && object.event_flag == item.flag,
            "{}: source/native pickup event differs",
            item.id
        );
        let matches = module
            .script_field_pickups
            .iter()
            .filter(|p| p.source_script == item.id)
            .collect::<Vec<_>>();
        ensure!(
            matches.len() == 1
                && matches[0].item_id.as_deref() == Some(item.item.as_str())
                && matches[0].quantity == item.quantity
                && matches[0].event_flag.as_deref() == Some(item.flag.as_str()),
            "{}: source/native pickup identity or quantity differs",
            item.id
        );
    }
    for sign in &design.signs {
        let name = authored_route_events::chunk(&design.map_prefix, sign.tile);
        let matches = pack
            .data
            .maps
            .values()
            .flat_map(|m| m.events.bg_events.iter().map(move |event| (m, event)))
            .filter(|(_, event)| event.script == sign.id)
            .collect::<Vec<_>>();
        ensure!(
            matches.len() == 1
                && matches[0].0.id == name
                && [matches[0].1.x, matches[0].1.y] == authored_route_events::local(sign.tile),
            "{}: source/native sign anchor differs",
            sign.id
        );
    }
    for door in &design.doors {
        let name = authored_route_events::chunk(&design.map_prefix, door.tile);
        let module = pack.data.maps.get(&name).context("native door chunk")?;
        let target = pack
            .data
            .maps
            .get(&door.target)
            .context("installed door destination")?
            .attributes
            .map_constant
            .as_deref()
            .context("installed door destination constant")?;
        for (index, tile, target_index) in [(door.warp_index, door.tile, door.target_warp)]
            .into_iter()
            .chain(
                door.return_warp_index
                    // Ordinary passive FLOOR arrivals use destination 1;
                    // an installed custom room can declare its exact target.
                    .map(|index| {
                        (
                            index,
                            door.arrival_tile,
                            door.return_target_warp.unwrap_or(1),
                        )
                    }),
            )
        {
            let warp = module
                .events
                .warps
                .iter()
                .find(|w| w.index == index)
                .context("native exterior warp")?;
            ensure!(
                [warp.x, warp.y] == authored_route_events::local(tile)
                    && warp.target_map == target
                    && warp.target_map_constant == target
                    && warp.target_warp_id == target_index,
                "{name}: source/native door or passive return {index} differs from {target}/{target_index}"
            );
        }
    }
    for room in &design.interiors {
        let module = pack
            .data
            .maps
            .get(&room.id)
            .context("existing route room")?;
        let rows = room
            .rows
            .iter()
            .flat_map(|row| row.split_whitespace())
            .map(|token| u16::from_str_radix(token, 16).context("existing room block"))
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            [module.attributes.width, module.attributes.height] == room.dimensions_metatiles
                && module.blocks == rows,
            "source/native room geometry differs"
        );
        crate::authored_route_modpack::validate_room(pack, module, room)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::{Value, json};

    use super::*;

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("real repository root")
    }

    fn source(variable: &str, fallback: &str) -> String {
        let path = std::env::var_os(variable)
            .map(PathBuf::from)
            .unwrap_or_else(|| root().join(fallback));
        std::fs::read_to_string(path).expect("actual route source input")
    }

    fn previous() -> String {
        source(
            "CRYSTAL_AUTHORED_ROUTE_REVISION_BEFORE_LAYOUT",
            "planning/minnesota/routes/MN_R08.layout.json",
        )
    }

    fn dossier() -> String {
        source(
            "CRYSTAL_AUTHORED_ROUTE_REVISION_DOSSIER",
            "planning/minnesota/routes/MN_R08.json",
        )
    }

    fn base() -> &'static CompiledGamePack {
        static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
        PACK.get_or_init(|| {
            if let Some(path) = std::env::var_os("CRYSTAL_AUTHORED_ROUTE_REVISION_TEST_PACK") {
                let path = PathBuf::from(path)
                    .canonicalize()
                    .expect("actual completed installed-route pack");
                return crate::read_verified_compiled_game_pack(path)
                    .expect("verified completed native pack");
            }
            // Default tests install current source through the production builder,
            // so an authorized editor revision cannot stale this native fixture.
            let path = root()
                .join("content-packs/text-tui.crystalpack")
                .canonicalize()
                .expect("completed standard TUI pack");
            let pack = crate::read_verified_compiled_game_pack(path)
                .expect("verified standard native content");
            crate::build_northern_lakes_modpack(&pack, &previous(), &dossier())
                .expect("current route installed through the production builder")
        })
    }

    fn with_point(layout: &str, point: [u16; 2], block: &str) -> String {
        let mut value: Value = serde_json::from_str(layout).expect("native editor source");
        value["layers"]
            .as_array_mut()
            .expect("authored strokes")
            .push(json!({
                "name":"Focused terrain revision fixture", "kind":"point",
                "position":point, "block":block
            }));
        serde_json::to_string(&value).expect("updated source")
    }

    fn revised() -> String {
        // One source FLOOR->lawn block in the southern ordinary junction;
        // native collision remains FLOOR. No actor, grass, gate or port moves.
        with_point(&previous(), [13, 44], "02")
    }

    #[test]
    fn authored_route_revision_ragged_padding_must_be_unchanged_walls() {
        let mut layout = Layout::parse(&previous()).expect("layout");
        layout.dimensions_metatiles = [4, 4];
        let chunk = InstalledChunk {
            name: "RaggedEdge".into(),
            label: "RaggedEdgeBlocks".into(),
            origin: [0, 0],
            size: [3, 4],
        };
        let walls = vec![0x05; 16];
        validate_clipped_padding(base(), &layout, &walls, &walls, &chunk)
            .expect("unchanged native forest may pad the bounding rectangle");
        let mut traversable = walls.clone();
        traversable[7] = 0x01;
        assert!(
            validate_clipped_padding(base(), &layout, &traversable, &traversable, &chunk)
                .expect_err("omitted source floor would invent a walking bypass")
                .to_string()
                .contains("padding is traversable")
        );
        let mut changed = walls.clone();
        changed[7] = 0x0a;
        assert!(
            validate_clipped_padding(base(), &layout, &walls, &changed, &chunk)
                .expect_err("even blocking changes outside native terrain must be rejected")
                .to_string()
                .contains("changes omitted ragged padding")
        );
    }

    #[test]
    fn authored_route_revision_preserves_every_nonterrain_field_and_strict_wire() {
        let base = base();
        let previous = previous();
        let revised = revised();
        let result = revise_authored_route_terrain(base, &previous, &revised, &dossier())
            .expect("bounded installed native terrain revision");
        assert_ne!(result.identity, base.identity);
        let design = RouteDesign::parse(&dossier()).expect("route design");
        let old = Layout::parse(&previous).expect("previous layout");
        let old_blocks = old.compile().expect("previous blocks");
        let chunks =
            installed_chunks(base, &design, &old, &old_blocks).expect("installed old terrain");
        let mut normalized = result.clone();
        for chunk in chunks {
            normalized
                .data
                .maps
                .get_mut(&chunk.name)
                .expect("same chunk")
                .blocks = base.data.maps[&chunk.name].blocks.clone();
            normalized.data.map_blocks.insert(
                chunk.label.clone(),
                base.data.map_blocks[&chunk.label].clone(),
            );
        }
        normalized.identity = base.identity.clone();
        assert_eq!(
            &normalized, base,
            "only exterior blocks, their encoded payloads and identity may change"
        );
        assert!(
            revise_authored_route_terrain(&result, &previous, &revised, &dossier()).is_err(),
            "the previous source cannot be reused after a real revision"
        );
        let path = std::env::temp_dir().join(format!(
            "geothite-route-revision-{}-{}.crystalpack",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        result
            .write_preserving_storage(&path)
            .expect("actual complete CBOR output");
        let reloaded =
            crate::read_verified_compiled_game_pack(&path).expect("strict native readback");
        assert_eq!(
            result, reloaded,
            "encoded and typed terrain survives actual pack IO"
        );
        std::fs::remove_file(path).expect("remove temporary pack, never render history");
    }

    #[test]
    fn authored_route_revision_rejects_stale_previous_terrain() {
        let stale = with_point(&previous(), [0, 0], "02");
        assert!(
            revise_authored_route_terrain(base(), &stale, &revised(), &dossier())
                .expect_err("installed forest cannot match claimed previous lawn")
                .to_string()
                .contains("installed blocks do not match previous layout")
        );
    }

    #[test]
    fn authored_route_revision_rejects_identity_extent_tileset_and_grid_drift() {
        let previous = Layout::parse(&previous()).expect("layout");
        for mutation in 0..4 {
            let mut revised = Layout::parse(&self::previous()).expect("layout");
            let mut design = RouteDesign::parse(&dossier()).expect("dossier");
            match mutation {
                0 => revised.id = "MN_R07".into(),
                1 => revised.dimensions_metatiles[0] -= 1,
                2 => revised.compiled_tileset = "johto".into(),
                3 => design.chunk_rows -= 1,
                _ => unreachable!("four explicit contract mutations"),
            }
            assert!(validate_layout_contract(&previous, &revised, &design).is_err());
        }
    }

    #[test]
    fn authored_route_revision_rejects_encoded_disagreement_and_shared_labels() {
        let design = RouteDesign::parse(&dossier()).expect("dossier");
        let layout = Layout::parse(&previous()).expect("layout");
        let blocks = layout.compile().expect("source terrain");
        let first_chunk = format!("{}C00R00", design.map_prefix);
        let label = base().data.maps[&first_chunk]
            .attributes
            .blocks_label
            .as_ref()
            .expect("blocks label");
        let mut corrupt = base().clone();
        let mut bytes =
            crate::decode_base64_bytes(&corrupt.data.map_blocks[label]).expect("native bytes");
        bytes[0] ^= 1;
        corrupt
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
        assert!(
            installed_chunks(&corrupt, &design, &layout, &blocks)
                .expect_err("actual encoded/native mismatch")
                .to_string()
                .contains("encoded blocks do not match")
        );
        let mut alias = base().clone();
        alias
            .data
            .maps
            .get_mut("NewBarkTown")
            .expect("original map")
            .attributes
            .blocks_label = Some(label.clone());
        assert!(
            installed_chunks(&alias, &design, &layout, &blocks)
                .expect_err("unrelated map must not share a revised label")
                .to_string()
                .contains("shared by another map")
        );
    }

    #[test]
    fn authored_route_revision_rejects_stale_actor_and_sign_coordinates() {
        for field in ["residents", "signs"] {
            let mut value: Value = serde_json::from_str(&dossier()).expect("dossier");
            let x = value[field][0]["tile"][0]
                .as_u64()
                .expect("native coordinate");
            value[field][0]["tile"][0] = json!(x + 1);
            let changed = serde_json::to_string(&value).expect("stale source");
            let error = revise_authored_route_terrain(base(), &previous(), &revised(), &changed)
                .expect_err("source must describe actual installed event anchors")
                .to_string();
            assert!(
                error.contains("source/native actor anchor")
                    || error.contains("source/native sign anchor"),
                "{error}"
            );
        }
    }

    #[test]
    fn authored_route_revision_rejects_render_only_source_art_drift() {
        let previous = previous();
        for (field, changed) in [
            ("tileset", json!("kanto")),
            (
                "derived_blocks",
                json!({"80": {"source_tileset": "kanto", "source_block": "77"}}),
            ),
            ("source_pixels_modified", json!(true)),
        ] {
            let mut value: Value = serde_json::from_str(&previous).expect("actual source layout");
            let compiled_tileset = value["compiled_tileset"].clone();
            value[field] = changed;
            assert_eq!(value["compiled_tileset"], compiled_tileset);
            let revised = serde_json::to_string(&value).expect("render-only art mutation");
            let error = revise_authored_route_terrain(base(), &previous, &revised, &dossier())
                .expect_err("renderer recipes cannot drift from retained native art");
            assert_eq!(
                error.to_string(),
                "terrain revision cannot change source art recipes"
            );
        }
    }

    #[test]
    fn authored_route_revision_rejects_actual_required_grass_damage() {
        let design = RouteDesign::parse(&dossier()).expect("design");
        let [x, y, _, _] = design.required_grass_crossings[0].bounds_core;
        let damaged = with_point(&previous(), [x / 2, y / 2], "01");
        let error = revise_authored_route_terrain(base(), &previous(), &damaged, &dossier())
            .expect_err("real habitat contract must survive a terrain-only edit");
        assert!(format!("{error:#}").contains("required grass crossing contains another terrain"));
    }
}
