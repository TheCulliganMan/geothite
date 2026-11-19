//! Give each Lake Pepin observation a recognizable bank and a useful return clue.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::command;
use crate::{
    AuthoredTileAnimation, CompiledGamePack, MapModule, authored_dialogue,
    authored_town_publication, minnesota_pokemon_pixel_art, native_tile_png,
};

const WEST: &str = "MnMississippiLowerWoodsC00R01";
const EAST: &str = "MnMississippiLowerWoodsC01R01";
const INLET: &str = "MnMississippiLowerWoodsC01R00";
const EDDY: &str = "EVENT_MN_PEPIN_EDDY_OBSERVED";
const SKETCH: &str = "EVENT_MN_PEPIN_DOCK_SKETCH_READ";
const SAFE: &str = "EVENT_MN_PEPIN_SAFE_INLET_IDENTIFIED";
const SHEET: &str = "gfx/tilesets/johto.2bpp";
const BLOCKS: &str = "data/tilesets/johto_metatiles.bin";
const GLYPHS: [u8; 2] = [0x71, 0x72];

fn install_foam(pack: &mut CompiledGamePack) -> Result<()> {
    // These aliases inherit any Johto animation file. An unused mark on Pepin must never
    // repaint a stair, a railway sleeper or an existing animation on a different map.
    for bank in [
        "johto",
        "mn_outdoor_johto_stairs",
        "mn_r10_rockstairs",
        "johto_glacier_rail",
        "mn_winter_iron_descent",
        "mn_iron_descent_thaw",
        "mn_winter_bemidji_northwoods",
    ] {
        let atlas = pack
            .runtime_files
            .get(&format!("data/tilesets/{bank}_metatiles.bin"))
            .context("inherited Johto atlas")?;
        let palettes = &pack
            .data
            .tilesets
            .get(bank)
            .context("inherited Johto palette")?
            .palette_map;
        ensure!(
            GLYPHS
                .iter()
                .all(|id| !atlas.contains(id) && usize::from(*id) < palettes.len()),
            "occupied or unaddressable eddy glyph in {bank}"
        );
    }
    let source: [[String; 8]; 4] = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/fen-trail-v01/pepin-eddy.json"
    ))?;
    let mut clips = BTreeMap::new();
    for (i, glyph) in GLYPHS.into_iter().enumerate() {
        let mut frames = Vec::with_capacity(4);
        for frame in &source {
            ensure!(
                frame
                    .iter()
                    .all(|r| r.len() == 16 && r.bytes().all(|v| (b'0'..=b'3').contains(&v))),
                "invalid Pepin foam pixels"
            );
            let pixels: Vec<_> = frame
                .iter()
                .flat_map(|row| row.as_bytes()[8 * i..8 * i + 8].iter().map(|v| *v - b'0'))
                .collect();
            let bytes = minnesota_pokemon_pixel_art::bitplanes(&pixels, 8, 8);
            frames.push(bytes.as_slice().try_into()?);
        }
        let clip = AuthoredTileAnimation {
            frame_ticks: 22,
            frames,
        };
        clip.validate()?;
        clips.insert(glyph, clip);
    }
    let graphics = pack.runtime_files.get_mut(SHEET).context(SHEET)?;
    ensure!(
        graphics.len() == 256 * 16,
        "expected the two full Johto VRAM pages"
    );
    let bank = pack.data.tilesets.get_mut("johto").context("Johto atlas")?;
    let water_palette = bank.palette_map[0x14];
    ensure!(water_palette == 3, "expected native water palette");
    for (&glyph, clip) in &clips {
        let i = usize::from(glyph);
        ensure!(
            graphics[16 * i..16 * i + 16] == [0; 16],
            "occupied original low-bank pixels"
        );
        graphics[16 * i..16 * i + 16].copy_from_slice(&clip.frames[0]);
        bank.palette_map[i] = water_palette;
    }
    let png = native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files
        .insert("gfx/tilesets/johto.png".into(), png);
    let atlas = pack.runtime_files.get_mut(BLOCKS).context(BLOCKS)?;
    ensure!(
        atlas.len() == 0x8d * 16 && atlas[0x35 * 16..0x36 * 16] == [0x14; 16],
        "unexpected original Pepin river blocks"
    );
    let mut block = [0x14u8; 16];
    block[4..6].copy_from_slice(&GLYPHS);
    atlas.extend(block);
    let water = bank
        .collision
        .get("35")
        .context("native water collision")?
        .clone();
    ensure!(
        water.iter().all(|v| v == "WATER") && bank.collision.insert("8d".into(), water).is_none(),
        "occupied Pepin foam block"
    );
    for (file, data) in [
        (
            "data/tilesets/johto.json",
            serde_json::to_vec(&bank.collision)?,
        ),
        (
            "data/tilesets/johto_palette_map.json",
            serde_json::to_vec(&bank.palette_map)?,
        ),
    ] {
        pack.runtime_files.insert(file.into(), data);
    }
    let path = "data/tilesets/johto_animations.json";
    ensure!(
        pack.runtime_files
            .insert(path.into(), serde_json::to_vec(&clips)?)
            .is_none(),
        "unexpected pre-existing Johto animation list"
    );
    Ok(())
}

fn add(room: &mut MapModule, label: &str, script: Value) -> Result<()> {
    ensure!(
        room.scripts.insert(label.into(), script).is_none(),
        "occupied Pepin branch {label}"
    );
    Ok(())
}

fn artist(room: &mut MapModule) -> Result<()> {
    let spot = 5 * usize::from(room.attributes.width) + 12;
    ensure!(
        room.blocks.get(spot) == Some(&0x35)
            && room
                .events
                .bg_events
                .iter()
                .any(|e| e.x == 24 && e.y == 10 && e.script == "MnPepinEddy"),
        "unexpected artist's eddy"
    );
    room.blocks[spot] = 0x8d;
    let root = room
        .scripts
        .get_mut("MnPepinDock")
        .and_then(Value::as_array_mut)
        .context("artist")?;
    ensure!(
        root.len() == 13
            && root[1] == command("checkevent", &[SAFE])
            && root[11] == command("setevent", &[SKETCH]),
        "unexpected western observer"
    );
    root[12] = command("sjump", &["MnPepinArtistNext"]);
    root.splice(
        3..3,
        [
            command("checkevent", &[SKETCH]),
            command("iftrue", &["MnPepinArtistNext"]),
        ],
    );
    add(
        room,
        "MnPepinArtistNext",
        json!([
            command("checkevent", &[EDDY]),
            command("iftrue", &["MnPepinArtistSkipper"]),
            command("jumptext", &["MnPepinText_artist_foam"])
        ]),
    )?;
    add(
        room,
        "MnPepinArtistSkipper",
        json!([command("jumptext", &["MnPepinText_artist_skipper"])]),
    )
}

fn skipper(room: &mut MapModule) -> Result<()> {
    let root = room
        .scripts
        .get_mut("MnPepinOperator")
        .and_then(Value::as_array_mut)
        .context("boat skipper")?;
    for (i, branch) in [(6, "MnPepinSkipperEddy"), (8, "MnPepinSkipperPines")] {
        ensure!(
            root.get(i) == Some(&command("iffalse", &["MnPepinOperatorWaiting"])),
            "unexpected Pepin comparison"
        );
        root[i] = command("iffalse", &[branch]);
    }
    for (branch, text) in [
        ("MnPepinSkipperEddy", "MnPepinText_operator_eddy"),
        ("MnPepinSkipperPines", "MnPepinText_operator_pines"),
    ] {
        add(room, branch, json!([command("jumptext", &[text])]))?;
    }
    Ok(())
}

/// Marks the western eddy, gives the artist and skipper progress-aware replies and matches the inlet to its pine bank.
///
/// Returns an error if the reviewed observer, water surface, comparison or free art slots have changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    install_foam(pack)?;
    for (name, labels) in [
        (
            WEST,
            &["dock_intro", "eddy", "artist_foam", "artist_skipper"][..],
        ),
        (
            EAST,
            &[
                "operator_partial",
                "operator_eddy",
                "operator_pines",
                "operator_compare",
                "operator_complete",
                "needs_surf",
                "reeds",
            ][..],
        ),
        (INLET, &["inlet_early", "reveal", "retreat", "caught"][..]),
    ] {
        let mut room = pack
            .data
            .maps
            .get(name)
            .context("Lower Woods river reach")?
            .clone();
        match name {
            WEST => artist(&mut room)?,
            EAST => skipper(&mut room)?,
            _ => {}
        }
        for suffix in labels {
            authored_dialogue::install_canonical(&mut room, &format!("MnPepinText_{suffix}"))?;
        }
        if name != INLET {
            room.script_flag_commands = crate::parse_script_flag_commands(name, &room.scripts)?;
            room.script_control_commands =
                crate::parse_script_control_commands(name, &room.scripts)?;
            room.script_text_commands = crate::parse_script_text_commands(name, &room.scripts)?;
        }
        room.script_text_bodies = crate::parse_script_text_bodies(name, &room.scripts)?;
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use anyhow::{Context, Result, ensure};
    use crystal_core::input::GameButton;
    use crystal_core::random::ReplayDivider;
    use crystal_core::state::GameState;
    use crystal_core::systems::script_control::ScriptControlAction;
    use crystal_core::world::map::{Direction, TilePosition};

    use super::EAST;

    #[test]
    #[ignore = "requires the external archive in MN_FEN_TRAIL_PACK"]
    fn owned_canoe_reaches_pepin_pine_observation_without_a_surf_partner() -> Result<()> {
        let pack = crate::read_loaded_verified_compiled_game_pack(std::path::PathBuf::from(
            std::env::var_os("MN_FEN_TRAIL_PACK").context("Fen Trail archive")?,
        ))?;
        let data = pack.pack().data();
        let music = pack.pack().audio_manifest.music.keys().cloned().collect();
        let mut session = data.overworld_session(EAST, TilePosition::new(50, 19), 0)?;
        let mut state = GameState::default();
        ensure!(
            state
                .bag
                .add_item(&data.items["CANOE"], 1)
                .map_err(anyhow::Error::msg)?
        );
        const READ: &str = "MnPepinReeds";
        const OBSERVED: &str = "EVENT_MN_PEPIN_REEDS_OBSERVED";
        // Inspect the refusal independently. Running isolated branch commands adds
        // presentation events which the interactive script runner normally consumes.
        let mut dry = state.clone();
        session.player.facing = Direction::Down;
        let read = session
            .check_interaction_checked(1)?
            .context("dry pine viewpoint")?;
        ensure!(read.script == READ && read.target_tile == TilePosition::new(50, 20));
        data.apply_script_variable_command_now_in_session(&mut dry, &session, EAST, READ, 0)?;
        ensure!(dry.script_runtime.script_value.as_deref() == Some("0"));
        for slot in [1, 2] {
            ensure!(matches!(
                data.apply_script_control_command_in_session(&mut dry, &session, EAST, READ, slot)?,
                ScriptControlAction::Continue { .. }
            ));
        }
        ensure!(
            data.script_text_command(EAST, READ, 3)?
                .text_label
                .as_deref()
                == Some("MnPepinText_needs_surf")
        );
        session.player.facing = Direction::Left;
        let mut random = ReplayDivider::new([255; 256]);
        for button in [
            GameButton::Left,
            GameButton::Down,
            GameButton::Down,
            GameButton::Right,
        ] {
            let frame = data.apply_overworld_input(
                &mut state,
                &mut session,
                [button],
                &music,
                &mut random,
            )?;
            ensure!(
                frame.movement.is_some(),
                "no movement transaction for {button:?} from {:?}",
                session.player.tile
            );
            data.apply_overworld_input(&mut state, &mut session, [], &music, &mut random)?;
        }
        ensure!(
            session.player.tile == TilePosition::new(49, 20)
                && data.is_canoe_mounted(&state, EAST, session.player.mode),
            "expected the canoe at (49,20), found {:?} in {:?}",
            session.player.tile,
            session.player.mode
        );
        let read = session
            .check_interaction_checked(1)?
            .context("afloat pine viewpoint")?;
        ensure!(read.script == READ && read.target_tile == TilePosition::new(50, 20));
        data.apply_script_variable_command_now_in_session(&mut state, &session, EAST, READ, 0)?;
        ensure!(state.script_runtime.script_value.as_deref() == Some("4"));
        let action =
            data.apply_script_control_command_in_session(&mut state, &session, EAST, READ, 1)?;
        ensure!(
            matches!(action,ScriptControlAction::Jump{target_script,..} if target_script=="MnPepinReedsSurf")
        );
        ensure!(!state.flags.is_event_flag_set(OBSERVED)?);
        data.apply_script_flag_mutation_in_session(
            &mut state,
            &mut session,
            EAST,
            "MnPepinReedsSurf",
            4,
        )?;
        ensure!(state.flags.is_event_flag_set(OBSERVED)?);
        Ok(())
    }
}
