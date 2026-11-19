//! Fixed-roll cartridge damage and move PP for three naturally raised Minnesota starters.
use std::path::PathBuf;

use crystal_core::battle::damage::{DamageContext, DamageResult, calculate_damage};
use crystal_core::battle::start::materialize_trainer_party;
use crystal_core::models::{Dv, Pokemon};

use crate::{GameDataSet, read_verified_compiled_game_pack};

const LEAGUE: [(&str, usize); 5] = [
    ("MN_LEAGUE_1", 4),
    ("MN_LEAGUE_2", 4),
    ("MN_LEAGUE_3", 4),
    ("MN_LEAGUE_4", 4),
    ("MN_CHAMPION", 6),
];

struct Starter {
    family: [&'static str; 3],
    moves: [&'static str; 4],
}

const STARTERS: [Starter; 3] = [
    Starter {
        family: ["POUCHID", "SLIPPERINA", "CYPRIELLE"],
        moves: ["ENERGY_BALL", "MOONBLAST", "PSYCHIC_M", "DAZZLING_GLEAM"],
    },
    Starter {
        family: ["BIRCHIT", "PORTABEAU", "VOYAGEAR"],
        moves: ["WATERFALL", "CRUNCH", "DRAIN_PUNCH", "HEADBUTT"],
    },
    Starter {
        family: ["KINDLIT", "RIMEPELT", "BOREALYNX"],
        moves: ["FLAMETHROWER", "ICE_BEAM", "BITE", "SLASH"],
    },
];

fn make_starter(data: &GameDataSet, build: &Starter) -> Pokemon {
    for (stages, level) in build.family.windows(2).zip([16, 36]) {
        assert!(
            data.evolutions.0[stages[0]]
                .iter()
                .any(|e| e.method == "LEVEL" && e.species == stages[1] && e.level == Some(level))
        );
    }
    for name in build.moves {
        let learned_naturally =
            build
                .family
                .iter()
                .zip([1..=16, 16..=36, 36..=50])
                .any(|(&species, levels)| {
                    data.learnsets[species]
                        .iter()
                        .any(|entry| entry.1 == name && levels.contains(&entry.0))
                });
        assert!(
            learned_naturally,
            "{name} unavailable to {:?}",
            build.family
        );
        assert!(data.moves[name].power > 0);
    }
    data.create_pokemon(build.family[2], 50, Dv::from_non_hp(8, 8, 8, 8))
        .expect("materialize a level-50 starter with no stat experience")
}

fn direct_hit(data: &GameDataSet, from: &Pokemon, to: &Pokemon, name: &str) -> DamageResult {
    calculate_damage(
        from,
        to,
        &data.moves[name],
        &data.battle_stat_multipliers,
        &data.type_categories,
        &data.type_effectiveness,
        &data.weather_modifiers,
        DamageContext {
            random_roll: 236,
            ..DamageContext::default()
        },
    )
    .expect("a direct hit has an installed native damage calculation")
}

// Only ordinary first-strike damage. Count an immediate use of Earthquake on the ground.
// Exclude counters, multi-hit/per-turn scripts, charge turns and conditional power.
fn plain_incoming(effect: &str) -> bool {
    matches!(
        effect,
        "NORMAL_HIT"
            | "ALL_UP_HIT"
            | "SPECIAL_DEFENSE_DOWN_HIT"
            | "DEFENSE_DOWN_HIT"
            | "CONFUSE_HIT"
            | "LEECH_HIT"
            | "FLINCH_HIT"
            | "PARALYZE_HIT"
            | "RECOIL_HIT"
            | "ALWAYS_HIT"
            | "PRIORITY_HIT"
            | "BURN_HIT"
            | "POISON_HIT"
            | "EARTHQUAKE"
    )
}

const RESERVE_MOVES: [&str; 4] = ["IRON_HEAD", "CRUNCH", "HYDRO_PUMP", "PIKE_DRILL"];

fn iron_range_reserve(data: &GameDataSet) -> Pokemon {
    let area = &data.wild_encounters["MnR09C01R00"];
    let water = area
        .water
        .as_ref()
        .expect("Iron Range daytime Surf encounter");
    assert!(area.water_rate.is_some_and(|rate| rate > 0));
    assert!(
        water
            .day
            .iter()
            .any(|s| s.species == "PIKEMAN" && s.level == 34)
    );
    assert!(
        data.evolutions.0["PIKEMAN"]
            .iter()
            .any(|e| e.method == "LEVEL" && e.level == Some(44) && e.species == "MUSKETUNGE")
    );
    for name in RESERVE_MOVES {
        assert!(
            [("PIKEMAN", 35..=44), ("MUSKETUNGE", 44..=50)]
                .iter()
                .any(|(stage, levels)| data.learnsets[*stage]
                    .iter()
                    .any(|e| levels.contains(&e.0) && e.1 == name)),
            "reserve must gain {name} after capture by level 50"
        );
    }
    data.create_pokemon("MUSKETUNGE", 50, Dv::from_non_hp(8, 8, 8, 8))
        .expect("level-50 Iron Range reserve")
}

fn strongest_reply<'a>(
    data: &GameDataSet,
    foe: &'a Pokemon,
    target: &Pokemon,
) -> Option<(&'a str, u16)> {
    foe.moves
        .iter()
        .filter(|m| plain_incoming(&data.moves[&m.name].effect))
        .map(|m| {
            (
                m.name.as_str(),
                direct_hit(data, foe, target, &m.name).damage,
            )
        })
        .max_by_key(|&(_, damage)| damage)
}

// Spend the strongest available successful hit. When several finish, minimize overkill.
fn spend_hits(mut hp: u16, hits: &[DamageResult; 4], pp: &mut [u8; 4]) -> u16 {
    while hp > 0 {
        let slot = (0..4)
            .filter(|&i| pp[i] > 0 && hits[i].damage > 0)
            .max_by_key(|&i| {
                (
                    hits[i].damage.min(hp),
                    std::cmp::Reverse(hits[i].damage),
                    std::cmp::Reverse(i),
                )
            });
        let Some(slot) = slot else { break };
        pp[slot] -= 1;
        hp = hp.saturating_sub(hits[slot].damage);
    }
    hp
}

fn total_pp(pp: [u8; 4]) -> u16 {
    pp.into_iter().map(u16::from).sum()
}

fn audit_starter(
    data: &GameDataSet,
    course: &[(&str, Vec<Pokemon>)],
    build: &Starter,
    reserve: &Pokemon,
) {
    let starter = make_starter(data, build);
    let initial_pp = build.moves.map(|name| data.moves[name].pp);
    let mut pp = initial_pp;
    let reserve_initial = RESERVE_MOVES.map(|name| data.moves[name].pp);
    let mut reserve_pp = reserve_initial;
    let mut per_room = Vec::new();
    let mut effective = 0;
    let mut immediate_threats = Vec::new();
    let mut hardest = (0, String::new());
    let mut unfinished = Vec::new();
    let mut reserve_finishes = 0;
    for (trainer, party) in course {
        let begin_pp = total_pp(pp);
        for foe in party {
            let hits = build
                .moves
                .map(|name| direct_hit(data, &starter, foe, name));
            let best = hits.iter().map(|hit| hit.damage).max().expect("four moves");
            if best > 0 {
                let hits_needed = foe.max_hp.div_ceil(best);
                if hardest.0 < hits_needed {
                    hardest = (hits_needed, format!("{trainer}/{}", foe.species.id));
                }
            }
            effective += usize::from(
                hits.iter()
                    .any(|hit| hit.type_multiplier.numerator > hit.type_multiplier.denominator),
            );
            if let Some((name, damage)) =
                strongest_reply(data, foe, &starter).filter(|&(_, damage)| damage >= starter.max_hp)
            {
                let (_, against_reserve) =
                    strongest_reply(data, foe, reserve).expect("opponent has a direct move");
                immediate_threats.push(format!(
                    "{trainer}/{} {name} {damage}, reserve takes at most {against_reserve}/{}",
                    foe.species.id, reserve.max_hp
                ));
            }
            let hp = spend_hits(foe.max_hp, &hits, &mut pp);
            if hp > 0 {
                let reason = if best == 0 {
                    "no damaging coverage"
                } else if total_pp(pp) == 0 {
                    "all PP spent"
                } else {
                    "damaging moves spent, remaining moves immune"
                };
                unfinished.push(format!("{trainer}/{} {hp} HP: {reason}", foe.species.id));
                let reserve_hits = RESERVE_MOVES.map(|name| direct_hit(data, reserve, foe, name));
                if spend_hits(hp, &reserve_hits, &mut reserve_pp) == 0 {
                    reserve_finishes += 1;
                }
            }
        }
        per_room.push(begin_pp - total_pp(pp));
    }
    let count: usize = course.iter().map(|(_, party)| party.len()).sum();
    println!(
        "{} {:?} | room PP {:?} | PP left {:?} of {:?} | complete {}/{} | super-effective option {effective}/{} | most hits {} {} | single-hit threats for {} HP {:?} | unfinished {:?} | Route 9 reserve finishes {reserve_finishes} for {} PP",
        starter.species.id,
        build.moves,
        per_room,
        pp,
        initial_pp,
        count - unfinished.len(),
        count,
        count,
        hardest.0,
        hardest.1,
        starter.max_hp,
        immediate_threats,
        unfinished,
        total_pp(reserve_initial) - total_pp(reserve_pp)
    );
    assert_eq!(
        per_room.iter().sum::<u16>() + total_pp(pp),
        total_pp(initial_pp)
    );
    assert_eq!(count, 22);
    assert!(effective <= count && reserve_finishes <= unfinished.len());
}

#[test]
#[ignore = "requires MN_LAKE_OBSERVATIONS_PACK pointing to a verified local Minnesota pack"]
fn lake_observations_level_fifty_starter_league_resources() {
    let path =
        PathBuf::from(std::env::var_os("MN_LAKE_OBSERVATIONS_PACK").expect("external pack path"));
    let pack = read_verified_compiled_game_pack(&path).expect("verified regional pack");
    let data = pack.data();
    let course: Vec<_> = LEAGUE
        .iter()
        .map(|&(id, count)| {
            let trainer = &data.trainers.trainers[id];
            let party = materialize_trainer_party(
                trainer,
                &data.pokemon,
                &data.learnsets,
                &data.moves,
                &data.growth_rates,
            )
            .expect("real Minnesota trainer party");
            assert_eq!(party.len(), count, "{id}");
            (trainer.name.as_str(), party)
        })
        .collect();
    println!(
        "{} | level 50, four non-HP DVs 8 (derived HP DV 0), no stat experience, badge modifiers, items, accuracy or criticals; fixed successful hits, no HP attrition or battle simulation",
        path.display()
    );
    let reserve = iron_range_reserve(data);
    for build in &STARTERS {
        audit_starter(data, &course, build, &reserve);
    }
}
