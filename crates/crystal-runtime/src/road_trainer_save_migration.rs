//! Align previously spinning, undefeated road trainers with their fixed sightlines.

use anyhow::{Context, Result};
use crystal_assets::GameDataSet;
use crystal_core::state::GameState;
use crystal_core::world::session::object_event_initial_facing;

pub(super) fn align_undefeated_trainers(data: &GameDataSet, state: &mut GameState) -> Result<()> {
    for (name, memory) in &mut state.map_object_overrides {
        let [b'M', b'n', b'R', r1, r2, b'C', c1, c2, b'R', y1, y2] = name.as_bytes() else {
            continue;
        };
        if ![r1, r2, c1, c2, y1, y2].iter().all(|n| n.is_ascii_digit()) {
            continue;
        }
        let map = data.map_module(name)?;
        for saved in &mut memory.object_structs.structs {
            let index = saved
                .map_object_index
                .checked_sub(1)
                .context("invalid road object index")?;
            let actor = map
                .objects
                .get(usize::from(index))
                .context("saved road actor is missing")?;
            if actor.object_type != "OBJECTTYPE_TRAINER"
                || actor.sightline_direction_override.is_none()
            {
                continue;
            }
            let battle = map
                .trainer_scripts
                .get(&actor.script)
                .context("missing road trainer event")?;
            if !state.flags.is_event_flag_set(&battle.event_flag)? {
                saved.facing = object_event_initial_facing(&actor.spritemovedata);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crystal_assets::GameDataSet;
    use crystal_core::battle::start::TrainerBattleRequest;
    use crystal_core::state::{GameState, OverworldMemory, OverworldObjectMapMemory};
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::movement::MovementMode;
    use crystal_core::world::session::OverworldObjectStructMemory;

    use super::super::{Compatibility, CompatibleSaveSource, gym_tests, upgrade_source_checkpoint};

    #[test]
    fn compatible_road_saves_restore_only_undefeated_sightline_facings() {
        let road = "MnR02C00R00";
        let mut map = gym_tests::map(road);
        let mut saved = OverworldObjectMapMemory::default();
        for i in 1..=3 {
            let mut actor = gym_tests::actor();
            if i < 3 {
                actor.script = format!("MN_R02_T0{i}");
                actor.object_identifier = Some(actor.script.clone());
                actor.object_type = "OBJECTTYPE_TRAINER".into();
                actor.spritemovedata = "SPRITEMOVEDATA_STANDING_LEFT".into();
                actor.sightline_direction_override = Some("LEFT".into());
                map.trainer_scripts.insert(
                    actor.script.clone(),
                    TrainerBattleRequest::new("LASS", &actor.script, format!("EVENT_TRAINER_{i}")),
                );
            }
            map.objects.push(actor);
            saved
                .object_structs
                .structs
                .push(OverworldObjectStructMemory {
                    slot: i,
                    map_object_index: i,
                    live_tile: TilePosition::new(i16::from(i), 4),
                    last_tile: Some(TilePosition::new(i16::from(i), 3)),
                    initial_tile: TilePosition::new(i16::from(i), 2),
                    facing: Some(Direction::Down),
                    step_duration: Some(8),
                    pending_random_wait: true,
                    last_tile_occupied_remaining_frames: 0,
                    initialized_fixed_spin: true,
                    strength_push_direction: None,
                    strength_moving: false,
                    fixed_facing: false,
                    sliding: false,
                    visible: true,
                    normal_following: false,
                    following_not_exact_leader_slot: None,
                });
        }
        let data = GameDataSet {
            maps: [(road.to_owned(), map)].into(),
            ..GameDataSet::default()
        };
        let mut original = GameState {
            overworld: OverworldMemory::Active {
                map_name: road.into(),
                tile: TilePosition::new(1, 1),
                facing: Direction::Up,
                mode: MovementMode::Normal,
            },
            map_object_overrides: [(road.to_owned(), saved)].into(),
            ..GameState::default()
        };
        original
            .flags
            .set_event_flag("EVENT_TRAINER_2", true)
            .expect("saved victory");
        let mut corrected = original.clone();
        corrected
            .map_object_overrides
            .get_mut(road)
            .expect("saved road")
            .object_structs
            .structs[0]
            .facing = Some(Direction::Left);
        let mut metadata = Compatibility {
            schema_version: 4,
            target_manifest: "northstar-story-flow-v1".into(),
            target_modpack_ids: Vec::new(),
            sources: Vec::new(),
            gym_reentry: None,
            cave_reentry: None,
            iron_relocations: None,
            layout_reentry: None,
            regional_parents: vec!["direct-story".into()],
            unchanged_checkpoint_parents: vec!["direct-regional".into()],
        };
        let regional = CompatibleSaveSource {
            modpack_id: "direct-regional".into(),
            modpack_hash: "b".repeat(64),
            content_hash: "a".repeat(64),
        };
        assert_eq!(
            upgrade_source_checkpoint(original.clone(), &metadata, &regional, &data)
                .expect("a current regional save keeps its actors"),
            original
        );
        for source_id in ["direct-story", "older-compatible"] {
            let source = CompatibleSaveSource {
                modpack_id: source_id.into(),
                modpack_hash: "a".repeat(64),
                content_hash: "b".repeat(64),
            };
            assert_eq!(
                upgrade_source_checkpoint(original.clone(), &metadata, &source, &data)
                    .expect("road migration"),
                corrected
            );
        }
        metadata.regional_parents.clear();
        let source = CompatibleSaveSource {
            modpack_id: "older-compatible".into(),
            modpack_hash: "a".repeat(64),
            content_hash: "b".repeat(64),
        };
        assert_eq!(
            upgrade_source_checkpoint(original.clone(), &metadata, &source, &data)
                .expect("previous release policy"),
            original
        );
    }
}
