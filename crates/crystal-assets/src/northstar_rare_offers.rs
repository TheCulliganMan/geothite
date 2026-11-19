//! Give each optional wild battle an explicit choice in its existing dialogue.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_dialogue;

/// Updates the eight remaining rare encounter prompts from the canonical story.
///
/// Returns an error if the original question or either installed copy differs.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    for (map, label) in [
        ("MnBoundaryIntake", "MnVerdigrimEncounterText"),
        ("MnDevilsKettle", "MnKettleAskText"),
        ("MnFieldMargin", "MnKensigardText_ask"),
        ("MnInnerBasin", "MnBasinText_ask"),
        ("MnLayeredFen", "MnFenText_ask"),
        ("MnMississippiLowerWoodsC01R00", "MnPepinText_ask"),
        ("MnR03Cave1F", "MnJawOfferText"),
        ("MnR08C00R00", "MnAncientOfferText"),
    ] {
        let room = pack
            .data
            .maps
            .get_mut(map)
            .with_context(|| format!("missing {map}"))?;
        let old = room
            .scripts
            .get(label)
            .with_context(|| format!("missing {label}"))?;
        let last_line = old
            .as_array()
            .and_then(|lines| lines.iter().rev().nth(1))
            .and_then(|line| line["args"][0].as_str());
        ensure!(
            last_line.is_some_and(|line| line.ends_with("closer?\""))
                && pack.data.map_scripts.get(label) == Some(old),
            "unexpected parent offer {map}/{label}"
        );
        let mut body = BTreeMap::from([(label.to_owned(), old.clone())]);
        let parent = crate::parse_script_text_bodies(map, &body)?;
        ensure!(
            parent.contains_key(label) && room.script_text_bodies.get(label) == parent.get(label),
            "unexpected native pages {map}/{label}"
        );
        authored_dialogue::install_canonical(room, label)?;
        let revised = &room.scripts[label];
        body.insert(label.to_owned(), revised.clone());
        room.script_text_bodies
            .extend(crate::parse_script_text_bodies(map, &body)?);
        pack.data
            .map_scripts
            .insert(label.to_owned(), revised.clone());
    }
    Ok(())
}
