//! Render local Crystal sprite references into ignored authoring review output.
#[allow(dead_code)]
#[path = "../src/minnesota_pokemon_pixel_art.rs"]
mod minnesota_pokemon_pixel_art;

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::read_verified_compiled_game_pack;
use minnesota_pokemon_pixel_art::png_rgb;

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let pack = read_verified_compiled_game_pack(root.join("content-packs/text-tui.crystalpack"))?;
    let files = pack.runtime_files();
    let output = root.join("target/minnesota-pokemon/gen2-reference");
    fs::create_dir_all(&output)?;
    for (name, species) in [
        ("starters", ["chikorita", "cyndaquil", "totodile"]),
        ("bugs", ["ledian", "scizor", "heracross"]),
    ] {
        let mut pixels = vec![[255_u8; 3]; 192 * 128];
        for (column, id) in species.iter().enumerate() {
            let key = |suffix: &str| format!("gfx/pokemon/{id}/{suffix}");
            let read = |suffix: &str| files.get(&key(suffix)).with_context(|| key(suffix));
            let palette_bytes = read("normal.gbcpal")?;
            ensure!(palette_bytes.len() == 8, "four RGB555 colors required");
            let palette: [[u8; 3]; 4] = std::array::from_fn(|i| {
                let word = u16::from_le_bytes([palette_bytes[i * 2], palette_bytes[i * 2 + 1]]);
                [0, 5, 10].map(|shift| {
                    let v = ((word >> shift) & 31) as u8;
                    (v << 3) | (v >> 2)
                })
            });
            let dimensions = read("front.dimensions")?[0];
            for (row, suffix, tiles_w, tiles_h) in [
                (
                    0,
                    "front.2bpp",
                    usize::from(dimensions & 15),
                    usize::from(dimensions >> 4),
                ),
                (1, "back.2bpp", 6, 6),
            ] {
                let bytes = read(suffix)?;
                ensure!(
                    bytes.len() >= tiles_w * tiles_h * 16,
                    "incomplete first frame"
                );
                let width = tiles_w * 8;
                let height = tiles_h * 8;
                let left = column * 64 + (64 - width) / 2;
                let top = row * 64 + (64 - height) / 2;
                for y in 0..height {
                    for x in 0..width {
                        let tile = if row == 1 {
                            (x / 8) * tiles_h + y / 8
                        } else {
                            (y / 8) * tiles_w + x / 8
                        };
                        let offset = tile * 16 + (y % 8) * 2;
                        let bit = 7 - x % 8;
                        let index =
                            ((bytes[offset] >> bit) & 1) | (((bytes[offset + 1] >> bit) & 1) << 1);
                        pixels[(top + y) * 192 + left + x] = palette[usize::from(index)];
                    }
                }
            }
        }
        let mut zoomed = Vec::with_capacity(192 * 128 * 64);
        for y in 0..128 * 8 {
            for x in 0..192 * 8 {
                zoomed.push(pixels[(y / 8) * 192 + x / 8]);
            }
        }
        fs::write(
            output.join(format!("{name}.png")),
            png_rgb(1536, 1024, &zoomed)?,
        )?;
    }
    println!(
        "Wrote local style references to {}. No source pack was modified.",
        output.display()
    );
    Ok(())
}
