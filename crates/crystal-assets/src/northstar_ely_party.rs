//! Halden introduces a Northwoods aquatic forager before Casey's signature pike.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::command;
use crate::northstar_battle_balance::{AuthoredMember, build_member};
use crate::northstar_regional_roster::RegionalRoster;
use crate::northstar_rival_arc;

const ROOM: &str = "MnLeagueRoom4";
const READY: &str = "MnHaldenReady";
const ORIGINAL: &str = "MnLeagueRoom4BattleStart";
const FIRST: &str = "MnHaldenFirstBattle";
const FIRST_TRAINER: &str = "MN_HALDEN_FIRST";
const RETURN_TRAINER: &str = "MN_LEAGUE_4";

pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = northstar_rival_arc::map(pack, ROOM)?;
    ensure!(
        !room.scripts.contains_key(FIRST) && !pack.data.map_scripts.contains_key(FIRST),
        "occupied first-challenge battle label"
    );
    let returning = pack.data.trainers.get(RETURN_TRAINER).context("Halden")?;
    let mut first = returning.clone();
    first.trainer_id = FIRST_TRAINER.into();
    let old = first.party.get_mut(1).context("Halden's second partner")?;
    ensure!(
        old.species == "MUSKETUNGE" && old.level == 50,
        "unexpected Halden partner"
    );
    *old = build_member(
        pack,
        &RegionalRoster::current()?,
        &AuthoredMember {
            species: "MARSHRACK".into(),
            level: old.level,
            moves: ["MEGA_DRAIN", "BUBBLEBEAM", "STOMP", "GROWTH"]
                .map(String::from)
                .into(),
            item: old.item.clone(),
        },
        old.dvs,
    )?;
    // Retain the original program and trainer for returning Champions and
    // copy its victory, defeat and door choreography for the first battle.
    let battle = room
        .scripted_trainer_battles
        .iter()
        .find(|b| b.source_script == ORIGINAL && b.request.trainer_id == RETURN_TRAINER)
        .context("installed Halden battle")?;
    let mut program = room
        .scripts
        .get(ORIGINAL)
        .context("Halden battle program")?
        .clone();
    let load = program
        .get_mut(battle.loadtrainer_command_index)
        .context("Halden trainer load")?;
    ensure!(
        *load == command("loadtrainer", &[&returning.trainer_class, RETURN_TRAINER]),
        "unexpected returning trainer"
    );
    *load = command("loadtrainer", &[&first.trainer_class, FIRST_TRAINER]);
    let jump = room
        .scripts
        .get_mut(READY)
        .and_then(Value::as_array_mut)
        .and_then(|commands| commands.last_mut())
        .context("Halden's first-challenge consent")?;
    ensure!(
        *jump == command("sjump", &[ORIGINAL]),
        "unexpected Halden invitation target"
    );
    *jump = command("sjump", &[FIRST]);
    room.scripts.insert(FIRST.into(), program);
    pack.data.trainers.insert(first)?;

    // The installed run-counter records are in publication order. Reindex the
    // new battle while retaining that order for all three existing records.
    let mut variables = std::mem::take(&mut room.script_variable_commands);
    crate::ely_league_rooms::finish(pack, &mut room)?;
    let mut first_variables = room
        .script_variable_commands
        .into_iter()
        .filter(|row| row.source_script == FIRST);
    let grant = first_variables.next().context("first battle run counter")?;
    ensure!(
        first_variables.next().is_none()
            && grant.command == "loadmem"
            && grant.target.as_deref() == Some("wMnLeagueRun")
            && grant.value_tokens == ["4"],
        "unexpected first battle run counter"
    );
    variables.push(grant);
    room.script_variable_commands = variables;
    room.script_audio_commands = crate::parse_script_audio_commands(ROOM, &room.scripts)?;
    crate::authored_town_publication::replace(pack, room)
}
