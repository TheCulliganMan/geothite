//! Canonical static meshes stay raw JSON and articulated catalogs stay GLB in
//! Git. Only compiled artifacts are compressed; hierarchy and clips stay intact.
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
        } else if path
            .extension()
            .is_some_and(|ext| ext == "json" || ext == "glb")
        {
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
// Length-delimited input identity prevents stale independently cached worker
// artifacts from silently using another canonical catalog or CPU kernel.
fn hash_named(identity: &mut Sha256, name: &str, bytes: &[u8]) {
    identity.update((name.len() as u64).to_le_bytes());
    identity.update(name.as_bytes());
    identity.update((bytes.len() as u64).to_le_bytes());
    identity.update(bytes);
}
fn collect_rust(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read kernel directory") {
        let entry = entry.expect("read kernel entry");
        let kind = entry.file_type().expect("read kernel type");
        assert!(!kind.is_symlink(), "kernel inputs cannot be symlinks");
        let path = entry.path();
        if kind.is_dir() {
            collect_rust(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}
fn finish_kernel_identity(root: &Path, out: &Path, mut identity: Sha256) {
    let workspace = root
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let mut paths = vec![
        root.join("build.rs"),
        root.join("Cargo.toml"),
        workspace.join("Cargo.toml"),
        workspace.join("Cargo.lock"),
        workspace.join("rust-toolchain.toml"),
        workspace.join("modpacks/voxel-view/profiles.json"),
        workspace.join("assets/terrain/painted-sandstone.png"),
        workspace.join("assets/terrain/painted-bedrock.png"),
        workspace.join("art/battles/arena-palettes.json"),
    ];
    println!("cargo:rerun-if-changed={}", root.join("src").display());
    println!(
        "cargo:rerun-if-changed={}",
        workspace.join("crates/crystal-render-api/src").display()
    );
    collect_rust(&root.join("src"), &mut paths);
    collect_rust(&workspace.join("crates/crystal-render-api/src"), &mut paths);
    paths.sort();
    let mut observed = Vec::new();
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let before = fs::metadata(&path).expect("kernel input metadata");
        let bytes = fs::read(&path).expect("read kernel input");
        let after = fs::metadata(&path).expect("kernel input metadata after read");
        assert_eq!(
            (before.len(), before.modified().ok()),
            (after.len(), after.modified().ok()),
            "kernel input changed during identity capture; retry build"
        );
        let name = path
            .strip_prefix(workspace)
            .expect("kernel within workspace")
            .to_str()
            .expect("UTF-8 kernel path")
            .replace('\\', "/");
        hash_named(&mut identity, &name, &bytes);
        observed.push((path, after.len(), after.modified().ok()));
    }
    // Cargo/rustc do not provide a transaction over live source edits. Reject
    // edits observed during capture; final main/worker builds need one frozen
    // source snapshot, and runtime identity comparison rejects known mismatch.
    for (path, length, modified) in observed {
        let current = fs::metadata(path).expect("kernel input final metadata");
        assert_eq!(
            (length, modified),
            (current.len(), current.modified().ok()),
            "kernel snapshot changed during identity capture; retry build"
        );
    }
    for name in [
        "CARGO_CFG_TARGET_ARCH",
        "CARGO_CFG_TARGET_ENDIAN",
        "CARGO_CFG_TARGET_FEATURE",
        "CARGO_FEATURE_SCENERY_FULL_DETAIL",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        hash_named(
            &mut identity,
            name,
            env::var(name).unwrap_or_default().as_bytes(),
        );
    }
    let digest: [u8; 32] = identity.finalize().into();
    write_changed(
        &out.join("terrain_kernel_identity.rs"),
        format!("const TERRAIN_KERNEL_ID: [u8; 32] = {digest:?};\n").as_bytes(),
    );
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("crate root"));
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    println!("cargo:rerun-if-changed=models");
    let mut terrain_identity = env::var_os("CARGO_FEATURE_TERRAIN_WORKER").map(|_| {
        let mut identity = Sha256::new();
        identity.update(b"geothite-terrain-kernel-model-v1\0");
        identity
    });
    let mut paths = Vec::new();
    collect(&root.join("models"), &mut paths);
    paths.sort();
    let mut inventory = String::from(
        "const EMBEDDED_MODELS: &[(&str, crate::model_storage::Source<'static>)] = &[\n",
    );
    // Derive legacy rig bindings from the same canonical model inputs. Moving
    // a species to an authored GLB must not leave a stale JSON include behind.
    let mut legacy_rigs = String::from("fn legacy_rig(species: &str) -> Option<&'static crate::species_rig::SpeciesRig> { match species {\n");
    let mut expected_outputs = HashSet::new();
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let raw = fs::read(&path).expect("read raw model");
        if let Some(identity) = terrain_identity.as_mut() {
            let name = path
                .strip_prefix(&root)
                .expect("model within crate")
                .to_str()
                .expect("UTF-8 model path")
                .replace('\\', "/");
            hash_named(identity, &format!("canonical/{name}"), &raw);
        }
        assert!(
            !raw.is_empty() && raw.len() <= 16 * 1024 * 1024,
            "invalid model size"
        );
        if path.extension().is_some_and(|ext| ext == "glb") {
            assert!(raw.len() >= 20, "truncated canonical GLB");
            assert_eq!(&raw[..4], b"glTF", "canonical GLB magic");
            assert_eq!(
                u32::from_le_bytes(raw[4..8].try_into().unwrap()),
                2,
                "canonical GLB version"
            );
            assert_eq!(
                u32::from_le_bytes(raw[8..12].try_into().unwrap()) as usize,
                raw.len(),
                "canonical GLB length"
            );
        } else {
            let text = std::str::from_utf8(&raw).expect("UTF-8 model JSON");
            let json: serde_json::Value = serde_json::from_str(text).expect("raw model JSON");
            assert!(
                json.is_object() && json.get("storage").is_none(),
                "canonical JSON must be raw"
            );
        }
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
        if let Some(species) = name.strip_prefix("models/battle_species/")
            .and_then(|name| name.strip_suffix(".mesh.json")) {
            assert!(species.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'));
            legacy_rigs.push_str(&format!(
                "{species:?} => {{ static RIG: OnceLock<crate::species_rig::SpeciesRig> = OnceLock::new(); Some(RIG.get_or_init(|| {{ let (mesh, anatomy) = parse_model(crate::model_storage::include_model!({name:?})).expect(\"valid named species mesh\"); crate::species_rig::motion::from_mesh({species:?}, mesh, anatomy) }})) }},\n"
            ));
        }
        let output = out.join(relative);
        let gzip_path = PathBuf::from(format!("{}.gz", output.display()));
        let metadata_path = PathBuf::from(format!("{}.meta.json", output.display()));
        write_changed(&gzip_path, &payload);
        write_changed(&metadata_path, &metadata);
        expected_outputs.extend([gzip_path, metadata_path]);
    }
    remove_obsolete(&out.join("models"), &expected_outputs);
    legacy_rigs.push_str("_ => None } }\n");
    write_changed(&out.join("legacy_species_rigs.rs"), legacy_rigs.as_bytes());
    inventory.push_str("];\n");
    write_changed(&out.join("model_inventory.rs"), inventory.as_bytes());
    if let Some(identity) = terrain_identity {
        finish_kernel_identity(&root, &out, identity);
    }
}
