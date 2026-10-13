#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;
mod radio_text_export {
    pub(super) use super::source_syntax::{code_part, source_arguments};
}
#[path = "../src/bin/pack_core/source_scripts.rs"]
mod source_scripts;

fn vendor() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("CRYSTAL_EXPORT_WORKSPACE")
        .expect("source fallthrough regression requires the pinned external workspace"))
        .join("vendor/pokecrystal")
}

#[test]
fn sudowoodo_source_fallthrough_retains_the_authored_battle_identity() {
    let vendor = vendor();
    let source = std::fs::read_to_string(vendor.join("maps/Route36.asm")).unwrap();
    let mut scripts = source_scripts::parse(&source).unwrap();
    let original_battle = scripts["WateredWeirdTreeScript"].clone();
    assert_eq!(original_battle[11]["command"], "loadwildmon");
    assert_eq!(original_battle[11]["args"], serde_json::json!(["SUDOWOODO", "20"]));
    assert_eq!(original_battle[12]["command"], "startbattle");
    source_scripts::materialize_event_fallthroughs(&vendor, &source, &mut scripts).unwrap();
    assert_eq!(scripts["WateredWeirdTreeScript"], original_battle);
    assert_eq!(scripts[".Fight@SudowoodoScript"].as_array().unwrap().last().unwrap(),
        &serde_json::json!({"command":"sjump","args":["WateredWeirdTreeScript"]}));
    assert_eq!(scripts["SudowoodoScript"].as_array().unwrap().last().unwrap(),
        &serde_json::json!({"command":"sjump","args":["WateredWeirdTreeScript"]}));
}

#[test]
fn fallthrough_does_not_cross_terminators_or_data_tables() {
    let source = "Stopped:\nend\nJumped:\nsjump Stopped\nTable:\ndb 1, 2\nBeforeText:\nclosetext\nText:\ntext \"HELLO\"\ndone\n";
    let mut scripts = source_scripts::parse(source).unwrap();
    let original = scripts.clone();
    source_scripts::materialize_event_fallthroughs(&vendor(), source, &mut scripts).unwrap();
    assert_eq!(scripts, original);
}
