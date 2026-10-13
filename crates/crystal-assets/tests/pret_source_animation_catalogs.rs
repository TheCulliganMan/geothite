#[path = "../src/bin/pack_core/source_animation_catalogs.rs"]
mod source_animation_catalogs;
#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;

#[test]
fn sprite_frame_flip_indices_encode_macro_bits() {
    assert_eq!(
        source_animation_catalogs::frame_attr_flags(&[5]).unwrap(),
        64
    );
    assert_eq!(
        source_animation_catalogs::frame_attr_flags(&[5, 6]).unwrap(),
        192
    );
    assert!(source_animation_catalogs::frame_attr_flags(&[7]).is_err());
}

#[test]
fn battle_aliases_local_targets_and_move_table_remain_authored() {
    let source = "BattleAnimations::\ndw BattleAnim_Dummy\ndw BattleAnim_Hit\nassert_table_length NUM_ATTACKS + 1\ndw BattleAnim_Extra\nBattleAnim_Dummy:\nBattleAnim_Alias:\nanim_ret\nBattleAnim_Hit:\nanim_jump .End\n.End\nanim_call BattleAnim_Alias\nanim_ret\nBattleAnim_Extra:\nanim_ret";
    let (animations, table) = source_animation_catalogs::battle(source, 1).unwrap();
    assert_eq!(
        animations["BattleAnim_Dummy"],
        animations["BattleAnim_Alias"]
    );
    assert_eq!(
        table,
        serde_json::json!(["BattleAnim_Dummy", "BattleAnim_Hit"])
    );
    assert!(animations.get("BattleAnim_Extra").is_some());
    assert!(
        source_animation_catalogs::battle(&source.replace("\n.End\n", "\n.Wrong\n"), 1).is_err()
    );
    assert!(source_animation_catalogs::battle(source, 2).is_err());
}
