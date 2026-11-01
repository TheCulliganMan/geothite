//! Actual ordinary tables at late progression, independently of city mimics.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_core::models::Dv;
use crystal_core::random::{CrystalRandom, ReplayDivider};
use crystal_core::state::GameState;
use crystal_core::world::encounters::{EncounterSurface, WildEncounterTable};
use crystal_core::world::map::TilePosition;
use crystal_core::world::movement::MovementMode;

use crate::{CompiledGamePack, GameDataSet};

fn pack() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_CITY_NOTE_BASE_PACK")
                .expect("explicit completed Minnesota pack"),
        );
        crate::read_verified_compiled_game_pack(path).expect("verified parent")
    })
}

fn late_party(data: &GameDataSet) -> GameState {
    let mut state = data
        .start_overworld_session_from_spawn(
            data.runtime_spawn_point(0).expect("native HOME"),
            &data.audio_ids().into_iter().map(str::to_owned).collect(),
        )
        .expect("native new-game state")
        .0;
    state.storage.party.add_pokemon(
        data.create_pokemon("POUCHID", 80, Dv::default())
            .expect("regional late party"),
    );
    state.sync_party_from_storage();
    state.badges.johto = [true; 8];
    for flag in &crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[..8] {
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("consistent late-game badge flags");
    }
    state
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_BASE_PACK: completed Minnesota wild tables"]
fn ordinary_wild_refuges_keep_actual_low_levels_and_basic_species_in_every_table() {
    let data = pack().data();
    assert!(data.wild_challenge.is_some());
    let state = late_party(data);
    let evolved = data
        .evolutions
        .0
        .values()
        .flatten()
        .map(|e| e.species.as_str())
        .collect::<BTreeSet<_>>();
    let mut maps = 0;
    let mut low = BTreeSet::new();
    let mut basic = BTreeSet::new();
    for (name, habitat) in &data.wild_encounters {
        if !name.starts_with("Mn") {
            continue;
        }
        maps += 1;
        let tables: Vec<&WildEncounterTable> = habitat
            .grass
            .iter()
            .chain(&habitat.water)
            .chain(habitat.zones.iter().map(|z| &z.grass))
            .chain(habitat.swarm_overrides.values().map(|s| &s.grass))
            .collect();
        for table in tables {
            for entry in table.morning.iter().chain(&table.day).chain(&table.night) {
                let native = data.ordinary_challenge_level(&state, name, entry.level, 2, 0);
                let challenge = data.ordinary_challenge_level(&state, name, entry.level, 2, 1);
                assert_eq!(native, entry.level, "{name}: native habitat level");
                assert!(challenge >= 79, "{name}: challenge component remains");
                for level in [native, challenge] {
                    assert!(
                        data.saved_wild_encounter_exists(name, &entry.species, level),
                        "both level components must retain valid save origins"
                    );
                    let actual = data
                        .create_pokemon(&entry.species, level, Dv::default())
                        .expect("exact wild species");
                    assert_eq!(
                        actual.species.id, entry.species,
                        "level must not force evolution"
                    );
                }
                if native <= 12 {
                    low.insert(entry.species.clone());
                }
                if !evolved.contains(entry.species.as_str()) {
                    basic.insert(entry.species.clone());
                }
            }
        }
    }
    assert!(maps > 20, "real regional habitat coverage");
    assert!(
        low.len() >= 5,
        "genuinely low-level species remain obtainable after eight badges"
    );
    assert!(
        basic.len() >= 10,
        "first-stage diversity is independent of native levels"
    );
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_BASE_PACK: actual early grass and water"]
fn ordinary_wild_refuges_production_rolls_keep_rarity_species_rng_and_repel() {
    let data = pack().data();
    let mut authored = data.clone();
    authored.wild_challenge = None;
    for surface in [EncounterSurface::Grass, EncounterSurface::Water] {
        let name = "MnR01C00R00";
        let (width, height) = data.saved_map_tile_bounds(name).expect("early route");
        let session = (0..height)
            .find_map(|y| {
                (0..width).find_map(|x| {
                    let mut s = data
                        .overworld_session(name, TilePosition::new(x as i16, y as i16), 0)
                        .ok()?;
                    if surface == EncounterSurface::Water {
                        s.player.mode = MovementMode::Surf;
                    }
                    (s.current_encounter_surface_checked_with_land_encounters(false)
                        .ok()?
                        == Some(surface))
                    .then_some(s)
                })
            })
            .expect("real ordinary habitat surface");
        let mut state = late_party(data);
        data.commit_overworld_snapshot(
            &mut state,
            &session,
            crystal_core::systems::map_context::SpawnMemoryUpdate::Preserve,
        );
        let mut baseline = state.clone();
        let mut a = ReplayDivider::new((0..=255).cycle().take(200000));
        let mut b = ReplayDivider::new((0..=255).cycle().take(200000));
        let mut rng = CrystalRandom::new(state.random_state, &mut a);
        let mut original_rng = CrystalRandom::new(baseline.random_state, &mut b);
        let (mut native, mut scaled) = (0, 0);
        for _ in 0..2048 {
            let roll = data
                .check_wild_encounter_after_step(&mut state, &session, &mut rng)
                .expect("production mixed roll");
            let original = authored
                .check_wild_encounter_after_step(&mut baseline, &session, &mut original_rng)
                .expect("authored roll");
            assert_eq!(rng.state(), original_rng.state(), "no additional RNG draws");
            assert_eq!(
                roll.is_some(),
                original.is_some(),
                "encounter frequency retained"
            );
            if let (Some(roll), Some(original)) = (roll, original) {
                assert_eq!(roll.slot_percent_roll, original.slot_percent_roll);
                if let (Some(actual), Some(old)) = (roll.resolved, original.resolved) {
                    assert_eq!(
                        actual.encounter, old.encounter,
                        "species and authored rarity retained"
                    );
                    if actual.level == old.level {
                        native += 1;
                    } else {
                        assert!(actual.level >= 77);
                        scaled += 1;
                    }
                }
            }
        }
        assert!(
            native > 0 && scaled > 0,
            "both native and scaled {surface:?} opportunities"
        );
        state.repel_steps_remaining = 100;
        state.active_repel_item = Some("REPEL".into());
        let (mut blocked, mut allowed) = (false, false);
        for _ in 0..2048 {
            if let Some(roll) = data
                .check_wild_encounter_after_step(&mut state, &session, &mut rng)
                .expect("Repel policy")
            {
                blocked |= roll.repelled_by.is_some();
                if let Some(e) = roll.resolved {
                    assert!(e.level >= 80);
                    allowed = true;
                }
            }
        }
        assert!(
            blocked && allowed,
            "Repel intentionally hides low levels while allowing peers"
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_BASE_PACK: actual Minnesota fishing group"]
fn ordinary_wild_refuges_fishing_keeps_both_native_and_scaled_encounters() {
    use crystal_core::world::collision::{Terrain, describe_collision, sample_collision};
    use crystal_core::world::fishing::ROD_OLD;
    use crystal_core::world::map::Direction;
    let data = pack().data();
    let name = "MnR01C00R00";
    let (width, height) = data.saved_map_tile_bounds(name).expect("route");
    let shore = (0..height)
        .find_map(|y| {
            (0..width).find_map(|x| {
                let s = data
                    .overworld_session(name, TilePosition::new(x as i16, y as i16), 0)
                    .ok()?;
                if !sample_collision(&s.map, &s.tileset, s.player.tile)
                    .is_some_and(|c| describe_collision(c.permission).terrain == Terrain::Land)
                {
                    return None;
                }
                for facing in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    if sample_collision(&s.map, &s.tileset, s.player.tile.moved(facing))
                        .is_some_and(|c| describe_collision(c.permission).terrain == Terrain::Water)
                    {
                        let mut s = s;
                        s.player.facing = facing;
                        return Some(s);
                    }
                }
                None
            })
        })
        .expect("real fishing shore");
    let mut authored = data.clone();
    authored.wild_challenge = None;
    let (mut native, mut scaled) = (false, false);
    for seed in 0_u32..256 {
        let mut state = late_party(data);
        data.commit_overworld_snapshot(
            &mut state,
            &shore,
            crystal_core::systems::map_context::SpawnMemoryUpdate::Preserve,
        );
        let mut baseline = state.clone();
        let bytes = (0_u32..2048)
            .map(|n| ((n * 73 + seed) % 256) as u8)
            .collect::<Vec<_>>();
        let a = data
            .cast_fishing_rod_in_session_with_divider(
                &mut state,
                &shore,
                ROD_OLD,
                &mut ReplayDivider::new(bytes.clone()),
            )
            .expect("mixed fishing");
        let b = authored
            .cast_fishing_rod_in_session_with_divider(
                &mut baseline,
                &shore,
                ROD_OLD,
                &mut ReplayDivider::new(bytes),
            )
            .expect("authored fishing");
        assert_eq!(
            state.random_state, baseline.random_state,
            "fishing adds no RNG draw"
        );
        assert_eq!(a.wild_battle.is_some(), b.wild_battle.is_some());
        if let (Some(a), Some(b)) = (a.wild_battle, b.wild_battle) {
            for battle in [&a, &b] {
                assert!(
                    battle
                        .enemy_pokemon
                        .moves
                        .iter()
                        .all(|known| !data.items.values().any(|item| {
                            !item.consumable
                                && item.tmhm_move.as_deref() == Some(known.name.as_str())
                        }))
                );
            }
            assert_eq!(a.enemy_pokemon.species.id, b.enemy_pokemon.species.id);
            if a.enemy_pokemon.level == b.enemy_pokemon.level {
                native = true;
            } else {
                assert!(a.enemy_pokemon.level >= 77);
                scaled = true;
            }
        }
        if native && scaled {
            break;
        }
    }
    assert!(
        native && scaled,
        "ordinary rod ecology keeps both level components"
    );
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_BASE_PACK: completed Minnesota wild tables"]
fn regional_wild_moves_and_captures_exclude_hms() {
    use crystal_core::battle::capture::{CaptureOutcome, store_captured_pokemon};
    use crystal_core::models::PokemonStorage;

    ordinary_wild_refuges_fishing_keeps_both_native_and_scaled_encounters();
    let data = pack().data();
    let hms: BTreeSet<_> = data
        .items
        .values()
        .filter(|item| !item.consumable)
        .filter_map(|item| item.tmhm_move.as_deref())
        .collect();
    assert!(!hms.is_empty());
    let mut species: BTreeSet<_> = data
        .wild_encounters
        .iter()
        .filter(|(name, _)| name.starts_with("Mn"))
        .flat_map(|(_, habitat)| {
            habitat
                .grass
                .iter()
                .chain(&habitat.water)
                .chain(habitat.zones.iter().map(|zone| &zone.grass))
                .chain(habitat.swarm_overrides.values().map(|swarm| &swarm.grass))
        })
        .flat_map(|table| table.morning.iter().chain(&table.day).chain(&table.night))
        .map(|entry| entry.species.as_str())
        .collect();
    // Include fishing-only species; all groups use the same encounter boundary.
    for group in data.fishing.groups.values() {
        for slot in group.rod_tables.values().flat_map(|table| &table.slots) {
            if let Some(id) = slot.species.as_deref() {
                species.insert(id);
            }
        }
    }
    for time in data.fishing.time_groups.values() {
        species.insert(time.day_species.as_str());
        species.insert(time.night_species.as_str());
    }
    assert!(!species.is_empty());
    let mut ordinary = data.clone();
    ordinary.wild_challenge = None;
    let id = species.iter().next().expect("regional species");
    let mut wild = ordinary
        .create_pokemon(id, 20, Dv::default())
        .expect("wild species");
    let hm = hms.iter().next().expect("HM item");
    wild.moves.push(crystal_core::models::LearnedMove {
        name: (*hm).to_string(),
        current_pp: 1,
        pp_ups: 0,
    });
    let mut johto = wild.clone();
    ordinary
        .equip_wild_challenge_moves(&mut johto, "Route29")
        .expect("Johto encounter");
    assert_eq!(johto.moves, wild.moves);
    ordinary
        .equip_wild_challenge_moves(&mut wild, "MnR01C00R00")
        .expect("ordinary regional encounter");
    assert!(wild.moves.iter().all(|m| !hms.contains(m.name.as_str())));
    assert!(!wild.moves.is_empty());
    for id in species {
        for level in [5, 10, 11, 20, 50, 100] {
            let mut wild = data
                .create_pokemon(id, level, Dv::default())
                .expect("wild species");
            data.equip_wild_challenge_moves(&mut wild, "MnR01C00R00")
                .expect("regional moves");
            assert!(!wild.moves.is_empty(), "{id} level {level}");
            assert!(
                wild.moves.iter().all(|m| !hms.contains(m.name.as_str())),
                "{id} level {level}: {:?}",
                wild.moves
            );
            let outcome = CaptureOutcome {
                caught: true,
                blocked: false,
                storage_full: false,
                wobble_count: 4,
                animation_shakes: 4,
                final_catch_rate: 255,
                ball_id: Some("MASTER_BALL".into()),
            };
            let mut storage = PokemonStorage::default();
            let captured = store_captured_pokemon(&outcome, &mut storage, 0, wild.clone())
                .expect("store")
                .expect("caught");
            assert_eq!(captured.pokemon.moves, wild.moves);
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_BASE_PACK: actual opening pack"]
fn opening_old_rod_scutlet_has_starter_level_moves_and_a_valid_origin() {
    use crystal_core::systems::map_context::SpawnMemoryUpdate;
    use crystal_core::world::fishing::ROD_OLD;
    use crystal_core::world::map::Direction;

    let data = pack().data();
    assert!(data.wild_challenge.is_some(), "test needs challenge rules");
    let map = "MnR01C00R00";
    let mut shore = data
        .overworld_session(map, TilePosition::new(19, 29), 0)
        .expect("first-route fishing bank");
    shore.player.facing = Direction::Down;
    let mut observed = false;
    for seed in 0_u32..512 {
        let (mut state, _) = data
            .start_overworld_session_from_spawn(
                data.runtime_spawn_point(0).expect("new-game spawn"),
                &data.audio_ids().into_iter().map(str::to_owned).collect(),
            )
            .expect("new game");
        state.storage.party.add_pokemon(
            data.create_pokemon("POUCHID", 5, Dv::default())
                .expect("starter"),
        );
        state.sync_party_from_storage();
        data.commit_overworld_snapshot(&mut state, &shore, SpawnMemoryUpdate::Preserve);
        let divider = (0_u32..2048).map(|n| ((n * 73 + seed) % 256) as u8);
        let result = data
            .cast_fishing_rod_in_session_with_divider(
                &mut state,
                &shore,
                ROD_OLD,
                &mut ReplayDivider::new(divider),
            )
            .expect("cast Old Rod from first-route bank");
        let Some(battle) = result.wild_battle else {
            continue;
        };
        let wild = battle.enemy_pokemon;
        assert!(wild.level <= 6, "{} level {}", wild.species.id, wild.level);
        assert!(
            data.saved_wild_encounter_exists(map, &wild.species.id, wild.level),
            "generated encounter must be a valid saved origin"
        );
        if wild.species.id == "SCUTLET" {
            assert!(wild.moves.iter().all(|m| m.name != "DRAGONBREATH"));
            let natural = data
                .create_pokemon("SCUTLET", wild.level, Dv::default())
                .expect("natural Scutlet");
            assert!(
                wild.moves
                    .iter()
                    .all(|m| natural.moves.iter().any(|n| n.name == m.name))
            );
            observed = true;
        }
    }
    assert!(observed, "Old Rod must encounter SCUTLET in the opening");
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_BASE_PACK: actual opening pack"]
fn opening_grass_rolls_preserve_native_moves_and_valid_origins() {
    use crystal_core::systems::map_context::SpawnMemoryUpdate;

    let data = pack().data();
    assert!(data.wild_challenge.is_some(), "test needs challenge rules");
    let map = "MnR01C00R00";
    let (width, height) = data.saved_map_tile_bounds(map).expect("first route");
    let session = (0..height)
        .find_map(|y| {
            (0..width).find_map(|x| {
                let session = data
                    .overworld_session(map, TilePosition::new(x as i16, y as i16), 0)
                    .ok()?;
                (session
                    .current_encounter_surface_checked_with_land_encounters(false)
                    .ok()?
                    == Some(EncounterSurface::Grass))
                .then_some(session)
            })
        })
        .expect("first-route grass");
    let (mut state, _) = data
        .start_overworld_session_from_spawn(
            data.runtime_spawn_point(0).expect("new-game spawn"),
            &data.audio_ids().into_iter().map(str::to_owned).collect(),
        )
        .expect("new game");
    state.storage.party.add_pokemon(
        data.create_pokemon("POUCHID", 5, Dv::default())
            .expect("starter"),
    );
    state.sync_party_from_storage();
    data.commit_overworld_snapshot(&mut state, &session, SpawnMemoryUpdate::Preserve);
    let mut divider = ReplayDivider::new((0..=255).cycle().take(200000));
    let mut rng = CrystalRandom::new(state.random_state, &mut divider);
    let mut encountered = 0;
    for _ in 0..512 {
        let roll = data
            .check_wild_encounter_after_step(&mut state, &session, &mut rng)
            .expect("production grass roll");
        let Some(resolved) = roll.as_ref().and_then(|r| r.resolved.as_ref()) else {
            continue;
        };
        assert!(
            resolved.level <= 6,
            "{} level {}",
            resolved.encounter.species,
            resolved.level
        );
        assert!(data.saved_wild_encounter_exists(map, &resolved.encounter.species, resolved.level));
        let battle = data
            .start_resolved_wild_encounter_after_step(&mut state, &session, &roll, &mut rng)
            .expect("materialize grass battle")
            .expect("resolved battle");
        let wild = battle.enemy_pokemon;
        let natural = data
            .create_pokemon(&wild.species.id, wild.level, Dv::default())
            .expect("natural wild moves");
        assert!(
            wild.moves
                .iter()
                .all(|m| natural.moves.iter().any(|n| n.name == m.name))
        );
        encountered += 1;
        break;
    }
    assert!(encountered > 0, "first-route grass produces an encounter");
}

#[test]
#[ignore = "requires CRYSTAL_CITY_NOTE_BASE_PACK: Minnesota and original-region catalog"]
fn opening_move_exception_is_regional_only() {
    let data = pack().data();
    assert!(data.wild_challenge.is_some(), "test needs challenge rules");
    let mut wild = data
        .create_pokemon("SCUTLET", 5, Dv::default())
        .expect("regional species");
    let expected = data.competitive_wild_moves(&wild).expect("challenge moves");
    data.equip_wild_challenge_moves(&mut wild, "Route29")
        .expect("original-region moves");
    assert_eq!(wild.moves, expected);
}
