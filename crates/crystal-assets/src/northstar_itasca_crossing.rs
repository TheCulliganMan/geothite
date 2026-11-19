//! The timber footbridge and the observer's readable water gauge on Lake Itasca.
use anyhow::{Context, Result, ensure};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring;
use crate::{BackgroundEvent, CompiledGamePack, authored_town_publication, native_tile_png};

const BANK: &str = "mn_headwaters";
const TILES: &str = "gfx/tilesets/mn_headwaters.2bpp";
const BLOCKS: &str = "data/tilesets/mn_headwaters_metatiles.bin";
const LAKE: &str = "MnLakeItasca";
const GATE: &str = "MnItascaGatehouse";
const MARKER: &str = "MN_ITASCA_MARKER";

// Two original 8x8 tiles. The pale enamel face has alternating long and short
// marks, two bolts and a dark, wet foot. Its timber backing meets the bank.
const STAFF: [[&[u8; 8]; 8]; 2] = [
    [
        b"23333332",
        b"23000032",
        b"23030032",
        b"23000032",
        b"23330032",
        b"23000032",
        b"23300032",
        b"23000032",
    ],
    [
        b"23330032",
        b"23000032",
        b"23300032",
        b"23000032",
        b"23030032",
        b"23111132",
        b"23333332",
        b"33232323",
    ],
];

fn crossing_art(pack: &mut CompiledGamePack, timber: u16) -> Result<()> {
    let bank = pack.data.tilesets.get_mut(BANK).context(BANK)?;
    let original = pack.runtime_files.get(TILES).context(TILES)?.clone();
    let atlas = pack.runtime_files.get_mut(BLOCKS).context(BLOCKS)?;
    ensure!(
        atlas.len() == 0x90 * 16 && original.len() == 192 * 16 && bank.palette_map.len() == 224,
        "unexpected headwaters graphics layout"
    );
    ensure!(
        atlas
            .iter()
            .all(|&g| (g & 127) < 96 && (bank.palette_map[usize::from(g)] & 8 != 0 || g < 96)),
        "original headwaters tiles exceed the two compact banks"
    );
    let shore: [u8; 16] = atlas[0x58 * 16..0x59 * 16].try_into()?;
    let deck = atlas.as_chunks::<16>().0[usize::from(timber)];
    ensure!(
        shore
            == [
                61, 20, 20, 20, 61, 20, 20, 20, 61, 20, 20, 20, 61, 20, 20, 20
            ],
        "expected the moving-water bank"
    );
    // The brook narrows to one metatile at the footbridge. Use the native
    // inside corners where the east bank enters and leaves that bottleneck.
    let east: [u8; 16] = atlas[0x59 * 16..0x5a * 16].try_into()?;
    let mut channel = shore;
    for i in [3, 7, 11, 15] {
        channel[i] = east[i];
    }
    let mut top_bend = east;
    top_bend[12..].copy_from_slice(&atlas[0x51 * 16 + 12..0x52 * 16]);
    let mut lower_bend = shore;
    lower_bend[3] = atlas[0x7a * 16 + 3];
    let mut gauge = shore;
    gauge[15] = atlas[0x79 * 16];
    gauge[8] = 0xe0;
    gauge[12] = 0xe1;
    let mut north = channel;
    north[8..].copy_from_slice(&deck[..8]);
    let mut south = channel;
    south[..8].copy_from_slice(&deck[8..]);
    for (id, block, roles) in [
        ("90", gauge, ["WATER", "WATER", "WALL", "WATER"]),
        (
            "91",
            north,
            ["WATER", "WATER", "BRIDGE_NS_DECK", "BRIDGE_NS_DECK"],
        ),
        (
            "92",
            south,
            ["BRIDGE_NS_DECK", "BRIDGE_NS_DECK", "WATER", "WATER"],
        ),
        ("93", channel, ["WATER"; 4]),
        ("94", top_bend, ["WATER"; 4]),
        ("95", lower_bend, ["WATER"; 4]),
    ] {
        ensure!(
            bank.collision
                .insert(id.into(), roles.map(String::from).to_vec())
                .is_none(),
            "occupied water block {id}"
        );
        atlas.extend(block);
    }
    // Both banks grow together, keeping every existing logical tile address.
    let mut graphics = vec![0u8; 256 * 16];
    graphics[..96 * 16].copy_from_slice(&original[..96 * 16]);
    graphics[128 * 16..224 * 16].copy_from_slice(&original[96 * 16..]);
    for (i, tile) in STAFF.iter().enumerate() {
        for (y, row) in tile.iter().enumerate() {
            for (x, v) in row.iter().enumerate() {
                let shade = v - b'0';
                let at = (224 + i) * 16 + 2 * y;
                graphics[at] |= (shade & 1) << (7 - x);
                graphics[at + 1] |= (shade >> 1) << (7 - x);
            }
        }
    }
    bank.palette_map.extend([8 | 5, 8 | 5]);
    let png = native_tile_png::from_2bpp(&graphics)?;
    pack.runtime_files
        .insert("gfx/tilesets/mn_headwaters.png".into(), png);
    pack.runtime_files.insert(TILES.into(), graphics);
    pack.runtime_files.insert(
        "data/tilesets/mn_headwaters.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    pack.runtime_files.insert(
        "data/tilesets/mn_headwaters_palette_map.json".into(),
        serde_json::to_vec(&bank.palette_map)?,
    );
    Ok(())
}

pub(crate) fn apply(pack: &mut CompiledGamePack, timber: u16) -> Result<()> {
    crossing_art(pack, timber)?;
    let mut lake = pack.data.maps.get(LAKE).context(LAKE)?.clone();
    ensure!(
        lake.attributes.tileset_name == BANK
            && lake.attributes.width == 24
            && lake.attributes.height == 24,
        "Lake Itasca geometry"
    );
    for (x, y, old, new) in [
        (11, 10, 0x58, 0x90),
        (11, 12, 0x82, 0x91),
        (11, 13, 0x81, 0x92),
        (12, 10, 0x59, 0x94),
        (11, 11, 0x58, 0x93),
        (11, 14, 0x54, 0x93),
        (11, 15, 0x58, 0x93),
        (11, 16, 0x58, 0x93),
        (11, 17, 0x58, 0x93),
        (11, 18, 0x58, 0x95),
    ] {
        let block = &mut lake.blocks[y * 24 + x];
        ensure!(*block == old, "unexpected Lake Itasca block at ({x},{y})");
        *block = new;
    }
    ensure!(
        lake.events.bg_events.iter().any(|e| e.x == 19
            && e.y == 21
            && e.script == MARKER
            && e.event_type == "BGEVENT_READ")
            && lake.events.bg_events.iter().all(|e| (e.x, e.y) != (22, 21)),
        "expected the original signed reading"
    );
    lake.events.bg_events.push(BackgroundEvent {
        x: 22,
        y: 21,
        event_type: "BGEVENT_READ".into(),
        script: MARKER.into(),
    });
    authoring::refresh_event_section(&mut lake);
    for label in [
        "MN_ITASCA_MARKERText",
        "MnItascaMarkerAfterText",
        "MnOverhaulItascaComparedText",
        "MN_ITASCA_OBSERVERText",
        "MN_ITASCA_TRAIL_WALKERText",
        "MN_ITASCA_WEST_CLOSEDText",
    ] {
        authored_dialogue::install_canonical(&mut lake, label)?;
    }
    let story: serde_json::Value =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    let replies = story["regional_guidance_revision"][LAKE]["replies"]
        .as_array()
        .context("Itasca trail walker")?;
    ensure!(replies.len() == 2, "unexpected Itasca trail branches");
    for reply in replies {
        let label = format!("{}Text", reply["id"].as_str().context("trail reply")?);
        let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(reply["pages"].clone())?;
        ensure!(
            lake.scripts
                .insert(label, authored_dialogue::compile(&pages)?)
                .is_some(),
            "expected existing trail reply"
        );
    }
    lake.script_text_bodies = crate::parse_script_text_bodies(LAKE, &lake.scripts)?;
    authored_town_publication::replace(pack, lake)?;
    let mut gate = pack.data.maps.get(GATE).context(GATE)?.clone();
    for label in ["MN_ITASCA_GATE_ATTENDANTText", "MN_ITASCA_GATE_WALKERText"] {
        authored_dialogue::install_canonical(&mut gate, label)?;
    }
    gate.script_text_bodies = crate::parse_script_text_bodies(GATE, &gate.scripts)?;
    authored_town_publication::replace(pack, gate)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anyhow::{Context, Result, ensure};
    use crystal_core::state::GameState;
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::movement::{MovementMode, StepOptions, StepOutcome};
    use crystal_core::world::session::OverworldInteractionTarget;

    use super::{LAKE, MARKER};

    #[test]
    #[ignore = "requires the built Minnesota edition in MN_HEADWATERS_PACK"]
    fn headwaters_reading_and_bridge_use_the_native_interaction_and_movement_layers() -> Result<()>
    {
        let pack = crate::read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("MN_HEADWATERS_PACK").context("headwaters pack")?,
        ))?;
        let data = pack.data();
        let state = GameState::reset_wram_for_new_game();
        for (x, y, facing, target) in [
            (19, 22, Direction::Up, TilePosition::new(19, 21)),
            (21, 21, Direction::Right, TilePosition::new(22, 21)),
        ] {
            let mut session = data.overworld_session(LAKE, TilePosition::new(x, y), 0)?;
            session.player.facing = facing;
            let hit = session
                .check_interaction_checked(1)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .context("look at the water readings")?;
            let active = data
                .resolve_overworld_interaction(&state, &hit)?
                .context("readable background")?;
            ensure!(
                active.target_tile == target
                    && active.script == MARKER
                    && matches!(active.target,OverworldInteractionTarget::Background{event_type} if event_type=="BGEVENT_READ"),
                "wrong interaction from ({x},{y})"
            );
        }
        // The bridge is named for its water axis. Both banks are joined on foot;
        // both columns of the moving brook remain open beneath it in Surf mode.
        for (x, y, mode, direction, steps) in [
            (21, 25, MovementMode::Normal, Direction::Right, 3),
            (24, 26, MovementMode::Normal, Direction::Left, 3),
            (22, 24, MovementMode::Surf, Direction::Down, 4),
            (23, 28, MovementMode::Surf, Direction::Up, 4),
        ] {
            let mut session = data.overworld_session(LAKE, TilePosition::new(x, y), 0)?;
            session.player.mode = mode;
            session.player.facing = direction;
            for _ in 0..steps {
                let outcome = session
                    .step_checked(direction, StepOptions::default())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                ensure!(
                    matches!(outcome, StepOutcome::Moved { .. }) && session.player.mode == mode,
                    "interrupted {mode:?} crossing toward {direction:?} at {:?}",
                    session.player.tile
                );
            }
        }
        Ok(())
    }
}
