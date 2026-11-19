//! Put the earned machines into the Gym battles and the regional learnsets.

use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::northstar_regional_roster::RegionalRoster;

const TIMEKEEPER: &str = "MN_GYM_1_TRAINER_1";
const TIMEKEEPER_CLASS: &str = "MN_TIMEKEEPER";
const TIMEKEEPER_SPRITE: &str = "SPRITE_MN_TIMEKEEPER";

const MACHINE_LEARNERS: &[(&str, &[&str])] = &[
    (
        "THUNDERPUNCH",
        &[
            "PIPULSE",
            "MILLIMP",
            "FLOURGEIST",
            "PUFFLARE",
            "POPUMMEL",
            "SQUEAKURD",
            "MESABLAST",
        ],
    ),
    (
        "STEEL_WING",
        &[
            "RUSTLEBEE",
            "PATCHEBEE",
            "RIVETQUEEN",
            "VANECHICK",
            "SILOHAWK",
            "BURROWBILL",
            "BANKWING",
            "SNOWPIP",
            "SQUALLFIN",
            "WHITEWHIRL",
        ],
    ),
];

// One native move slot changes on each existing member. Nibbin keeps Scratch
// for a lighter hit. Rochester's quick finisher and Minneapolis's ace keep their
// existing moves, while their leads demonstrate the prize.
const LEADER_LESSONS: &[(&str, &str, u8, &str, &str)] = &[
    ("MN_GYM_1", "NIBBIN", 10, "QUICK_ATTACK", "HEADBUTT"),
    ("MN_GYM_2", "SODCALF", 14, "TAIL_WHIP", "MUD_SLAP"),
    ("MN_GYM_3", "PIPULSE", 19, "SPARK", "THUNDERPUNCH"),
    ("MN_GYM_3", "SPARKVEIL", 20, "THUNDERBOLT", "THUNDERSHOCK"),
    ("MN_GYM_4", "PATCHEBEE", 23, "METAL_CLAW", "STEEL_WING"),
    ("MN_GYM_8", "LUTEBRISK", 42, "ICE_BEAM", "ICY_WIND"),
];

/// Completes the gym lessons on the campaign parent that predates them and preserves the
/// previously published lessons and native trainer art on its sibling.
///
/// # Errors
/// Returns an error for incomplete gym lessons, missing actors, or incompatible original teams.
pub(crate) fn complete(pack: &mut CompiledGamePack) -> Result<()> {
    let tavel = pack
        .data
        .trainers
        .trainers
        .get(TIMEKEEPER)
        .context("Rochester student")?;
    if tavel.trainer_class != TIMEKEEPER_CLASS {
        return install(pack);
    }
    let gym = pack
        .data
        .maps
        .get("MnRochesterGym")
        .context("Rochester Gym")?;
    ensure!(
        gym.objects.iter().any(|actor| actor.script == TIMEKEEPER
            && actor.sprite == TIMEKEEPER_SPRITE
            && actor.object_type == "OBJECTTYPE_TRAINER"
            && actor.radius == 2)
            && gym
                .trainer_scripts
                .get(TIMEKEEPER)
                .is_some_and(|battle| battle.trainer_class == TIMEKEEPER_CLASS),
        "published Timekeeper lost his native sight battle"
    );
    for &(machine, learners) in MACHINE_LEARNERS {
        for &learner in learners {
            let species = pack
                .data
                .pokemon
                .get(learner)
                .context("regional machine learner")?;
            ensure!(
                species.tmhm_learnset.iter().any(|name| name == machine),
                "{learner} lost {machine}"
            );
        }
    }
    for &(trainer, species, level, previous, replacement) in LEADER_LESSONS {
        let leader = pack
            .data
            .trainers
            .trainers
            .get(trainer)
            .context("published Gym leader")?;
        let member = leader
            .party
            .iter()
            .find(|m| m.species == species && m.level == level)
            .with_context(|| format!("{trainer} lost its level-{level} {species}"))?;
        ensure!(
            member.moves.iter().any(|m| m.name == replacement)
                && !member.moves.iter().any(|m| m.name == previous),
            "{trainer}'s {species} lost its {replacement} lesson"
        );
    }
    Ok(())
}

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let roster = RegionalRoster::current()?;
    for &(machine, learners) in MACHINE_LEARNERS {
        ensure!(
            pack.data.moves.contains_key(machine),
            "unknown machine {machine}"
        );
        for &species in learners {
            ensure!(
                roster.allowed.contains(species),
                "inactive learner {species}"
            );
            let profile = pack
                .data
                .pokemon
                .get_mut(species)
                .context("regional machine learner")?;
            if !profile
                .tmhm_learnset
                .iter()
                .any(|learned| learned == machine)
            {
                profile.tmhm_learnset.push(machine.to_owned());
            }
        }
    }
    for &(trainer, species, level, previous, replacement) in LEADER_LESSONS {
        ensure!(
            roster.target(pack, species, Some(level))? == species,
            "invalid Gym member {species}"
        );
        let profile = &pack.data.pokemon[species];
        let learnset = pack
            .data
            .learnsets
            .get(species)
            .context("leader learnset")?;
        ensure!(
            profile.tmhm_learnset.iter().any(|name| name == replacement)
                || learnset
                    .iter()
                    .any(|entry| entry.0 <= level && entry.1 == replacement),
            "illegal Gym lesson: {species}/{replacement} at {level}"
        );
        let pp = pack
            .data
            .moves
            .get(replacement)
            .context("native prize move")?
            .pp;
        let party = &mut pack
            .data
            .trainers
            .trainers
            .get_mut(trainer)
            .context("native Gym leader")?
            .party;
        let member = party
            .iter_mut()
            .find(|m| m.species == species && m.level == level)
            .with_context(|| format!("expected {species} at level {level} on {trainer}"))?;
        ensure!(
            !member.moves.iter().any(|m| m.name == replacement),
            "duplicate lesson on {species}"
        );
        let slot = member
            .moves
            .iter_mut()
            .find(|m| m.name == previous)
            .with_context(|| format!("missing {previous} on {species}"))?;
        *slot = LearnedMove {
            name: replacement.into(),
            current_pp: pp,
            pp_ups: 0,
        };
    }
    install_timekeeper(pack)
}

fn install_timekeeper(pack: &mut CompiledGamePack) -> Result<()> {
    let art = crate::minnesota_native_npc::compile(
        include_str!("../../../planning/minnesota/gyms/battle-lessons-v01/tavel.json"),
        None,
    )?;
    for (path, bytes) in art {
        ensure!(
            pack.runtime_files.insert(path, bytes).is_none(),
            "duplicate Tavel art"
        );
    }
    ensure!(
        pack.data
            .trainer_class_names
            .insert(TIMEKEEPER_CLASS.into(), "Timekeeper".into())
            .is_none()
            && pack
                .data
                .sprite_palette_defaults
                .insert(TIMEKEEPER_SPRITE.into(), 0)
                .is_none(),
        "duplicate Tavel class"
    );
    let trainer = pack
        .data
        .trainers
        .trainers
        .get_mut(TIMEKEEPER)
        .context("Rochester timekeeper")?;
    ensure!(
        trainer.trainer_class == "SCHOOLBOY" && trainer.name == "TAVEL",
        "expected Tavel"
    );
    trainer.trainer_class = TIMEKEEPER_CLASS.into();

    let mut room = pack
        .data
        .maps
        .get("MnRochesterGym")
        .context("Rochester Gym")?
        .clone();
    let actor = room
        .objects
        .iter_mut()
        .find(|actor| actor.script == TIMEKEEPER)
        .context("Tavel's field actor")?;
    ensure!(
        actor.sprite == "SPRITE_YOUNGSTER"
            && actor.object_type == "OBJECTTYPE_TRAINER"
            && actor.radius == 2,
        "expected on-sight student"
    );
    actor.sprite = TIMEKEEPER_SPRITE.into();
    let tile = [actor.x, actor.y];
    crate::minnesota_trainer_art::rebind_actor_event_source(
        &mut room,
        TIMEKEEPER,
        tile,
        TIMEKEEPER_SPRITE,
    )?;
    let script = room
        .scripts
        .get_mut(TIMEKEEPER)
        .context("Tavel's trainer event")?;
    ensure!(
        script[0]["command"] == "trainer"
            && script[0]["args"][0] == "SCHOOLBOY"
            && script[0]["args"][1] == TIMEKEEPER,
        "expected Tavel's sight battle"
    );
    script[0]["args"][0] = TIMEKEEPER_CLASS.into();
    room.trainer_scripts = crate::parse_trainer_scripts(&room.id, &room.scripts)?;
    authoring::refresh_event_section(&mut room);
    crate::authored_town_publication::replace(pack, room)
}
