//! Honor each shoreline observation and offer tackle to already-equipped anglers.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::install_canonical;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;
use crate::mille_lacs_shore_modpack::{SHORE_WATCH_REPORT_EVENTS, SHORE_WATCH_REWARDED_EVENT};
use crate::{CompiledGamePack, MapModule};

const SHORE: &str = "MnR18C00R00";
const PARK: &str = "MnMilleLacs";
const NEEDS_NEWS: &str = "MN_R18_WatchNeedsNews";
const HAS_ROD: &str = "MN_R18_WatchHasRod";
const PLACES: [(&str, &str); 3] = [
    ("MnShoreWatchReeds", "MnShoreWatchReedsText"),
    ("MnShoreWatchMeadow", "MnShoreWatchMeadowText"),
    ("MnShoreWatchBeach", "MnShoreWatchBeachText"),
];

/// Updates the two Shore Watch rangers using the existing volunteer reports and reward receipt.
///
/// Returns an error if the installed report or fishing reward branches differ from the source quest.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut shore = pack.data.maps.get(SHORE).context(SHORE)?.clone();
    let ranger = shore
        .scripts
        .get("MN_R18_DOCK_RANGER")
        .and_then(Value::as_array)
        .context("dock ranger")?;
    ensure!(
        SHORE_WATCH_REPORT_EVENTS
            .iter()
            .all(|event| ranger.windows(2).any(|r| r
                == [
                    command("checkevent", &[event]),
                    command("iffalse", &[NEEDS_NEWS])
                ]))
            && ["GOOD_ROD", "SUPER_ROD"].iter().all(|it| ranger
                .windows(2)
                .any(|r| r == [command("checkitem", &[it]), command("iftrue", &[HAS_ROD])]))
            && shore.scripts.contains_key("MN_R18_DOCK_RANGERText")
            && shore
                .scripts
                .get(HAS_ROD)
                .and_then(Value::as_array)
                .is_some_and(|r| r.ends_with(&[
                    command("setevent", &[SHORE_WATCH_REWARDED_EVENT]),
                    command("end", &[])
                ])),
        "unexpected Shore Watch dock ranger"
    );
    let mut ask = Vec::new();
    let mut partial = Vec::new();
    for (flag, (branch, text)) in SHORE_WATCH_REPORT_EVENTS.into_iter().zip(PLACES) {
        ask.extend([
            command("checkevent", &[flag]),
            command("iftrue", &["MnShoreWatchMissing"]),
        ]);
        partial.extend([
            command("checkevent", &[flag]),
            command("iffalse", &[branch]),
        ]);
        add(&mut shore, branch, json!([command("jumptext", &[text])]))?;
        install_canonical(&mut shore, text)?;
    }
    // An incomplete return should send the walker to one remaining volunteer.
    // A first visit explains the boat wake and all three observations.
    ask.push(command("jumptext", &["MN_R18_DOCK_RANGERText"]));
    partial.push(command("end", &[]));
    *shore.scripts.get_mut(NEEDS_NEWS).context(NEEDS_NEWS)? = ask.into();
    add(&mut shore, "MnShoreWatchMissing", partial.into())?;

    *shore.scripts.get_mut(HAS_ROD).context(HAS_ROD)? = json!([
        command("opentext", &[]),
        command("writetext", &["MN_R18_WatchRewardIntroText"]),
        command("promptbutton", &[]),
        command("giveitem", &["LURE_BALL", "3"]),
        command("iffalse", &["MnShoreWatchHoldLures"]),
        command("setevent", &[SHORE_WATCH_REWARDED_EVENT]),
        command("writetext", &["MnShoreWatchLuresText"]),
        command("waitbutton", &[]),
        command("closetext", &[]),
        command("end", &[])
    ]);
    add(
        &mut shore,
        "MnShoreWatchHoldLures",
        json!([
            command("writetext", &["MnShoreWatchLuresHeldText"]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[])
        ]),
    )?;
    for text in ["MnShoreWatchLuresText", "MnShoreWatchLuresHeldText"] {
        install_canonical(&mut shore, text)?;
    }
    shore.script_item_grants = crate::parse_script_item_grants(SHORE, &shore.scripts)?;
    publish(pack, shore)?;

    let mut park = pack.data.maps.get(PARK).context(PARK)?.clone();
    let guide = park
        .scripts
        .get_mut("MnWayShoreReport")
        .and_then(Value::as_array_mut)
        .context("park ranger")?;
    let map = command("jumptextfaceplayer", &["MnWayShoreReportText"]);
    ensure!(
        guide.last() == Some(&map)
            && guide.contains(&command("checkevent", &[SHORE_WATCH_REWARDED_EVENT])),
        "unexpected park report or return"
    );
    guide.pop();
    for flag in SHORE_WATCH_REPORT_EVENTS {
        guide.extend([
            command("checkevent", &[flag]),
            command("iffalse", &["MnShoreWatchParkLocations"]),
        ]);
    }
    guide.push(command("jumptextfaceplayer", &["MnShoreWatchReturnText"]));
    add(&mut park, "MnShoreWatchParkLocations", json!([map]))?;
    let advice = park
        .scripts
        .get_mut("MnWayShoreReportOriginal")
        .context("park casting advice")?;
    ensure!(
        *advice == json!([command("jumptextfaceplayer", &["MN_ML_PARK_RANGERText"])]),
        "unexpected park advice"
    );
    *advice = json!([command("jumptextfaceplayer", &["MnShoreWatchCastingText"])]);
    install_canonical(&mut park, "MnShoreWatchReturnText")?;
    install_canonical(&mut park, "MnShoreWatchCastingText")?;
    publish(pack, park)
}

fn add(room: &mut MapModule, name: &str, body: Value) -> Result<()> {
    ensure!(
        room.scripts.insert(name.into(), body).is_none(),
        "occupied Shore Watch label {name}"
    );
    Ok(())
}

fn publish(pack: &mut CompiledGamePack, mut room: MapModule) -> Result<()> {
    room.script_flag_commands = crate::parse_script_flag_commands(&room.id, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(&room.id, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(&room.id, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(&room.id, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result, ensure};
    use crystal_core::models::bag::{BALL_POCKET_CAPACITY, MAX_ITEM_STACK};
    use crystal_core::state::GameState;
    use crystal_core::systems::script_control::ScriptControlAction;
    use crystal_core::systems::script_items::ScriptItemGrantOutcome;
    use crystal_core::world::map::TilePosition;

    use super::{HAS_ROD, SHORE, SHORE_WATCH_REPORT_EVENTS, SHORE_WATCH_REWARDED_EVENT};
    use crate::{RuntimeMutationCommand, RuntimeMutationResult, RuntimeScriptCommandRef};

    #[test]
    #[ignore = "requires the Shore Watch archive in MN_SHORE_WATCH_PACK"]
    fn shore_watch_lure_reward_waits_for_all_three_balls_to_fit() -> Result<()> {
        let path = std::env::var_os("MN_SHORE_WATCH_PACK").context("MN_SHORE_WATCH_PACK")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let mut state = GameState::reset_wram_for_new_game();
        state.flags.set_engine_flag("ENGINE_MINERALBADGE", true)?;
        for flag in SHORE_WATCH_REPORT_EVENTS {
            state.flags.set_event_flag(flag, true)?;
        }
        let mut room = data.overworld_session(SHORE, TilePosition::new(26, 23), 0)?;
        let rod = data.items.get("SUPER_ROD").context("Super Rod")?;
        let lure = data.items.get("LURE_BALL").context("Lure Ball")?;
        let ball = data.items.get("POKE_BALL").context("Poke Ball")?;
        ensure!(state.bag.add_item(rod, 1).map_err(anyhow::Error::msg)?);
        ensure!(
            state
                .bag
                .add_item(lure, MAX_ITEM_STACK - 1)
                .map_err(anyhow::Error::msg)?
        );
        for _ in 1..BALL_POCKET_CAPACITY {
            ensure!(
                state
                    .bag
                    .add_item(ball, MAX_ITEM_STACK)
                    .map_err(anyhow::Error::msg)?
            );
        }
        let initial = state.bag.clone();
        let audio = data.audio_ids().into_iter().map(str::to_owned).collect();
        for fits in [false, true] {
            if fits {
                ensure!(state.bag.remove_item(lure, 2).map_err(anyhow::Error::msg)?);
            }
            let result = data.apply_runtime_mutation_command(
                &mut state,
                &mut room,
                RuntimeMutationCommand::GrantScriptItem(RuntimeScriptCommandRef::new(
                    SHORE, HAS_ROD, 3,
                )),
                &audio,
                &audio,
                &audio,
            )?;
            let branch =
                data.apply_script_control_command_in_session(&mut state, &room, SHORE, HAS_ROD, 4)?;
            if fits {
                ensure!(matches!(
                    result.result,
                    RuntimeMutationResult::ScriptItemGranted(ScriptItemGrantOutcome::Granted {
                        quantity: 3,
                        ..
                    })
                ));
                ensure!(matches!(branch, ScriptControlAction::Continue { .. }));
                data.apply_script_flag_mutation_in_session(
                    &mut state, &mut room, SHORE, HAS_ROD, 5,
                )?;
                ensure!(
                    state.flags.is_event_flag_set(SHORE_WATCH_REWARDED_EVENT)?
                        && state.bag.quantity(lure) == MAX_ITEM_STACK
                        && state.bag.quantity(rod) == 1
                );
            } else {
                ensure!(matches!(
                    result.result,
                    RuntimeMutationResult::ScriptItemGranted(ScriptItemGrantOutcome::BagFull {
                        quantity: 3,
                        ..
                    })
                ));
                ensure!(
                    matches!(branch,ScriptControlAction::Jump{target_script,..} if target_script=="MnShoreWatchHoldLures")
                );
                ensure!(
                    state.bag == initial
                        && !state.flags.is_event_flag_set(SHORE_WATCH_REWARDED_EVENT)?
                );
            }
        }
        Ok(())
    }
}
