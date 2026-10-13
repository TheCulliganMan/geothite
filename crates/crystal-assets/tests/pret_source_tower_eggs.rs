#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;
#[path = "../src/bin/pack_core/source_tower_eggs.rs"]
mod source_tower_eggs;

#[test]
fn structured_mon_layout_and_odd_egg_thresholds_are_preserved() {
    let mon = "db PICHU\ndb NO_ITEM\ndb THUNDERSHOCK, CHARM, DIZZY_PUNCH, 0\ndw 02048\nbigdt 125\nbigdw 0\nbigdw 0\nbigdw 0\nbigdw 0\nbigdw 0\ndn 0,0,0,0\ndb 30,20,10,0\ndb 20\ndb 0,0,0\ndb 5\ndb 0,0\nbigdw 0\nbigdw 17\nbigdw 9\nbigdw 6\nbigdw 11\nbigdw 8\nbigdw 8\ndname \"EGG\", MON_NAME_LENGTH\n";
    let records = source_tower_eggs::records(mon).unwrap();
    assert_eq!(records[0]["originalTrainerId"], 2048);
    assert_eq!(
        records[0]["stats"],
        serde_json::json!([0, 17, 9, 6, 11, 8, 8])
    );
    let source = format!(
        "MACRO odd_egg_prob\ndw odd_egg_prob_total * $ffff / 100\nENDM\nodd_egg_prob 25\nodd_egg_prob 75\nOddEggs:\n{mon}{mon}"
    );
    let eggs = source_tower_eggs::odd_eggs(&source, "ODD").unwrap();
    assert_eq!(eggs[0]["probability"], 25);
    assert_eq!(eggs[1]["probability"], 75);
    assert_eq!(eggs[0]["moves"].as_array().unwrap().len(), 3);
    assert_eq!(eggs[0]["hatchCycles"], 20);
    assert!(
        source_tower_eggs::odd_eggs(&source.replace("odd_egg_prob 75", "odd_egg_prob 74"), "ODD")
            .is_err()
    );
    assert!(source_tower_eggs::records(&mon.replace("bigdt 125", "dw 125")).is_err());
}
