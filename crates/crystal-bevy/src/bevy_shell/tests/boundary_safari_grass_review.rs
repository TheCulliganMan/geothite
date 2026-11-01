//! Selected closed-preserve geometry fixtures; never grant a paid permit.
use super::*;

#[test]
#[ignore = "requires selected grass review pack and retained-save directory"]
fn safari_grass_review_native_reached_pine_portage_checkpoint_and_continue() {
    let l = layout();
    let (root, runtime, mut c) = fixture((22, 21), false);
    mount_surf(&mut c, GameButton::Left, (21, 21));
    step(&mut c, GameButton::Right, (22, 21));
    let g = grid(&runtime, (22, 21), &l);
    g.walk(&mut c, (29, 21));
    assert_no_expedition(&c);
    let directory = PathBuf::from(
        std::env::var_os("CRYSTAL_BOUNDARY_SAFARI_EXPANSION_SAVE_DIR").expect("retained saves"),
    );
    std::fs::create_dir_all(&directory).expect("checkpoint directory");
    let path = directory.join("pine-portage-reached.crystalsave");
    assert!(!path.exists());
    c.save(&path).expect("actual portage checkpoint");
    c = save_continue(root, runtime, &mut c);
    assert_eq!(position(&mut c), (29, 21));
}

#[test]
#[ignore = "requires selected grass review pack and retained-save directory"]
fn safari_grass_review_native_public_pine_seam_both_lanes_and_continue() {
    for x in [14, 15] {
        let (root, runtime, mut c) = fixture((x, 111), false);
        let before = c.snapshot().expect("initial public court");
        let original_flags = flags(&c);
        for _ in 0..8 {
            press(&mut c, GameButton::Down);
            if c.snapshot()
                .expect("native Pine arrival")
                .overworld
                .map_name
                == "MnPreservePineEntry"
            {
                break;
            }
        }
        let outside = c.snapshot().expect("actual Pine arrival").overworld;
        assert_eq!(outside.map_name, "MnPreservePineEntry");
        assert_eq!((outside.tile.x, outside.tile.y), (x + 2, 0));
        assert_no_expedition(&c);
        c = save_continue(root, runtime, &mut c);
        for _ in 0..8 {
            press(&mut c, GameButton::Up);
            if c.snapshot()
                .expect("native court return")
                .overworld
                .map_name
                == "MnBoundarySafariC00R02"
            {
                break;
            }
        }
        assert_eq!(position(&mut c), (x, 111));
        let after = c.snapshot().expect("returned without paid resources");
        assert_eq!(after.party, before.party);
        assert_eq!(after.bag, before.bag);
        assert_eq!(flags(&c), original_flags);
        assert_eq!(c.shell.shell.session().state().money, 1000);
        assert_no_expedition(&c);
    }
}

fn live_tree(c: &VisibleShellController) -> u16 {
    c.shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(5, 15)
        .expect("native thicket tree")
}
fn cut(c: &mut VisibleShellController, facing: GameButton) {
    let at = position(c);
    press(c, facing);
    assert_eq!(position(c), at);
    for _ in 0..96 {
        if live_tree(c) == 0x3f {
            close_service(c);
            return;
        }
        press(c, GameButton::A);
    }
    panic!("native selected-pack Cut did not open the tree");
}
#[test]
#[ignore = "requires selected grass review pack and retained-save directory"]
fn safari_grass_review_native_cut_permissions_thicket_inside_reset_and_recut() {
    for (knows, hive) in [(false, true), (true, false), (true, true)] {
        let (root, runtime, mut c) = fixture((10, 30), false);
        let mut partner = runtime
            .data()
            .create_pokemon("VOYAGEAR", 40, Dv::default())
            .expect("initial regional Cut partner");
        assert!(partner.species.tmhm_learnset.iter().any(|m| m == "CUT"));
        assert!(partner.moves.iter().all(|m| m.name != "CUT"));
        if knows {
            partner.moves[0] = LearnedMove {
                name: "CUT".into(),
                current_pp: runtime.data().moves["CUT"].pp,
                pp_ups: 0,
            };
        }
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
        state.badges.johto[1] = hive;
        state
            .flags
            .set_engine_flag("ENGINE_HIVEBADGE", hive)
            .expect("consistent initial Cut badge");
        mark_runtime_snapshot_dirty(&mut c.shell);
        // All explicit fixture setup ends here. No position, permission or map repairs follow.
        assert_eq!(live_tree(&c), 0x63);
        if !knows || !hive {
            press(&mut c, GameButton::Down);
            for _ in 0..8 {
                press(&mut c, GameButton::A);
            }
            close_service(&mut c);
            assert_eq!(live_tree(&c), 0x63);
            assert_eq!(position(&mut c), (10, 30));
            assert_no_expedition(&c);
            continue;
        }
        cut(&mut c, GameButton::Down);
        let l = layout();
        let mut g = grid(&runtime, (10, 30), &l);
        // The planner follows the observed native temporary replacement; it never edits live terrain.
        // RouteGrid caches only the initial connected component. The actual
        // Cut opens the previously separate inner component as well.
        g.floor.extend(grid(&runtime, (9, 35), &l).floor);
        g.floor.insert((10, 31));
        g.walk(&mut c, (9, 35));
        c = save_continue(root, runtime.clone(), &mut c);
        assert_eq!(
            live_tree(&c),
            0x63,
            "native Continue regenerates temporary Cut tree"
        );
        g.walk(&mut c, (10, 32));
        cut(&mut c, GameButton::Up);
        g.walk(&mut c, (10, 30));
        assert_no_expedition(&c);
        assert_eq!(c.shell.shell.session().state().money, 1000);
    }
}

#[test]
#[ignore = "traversal only; duplicate indoor Dorothy remains a separate open story issue"]
fn safari_grass_review_native_cabin_doors_with_current_residents_and_continue() {
    let l = layout();
    let door = &l["root_beer_lady"]["door"];
    let arrival = point(&door["arrival_tile"]);
    let (root, runtime, mut c) = fixture(arrival, false);
    let g = grid(&runtime, arrival, &l);
    for x in [5, 6] {
        g.walk(&mut c, point(&door["tile"]));
        let entered = c.snapshot().expect("actual cabin arrival").overworld;
        assert_eq!(entered.map_name, CABIN);
        assert_eq!((entered.tile.x, entered.tile.y), (5, 6));
        indoor_walk(&mut c, &runtime, (6, 6));
        indoor_walk(&mut c, &runtime, (5, 6));
        if x == 5 {
            c = save_continue(root.clone(), runtime.clone(), &mut c);
        }
        indoor_walk(&mut c, &runtime, (x, 7));
        for _ in 0..8 {
            if c.snapshot().expect("native cabin exit").overworld.map_name != CABIN {
                break;
            }
            press(&mut c, GameButton::Down);
        }
        assert_eq!(position(&mut c), arrival);
        assert_no_expedition(&c);
    }
}
