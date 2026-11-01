//! Reject source story promises that differ from the actual prepared Q07 module.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use serde_json::Value;

use crate::boundary_basecamp_story::{BOUNDARY_SURVEY_CHIEF, BOUNDARY_SURVEY_RECEIVER};
use crate::{CompiledGamePack, MapModule};

/// Compares the source Q07 contract with the caller's prepared native shelter.
///
/// Invoke after `prepare_boundary_basecamp_story` and before publishing either
/// module. The existing native descriptor, trainer, flag-command tables and
/// receiver event provide the authority; this function adds no script/catalog,
/// progression state or map mutation. Current matching inputs produce exactly
/// the same compiled pack bytes as before this validation was installed.
///
/// # Errors
/// Returns an error for a changed source chief battle/party/prerequisite/win
/// event, receiver prerequisite/result, source/native actor/receiver/arrival
/// anchor or Q07 identity. Missing or ambiguous prepared native bindings also
/// return an error. All inputs remain unchanged on success and failure.
pub(super) fn validate(
    source: &Value,
    prepared: &CompiledGamePack,
    shelter: &MapModule,
) -> Result<()> {
    let rooms = source["interiors"]
        .as_array()
        .context("Q07 source interiors")?;
    let rooms = rooms
        .iter()
        .filter(|room| room["id"].as_str() == Some(shelter.id.as_str()))
        .collect::<Vec<_>>();
    ensure!(
        rooms.len() == 1,
        "Q07 source shelter anchor differs from prepared native room"
    );
    let room = rooms[0];
    ensure!(
        room["native_constant"].as_str() == shelter.attributes.map_constant.as_deref(),
        "Q07 source shelter constant differs from prepared native room"
    );
    let residents = room["residents"]
        .as_array()
        .context("Q07 source residents")?;
    let chiefs = residents
        .iter()
        .filter(|actor| actor["id"] == BOUNDARY_SURVEY_CHIEF)
        .collect::<Vec<_>>();
    ensure!(
        chiefs.len() == 1,
        "Q07 source chief anchor is missing or ambiguous"
    );
    let chief = chiefs[0];
    let objects = shelter
        .objects
        .iter()
        .filter(|actor| actor.script == BOUNDARY_SURVEY_CHIEF)
        .collect::<Vec<_>>();
    ensure!(
        objects.len() == 1,
        "Q07 prepared native chief is missing or ambiguous"
    );
    let native_chief = objects[0];
    ensure!(
        chief["tile"] == serde_json::json!([native_chief.x, native_chief.y])
            && chief["object_identifier"].as_str() == native_chief.object_identifier.as_deref()
            && chief["script"] == native_chief.script,
        "Q07 source chief actor anchor differs from prepared native body"
    );
    let battles = shelter
        .scripted_trainer_battles
        .iter()
        .filter(|battle| battle.source_script == native_chief.script)
        .collect::<Vec<_>>();
    ensure!(
        battles.len() == 1,
        "Q07 prepared native chief battle is missing or ambiguous"
    );
    let battle = battles[0];
    ensure!(
        chief["battle"].as_str() == Some(battle.request.trainer_id.as_str()),
        "Q07 source chief battle differs from actual prepared native trainer"
    );
    let trainer = prepared
        .data()
        .trainers
        .trainers
        .get(&battle.request.trainer_id)
        .context("Q07 prepared native chief party")?;
    let party = chief["party"]
        .as_array()
        .context("Q07 source chief party")?;
    ensure!(
        party.len() == trainer.party.len(),
        "Q07 source chief party differs from actual native party"
    );
    for (source_member, native) in party.iter().zip(&trainer.party) {
        let fields = source_member
            .as_object()
            .context("Q07 source chief party member")?;
        ensure!(
            fields.len() == 2 && fields.contains_key("species") && fields.contains_key("level"),
            "Q07 source chief party declares unsupported fields beyond species/level"
        );
        ensure!(
            source_member["species"].as_str() == Some(native.species.as_str())
                && source_member["level"].as_u64() == Some(u64::from(native.level)),
            "Q07 source chief party differs from actual native species/level"
        );
    }
    let flag_commands = &shelter.script_flag_commands;
    let chief_wins = flag_commands
        .iter()
        .filter(|command| {
            command.source_script == native_chief.script && command.command == "setevent"
        })
        .collect::<Vec<_>>();
    ensure!(
        chief_wins.len() == 1,
        "Q07 prepared native chief needs one victory event"
    );
    let victory = chief_wins[0].flag_id.as_str();
    ensure!(
        chief["win_flag"].as_str() == Some(victory),
        "Q07 source chief win_flag differs from actual native victory event"
    );
    let badges = flag_commands
        .iter()
        .filter(|command| {
            command.source_script == native_chief.script && command.command == "checkflag"
        })
        .collect::<Vec<_>>();
    let native_badges = badges
        .iter()
        .map(|command| command.flag_id.as_str())
        .collect::<BTreeSet<_>>();
    let johto_badges = BADGE_ENGINE_FLAGS[..8]
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    ensure!(
        badges.len() == 8 && native_badges == johto_badges,
        "Q07 prepared native chief must check all eight distinct Johto badges"
    );
    // Translate the existing core badge catalog to the dossier's ordinal
    // reservations; do not maintain a second list of native badge identities.
    let mut required = (1..=johto_badges.len())
        .map(|index| format!("BADGE_{index}"))
        .collect::<BTreeSet<_>>();
    required.extend(
        flag_commands
            .iter()
            .filter(|command| {
                command.source_script == native_chief.script
                    && command.command == "checkevent"
                    && command.flag_id != victory
            })
            .map(|command| command.flag_id.clone()),
    );
    ensure!(
        string_set(&chief["requires"], "chief prerequisites")? == required,
        "Q07 source chief prerequisites differ from actual native badge/event guards"
    );

    let receiver = &room["receiver_contract"];
    let receivers = shelter
        .events
        .bg_events
        .iter()
        .filter(|event| event.script == BOUNDARY_SURVEY_RECEIVER)
        .collect::<Vec<_>>();
    ensure!(
        receivers.len() == 1,
        "Q07 prepared native receiver is missing or ambiguous"
    );
    let native_receiver = receivers[0];
    ensure!(
        receiver["id"] == native_receiver.script
            && receiver["tile"] == serde_json::json!([native_receiver.x, native_receiver.y]),
        "Q07 source receiver anchor differs from actual native read event"
    );
    let receiver_sets = flag_commands
        .iter()
        .filter(|command| {
            command.source_script == native_receiver.script && command.command == "setevent"
        })
        .map(|command| command.flag_id.clone())
        .collect::<BTreeSet<_>>();
    ensure!(
        receiver_sets.len() == 1,
        "Q07 prepared native receiver needs one restoration event"
    );
    let receiver_requires = flag_commands
        .iter()
        .filter(|command| {
            command.source_script == native_receiver.script
                && command.command == "checkevent"
                && !receiver_sets.contains(&command.flag_id)
        })
        .map(|command| command.flag_id.clone())
        .collect::<BTreeSet<_>>();
    ensure!(
        receiver_requires == BTreeSet::from([victory.to_owned()])
            && string_set(&receiver["requires"], "receiver prerequisites")? == receiver_requires,
        "Q07 source receiver prerequisites differ from actual native chief-victory guard"
    );
    ensure!(
        string_set(&receiver["sets"], "receiver result flags")? == receiver_sets,
        "Q07 source receiver sets differ from actual native restoration event"
    );
    let approach = point(&receiver["approach"])?;
    ensure!(
        approach[0].abs_diff(native_receiver.x) + approach[1].abs_diff(native_receiver.y) == 1
            && crate::authored_route_modpack::collision(
                prepared,
                &shelter.blocks,
                shelter.attributes.width,
                approach,
                &shelter.attributes.tileset_name
            )? == "FLOOR"
            && !shelter
                .objects
                .iter()
                .any(|actor| [actor.x, actor.y] == approach),
        "Q07 source receiver approach differs from actual adjacent clear native FLOOR"
    );
    let metatile = [native_receiver.x / 2, native_receiver.y / 2];
    let block = *shelter
        .blocks
        .get(
            usize::from(metatile[1]) * usize::from(shelter.attributes.width)
                + usize::from(metatile[0]),
        )
        .context("Q07 actual native receiver block")?;
    let source_block = u16::from_str_radix(
        receiver["source_block"]
            .as_str()
            .context("Q07 source receiver block")?,
        16,
    )?;
    ensure!(
        receiver["source_metatile"] == serde_json::json!(metatile) && source_block == block,
        "Q07 source receiver metatile differs from actual native machine"
    );
    let kind = crate::authored_route_modpack::collision(
        prepared,
        &shelter.blocks,
        shelter.attributes.width,
        [native_receiver.x, native_receiver.y],
        &shelter.attributes.tileset_name,
    )?;
    ensure!(
        receiver["collision"].as_str() == Some(kind),
        "Q07 source receiver collision differs from actual native machine"
    );
    let arrival = point(&room["clear_arrival"])?;
    let arrival_index = u16::try_from(
        room["passive_arrival"]["index"]
            .as_u64()
            .context("Q07 source passive index")?,
    )?;
    let passive = shelter
        .events
        .warps
        .iter()
        .find(|warp| warp.index == arrival_index)
        .context("Q07 actual passive arrival record")?;
    ensure!(
        [passive.x, passive.y] == arrival
            && room["passive_arrival"]["tile"] == serde_json::json!(arrival),
        "Q07 source arrival anchor differs from actual prepared native record"
    );
    let q07 = &source["story_contract"]["Q07"];
    ensure!(
        q07["shelter"] == shelter.id
            && q07["chief_actor"] == native_chief.script
            && q07["battle"] == battle.request.trainer_id
            && q07["MS11_sets"].as_str() == Some(victory)
            && q07["MS12_sets"]
                .as_str()
                .is_some_and(|flag| receiver_sets.contains(flag))
            && q07["free_access"] == true
            && q07["paid_state_required"] == false,
        "Q07 source story anchors differ from actual free native chief/receiver bindings"
    );
    Ok(())
}

fn string_set(value: &Value, description: &str) -> Result<BTreeSet<String>> {
    let values = value
        .as_array()
        .with_context(|| format!("Q07 source {description} array"))?;
    let result = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .with_context(|| format!("Q07 source {description} flag"))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(
        result.len() == values.len(),
        "Q07 source {description} contain duplicate flags"
    );
    Ok(result)
}
fn point(value: &Value) -> Result<[u16; 2]> {
    Ok([
        u16::try_from(value[0].as_u64().context("Q07 source anchor x")?)?,
        u16::try_from(value[1].as_u64().context("Q07 source anchor y")?)?,
    ])
}
