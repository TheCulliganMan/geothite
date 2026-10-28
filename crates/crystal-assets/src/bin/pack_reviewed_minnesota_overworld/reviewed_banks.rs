//! Verify review provenance and recompile sources before installing runtime art.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_assets::CompiledGamePack;
use crystal_assets::minnesota_overworld_sprites::{OverworldCompilation, OverworldManifest};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn read(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| path.display().to_string())?)
        .context("parse reviewed authoring JSON")
}
fn array(value: &Value) -> Result<&Vec<Value>> {
    value.as_array().context("expected an authoring array")
}
fn string(value: &Value) -> Result<&str> {
    value.as_str().context("expected an authoring string")
}
fn checked_record(root: &Path, record: &Value) -> Result<Vec<u8>> {
    let path = Path::new(string(&record["path"])?).canonicalize()?;
    ensure!(
        path.starts_with(root),
        "review record escapes overworld root"
    );
    let bytes = fs::read(&path)?;
    ensure!(
        format!("{:x}", Sha256::digest(&bytes)) == string(&record["sha256"])?,
        "reviewed bytes changed: {}",
        path.display()
    );
    Ok(bytes)
}
fn active_roster(root: &Path) -> Result<BTreeMap<String, String>> {
    let roster = read(&root.join("roster.json"))?;
    let curation = read(&root.join("roster-curation.json"))?;
    ensure!(
        curation["schema_version"] == 1,
        "unsupported curation schema"
    );
    let mut retired = BTreeSet::new();
    for family in array(&curation["retired_families"])? {
        ensure!(
            retired.insert(string(&family["id"])?),
            "duplicate retired family"
        );
    }
    let mut families = BTreeSet::new();
    let mut active_families = 0;
    let mut active = BTreeMap::new();
    for family in array(&roster["families"])? {
        let id = string(&family["id"])?;
        ensure!(families.insert(id), "duplicate roster family");
        if retired.contains(id) {
            continue;
        }
        active_families += 1;
        for member in array(&family["members"])? {
            ensure!(
                active
                    .insert(
                        string(&member["id"])?.to_owned(),
                        string(&member["name"])?.to_owned()
                    )
                    .is_none(),
                "duplicate active species"
            );
        }
    }
    ensure!(retired.is_subset(&families), "unknown retired family");
    ensure!(
        !active.is_empty()
            && curation["active_species_count"].as_u64() == Some(active.len() as u64)
            && curation["active_family_count"].as_u64() == Some(active_families),
        "curation counts differ from active roster"
    );
    Ok(active)
}

pub(super) enum Coverage {
    Complete,
    Reviewed,
}

pub(super) fn load(
    root: &Path,
    overworld: &Path,
    base: &CompiledGamePack,
    coverage: Coverage,
) -> Result<OverworldCompilation> {
    let overworld = overworld.canonicalize()?;
    let registry = read(&overworld.join("accepted.json"))?;
    let active = active_roster(root)?;
    let entries = array(&registry["species"])?;
    ensure!(
        registry["schema_version"] == 1
            && registry["required_species"].as_u64() == Some(active.len() as u64)
            && registry["accepted_species"].as_u64() == Some(entries.len() as u64)
            && !entries.is_empty(),
        "review registry counts differ from current roster"
    );
    if matches!(coverage, Coverage::Complete) {
        ensure!(
            registry["complete"] == true && entries.len() == active.len(),
            "complete reviewed roster required: {} accepted, {} active",
            entries.len(),
            active.len()
        );
    }
    let palettes = read(&root.join("palettes.json"))?;
    let profiles = read(&root.join("species-profiles.json"))?;
    let mut shiny = BTreeMap::new();
    for profile in array(&profiles["species"])? {
        ensure!(
            shiny
                .insert(string(&profile["id"])?, &profile["shiny_rgb5"])
                .is_none(),
            "duplicate profile"
        );
    }
    let mut compiled = BTreeMap::new();
    let mut files = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for entry in entries {
        let id = string(&entry["species_id"])?;
        let name = active
            .get(id)
            .context("review references inactive species")?;
        ensure!(
            seen.insert(id) && entry["name"] == *name && entry["status"] == "accepted",
            "invalid reviewed identity {id}"
        );
        let token = name.to_ascii_lowercase();
        ensure!(
            token.bytes().all(|b| b.is_ascii_lowercase()),
            "unsafe species token"
        );
        ensure!(
            base.data().pokemon.contains_key(&name.to_ascii_uppercase()),
            "base pack lacks {name}"
        );
        ensure!(
            entry["root_visual_review"]["accepted"] == true,
            "missing root visual acceptance {id}"
        );
        let proof: Value = serde_json::from_slice(&checked_record(
            &overworld,
            &entry["source_feature_review"],
        )?)?;
        ensure!(
            proof["result"] == "pass" && proof["species_id"] == id,
            "source feature review is not passing for {id}"
        );
        let manifest_record = &entry["selected_manifest"];
        let manifest_bytes = checked_record(&overworld, manifest_record)?;
        ensure!(
            proof["manifest_sha256"] == manifest_record["sha256"],
            "source review manifest mismatch {id}"
        );
        for source in array(&entry["sources"])? {
            checked_record(&overworld, source)?;
        }
        let manifest_path = string(&manifest_record["path"])?;
        if !compiled.contains_key(manifest_path) {
            let manifest = OverworldManifest::from_json(&manifest_bytes)?;
            let candidate = manifest.compile(
                Path::new(manifest_path)
                    .parent()
                    .context("manifest directory")?,
            )?;
            compiled.insert(manifest_path, candidate);
        }
        let candidate = &compiled[manifest_path];
        let records = array(&entry["runtime_files"])?;
        ensure!(
            records.len() == 4,
            "four reviewed runtime files required for {id}"
        );
        for suffix in ["2bpp", "png", "gbcpal", "shiny.gbcpal"] {
            let filename = if suffix == "shiny.gbcpal" {
                "overworld-shiny.gbcpal".to_owned()
            } else {
                format!("overworld.{suffix}")
            };
            let key = format!("gfx/pokemon/{token}/{filename}");
            let expected = overworld.join("native").join(&key).canonicalize()?;
            let matching: Vec<_> = records
                .iter()
                .filter(|r| {
                    r["path"].as_str().is_some_and(|p| {
                        Path::new(p).canonicalize().ok().as_ref() == Some(&expected)
                    })
                })
                .collect();
            ensure!(matching.len() == 1, "missing/duplicate reviewed file {key}");
            let bytes = checked_record(&overworld, matching[0])?;
            if suffix == "2bpp" {
                ensure!(
                    proof["bank_sha256"] == matching[0]["sha256"],
                    "source review bank mismatch {id}"
                );
            }
            ensure!(
                candidate.runtime_files.get(&key) == Some(&bytes),
                "source recompile differs for {key}"
            );
            if suffix == "gbcpal" {
                let words: [u16; 4] =
                    serde_json::from_value(palettes["pokemon"][id]["gbc_rgb555_words"].clone())?;
                ensure!(
                    bytes
                        == words
                            .into_iter()
                            .flat_map(u16::to_le_bytes)
                            .collect::<Vec<_>>(),
                    "normal palette changed for {id}"
                );
            } else if suffix == "shiny.gbcpal" {
                let colors: [[u8; 3]; 2] = serde_json::from_value(
                    (*shiny.get(id).context("missing shiny profile")?).clone(),
                )?;
                ensure!(
                    colors.iter().flatten().all(|v| *v < 32),
                    "invalid shiny RGB5"
                );
                let words = [32767, rgb555(colors[0]), rgb555(colors[1]), 0];
                ensure!(
                    bytes
                        == words
                            .into_iter()
                            .flat_map(u16::to_le_bytes)
                            .collect::<Vec<_>>(),
                    "shiny palette changed for {id}"
                );
            }
            ensure!(
                files.insert(key, bytes).is_none(),
                "duplicate runtime asset"
            );
        }
    }
    if matches!(coverage, Coverage::Complete) {
        ensure!(
            seen == active.keys().map(String::as_str).collect(),
            "review coverage differs from active roster"
        );
    }
    Ok(OverworldCompilation {
        runtime_files: files,
        review_files: BTreeMap::new(),
    })
}
fn rgb555([r, g, b]: [u8; 3]) -> u16 {
    u16::from(r) | (u16::from(g) << 5) | (u16::from(b) << 10)
}

pub(super) fn verify_preservation(
    base: &CompiledGamePack,
    published: &CompiledGamePack,
    banks: &OverworldCompilation,
) -> Result<()> {
    ensure!(
        serde_json::to_vec(base.data())? == serde_json::to_vec(published.data())?,
        "game data changed during art publication"
    );
    ensure!(
        base.compiled_audio() == published.compiled_audio()
            && base.audio_manifest()? == published.audio_manifest()?,
        "audio changed during art publication"
    );
    let mut expected = base.runtime_files().clone();
    for (key, bytes) in &banks.runtime_files {
        if let Some(old) = expected.insert(key.clone(), bytes.clone()) {
            ensure!(old == *bytes, "existing asset conflict {key}");
        }
    }
    ensure!(
        &expected == published.runtime_files(),
        "unrelated runtime content changed"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use serde_json::json;
    use sha2::{Digest, Sha256};

    use super::{active_roster, checked_record};

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "geothite-reviewed-banks-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("create distinct fixture");
            Self(path.canonicalize().expect("canonical fixture"))
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn changed_or_escaped_review_records_cannot_publish() {
        let root = Fixture::new();
        let file = root.0.join("bank.bin");
        fs::write(&file, b"reviewed").expect("write reviewed file");
        let record = json!({"path": file, "sha256": format!("{:x}", Sha256::digest(b"reviewed"))});
        assert_eq!(
            checked_record(&root.0, &record).expect("unchanged review"),
            b"reviewed"
        );
        fs::write(&file, b"changed").expect("replace fixture");
        assert!(
            checked_record(&root.0, &record)
                .expect_err("reject changed bytes")
                .to_string()
                .contains("changed")
        );
        let outside = Fixture::new();
        assert!(
            checked_record(&outside.0, &record)
                .expect_err("reject external path")
                .to_string()
                .contains("escapes")
        );
    }

    #[test]
    fn active_scope_keeps_complete_uncut_families_and_rejects_stale_counts() {
        let root = Fixture::new();
        fs::write(root.0.join("roster.json"), serde_json::to_vec(&json!({"families":[
            {"id":"MN_F001","members":[{"id":"MN_001","name":"First"},{"id":"MN_002","name":"Second"}]},
            {"id":"MN_F002","members":[{"id":"MN_003","name":"Cut"}]}
        ]})).expect("roster JSON")).expect("write roster");
        let mut curation = json!({"schema_version":1,"active_species_count":2,"active_family_count":1,"retired_families":[{"id":"MN_F002"}]});
        let save = |value: &serde_json::Value| {
            fs::write(
                root.0.join("roster-curation.json"),
                serde_json::to_vec(value).expect("curation JSON"),
            )
            .expect("write curation")
        };
        save(&curation);
        let active = active_roster(&root.0).expect("complete active family");
        assert_eq!(
            active.keys().map(String::as_str).collect::<Vec<_>>(),
            ["MN_001", "MN_002"]
        );
        curation["active_species_count"] = json!(1);
        save(&curation);
        assert!(
            active_roster(&root.0)
                .expect_err("stale scope")
                .to_string()
                .contains("counts")
        );
        curation["retired_families"][0]["id"] = json!("MN_F999");
        save(&curation);
        assert!(
            active_roster(&root.0)
                .expect_err("unknown cut")
                .to_string()
                .contains("unknown")
        );
    }
}
