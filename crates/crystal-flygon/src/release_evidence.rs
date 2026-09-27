//! Fail closed when the opening evidence belongs to another game or profile.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn sha(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn normalized_config(value: Value) -> Result<Value> {
    let config: crystal_flygon::Config = serde_json::from_value(value)?;
    let mut value = serde_json::to_value(config)?;
    if !value["operant"].is_object() {
        return Err("Release evidence must use operant mode".into());
    }
    value["operant"]["action_seed"] = serde_json::json!(0);
    Ok(value)
}
pub fn check(bundle: &Path, evidence: &[String]) -> Result<()> {
    if evidence.len() < 2 {
        return Err("Two fresh opening runs are required".into());
    }
    let index = fs::read_to_string(bundle.join("index.html"))?;
    let marker = "import('./crystal-bevy-";
    let suffix = index
        .split_once(marker)
        .ok_or("Game entry must pin a versioned module")?
        .1;
    let id = suffix
        .split_once(".js')")
        .ok_or("Missing game module terminator")?
        .0;
    if id.len() != 64 || !id.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid game module hash".into());
    }
    let wasm = format!("crystal-bevy-{id}.wasm");
    let game_sha = sha(&bundle.join(&wasm))?;
    let profile = normalized_config(read(&bundle.join("flygon-operant.json"))?)?;
    let mut seeds = BTreeSet::new();
    let mut roots = BTreeSet::new();
    for dir in evidence {
        let root = Path::new(dir).canonicalize()?;
        if !roots.insert(root.clone()) {
            return Err("Repeated evidence directory".into());
        }
        let samples = read(&root.join("runtime.json"))?;
        let samples = samples.as_array().ok_or("Malformed runtime evidence")?;
        let first = samples.first().ok_or("Empty runtime evidence")?;
        if first
            .pointer("/observation/map_info/name")
            .and_then(Value::as_str)
            != Some("PlayersHouse2F")
            || !first
                .pointer("/observation/status/party")
                .and_then(Value::as_array)
                .is_some_and(|p| p.is_empty())
        {
            return Err(format!("{} does not begin with a fresh opening", root.display()).into());
        }
        let mut previous = None;
        let mut exit = false;
        for sample in samples {
            let map = sample
                .pointer("/observation/map_info/name")
                .and_then(Value::as_str);
            if previous == Some("NewBarkTown") && map == Some("Route29") {
                exit = true;
            }
            if map.is_some() {
                previous = map;
            }
            for trace in sample
                .get("trace")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(button) = trace.pointer("/action/button").and_then(Value::as_str) {
                    for row in trace
                        .pointer("/action/readouts")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                    {
                        if row["button"] == button && row["probability"].as_f64() == Some(0.0) {
                            return Err("Evidence submitted an excluded action".into());
                        }
                    }
                }
            }
        }
        if !exit {
            return Err(format!(
                "{} has no observed New Bark → Route 29 exit",
                root.display()
            )
            .into());
        }
        // Ordinary watcher runs already produce a checksummed paired save.
        // Do not require a separately hand-written gate summary to release.
        let checkpoint = if root.join("route29-gate.json").exists() {
            let gate = read(&root.join("route29-gate.json"))?;
            Path::new(gate["verifiedCheckpoint"].as_str().ok_or("Missing verified checkpoint")?).to_path_buf()
        } else {
            let generation=fs::read_to_string(root.join("latest-checkpoint"))?;
            let generation=generation.trim();
            if !generation.starts_with("checkpoint-") || !generation.bytes().all(|b| b.is_ascii_alphanumeric() || b==b'-') {
                return Err("Invalid paired checkpoint generation".into());
            }
            root.join(generation)
        };
        let manifest = read(&checkpoint.join("manifest.json"))?;
        for (file, key) in [
            ("game.crystalsave", "gameSha256"),
            ("brain.json", "brainSha256"),
        ] {
            if manifest[key].as_str() != Some(sha(&checkpoint.join(file))?.as_str()) {
                return Err(format!("Checkpoint checksum mismatch: {file}").into());
            }
        }
        let artifacts = manifest["artifacts"]
            .as_array()
            .ok_or("Missing artifact pins")?;
        if !artifacts.iter().any(|a| {
            a["sha256"].as_str() == Some(&game_sha)
                && a["url"].as_str().is_some_and(|u| u.ends_with(&wasm))
        }) {
            return Err("Evidence used a different game binary".into());
        }
        if !artifacts.iter().any(|a| {
            a["url"]
                .as_str()
                .is_some_and(|u| u.ends_with(".crystalpack"))
        }) {
            return Err("Evidence has no game-content pin".into());
        }
        for artifact in artifacts.iter().filter(|a| {
            a["url"]
                .as_str()
                .is_some_and(|u| u.ends_with(".crystalpack"))
        }) {
            let name = artifact["url"]
                .as_str()
                .unwrap()
                .rsplit('/')
                .next()
                .unwrap();
            if artifact["sha256"].as_str() != Some(sha(&bundle.join(name))?.as_str()) {
                return Err("Evidence used different game content".into());
            }
        }
        let neural = manifest["neuralArtifacts"].as_array().ok_or("Evidence has no neural binary pin")?;
        if neural.len() != 1 { return Err("Evidence needs one neural binary".into()); }
        let name = neural[0]["url"].as_str().ok_or("Missing neural URL")?.rsplit('/').next().unwrap();
        if !name.starts_with("crystal_flygon-") || !name.ends_with(".wasm") || neural[0]["sha256"].as_str() != Some(sha(&bundle.join("flygon").join(name))?.as_str()) {
            return Err("Evidence used a different neural binary".into());
        }
        let brain = read(&checkpoint.join("brain.json"))?;
        if brain["interface_id"] != read(&bundle.join("flygon-manifest.json"))?["interface_id"] {
            return Err("Evidence used a different neural interface".into());
        }
        let operant = &brain["config"]["operant"];
        if normalized_config(brain["config"].clone())? != profile {
            return Err("Evidence configuration differs from the release profile".into());
        }
        seeds.insert(operant["action_seed"].as_u64().unwrap_or(0));
    }
    if seeds.len() < 2 {
        return Err("Opening evidence needs two distinct seeds".into());
    }
    println!(
        "Verified {} fresh opening runs against {game_sha}",
        roots.len()
    );
    Ok(())
}
