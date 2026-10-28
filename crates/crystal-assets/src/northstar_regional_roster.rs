//! Current curated Minnesota identities and legal level-aware story bindings.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;

pub(crate) struct RegionalRoster {
    pub allowed: BTreeSet<String>,
    bindings: BTreeMap<String, String>,
    ancestors: BTreeMap<String, (u8, Option<String>)>,
}

impl RegionalRoster {
    pub fn parse(source: &str, roster: &str, curation: &str, profiles: &str) -> Result<Self> {
        let source: Value = serde_json::from_str(source)?;
        let roster: Value = serde_json::from_str(roster)?;
        let curation: Value = serde_json::from_str(curation)?;
        let profiles: Value = serde_json::from_str(profiles)?;
        ensure!(
            source["schema_version"] == 1
                && source["id"] == "northstar-minnesota-only-v1"
                && curation["schema_version"] == 1,
            "supported regional story policy required"
        );
        let retired = curation["retired_families"]
            .as_array()
            .context("curated retirements")?
            .iter()
            .map(|f| f["id"].as_str().context("retired family"))
            .collect::<Result<BTreeSet<_>>>()?;
        let profile = profiles["species"]
            .as_array()
            .context("native profiles")?
            .iter()
            .map(|p| Ok((p["id"].as_str().context("profile id")?, p)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let mut allowed = BTreeSet::new();
        let mut ancestors = BTreeMap::new();
        let mut families = 0;
        for family in roster["families"]
            .as_array()
            .context("canonical families")?
        {
            if retired.contains(family["id"].as_str().context("family id")?) {
                continue;
            }
            families += 1;
            let mut previous = None;
            let mut minimum = 1;
            for member in family["members"].as_array().context("family members")? {
                let name = member["name"]
                    .as_str()
                    .context("runtime name")?
                    .to_ascii_uppercase();
                ensure!(
                    allowed.insert(name.clone()),
                    "duplicate active species {name}"
                );
                ancestors.insert(name.clone(), (minimum, previous));
                let id = member["id"].as_str().context("stable species id")?;
                let evolution = &profile.get(id).context("complete species profile")?["evolution"];
                if evolution["method"] == "LEVEL" {
                    minimum = minimum.max(u8::try_from(
                        evolution["level"]
                            .as_u64()
                            .context("native evolution level")?,
                    )?);
                }
                previous = Some(name);
            }
        }
        ensure!(
            allowed.len() as u64
                == curation["active_species_count"]
                    .as_u64()
                    .context("curated species count")?
                && families as u64
                    == curation["active_family_count"]
                        .as_u64()
                        .context("curated family count")?,
            "regional source and active curation differ"
        );
        let bindings: BTreeMap<String, String> =
            serde_json::from_value(source["legacy_species_bindings"].clone())?;
        ensure!(
            bindings.values().all(|n| allowed.contains(n)),
            "a story binding uses a cut or non-Minnesota species"
        );
        let starters: Vec<String> = serde_json::from_value(source["starters"].clone())?;
        ensure!(
            starters == ["POUCHID", "KINDLIT", "BIRCHIT"] && source["egg"]["species"] == "GILLPUP",
            "use the canonical regional starters and nursery egg"
        );
        Ok(Self {
            allowed,
            bindings,
            ancestors,
        })
    }

    /// Map a story appearance, walking back its own family when an evolved
    /// target would require a higher level. An unmapped legacy name is an error.
    pub fn target(
        &self,
        pack: &CompiledGamePack,
        original: &str,
        level: Option<u8>,
    ) -> Result<String> {
        let mut name = if self.allowed.contains(original) {
            original
        } else {
            self.bindings
                .get(original)
                .map(String::as_str)
                .with_context(|| format!("unmapped out-of-region story species {original}"))?
        };
        if let Some(level) = level {
            loop {
                let (minimum, previous) = self.ancestors.get(name).context("active family rule")?;
                if level >= *minimum {
                    break;
                }
                name = previous
                    .as_deref()
                    .context("no legal family member at this level")?;
            }
        }
        ensure!(
            pack.data.pokemon.contains_key(name),
            "active Minnesota story species {name} is absent from the actual pack"
        );
        Ok(name.into())
    }
}
