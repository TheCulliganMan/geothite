//! Compare ordinary St. Cloud parties with the same partners in a different lead slot.
#[path = "saven_route_five_catch.rs"]
mod saven_route_five_catch;

use crystal_core::battle::turn::BattleAction;

use super::{attack, badge_battle_parties, challenge_with_party};
use crate::read_verified_compiled_game_pack;

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and resolves 18 St. Cloud battles"]
fn st_cloud_retries_with_an_already_raised_partner_leading() {
    let file = std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK").expect("Minnesota pack");
    let pack = read_verified_compiled_game_pack(file).expect("verified archive");
    let data = pack.data();
    for (starter, alternative) in ["CACHEEK", "CHORIP", "CACHEEK"].into_iter().enumerate() {
        let party = badge_battle_parties::party(data, starter, 5);
        assert_eq!(party.len(), 4);
        let reserve = party
            .iter()
            .position(|p| p.species.id == alternative)
            .expect("available partner before the fifth badge");
        assert_ne!(reserve, 0, "retry must change the opening");
        if starter == 1 {
            assert!(
                party[0].moves.iter().any(|m| m.name == "ICY_WIND"),
                "Rimepelt's earned Ice move"
            );
        }
        for seed in [1701, 4242, 9001] {
            for use_partner in [false, true] {
                let mut team = party.clone();
                if use_partner {
                    // Reorder outside the battle. The remaining partners keep their
                    // relative positions for ordinary forced replacements.
                    let partner = team.remove(reserve);
                    team.insert(0, partner);
                }
                let lead = team[0].species.id.clone();
                let mut moves = Vec::new();
                let finish =
                    challenge_with_party(data, 5, "MN_GYM_5", team, seed, |data, combat| {
                        let choice = attack(data, combat);
                        if seed == 1701
                            && let BattleAction::Move { slot } = &choice
                        {
                            let selected = combat
                                .player
                                .moves
                                .get(*slot)
                                .map_or("STRUGGLE", |m| m.name.as_str());
                            moves.push(format!(
                                "{}({})>{}({}):{}:last-enemy={}",
                                combat.player.species.id,
                                combat.player.hp,
                                combat.enemy.species.id,
                                combat.enemy.hp,
                                selected,
                                combat.enemy_last_move.as_deref().unwrap_or("-")
                            ));
                        }
                        choice
                    });
                assert_eq!(
                    if finish.won {
                        finish.enemy_left
                    } else {
                        finish.player_left
                    },
                    0
                );
                println!(
                    "st-cloud,{starter},{seed},{lead},{},{},{},{}",
                    finish.won, finish.turns, finish.player_left, finish.enemy_left
                );
                if !moves.is_empty() {
                    println!(
                        "st-cloud-choices,{starter},{seed},{lead},{}",
                        moves.join("|")
                    );
                }
            }
        }
    }
}
