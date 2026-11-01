//! An overwrite answer must wait for the complete source question.
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::{ButtonInput, KeyCode};
use crystal_core::input::GameButton;

use super::super::{
    BevyRuntimeShell, GAME_TICK_SECONDS, RuntimeTickTimer, StartMenuOption, VisibleSaveFlowStage,
    VisibleShellController, scene_dialog_yes_no_active, visible_scene_dialog_entries,
    visible_start_menu_options,
};
use super::integrated_shell_test_app;
use super::winona::{load, press};

#[test]
#[ignore = "requires the external candidate and an isolated CRYSTAL_WINONA_SAVE_DIR"]
fn overhaul_save_overwrite_reads_the_whole_question_before_accepting_an_answer() {
    let (root, runtime) = load();
    let directory =
        PathBuf::from(std::env::var_os("CRYSTAL_WINONA_SAVE_DIR").expect("isolated saves"));
    fs::create_dir_all(&directory).expect("save directory");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = directory.join(format!(
        "overwrite-question-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists());
    let mut c = VisibleShellController::new_game(root, runtime, "CHRIS", Some(path.clone()))
        .expect("production New Game");
    c.set_runtime_journal_enabled(false);
    c.save(&path).expect("existing same-pack save");
    let bytes = fs::read(&path).expect("original save bytes");
    press(&mut c, GameButton::Start);
    let snapshot = c.snapshot().expect("Start menu");
    let options = visible_start_menu_options(&c.shell, &snapshot);
    let save = options
        .iter()
        .position(|option| *option == StartMenuOption::Save)
        .expect("Save row");
    for _ in 0..save {
        press(&mut c, GameButton::Down);
    }
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.save_flow.as_ref().expect("overwrite flow").stage,
        VisibleSaveFlowStage::OverwritePrompt
    );
    assert!(
        c.shell.special_boundary.is_some(),
        "read the existing-file warning first"
    );
    // The native frame loop must start this shell-owned printer itself.
    // Controller settling can initialize it and conceal a missing native tick.
    let mut unheld = integrated_shell_test_app(c.shell);
    unheld.insert_resource(RuntimeTickTimer::new(f64::from(GAME_TICK_SECONDS)));
    for _ in 0..256 {
        unheld.update();
    }
    c = VisibleShellController {
        shell: unheld
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("same production session after unheld native frames"),
    };
    let snapshot = c.snapshot().expect("warning page");
    assert!(!scene_dialog_yes_no_active(&snapshot, &c.shell));
    let warning = visible_scene_dialog_entries(&snapshot, &c.shell)
        .expect("visible existing-file warning")
        .join(" ");
    assert!(warning.contains("There is already a"), "{warning}");
    assert!(
        !warning.contains("overwrite"),
        "read the warning before its final question"
    );
    press(&mut c, GameButton::Down);
    assert_eq!(
        c.shell
            .save_flow
            .as_ref()
            .expect("reading warning")
            .yes_no_index,
        0
    );
    // A held acceleration/acknowledgment may reveal the final question, but
    // cannot also answer it or write the existing save on a later held frame.
    let mut app = integrated_shell_test_app(c.shell);
    app.insert_resource(RuntimeTickTimer::new(f64::from(GAME_TICK_SECONDS)));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyZ);
    for _ in 0..60 {
        app.update();
        assert_eq!(
            app.world()
                .resource::<BevyRuntimeShell>()
                .save_flow
                .as_ref()
                .expect("unanswered save")
                .stage,
            VisibleSaveFlowStage::OverwritePrompt
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear_just_pressed(KeyCode::KeyZ);
    }
    assert_eq!(fs::read(&path).expect("held A does not overwrite"), bytes);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyZ);
    app.update();
    c = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("same production session"),
    };
    for _ in 0..8 {
        if c.shell.special_boundary.is_none() {
            break;
        }
        press(&mut c, GameButton::A);
        assert_eq!(fs::read(&path).expect("unmodified save"), bytes);
    }
    assert!(c.shell.special_boundary.is_none());
    let snapshot = c.snapshot().expect("complete question");
    assert!(scene_dialog_yes_no_active(&snapshot, &c.shell));
    let entries = visible_scene_dialog_entries(&snapshot, &c.shell).expect("visible question");
    assert!(
        entries.join(" ").to_ascii_lowercase().contains("overwrite"),
        "{entries:?}"
    );
    assert_eq!(
        c.shell
            .save_flow
            .as_ref()
            .expect("unanswered overwrite")
            .stage,
        VisibleSaveFlowStage::OverwritePrompt
    );
    press(&mut c, GameButton::Down);
    assert_eq!(
        c.shell
            .save_flow
            .as_ref()
            .expect("NO selection")
            .yes_no_index,
        1
    );
    press(&mut c, GameButton::A);
    assert!(!c.shell.save_menu_open && c.shell.save_flow.is_none());
    assert_eq!(fs::read(&path).expect("declined overwrite"), bytes);
}
