//! Show daylight at the two trail exits and reserve the ladder for the shell gallery.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_dialogue, authored_town_publication};

const ROOM: &str = "MnR01Cave1F";
const ATLAS: &str = "data/tilesets/cave_metatiles.bin";

/// Installs the two ground-level exits and the guide's directions to the lower gallery.
///
/// Returns an error if the installed room or the native exit artwork differs from its donor.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context(ROOM)?.clone();
    let width = usize::from(room.attributes.width);
    ensure!(
        room.attributes.tileset_name == "cave"
            && (width, room.attributes.height, room.blocks.len()) == (20, 8, 160)
            && room.blocks[4 * width + 5] == 0x0b,
        "unexpected shell gallery ladder or Hollow dimensions"
    );
    let exits = [5 * width + 4, 6 * width + 15];
    for &exit in &exits {
        ensure!(
            room.blocks[exit] == 0x07 && room.blocks[exit + width] == 0x2e,
            "unexpected previous Hollow exit at block {exit}"
        );
    }

    let bank = pack.data.tilesets.get_mut("cave").context("cave bank")?;
    let atlas = pack.runtime_files.get_mut(ATLAS).context(ATLAS)?;
    let (blocks, tail) = atlas.as_chunks::<16>();
    ensure!(
        tail.is_empty()
            && blocks.len() == bank.collision.len()
            && blocks.get(0x24) == Some(&[0x01; 16])
            && bank
                .collision
                .get("24")
                .is_some_and(|r| r == &["FLOOR", "FLOOR", "FLOOR", "WARP_CARPET_DOWN"]),
        "unexpected native cave threshold"
    );
    let mut sill = *blocks.get(0x2e).context("solid limestone")?;
    let light = &blocks.get(0x23).context("cave daylight")?[2..4];
    let solid = bank.collision.get("2e").context("limestone permissions")?;
    ensure!(
        sill == [0x26; 16] && light == [0x1a, 0x1b] && solid.iter().all(|r| r == "WALL"),
        "unexpected cave daylight or limestone"
    );
    sill[2..4].copy_from_slice(light);
    let next = u8::try_from(blocks.len()).context("no room for Hollow exit sill")?;
    ensure!(
        !bank.collision.contains_key(&format!("{next:02x}")),
        "occupied cave sill"
    );
    bank.collision.insert(format!("{next:02x}"), solid.clone());
    atlas.extend_from_slice(&sill);
    pack.runtime_files.insert(
        "data/tilesets/cave.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    for exit in exits {
        room.blocks[exit] = 0x24;
        room.blocks[exit + width] = u16::from(next);
    }
    for label in [
        "MN_R01_HOLLOW_GUIDEText",
        "MN_R01_HOLLOW_GUIDEFieldHMGivenText",
    ] {
        ensure!(
            room.scripts.contains_key(label),
            "missing Hollow guide page {label}"
        );
        authored_dialogue::install_canonical(&mut room, label)?;
        let body = BTreeMap::from([(label.to_owned(), room.scripts[label].clone())]);
        room.script_text_bodies
            .extend(crate::parse_script_text_bodies(ROOM, &body)?);
    }
    authored_town_publication::replace(pack, room)
}
