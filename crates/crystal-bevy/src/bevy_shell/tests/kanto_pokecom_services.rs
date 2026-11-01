// Initial positions are review fixtures. Every service branch, text page,
// cancellation and subsequent movement is driven by production joypad input.
#[test]
#[ignore = "requires an external verified pack via CRYSTAL_RENDER_TEST_PACK"]
fn kanto_pokecom_office_computers_serve_all_choices_and_release_input() {
    const MAP: &str = "PokecomCenterAdminOfficeMobile";
    fn ready(controller: &mut VisibleShellController, menu_id: &str) -> bool {
        controller
            .snapshot()
            .unwrap()
            .ui
            .menu
            .as_ref()
            .is_some_and(|menu| menu.menu_id == menu_id)
            && controller.shell.field_notice.is_none()
            && controller.shell.pc_notice.is_none()
    }
    fn read_pages(controller: &mut VisibleShellController, label: &str, menu_id: Option<&str>) {
        let raw = controller.shell.shell.presentation_snapshot().unwrap();
        assert_eq!(
            raw.ui.text.as_ref().map(|text| text.label.as_str()),
            Some(label),
            "a source choice must open its own authored text"
        );
        let expected = visible_field_dialog_pages(&raw, &controller.shell).expect("source pages");
        assert!(!expected.is_empty());
        let stream = expected.join("\u{1e}");
        let tile = raw.overworld.tile;
        let mut seen = std::collections::BTreeSet::new();
        let mut checked_input_owner = false;
        for _ in 0..512 {
            let snapshot = controller.snapshot().unwrap();
            if snapshot.ui.text_window_open
                && snapshot
                    .ui
                    .text
                    .as_ref()
                    .is_some_and(|text| text.label == label)
                && let Some(reveal) = controller.shell.field_text_reveal.as_ref()
                && reveal.text == stream
                && let Some(page) = expected.get(reveal.page_index)
                && reveal.visible_chars >= page.chars().count()
            {
                assert_eq!(
                    snapshot.ui.text.as_ref().unwrap().asm_text.as_deref(),
                    Some(page.as_str())
                );
                seen.insert(reveal.page_index);
                if !checked_input_owner {
                    controller.press(GameButton::Left).unwrap();
                    assert_eq!(
                        controller.snapshot().unwrap().overworld.tile,
                        tile,
                        "office dialogue owns directional input"
                    );
                    checked_input_owner = true;
                }
            }
            let done = match menu_id {
                Some(menu) => ready(controller, menu),
                None => {
                    !snapshot.ui.window_open
                        && !snapshot.ui.text_window_open
                        && controller.shell.active_script_cursor.is_none()
                }
            };
            if done {
                break;
            }
            controller.press(GameButton::A).unwrap();
        }
        assert_eq!(
            seen,
            (0..expected.len()).collect(),
            "every authored page must be visibly presented: {label}"
        );
        // Independent source passages also guard against a truncated paginator
        // reporting a shorter stream as if it were the complete document.
        let passages: &[&str] = if label.ends_with("Text_PokeComClub") {
            &[
                "CLUB upstairs in",
                "CENTER.",
                "battle or trade",
                "MOBILE ADAPTER.",
                "friend must have",
                "the same kind of",
                "you.",
            ]
        } else if label.ends_with("Text_MobileCenter") {
            &[
                "To use the TRADE",
                "CORNER or read the",
                "MOBILE CENTER.",
                "You must register",
                "connecting there.",
            ]
        } else if label.ends_with("Text_UsePhone") {
            &[
                "Please ensure that",
                "properly linked.",
                "signal is strong.",
                "Don't touch or",
                "while linking.",
            ]
        } else if label.ends_with("Text_DontUsePhone") {
            &[
                "If the server is",
                "be possible to log",
                "back later.",
                "error messages,",
                "instructions.",
            ]
        } else if label.ends_with("Computer3Text") {
            &[
                "The ADMINISTRATION",
                "e-mail. It says",
                "CENTER staff",
                "Wireless communi-",
                "to interact across",
                "keep working for",
                "the trainers in",
                "without barriers!",
            ]
        } else if label.ends_with("Computer1Text") {
            &[
                "It's a notice",
                "about where MOBILE",
                "ADAPTERS are to be",
                "used",
            ]
        } else {
            assert!(label.ends_with("Computer2Text"));
            &["It's a notice", "about using the", "phone"]
        };
        let visible = seen
            .iter()
            .map(|&index| expected[index].as_str())
            .collect::<Vec<_>>()
            .join(" ");
        let visible = visible.split_whitespace().collect::<Vec<_>>().join(" ");
        for passage in passages {
            assert!(
                visible.contains(passage),
                "missing source passage {passage:?} in {label}: {visible}"
            );
        }
        assert!(checked_input_owner);
        assert_eq!(controller.snapshot().unwrap().overworld.tile, tile);
        if let Some(menu) = menu_id {
            assert!(
                ready(controller, menu),
                "source help must return to its own menu"
            );
        } else {
            let snapshot = controller.snapshot().unwrap();
            assert!(!snapshot.ui.window_open && !snapshot.ui.text_window_open);
        }
    }

    let pack = PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(pack.parent().unwrap());
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack).unwrap();
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).unwrap();
    for (computer, choice, x, y, suffix) in [
        (1, Some(0), 6, 27, "_PokeComClub"),
        (1, Some(1), 6, 27, "_MobileCenter"),
        (1, Some(2), 6, 27, "_Quit"),
        (2, Some(0), 6, 29, "_UsePhone"),
        (2, Some(1), 6, 29, "_DontUsePhone"),
        (2, Some(2), 6, 29, "_Quit"),
        (3, None, 3, 27, ""),
    ] {
        let mut shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().unwrap(),
                map_name: MAP.into(),
                tile_x: x,
                tile_y: y,
            },
            BevyShellConfig::default(),
        )
        .unwrap();
        shell.shell.set_trainer_identity("CHRIS", 1).unwrap();
        let mut controller = VisibleShellController { shell };
        settle_visible_shell_until_input(&mut controller.shell).unwrap();
        controller.press(GameButton::Up).unwrap();
        assert_eq!(
            controller.snapshot().unwrap().overworld.tile,
            TilePosition::new(x, y),
            "face the actual terminal from its unobstructed customer side"
        );
        assert_eq!(
            controller.shell.shell.session().overworld().player.facing,
            Direction::Up
        );
        if let Some(directory) = std::env::var_os("KANTO_INTERIOR_STATE_REVIEW_DIR") {
            let directory = PathBuf::from(directory).join("pokecom-office");
            std::fs::create_dir_all(&directory).unwrap();
            controller
                .save(directory.join(format!("computer{computer}{suffix}.crystalsave")))
                .unwrap();
        }
        let script = format!("{MAP}Computer{computer}");
        let intro = format!("{script}Text");
        controller.press(GameButton::A).unwrap();
        if let Some(choice) = choice {
            let menu_id = format!(".Computer{computer}MenuHeader@{script}");
            read_pages(&mut controller, &intro, Some(&menu_id));
            let menu = controller.snapshot().unwrap().ui.menu.unwrap();
            assert!(
                menu.layout
                    .vertical_menus
                    .iter()
                    .any(|menu| menu.options.len() == 3)
            );
            let expected = if computer == 1 {
                ["POKECOM CLUB", "MOBILE CENTER", "QUIT"]
            } else {
                ["USING PHONE", "NO CONNECTION", "QUIT"]
            };
            let vertical = selected_vertical_menu(&menu, &controller.shell.menu_cursor).unwrap();
            assert_eq!(
                vertical
                    .options
                    .iter()
                    .map(|option| crate::visible_script_menu_option_label(&menu, option))
                    .collect::<Vec<_>>(),
                expected,
                "actual packed source choices need readable labels without changing selection identity"
            );
            for _ in 0..choice {
                controller.press(GameButton::Down).unwrap();
            }
            controller.press(GameButton::A).unwrap();
            if choice < 2 {
                read_pages(&mut controller, &format!("{intro}{suffix}"), Some(&menu_id));
                controller.press(GameButton::B).unwrap();
            }
            settle_visible_shell_until_input(&mut controller.shell).unwrap();
        } else {
            read_pages(&mut controller, &intro, None);
        }
        let closed = controller.snapshot().unwrap();
        assert!(
            closed.ui.menu.is_none()
                && !closed.ui.window_open
                && controller.shell.active_script_cursor.is_none(),
            "terminal must release input: {computer}{suffix}"
        );
        controller.press(GameButton::Start).unwrap();
        assert!(
            controller.snapshot().unwrap().ui.menu.is_some(),
            "production Start must reopen"
        );
        controller.press(GameButton::B).unwrap();
        let target = TilePosition::new(x - 1, y);
        for _ in 0..16 {
            if controller.snapshot().unwrap().overworld.tile == target {
                break;
            }
            controller.press(GameButton::Left).unwrap();
        }
        assert_eq!(
            controller.snapshot().unwrap().overworld.tile,
            target,
            "walk away through ordinary input"
        );
        println!(
            "office computer{computer}{suffix}: all authored pages, source menu/branch, cancel, Start and walking passed"
        );
    }
}
