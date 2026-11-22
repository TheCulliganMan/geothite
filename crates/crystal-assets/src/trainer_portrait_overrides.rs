//! Pack-owned battle artwork selected by trainer identity, independently of class.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::models::TrainerCatalog;
use serde::{Deserialize, Serialize};

/// Optional resource whose bytes participate in the existing pack identity.
pub const TRAINER_PORTRAIT_OVERRIDES_PATH: &str = "gfx/trainers/portrait-overrides.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TrainerPortraitOverride {
    pub trainer_class: String,
    pub portrait: String,
}

/// Validated presentation bindings. Missing entries retain ordinary class art.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrainerPortraitOverrides {
    pub(crate) trainers: BTreeMap<String, TrainerPortraitOverride>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u8,
    trainers: BTreeMap<String, TrainerPortraitOverride>,
}

impl TrainerPortraitOverrides {
    /// Read the optional manifest and validate exact trainer/class references and banks.
    ///
    /// # Errors
    /// Rejects malformed manifests, unknown trainers, mismatched classes, unsafe
    /// portrait names, or missing/invalid native battle resources.
    pub fn from_runtime_files(
        files: &BTreeMap<String, Vec<u8>>,
        catalog: &TrainerCatalog,
    ) -> Result<Self> {
        let Some(bytes) = files.get(TRAINER_PORTRAIT_OVERRIDES_PATH) else {
            return Ok(Self::default());
        };
        let manifest: Manifest =
            serde_json::from_slice(bytes).context("trainer portrait overrides")?;
        ensure!(
            manifest.schema_version == 1,
            "unsupported trainer portrait override version"
        );
        for (id, entry) in &manifest.trainers {
            let trainer = catalog
                .get(id)
                .with_context(|| format!("unknown portrait trainer {id}"))?;
            ensure!(
                trainer.trainer_id == *id && trainer.trainer_class == entry.trainer_class,
                "portrait trainer class mismatch: {id}"
            );
            ensure!(
                valid_portrait_id(&entry.portrait),
                "invalid portrait identity: {}",
                entry.portrait
            );
            let stem = format!("gfx/trainers/{}", entry.portrait);
            let png = files
                .get(&format!("{stem}.png"))
                .context("missing override portrait PNG")?;
            ensure!(
                png.len() >= 24
                    && &png[..8] == b"\x89PNG\r\n\x1a\n"
                    && png[16..20] == 56_u32.to_be_bytes()
                    && png[20..24] == 56_u32.to_be_bytes(),
                "override portrait must be a native 56x56 PNG"
            );
            ensure!(
                files
                    .get(&format!("{stem}.2bpp"))
                    .is_some_and(|bytes| bytes.len() == 784),
                "override portrait must have a 784-byte tile bank"
            );
            ensure!(
                files
                    .get(&format!("{stem}.gbcpal"))
                    .is_some_and(|bytes| bytes.len() == 8),
                "override portrait must have an eight-byte palette"
            );
        }
        Ok(Self {
            trainers: manifest.trainers,
        })
    }

    /// Resolve only an exact ID/class binding; other trainers retain class artwork.
    pub fn resolve(&self, trainer_id: &str, trainer_class: &str) -> String {
        self.trainers
            .get(trainer_id)
            .filter(|entry| entry.trainer_class == trainer_class)
            .map(|entry| entry.portrait.clone())
            .unwrap_or_else(|| trainer_class_portrait(trainer_class))
    }

    #[cfg(any(test, feature = "pack-authoring"))]
    pub(crate) fn to_json(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec_pretty(&Manifest {
            schema_version: 1,
            trainers: self.trainers.clone(),
        })?)
    }
}

pub(crate) fn valid_portrait_id(id: &str) -> bool {
    id.starts_with("mn_portrait_")
        && id.len() > "mn_portrait_".len()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

/// Convert a trainer class to its established battle resource stem.
pub fn trainer_class_portrait(trainer_class: &str) -> String {
    let normalized = trainer_class.trim().to_ascii_lowercase();
    if normalized == "pokemon_prof" {
        return "oak".into();
    }
    if normalized == "medium" {
        return normalized;
    }
    if normalized.ends_with('m') && !normalized.ends_with("_m") {
        format!("{}_m", &normalized[..normalized.len() - 1])
    } else if normalized.ends_with('f') && !normalized.ends_with("_f") {
        format!("{}_f", &normalized[..normalized.len() - 1])
    } else {
        normalized
    }
}
