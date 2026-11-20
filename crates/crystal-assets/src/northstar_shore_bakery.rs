//! The North Shore baker sells a blueberry slice and keeps the cracked pie on her table.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::json;

use crate::authored_dialogue;
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring::command;
use crate::northstar_rival_arc::put;
use crate::{CompiledGamePack, CompiledTilesetExtension};

const ROOM: &str = "MnNorthShorePieShop";
const BASE: &str = "game_corner";
const TILESET: &str = "mn_shore_bakery";
const ITEM: &str = "MN_BLUEBERRY_SLICE";
const BAKER: &str = "MN_SHORE_BAKER";
const OFFER: &str = "MN_SHORE_BAKERText";
const TABLE: &str = "MnShorePieTable";
const FIRST_PIE_GLYPH: u8 = 0xc8;
const TABLE_ART: &[u8] =
    include_bytes!("../../../planning/minnesota/landmarks/north-shore-pie-table.json");

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context(ROOM)?.clone();
    ensure!(
        room.attributes.tileset_name == BASE
            && room.attributes.width == 4
            && room.attributes.height == 4
            && room.blocks.get(5) == Some(&0x15)
            && room.events.bg_events.is_empty()
            && room.scripts.get(BAKER) == Some(&json!([command("jumptextfaceplayer", &[OFFER])]))
            && kind(pack, &room, [5, 3])? == "FLOOR"
            && kind(pack, &room, [6, 3])? == "COUNTER"
            && kind(pack, &room, [7, 3])? == "FLOOR"
            && !room.objects.iter().any(|o| (o.x, o.y) == (7, 3)),
        "unexpected blueberry counter or table"
    );
    let baker = room
        .objects
        .iter_mut()
        .find(|o| o.script == BAKER && (o.x, o.y) == (5, 3))
        .context("North Shore baker")?;
    baker.x = 7;
    baker.spritemovedata = "SPRITEMOVEDATA_STANDING_LEFT".into();
    room.attributes.tileset_name = TILESET.into();
    room.blocks[5] = 0x40;
    pie_table(pack)?;
    for y in [2, 3] {
        for x in [2, 3] {
            room.events.bg_events.push(BackgroundEvent {
                x,
                y,
                event_type: "BGEVENT_READ".into(),
                script: TABLE.into(),
            });
        }
    }

    // Match the price and healing of a State Fair serving of mini donuts.
    let mut slice = pack
        .data
        .items
        .get("FAIR_MINI_DONUTS")
        .context("regional trail snack")?
        .clone();
    ensure!(
        slice.price == 350
            && slice.parameter == 50
            && slice.effect == "RestoreHPEffect"
            && slice.field_usable
            && slice.battle_usable
            && slice.consumable,
        "expected native healing food"
    );
    slice.name = "PIE SLICE".into();
    slice.description = "Blueberry filling.\nRestores 50 HP.".into();
    slice.script_name = ITEM.into();
    ensure!(
        pack.data.items.insert(ITEM.into(), slice).is_none(),
        "occupied bakery item"
    );
    for label in [
        OFFER,
        "MnShorePieTableText",
        "MnShorePiePoorText",
        "MnShorePieFullText",
        "MnShorePiePaidText",
    ] {
        authored_dialogue::install_canonical(&mut room, label)?;
    }
    put(
        &mut room,
        TABLE,
        vec![command("jumptext", &["MnShorePieTableText"])],
    );
    put(
        &mut room,
        BAKER,
        vec![
            command("faceplayer", &[]),
            command("opentext", &[]),
            command("writetext", &[OFFER]),
            command("yesorno", &[]),
            command("iffalse", &["MnShorePieClose"]),
            command("checkmoney", &["YOUR_MONEY", "350"]),
            command("ifequal", &["HAVE_LESS", "MnShorePiePoor"]),
            command("giveitem", &[ITEM, "1"]),
            command("iffalse", &["MnShorePieFull"]),
            command("takemoney", &["YOUR_MONEY", "350"]),
            command("writetext", &["MnShorePiePaidText"]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[]),
        ],
    );
    put(
        &mut room,
        "MnShorePieClose",
        vec![command("closetext", &[]), command("end", &[])],
    );
    for (label, text) in [
        ("MnShorePiePoor", "MnShorePiePoorText"),
        ("MnShorePieFull", "MnShorePieFullText"),
    ] {
        put(
            &mut room,
            label,
            vec![
                command("writetext", &[text]),
                command("waitbutton", &[]),
                command("closetext", &[]),
                command("end", &[]),
            ],
        );
    }
    crate::northstar_story_revision::publish(pack, room)
}

fn pie_table(pack: &mut CompiledGamePack) -> Result<()> {
    let file = |path: &str| {
        pack.runtime_files
            .get(path)
            .cloned()
            .with_context(|| format!("native bakery source {path}"))
    };
    let extension = CompiledTilesetExtension {
        manifest_id: "mn-shore-bakery-table-v1".into(),
        tileset_id: TILESET.into(),
        behavior_source_tileset_id: Some(BASE.into()),
        definition: pack
            .data
            .tilesets
            .get(BASE)
            .context("native cafe tiles")?
            .clone(),
        metatiles: file("data/tilesets/game_corner_metatiles.bin")?,
        tile_graphics_2bpp: file("gfx/tilesets/game_corner.2bpp")?,
        tile_graphics_png: file("gfx/tilesets/game_corner.png")?,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    crate::northstar_trail_caches::install_landmark_art_at(pack, TABLE_ART, FIRST_PIE_GLYPH)?;
    let table = pack.data.tilesets.get_mut(TILESET).context("bakery art")?;
    // The pastry occupies four cells on the ivory runner. The room's amber
    // palette shares its ivory and ink with the red cloth on either side.
    for offset in [1, 2, 5, 6] {
        table.palette_map[usize::from(FIRST_PIE_GLYPH) + offset] = 8 | 4;
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}_palette_map.json"),
        serde_json::to_vec(&table.palette_map)?,
    );
    Ok(())
}
