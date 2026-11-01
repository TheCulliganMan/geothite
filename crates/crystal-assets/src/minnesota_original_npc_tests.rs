//! External-pack checks for revising one authored NPC without altering gameplay.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::minnesota_pokemon_pixel_art::{bitplanes, colored, png_rgb};
use crate::minnesota_sprite_sheet::MinnesotaSheet;
use crate::{read_verified_compiled_game_pack, revise_original_npc_overworld};

#[test]
#[ignore = "requires CRYSTAL_NPC_REVISION_PARENT, SOURCE and ART fixtures"]
fn original_npc_overworld_revision_guards() {
    let fixture = |name: &str| PathBuf::from(std::env::var_os(name).expect("explicit NPC fixture"));
    let base = read_verified_compiled_game_pack(fixture("CRYSTAL_NPC_REVISION_PARENT"))
        .expect("verified parent pack");
    let source = std::fs::read_to_string(fixture("CRYSTAL_NPC_REVISION_SOURCE"))
        .expect("single existing NPC placement");
    let placement: serde_json::Value = serde_json::from_str(&source).expect("placement JSON");
    let class = placement["trainers"][0]["trainer_class"]
        .as_str()
        .expect("existing original class");
    let stem = class.to_ascii_lowercase();
    let root = fixture("CRYSTAL_NPC_REVISION_ART");
    let mut art = BTreeMap::new();
    for kind in ["sprites", "trainers"] {
        for extension in ["png", "2bpp", "gbcpal"] {
            let path = format!("gfx/{kind}/{stem}.{extension}");
            art.insert(
                path.clone(),
                std::fs::read(root.join(path)).expect("native artifact"),
            );
        }
    }
    let revised = revise_original_npc_overworld(&base, &source, &art).expect("valid hand edit");
    assert_eq!(revised.data(), base.data(), "all gameplay data survives");
    assert_eq!(
        revised.compiled_audio(),
        base.compiled_audio(),
        "all audio survives"
    );
    assert_eq!(revised.runtime_files().len(), base.runtime_files().len());
    assert_ne!(
        revised.identity().expect("revised identity"),
        base.identity().expect("parent identity")
    );
    let changed: Vec<_> = base
        .runtime_files()
        .iter()
        .filter(|(path, bytes)| revised.runtime_files().get(*path) != Some(*bytes))
        .map(|(path, _)| path.clone())
        .collect();
    assert_eq!(
        changed,
        vec![
            format!("gfx/sprites/{stem}.2bpp"),
            format!("gfx/sprites/{stem}.png")
        ],
        "only the requested native overworld pixels and tiles may change"
    );
    assert!(
        revise_original_npc_overworld(&revised, &source, &art)
            .expect_err("no-op revision must fail")
            .to_string()
            .contains("must change")
    );
    let unknown = source.replace(class, "MN_UNREGISTERED_NPC");
    assert!(revise_original_npc_overworld(&base, &unknown, &art).is_err());
    let mut missing = art.clone();
    missing.remove(&format!("gfx/sprites/{stem}.gbcpal"));
    assert!(
        revise_original_npc_overworld(&base, &source, &missing)
            .expect_err("incomplete native bundle must fail")
            .to_string()
            .contains("exact native art bundle")
    );

    // This battle edit is itself a valid matched PNG/tile/palette bundle. It
    // must fail the preservation rule rather than just native-file validation.
    let png_path = format!("gfx/trainers/{stem}.png");
    let sheet = MinnesotaSheet::from_png(&art[&png_path]).expect("native battle source");
    let palette_bytes = &art[&format!("gfx/trainers/{stem}.gbcpal")];
    let mut palette = [0_u16; 4];
    for (slot, bytes) in palette.iter_mut().zip(palette_bytes.as_chunks::<2>().0) {
        *slot = u16::from_le_bytes([bytes[0], bytes[1]]);
    }
    let inks = colored(&[0, 1, 2, 3], palette);
    let mut indices = Vec::new();
    for y in 0..56 {
        for x in 0..56 {
            let pixel = sheet.pixel(x, y).expect("battle pixel");
            indices.push(
                inks.iter()
                    .position(|ink| *ink == pixel)
                    .expect("native ink") as u8,
            );
        }
    }
    let index = indices
        .iter()
        .position(|ink| *ink != 0)
        .expect("nonempty battle sprite");
    indices[index] = if indices[index] == 1 { 2 } else { 1 };
    let mut altered_battle = art.clone();
    altered_battle.insert(
        png_path,
        png_rgb(56, 56, &colored(&indices, palette)).expect("valid battle PNG"),
    );
    altered_battle.insert(
        format!("gfx/trainers/{stem}.2bpp"),
        bitplanes(&indices, 56, 56),
    );
    assert!(
        revise_original_npc_overworld(&base, &source, &altered_battle)
            .expect_err("overworld edit cannot alter valid battle art")
            .to_string()
            .contains("preserve battle art")
    );
}
