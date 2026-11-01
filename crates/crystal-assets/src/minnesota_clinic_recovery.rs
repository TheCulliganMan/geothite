//! Recovery points for Minnesota's ordinary native nurse services.

use anyhow::{Context, Result};
use crystal_core::world::map::TilePosition;

use crate::{GameDataSet, MapModule};

impl GameDataSet {
    pub(crate) fn minnesota_clinic(&self, map_name: &str) -> Option<&MapModule> {
        let map = self.maps.get(map_name)?;
        (map_name.starts_with("Mn")
            && matches!(
                map.attributes.tileset_name.as_str(),
                "pokecenter" | "pokecom_center"
            )
            && map
                .objects
                .iter()
                .any(|object| object.sprite == "SPRITE_NURSE"))
        .then_some(map)
    }

    pub(crate) fn minnesota_clinic_for_constant(&self, constant: &str) -> Option<&MapModule> {
        let map_name = self.map_name_for_constant(constant).ok()?;
        self.minnesota_clinic(&map_name)
    }

    /// Resolve a clinic omitted from the optional recovery spawn catalog.
    /// Its authored arrival warp is the same safe interior tile used on entry.
    pub(crate) fn minnesota_clinic_arrival(
        &self,
        constant: &str,
    ) -> Result<Option<(String, TilePosition)>> {
        let Some(map) = self.minnesota_clinic_for_constant(constant) else {
            return Ok(None);
        };
        let arrival = map
            .events
            .warps
            .iter()
            .find(|warp| warp.index == 3)
            .with_context(|| format!("Minnesota clinic {} has no arrival warp", map.id))?;
        let tile = crystal_core::world::session::raw_event_tile_to_runtime_tile_checked(
            arrival.x, arrival.y,
        )
        .with_context(|| format!("Minnesota clinic {} arrival is out of range", map.id))?;
        Ok(Some((map.id.clone(), tile)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use crystal_core::models::{Dv, LearnedMove};
    use crystal_core::save::{
        SaveModpackIdentity, read_save_game_for_modpack, write_save_game_for_modpack,
    };
    use crystal_core::state::{GameState, OverworldMemory};
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::movement::MovementMode;

    use crate::{RuntimeMutationCommand, RuntimeMutationResult, read_verified_compiled_game_pack};

    #[test]
    #[ignore = "requires CRYSTAL_WINONA_TEST_PACK with Minnesota clinics"]
    fn healing_in_an_unregistered_clinic_becomes_the_saved_whiteout_destination() {
        let pack = read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_WINONA_TEST_PACK").expect("Minnesota pack"),
        ))
        .expect("verified Minnesota pack");
        let data = pack.data();
        let clinic = "MnBrainerdPokecenter1F";
        let constant = "MN_BRAINERD_POKECENTER_1F";
        assert!(data.runtime_spawn_point_for_map_constant(constant).is_err());
        let (_, arrival) = data
            .minnesota_clinic_arrival(constant)
            .expect("clinic arrival")
            .expect("clinic");
        let mut state = GameState::default();
        state.last_spawn_map_constant = Some("MN_WINONA_POKECENTER_1F".into());
        state.overworld = OverworldMemory::Active {
            map_name: clinic.into(),
            tile: arrival,
            facing: Direction::Up,
            mode: MovementMode::Normal,
        };
        let mut partner = data
            .create_pokemon("BIRCHIT", 5, Dv::default())
            .expect("starter");
        partner.hp = 1;
        state.storage.party.pokemon[0] = Some(partner);
        state.sync_party_from_storage();
        data.apply_special_routine(&mut state, "HealParty", &BTreeSet::new())
            .expect("nurse heal");
        assert_eq!(state.last_spawn_map_constant.as_deref(), Some(constant));
        assert_eq!(
            state.storage.party.pokemon[0].as_ref().expect("partner").hp,
            state.storage.party.pokemon[0]
                .as_ref()
                .expect("partner")
                .max_hp
        );

        let mut travel_state = state.clone();
        let teleport = data.field_moves.teleport.move_id.clone();
        travel_state.storage.party.pokemon[0]
            .as_mut()
            .expect("partner")
            .moves
            .push(LearnedMove {
                name: teleport,
                current_pp: 1,
                pp_ups: 0,
            });
        travel_state.sync_party_from_storage();
        let mut travel_session = data
            .overworld_session("MnWinona", TilePosition::new(14, 18), 0)
            .expect("outdoor travel start");
        travel_state.overworld = OverworldMemory::Active {
            map_name: "MnWinona".into(),
            tile: TilePosition::new(14, 18),
            facing: Direction::Down,
            mode: MovementMode::Normal,
        };
        let travel = data
            .use_teleport_field_move_in_session(
                &mut travel_state,
                &mut travel_session,
                0,
                &BTreeSet::new(),
            )
            .expect("teleport to unregistered clinic");
        assert_eq!(travel.destination_spawn_identifier, None);
        assert_eq!(travel.destination_map, clinic);
        assert_eq!(travel.destination_tile, arrival);
        data.validate_saved_pending_field_moves(&travel_state)
            .expect("pending teleport remains valid on save");
        let hash = "0".repeat(64);
        let identity =
            SaveModpackIdentity::new("clinic-recovery-test", &hash).expect("test save identity");
        let save_path = std::env::temp_dir().join(format!(
            "geothite-clinic-teleport-{}.crystalsave",
            std::process::id()
        ));
        write_save_game_for_modpack(&save_path, travel_state, &identity, &hash)
            .expect("save pending clinic teleport");
        let mut travel_state = read_save_game_for_modpack(&save_path, &identity, &hash)
            .expect("reload pending clinic teleport")
            .into_state();
        std::fs::remove_file(save_path).expect("remove temporary save");
        data.validate_saved_pending_field_moves(&travel_state)
            .expect("reloaded teleport remains valid");
        let mut tampered = travel_state.clone();
        tampered
            .script_runtime
            .pending_field_travel
            .as_mut()
            .expect("pending teleport")
            .destination_tile = TilePosition::new(0, 0);
        assert!(data.validate_saved_pending_field_moves(&tampered).is_err());
        data.commit_pending_field_travel(&mut travel_state, &mut travel_session, &BTreeSet::new())
            .expect("complete teleport");
        assert_eq!(travel_session.map.name, clinic);
        assert_eq!(travel_session.player.tile, arrival);

        state.storage.party.pokemon[0].as_mut().expect("partner").hp = 0;
        state.sync_party_from_storage();
        let mut session = data
            .overworld_session("MnWinonaHome2F", TilePosition::new(3, 3), 0)
            .expect("distant starting point");
        state.overworld = OverworldMemory::Active {
            map_name: "MnWinonaHome2F".into(),
            tile: TilePosition::new(3, 3),
            facing: Direction::Down,
            mode: MovementMode::Normal,
        };
        let outcome = data
            .apply_runtime_mutation_command(
                &mut state,
                &mut session,
                RuntimeMutationCommand::ResolvePoisonBlackoutToLastSpawn,
                &BTreeSet::new(),
                &BTreeSet::new(),
                &BTreeSet::new(),
            )
            .expect("whiteout returns to the healed clinic");
        let RuntimeMutationResult::BlackoutResolved(recovery) = outcome.result else {
            panic!("expected blackout recovery");
        };
        assert_eq!(recovery.map_name, clinic);
        assert_eq!(recovery.tile, arrival);
        assert_eq!(session.map.name, clinic);
        assert_eq!(session.player.tile, arrival);
    }
}
