//! Put the two northern clinics' PC interactions on the visible terminals.
use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, authored_town_geometry, authored_town_publication};

const CLINICS: [(&str, &str); 2] = [
    ("MnElyPokecenter1F", "MnElyPC"),
    ("MnSplitRockPokecenter1F", "MnSplitRockPC"),
];

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, label) in CLINICS {
        let mut clinic = pack.data.maps.get(name).context(name)?.clone();
        ensure!(
            authored_town_geometry::kind(pack, &clinic, [8, 0])? == "WALL"
                && authored_town_geometry::kind(pack, &clinic, [9, 1])? == "PC",
            "unexpected furniture in {name}"
        );
        let [terminal] = clinic.events.bg_events.as_mut_slice() else {
            anyhow::bail!("unexpected background events in {name}");
        };
        ensure!(
            terminal.script == label
                && terminal.event_type == "BGEVENT_READ"
                && (terminal.x, terminal.y) == (8, 0),
            "expected {name}'s misplaced PC"
        );
        (terminal.x, terminal.y) = (9, 1);
        authoring::refresh_event_section(&mut clinic);
        authored_town_publication::replace(pack, clinic)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result, ensure};
    use crystal_core::state::GameState;
    use crystal_core::world::map::{Direction, TilePosition};

    #[test]
    #[ignore = "requires a League Journey archive in MN_LEAGUE_JOURNEY_PACK"]
    fn league_journey_clinics_open_storage_at_the_terminal() -> Result<()> {
        let path = std::env::var_os("MN_LEAGUE_JOURNEY_PACK").context("MN_LEAGUE_JOURNEY_PACK")?;
        let pack = crate::read_verified_compiled_game_pack(path)?;
        let data = pack.data();
        let state = GameState::reset_wram_for_new_game();
        for (name, script) in super::CLINICS {
            let mut session = data.overworld_session(name, TilePosition::new(9, 2), 0)?;
            session.player.facing = Direction::Up;
            let hit = session.check_interaction_checked(1)?.context("terminal")?;
            let dispatch = data
                .resolve_overworld_interaction_dispatch(&state, &hit)?
                .context("PC dispatch")?;
            ensure!(dispatch.script == script);
            ensure!(
                data.maps[name].scripts[script]
                    .as_array()
                    .context("PC script")?
                    .contains(&crate::grand_marais_modpack::authoring::command(
                        "special",
                        &["PokemonCenterPC"]
                    ))
            );
            let mut wall = data.overworld_session(name, TilePosition::new(8, 1), 0)?;
            wall.player.facing = Direction::Up;
            ensure!(
                wall.check_interaction_checked(1)?.is_none(),
                "PC in the wall at {name}"
            );
        }
        Ok(())
    }
}
