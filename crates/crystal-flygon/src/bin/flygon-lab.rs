//! Persistent full-graph experiment process. Tune via JSON lines without builds.
use crystal_flygon::Brain;
use std::{
    fs,
    io::{self, BufRead, Write},
    time::Instant,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("Usage: flygon-lab DATA_DIR CONFIG_JSON".into());
    }
    let start = Instant::now();
    let graph = fs::read(format!("{}/graph.bin", args[1]))?;
    let metadata = fs::read_to_string(format!("{}/metadata.json", args[1]))?;
    let mut brain =
        Brain::new(&graph, &metadata, &fs::read_to_string(&args[2])?).map_err(io::Error::other)?;
    drop(graph);
    drop(metadata);
    eprintln!("Full brain loaded in {:?}", start.elapsed());
    println!("{}", brain.telemetry().map_err(io::Error::other)?);
    io::stdout().flush()?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let start = Instant::now();
        let result = (|| -> Result<serde_json::Value, String> {
            let v: serde_json::Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
            let json = match v["op"].as_str().unwrap_or("") {
                "cue_probe" => brain.action_memory_probe(
                    v["context"].as_str().ok_or("Missing context")?,
                    v["button"].as_str().ok_or("Missing button")?,
                    v["size"].as_u64().unwrap_or(16) as usize,
                    v["tonic"].as_f64().unwrap_or(6.9) as f32,
                )?,
                "cue_pair" => brain.action_memory_pair(
                    v["context"].as_str().ok_or("Missing context")?,
                    v["button"].as_str().ok_or("Missing button")?,
                    v["size"].as_u64().unwrap_or(16) as usize,
                    v["tonic"].as_f64().unwrap_or(6.9) as f32,
                    v["reward"].as_bool().unwrap_or(true),
                )?,
                "cue_punish" => brain.action_memory_punish(
                    v["context"].as_str().ok_or("Missing context")?,
                    v["button"].as_str().ok_or("Missing button")?,
                    v["size"].as_u64().unwrap_or(8) as usize,
                    v["tonic"].as_f64().unwrap_or(6.9) as f32,
                )?,
                "operant_decide" => brain.operant_decide(&v["observation"].to_string())?,
                "operant_feedback" => brain.operant_feedback(&v["observation"].to_string())?,
                "record" => brain.run_record()?,
                "step" => brain.step(v["ms"].as_f64().unwrap_or(50.0) as f32)?,
                "currents" => {
                    brain.inject_currents(&v["pairs"].to_string())?;
                    brain.telemetry()?
                }
                "stimulate" => {
                    let indices: Vec<u32> =
                        serde_json::from_value(v["indices"].clone()).map_err(|e| e.to_string())?;
                    brain.stimulate(&indices, v["strength"].as_f64().unwrap_or(8.0) as f32)?;
                    brain.telemetry()?
                }
                "aversive" => {
                    brain.aversive_pulse()?;
                    brain.telemetry()?
                }
                "reward" => {
                    brain.appetitive_pulse()?;
                    brain.telemetry()?
                }
                "configure" => {
                    brain.configure(&v["config"].to_string())?;
                    brain.telemetry()?
                }
                "observe" => brain.observe(&v["observation"].to_string())?,
                "inspect" => brain.inspect(v["index"].as_u64().ok_or("Missing index")? as u32)?,
                "action" => brain.action()?,
                "memory" => brain.memory()?,
                "reset" => {
                    brain.reset_dynamics();
                    brain.telemetry()?
                }
                "erase" => {
                    brain.erase_memory();
                    brain.telemetry()?
                }
                "save" => {
                    fs::write(
                        v["path"].as_str().ok_or("Missing path")?,
                        brain.checkpoint()?,
                    )
                    .map_err(|e| e.to_string())?;
                    brain.telemetry()?
                }
                "restore" => {
                    let json = fs::read_to_string(v["path"].as_str().ok_or("Missing path")?)
                        .map_err(|e| e.to_string())?;
                    brain.restore(&json)?;
                    brain.telemetry()?
                }
                "outcome" => brain.outcome(
                    &v["observation"].to_string(),
                    v["button"].as_str().unwrap_or(""),
                )?,
                _ => return Err("Unknown operation".into()),
            };
            serde_json::from_str(&json).map_err(|e| e.to_string())
        })();
        println!(
            "{}",
            serde_json::json!({"result":result.as_ref().ok(),"error":result.as_ref().err(),"wall_ms":start.elapsed().as_secs_f64()*1000.0})
        );
        io::stdout().flush()?;
    }
    Ok(())
}
