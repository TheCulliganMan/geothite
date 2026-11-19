//! Restore Brainerd's swimming beach and make Bemidji's garden directions follow keeper wins.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::grand_marais_modpack::authoring::command;
use crate::{
    AuthoredTileAnimation, CompiledGamePack, authored_dialogue, authored_town_publication,
};

const RIPPLE_TILES: [(u8, u8); 8] = [
    (0x22, 1),
    (0x2d, 1),
    (0x5a, 1), // darker water
    (0x52, 0),
    (0x64, 0),
    (0x6c, 0),
    (0x73, 0),
    (0x79, 0), // shallows
];

/// Updates the two northern gym rooms in a Badge Trails parent.
///
/// Returns an error for a different beach, missing garden script or occupied new label.
pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    // Field already ships Lakeside. Intro still uses the original square-tiled lake.
    if !pack.runtime_files.contains_key("gfx/tilesets/mn_gym_6.pal") {
        crate::northstar_brainerd_lakeside::install(pack)?;
    }
    #[derive(Deserialize)]
    struct Beach {
        room: Vec<u16>,
    }
    let beach: Beach = serde_json::from_str(include_str!(
        "../../../planning/minnesota/gyms/brainerd-lakeside-v01/shallows.json"
    ))?;
    ensure!(
        pack.data
            .maps
            .get("MnBrainerdGym")
            .is_some_and(|m| m.blocks == beach.room),
        "unexpected installed Brainerd beach"
    );
    // Both authorized lineages now share exactly these reviewed atlas bytes.
    let sheet = pack
        .runtime_files
        .get("gfx/tilesets/mn_gym_6.2bpp")
        .context("native beach art")?;
    ensure!(
        format!("{:x}", Sha256::digest(sheet))
            == "a24fa21ceb128881daac74d8f93e36e5dacc8f62213fd47896c8aa00d8becceb",
        "unexpected Brainerd beach ink"
    );
    animate_beach(pack)?;
    update_garden(pack)
}

fn animate_beach(pack: &mut CompiledGamePack) -> Result<()> {
    let sheet = pack
        .runtime_files
        .get("gfx/tilesets/mn_gym_6.2bpp")
        .context("Brainerd beach ripples")?;
    let mut clips = BTreeMap::new();
    for (index, (tile, background)) in RIPPLE_TILES.into_iter().enumerate() {
        let start = usize::from(tile) * 16;
        let still: [u8; 16] = sheet
            .get(start..start + 16)
            .context("beach glyph missing")?
            .try_into()?;
        ensure!(
            pack.data.tilesets["mn_gym_6"].palette_map[usize::from(tile)] == 3,
            "ripple is not native beach water"
        );
        let mut frames = Vec::new();
        // Each selected glyph is open water with room for its crest to travel
        // one pixel. Reverse alternating cycles to avoid a whole lake in lockstep.
        for (right, down) in if index % 2 == 0 {
            [(false, false), (true, false), (true, true), (false, true)]
        } else {
            [(false, false), (false, true), (true, true), (true, false)]
        } {
            let mut frame = [0; 16];
            for plane in 0..2 {
                let fill = if (background >> plane) & 1 == 1 {
                    255
                } else {
                    0
                };
                ensure!(
                    still[14 + plane] == fill
                        && still
                            .iter()
                            .skip(plane)
                            .step_by(2)
                            .all(|row| (row ^ fill) & 1 == 0),
                    "ripple at {tile:02x} touches its lower or right edge"
                );
                for y in 0..8 {
                    let row = if down && y == 0 {
                        fill
                    } else {
                        still[2 * (y - usize::from(down)) + plane]
                    };
                    frame[2 * y + plane] = if right {
                        ((row ^ fill) >> 1) ^ fill
                    } else {
                        row
                    };
                }
            }
            frames.push(frame);
        }
        let clip = AuthoredTileAnimation {
            frame_ticks: 22,
            frames,
        };
        clip.validate()?;
        clips.insert(tile, clip);
    }
    ensure!(
        pack.runtime_files
            .insert(
                "data/tilesets/mn_gym_6_animations.json".to_owned(),
                serde_json::to_vec(&clips)?
            )
            .is_none(),
        "unexpected existing gym water animation"
    );
    Ok(())
}

fn update_garden(pack: &mut CompiledGamePack) -> Result<()> {
    const GYM: &str = "MnBemidjiGym";
    let mut room = pack.data.maps.get(GYM).context("Bemidji garden")?.clone();
    let gift = room
        .scripts
        .get_mut("MnBemidjiGymGuide")
        .context("garden HM guide")?;
    let entries = gift.as_array_mut().context("garden gift program")?;
    let gift_page = entries
        .iter_mut()
        .find(|entry| **entry == command("writetext", &["MnBemidjiGymGuideCutGiftText"]))
        .context("garden HM gift page")?;
    *gift_page = command("writetext", &["MnGardenCutText"]);

    let repeat = "MnBemidjiGymCircuitBeforeChampion";
    ensure!(
        room.scripts.get(repeat)
            == Some(&json!([command(
                "jumptextfaceplayer",
                &["MnBemidjiGymGuideText"]
            )])),
        "unexpected garden guide branch"
    );
    room.scripts.insert(
        repeat.into(),
        json!([
            command("checkevent", &["EVENT_MN_GYM_7_DEFEATED"]),
            command("iftrue", &["MnGardenReturn"]),
            command("checkevent", &["EVENT_MN_GYM_7_TRAINER_1_DEFEATED"]),
            command("iftrue", &["MnGardenSawMoslen"]),
            command("checkevent", &["EVENT_MN_GYM_7_TRAINER_2_DEFEATED"]),
            command("iftrue", &["MnGardenWest"]),
            command("jumptextfaceplayer", &["MnGardenForkText"]),
        ]),
    );
    for (label, program) in [
        (
            "MnGardenSawMoslen",
            json!([
                command("checkevent", &["EVENT_MN_GYM_7_TRAINER_2_DEFEATED"]),
                command("iftrue", &["MnGardenLeader"]),
                command("jumptextfaceplayer", &["MnGardenNorthText"]),
            ]),
        ),
        (
            "MnGardenWest",
            json!([command("jumptextfaceplayer", &["MnGardenWestText"])]),
        ),
        (
            "MnGardenLeader",
            json!([command("jumptextfaceplayer", &["MnGardenLeaderText"])]),
        ),
        (
            "MnGardenReturn",
            json!([command("jumptextfaceplayer", &["MnGardenReturnText"])]),
        ),
    ] {
        ensure!(
            room.scripts.insert(label.into(), program).is_none()
                && !pack.data.map_scripts.contains_key(label),
            "occupied garden script {label}"
        );
    }
    for label in [
        "MnGardenCutText",
        "MnGardenForkText",
        "MnGardenNorthText",
        "MnGardenWestText",
        "MnGardenLeaderText",
        "MnGardenReturnText",
    ] {
        ensure!(
            !room.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
            "occupied garden dialogue {label}"
        );
        authored_dialogue::install_canonical(&mut room, label)?;
    }
    room.script_flag_commands = crate::parse_script_flag_commands(GYM, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(GYM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(GYM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(GYM, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
