//! Reconstruct the original battle graphics build before indexing OAM tiles.
//!
//! The legacy external pack contains PNG-order 2bpp tiles, but OAM indexes the
//! output of pret/pokecrystal's Makefile and tools/gfx.c. Keep this transform
//! independent of OAM, palettes, object callbacks and presentation clocks.

type Tile = [u8; 16];

#[derive(Clone, Copy, Default)]
struct Rules {
    trim: bool,
    remove_blank: bool,
    duplicates: bool,
    xflip: bool,
    yflip: bool,
    keep_blank: bool,
}

#[cfg(all(test, feature = "bevy-shell"))]
mod raster_tests {
    use super::super::*;

    #[test]
    fn battle_sprite_preprocessing_reaches_shared_raster_with_signed_oam_and_flips() {
        let assets = AssetRoot::new_temporary_in(std::env::temp_dir()).unwrap();
        let graphics = assets.runtime_assets().join("gfx/battle_anims");
        std::fs::create_dir_all(&graphics).unwrap();
        let mut tile = [0; 16];
        tile[0] = 0x80; // (0, 0), shade 1
        tile[3] = 0x01; // (7, 1), shade 2
        tile[14] = 0x10; // (3, 7), shade 3
        tile[15] = 0x10;
        let raw: Vec<u8> = [[0; 16], tile, [0; 16]].into_iter().flatten().collect();
        std::fs::write(graphics.join("hit.2bpp"), raw).unwrap();
        std::fs::write(graphics.join("battle_anims.pal"),
            "; gray\nRGB 31,31,31\nRGB 25,25,25\nRGB 13,13,13\nRGB 0,0,0\n").unwrap();
        let object = serde_json::json!({"flags": 0x60, "gfx_id": "TEST", "palette": "PAL_BATTLE_OB_GRAY"});
        let frame = serde_json::json!({"oam_set": "TEST", "xflip": false, "yflip": false});
        let bundle = serde_json::json!({
            "gfx_table": {"TEST": [1, "TestHitGFX"]},
            "gfx_sources": {"TestHitGFX": "gfx/battle_anims/hit.2bpp.lz"},
            "oam_sets": {"TEST": {"tile_offset": 0, "entries": [{
                "x": -7, "y": -3, "tile_id": 0, "xflip": false, "yflip": false, "obp": 0
            }]}}
        });
        let mut art = RenderedTilesetArt::default();
        let mut images = Assets::<Image>::default();
        for enemy in [false, true] {
            let rendered = battle_anim_rendered_frame(
                &mut art, &bundle, &assets, "TEST", &object, "TEST", 0, &frame,
                enemy, false, false, None, 0xe4, 0xe4, None, &mut images,
            ).unwrap();
            assert_eq!((rendered.offset_x, rendered.offset_y),
                       if enemy { (-1, -5) } else { (-7, -3) });
            let image = images.get(&rendered.sprite.handle).unwrap();
            assert_eq!((image.width(), image.height()), (8, 8));
            let mut expected = vec![0; 8 * 8 * 4];
            for (x, y, shade) in [(0, 0, 206), (7, 1, 107), (3, 7, 0)] {
                let (x, y) = if enemy { (7 - x, 7 - y) } else { (x, y) };
                expected[(y * 8 + x) * 4..(y * 8 + x) * 4 + 4]
                    .copy_from_slice(&[shade, shade, shade, 255]);
            }
            assert_eq!(image.data, expected);
        }
    }
}

fn rules(name: &str) -> Rules {
    let mut rules = Rules::default();
    match name {
        "angels" | "bubble" | "charge" => rules.trim = true,
        "egg" | "explosion" | "hit" | "horn" | "lightning" | "noise"
        | "reflect" | "rocks" | "skyattack" | "status" => rules.remove_blank = true,
        "beam" => {
            rules.xflip = true;
            rules.yflip = true;
            rules.remove_blank = true;
        }
        "misc" => {
            rules.duplicates = true;
            rules.xflip = true;
        }
        "objects" => {
            rules.xflip = true;
            rules.remove_blank = true;
        }
        "pokeball" => {
            rules.xflip = true;
            rules.keep_blank = true;
        }
        _ => {}
    }
    rules
}

fn flipped(tile: &Tile, xflip: bool, yflip: bool) -> Tile {
    let mut output = [0; 16];
    for y in 0..8 {
        for plane in 0..2 {
            let byte = tile[y * 2 + plane];
            output[(if yflip { 7 - y } else { y }) * 2 + plane] =
                if xflip { byte.reverse_bits() } else { byte };
        }
    }
    output
}

fn remove_matching(tiles: &mut Vec<Tile>, xflip: bool, yflip: bool, keep_blank: bool) {
    let mut retained = Vec::with_capacity(tiles.len());
    for tile in tiles.iter() {
        if (keep_blank && *tile == [0; 16])
            || !retained.contains(&flipped(tile, xflip, yflip))
        {
            retained.push(*tile);
        }
    }
    *tiles = retained;
}

/// Apply only the original per-sheet build operations to a PNG-order stream.
/// This is deliberately not a generic tile optimizer: e.g. --remove-xflip
/// does not remove an identical asymmetric tile, and each flip pass compares
/// against the tiles retained by that pass, in source order.
pub(super) fn preprocess(name: &str, bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    if bytes.len() % 16 != 0 {
        return Err("battle animation graphics are not 2bpp tile aligned");
    }
    let rules = rules(name);
    let mut tiles: Vec<Tile> = bytes
        .chunks_exact(16)
        .map(|tile| tile.try_into().expect("complete 2bpp tile"))
        .collect();
    // gfx.c runs these passes in this order, regardless of Makefile flag order.
    if rules.trim {
        while tiles.len() > 1 && tiles.last() == Some(&[0; 16]) {
            tiles.pop();
        }
    }
    if rules.duplicates {
        remove_matching(&mut tiles, false, false, rules.keep_blank);
    }
    if rules.xflip {
        remove_matching(&mut tiles, true, false, rules.keep_blank);
    }
    if rules.yflip {
        remove_matching(&mut tiles, false, true, rules.keep_blank);
    }
    if rules.xflip && rules.yflip {
        remove_matching(&mut tiles, true, true, rules.keep_blank);
    }
    if rules.remove_blank {
        tiles.retain(|tile| *tile != [0; 16]);
    }
    Ok(tiles.into_iter().flatten().collect())
}

/// Correct legacy PNG-order pack entries, while accepting already-compiled
/// packs unchanged. The source graphics table declares the compiled size for
/// every sheet. Never guess by trimming to that size or padding missing art.
pub(super) fn normalize(
    name: &str,
    bytes: Vec<u8>,
    declared_tiles: usize,
) -> Result<Vec<u8>, &'static str> {
    let declared_bytes = declared_tiles
        .checked_mul(16)
        .ok_or("battle animation declared tile count overflows")?;
    if bytes.len() == declared_bytes {
        return Ok(bytes);
    }
    let processed = preprocess(name, &bytes)?;
    if processed.len() != declared_bytes {
        return Err("battle animation graphics do not match the declared compiled tile count");
    }
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(tiles: &[Tile]) -> Vec<u8> {
        tiles.iter().flatten().copied().collect()
    }

    fn asymmetric() -> Tile {
        [0x80, 0x10, 0x40, 0x08, 0x20, 0x04, 0x10, 0x02,
         0x08, 0x01, 0x04, 0x80, 0x02, 0x40, 0x01, 0x20]
    }

    #[test]
    fn hit_compacts_internal_blanks_without_reordering_or_deduplicating_ink() {
        let a = asymmetric();
        let mut b = a;
        b[0] ^= 0x10;
        assert_eq!(preprocess("hit", &bytes(&[[0; 16], a, [0; 16], b, a, [0; 16]])).unwrap(),
                   bytes(&[a, b, a]));
    }

    #[test]
    fn trimmed_sheets_keep_internal_blanks_and_at_least_one_tile() {
        let a = asymmetric();
        for name in ["angels", "bubble", "charge"] {
            assert_eq!(preprocess(name, &bytes(&[[0; 16], a, [0; 16], a, [0; 16]])).unwrap(),
                       bytes(&[[0; 16], a, [0; 16], a]));
            assert_eq!(preprocess(name, &bytes(&[[0; 16]; 3])).unwrap(), bytes(&[[0; 16]]));
        }
    }

    #[test]
    fn pokeball_keeps_blank_slots_and_identical_asymmetric_tiles() {
        let a = asymmetric();
        // These are independent literal bit reversals of the source tile.
        let x = [1, 8, 2, 16, 4, 32, 8, 64, 16, 128, 32, 1, 64, 2, 128, 4];
        assert_eq!(preprocess("pokeball", &bytes(&[a, x, [0; 16], [0; 16], a])).unwrap(),
                   bytes(&[a, [0; 16], [0; 16], a]));
    }

    #[test]
    fn beam_runs_x_y_and_xy_passes_before_removing_blanks() {
        let a = asymmetric();
        let x = [1, 8, 2, 16, 4, 32, 8, 64, 16, 128, 32, 1, 64, 2, 128, 4];
        let y = [1, 32, 2, 64, 4, 128, 8, 1, 16, 2, 32, 4, 64, 8, 128, 16];
        let xy = [128, 4, 64, 2, 32, 1, 16, 128, 8, 64, 4, 32, 2, 16, 1, 8];
        assert_eq!(preprocess("beam", &bytes(&[a, x, y, xy, [0; 16], a])).unwrap(),
                   bytes(&[a, a]));
    }

    #[test]
    fn misc_removes_exact_duplicates_then_mirrors_but_keeps_one_blank() {
        let a = asymmetric();
        let x = [1, 8, 2, 16, 4, 32, 8, 64, 16, 128, 32, 1, 64, 2, 128, 4];
        assert_eq!(preprocess("misc", &bytes(&[a, a, x, [0; 16], [0; 16]])).unwrap(),
                   bytes(&[a, [0; 16]]));
    }

    #[test]
    fn unaffected_sheets_retain_every_source_tile() {
        let source = bytes(&[[0; 16], asymmetric(), asymmetric(), [0; 16]]);
        for name in ["aeroblast", "cut", "fire", "flower", "globe", "haze", "ice",
                     "plant", "poison", "powder", "psychic", "rope", "sand", "shapes",
                     "shine", "smoke", "speed", "water", "wave", "web", "whip", "wind"] {
            assert_eq!(preprocess(name, &source).unwrap(), source, "{name}");
        }
    }

    #[test]
    fn compiled_streams_are_not_processed_twice() {
        // Even an intentionally compiled blank remains authoritative.
        let compiled = bytes(&[asymmetric(), [0; 16]]);
        assert_eq!(normalize("hit", compiled.clone(), 2).unwrap(), compiled);
        assert_eq!(normalize("hit", bytes(&[[0; 16], asymmetric(), [0; 16]]), 1).unwrap(),
                   bytes(&[asymmetric()]));
    }

    #[test]
    fn malformed_graphics_fail_instead_of_cropping_or_inventing_tiles() {
        assert!(normalize("hit", vec![0; 17], 1).is_err());
        assert!(normalize("hit", bytes(&[asymmetric()]), 2).is_err());
        assert!(normalize("unknown", bytes(&[asymmetric(); 2]), 1).is_err());
        assert!(normalize("wind", vec![], usize::MAX).is_err());
    }
}
