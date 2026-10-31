//! Native TM selection must retain every nonselected move, including Surf.
use crystal_core::models::{Dv, LearnedMove};

use super::super::super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::super::super::r10_gallery::continue_save;
use super::super::super::winona::load;
use super::learn_machine_on;

#[test]
#[ignore = "requires current Minnesota pack; explicit carried partner/machine fixture"]
fn native_machine_replacement_selects_fourth_move_preserves_surf_and_continues() {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnEly".into(),
            tile_x: 11,
            tile_y: 22,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared local machine fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MUDMENDER", 46, Dv::default())
            .expect("legal regional partner");
        assert!(
            pokemon
                .species
                .tmhm_learnset
                .iter()
                .any(|name| name == "BLIZZARD")
        );
        pokemon.moves = ["SURF", "PSYCHIC_M", "REST", "SLEEP_TALK"]
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
        let state = controller.shell.shell.session_mut().state_mut();
        state.storage.party = Default::default();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["TM_BLIZZARD"], 1)
                .expect("declared carried TM")
        );
    }
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    // Only native menu input and actual Save/Continue follow setup.
    learn_machine_on(
        &mut controller,
        &runtime,
        "TM_BLIZZARD",
        "BLIZZARD",
        "SLEEP_TALK",
        0,
    );
    controller = continue_save(root, runtime.clone(), &mut controller);
    let snapshot = controller
        .snapshot()
        .expect("continued selected replacement");
    let moves = snapshot.party.slots[0]
        .pokemon
        .moves
        .iter()
        .map(|known| known.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(moves, ["SURF", "PSYCHIC_M", "REST", "BLIZZARD"]);
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_BLIZZARD"]),
        0
    );
}
