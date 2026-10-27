//! Review authored progression against the supplied pack's actual move mechanics.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_assets::GameDataSet;
use crystal_core::battle::damage::{MoveCategory, is_physical_move};
use crystal_core::models::{BaseStats, Move};
use serde::Deserialize;

#[path = "audit_pokemon_balance/combat.rs"]
mod combat;
#[path = "audit_pokemon_balance/policy.rs"]
mod policy;

#[derive(Deserialize)]
struct Roster {
    families: Vec<Family>,
}

#[derive(Deserialize)]
struct Family {
    id: String,
    starter_role: Option<String>,
    rarity: Option<String>,
    classification: Option<String>,
    members: Vec<Member>,
}

#[derive(Deserialize)]
struct Member {
    id: String,
    name: String,
    types: Vec<String>,
    evolves_to: Option<String>,
}

#[derive(Deserialize)]
struct Profiles {
    species: Vec<Profile>,
}

#[derive(Deserialize)]
struct Profile {
    id: String,
    base_stats: BaseStats,
    level_moves: Vec<(u8, String)>,
    tmhm_moves: Vec<String>,
    egg_moves: Vec<String>,
    growth_rate: String,
    evolution: Option<Evolution>,
}

#[derive(Deserialize)]
struct Evolution {
    level: u8,
}

#[derive(Deserialize)]
struct Curation {
    source_species_count: usize,
    active_species_count: usize,
    active_family_count: usize,
    retired_families: Vec<RetiredFamily>,
}

#[derive(Deserialize)]
struct RetiredFamily {
    id: String,
}

#[derive(Deserialize)]
struct Extensions {
    moves: Vec<Extension>,
}

#[derive(Deserialize)]
struct Extension {
    id: String,
    battle: Move,
    category: MoveCategory,
    learners: Vec<Learner>,
}

#[derive(Deserialize)]
struct Learner {
    species_id: String,
    runtime_name: String,
    national_number: Option<u16>,
    level: u8,
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(&fs::read(path).with_context(|| path.display().to_string())?)
        .with_context(|| path.display().to_string())
}

fn total(stats: &BaseStats) -> u16 {
    stats.hp
        + stats.attack
        + stats.defense
        + stats.speed
        + stats.special_attack
        + stats.special_defense
}

// Fixed damage, retaliation, variable multihit and charge/recharge moves are not a
// dependable everyday attack. They remain useful choices outside this floor.
fn reliable(mov: &Move) -> bool {
    mov.power >= 10
        && mov.accuracy >= 80
        && ![
            "OHKO",
            "STATIC_DAMAGE",
            "LEVEL_DAMAGE",
            "SUPER_FANG",
            "COUNTER",
            "MIRROR_COAT",
            "REVERSAL",
            "HIDDEN_POWER",
            "ROLLOUT",
            "FURY_CUTTER",
            "MULTI_HIT",
            "SKULL_BASH",
            "CHARGE",
            "SKY_ATTACK",
            "SOLARBEAM",
            "HYPER_BEAM",
            "EXPLOSION",
            "SELFDESTRUCT",
            "BIDE",
            "MAGNITUDE",
            "PRESENT",
            "RETURN",
            "FRUSTRATION",
        ]
        .contains(&mov.effect.as_str())
}

fn power(mov: &Move) -> u16 {
    match mov.effect.as_str() {
        "DOUBLE_HIT" | "POISON_MULTI_HIT" => mov.power * 2,
        _ => mov.power,
    }
}

fn type_token(kind: &str) -> &str {
    if kind == "PSYCHIC" {
        "PSYCHIC_TYPE"
    } else {
        kind
    }
}

fn record_earned(
    moves: &mut BTreeMap<String, u8>,
    name: &str,
    level: u8,
    arrival: u8,
    first_stage: bool,
) {
    if first_stage || level >= arrival {
        moves
            .entry(name.to_string())
            .and_modify(|old| *old = (*old).min(level))
            .or_insert(level);
    }
}

fn attack_ratio(data: &GameDataSet, stats: &BaseStats, mov: &Move) -> f64 {
    let attack = if is_physical_move(&data.type_categories, mov)
        .expect("verified pack supplies every move category")
    {
        stats.attack
    } else {
        stats.special_attack
    };
    f64::from(attack) / f64::from(stats.attack.max(stats.special_attack))
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 2 || args.len() == 3,
        "usage: audit_pokemon_balance PACK POKEMON_ROOT [--check|--check-pack|--mechanics]"
    );
    ensure!(
        args.len() == 2 || ["--check", "--check-pack", "--mechanics"].contains(&args[2].as_str()),
        "unknown option"
    );
    let pack = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let data = pack.data();
    if args.get(2).is_some_and(|arg| arg == "--mechanics") {
        for mov in data.moves.values().filter(|mov| {
            [
                "CONFUSION",
                "PSYBEAM",
                "FURY_CUTTER",
                "MAGNITUDE",
                "LICK",
                "BITE",
                "SHADOW_BALL",
                "FIRE_PUNCH",
                "WATERFALL",
                "ICE_PUNCH",
                "RAZOR_LEAF",
                "LEECH_LIFE",
                "PIN_MISSILE",
                "DOUBLE_KICK",
                "LOW_KICK",
                "SLAM",
                "ROCK_SLIDE",
                "IRON_TAIL",
                "SLUDGE_BOMB",
                "FLAMETHROWER",
                "SWIFT",
                "HIDDEN_POWER",
                "TWINEEDLE",
            ]
            .contains(&mov.name.as_str())
        }) {
            println!(
                "{} {} power={} accuracy={} effect={} physical={:?}",
                mov.name,
                mov.move_type,
                mov.power,
                mov.accuracy,
                mov.effect,
                is_physical_move(&data.type_categories, mov)
            );
        }
        return Ok(());
    }
    let root = Path::new(&args[1]);
    let policy = policy::Policy::load(root, data)?;
    let roster: Roster = read(&root.join("roster.json"))?;
    let profiles: Profiles = read(&root.join("species-profiles.json"))?;
    let source_count = profiles.species.len();
    let profiles: BTreeMap<_, _> = profiles
        .species
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect();
    ensure!(source_count == profiles.len(), "duplicate profile IDs");
    let roster_ids: BTreeSet<_> = roster
        .families
        .iter()
        .flat_map(|f| &f.members)
        .map(|m| m.id.clone())
        .collect();
    ensure!(
        roster_ids == profiles.keys().cloned().collect(),
        "roster/profile coverage differs"
    );
    ensure!(
        roster_ids.len()
            == roster
                .families
                .iter()
                .map(|f| f.members.len())
                .sum::<usize>(),
        "duplicate roster species IDs"
    );
    for family in &roster.families {
        ensure!(
            (1..=3).contains(&family.members.len()),
            "invalid family size"
        );
        for (stage, member) in family.members.iter().enumerate() {
            ensure!(
                (1..=2).contains(&member.types.len()),
                "invalid species types"
            );
            ensure!(
                member.evolves_to.as_ref() == family.members.get(stage + 1).map(|m| &m.id),
                "family evolution order differs"
            );
            let profile = &profiles[&member.id];
            ensure!(
                profile
                    .level_moves
                    .windows(2)
                    .all(|pair| pair[0].0 <= pair[1].0)
                    && profile
                        .level_moves
                        .iter()
                        .all(|(level, _)| (1..=100).contains(level)),
                "{}: learnset must be ordered in levels 1–100",
                member.id
            );
            ensure!(
                profile
                    .evolution
                    .as_ref()
                    .is_none_or(|e| (2..=100).contains(&e.level)),
                "invalid level evolution"
            );
        }
    }
    let curation: Curation = read(&root.join("roster-curation.json"))?;
    ensure!(
        curation.source_species_count == source_count,
        "stale source species count"
    );
    let expected_active_species = curation.active_species_count;
    let expected_active_families = curation.active_family_count;
    let retired: BTreeSet<_> = curation
        .retired_families
        .into_iter()
        .map(|f| f.id)
        .collect();
    let active_names: BTreeSet<_> = roster
        .families
        .iter()
        .filter(|f| !retired.contains(&f.id))
        .flat_map(|f| &f.members)
        .map(|m| m.name.to_ascii_uppercase())
        .collect();
    ensure!(
        active_names.len() == expected_active_species
            && roster
                .families
                .iter()
                .filter(|f| !retired.contains(&f.id))
                .count()
                == expected_active_families,
        "stale active curation counts or duplicate runtime names"
    );
    if args.get(2).is_some_and(|arg| arg == "--check-pack") {
        let published: BTreeSet<_> = data
            .pokemon
            .values()
            .filter(|p| p.int_id > 251)
            .map(|p| p.id.clone())
            .collect();
        ensure!(
            published == active_names,
            "published regional roster differs from active curation"
        );
    }
    let mut extensions: Extensions = read(&root.join("northwoods-moves.json"))?;
    let fairy_path = root.join("fairy-moves.json");
    if fairy_path.try_exists()? {
        let fairy: Extensions = read(&fairy_path)?;
        extensions.moves.extend(fairy.moves);
    }
    let names: BTreeMap<_, _> = roster
        .families
        .iter()
        .flat_map(|f| &f.members)
        .map(|m| (m.id.as_str(), m.name.to_ascii_uppercase()))
        .collect();
    for extension in &extensions.moves {
        ensure!(
            data.moves.get(&extension.id) == Some(&extension.battle)
                && data.type_categories.moves.get(&extension.id) == Some(&extension.category),
            "reference pack has stale mechanics for {}",
            extension.id
        );
        let mut seen = BTreeSet::new();
        for learner in &extension.learners {
            let identity_matches = if let Some(number) = learner.national_number {
                (1..=251).contains(&number)
                    && learner.species_id == format!("NAT_{number:03}")
                    && data
                        .pokemon
                        .iter()
                        .find(|(_, species)| species.int_id == number)
                        .is_some_and(|(runtime_id, _)| {
                            data.learnsets.get(runtime_id).is_some_and(|moves| {
                                moves.iter().any(|entry| {
                                    entry.0 == learner.level && entry.1 == extension.id
                                })
                            })
                        })
            } else {
                names.get(learner.species_id.as_str()) == Some(&learner.runtime_name)
            };
            ensure!(
                seen.insert(&learner.species_id)
                    && identity_matches
                    && (1..=100).contains(&learner.level),
                "invalid learner identity/level for {}: {}",
                extension.id,
                learner.species_id
            );
        }
    }
    let mut combat = combat::CombatReview::new(data, &roster, &profiles, &extensions)?;
    let mut count = 0;
    let mut issues = 0;
    for family in roster.families.iter().filter(|f| !retired.contains(&f.id)) {
        let mut inherited = BTreeMap::<String, u8>::new();
        let mut arrival = 1;
        for (stage, member) in family.members.iter().enumerate() {
            count += 1;
            let profile = profiles.get(&member.id).context("missing profile")?;
            if args.get(2).is_some_and(|arg| arg == "--check-pack") {
                combat.verify_published(data, member)?;
            }
            for name in profile
                .level_moves
                .iter()
                .map(|(_, name)| name)
                .chain(&profile.tmhm_moves)
                .chain(&profile.egg_moves)
            {
                ensure!(
                    data.moves.contains_key(name),
                    "{}: unknown move {name}",
                    member.id
                );
            }
            let mut moves = inherited.clone();
            for (level, name) in &profile.level_moves {
                // Evolution cannot retroactively learn a level-one reminder move.
                record_earned(&mut moves, name, *level, arrival, stage == 0);
            }
            for extension in &extensions.moves {
                for learner in extension
                    .learners
                    .iter()
                    .filter(|l| l.species_id == member.id)
                {
                    ensure!(
                        data.moves.contains_key(&extension.id),
                        "pack lacks extension {}",
                        extension.id
                    );
                    record_earned(
                        &mut moves,
                        &extension.id,
                        learner.level,
                        arrival,
                        stage == 0,
                    );
                }
            }
            let mut gaps = policy.findings(family, profile, stage);
            gaps.extend(combat.check_defaults(member)?);
            if stage == 0
                && family.rarity.is_none()
                && !moves.iter().any(|(name, at)| {
                    let mov = &data.moves[name];
                    *at <= 10 && mov.move_type == type_token(&member.types[0]) && reliable(mov)
                })
            {
                gaps.push("first-type attack by level 10".to_string());
            }
            if stage == 0
                && family.starter_role.is_some()
                && !moves.iter().any(|(name, at)| {
                    let mov = &data.moves[name];
                    *at <= 10
                        && mov.move_type == type_token(&member.types[0])
                        && reliable(mov)
                        && power(mov) >= 35
                        && attack_ratio(data, &profile.base_stats, mov) >= 0.75
                })
            {
                gaps.push("stat-matched starter attack before the first gym".to_string());
            }
            let deadline = arrival.max(policy.type_access_level);
            for kind in &member.types {
                let available = moves.iter().any(|(name, level)| {
                    let mov = &data.moves[name];
                    *level <= deadline
                        && mov.move_type == type_token(kind)
                        && ((reliable(mov) && power(mov) >= 40)
                            || mov.name == "NIGHT_SHADE"
                            || mov.name == "MAGNITUDE")
                });
                if !available {
                    gaps.push(format!("{kind} STAB by {deadline}"));
                }
            }
            let deadline = arrival.max(policy.stat_matched_attack_level);
            let practical = moves.iter().any(|(name, level)| {
                let mov = &data.moves[name];
                *level <= deadline
                    && member
                        .types
                        .iter()
                        .any(|kind| type_token(kind) == mov.move_type)
                    && reliable(mov)
                    && (power(mov) >= 60 || mov.name == "RAZOR_LEAF")
                    && attack_ratio(data, &profile.base_stats, mov) >= 0.75
            });
            if !practical {
                gaps.push(format!("stat-matched STAB by {deadline}"));
            }
            let bst = total(&profile.base_stats);
            if let Some(evolution) = &profile.evolution {
                ensure!(
                    evolution.level > arrival,
                    "{}: evolution levels must increase",
                    member.id
                );
            }
            if let Err(error) = combat.check_battles(member, profile, &moves, arrival) {
                gaps.push(error.to_string());
            }
            combat.check_evolution(member, profile)?;
            if !gaps.is_empty() {
                issues += gaps.len();
                println!(
                    "{} {} (BST {bst}): {}",
                    member.id,
                    member.name,
                    gaps.join("; ")
                );
            }
            let departure = profile.evolution.as_ref().map_or(100, |e| e.level);
            inherited = moves
                .into_iter()
                .filter(|(_, level)| *level <= departure)
                .collect();
            arrival = departure;
        }
    }
    println!(
        "Reviewed {count} active regional species; {issues} progression findings. Native reference: {} species, {} moves. Checks use earned level-up moves plus extension learners; TMs and reminder-only moves do not satisfy the floor.",
        data.pokemon.values().filter(|p| p.int_id <= 251).count(),
        data.moves.len()
    );
    println!(
        "Production engine: {} default move sets, {} battles spending PP and dealing damage, {} evolution boundaries.",
        combat.default_sets, combat.battles, combat.evolutions
    );
    if args.len() == 3 {
        ensure!(issues == 0, "Pokémon balance regression");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attack(power: u16, accuracy: u8, effect: &str) -> Move {
        Move {
            source_index: 1,
            name: "SAMPLE".to_string(),
            move_type: "BUG".to_string(),
            power,
            accuracy,
            pp: 20,
            effect: effect.to_string(),
            effect_chance: 0,
            stat: None,
            amount: None,
        }
    }

    #[test]
    fn evolution_retains_earned_moves_without_granting_reminder_moves() {
        let mut moves = BTreeMap::from([("EMBER".to_string(), 5)]);
        record_earned(&mut moves, "POWDER_SNOW", 1, 36, false);
        record_earned(&mut moves, "ICE_BEAM", 36, 36, false);
        assert_eq!(
            moves,
            BTreeMap::from([("EMBER".to_string(), 5), ("ICE_BEAM".to_string(), 36)])
        );
        record_earned(&mut moves, "EMBER", 40, 36, false);
        assert_eq!(moves["EMBER"], 5);
    }

    #[test]
    fn power_floor_distinguishes_two_hits_from_conditional_and_inaccurate_moves() {
        let double = attack(30, 100, "DOUBLE_HIT");
        assert!(reliable(&double));
        assert_eq!(power(&double), 60);
        for mov in [
            attack(14, 85, "MULTI_HIT"),
            attack(10, 95, "FURY_CUTTER"),
            attack(100, 75, "NORMAL_HIT"),
            attack(1, 100, "LEVEL_DAMAGE"),
        ] {
            assert!(!reliable(&mov));
        }
        assert_eq!(type_token("PSYCHIC"), "PSYCHIC_TYPE");
        assert_eq!(type_token("FAIRY"), "FAIRY");
    }
}
