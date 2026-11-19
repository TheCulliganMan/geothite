//! Direct the third badge holder to the current dispatch lead and the fourth to the bluff stairs.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule, authored_dialogue, authored_town_publication};

const GYM: &str = "MnMinneapolisGym";
const AVRIN: &str = "MnMinneapolisGymRematchBeforeChampion";
const IDLE: &str = "MnFlowAvrinAfterRing";
const SKETCH: &str = "EVENT_MN_DISPATCH_NOTE";

fn install_pages(room: &mut MapModule, label: &str) -> Result<()> {
    authored_dialogue::install_canonical(room, label)?;
    let body = BTreeMap::from([(label.to_owned(), room.scripts[label].clone())]);
    room.script_text_bodies
        .extend(crate::parse_script_text_bodies(&room.id, &body)?);
    Ok(())
}

/// Updates Avrin's repeat visit before the League and the directions on Route 5.
///
/// Returns an error if the parent has a different dispatch handoff or warden.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut gym = pack.data.maps.get(GYM).context(GYM)?.clone();
    let former = json!([
        command("checkevent", &[SKETCH]),
        command("iftrue", &[IDLE]),
        command("jumptext", &["MnMinneapolisGymLeaderAfterText"]),
    ]);
    ensure!(
        gym.scripts.get(AVRIN) == Some(&former)
            && pack.data.map_scripts.get(AVRIN) == Some(&former),
        "unexpected former Avrin handoff"
    );
    gym.scripts.insert(
        AVRIN.into(),
        json!([
            command("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
            command("iftrue", &[IDLE]),
            command("checkevent", &["EVENT_MN_DEEP_BORE_MANIFEST_READ"]),
            command("iftrue", &[IDLE]),
            command("checkevent", &[SKETCH]),
            command("iftrue", &["MnBadgeAvrinMina"]),
            command("checkevent", &["EVENT_MN_ICE_CAPTIVE_RESCUED"]),
            command("iffalse", &["MnBadgeAvrinDispatch"]),
            command("jumptext", &["MnMinneapolisGymLeaderAfterText"]),
        ]),
    );
    for (branch, label) in [
        ("MnBadgeAvrinDispatch", "MnBadgeAvrinDispatchText"),
        ("MnBadgeAvrinMina", "MnBadgeAvrinMinaText"),
    ] {
        ensure!(
            !gym.scripts.contains_key(branch)
                && !gym.scripts.contains_key(label)
                && !pack.data.map_scripts.contains_key(branch)
                && !pack.data.map_scripts.contains_key(label),
            "occupied Avrin handoff {branch}"
        );
        gym.scripts
            .insert(branch.into(), json!([command("jumptext", &[label])]));
        install_pages(&mut gym, label)?;
    }
    gym.script_flag_commands = crate::parse_script_flag_commands(GYM, &gym.scripts)?;
    gym.script_control_commands = crate::parse_script_control_commands(GYM, &gym.scripts)?;
    gym.script_text_commands = crate::parse_script_text_commands(GYM, &gym.scripts)?;
    authored_town_publication::replace(pack, gym)?;

    let name = "MnR05C00R02";
    let label = "MN_R05_WARDENText";
    let mut route = pack.data.maps.get(name).context(name)?.clone();
    let body = route.scripts.get(label).context(label)?;
    ensure!(
        body.as_array().is_some_and(
            |lines| lines.first() == Some(&command("text", &["\"For ST CLOUD, hop\""]))
        ) && pack.data.map_scripts.get(label) == Some(body),
        "unexpected bluff warden directions"
    );
    install_pages(&mut route, label)?;
    authored_town_publication::replace(pack, route)
}
