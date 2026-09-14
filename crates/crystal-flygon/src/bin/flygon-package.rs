//! Assemble a self-contained optional browser distribution without rebuilding the
//! game. Refuses existing output and verifies pinned neural assets before copying.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};
#[path = "../preview.rs"]
mod preview;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const UI: &[&str] = &[
    "asset-progress.js",
    "loading-progress.css",
    "flygon.html",
    "flygon.css",
    "flygon.js",
    "flygon-worker.js",
    "flygon-view.js",
    "flygon-view-worker.js",
];
const PROFILES: &[&str] = &[
    "operant",
    "config",
    "config-fast",
    "balanced",
    "refractory",
    "conditioning",
    "conditioning-fast",
];
#[derive(Serialize, Deserialize)]
struct FileEntry {
    bytes: u64,
    sha256: String,
}
#[derive(Serialize, Deserialize)]
struct Release {
    schema_version: u32,
    modpack: Value,
    files: BTreeMap<String, FileEntry>,
}
fn stream_hash(path: &Path, hash: &mut Sha256) -> Result<u64> {
    let mut file = fs::File::open(path)?;
    let mut buf = [0; 65536];
    let mut len = 0;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
        len += n as u64;
    }
    Ok(len)
}
fn entry(path: &Path) -> Result<FileEntry> {
    let mut h = Sha256::new();
    let bytes = stream_hash(path, &mut h)?;
    Ok(FileEntry {
        bytes,
        sha256: format!("{:x}", h.finalize()),
    })
}
fn safe_file(path: &Path) -> Result<()> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(format!("Expected a regular file: {}", path.display()).into());
    }
    Ok(())
}
fn inventory(root: &Path, relative: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(root.join(relative))? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let name = relative.join(entry.file_name());
        if kind.is_symlink() {
            return Err(format!("Release contains a symlink: {}", name.display()).into());
        }
        if kind.is_dir() {
            inventory(root, &name, files)?;
        } else if kind.is_file() {
            files.push(name.to_str().ok_or("Invalid release filename")?.to_owned());
        } else {
            return Err("Release contains a non-file asset".into());
        }
    }
    Ok(())
}
fn verify(root: &Path) -> Result<()> {
    let release: Release = serde_json::from_slice(&fs::read(root.join("flygon-release.json"))?)?;
    if release.schema_version != 1 {
        return Err("Unsupported release manifest".into());
    }
    let mut actual_files = Vec::new();
    inventory(root, Path::new(""), &mut actual_files)?;
    actual_files.sort();
    let mut expected_files: Vec<_> = release
        .files
        .keys()
        .cloned()
        .chain(["flygon-release.json".into()])
        .collect();
    expected_files.sort();
    if actual_files != expected_files {
        return Err("Release inventory differs from manifest (missing or extra files)".into());
    }
    for name in UI.iter().copied().chain([
        "index.html",
        "flygon/crystal_flygon.js",
        "flygon/crystal_flygon_bg.wasm",
        "flygon-data/graph.bin",
        "flygon-data/metadata.json",
        "flygon-dataset.json",
        "flygon-operant.json",
    ]) {
        if !release.files.contains_key(name) {
            return Err(format!("Release omits required asset: {name}").into());
        }
    }
    for (name, expected) in &release.files {
        if Path::new(name)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("Unsafe manifest path".into());
        }
        let file = root.join(name);
        safe_file(&file)?;
        let actual = entry(&file)?;
        if actual.bytes != expected.bytes || actual.sha256 != expected.sha256 {
            return Err(format!("Release hash mismatch: {name}").into());
        }
    }
    println!("Verified {} release files", release.files.len());
    Ok(())
}
fn copy_file(
    source: &Path,
    out: &Path,
    name: &str,
    files: &mut BTreeMap<String, FileEntry>,
) -> Result<()> {
    safe_file(source)?;
    let target = out.join(name);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, &target)?;
    files.insert(name.into(), entry(&target)?);
    Ok(())
}
fn assemble(repo: &Path, game: &Path, data: &Path, out: &Path) -> Result<()> {
    if out.exists() {
        return Err("Output already exists; choose a new release directory".into());
    }
    let manifest: Value =
        serde_json::from_slice(&fs::read(repo.join("modpacks/flygon/manifest.json"))?)?;
    if manifest["schema_version"] != 1
        || manifest["id"] != "flygon"
        || manifest["model_id"] != crystal_flygon::MODEL_ID
        || manifest["interface_id"] != crystal_flygon::INTERFACE_ID
    {
        return Err("Modpack/runtime identity mismatch".into());
    }
    let dataset: Value =
        serde_json::from_slice(&fs::read(repo.join("modpacks/flygon/dataset.json"))?)?;
    let mut graph_hash = Sha256::new();
    for name in ["graph.bin", "metadata.json"] {
        safe_file(&data.join(name))?;
        stream_hash(&data.join(name), &mut graph_hash)?;
    }
    if format!("{:x}", graph_hash.finalize())
        != dataset["graph_id"]
            .as_str()
            .ok_or("Missing graph identity")?
    {
        return Err("Prepared dataset differs from the pinned graph".into());
    }
    for profile in PROFILES {
        let config: crystal_flygon::Config = serde_json::from_slice(&fs::read(
            repo.join(format!("modpacks/flygon/{profile}.json")),
        )?)?;
        config.validate().map_err(io::Error::other)?;
    }
    let mut index = fs::read_to_string(game.join("index.html"))?;
    let anchor = "const bridge = createGameBridge(wasm);";
    if !index.contains("__flygonGameBridge") {
        if !index.contains(anchor) {
            return Err("Game bundle lacks the supported observation bridge; use a compatible existing bundle".into());
        }
        index=index.replace(anchor,&format!("{anchor}\nif (new URLSearchParams(location.search).get('flygon') === '1') window.__flygonGameBridge = bridge;"));
    }
    if !index.contains("__flygonActivateAudio") {
        if !index.contains(anchor) {
            return Err("Game bundle lacks the supported audio bridge".into());
        }
        index=index.replace(anchor,&format!("{anchor}\nif (new URLSearchParams(location.search).get('flygon') === '1') window.__flygonActivateAudio = resumeAudio;"));
    }
    for name in ["crystal_flygon.js", "crystal_flygon_bg.wasm"] {
        safe_file(&game.join("flygon").join(name))?;
    }
    let glue = fs::read_to_string(game.join("flygon/crystal_flygon.js"))?;
    if !glue.contains("AnatomyView") || !glue.contains("view_connections") {
        return Err("Neural WASM bindings are stale; build Flygon first".into());
    }
    let parent = out.parent().ok_or("Output needs a parent directory")?;
    fs::create_dir_all(parent)?;
    let staging = parent.join(format!(".flygon-stage-{}", std::process::id()));
    fs::create_dir(&staging)?;
    let result = (|| -> Result<()> {
        let mut files = BTreeMap::new();
        // Game inventory is copied unchanged except for the two optional bridges.
        // No saves, source maps, tests, stale Flygon files or developer captures.
        for file in fs::read_dir(game)? {
            let file = file?;
            let name = file
                .file_name()
                .into_string()
                .map_err(|_| "Non-UTF8 game filename")?;
            if name.starts_with("flygon")
                || name.starts_with('.')
                || name.contains(".test.")
                || (name == "index.html" || name == "index.html.gz")
            {
                continue;
            }
            if !["js", "css", "wasm", "crystalpack", "gz"]
                .iter()
                .any(|ext| name.ends_with(&format!(".{ext}")))
            {
                continue;
            }
            copy_file(&file.path(), &staging, &name, &mut files)?;
        }
        fs::write(staging.join("index.html"), index)?;
        files.insert("index.html".into(), entry(&staging.join("index.html"))?);
        for name in UI {
            copy_file(
                &repo.join("web-client").join(name),
                &staging,
                name,
                &mut files,
            )?;
        }
        for name in ["crystal_flygon.js", "crystal_flygon_bg.wasm"] {
            copy_file(
                &game.join("flygon").join(name),
                &staging,
                &format!("flygon/{name}"),
                &mut files,
            )?;
        }
        for name in PROFILES
            .iter()
            .copied()
            .chain(["view", "dataset", "manifest"])
        {
            copy_file(
                &repo.join(format!("modpacks/flygon/{name}.json")),
                &staging,
                &format!("flygon-{name}.json"),
                &mut files,
            )?;
        }
        for name in ["graph.bin", "metadata.json"] {
            copy_file(
                &data.join(name),
                &staging,
                &format!("flygon-data/{name}"),
                &mut files,
            )?;
        }
        for (source, name) in [
            ("modpacks/flygon/README.md", "FLYGON.md"),
            ("modpacks/flygon/ATTRIBUTION.md", "FLYGON-ATTRIBUTION.md"),
        ] {
            copy_file(&repo.join(source), &staging, name, &mut files)?;
        }
        let runtime = staging.join("flygon-dev.json");
        fs::write(&runtime, br#"{"local_clock":false,"live_view":false}"#)?;
        files.insert("flygon-dev.json".into(), entry(&runtime)?);
        fs::write(
            staging.join("flygon-release.json"),
            serde_json::to_vec_pretty(&Release {
                schema_version: 1,
                modpack: manifest,
                files,
            })?,
        )?;
        verify(&staging)?;
        fs::rename(&staging, out)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result?;
    println!(
        "{}",
        json!({"release":out,"entrypoint":"/flygon","simulation":"local Rust/WASM","deployed":false})
    );
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 3 && args[0] == "--preview" {
        verify(Path::new(&args[1]))?;
        return preview::serve(Path::new(&args[1]), None, None, args[2].parse()?);
    }
    if args.len() == 5 && args[0] == "--dev" {
        return preview::serve(
            Path::new(&args[2]),
            Some(Path::new(&args[3])),
            Some(Path::new(&args[1])),
            args[4].parse()?,
        );
    }
    if args.len() == 2 && args[0] == "--verify" {
        return verify(Path::new(&args[1]));
    }
    if args.len() != 8 {
        return Err("Usage: flygon-package --workspace REPO --game GAME_BUNDLE --data PREPARED_DATA --out NEW_DIRECTORY\n       flygon-package --verify RELEASE_DIRECTORY\n       flygon-package --preview RELEASE_DIRECTORY PORT\n       flygon-package --dev REPO GAME_BUNDLE PREPARED_DATA PORT".into());
    }
    let mut flags = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !["--workspace", "--game", "--data", "--out"].contains(&pair[0].as_str())
            || flags
                .insert(pair[0].as_str(), PathBuf::from(&pair[1]))
                .is_some()
        {
            return Err("Unknown or repeated packaging option".into());
        }
    }
    assemble(
        &flags["--workspace"],
        &flags["--game"],
        &flags["--data"],
        &flags["--out"],
    )
}
