//! Point the lower Tamarack Loop ranger to the erratic that holds NeverMeltIce.
use anyhow::{Context, Result};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_publication;

const RANGER_MAP: &str = "MnR20C00R00";
const CLUE: &str = "MnDiscoveryR20ClueText";

/// Publishes the ranger's revised pre-collection pages on the northern trail.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(RANGER_MAP)
        .context("Tamarack ranger map")?
        .clone();
    let story: Value =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    let pages: Vec<AuthoredDialoguePage> =
        serde_json::from_value(story["story_flow_revision"]["dialogue"][RANGER_MAP][CLUE].clone())?;
    *room
        .scripts
        .get_mut(CLUE)
        .context("installed ranger clue")? = authored_dialogue::compile(&pages)?;
    room.script_text_bodies = crate::parse_script_text_bodies(RANGER_MAP, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
