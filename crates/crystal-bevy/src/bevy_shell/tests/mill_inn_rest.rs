//! The mill inn's offer, bed and public doorway use ordinary field inputs.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::minnesota_fossils::talk;
use super::{VisibleShellController, at, fixture, load, mark_runtime_snapshot_dirty, walk};

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the Mill Workers Inn rest service"]
fn mill_workers_inn_offers_free_rest_without_moving_the_clinic() {
    const CITY: &str = "MnMinneapolis";
    const INN: &str = "MnMinneapolisRiverInn";
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, CITY, (53, 60));
    let mut rested = Vec::new();
    let state = c.shell.shell.session_mut().state_mut();
    let last_clinic = state.last_spawn_map_constant.clone();
    assert!(last_clinic.is_some());
    for (species, hp) in [("POUCHID", 1), ("BRISTUB", 0)] {
        let mut mon = runtime
            .data()
            .create_pokemon(species, 20, Dv::default())
            .expect("regional guest");
        rested.push((
            mon.hp,
            None,
            mon.moves.iter().map(|m| m.current_pp).collect::<Vec<_>>(),
        ));
        mon.hp = hp;
        mon.status = (hp > 0).then(|| "PARALYSIS".to_owned());
        for m in &mut mon.moves {
            m.current_pp = 0;
        }
        assert!(state.storage.party.add_pokemon(mon));
    }
    state.sync_party_from_storage();
    state.money = 0;
    mark_runtime_snapshot_dirty(&mut c.shell);
    let vitals = |c: &mut VisibleShellController| {
        c.snapshot()
            .expect("guest party")
            .party
            .slots
            .into_iter()
            .map(|s| {
                let mon = s.pokemon;
                (
                    mon.hp,
                    mon.status,
                    mon.moves
                        .into_iter()
                        .map(|m| m.current_pp)
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    let injured = vitals(&mut c);
    walk(&mut c, &runtime, (53, 59));
    assert_eq!(c.snapshot().expect("inn lobby").overworld.map_name, INN);
    assert_eq!(at(&mut c), (3, 6));
    walk(&mut c, &runtime, (2, 4));
    talk(&mut c, GameButton::Up, &[false]);
    assert_eq!(vitals(&mut c), injured);
    talk(&mut c, GameButton::Up, &[true]);
    assert_eq!(vitals(&mut c), rested);

    walk(&mut c, &runtime, (3, 7));
    assert_eq!(
        c.snapshot().expect("Minneapolis street").overworld.map_name,
        CITY
    );
    assert_eq!(at(&mut c), (53, 60));
    walk(&mut c, &runtime, (53, 59));
    walk(&mut c, &runtime, (6, 3));
    talk(&mut c, GameButton::Right, &[true]);
    assert_eq!(at(&mut c), (6, 3));
    assert_eq!(vitals(&mut c), rested);
    walk(&mut c, &runtime, (4, 7));
    assert_eq!(
        c.snapshot()
            .expect("second street return")
            .overworld
            .map_name,
        CITY
    );
    assert_eq!(at(&mut c), (53, 60));
    let state = c.shell.shell.session().state();
    assert_eq!(state.money, 0);
    assert_eq!(state.last_spawn_map_constant, last_clinic);
}
