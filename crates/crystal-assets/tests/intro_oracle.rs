#[path = "../src/bin/pack_core/source_intro_oracle.rs"]
mod source_intro_oracle;

#[test]
fn authored_lcd_callback_paths_measure_different_instruction_costs() {
    let Some(root) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let zero =
        source_intro_oracle::measure_routine(root, "LCD", &[("hLCDCPointer".into(), 0)]).unwrap();
    let scroll =
        source_intro_oracle::measure_routine(root, "LCD", &[("hLCDCPointer".into(), 0x43)])
            .unwrap();
    assert_eq!(zero["machine_cycles"], 18);
    assert_eq!(scroll["machine_cycles"], 40);
    assert_eq!(
        zero["instructions"].as_array().unwrap().last().unwrap()["opcode"],
        0xd9
    );
}

#[test]
fn pinned_rom_trace_completes_every_authored_intro_scene() {
    let Some(root) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
        return;
    };
    let trace = source_intro_oracle::trace(std::path::Path::new(&root)).unwrap();
    assert_eq!(trace["validated"], false);
    assert_eq!(trace["scenes"].as_array().unwrap().len(), 28);
    let records = trace["records"].as_array().unwrap();
    assert_eq!((trace["first_input"]["cpu_t_cycles"].as_u64().unwrap()-trace["entry"]["cpu_t_cycles"].as_u64().unwrap()-trace["first_input"]["interrupt_t_cycles_since_entry"].as_u64().unwrap())/4,59);
    assert!(records.iter().all(|record|record["instruction_machine_cycles"].as_u64().is_some_and(|cycles|cycles>0)));
    assert_eq!(
        records
            .iter()
            .filter(|record| record["hook"] == "Decompress")
            .count(),
        37
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record["hook"] == "InitSpriteAnimStruct")
            .count(),
        19
    );
    assert!(
        records
            .iter()
            .filter(|record| record["hook"] == "DoNextFrameForAllSprites")
            .count()
            > 1000
    );
    assert!(records.iter().all(|record| {
        record["elapsed_t_cycles"]
            .as_u64()
            .is_some_and(|cycles| cycles > 0)
    }));
}
