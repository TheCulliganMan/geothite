#[path = "../src/bin/pack_core/source_field_moves.rs"]
mod source_field_moves;
#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;

#[test]
fn authored_replacement_groups_keep_tilesets_blocks_and_animation_variants() {
    let source = "CutTreeBlockPointers:\n.johto:\n db $03, $02, 1\n db $5b, $3c, 0\n db -1\n.forest:\n db $0f, $17, 0\n db -1\nWhirlpoolBlockPointers:\n.johto:\n db $07, $36, 0\n db -1\n";
    let (cut, whirlpool) = source_field_moves::replacements(source).unwrap();
    assert_eq!(cut["johto"]["3"]["variant"], "grass");
    assert_eq!(cut["johto"]["91"]["replacement_block_id"], 60);
    assert_eq!(cut["forest"]["15"]["replacement_block_id"], 23);
    assert_eq!(whirlpool["johto"]["7"]["variant"], "whirlpool");
    assert_eq!(whirlpool["johto"].as_object().unwrap().len(), 1);
}

#[test]
fn badge_check_must_belong_to_the_requested_field_function() {
    let source = "CutFunction:\n.CheckAble:\n ld de, ENGINE_HIVEBADGE\nFlashFunction:\n ld de, ENGINE_ZEPHYRBADGE\n";
    assert_eq!(source_field_moves::badge(source, "CutFunction").unwrap()["index"], 1);
    let missing = "CutFunction:\n ret\nFlashFunction:\n ld de, ENGINE_ZEPHYRBADGE\n";
    assert!(source_field_moves::badge(missing, "CutFunction").is_err());
}

#[test]
fn unknown_animation_or_duplicate_source_block_is_rejected() {
    for rows in ["db $03, $02, 2", "db $03, $02, 1\ndb $03, $04, 0"] {
        let source = format!(
            "CutTreeBlockPointers:\n.johto:\n{rows}\nWhirlpoolBlockPointers:\n.johto:\ndb $07, $36, 0\n"
        );
        assert!(source_field_moves::replacements(&source).is_err());
    }
}
