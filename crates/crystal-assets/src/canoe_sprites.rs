//! Native canoe banks, using source trainer heads and explicitly authored hulls.
use anyhow::{Context, Result};

use crate::CompiledGamePack;
use crate::minnesota_overworld_sprites::decode_overworld_bitplanes;
use crate::minnesota_pokemon_pixel_art::png_rgba;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    for trainer in ["chris", "kris"] {
        let source = pack
            .runtime_files
            .get(&format!("gfx/sprites/{trainer}.2bpp"))
            .context("native trainer sprite bank")?;
        let bank = canoe_bank(source)?;
        pack.runtime_files
            .insert(format!("gfx/sprites/mn_canoe_{trainer}.png"), bank);
        // Dedicated four-color cedar/cream palette, independent of trainer NPC banks.
        pack.runtime_files.insert(
            format!("gfx/sprites/mn_canoe_{trainer}.gbcpal"),
            [0x7fffu16, 0x3b5f, 0x15d3, 0x0842]
                .into_iter()
                .flat_map(u16::to_le_bytes)
                .collect(),
        );
    }
    pack.runtime_files
        .insert("gfx/sprites/mn_canoe_rack.png".into(), display_bank()?);
    pack.runtime_files.insert(
        "gfx/sprites/mn_canoe_rack.gbcpal".into(),
        [0x7fffu16, 0x3b5f, 0x15d3, 0x0842]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect(),
    );
    pack.data
        .sprite_palette_defaults
        .insert("SPRITE_MN_CANOE_RACK".into(), 1);
    Ok(())
}

fn display_bank() -> Result<Vec<u8>> {
    let rows = [
        "......3333......",
        ".....322223.....",
        "....32111123....",
        "....32111123....",
        "...3211111123...",
        "...3211111123...",
        "...3222222223...",
        "...3211111123...",
        "...3211111123...",
        "...3222222223...",
        "...3211111123...",
        "...3211111123...",
        "....32111123....",
        "....32111123....",
        "...3332222333...",
        "...33.3333.33...",
    ];
    let gray = [255, 170, 85, 0];
    let pixels = rows
        .iter()
        .flat_map(|row| {
            row.bytes().map(|byte| {
                let value = if byte == b'.' {
                    255
                } else {
                    gray[usize::from(byte - b'0')]
                };
                [value, value, value, 255]
            })
        })
        .collect::<Vec<_>>();
    png_rgba(16, 16, &pixels)
}

// Hand-authored hulls: down/up/left, followed by their paddle-stroke frames.
// White is transparent in the native sprite loader; all remaining pixels use
// the standard four-shade source indices. Retain the actual trainer's head.
fn canoe_bank(trainer: &[u8]) -> Result<Vec<u8>> {
    let source = decode_overworld_bitplanes(trainer)?;
    let mut pixels = vec![[255, 255, 255, 255]; 16 * 96];
    let gray = [255, 170, 85, 0];
    let hulls = [
        [
            "......3333......",
            ".....322223.....",
            "....32111123....",
            "...3211111123...",
            "..321111111123..",
            "..321111111123..",
            "...3211111123...",
            "....32111123....",
            ".....321123.....",
            "......3223......",
            ".......33.......",
        ],
        [
            ".......33.......",
            "......3223......",
            ".....321123.....",
            "....32111123....",
            "...3211111123...",
            "..321111111123..",
            "..321111111123..",
            "...3211111123...",
            "....32222223....",
            ".....322223.....",
            "......3333......",
        ],
        [
            "................",
            "................",
            "................",
            "................",
            "3.............3.",
            "323333333333323.",
            ".3211111111123..",
            "..32222222223...",
            "...322222223....",
            "....3333333.....",
            "................",
        ],
    ];
    for frame in 0..6 {
        let facing = frame % 3;
        for (y, row) in hulls[facing].iter().enumerate() {
            for (x, byte) in row.bytes().enumerate() {
                if byte != b'.' {
                    let value = gray[usize::from(byte - b'0')];
                    pixels[(frame * 16 + y + 5) * 16 + x] = [value, value, value, 255];
                }
            }
        }
        // Seat the rider; legs remain inside the hull instead of walking on it.
        for y in 0..10 {
            for x in 3..13 {
                let value = gray[usize::from(source[facing * 256 + y * 16 + x])];
                if value != 255 {
                    pixels[(frame * 16 + y) * 16 + x] = [value, value, value, 255];
                }
            }
        }
        // Two bent knees and separated boot tips stay tucked inside the bow.
        // Only the forward view needs this outline beneath the native face.
        if facing == 0 {
            for (row, outline) in ["321123", "313313", "33..33"].iter().enumerate() {
                for (column, byte) in outline.bytes().enumerate() {
                    if byte != b'.' {
                        let value = gray[usize::from(byte - b'0')];
                        pixels[(frame * 16 + 10 + row) * 16 + 5 + column] =
                            [value, value, value, 255];
                    }
                }
            }
        }
        // A diagonal shaft and broad blade distinguish paddling from walking.
        let paddle = if frame < 3 {
            [
                (10, 7),
                (11, 8),
                (12, 9),
                (13, 10),
                (14, 11),
                (14, 12),
                (15, 12),
                (15, 13),
            ]
        } else {
            [
                (5, 7),
                (4, 8),
                (3, 9),
                (2, 10),
                (1, 11),
                (0, 12),
                (1, 12),
                (0, 13),
            ]
        };
        for (index, (x, y)) in paddle.into_iter().enumerate() {
            let value = if index >= 4 { 85 } else { 0 };
            pixels[(frame * 16 + y) * 16 + x] = [value, value, value, 255];
        }
    }
    png_rgba(16, 96, &pixels)
}

#[cfg(test)]
mod tests {
    use super::canoe_bank;
    use crate::minnesota_sprite_sheet::MinnesotaSheet;

    #[test]
    fn canoe_sprites_read_native_bitplanes_and_retain_the_seated_head() {
        let mut source = [0; 384];
        source[8] = 0x08;
        source[9] = 0x08;
        let bytes = canoe_bank(&source).expect("native six-frame trainer bank");
        let image = MinnesotaSheet::from_png(&bytes).expect("generated RGBA sprite bank");
        assert_eq!((image.width, image.height), (16, 96));
        assert_eq!(image.pixel(4, 4), Some([0; 3]));
        assert_eq!(image.pixel(4, 52), Some([0; 3]));
        assert_ne!(image.pixel(13, 10), image.pixel(13, 58));
        assert!(canoe_bank(&source[..383]).is_err());
    }
}
