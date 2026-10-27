// Exercise the restored family through the real Pack and FIGHT controller paths.
#[test]
fn northwoods_sticky_family_evolves_at_24_and_44_through_production_pack() {
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    for (actor, level, target, stage_move) in [
        ("JOTLET", 22, "JOTLET", None),
        ("JOTLET", 23, "MEMOFOLD", Some("DISABLE")),
        ("MEMOFOLD", 42, "MEMOFOLD", None),
        ("MEMOFOLD", 43, "RECALLISK", Some("PSYCH_UP")),
    ] {
        let mut shell = route36_overworld_shell_for_battle_render_regression();
        if shell.runtime.data().saved_species("JOTLET").is_none() {
            return;
        }
        shell
            .shell
            .set_trainer_identity("CHRIS", 1)
            .expect("fixture owner");
        {
            let state = shell.shell.session_mut().state_mut();
            state.storage.party.pokemon.fill(None);
            state.sync_party_from_storage();
        }
        let dvs = Dv::from_non_hp(9, 10, 10, 10);
        shell
            .shell
            .add_party_pokemon(actor, level, None, None, "CHRIS", 1, dvs)
            .expect("owned pre-evolution Pokemon");
        let moves = ["POUND", "CONFUSION", "PSYBEAM"].map(|id| {
            assert!(
                shell.runtime.data().learnsets[actor]
                    .iter()
                    .any(|entry| entry.0 <= level && entry.1 == id)
            );
            crate::core::models::LearnedMove {
                name: id.to_string(),
                current_pp: shell.runtime.data().moves[id].pp,
                pp_ups: 0,
            }
        });
        {
            let state = shell.shell.session_mut().state_mut();
            state.storage.party.pokemon[0]
                .as_mut()
                .expect("owned actor")
                .moves = moves.into();
            state.sync_party_from_storage();
        }
        shell
            .shell
            .add_bag_item("RARE_CANDY", 1)
            .expect("one native Rare Candy");
        let mut controller = VisibleShellController { shell };
        controller
            .press(GameButton::Start)
            .expect("open production Start");
        for _ in 0..10 {
            if visible_start_menu_entries(&controller.shell)
                .expect("Start entries")
                .iter()
                .any(|entry| entry == ">PACK")
            {
                break;
            }
            controller.press(GameButton::Down).expect("select PACK");
        }
        assert!(
            visible_start_menu_entries(&controller.shell)
                .expect("selected PACK")
                .iter()
                .any(|entry| entry == ">PACK")
        );
        controller.press(GameButton::A).expect("open PACK");
        assert_eq!(
            controller.shell.field_pack_pocket,
            Some(FieldPackPocket::Items)
        );
        let index = controller
            .snapshot()
            .expect("native bag")
            .bag
            .items
            .iter()
            .filter(|item| item.quantity > 0)
            .position(|item| item.item_id == "RARE_CANDY")
            .expect("actual carried Rare Candy");
        for _ in 0..index {
            controller
                .press(GameButton::Down)
                .expect("select Rare Candy");
        }
        controller.press(GameButton::A).expect("item action menu");
        assert!(controller.shell.field_pack_action_cursor.is_some());
        controller.press(GameButton::A).expect("USE Rare Candy");
        assert_eq!(
            controller.shell.field_pack_target_mode,
            Some(FieldPackTargetMode::PartyPokemon)
        );
        controller
            .press(GameButton::A)
            .expect("select owned Pokemon");
        let mut visible_text = String::new();
        for _ in 0..64 {
            let snapshot = controller
                .snapshot()
                .expect("visible native growth/evolution");
            if let Some(text) = snapshot.ui.text.and_then(|text| text.asm_text) {
                visible_text.push_str(&text);
                visible_text.push(' ');
            }
            if controller.shell.field_notice.is_none()
                && controller.shell.field_notice_queue.is_empty()
                && controller.shell.field_evolution_cancellation.is_none()
            {
                break;
            }
            controller
                .press(GameButton::A)
                .expect("advance actual growth/evolution text");
        }
        let after = controller.snapshot().expect("completed native item use");
        let pokemon = &after.party.slots[0].pokemon;
        assert_eq!(pokemon.species.id, target, "{actor} at level{}", level + 1);
        assert_eq!(pokemon.level, level + 1);
        assert_eq!(pokemon.dvs, dvs);
        assert!(visible_text.contains("grew"), "{visible_text}");
        if let Some(stage_move) = stage_move {
            assert!(visible_text.contains("evolved"), "{visible_text}");
            assert!(
                pokemon.moves.iter().any(|entry| entry.name == stage_move),
                "{target} learns {stage_move} at acquisition"
            );
        } else {
            assert!(
                !visible_text.contains("evolved"),
                "no evolution before the authored boundary"
            );
        }
        assert!(controller.shell.field_evolution_cancellation.is_none());
        let state = controller.shell.shell.session().state();
        assert_eq!(
            state
                .bag
                .quantity(&controller.shell.runtime.data().items["RARE_CANDY"]),
            0
        );
    }
}

#[test]
fn northwoods_sticky_family_uses_stage_legal_moves_through_production_controller() {
    if std::env::var_os("CRYSTAL_RENDER_TEST_PACK").is_none() {
        return;
    }
    let probe = route36_overworld_shell_for_battle_render_regression();
    if probe.runtime.data().saved_species("JOTLET").is_none() {
        return;
    }
    let move_id = "PSYBEAM";
    let label = probe.runtime.data().battle_animation_table
        [usize::from(probe.runtime.data().moves[move_id].source_index)]
    .clone();
    drop(probe);
    for (actor, level) in [("JOTLET", 23), ("MEMOFOLD", 24), ("RECALLISK", 44)] {
        assert_northwoods_controller_move_hits_at_level(move_id, actor, &label, true, level);
    }
}
