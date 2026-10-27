//! Pin the authored gym content while allowing its three city warp bindings.
use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::MapModule;

pub(crate) fn content_hash(module: &MapModule) -> Result<String> {
    let mut content = module.clone();
    content.events.warps.clear();
    content
        .map_event_section_commands
        .retain(|command| command.command != "warp_event");
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&serde_json::to_value(content)?)?)
    ))
}
