//! Place the walker's spare ball in the dry pocket behind the two southern firs.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::json;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring::{command, refresh_event_section};
use crate::{CompiledGamePack, authored_city_expansion_events, authored_town_publication};

const FEN: &str = "MnLayeredFen";
const PICKUP: &str = "MnFenSouthSpareBall";
const HINT: &str = "MnFenSouthSpareBallText";
const RETURN: &str = "MnFenObserverAfter";
const HOME: &str = "MnFenText_observer_after";
const CLAIMED: &str = "MnFenSouthBallFound";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    tile: [u16; 2],
    approach: [u16; 2],
    item: String,
    flag: String,
    observer_pages: Vec<AuthoredDialoguePage>,
}

/// Installs one optional native hidden item and a post-opening clue until it is found.
///
/// Returns an error if the walking pocket, observer, item or flag allocation changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let cache: Cache = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/fen-clearings-v01/southern-cache.json"
    ))?;
    let mut fen = pack.data.maps.get(FEN).context(FEN)?.clone();
    let [x, y] = cache.tile;
    ensure!(
        [x, y] == [8, 23]
            && cache.approach == [8, 22]
            && kind(pack, &fen, cache.tile)? == "FLOOR"
            && kind(pack, &fen, cache.approach)? == "FLOOR"
            && kind(pack, &fen, [7, 23])? == "WALL"
            && kind(pack, &fen, [8, 24])? == "WALL"
            && kind(pack, &fen, [9, 23])? == "HEADBUTT_TREE"
            && kind(pack, &fen, [9, 22])? == "HEADBUTT_TREE"
            && reachable(pack, &fen, [39, 16], false)?.contains(&cache.approach),
        "missing dry pocket beside the cove's two southern firs"
    );
    let previous = json!([command("jumptext", &[HOME])]);
    ensure!(
        fen.scripts.get(RETURN) == Some(&previous)
            && pack.data.map_scripts.get(RETURN) == Some(&previous)
            && fen
                .events
                .bg_events
                .iter()
                .all(|e| [e.x, e.y] != cache.tile && e.event_type != "BGEVENT_ITEM")
            && fen.script_field_pickups.is_empty()
            && [PICKUP, HINT, CLAIMED]
                .iter()
                .all(|k| !pack.data.map_scripts.contains_key(*k))
            && pack.data.items.contains_key(&cache.item)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(&cache.flag),
        "unexpected hidden item, observer, or occupied label in the Fen"
    );
    authored_city_expansion_events::flags(pack, std::slice::from_ref(&cache.flag))?;
    fen.events.bg_events.push(BackgroundEvent {
        x,
        y,
        event_type: "BGEVENT_ITEM".into(),
        script: PICKUP.into(),
    });
    fen.scripts.insert(
        PICKUP.into(),
        json!([command("hiddenitem", &[&cache.item, &cache.flag])]),
    );
    fen.scripts.insert(CLAIMED.into(), previous);
    fen.scripts.insert(
        RETURN.into(),
        json!([
            command("checkevent", &[&cache.flag]),
            command("iftrue", &[CLAIMED]),
            command("opentext", &[]),
            command("writetext", &[HOME]),
            command("promptbutton", &[]),
            command("writetext", &[HINT]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[])
        ]),
    );
    let pages = authored_dialogue::compile(&cache.observer_pages)?;
    fen.scripts.insert(HINT.into(), pages.clone());
    fen.script_text_bodies
        .extend(crate::parse_script_text_bodies(
            FEN,
            &BTreeMap::from([(HINT.into(), pages)]),
        )?);
    fen.script_field_pickups = crate::parse_script_field_pickups(FEN, &fen.scripts, &fen.objects)?;
    fen.script_flag_commands = crate::parse_script_flag_commands(FEN, &fen.scripts)?;
    fen.script_control_commands = crate::parse_script_control_commands(FEN, &fen.scripts)?;
    fen.script_text_commands = crate::parse_script_text_commands(FEN, &fen.scripts)?;
    fen.script_runtime_commands = crate::parse_script_runtime_commands(FEN, &fen.scripts)?;
    refresh_event_section(&mut fen);
    authored_town_publication::replace(pack, fen)
}

#[cfg(test)]
#[path = "northstar_fen_cache_tests.rs"]
mod tests;
