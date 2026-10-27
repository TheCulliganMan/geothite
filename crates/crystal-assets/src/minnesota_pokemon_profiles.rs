//! Strict authoring contract for original Minnesota species profiles.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_core::models::BaseStats;
use serde::Deserialize;

use crate::PokemonCryMetadata;

#[derive(Debug, Deserialize)]
pub(crate) struct Roster {
    pub families: Vec<Family>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Family {
    pub id: String,
    pub members: Vec<Member>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Member {
    pub id: String,
    pub name: String,
    pub types: Vec<String>,
    pub evolves_to: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileBatch {
    schema_version: u16,
    species: Vec<Profile>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    pub id: String,
    pub classification: String,
    pub height_inches: u16,
    pub weight_tenths_lb: u16,
    pub dex_pages: Vec<String>,
    pub base_stats: BaseStats,
    pub catch_rate: u8,
    pub base_exp: u16,
    pub gender_ratio: u8,
    pub hatch_cycles: u8,
    pub growth_rate: String,
    pub egg_groups: [String; 2],
    pub level_moves: Vec<(u8, String)>,
    pub tmhm_moves: Vec<String>,
    pub egg_moves: Vec<String>,
    pub evolution: Option<ProfileEvolution>,
    pub held_items: [Option<String>; 2],
    pub menu_icon: String,
    pub cry: PokemonCryMetadata,
    pub shiny_rgb5: [[u8; 3]; 2],
    /// An explicit species footprint already present in the supplied base.
    pub footprint_species: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfileEvolution {
    pub method: String,
    #[serde(default)]
    pub level: Option<u8>,
    #[serde(default)]
    pub item: Option<String>,
    #[serde(default)]
    pub held_item: Option<String>,
    #[serde(default)]
    pub happiness: Option<String>,
    #[serde(default)]
    pub stat_ratio: Option<String>,
}

pub(crate) fn load_profiles(directory: &Path) -> Result<BTreeMap<String, Profile>> {
    let mut paths = if directory.is_file() {
        vec![directory.to_path_buf()]
    } else {
        fs::read_dir(directory)
            .with_context(|| format!("read profile directory {}", directory.display()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?
    };
    paths.sort();
    let mut profiles = BTreeMap::new();
    for path in paths {
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let batch: ProfileBatch = serde_json::from_slice(&fs::read(&path)?)
            .with_context(|| format!("parse profile batch {}", path.display()))?;
        ensure!(batch.schema_version == 1, "unsupported profile schema");
        ensure!(
            !batch.species.is_empty(),
            "empty profile batch {}",
            path.display()
        );
        for profile in batch.species {
            profile
                .validate()
                .with_context(|| format!("profile {}", profile.id))?;
            let id = profile.id.clone();
            ensure!(
                profiles.insert(id.clone(), profile).is_none(),
                "duplicate profile {id}"
            );
        }
    }
    ensure!(!profiles.is_empty(), "no species profile batches found");
    Ok(profiles)
}

impl Profile {
    pub(crate) fn height_digits(&self) -> Result<u16> {
        (self.height_inches / 12)
            .checked_mul(100)
            .and_then(|feet| feet.checked_add(self.height_inches % 12))
            .context("height exceeds Crystal digit field")
    }

    pub(crate) fn shiny_palette(&self) -> Result<[u16; 4]> {
        let mut palette = [0x7fff, 0, 0, 0];
        for (index, color) in self.shiny_rgb5.iter().enumerate() {
            ensure!(
                color.iter().all(|channel| *channel < 32),
                "shiny RGB5 outside0..31"
            );
            palette[index + 1] =
                u16::from(color[0]) | (u16::from(color[1]) << 5) | (u16::from(color[2]) << 10);
        }
        ensure!(
            palette
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == 4,
            "shiny palette must have four distinct colors"
        );
        Ok(palette)
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.id.starts_with("MN_")
                && self.id[3..].parse::<u16>().is_ok_and(|number| number > 0),
            "invalid regional species ID"
        );
        ensure!(
            !self.classification.is_empty()
                && self.classification.trim() == self.classification
                && self.classification.is_ascii()
                && !self.classification.chars().any(char::is_control)
                && self.classification.len() <= 11,
            "classification must be1..11 trimmed ASCII characters"
        );
        ensure!(
            self.dex_pages.len() == 2,
            "exactly two Pokédex pages required"
        );
        for page in &self.dex_pages {
            let lines: Vec<_> = page.split(" @ ").collect();
            ensure!(
                lines.len() == 3
                    && lines.iter().all(|line| !line.is_empty()
                        && line.trim() == *line
                        && line.is_ascii()
                        && !line.contains('@')
                        && line.len() <= 18
                        && !line.chars().any(char::is_control)),
                "each dex page must contain three trimmed ASCII lines of1..18 characters, separated by ' @ '"
            );
        }
        ensure!(
            self.height_inches > 0 && self.weight_tenths_lb > 0,
            "positive height/weight required"
        );
        self.height_digits()?;
        ensure!(
            self.catch_rate > 0 && self.base_exp > 0 && self.hatch_cycles > 0,
            "catch rate, base experience and hatch cycles must be positive"
        );
        for value in [
            self.base_stats.hp,
            self.base_stats.attack,
            self.base_stats.defense,
            self.base_stats.speed,
            self.base_stats.special_attack,
            self.base_stats.special_defense,
        ] {
            ensure!((1..=255).contains(&value), "base stats must be1..255");
        }
        ensure!(
            !self.level_moves.is_empty() && self.level_moves.iter().any(|(level, _)| *level == 1),
            "learnset needs level1 moves"
        );
        ensure!(
            self.level_moves
                .iter()
                .all(|(level, _)| (1..=100).contains(level))
                && self
                    .level_moves
                    .windows(2)
                    .all(|pair| pair[0].0 <= pair[1].0),
            "level moves must be sorted levels1..100"
        );
        self.shiny_palette()?;
        if let Some(evolution) = &self.evolution
            && matches!(evolution.method.as_str(), "LEVEL" | "STAT")
        {
            ensure!(
                evolution
                    .level
                    .is_some_and(|level| (1..=100).contains(&level)),
                "level/stat evolution needs level 1..100"
            );
        }
        Ok(())
    }
}
