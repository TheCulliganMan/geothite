use crystal_assets::validate_runtime_intro_timing_oracle;
use serde_json::json;

#[test]
fn raw_rust_measurements_cannot_be_published_as_verified_intro_timing() {
    let mut oracle = json!({"runner":"pack_core Rust intro oracle","rom_sha1":"f4cd194bdee0d04ca4eac29e09b8e4e9d818c133","schema_version":1,"validated":false,"instruction_core":"sm83_mcycle_v1","timer_model":"cgb_timer_falling_edge_delayed_reload_v1"});
    assert!(validate_runtime_intro_timing_oracle(&oracle).is_err());
    oracle["validated"] = json!(true);
    assert!(validate_runtime_intro_timing_oracle(&oracle).is_err());
    oracle["certification"] = json!({"scene_count":28,"decompression_count":37,"sprite_allocation_count":19,"complete_phase_machine_matches_rom":true,"audio_irq_model_matches_rom":true});
    validate_runtime_intro_timing_oracle(&oracle).unwrap();
    oracle["certification"]["audio_irq_model_matches_rom"] = json!(false);
    assert!(validate_runtime_intro_timing_oracle(&oracle).is_err());
    oracle["certification"]["audio_irq_model_matches_rom"] = json!(true);
    oracle["rom_sha1"] = json!("0000000000000000000000000000000000000000");
    assert!(validate_runtime_intro_timing_oracle(&oracle).is_err());
}

#[test]
fn historical_timing_identity_remains_bound_to_its_original_rom() {
    let mut oracle = json!({"runner":"tools/asm-oracle/intro_trace.py --timing","rom_sha1":"f4cd194bdee0d04ca4eac29e09b8e4e9d818c133"});
    validate_runtime_intro_timing_oracle(&oracle).unwrap();
    oracle["runner"] = json!("unknown runner");
    assert!(validate_runtime_intro_timing_oracle(&oracle).is_err());
}
