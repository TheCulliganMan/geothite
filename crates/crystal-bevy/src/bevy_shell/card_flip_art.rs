//! Decode Card Flip's compiled tile order without changing its source PNG art.

use anyhow::{Context, Result, ensure};
use crystal_assets::AssetRoot;
use image::RgbaImage;

pub(super) fn load(
    asset_root: &AssetRoot,
    name: &str,
    width: u32,
    height: u32,
    tiles: usize,
) -> Result<RgbaImage> {
    let path = asset_root
        .runtime_assets()
        .join(format!("gfx/card_flip/{name}.2bpp"));
    let bytes = crate::read_runtime_asset(path).context("compiled Card Flip native tile bank")?;
    ensure!(
        bytes.len() == tiles * 16,
        "unexpected compiled Card Flip bank size"
    );
    decode(&bytes, width, height)
}

fn decode(bytes: &[u8], width: u32, height: u32) -> Result<RgbaImage> {
    ensure!(
        width.is_multiple_of(8)
            && height.is_multiple_of(8)
            && bytes.len().is_multiple_of(16)
            && bytes.len() / 16 <= (width * height / 64) as usize,
        "invalid Card Flip atlas dimensions"
    );
    let mut image = RgbaImage::from_pixel(width, height, image::Rgba([255; 4]));
    for (tile, data) in bytes.as_chunks::<16>().0.iter().enumerate() {
        for y in 0..8 {
            for x in 0..8 {
                let index =
                    ((data[y * 2] >> (7 - x)) & 1) | (((data[y * 2 + 1] >> (7 - x)) & 1) << 1);
                let gray = [255, 170, 85, 0][usize::from(index)];
                let px = tile as u32 % (width / 8) * 8 + x as u32;
                let py = tile as u32 / (width / 8) * 8 + y as u32;
                image.put_pixel(px, py, image::Rgba([gray, gray, gray, 255]));
            }
        }
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    #[test]
    fn compiled_tile_order_and_both_bitplanes_survive_decode() {
        let mut native = vec![0; 32];
        native[0] = 0b1010_0000;
        native[1] = 0b0110_0000;
        native[16] = 0xff;
        let image = super::decode(&native, 24, 8).expect("two-tile native bank");
        for (x, gray) in [(0, 170), (1, 85), (2, 0), (3, 255), (8, 170), (16, 255)] {
            assert_eq!(image.get_pixel(x, 0).0, [gray, gray, gray, 255]);
        }
    }
}
