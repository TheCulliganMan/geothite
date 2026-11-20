use anyhow::Result;

use super::{NativePortrait, encode, replacement_files};

fn previous() -> NativePortrait {
    serde_json::from_str(include_str!(
        "../../../planning/minnesota/npcs/tamarack-ice/01-tamarack/native-pixels-v4.json"
    ))
    .expect("native professor source")
}

fn revised(previous: &NativePortrait) -> NativePortrait {
    let mut revised = previous.clone();
    let row = &mut revised.battle_rows[20];
    let column = row.find('1').expect("existing light pixel");
    row.replace_range(column..column + 1, "2");
    revised
}

#[test]
fn battle_revision_rejects_stale_banks_and_field_changes() -> Result<()> {
    let previous = previous();
    let revised = revised(&previous);
    let installed = encode(&previous)?;
    let replacements = replacement_files(&installed, &previous, &revised)?;
    assert_eq!(replacements.len(), 3);
    assert!(
        replacements
            .keys()
            .all(|key| key.starts_with("gfx/trainers/"))
    );
    for extension in ["png", "2bpp", "gbcpal"] {
        let key = format!("gfx/trainers/{}.{}", previous.id, extension);
        let mut stale = installed.clone();
        stale.insert(key, Vec::new());
        assert!(replacement_files(&stale, &previous, &revised).is_err());
    }
    let mut field_change = revised.clone();
    field_change.overworld_frames[0].rows[0] = "3".repeat(16);
    assert!(replacement_files(&installed, &previous, &field_change).is_err());
    let mut palette_change = revised;
    palette_change.palette_rgb555[1] -= 1;
    assert!(replacement_files(&installed, &previous, &palette_change).is_err());
    assert!(replacement_files(&installed, &previous, &previous).is_err());
    Ok(())
}

#[test]
fn native_portrait_indices_roundtrip_independently() -> Result<()> {
    let source = previous();
    let encoded = encode(&source)?;
    let tiles = &encoded[&format!("gfx/trainers/{}.2bpp", source.id)];
    assert_eq!(tiles.len(), 784);
    for (y, row) in source.battle_rows.iter().enumerate() {
        for (x, symbol) in row.bytes().enumerate() {
            let offset = ((y / 8) * 7 + x / 8) * 16 + (y % 8) * 2;
            let shift = 7 - x % 8;
            let decoded =
                ((tiles[offset] >> shift) & 1) | (((tiles[offset + 1] >> shift) & 1) << 1);
            assert_eq!(decoded, if symbol == b'.' { 0 } else { symbol - b'0' });
        }
    }
    Ok(())
}

#[test]
fn preimage_accepts_equivalent_png_encoding_but_rejects_alpha_changes() -> Result<()> {
    let previous = previous();
    let revised = revised(&previous);
    let mut installed = encode(&previous)?;
    let key = format!("gfx/trainers/{}.png", previous.id);
    let decoded = super::MinnesotaSheet::from_png(&installed[&key])?;
    let mut pixels = Vec::with_capacity(56 * 56);
    for y in 0..56 {
        for x in 0..56 {
            let [r, g, b] = decoded.pixel(x, y).expect("native pixel");
            pixels.push([r, g, b, 255]);
        }
    }
    installed.insert(
        key.clone(),
        crate::minnesota_pokemon_pixel_art::png_rgba(56, 56, &pixels)?,
    );
    replacement_files(&installed, &previous, &revised)?;
    pixels[0][3] = 0;
    installed.insert(
        key,
        crate::minnesota_pokemon_pixel_art::png_rgba(56, 56, &pixels)?,
    );
    assert!(replacement_files(&installed, &previous, &revised).is_err());
    Ok(())
}

#[test]
#[ignore = "requires CRYSTAL_PORTRAIT_PARENT and CRYSTAL_PORTRAIT_PLAN"]
fn trainer_portrait_revision_preserves_every_unrelated_pack_field() -> Result<()> {
    let parent = std::env::var_os("CRYSTAL_PORTRAIT_PARENT").expect("explicit parent pack");
    let plan = std::env::var_os("CRYSTAL_PORTRAIT_PLAN").expect("explicit portrait plan");
    let base = crate::read_verified_compiled_game_pack(parent)?;
    let source = std::fs::read_to_string(plan)?;
    let revised = super::revise_minnesota_trainer_portraits(&base, &source)?;
    assert_ne!(
        base.identity, revised.identity,
        "art revision changes save identity"
    );
    assert_eq!(base.runtime_files.len(), revised.runtime_files.len());
    let changed: Vec<_> = base
        .runtime_files
        .iter()
        .filter(|(key, bytes)| revised.runtime_files.get(*key) != Some(*bytes))
        .map(|(key, _)| key)
        .collect();
    assert!(!changed.is_empty());
    assert!(changed.iter().all(|key| key.starts_with("gfx/trainers/mn_")
        && (key.ends_with(".png") || key.ends_with(".2bpp"))));
    let mut normalized = revised;
    normalized.runtime_files.clone_from(&base.runtime_files);
    normalized.identity.clone_from(&base.identity);
    assert_eq!(
        normalized, base,
        "gameplay, field banks, audio and metadata stay exact"
    );
    assert!(
        super::revise_minnesota_trainer_portraits(
            &normalized,
            r#"{"schema_version":1,"portraits":[]}"#
        )
        .is_err()
    );
    Ok(())
}
