#[path = "../src/bin/pack_core/source_phone.rs"]
mod source_phone;
#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;

#[test]
fn phone_indices_preserve_unused_slots_and_stop_at_catalog_boundary() {
    let source = "const_def\nconst PHONE_00\nconst PHONE_MOM\nconst_skip 3\nconst PHONE_JACK\nDEF NUM_PHONE_CONTACTS EQU const_value - 1\nconst_def\nconst PHONE_OTHER";
    let slots = source_phone::slots(source, "PHONE_", "DEF NUM_PHONE_CONTACTS ").unwrap();
    assert_eq!(slots[&0], "PHONE_00");
    assert_eq!(slots[&1], "PHONE_MOM");
    assert_eq!(slots[&5], "PHONE_JACK");
    assert!(!slots.values().any(|id| id == "PHONE_OTHER"));
    assert!(
        source_phone::slots(
            "const_def\nconst PHONE_MOM\nconst_def\nconst PHONE_ELM",
            "PHONE_",
            "DEF NUM_PHONE_CONTACTS "
        )
        .is_err()
    );
}
