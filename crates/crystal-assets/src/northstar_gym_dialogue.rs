//! Canonical room clues and trainer lessons, preserving reward conversations.

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::MapModule;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::gym_design::Gym;

pub(crate) fn install(room: &mut MapModule, gym: &Gym, source: &str) -> Result<()> {
    let story: Value = serde_json::from_str(source)?;
    let texts = story["gym_overhaul"]["dialogue"][&gym.id]
        .as_object()
        .context("canonical gym dialogue")?;
    for (label, pages) in texts {
        ensure!(
            room.scripts.contains_key(label),
            "canonical gym text has no installed owner: {label}"
        );
        let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(pages.clone())?;
        room.scripts
            .insert(label.clone(), authored_dialogue::compile(&pages)?);
    }
    for label in [
        format!("{}GuideText", gym.id),
        format!("{}LeaderSeenText", gym.id),
    ]
    .into_iter()
    .chain(gym.trainers.iter().flat_map(|trainer| {
        [
            format!("{}Seen", trainer.id),
            format!("{}AfterText", trainer.id),
        ]
    })) {
        ensure!(
            texts.contains_key(&label),
            "missing revised room lesson: {label}"
        );
    }
    Ok(())
}
