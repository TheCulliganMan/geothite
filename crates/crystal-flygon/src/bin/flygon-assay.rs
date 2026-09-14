//! Reproducible full-graph causal memory experiment. No game actions or builds.
use crystal_flygon::{Brain, Config};
use serde_json::{Value, json};
use std::{fs, time::Instant};
fn probe(brain: &mut Brain, cue: &str) -> Result<Value, String> {
    let result: Value =
        serde_json::from_str(&brain.conditioning_probe(cue)?).map_err(|e| e.to_string())?;
    Ok(
        json!({"outputs":result["outputs"],"sensory":result["sensory"],
        "changed_edges":result["telemetry"]["changed_edges"],
        "reward_during_probe":false,"learning_during_probe":false}),
    )
}
fn evaluate(brain: &mut Brain) -> Result<Value, String> {
    Ok(json!({"A":probe(brain,"A")?,"B":probe(brain,"B")?}))
}
fn spikes(result: &Value, cue: &str) -> Vec<u64> {
    result[cue]["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["spikes"].as_u64().unwrap())
        .collect()
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("Usage: flygon-assay DATA_DIR CONDITIONING_CONFIG_JSON > report.json".into());
    }
    let read = |name: &str| fs::read(format!("{}/{}", args[1], name)).map_err(|e| e.to_string());
    let graph = read("graph.bin")?;
    let metadata = String::from_utf8(read("metadata.json")?).map_err(|e| e.to_string())?;
    let config_json = fs::read_to_string(&args[2]).map_err(|e| e.to_string())?;
    let mut config: Config = serde_json::from_str(&config_json).map_err(|e| e.to_string())?;
    let start = Instant::now();
    let mut brain = Brain::new(&graph, &metadata, &config_json)?;
    drop(graph);
    drop(metadata);
    let identity: Value = serde_json::from_str(&brain.telemetry()?).map_err(|e| e.to_string())?;
    let baseline = evaluate(&mut brain)?;
    let mut conditions = Vec::new();
    let mut passed = true;
    for trained in ["A", "B"] {
        let other = if trained == "A" { "B" } else { "A" };
        for condition in ["paired", "unpaired", "frozen"] {
            brain.erase_memory();
            brain.reset_dynamics();
            config.learning = condition != "frozen";
            brain.configure(&serde_json::to_string(&config).map_err(|e| e.to_string())?)?;
            for _ in 0..6 {
                brain.reset_dynamics();
                brain.conditioning_cue(trained)?;
                brain.step(100.0)?;
                if condition == "unpaired" {
                    brain.conditioning_cue("off")?;
                    for _ in 0..5 {
                        brain.step(1000.0)?;
                    }
                }
                brain.appetitive_pulse()?;
                brain.step(200.0)?;
            }
            let evaluation = evaluate(&mut brain)?;
            let checkpoint = brain.checkpoint()?;
            let first = brain.step(50.0)?;
            brain.restore(&checkpoint)?;
            let exact = first == brain.step(50.0)?;
            let specific = spikes(&evaluation, other) == spikes(&baseline, other);
            let altered = spikes(&evaluation, trained) != spikes(&baseline, trained);
            let frozen_clean = condition != "frozen" || evaluation["A"]["changed_edges"] == 0;
            let causal = if condition == "paired" {
                specific && altered
            } else {
                specific && !altered
            };
            brain.restore(&checkpoint)?;
            brain.erase_memory();
            let erased = evaluate(&mut brain)?;
            let erase_restores = ["A", "B"]
                .iter()
                .all(|cue| spikes(&erased, cue) == spikes(&baseline, cue));
            let ok = causal && exact && erase_restores && frozen_clean;
            passed &= ok;
            eprintln!(
                "{trained} {condition}: causal={causal} checkpoint={exact} erase={erase_restores}"
            );
            conditions.push(
                json!({"trained_cue":trained,"condition":condition,"trials":6,
                "evaluation":evaluation,"erased":erased,"checkpoint_exact":exact,"passed":ok}),
            );
        }
    }
    println!("{}",serde_json::to_string_pretty(&json!({
        "scope":"Artificial KC currents and MBON tonic current in full retained graph; not natural sensory learning or Pokemon skill",
        "model_id":identity["model_id"],"interface_id":identity["interface_id"],"graph_id":identity["graph_id"],"neurons":identity["neurons"],"edges":identity["edges"],
        "config":serde_json::from_str::<Value>(&config_json).map_err(|e|e.to_string())?,
        "baseline":baseline,"conditions":conditions,"passed":passed,"wall_seconds":start.elapsed().as_secs_f64()
    })).map_err(|e|e.to_string())?);
    if !passed {
        return Err("Causal memory criteria failed; inspect report".into());
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
