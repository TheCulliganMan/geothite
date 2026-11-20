//! Finish the move-machine sales island with the MART's existing counter artwork.
use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, MapModule};

pub(super) fn finish_machine_counter(
    pack: &mut CompiledGamePack,
    room: &mut MapModule,
) -> Result<()> {
    ensure!(
        room.id == "MnMinneapolisDepartmentStore5F"
            && room.attributes.tileset_name == "mart"
            && (room.attributes.width, room.attributes.height) == (8, 4)
            && room.blocks.get(3..5) == Some(&[0x03, 0x03])
            && room.blocks.get(11..13) == Some(&[0x2b, 0x2b])
            && room.blocks.get(19..21) == Some(&[0x1a, 0x1b])
            && room.blocks.get(27..29) == Some(&[0x1c, 0x1d]),
        "expected the fifth-floor trading booth and link carpet"
    );
    let atlas = pack
        .runtime_files
        .get_mut("data/tilesets/mart_metatiles.bin")
        .context("native store artwork")?;
    let bank = pack
        .data
        .tilesets
        .get_mut("mart")
        .context("store tileset")?;
    ensure!(
        atlas.len() == 64 * 16 && bank.collision.len() == 64,
        "unexpected store art bank"
    );

    // The native checkout spans tile columns 6–9. Move the two endcaps
    // from Goldenrod's wider lobby desk onto the ends of its straight lip.
    // Both lower quadrants remain the shop floor for the aisle behind it.
    let straight: [u8; 16] = atlas[0x0d * 16..0x0e * 16].try_into()?;
    let footing = bank.collision.get("0d").context("counter footing")?.clone();
    for (block, source, column, source_column) in [(0x40, 0x0c, 0, 2), (0x41, 0x0e, 2, 0)] {
        let mut endcap = straight;
        for row in [0, 4] {
            let start = source * 16 + row + source_column;
            endcap[row + column..row + column + 2].copy_from_slice(&atlas[start..start + 2]);
        }
        ensure!(
            bank.collision
                .insert(format!("{block:02x}"), footing.clone())
                .is_none(),
            "occupied store counter endcap"
        );
        atlas.extend(endcap);
    }
    room.blocks[3..5].fill(0x14); // Glass display cabinets along the back wall.
    room.blocks[11..13].fill(0x04); // The customer aisle shares the surrounding floor.
    room.blocks[27..29].copy_from_slice(&[0x40, 0x41]);
    Ok(())
}
