//! Current curated Minnesota identities and legal level-aware story bindings.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

#[cfg(any(test, feature = "pack-authoring"))]
use crate::CompiledGamePack;

pub(crate) struct RegionalRoster {
    pub allowed: BTreeSet<String>,
    #[cfg(any(test, feature = "pack-authoring"))]
    bindings: BTreeMap<String, String>,
    // Runtime parsing still validates the complete evolution profile. Only
    // authoring needs to retain that ancestry for level-aware substitutions.
    #[cfg(any(test, feature = "pack-authoring"))]
    ancestors: BTreeMap<String, (u8, Option<String>)>,
}

impl RegionalRoster {
    pub(crate) fn current() -> Result<Self> {
        Self::active(
            include_str!("../../../planning/minnesota/pokemon/roster.json"),
            include_str!("../../../planning/minnesota/pokemon/roster-curation.json"),
            include_str!("../../../planning/minnesota/pokemon/species-profiles.json"),
        )
    }

    #[cfg(any(test, feature = "pack-authoring"))]
    pub(crate) fn with_bindings(mut self, bindings: &[(&str, &str)]) -> Result<Self> {
        for &(original, target) in bindings {
            ensure!(
                self.allowed.contains(target),
                "inactive replacement {target}"
            );
            self.bindings.insert(original.into(), target.into());
        }
        Ok(self)
    }

    #[cfg(any(test, feature = "pack-authoring"))]
    pub fn parse(source: &str, roster: &str, curation: &str, profiles: &str) -> Result<Self> {
        let source: Value = serde_json::from_str(source)?;
        ensure!(
            source["schema_version"] == 1 && source["id"] == "northstar-minnesota-only-v1",
            "supported regional story policy required"
        );
        let mut policy = Self::active(roster, curation, profiles)?;
        let bindings: BTreeMap<String, String> =
            serde_json::from_value(source["legacy_species_bindings"].clone())?;
        ensure!(
            bindings.values().all(|n| policy.allowed.contains(n)),
            "a story binding uses a cut or non-Minnesota species"
        );
        let starters: Vec<String> = serde_json::from_value(source["starters"].clone())?;
        ensure!(
            starters == ["POUCHID", "KINDLIT", "BIRCHIT"] && source["egg"]["species"] == "LOOBIT",
            "use the canonical regional starters and nursery egg"
        );
        policy.bindings = bindings;
        Ok(policy)
    }

    pub(crate) fn active(roster: &str, curation: &str, profiles: &str) -> Result<Self> {
        let roster: Value = serde_json::from_str(roster)?;
        let curation: Value = serde_json::from_str(curation)?;
        let profiles: Value = serde_json::from_str(profiles)?;
        ensure!(
            curation["schema_version"] == 1,
            "supported roster curation required"
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
        Ok(Self {
            allowed,
            #[cfg(any(test, feature = "pack-authoring"))]
            bindings: BTreeMap::new(),
            #[cfg(any(test, feature = "pack-authoring"))]
            ancestors,
        })
    }

    /// Map a story appearance, walking back its own family when an evolved
    /// target would require a higher level. An unmapped legacy name is an error.
    #[cfg(any(test, feature = "pack-authoring"))]
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
