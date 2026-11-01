//! Use earned machines through native Pack teaching before League preparation.
use crystal_runtime::CrystalRuntime;

use super::super::northstar_campaign::learn_machine_on;
use super::VisibleShellController;

/// Returns whether an actual carried single-use machine was taught.
/// Retains stronger same-type attacks, regardless of their remaining PP, and
/// preserves Fire Blast when adding missing Ice coverage.
pub(super) fn teach_carried(c: &mut VisibleShellController, r: &CrystalRuntime) -> bool {
    let mut taught = false;
    for (species, item, learned, forgotten) in [
        ("BOREALYNX", "TM_ICY_WIND", "ICY_WIND", "HEADBUTT"),
        ("PARAFLOSS", "TM_GIGA_DRAIN", "GIGA_DRAIN", "SWEET_SCENT"),
        ("GOUGECREST", "TM_MUD_SLAP", "MUD_SLAP", "FOCUS_ENERGY"),
    ] {
        if c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items[item])
            != 1
        {
            continue;
        }
        let new_move = &r.data().moves[learned];
        let snapshot = c.snapshot().expect("carried earned machine recipients");
        let Some(slot) = snapshot
            .party
            .slots
            .iter()
            .find(|slot| {
                slot.pokemon.species.id == species
                    && slot
                        .pokemon
                        .species
                        .tmhm_learnset
                        .iter()
                        .any(|name| name == learned)
                    && !slot.pokemon.moves.iter().any(|known| known.name == learned)
                    && slot
                        .pokemon
                        .moves
                        .iter()
                        .any(|known| known.name == forgotten)
                    && !slot.pokemon.moves.iter().any(|known| {
                        let old_move = &r.data().moves[&known.name];
                        new_move.power > 0
                            && old_move.move_type == new_move.move_type
                            && old_move.power >= new_move.power
                    })
            })
            .map(|slot| slot.index)
        else {
            continue;
        };
        learn_machine_on(c, r, item, learned, forgotten, slot);
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items[item]),
            0
        );
        assert!(
            c.snapshot().expect("native earned coverage").party.slots[slot]
                .pokemon
                .moves
                .iter()
                .any(|known| known.name == learned)
        );
        taught = true;
    }
    taught
}

#[test]
#[ignore = "requires a current Minnesota pack; declared local party/TM then native teaching and Continue"]
fn earned_league_machine_preserves_stronger_ice_and_teaches_missing_coverage_then_continues() {
    use crystal_core::models::{Dv, LearnedMove};

    use super::super::super::super::{
        BevyShellConfig, BevyShellStart, complete_visible_smoke_player_name_if_needed,
        initialize_bevy_runtime_shell, mark_runtime_snapshot_dirty,
    };
    use super::super::super::r10_gallery::continue_save;
    use super::super::super::winona::load;

    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnElyPokecenter1F".into(),
            tile_x: 3,
            tile_y: 6,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("declared local machine fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    {
        // Both legal carried movesets and the one prior machine are declared
        // setup. After this block, teaching and persistence use the controller.
        let state = controller.shell.shell.session_mut().state_mut();
        state.storage.party = Default::default();
        for (index, moves) in [
            ["ICE_BEAM", "HEADBUTT", "FIRE_BLAST", "QUICK_ATTACK"],
            ["REST", "HEADBUTT", "FIRE_BLAST", "QUICK_ATTACK"],
        ]
        .into_iter()
        .enumerate()
        {
            let mut pokemon = runtime
                .data()
                .create_pokemon("BOREALYNX", 51, Dv::default())
                .expect("active regional partner");
            assert!(
                pokemon
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|name| name == "REST")
            );
            pokemon.moves = moves
                .into_iter()
                .map(|name| LearnedMove {
                    name: name.into(),
                    current_pp: if index == 0 && name == "FIRE_BLAST" {
                        0
                    } else {
                        runtime.data().moves[name].pp
                    },
                    pp_ups: 0,
                })
                .collect();
            assert!(state.storage.party.add_pokemon(pokemon));
        }
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["TM_ICY_WIND"], 1)
                .expect("declared carried TM")
        );
    }
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller.set_runtime_journal_enabled(false);
    let before = controller.snapshot().expect("declared movesets").party;
    assert!(teach_carried(&mut controller, &runtime));
    let after = controller.snapshot().expect("native taught coverage").party;
    assert_eq!(after.slots[0].pokemon, before.slots[0].pokemon);
    let recipient = &after.slots[1].pokemon;
    for name in ["REST", "ICY_WIND", "FIRE_BLAST", "QUICK_ATTACK"] {
        assert!(recipient.moves.iter().any(|known| known.name == name));
    }
    assert!(!recipient.moves.iter().any(|known| known.name == "HEADBUTT"));
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_ICY_WIND"]),
        0
    );
    assert!(!teach_carried(&mut controller, &runtime));
    controller = continue_save(root, runtime.clone(), &mut controller);
    assert_eq!(
        controller
            .snapshot()
            .expect("native machine Continue")
            .party,
        after
    );
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["TM_ICY_WIND"]),
        0
    );
}
