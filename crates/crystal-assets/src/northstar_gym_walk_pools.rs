//! The three deep pockets around the broad, walkable Brainerd shoal.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{CompiledGamePack, authored_dialogue, authored_town_publication};

const BANK: &str = "mn_gym_6";
const GYM: &str = "MnBrainerdGym";
const LAYOUT: &str = "data/tilesets/mn_gym_6_metatiles.bin";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pools {
    canoe_water: Vec<[usize; 2]>,
    contours: Vec<Contour>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contour {
    at: [usize; 2],
    before: [u8; 4],
    after: [u8; 4],
}

pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let pools: Pools = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/gym-walk-v01/brainerd-pools.json"
    ))?;
    let mut room = pack.data.maps.get(GYM).context(GYM)?.clone();
    ensure!(
        room.attributes.tileset_name == BANK
            && (room.attributes.width, room.attributes.height) == (9, 7)
            && room.blocks.len() == 63,
        "expected the private Brainerd beach"
    );
    // Every edited water metatile occurs once. Preserve every other instance
    // of the gym's original floor, sand, actors, doors and shallow water.
    let square = |[x, y]: [usize; 2]| -> Result<(usize, usize)> {
        ensure!(x < 18 && y < 14, "pool extends past shore");
        let block = room.blocks[(y / 2) * 9 + x / 2];
        ensure!(
            room.blocks.iter().filter(|&&b| b == block).count() == 1,
            "pool edit would affect another metatile"
        );
        Ok((usize::from(block), y % 2 * 2 + x % 2))
    };
    let art = pack.runtime_files.get_mut(LAYOUT).context(LAYOUT)?;
    ensure!(art.len() == 54 * 16, "expected the installed Lakeside art");
    for contour in pools.contours {
        let (block, quadrant) = square(contour.at)?;
        let base = 16 * block + (quadrant / 2) * 8 + (quadrant % 2) * 2;
        for (i, offset) in [0, 1, 4, 5].into_iter().enumerate() {
            ensure!(
                art[base + offset] == contour.before[i],
                "Lakeside contour changed at {:?}",
                contour.at
            );
            art[base + offset] = contour.after[i];
        }
    }
    let bank = pack.data.tilesets.get_mut(BANK).context(BANK)?;
    for at in pools.canoe_water {
        let (block, quadrant) = square(at)?;
        let permission = &mut bank
            .collision
            .get_mut(&format!("{block:02x}"))
            .context("missing pool collision")?[quadrant];
        ensure!(
            permission == "FLOOR",
            "unexpected deep-water footing at {at:?}"
        );
        *permission = "WATER".into();
    }
    pack.runtime_files.insert(
        "data/tilesets/mn_gym_6.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    authored_dialogue::install_canonical(&mut room, "MnBrainerdGymGuideText")?;
    room.script_text_bodies = crate::parse_script_text_bodies(GYM, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
