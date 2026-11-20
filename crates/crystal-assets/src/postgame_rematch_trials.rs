//! The same postgame challenger accepts each installed offer independently.
use crystal_core::battle::turn::BattleCombatState;
use crystal_core::models::Pokemon;
use crystal_core::state::BattleMemory;
use crystal_core::systems::battle_rewards::apply_experience_level_ups;
use crystal_core::systems::experience::calculate_experience;
use crystal_core::systems::script_flags::apply_script_flag_mutation;

use super::{TrialDivider, attack, challenge_state, ely_challenger_parties, trial_state};
use crate::northstar_regional_roster::RegionalRoster;
use crate::{GameDataSet, read_verified_compiled_game_pack};

const MATCHES: [(&str, &str, &str, u8, u8); 9] = [
    ("MnRochesterGym", "MN_GYM_1_REMATCH", "CROOKVEIL", 55, 57),
    ("MnMankatoGym", "MN_GYM_2_REMATCH", "THUNDRAZE", 55, 57),
    ("MnMinneapolisGym", "MN_GYM_3_REMATCH", "BEETYNAMO", 55, 57),
    ("MnStPaulGym", "MN_GYM_4_REMATCH", "LATCHQUILL", 55, 57),
    ("MnStCloudGym", "MN_GYM_5_REMATCH", "SILOHAWK", 56, 58),
    ("MnBrainerdGym", "MN_GYM_6_REMATCH", "PADDARRAY", 56, 58),
    ("MnBemidjiGym", "MN_GYM_7_REMATCH", "CYPRIELLE", 56, 58),
    ("MnDuluthGym", "MN_GYM_8_REMATCH", "VERNACAPE", 58, 60),
    (
        "MnStCloudStation",
        "MN_REED_REFINEMENT",
        "MOSQUITAN",
        57,
        59,
    ),
];

fn challenger(data: &GameDataSet) -> Vec<Pokemon> {
    let mut team = ely_challenger_parties::party(data, 1);
    for member in &mut team {
        // A controlled postgame experience threshold. Keep this partner's four
        // acquired moves, declining later offers and optional evolutions.
        assert_eq!(member.moves.len(), 4);
        member.experience =
            calculate_experience(&data.growth_rates, &member.species.growth_rate, 57)
                .expect("level 57 on this partner's own growth curve");
        let gained = apply_experience_level_ups(
            member,
            &data.moves,
            &data.learnsets,
            &data.growth_rates,
            &data.battle_reward_rules,
        )
        .expect("native level and stat gains");
        assert_eq!(gained.level_after, 57);
        assert!(gained.learned_moves.is_empty());
        assert_eq!(member.hp, member.max_hp);
    }
    team
}

fn first_healthy(_: &GameDataSet, combat: &BattleCombatState) -> usize {
    combat
        .player_party
        .iter()
        .position(|p| p.hp > 0)
        .expect("healthy reserve")
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves nine postgame battles"]
fn postgame_rematches_field_the_announced_parties_and_finish_all_nine_battles() {
    let path =
        std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK").expect("Minnesota review pack");
    let pack = read_verified_compiled_game_pack(path).expect("verified archive");
    let data = pack.data();
    let roster = RegionalRoster::current()
        .expect("active regional species")
        .allowed;
    let team = challenger(data);
    assert!(team.iter().all(|p| roster.contains(&p.species.id)));
    let mut champion = trial_state(data, 9, team);
    for (map, script, flag) in [
        (
            "MnBoundaryIntake",
            "MnVerdigrimBypass",
            "EVENT_MN_SURVEY_RESTORED",
        ),
        (
            "MnLeagueHallOfFame",
            "MnElyRecordVictory",
            "EVENT_MN_LEAGUE_CHAMPION",
        ),
    ] {
        let grant = data.maps[map]
            .script_flag_commands
            .iter()
            .find(|c| c.source_script == script && c.command == "setevent" && c.flag_id == flag)
            .expect("story grant in the installed edition");
        apply_script_flag_mutation(&mut champion, grant.clone())
            .expect("champion's completed story");
    }
    for (map, id, ace, low, high) in MATCHES {
        let room = &data.maps[map];
        let reed = id == "MN_REED_REFINEMENT";
        let (offer, count_text, size) = if reed {
            ("MN_ARC_AFTERMATH_REED".to_owned(), "Six partners.", 6)
        } else {
            (format!("{map}RematchOffer"), "Five partners.", 5)
        };
        let mut entries = room
            .scripted_trainer_battles
            .iter()
            .filter(|e| e.request.trainer_id == id);
        let entry = entries.next().expect("advertised opponent has a battle");
        assert!(entries.next().is_none(), "ambiguous rematch in {map}");
        assert_eq!(entry.source_script, offer);
        let source = room.scripts[&offer].as_array().expect("battle invitation");
        let load = entry.loadtrainer_command_index;
        let start = entry.startbattle_command_index;
        assert_eq!(source[load]["args"][1], id);
        assert_eq!(source[start]["command"], "startbattle");
        assert_eq!(source[start + 1]["command"], "reloadmapafterbattle");
        let yes = source[..load]
            .iter()
            .position(|c| c["command"] == "yesorno")
            .expect("explicit choice");
        assert_eq!(source[yes + 1]["command"], "iffalse");
        let declined = source[yes + 1]["args"][0]
            .as_str()
            .expect("decline destination");
        assert!(
            room.scripts[declined]
                .as_array()
                .expect("decline response")
                .iter()
                .all(|c| !matches!(
                    c["command"].as_str(),
                    Some("loadtrainer" | "startbattle" | "setevent")
                ))
        );
        let announcement = room
            .script_text_commands
            .iter()
            .find(|c| c.source_script == offer && c.command == "writetext" && c.command_index < yes)
            .and_then(|c| c.text_label.as_ref())
            .expect("spoken invitation");
        let words = room.script_text_bodies[announcement]
            .commands
            .iter()
            .flat_map(|c| &c.args)
            .map(|s| s.trim_matches('"'))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            words.contains(count_text) && words.contains(&format!("LV {low}-{high}.")),
            "{map}: {words}"
        );

        let trainer = &data.trainers.trainers[id];
        let mut state = champion.clone();
        let mut divider = TrialDivider(1701);
        let mut seen = false;
        let result = challenge_state(
            data,
            &mut state,
            id,
            &mut divider,
            |data, live, combat| {
                if !seen {
                    assert!(
                        matches!(&live.battle, BattleMemory::Trainer { trainer_id, trainer_class, source_script, .. }
                    if trainer_id == id && trainer_class == &trainer.trainer_class && source_script == &offer)
                    );
                    assert_eq!(combat.enemy_party.len(), size);
                    assert_eq!(combat.enemy_party.len(), trainer.party.len());
                    let weakest = combat.enemy_party.iter().map(|p| p.level).min();
                    let strongest = combat
                        .enemy_party
                        .iter()
                        .max_by_key(|p| p.level)
                        .expect("ace");
                    assert_eq!(
                        (weakest, strongest.level, strongest.species.id.as_str()),
                        (Some(low), high, ace)
                    );
                    for (actual, expected) in combat.enemy_party.iter().zip(&trainer.party) {
                        assert!(roster.contains(&actual.species.id));
                        assert_eq!(
                            (
                                &actual.species.id,
                                actual.level,
                                &actual.item,
                                &actual.moves
                            ),
                            (
                                &expected.species,
                                expected.level,
                                &expected.item,
                                &expected.moves
                            )
                        );
                    }
                    seen = true;
                }
                attack(data, combat)
            },
            Some(first_healthy),
            false,
        );
        assert!(seen);
        assert_eq!(result.player_left > 0, result.won);
        if result.won {
            assert_eq!(result.enemy_left, 0);
        }
        let terminal = data
            .complete_scripted_trainer_battle(
                &mut state,
                map,
                map,
                &offer,
                start,
                result.won,
                false,
                &mut divider,
            )
            .expect("normal win/loss dispatch");
        assert_eq!(terminal.continued_after_battle, result.won);
        assert_eq!(terminal.prize_money > 0, result.won);
        println!(
            "postgame,{map},{id},1701,{},{},{},{},{}",
            result.won, result.turns, result.player_left, result.enemy_left, terminal.prize_money
        );
    }
}
