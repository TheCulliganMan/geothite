#[path="../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;
mod radio_text_export { pub(super) use super::source_syntax::{code_part,source_arguments}; }
#[path="../src/bin/pack_core/source_scripts.rs"]
mod source_scripts;
#[path="../src/bin/pack_core/source_expressions.rs"]
mod source_expressions;
use std::collections::BTreeMap;

#[test]
fn source_palette_expressions_and_precedence_are_exact() {
    let constants=BTreeMap::from([("PAL_NPC_RED".to_owned(),1),("PAL_NPC_BLUE".to_owned(),2)]);
    assert_eq!(source_expressions::evaluate("(PAL_NPC_RED << 4)",&constants).unwrap(),16);
    assert_eq!(source_expressions::evaluate("(PAL_NPC_BLUE << 4)",&constants).unwrap(),32);
    assert_eq!(source_expressions::evaluate("2 + 3 * 4",&constants).unwrap(),14);
    assert_eq!(source_expressions::evaluate("($80 | %11) >> 1",&constants).unwrap(),65);
    assert!(source_expressions::evaluate("UNKNOWN << 4",&constants).is_err());
    assert!(source_expressions::evaluate("2 / 0",&constants).is_err());
}

#[test]
fn source_local_prices_are_expanded_without_changing_dialogue() {
    let mut scripts=BTreeMap::from([("Shop".into(),serde_json::json!([
        {"command":"setval","args":["(PAL_NPC_RED << 4)"]},
        {"command":"db","args":["\"FRESH WATER ¥{d:PRICE}@\""]}
    ]))]);
    source_expressions::normalize("DEF PRICE    EQU 200",&mut scripts,&BTreeMap::from([("PAL_NPC_RED".into(),1)])).unwrap();
    assert_eq!(scripts["Shop"][0]["args"][0],"16");
    assert_eq!(scripts["Shop"][1]["args"][0],"\"FRESH WATER ¥200@\"");
}

#[test]
fn underground_source_macros_expand_each_authored_door_and_warp_order() {
    let source="MACRO ugdoor_def\nENDM\nugdoor_def 16, 6, $3e, $2d\nugdoor_def 12, 6, $3f, $2a, 12, 8, $3d, $2d\nCallback:\nfor n, 1, ugdoor_n + 1\ncheckevent EVENT_DOOR_{d:n}_OPEN\niffalse .door_{d:n}_closed\nchangeugdoor n, OPEN\n.door_{d:n}_closed\nendr\nendcallback\nExit:\nwarpfacing UP, HALL_OF_FAME, 4, 13\n";
    let parsed=source_scripts::parse(source).unwrap();
    let serialized=serde_json::to_string(&parsed).unwrap();
    assert!(!serialized.contains("{d:"));
    assert!(!serialized.contains("changeugdoor"));
    assert_eq!(parsed["Callback"].as_array().unwrap().iter().filter(|cmd|cmd["command"]=="changeblock").count(),3);
    assert_eq!(parsed["Exit"][0]["args"],serde_json::json!(["HALL_OF_FAME","4","13","UP"]));
    assert!(parsed.contains_key(".door_1_closed@Callback"));
    assert!(parsed.contains_key(".door_2_closed@Callback"));
}
