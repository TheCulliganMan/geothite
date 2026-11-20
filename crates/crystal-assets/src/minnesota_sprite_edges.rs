//! Remove reviewed neighboring-sheet fragments without replacing installed art.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::minnesota_pokemon_pixel_art::png_rgb;
use crate::minnesota_sprite_sheet::MinnesotaSheet;

/// Repair the six contaminated views in the installed Minnesota artwork.
///
/// Only the reviewed detached fragments and their matching 2bpp pixels change.
/// Returns an error for missing art, unexpected geometry, fragment counts, or
/// inconsistent PNG/2bpp backgrounds. The input pack remains unchanged.
pub fn repair_minnesota_sprite_edges(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut pack = base.clone();
    for (species, view, bounds, count, digest) in [
        (
            "scutlet",
            "back",
            [45, 44, 47, 48],
            6,
            "b4b0c162ee4dd08308fc2a24d124050126f263e3deca22301f00e620c2857659",
        ),
        (
            "dynarill",
            "front",
            [47, 39, 50, 52],
            24,
            "cbac3ff5d6d9300d99812bf5fd8caf64fcb08835eef1639d3c6819f54c2e6a03",
        ),
        (
            "mudnip",
            "front",
            [47, 36, 48, 55],
            8,
            "60a19b49738323570f8220056864246935701e42b5bbe8299d003c7723a6d0db",
        ),
        (
            "pikeling",
            "front",
            [44, 39, 47, 42],
            7,
            "7fb24e205b90034db61938a974b18a8be0ef609e5e33083559343d22d559c661",
        ),
        (
            "pikeman",
            "front",
            [52, 27, 53, 33],
            6,
            "28947f69b6ea8fd3a9ad04d2dfcbfe74a77b96c19d92c8a4cd63c0ce82ff0da1",
        ),
        (
            "crookit",
            "front",
            [47, 51, 48, 54],
            3,
            "f0d260e9c14a1bf8f995273c5bf7f56785718fbb1e4448c25c2711d2c1180748",
        ),
    ] {
        let stem = format!("gfx/pokemon/{species}/{view}");
        let png_key = format!("{stem}.png");
        let tile_key = format!("{stem}.2bpp");
        let sheet = MinnesotaSheet::from_png(
            pack.runtime_files
                .get(&png_key)
                .context("missing sprite PNG")?,
        )?;
        let size = if view == "back" { 48 } else { 56 };
        ensure!(
            sheet.width == size && sheet.height == size,
            "unexpected {stem} size"
        );
        let tiles = pack
            .runtime_files
            .get_mut(&tile_key)
            .context("missing sprite tiles")?;
        ensure!(
            tiles.len() == size * size / 4,
            "unexpected {stem} tile bank"
        );
        let mut pixels = Vec::with_capacity(size * size);
        let mut changed = 0;
        for y in 0..size {
            for x in 0..size {
                let mut pixel = sheet.pixel(x, y).context("sprite pixel")?;
                if (bounds[0]..bounds[2]).contains(&x) && (bounds[1]..bounds[3]).contains(&y) {
                    let tile = if view == "back" {
                        x / 8 * (size / 8) + y / 8
                    } else {
                        y / 8 * (size / 8) + x / 8
                    };
                    let offset = tile * 16 + y % 8 * 2;
                    let mask = 1 << (7 - x % 8);
                    let ink = (tiles[offset] & mask != 0) || (tiles[offset + 1] & mask != 0);
                    ensure!(
                        ink == (pixel != [255; 3]),
                        "{stem} PNG/tile background mismatch"
                    );
                    if ink {
                        changed += 1;
                        pixel = [255; 3];
                        tiles[offset] &= !mask;
                        tiles[offset + 1] &= !mask;
                    }
                }
                pixels.push(pixel);
            }
        }
        ensure!(
            changed == count || changed == 0,
            "unexpected {stem} fragment: {changed} pixels"
        );
        if changed != 0 {
            ensure!(
                format!("{:x}", Sha256::digest(&base.runtime_files[&png_key])) == digest,
                "unreviewed {stem} artwork"
            );
            pack.runtime_files
                .insert(png_key, png_rgb(size, size, &pixels)?);
        }
    }
    pack.identity = if pack.audio_compression.is_some() {
        crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?
    } else {
        crate::derive_compiled_game_pack_identity(
            pack.format_version,
            &pack.data,
            &pack.compiled_audio,
            &pack.runtime_files,
            &pack.report,
        )?
    };
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_edge_repair_preserves_pack_and_is_idempotent() -> Result<()> {
        let Some(path) = std::env::var_os("MINNESOTA_SPRITE_EDGE_PACK") else {
            return Ok(());
        };
        let base = crate::read_verified_compiled_game_pack(path)?;
        let fixed = repair_minnesota_sprite_edges(&base)?;
        let changed: Vec<_> = base
            .runtime_files
            .iter()
            .filter(|(key, bytes)| fixed.runtime_files.get(*key) != Some(*bytes))
            .map(|(key, _)| key.as_str())
            .collect();
        assert_eq!(changed.len(), 12, "only six PNG/2bpp pairs change");
        for key in changed {
            assert!(
                [
                    "scutlet/back",
                    "dynarill/front",
                    "mudnip/front",
                    "pikeling/front",
                    "pikeman/front",
                    "crookit/front"
                ]
                .iter()
                .any(|stem| key == format!("gfx/pokemon/{stem}.png")
                    || key == format!("gfx/pokemon/{stem}.2bpp"))
            );
        }
        let mut restored = fixed.clone();
        restored.runtime_files = base.runtime_files.clone();
        restored.identity = base.identity.clone();
        assert_eq!(
            restored, base,
            "all gameplay, audio and other pack fields stay exact"
        );
        assert_eq!(repair_minnesota_sprite_edges(&fixed)?, fixed);
        Ok(())
    }
}
