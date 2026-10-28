//! Species-specific, three-ink native overworld frames. NPC art stays separate.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use bevy::prelude::{Assets, Image};
use crystal_assets::AssetRoot;
use image::{Rgba, RgbaImage};

use crate::core::models::pokemon::Pokemon;

use super::{Palette, SpriteArt, SpriteFrame};

const PREFIX: &str = "pokemon_overworld:";
const FRAME_SIZE: u32 = 16;
const FRAME_COUNT: usize = 6;
const BANK_BYTES: usize = FRAME_COUNT * 64;

fn valid_token(token: &str) -> bool {
    !token.is_empty()
        && token
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn root_for(assets: &AssetRoot, token: &str) -> PathBuf {
    assets.runtime_assets().join("gfx/pokemon").join(token)
}

/// Resolve only an actually installed custom bank; callers validate species identity.
pub(super) fn sprite_id_if_present(assets: &AssetRoot, species: &str) -> Option<String> {
    let token = species.to_ascii_lowercase();
    (valid_token(&token)
        && crate::runtime_asset_exists(root_for(assets, &token).join("overworld.2bpp")))
    .then(|| format!("{PREFIX}{token}:normal"))
}

/// Select shiny art only for a species bank, leaving ordinary NPC IDs untouched.
pub(super) fn shiny_variant(sprite_id: &str) -> Option<String> {
    sprite_id
        .strip_prefix(PREFIX)?
        .strip_suffix(":normal")
        .map(|token| format!("{PREFIX}{token}:shiny"))
}

/// Use each party member's own species and appearance; eggs retain the egg icon.
pub(super) fn party_icon_id_if_present(assets: &AssetRoot, pokemon: &Pokemon) -> Option<String> {
    if pokemon.is_egg {
        return None;
    }
    let normal = sprite_id_if_present(assets, &pokemon.species.id)?;
    if super::visible_pokemon_is_shiny(pokemon) {
        shiny_variant(&normal)
    } else {
        Some(normal)
    }
}

/// Load a namespaced species bank through the existing directional sprite pipeline.
/// Returns `None` for legacy NPC IDs. Malformed custom banks are errors, never
/// permission to substitute a shared icon or a battle picture.
pub(super) fn load(
    assets: &AssetRoot,
    sprite_id: &str,
    images: &mut Assets<Image>,
) -> Result<Option<SpriteArt>> {
    let Some((source, palette)) = read_source(assets, sprite_id)? else {
        return Ok(None);
    };
    super::map_sprite_sheet_frames(
        sprite_id,
        &source,
        FRAME_SIZE as usize,
        FRAME_COUNT,
        &palette,
        images,
    )
    .map(Some)
}

/// Reuse the authored front idle/step frames at native menu-icon size.
/// Legacy IDs return `None`; malformed installed species art returns an error.
pub(super) fn load_party_frames(
    assets: &AssetRoot,
    sprite_id: &str,
    images: &mut Assets<Image>,
) -> Result<Option<[SpriteFrame; 2]>> {
    let Some((source, palette)) = read_source(assets, sprite_id)? else {
        return Ok(None);
    };
    Ok(Some([0, 3].map(|frame| {
        super::create_sprite_frame(&source, FRAME_SIZE as usize, frame, &palette, false, images)
    })))
}

fn read_source(assets: &AssetRoot, sprite_id: &str) -> Result<Option<(RgbaImage, Palette)>> {
    let Some(key) = sprite_id.strip_prefix(PREFIX) else {
        return Ok(None);
    };
    let (token, appearance) = key
        .split_once(':')
        .context("species overworld appearance")?;
    ensure!(valid_token(token), "invalid species overworld token");
    let palette_file = match appearance {
        "normal" => "overworld.gbcpal",
        "shiny" => "overworld-shiny.gbcpal",
        _ => anyhow::bail!("invalid species overworld appearance"),
    };
    let root = root_for(assets, token);
    let bytes = crate::read_runtime_asset(root.join("overworld.2bpp"))
        .with_context(|| format!("read species overworld bank {token}"))?;
    let palette_bytes = crate::read_runtime_asset(root.join(palette_file))
        .with_context(|| format!("read species overworld palette {token}/{palette_file}"))?;
    let palette = decode_palette(&palette_bytes)?;
    let source = decode_bank(&bytes)?;
    Ok(Some((source, palette)))
}

fn decode_palette(bytes: &[u8]) -> Result<Palette> {
    ensure!(
        bytes.len() == 8,
        "species overworld palette must be eight bytes"
    );
    let mut palette = [[0; 3]; 4];
    for (index, color) in palette.iter_mut().enumerate() {
        let offset = index * 2;
        let word = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        ensure!(
            word <= 0x7fff,
            "species overworld palette must use RGB555 words"
        );
        *color = super::gbc_color_to_rgb(word);
    }
    Ok(palette)
}

fn decode_bank(bytes: &[u8]) -> Result<RgbaImage> {
    ensure!(
        bytes.len() == BANK_BYTES,
        "species overworld bank must contain six 16x16 frames (384 bytes)"
    );
    let mut source = RgbaImage::new(FRAME_SIZE, FRAME_SIZE * FRAME_COUNT as u32);
    for frame in 0..FRAME_COUNT {
        ensure!(
            bytes[frame * 64..(frame + 1) * 64]
                .iter()
                .any(|byte| *byte != 0),
            "species overworld frame {frame} is blank"
        );
        for y in 0..16 {
            for x in 0..16 {
                let offset = (frame * 4 + (y / 8) * 2 + x / 8) * 16 + (y % 8) * 2;
                let shift = 7 - x % 8;
                let index =
                    ((bytes[offset] >> shift) & 1) | (((bytes[offset + 1] >> shift) & 1) << 1);
                let gray = [255, 170, 85, 0][usize::from(index)];
                // Literal OBJ alpha is encoded before the legacy PNG mapper.
                // Enclosed index-zero pixels therefore never become opaque white.
                source.put_pixel(
                    x as u32,
                    (frame * 16 + y) as u32,
                    Rgba([gray, gray, gray, if index == 0 { 0 } else { 255 }]),
                );
            }
        }
    }
    Ok(source)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::{Assets, Image};
    use crystal_assets::AssetRoot;

    use super::{
        BANK_BYTES, decode_bank, decode_palette, load, load_party_frames, root_for, shiny_variant,
    };

    fn fixture() -> (AssetRoot, Vec<u8>) {
        let root = AssetRoot::new_temporary_in(std::env::temp_dir()).expect("exclusive test mount");
        let folder = root_for(&root, "pouchid");
        std::fs::create_dir_all(&folder).expect("test art directory");
        let mut bytes = vec![0; BANK_BYTES];
        for frame in 0..6 {
            for y in 2..14 {
                for x in 2..14 {
                    let index = if (x, y) == (8, 8) {
                        0
                    } else if (x, y) == (2 + frame, 3) {
                        3
                    } else {
                        1
                    };
                    let offset = (frame * 4 + (y / 8) * 2 + x / 8) * 16 + y % 8 * 2;
                    let shift = 7 - x % 8;
                    bytes[offset] |= (index & 1) << shift;
                    bytes[offset + 1] |= ((index >> 1) & 1) << shift;
                }
            }
        }
        std::fs::write(folder.join("overworld.2bpp"), &bytes).expect("test bank");
        for (name, words) in [
            ("overworld.gbcpal", [32767u16, 31, 992, 0]),
            ("overworld-shiny.gbcpal", [32767u16, 31744, 1023, 0]),
        ] {
            let palette = words
                .into_iter()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>();
            std::fs::write(folder.join(name), palette).expect("test palette");
        }
        (root, bytes)
    }

    fn pixels<'a>(images: &'a Assets<Image>, frame: &super::super::SpriteFrame) -> &'a [u8] {
        let image = images
            .get(&frame.handle)
            .expect("production native texture");
        assert_eq!((image.width(), image.height()), (16, 16));
        image.data.as_deref().expect("readable native pixels")
    }

    #[test]
    fn species_overworld_native_raster_preserves_six_poses_alpha_mirror_and_shiny() {
        let (root, _) = fixture();
        let mut images = Assets::<Image>::default();
        let normal = load(&root, "pokemon_overworld:pouchid:normal", &mut images)
            .expect("native loader")
            .expect("custom bank");
        let shiny = load(&root, "pokemon_overworld:pouchid:shiny", &mut images)
            .expect("shiny loader")
            .expect("custom bank");
        let normal_frames = [
            &normal.down.standing,
            &normal.up.standing,
            &normal.left.standing,
            normal.down.walking.as_ref().expect("down step"),
            normal.up.walking.as_ref().expect("up step"),
            normal.left.walking.as_ref().expect("left step"),
        ];
        let fingerprints = normal_frames
            .iter()
            .map(|frame| pixels(&images, frame).to_vec())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            fingerprints.len(),
            6,
            "all authored poses reach native textures"
        );
        for frame in normal_frames {
            let rgba = pixels(&images, frame);
            assert_eq!(
                rgba[(8 * 16 + 8) * 4 + 3],
                0,
                "enclosed OBJ zero stays transparent"
            );
            assert_eq!(
                &rgba[(4 * 16 + 4) * 4..(4 * 16 + 4) * 4 + 4],
                &[255, 0, 0, 255]
            );
        }
        assert_eq!(
            &pixels(&images, &shiny.down.standing)[(4 * 16 + 4) * 4..(4 * 16 + 4) * 4 + 4],
            &[0, 0, 255, 255]
        );
        for (left, right) in [
            (&normal.left.standing, &normal.right.standing),
            (
                normal.left.walking.as_ref().expect("left step"),
                normal.right.walking.as_ref().expect("right step"),
            ),
        ] {
            let left = pixels(&images, left);
            let right = pixels(&images, right);
            for y in 0..16 {
                for x in 0..16 {
                    assert_eq!(
                        &left[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4],
                        &right[(y * 16 + 15 - x) * 4..(y * 16 + 15 - x) * 4 + 4]
                    );
                }
            }
        }
    }

    #[test]
    fn party_icons_reuse_only_front_idle_and_step_with_own_palette_and_alpha() {
        let (root, _) = fixture();
        let mut images = Assets::<Image>::default();
        for appearance in ["normal", "shiny"] {
            let frames = load_party_frames(
                &root,
                &format!("pokemon_overworld:pouchid:{appearance}"),
                &mut images,
            )
            .expect("valid party bank")
            .expect("species-specific party frames");
            for (frame, pupil_x) in frames.iter().zip([2, 5]) {
                let rgba = pixels(&images, frame);
                assert_eq!(
                    frame.size,
                    super::super::Vec2::splat(super::super::TILE_SIZE * 2.0)
                );
                assert_eq!(&rgba[(3 * 16 + pupil_x) * 4..][..4], &[0, 0, 0, 255]);
                assert_eq!(rgba[(8 * 16 + 8) * 4 + 3], 0, "enclosed alpha hole");
                assert_eq!(
                    &rgba[(4 * 16 + 4) * 4..][..4],
                    if appearance == "shiny" {
                        &[0, 0, 255, 255]
                    } else {
                        &[255, 0, 0, 255]
                    }
                );
            }
        }
        assert_eq!(images.len(), 4, "only two textures per cached appearance");
        assert!(
            load_party_frames(&root, "ICON_MONSTER", &mut images)
                .expect("legacy dispatch")
                .is_none()
        );
        std::fs::write(root_for(&root, "pouchid").join("overworld.2bpp"), [0; 383])
            .expect("malformed fixture");
        assert!(load_party_frames(&root, "pokemon_overworld:pouchid:normal", &mut images).is_err());
    }

    #[test]
    fn custom_species_resolution_obeys_visible_replacements_and_rejects_invalid_banks() {
        let (root, bytes) = fixture();
        let icons = BTreeMap::from([("POUCHID".into(), "ICON_ODDISH".into())]);
        assert_eq!(
            super::super::resolve_visible_object_sprite_asset_id(
                &root,
                "SPRITE_POUCHID",
                &BTreeMap::new(),
                &icons
            ),
            "pokemon_overworld:pouchid:normal"
        );
        assert_eq!(
            super::super::resolve_visible_object_sprite_asset_id(
                &root,
                "SPRITE_POUCHID",
                &BTreeMap::from([("SPRITE_POUCHID".into(), "SPRITE_YOUNGSTER".into())]),
                &icons
            ),
            "youngster"
        );
        assert_eq!(
            shiny_variant("pokemon_overworld:pouchid:normal").as_deref(),
            Some("pokemon_overworld:pouchid:shiny")
        );
        assert!(shiny_variant("youngster").is_none());
        let mut images = Assets::<Image>::default();
        assert!(
            load(&root, "youngster", &mut images)
                .expect("legacy dispatch")
                .is_none()
        );
        assert!(load(&root, "pokemon_overworld:../pouchid:normal", &mut images).is_err());
        assert!(load(&root, "pokemon_overworld:pouchid:day", &mut images).is_err());
        assert!(decode_bank(&bytes[..383]).is_err());
        assert!(decode_bank(&[0; 384]).is_err());
        assert!(decode_palette(&[0; 7]).is_err());
        assert!(decode_palette(&[255; 8]).is_err());
    }
}
