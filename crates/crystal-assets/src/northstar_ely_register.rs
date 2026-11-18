//! The team register and family photograph at the end of the Ely League.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::grand_marais_modpack::authoring::{command, text};
use crate::{
    CompiledGamePack, CompiledTilesetExtension, MapModule, authored_town_geometry,
    northstar_cave_geometry,
};
use crate::{northstar_lair_authoring, northstar_story_revision, northstar_trail_caches};

const HALL: &str = "MnLeagueHallOfFame";
const GALLERY: &str = "mn_ely_register";
const REGISTER: &str = "MnElyReadRegister";
const RECORDER: &str = "MnElyRecordVictory";
const PHOTO: &str = "MnElyReadFamilyPortrait";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Gallery {
    tiles: BTreeMap<char, u16>,
    rows: [String; 14],
    palettes: [u8; 24],
}

fn install_oak_tiles(pack: &mut CompiledGamePack) -> Result<()> {
    let base_file = |name: &str| {
        pack.runtime_files
            .get(name)
            .cloned()
            .with_context(|| format!("Ely oak source {name}"))
    };
    let source = CompiledTilesetExtension {
        manifest_id: "mn-ely-register".into(),
        tileset_id: GALLERY.into(),
        behavior_source_tileset_id: None,
        definition: pack.data.tilesets["traditional_house"].clone(),
        metatiles: base_file("data/tilesets/traditional_house_metatiles.bin")?,
        tile_graphics_png: base_file("gfx/tilesets/traditional_house.png")?,
        tile_graphics_2bpp: base_file("gfx/tilesets/traditional_house.2bpp")?,
    };
    crate::validate_compiled_tileset_extension(&source)?;
    crate::insert_compiled_tileset_extension(pack, source)?;
    pack.runtime_files.insert(
        format!("gfx/tilesets/{GALLERY}.pal"),
        include_bytes!("../../../planning/minnesota/revisions/ely-register-v01/hall.pal").to_vec(),
    );
    Ok(())
}

// Four player-sized pieces make up each native block. The room changes its
// furniture and floor but retains the exact collision of every old square.
fn compose(pack: &mut CompiledGamePack, room: &mut MapModule, design: &Gallery) -> Result<()> {
    let path = format!("data/tilesets/{GALLERY}_metatiles.bin");
    let mut blocks = pack
        .runtime_files
        .get(&path)
        .context("native oak blocks")?
        .clone();
    let old_collision = &pack.data.tilesets["ice_path"].collision;
    let mut surface = pack.data.tilesets[GALLERY].clone();
    for (i, &palette) in design.palettes.iter().enumerate() {
        ensure!(palette < 8, "gallery tile needs a native palette index");
        let block = 0x40 + i / 4;
        let piece = i % 4;
        let first = block * 16 + (piece % 2) * 2 + (piece / 2) * 8;
        for step in [0, 1, 4, 5] {
            let &glyph = blocks
                .get(first + step)
                .context("gallery furnishing glyph")?;
            ensure!(
                glyph >= 0x80 && !blocks[..0x40 * 16].contains(&glyph),
                "gallery furnishing must own its glyph"
            );
            surface.palette_map[usize::from(glyph)] = 8 | palette;
        }
    }
    ensure!(
        (0x40 + design.palettes.len() / 4) * 16 == blocks.len()
            && room.blocks.len() == 35
            && design
                .rows
                .iter()
                .all(|row| row.len() == 14 && row.is_ascii()),
        "expected the five-by-seven Ely hall"
    );
    let mut plan = Vec::with_capacity(49);
    for i in 0..49 {
        let x = (i % 7) * 2;
        let y = (i / 7) * 2;
        let mut collision = Vec::with_capacity(4);
        let mut assembled = [0_u8; 16];
        for q in 0..4 {
            let key = char::from(design.rows[y + q / 2].as_bytes()[x + q % 2]);
            let kind = if x < 10 {
                let previous = room.blocks[(y / 2) * 5 + x / 2];
                old_collision[&format!("{previous:02x}")][q].as_str()
            } else if matches!(key, 'a' | 'b') {
                "FLOOR"
            } else {
                "WALL"
            };
            collision.push(kind.to_owned());
            let piece = usize::from(*design.tiles.get(&key).context("gallery floorplan symbol")?);
            let src = piece / 4 * 16 + (piece % 2) * 2 + (piece % 4 / 2) * 8;
            let to = (q % 2) * 2 + (q / 2) * 8;
            for d in [0, 1, 4, 5] {
                assembled[to + d] = *blocks.get(src + d).context("native gallery quadrant")?;
            }
        }
        let existing = blocks
            .as_chunks::<16>()
            .0
            .iter()
            .enumerate()
            .position(|(id, bytes)| {
                bytes == &assembled
                    && surface.collision.get(&format!("{id:02x}")) == Some(&collision)
            });
        let id = if let Some(id) = existing {
            id
        } else {
            let id = blocks.len() / 16;
            ensure!(
                id < 256
                    && surface
                        .collision
                        .insert(format!("{id:02x}"), collision)
                        .is_none(),
                "gallery block capacity"
            );
            blocks.extend(assembled);
            id
        };
        plan.push(u16::try_from(id)?);
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{GALLERY}_palette_map.json"),
        serde_json::to_vec(&surface.palette_map)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{GALLERY}.json"),
        serde_json::to_vec(&surface.collision)?,
    );
    pack.runtime_files.insert(path, blocks);
    pack.data.tilesets.insert(GALLERY.into(), surface);
    room.attributes.tileset_name = GALLERY.into();
    room.attributes.width = 7;
    room.blocks = plan;
    Ok(())
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let before = pack
        .data
        .maps
        .get(HALL)
        .context("Ely Hall of Fame")?
        .clone();
    let mut room = before.clone();
    let recorder = room
        .objects
        .iter()
        .find(|o| o.script == RECORDER && [o.x, o.y] == [5, 5])
        .and_then(|o| o.object_identifier.clone())
        .context("Ely recorder")?;
    ensure!(
        room.attributes.tileset_name == "ice_path"
            && room
                .objects
                .iter()
                .any(|o| o.script == "MN_INTERIOR_ELY_FAMILY_PHOTO" && [o.x, o.y] == [2, 9])
            && room.events.bg_events.is_empty()
            && room.script_block_changes.is_empty()
            && room
                .script_runtime_commands
                .iter()
                .any(|c| c.source_script == RECORDER && c.command == "halloffame"),
        "expected the released Ely victory ceremony"
    );
    install_oak_tiles(pack)?;
    northstar_trail_caches::install_landmark_art_at(
        pack,
        include_bytes!("../../../planning/minnesota/revisions/ely-register-v01/furnishings.json"),
        0x80,
    )?;
    let design = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/ely-register-v01/gallery.json"
    ))?;
    compose(pack, &mut room, &design)?;
    // Set the same talking partner as an A press on the recorder. Jump to the
    // original script, retaining every instruction and saved command index.
    room.scripts.insert(
        REGISTER.into(),
        json!([
            command("setlasttalked", &[&recorder]),
            command("sjump", &[RECORDER])
        ]),
    );
    northstar_lair_authoring::read(&mut room, [6, 4], REGISTER);
    text(
        &mut room,
        "MnElyPortraitCaption",
        &["In a child's hand,", "on the back:", "She won too!"],
    );
    room.scripts.insert(
        PHOTO.into(),
        json!([
            command("pokepic", &["JIGLET"]),
            command("waitbutton", &[]),
            command("closepokepic", &[]),
            command("jumptext", &["MnElyPortraitCaption"]),
        ]),
    );
    for x in [11, 12] {
        northstar_lair_authoring::read(&mut room, [x, 6], PHOTO);
    }
    for y in 0..14 {
        for x in 0..10 {
            ensure!(
                authored_town_geometry::kind(pack, &before, [x, y])?
                    == authored_town_geometry::kind(pack, &room, [x, y])?,
                "changed saved Ely standing square {x},{y}"
            );
        }
    }
    let doorways = [[4, 12], [5, 12], [4, 13], [5, 13]];
    for entrance in [[4, 11], [5, 11]] {
        let accessible = northstar_cave_geometry::resting_tiles_avoiding(
            pack, &room, entrance, false, &doorways,
        )?;
        let previous = northstar_cave_geometry::resting_tiles_avoiding(
            pack, &before, entrance, false, &doorways,
        )?;
        ensure!(
            previous.is_subset(&accessible)
                && accessible.iter().filter(|p| p[0] < 10).count() == previous.len()
                && [[5, 6], [6, 5], [11, 7], [12, 7], [3, 9], [4, 11], [5, 11]]
                    .iter()
                    .all(|p| accessible.contains(p))
                && accessible.iter().filter(|p| p[0] >= 10).count() == 10,
            "old standing tiles and new gallery aisles must return to the same doorway"
        );
    }
    northstar_story_revision::publish(pack, room)
}
