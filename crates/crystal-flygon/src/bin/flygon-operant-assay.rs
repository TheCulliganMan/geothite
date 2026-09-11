//! Full-graph causal controls for the artificial action-memory BCI.
use crystal_flygon::Brain;
use serde_json::{Value, json};
use std::{fs, io};
fn parse(s: Result<String, String>) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(&s.map_err(io::Error::other)?)?)
}
fn readout<'a>(v: &'a Value, button: &str) -> &'a Value {
    v["action"]["readouts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["button"] == button)
        .unwrap()
}
fn uniform(v: &Value) -> bool {
    v["action"]["readouts"].as_array().unwrap().iter().all(|r| {
        (r["probability"].as_f64().unwrap()
            - 1.0 / v["action"]["readouts"].as_array().unwrap().len() as f64)
            .abs()
            < 1e-6
    })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err("Usage: flygon-operant-assay DATA_DIR OPERANT_CONFIG_JSON".into());
    }
    let config = fs::read_to_string(&args[2])?;
    let mut brain = Brain::new(
        &fs::read(format!("{}/graph.bin", args[1]))?,
        &fs::read_to_string(format!("{}/metadata.json", args[1]))?,
        &config,
    )
    .map_err(io::Error::other)?;
    let identity = parse(brain.summary())?;
    let fixture = json!({"status":{"screen":"overworld"},"map_info":{"name":"PlayersHouse2F","player":{"x":1,"y":3,"facing":"Right"}},"observe":{"menus":[],"visible_dialogue":null}});
    let baseline = parse(brain.operant_decide(&fixture.to_string()))?;
    let context = baseline["context"].as_str().unwrap().to_string();
    let checkpoint = brain.checkpoint().map_err(io::Error::other)?;
    let mut positive = json!(null);
    for _ in 0..6 {
        positive = parse(brain.action_memory_pair(&context, "up", 8, 6.9, true))?;
    }
    let paired = parse(brain.operant_decide(&fixture.to_string()))?;
    let mut negative = json!(null);
    for _ in 0..6 {
        negative = parse(brain.action_memory_punish(&context, "b", 8, 6.9))?;
    }
    let punished = parse(brain.operant_decide(&fixture.to_string()))?;
    brain.erase_memory();
    let erased = parse(brain.operant_decide(&fixture.to_string()))?;
    brain.restore(&checkpoint).map_err(io::Error::other)?;
    let mut frozen_config: Value = serde_json::from_str(&config)?;
    frozen_config["learning"] = json!(false);
    brain
        .configure(&frozen_config.to_string())
        .map_err(io::Error::other)?;
    for _ in 0..6 {
        brain
            .action_memory_pair(&context, "up", 8, 6.9, true)
            .map_err(io::Error::other)?;
        brain
            .action_memory_punish(&context, "b", 8, 6.9)
            .map_err(io::Error::other)?;
    }
    let frozen = parse(brain.operant_decide(&fixture.to_string()))?;
    brain.restore(&checkpoint).map_err(io::Error::other)?;
    brain.configure(&config).map_err(io::Error::other)?;
    let _ = parse(brain.operant_decide(&fixture.to_string()))?;
    let mut story_fixture = fixture.clone();
    story_fixture["reward_state"] = json!({"version":1,"event_flags":["EVENT_BEAT_RED"]});
    let story_feedback = parse(brain.operant_feedback(&story_fixture.to_string()))?;
    let story_pam_spikes = story_feedback["training"]["telemetry"]["circuit_activity"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| p["population"] == "PAM01")
        .filter_map(|p| p["spikes"].as_u64())
        .sum::<u64>();
    let _ = parse(brain.operant_decide(&story_fixture.to_string()))?;
    let mut new_place = story_fixture.clone();
    new_place["map_info"]["name"] = json!("Route29");
    let place_feedback = parse(brain.operant_feedback(&new_place.to_string()))?;
    let _ = parse(brain.operant_decide(&story_fixture.to_string()))?;
    let repeated_place = parse(brain.operant_feedback(&new_place.to_string()))?;
    let checks = json!({"new_place_double_conditioning":place_feedback["event"]["conditioning_pairings"]==12,"repeat_place_no_conditioning":repeated_place["event"]["conditioning_pairings"]==0,"story_event_recruits_pam":story_pam_spikes>0,"story_event_changes_existing_edges":story_feedback["training"]["telemetry"]["changed_edges"].as_u64().unwrap_or(0)>0,"naive_uniform":uniform(&baseline),"rewarded_action_preferred":readout(&paired,"up")["probability"].as_f64().unwrap()>0.8,"aversive_action_suppressed":readout(&punished,"b")["probability"].as_f64().unwrap()<0.03,"erasure_restores_uniform":uniform(&erased),"frozen_learning_stays_uniform":uniform(&frozen),"frozen_no_weight_changes":frozen["telemetry"]["changed_edges"]==0});
    let report = json!({"scope":"Artificial action-cue mechanism assay using a synthetic context; not gameplay progress","graph_id":identity["graph_id"],"model_id":identity["model_id"],"interface_id":identity["interface_id"],"checks":checks,"story_feedback":story_feedback,"place_feedback":place_feedback,"baseline":baseline["action"]["readouts"],"paired":paired["action"]["readouts"],"punished":punished["action"]["readouts"],"erased":erased["action"]["readouts"],"frozen":frozen["action"]["readouts"],"positive_circuit_activity":positive["telemetry"]["circuit_activity"],"negative_circuit_activity":negative["telemetry"]["circuit_activity"]});
    println!("{}", serde_json::to_string_pretty(&report)?);
    if checks.as_object().unwrap().values().any(|v| v != true) {
        return Err("Action-memory causal check failed".into());
    }
    Ok(())
}
