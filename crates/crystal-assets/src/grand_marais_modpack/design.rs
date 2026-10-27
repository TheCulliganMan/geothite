//! One editable source for town pixels, warps, signs and actors.
use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TownDesign {
    pub schema_version: u16,
    pub name: String,
    pub tileset: String,
    pub width: u16,
    pub height: u16,
    pub rows: Vec<String>,
    pub doors: Vec<Door>,
    pub actors: Vec<Actor>,
    pub signs: Vec<Sign>,
    pub interiors: Vec<Interior>,
    pub dialogue: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    pub notes: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Interior {
    pub map: String,
    pub template: String,
    pub width: u16,
    pub height: u16,
    pub rows: Vec<String>,
    pub exits: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Door {
    pub id: u16,
    pub tile: [u16; 2],
    pub target: String,
    pub label: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Actor {
    pub id: String,
    pub sprite: String,
    pub tile: [i16; 2],
    pub lines: Vec<String>,
    #[serde(default)]
    pub signal_lines: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Sign {
    pub tile: [u16; 2],
    pub label: String,
    pub lines: Vec<String>,
}

impl TownDesign {
    pub fn parse(text: &str) -> Result<Self> {
        let design: Self = serde_json::from_str(text)?;
        ensure!(
            design.schema_version == 1
                && design.name == "Grand Marais"
                && design.tileset == "johto",
            "unsupported Grand Marais design"
        );
        ensure!(
            design.rows.len() == usize::from(design.height),
            "town row count"
        );
        ensure!(!design.notes.is_empty(), "town requires authoring notes");
        for row in &design.rows {
            ensure!(
                row.split_whitespace().count() == usize::from(design.width),
                "town row width"
            );
        }
        for (index, door) in design.doors.iter().enumerate() {
            ensure!(
                usize::from(door.id) == index + 1 && !door.label.is_empty(),
                "doors must be numbered in interior-return order"
            );
            ensure!(
                door.tile[0] < design.width * 2 && door.tile[1] < design.height * 2,
                "door outside town"
            );
        }
        Ok(design)
    }
}
