use base64::{engine::general_purpose::STANDARD, Engine};
use crystal_assets::read_verified_compiled_game_pack;

#[test]
#[ignore = "requires completed external base and outdoor variant packs"]
fn outdoor_variant_changes_only_four_blocks_and_preserves_every_audio_record() {
    let base_path = std::env::var("CRYSTAL_RENDER_TEST_PACK")
        .expect("set CRYSTAL_RENDER_TEST_PACK to the completed canonical base pack");
    let variant_path = std::env::var("CRYSTAL_OUTDOOR_VARIANT_PACK")
        .expect("set CRYSTAL_OUTDOOR_VARIANT_PACK to the completed separate outdoor pack");
    let base = read_verified_compiled_game_pack(base_path).unwrap();
    let variant = read_verified_compiled_game_pack(variant_path).unwrap();
    let mut expected = base.data().clone();
    let mut changed = 0;
    for (name, edits) in [
        ("CherrygroveCity", &[(17, 0), (18, 0)][..]),
        ("Route30", &[(9, 25), (9, 26)][..]),
    ] {
        let map = expected.maps.get_mut(name).unwrap();
        let width = usize::from(map.attributes.width);
        let label = map.attributes.blocks_label.as_ref().unwrap();
        let mut bytes = STANDARD.decode(&expected.map_blocks[label]).unwrap();
        for &(x, y) in edits {
            let i = y * width + x;
            assert_eq!(u16::from(bytes[i]), map.blocks[i]);
            assert_ne!(
                map.blocks[i], 0x05,
                "canonical source {name} ({x},{y}) was already overridden"
            );
            bytes[i] = 0x05;
            map.blocks[i] = 0x05;
            changed += 1;
        }
        expected
            .map_blocks
            .insert(label.clone(), STANDARD.encode(bytes));
    }
    assert_eq!(changed, 4);
    assert_eq!(
        serde_json::to_value(variant.data()).unwrap(),
        serde_json::to_value(expected).unwrap(),
        "all events, collision definitions, other maps and catalogs must remain identical"
    );
    assert_eq!(variant.runtime_files(), base.runtime_files());
    assert_eq!(variant.compiled_audio(), base.compiled_audio());
    assert_eq!(
        serde_json::to_value(variant.audio_manifest().unwrap()).unwrap(),
        serde_json::to_value(base.audio_manifest().unwrap()).unwrap(),
        "PCM hashes, frame counts, loop ranges and program identity must be retained"
    );
    assert_eq!(
        serde_json::to_value(variant.report()).unwrap(),
        serde_json::to_value(base.report()).unwrap()
    );
    assert_eq!(variant.format_version(), base.format_version());
    assert_ne!(
        variant.identity().unwrap().content_hash,
        base.identity().unwrap().content_hash
    );
}
