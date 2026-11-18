//! Shiner Lake's footbridge, navigable outlet, fishing dock and sheltered landing.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    MovementMode, PlayerMovementState, StepOptions, StepOutcome, attempt_step_with_occupied_tiles,
};
use serde::Deserialize;

use crate::authored_town_geometry::kind;
use crate::northstar_cave_geometry::resting_tiles_avoiding;
use crate::{CompiledGamePack, MapModule, authored_town_publication, native_tile_png};

const MAP: &str = "MnR15C00R00";
const REWARD: &str = "MN_R15_SHINER_CREEL";
const PLAN: &str =
    include_str!("../../../planning/minnesota/routes/r15-shiner-narrows-v01/bridge.json");
const FIRST_BLOCK: usize = 0x85;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Layout {
    glyphs: Vec<[String; 8]>,
    palettes: Vec<u8>,
    blocks: Vec<Block>,
    placements: Vec<Placement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    name: String,
    glyphs: [u8; 16],
    collision: [String; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    at: [u16; 2],
    from: u16,
    to: u16,
}

fn publish_art(pack: &mut CompiledGamePack, layout: &Layout) -> Result<()> {
    let bank = pack
        .data
        .tilesets
        .get_mut("kanto")
        .context("Shiner Lake tiles")?;
    let graphics = pack
        .runtime_files
        .get_mut("gfx/tilesets/kanto.2bpp")
        .context("Kanto glyphs")?;
    ensure!(
        graphics.len() == 96 * 16,
        "expected original six-row Kanto sheet"
    );
    let first_glyph = graphics.len() / 16;
    let end_glyph = first_glyph + layout.glyphs.len();
    ensure!(
        layout.glyphs.len().is_multiple_of(16)
            && end_glyph <= bank.palette_map.len()
            && end_glyph <= 256
            && layout.palettes.len() == layout.glyphs.len(),
        "invalid Shiner Lake atlas extension"
    );
    for (index, tile) in layout.glyphs.iter().enumerate() {
        for line in tile {
            let bytes = line.as_bytes();
            ensure!(bytes.len() == 8, "Shiner Lake glyph row width");
            let mut planes = [0u8; 2];
            for &ink in bytes {
                ensure!(matches!(ink, b'0'..=b'3'), "invalid Shiner Lake ink");
                let shade = ink - b'0';
                planes[0] = (planes[0] << 1) | (shade & 1);
                planes[1] = (planes[1] << 1) | (shade >> 1);
            }
            graphics.extend(planes);
        }
        let palette = layout.palettes[index];
        ensure!(palette < 8, "invalid native Shiner Lake palette");
        bank.palette_map[first_glyph + index] = palette;
    }
    let png = native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files
        .insert("gfx/tilesets/kanto.png".into(), png);
    let metatiles = pack
        .runtime_files
        .get_mut("data/tilesets/kanto_metatiles.bin")
        .context("Kanto blocks")?;
    ensure!(
        metatiles.len() == FIRST_BLOCK * 16 && metatiles.iter().all(|&g| g < 96),
        "new timber glyphs must be outside the released atlas"
    );
    for (index, block) in layout.blocks.iter().enumerate() {
        ensure!(
            block.glyphs.iter().all(|&g| usize::from(g) < end_glyph)
                && bank
                    .collision
                    .insert(
                        format!("{:02x}", FIRST_BLOCK + index),
                        block.collision.to_vec()
                    )
                    .is_none(),
            "invalid or occupied Shiner Lake block {}",
            block.name
        );
        metatiles.extend(block.glyphs);
    }
    pack.runtime_files.insert(
        "data/tilesets/kanto.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    pack.runtime_files.insert(
        "data/tilesets/kanto_palette_map.json".into(),
        serde_json::to_vec(&bank.palette_map)?,
    );
    Ok(())
}

fn place(pack: &CompiledGamePack, room: &mut MapModule, layout: &Layout) -> Result<()> {
    for edit in &layout.placements {
        let [x, y] = edit.at;
        ensure!(
            x > 0 && y > 0 && x + 1 < room.attributes.width && y + 1 < room.attributes.height,
            "Shiner Lake must preserve map edges"
        );
        let index = usize::from(room.attributes.width) * usize::from(y) + usize::from(x);
        ensure!(
            room.blocks.get(index) == Some(&edit.from) && edit.from != edit.to,
            "Shiner Lake preimage at {x},{y}"
        );
        let squares = [
            [2 * x, 2 * y],
            [2 * x + 1, 2 * y],
            [2 * x, 2 * y + 1],
            [2 * x + 1, 2 * y + 1],
        ];
        let before = squares
            .iter()
            .map(|&at| kind(pack, room, at))
            .collect::<Result<Vec<_>>>()?;
        room.blocks[index] = edit.to;
        for (tile, old) in squares.into_iter().zip(before) {
            let new = kind(pack, room, tile)?;
            ensure!(
                old == new || old == "WALL" || (old == "FLOOR" && new == "BRIDGE_NS_DECK"),
                "previous standing, surfing or hop permission lost at {tile:?}"
            );
        }
    }
    Ok(())
}

fn check_crossing(pack: &CompiledGamePack, room: &MapModule) -> Result<()> {
    let map = OverworldMapData::from_attributes(MAP, &room.attributes, room.blocks.clone());
    let collision = pack.data.tileset_collision("kanto")?;
    let options = StepOptions {
        force_step_after_turn: true,
        ..Default::default()
    };
    let mut options_walk = PlayerMovementState::new(TilePosition::new(37, 28));
    for dir in [Direction::Right, Direction::Left] {
        for _ in 0..9 {
            ensure!(
                matches!(
                    attempt_step_with_occupied_tiles(
                        &mut options_walk,
                        dir,
                        &map,
                        &collision,
                        options,
                        &[]
                    ),
                    StepOutcome::Moved { .. }
                ),
                "crossing on foot"
            );
        }
    }
    ensure!(
        options_walk.tile == TilePosition::new(37, 28),
        "round trip across both sills"
    );
    for x in 38..=45 {
        let mut boat =
            PlayerMovementState::new(TilePosition::new(x, 24)).with_mode(MovementMode::Surf);
        for dir in [Direction::Down, Direction::Up] {
            for _ in 0..8 {
                ensure!(
                    matches!(
                        attempt_step_with_occupied_tiles(
                            &mut boat,
                            dir,
                            &map,
                            &collision,
                            options,
                            &[]
                        ),
                        StepOutcome::Moved { .. }
                    ) && boat.mode == MovementMode::Surf,
                    "navigable bridge lane {x}"
                );
            }
        }
        ensure!(
            boat.tile == TilePosition::new(x, 24),
            "two-way channel under the bridge"
        );
    }
    let mut boat =
        PlayerMovementState::new(TilePosition::new(38, 28)).with_mode(MovementMode::Surf);
    ensure!(
        matches!(
            attempt_step_with_occupied_tiles(
                &mut boat,
                Direction::Left,
                &map,
                &collision,
                options,
                &[]
            ),
            StepOutcome::Blocked { .. }
        ),
        "canoe cannot climb the bridge abutment"
    );
    let portals: Vec<_> = room.events.warps.iter().map(|w| [w.x, w.y]).collect();
    let dry = resting_tiles_avoiding(pack, room, [46, 28], false, &portals)?;
    let shore = [49, 32];
    let lake = resting_tiles_avoiding(pack, room, [37, 14], true, &portals)?;
    let returning = resting_tiles_avoiding(pack, room, shore, true, &portals)?;
    ensure!(
        dry.contains(&[37, 28])
            && !dry.contains(&shore)
            && lake.contains(&shore)
            && returning.contains(&[37, 14]),
        "dock, water-only creel and return to the road"
    );
    Ok(())
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let layout: Layout = serde_json::from_str(PLAN)?;
    let mut room = pack.data.maps.get(MAP).context("Central Lakes")?.clone();
    ensure!(
        room.attributes.tileset_name == "kanto",
        "native Central Lakes map"
    );
    publish_art(pack, &layout)?;
    place(pack, &mut room, &layout)?;
    check_crossing(pack, &room)?;
    crate::northstar_cave_encounters::add_hidden_item(
        pack,
        &mut room,
        REWARD,
        "LURE_BALL",
        [50, 32],
    )?;
    room.script_field_pickups =
        crate::parse_script_field_pickups(MAP, &room.scripts, &room.objects)?;
    ensure!(
        room.script_field_pickups
            .iter()
            .any(|item| item.source_script == REWARD
                && item.item_id.as_deref() == Some("LURE_BALL")
                && item.quantity == 1
                && item.event_flag.as_deref() == Some("EVENT_MN_R15_SHINER_CREEL")),
        "one-time creel discovery"
    );
    crate::grand_marais_modpack::authoring::refresh_event_section(&mut room);
    authored_town_publication::replace(pack, room)
}
