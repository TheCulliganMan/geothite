//! Canonical meshes stay raw JSON in Git. Only compiled artifacts are compressed.
use flate2::{Compression, GzBuilder};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

fn collect(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read model directory") {
        let entry = entry.expect("read model entry");
        let kind = entry.file_type().expect("read model type");
        assert!(!kind.is_symlink(), "model inputs cannot be symlinks");
        let path = entry.path();
        if kind.is_dir() {
            collect(&path, paths);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            paths.push(path);
        }
    }
}
fn write_changed(path: &Path, bytes: &[u8]) {
    if fs::read(path).ok().as_deref() != Some(bytes) {
        fs::create_dir_all(path.parent().expect("output parent")).expect("create output directory");
        fs::write(path, bytes).expect("write generated model");
    }
}
// A removed or renamed source must fail just like a clean build; an old Cargo
// output must never satisfy a now-invalid include_model! reference.
fn remove_obsolete(directory: &Path, expected: &HashSet<PathBuf>) {
    if !directory.exists() {
        return;
    }
    for entry in fs::read_dir(directory).expect("read generated model directory") {
        let entry = entry.expect("read generated model entry");
        let path = entry.path();
        let kind = entry.file_type().expect("generated model type");
        assert!(
            !kind.is_symlink(),
            "generated model outputs cannot be symlinks"
        );
        if kind.is_dir() {
            remove_obsolete(&path, expected);
        } else if !expected.contains(&path) {
            fs::remove_file(path).expect("remove obsolete generated model");
        }
    }
}
fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("crate root"));
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    println!("cargo:rerun-if-changed=models");
    let mut paths = Vec::new();
    collect(&root.join("models"), &mut paths);
    paths.sort();
    let mut inventory = String::from(
        "const EMBEDDED_MODELS: &[(&str, crate::model_storage::Source<'static>)] = &[\n",
    );
    let mut expected_outputs = HashSet::new();
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let raw = fs::read(&path).expect("read raw model");
        assert!(
            !raw.is_empty() && raw.len() <= 16 * 1024 * 1024,
            "invalid model size"
        );
        let text = std::str::from_utf8(&raw).expect("UTF-8 model JSON");
        let json: serde_json::Value = serde_json::from_str(text).expect("raw model JSON");
        assert!(
            json.is_object() && json.get("storage").is_none(),
            "canonical models must be raw JSON"
        );
        let mut encoder = GzBuilder::new()
            .mtime(0)
            .operating_system(255)
            .write(Vec::new(), Compression::new(6));
        encoder.write_all(&raw).expect("compress build artifact");
        let payload = encoder.finish().expect("finish compression");
        let metadata = serde_json::to_vec(&serde_json::json!({
            "storage": "geothite-model-gzip-file-v1",
            "bytes": raw.len(), "sha256": format!("{:x}", Sha256::digest(&raw)),
            "gzip_bytes": payload.len(), "gzip_sha256": format!("{:x}", Sha256::digest(&payload))
        }))
        .expect("serialize generated identity");
        let relative = path.strip_prefix(&root).expect("model within crate");
        let name = relative
            .to_str()
            .expect("UTF-8 model path")
            .replace('\\', "/");
        inventory.push_str(&format!(
            "({name:?}, crate::model_storage::include_model!({name:?})),\n"
        ));
        let output = out.join(relative);
        let gzip_path = PathBuf::from(format!("{}.gz", output.display()));
        let metadata_path = PathBuf::from(format!("{}.meta.json", output.display()));
        write_changed(&gzip_path, &payload);
        write_changed(&metadata_path, &metadata);
        expected_outputs.extend([gzip_path, metadata_path]);
    }
    remove_obsolete(&out.join("models"), &expected_outputs);
    inventory.push_str("];\n");
    write_changed(&out.join("model_inventory.rs"), inventory.as_bytes());
}
