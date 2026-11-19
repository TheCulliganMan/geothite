//! Three existing sight battles lead west to Moorhead's levee.
use anyhow::{Context, Result, ensure};
use crystal_core::models::LearnedMove;
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_regional_roster::RegionalRoster;

const FINISH: &str = "MN_MOORHEAD_LEVEE_HELPER";
const PAID: &str = "EVENT_MN_PRAIRIE_CIRCUIT_REWARDED";
const MARK: &str = "EVENT_MN_MOORHEAD_GAUGE_READ";
const BADGE: &str = "ENGINE_MINERALBADGE";
const RUNNERS: [(&str, &str, &str); 3] = [
    ("MN_R16_T01", "EVENT_MN_R16_TRAINER_01", "MnPrairieLyleText"),
    ("MN_R16_T04", "EVENT_MN_R16_TRAINER_04", "MnPrairieJuneText"),
    (
        "MN_R16_T09",
        "EVENT_MN_R16_TRAINER_09",
        "MnPrairieFinchText",
    ),
];

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let flags = &pack.data.story_event_script_constants.global;
    ensure!(
        flags.contains_key(BADGE) && flags.contains_key(MARK),
        "prairie badge and flood marker"
    );
    ensure!(pack.data.items.contains_key("NUGGET"), "native Nugget");
    for (trainer, win, _) in RUNNERS {
        let name = if trainer == "MN_R16_T09" {
            "MnR16C01R00"
        } else {
            "MnR16C00R00"
        };
        let room = pack.data.maps.get(name).context("prairie course")?;
        let binding = room
            .trainer_scripts
            .get(trainer)
            .context("prairie trainer")?;
        ensure!(
            binding.event_flag == win
                && flags.contains_key(win)
                && room.objects.iter().any(|o| o.script == trainer
                    && o.object_type == "OBJECTTYPE_TRAINER"
                    && o.radius > 0),
            "missing sight battle or win flag: {trainer}"
        );
    }
    crate::northstar_lair_authoring::flag(pack, PAID)?;
    specialize_teams(pack)?;
    install_finish(pack)?;
    for (name, labels) in [
        (
            "MnR16C00R00",
            &[
                "MN_R16_RIVER_WARDENText",
                "MN_R16_T01Seen",
                "MN_R16_T04Seen",
                "MN_R16_T04Won",
                "MN_R16_T04Lost",
                "MN_R16_T04ReturnText",
            ][..],
        ),
        (
            "MnR16C01R00",
            &["MN_R16_SIGN_SEEDText", "MN_R16_T09Seen"][..],
        ),
    ] {
        let mut room = pack.data.maps.get(name).context("Route 16")?.clone();
        for &label in labels {
            ensure!(
                room.scripts.contains_key(label),
                "missing original text {label}"
            );
            authored_dialogue::install_canonical(&mut room, label)?;
        }
        room.script_text_bodies = crate::parse_script_text_bodies(name, &room.scripts)?;
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}

fn install_finish(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnMoorhead")
        .context("Moorhead")?
        .clone();
    let original = "MN_MOORHEAD_LEVEE_HELPERText";
    ensure!(
        room.scripts.get(FINISH) == Some(&json!([c("jumptextfaceplayer", &[original])])),
        "expected the levee helper's original conversation"
    );
    for label in [
        "MnPrairieNeedLakeText",
        "MnPrairieIntroText",
        "MnPrairieLyleText",
        "MnPrairieJuneText",
        "MnPrairieFinchText",
        "MnPrairieGaugeText",
        "MnPrairiePaidText",
        "MnPrairieFullText",
    ] {
        ensure!(
            !room.scripts.contains_key(label),
            "occupied prairie text {label}"
        );
        authored_dialogue::install_canonical(&mut room, label)?;
    }

    let mut main = vec![
        c("faceplayer", &[]),
        c("opentext", &[]),
        c("checkevent", &[PAID]),
        c("iftrue", &["MnPrairieFinished"]),
        c("checkflag", &[BADGE]),
        c("iffalse", &["MnPrairieNeedLake"]),
    ];
    for (_, win, _) in RUNNERS {
        main.extend([c("checkevent", &[win]), c("iffalse", &["MnPrairieMissing"])]);
    }
    main.extend([
        c("checkevent", &[MARK]),
        c("iffalse", &["MnPrairieGauge"]),
        c("giveitem", &["NUGGET", "1"]),
        c("iffalse", &["MnPrairieFull"]),
        c("setevent", &[PAID]),
        c("playsound", &["SFX_ITEM"]),
        c("writetext", &["MnPrairiePaidText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("end", &[]),
    ]);
    room.scripts.insert(FINISH.into(), Value::Array(main));
    let mut branches = vec![(
        "MnPrairieMissing",
        vec![
            c("writetext", &["MnPrairieIntroText"]),
            c("promptbutton", &[]),
            c("sjump", &["MnPrairieCheckLyle"]),
        ],
    )];
    // Display just the remaining landmarks in one conversation, in west-to-east order.
    let stops = [
        ("MnPrairieCheckLyle", "MnPrairieCheckJune"),
        ("MnPrairieCheckJune", "MnPrairieCheckFinch"),
        ("MnPrairieCheckFinch", "MnPrairieDoneListing"),
    ];
    for ((here, next), (_, win, clue)) in stops.into_iter().zip(RUNNERS) {
        branches.push((
            here,
            vec![
                c("checkevent", &[win]),
                c("iftrue", &[next]),
                c("writetext", &[clue]),
                c("promptbutton", &[]),
                c("sjump", &[next]),
            ],
        ));
    }
    branches.push((
        "MnPrairieDoneListing",
        vec![c("closetext", &[]), c("end", &[])],
    ));
    for (branch, text) in [
        ("MnPrairieFinished", original),
        ("MnPrairieNeedLake", "MnPrairieNeedLakeText"),
        ("MnPrairieGauge", "MnPrairieGaugeText"),
        ("MnPrairieFull", "MnPrairieFullText"),
    ] {
        branches.push((
            branch,
            vec![
                c("writetext", &[text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[]),
            ],
        ));
    }
    for (label, program) in branches {
        ensure!(
            room.scripts
                .insert(label.into(), Value::Array(program))
                .is_none(),
            "occupied prairie script {label}"
        );
    }
    authoring::finish(pack, &mut room)?;
    room.script_audio_commands = crate::parse_script_audio_commands(&room.id, &room.scripts)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(&room.id, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}

fn specialize_teams(pack: &mut CompiledGamePack) -> Result<()> {
    let roster = RegionalRoster::current()?;
    let lineups = [
        (
            "MN_R16_T01",
            "LYLE",
            ["CACHEEK", "FLOSSAIL", "ZAPSAP"],
            [
                ("JIGLET", ["METAL_CLAW", "WATER_PULSE", "SWIFT", "PURSUIT"]),
                ("CACHEEK", ["DIG", "HEADBUTT", "THIEF", "SUPER_FANG"]),
                (
                    "RONDIVE",
                    ["TWINEEDLE", "QUICK_ATTACK", "BUBBLEBEAM", "RAPID_SPIN"],
                ),
            ],
        ),
        (
            "MN_R16_T09",
            "FINCH",
            ["ZAPSAP", "BOOMRUFF", "TOTLIT"],
            [
                ("TAPDUCK", ["QUICK_ATTACK", "ENCORE", "HEADBUTT", "AGILITY"]),
                (
                    "VANECHICK",
                    ["WING_ATTACK", "FAINT_ATTACK", "SCARY_FACE", "SAND_ATTACK"],
                ),
                (
                    "BOOMRUFF",
                    ["WING_ATTACK", "BELLY_DRUM", "STOMP", "MAGNITUDE"],
                ),
            ],
        ),
    ];
    for (id, name, prior, design) in lineups {
        let trainer = pack
            .data
            .trainers
            .trainers
            .get(id)
            .context("Route 16 party")?;
        // Field's Lyle has two partners. Its sibling edition added a third at
        // level 30. Each keeps its own party size, level curve and slot data.
        let (expected_species, expected_levels): (&[&str], &[u8]) =
            if id == "MN_R16_T01" && trainer.party.len() == 2 {
                (&["FLOSSAIL", "CACHEEK"], &[31, 31])
            } else {
                (&prior, &[31, 31, 30])
            };
        ensure!(
            trainer.name == name
                && trainer
                    .party
                    .iter()
                    .map(|p| p.species.as_str())
                    .eq(expected_species.iter().copied())
                && trainer
                    .party
                    .iter()
                    .map(|p| p.level)
                    .eq(expected_levels.iter().copied()),
            "unexpected prairie party: {id}"
        );
        let mut team = trainer.party.clone();
        for (member, (species, moves)) in team.iter_mut().zip(design) {
            ensure!(
                roster.target(pack, species, Some(member.level))? == species,
                "inactive or premature {species}"
            );
            let learnset = pack
                .data
                .learnsets
                .get(species)
                .context("regional learnset")?;
            member.species = species.into();
            member.moves.clear();
            for name in moves {
                ensure!(
                    learnset
                        .iter()
                        .any(|entry| entry.0 <= member.level && entry.1 == name),
                    "{species} cannot naturally learn {name} by level {}",
                    member.level
                );
                member.moves.push(LearnedMove {
                    name: name.into(),
                    current_pp: pack.data.moves.get(name).context("native move")?.pp,
                    pp_ups: 0,
                });
            }
        }
        pack.data
            .trainers
            .trainers
            .get_mut(id)
            .context("prairie trainer")?
            .party = team;
    }
    Ok(())
}
