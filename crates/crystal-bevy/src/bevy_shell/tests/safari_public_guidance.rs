//! Public check-in directions and an honest denial without native Surf access.

use crystal_core::input::GameButton;
use crystal_core::world::map::TilePosition;

use super::super::super::{VisibleShellController, visible_field_dialog_pages};
use super::{continue_save, fixture, load, press, save};

fn text(c: &mut VisibleShellController) -> String {
    let snapshot = c.snapshot().expect("actual public notice");
    if let Some(text) = snapshot
        .ui
        .text
        .as_ref()
        .and_then(|text| text.asm_text.as_ref())
    {
        return text.clone();
    }
    visible_field_dialog_pages(&snapshot, &c.shell)
        .expect("native public text owns the field")
        .join("\n")
}

fn close(c: &mut VisibleShellController) {
    for _ in 0..64 {
        if !c.snapshot().expect("notice settling").ui.text_window_open
            && c.shell.field_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(c, GameButton::A);
    }
    assert!(!c.snapshot().expect("notice closed").ui.text_window_open);
    assert!(c.shell.field_notice.is_none());
    assert!(!c.has_pending_script_work());
}

#[test]
#[ignore = "requires enabled Safari review with native public check-in guidance"]
fn safari_public_boards_keep_trails_free_and_continue_the_actual_page() {
    let (root, runtime, _) = load();
    for (name, map, at, facing, first, second) in [
        (
            "public-basecamp-board",
            "MnBoundaryWaters",
            (29, 10),
            GameButton::Left,
            ["SAFARI CHECK-IN", "Ranger: just east."],
            ["Public trails and", "survey stay free."],
        ),
        (
            "public-pine-notice",
            "MnPreservePineEntry",
            (12, 11),
            GameButton::Right,
            ["PRESERVE ENTRY", "Free trail ahead."],
            ["Safari check-in:", "Basecamp ranger."],
        ),
    ] {
        // These two public reading positions are explicit local fixtures.
        // Input below reads the actual native sign, without starting a permit.
        let mut c = fixture(&root, &runtime, map, at);
        let money = c.snapshot().expect("public approach").trainer.money;
        press(&mut c, facing);
        press(&mut c, GameButton::A);
        assert_eq!(text(&mut c), first.join("\n"));
        save(&mut c, &format!("{name}-page-one"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert_eq!(text(&mut c), first.join("\n"));
        press(&mut c, GameButton::Down);
        let held = c.snapshot().expect("public notice owns input");
        assert_eq!(held.overworld.tile, TilePosition::new(at.0, at.1));
        assert!(held.safari.is_none());
        assert_eq!(held.trainer.money, money);
        press(&mut c, GameButton::A);
        assert_eq!(text(&mut c), second.join("\n"));
        save(&mut c, &format!("{name}-page-two"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert_eq!(text(&mut c), second.join("\n"));
        assert!(!text(&mut c).to_ascii_lowercase().contains("closed"));
        save(&mut c, &format!("{name}-page-two-continued"));
        close(&mut c);
        let closed = c.snapshot().expect("free public notice complete");
        assert!(closed.safari.is_none());
        assert_eq!(closed.trainer.money, money);
        save(&mut c, &format!("{name}-closed"));
    }
}

#[test]
#[ignore = "requires enabled Safari review whose parent has no CANOE item"]
fn safari_no_surf_denial_names_an_available_requirement_without_charge() {
    let (root, runtime, _) = load();
    assert!(!runtime.data().items.contains_key("CANOE"));
    let zone = runtime
        .data()
        .safari_zone
        .as_ref()
        .expect("enabled expedition");
    let mut c = fixture(&root, &runtime, &zone.admission_map, (30, 10));
    assert!(
        c.snapshot()
            .expect("no field partner fixture")
            .party
            .slots
            .is_empty()
    );
    let money = c.snapshot().expect("original money").trainer.money;
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    assert!(
        c.snapshot()
            .expect("actual ranger offer")
            .safari_prompt
            .is_some()
    );
    press(&mut c, GameButton::A);
    assert_eq!(text(&mut c), "Bring a SURF\npartner with");
    assert!(!text(&mut c).contains("CANOE"));
    press(&mut c, GameButton::Down);
    let refused = c.snapshot().expect("native Surf refusal");
    assert!(refused.safari.is_none());
    assert_eq!(refused.trainer.money, money);
    assert_eq!(refused.overworld.tile, zone.return_to.tile);
    press(&mut c, GameButton::A);
    assert_eq!(text(&mut c), "partner with\nits badge.");
    close(&mut c);
    save(&mut c, "admission-surf-denial-closed");
}
