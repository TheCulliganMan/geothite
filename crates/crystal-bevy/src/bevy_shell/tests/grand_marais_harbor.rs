//! Explicit local Mew/Surf/Fog/bank fixture; all later travel uses production
//! controller input, including native Max Repel consumption from the initial bag.
//! This does not establish earned regional Surf progression.
use crystal_runtime::RuntimeShellSnapshot;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::*;

const TOWN: &str = "MnGrandMarais";
const REPEL: &str = "MAX_REPEL";

fn repel_steps(c: &VisibleShellController) -> u16 {
    c.shell.shell.session().state().repel_steps_remaining
}

fn repel_quantity(c: &mut VisibleShellController) -> u16 {
    c.snapshot()
        .expect("actual native Max Repel inventory")
        .bag
        .items
        .iter()
        .filter(|item| item.item_id == REPEL)
        .map(|item| item.quantity)
        .sum()
}

fn menu_press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("native Pack input");
    c.wait_frames(16).expect("native Pack presentation");
}

fn menu_cursor(
    c: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..32 {
        let selected = index(c);
        if selected == target {
            return;
        }
        menu_press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native Pack cursor did not reach {target}");
}

pub(in crate::bevy_shell::tests) fn use_max_repel(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
) {
    settle(c);
    assert_eq!(r.data().items[REPEL].repel_steps, Some(250));
    assert_eq!(repel_steps(c), 0);
    let before_quantity = repel_quantity(c);
    assert!(before_quantity > 0, "explicit initial Max Repel available");
    let party = c.snapshot().expect("party before native item use").party;
    menu_press(c, GameButton::Start);
    let snapshot = c.snapshot().expect("production Start menu");
    let target = super::super::super::visible_start_menu_options(&c.shell, &snapshot)
        .iter()
        .position(|option| *option == super::super::super::StartMenuOption::Pack)
        .expect("native PACK option");
    menu_cursor(c, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    menu_press(c, GameButton::A);
    let target = c
        .snapshot()
        .expect("initial Max Repel bag")
        .bag
        .items
        .iter()
        .filter(|item| item.quantity > 0)
        .position(|item| item.item_id == REPEL)
        .expect("initial Max Repel item");
    menu_cursor(c, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("Pack cursor")
            .option_index
    });
    menu_press(c, GameButton::A);
    let snapshot = c.snapshot().expect("native Max Repel actions");
    let actions = super::super::super::visible_selected_pack_item_actions(
        &snapshot,
        &c.shell,
        &super::super::super::FieldPackPocket::Items,
        false,
    )
    .expect("native field item actions");
    let target = actions
        .iter()
        .position(|action| *action == super::super::super::FieldPackAction::Use)
        .expect("native USE action");
    menu_cursor(c, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action cursor")
            .option_index
    });
    menu_press(c, GameButton::A);
    let snapshot = c.snapshot().expect("native Max Repel use notice");
    let expected = snapshot
        .presentation
        .asm_text_for_label("_ItemUsedText")
        .expect("original native item-use text")
        .replace("<PLAYER>", &snapshot.trainer.player_name)
        .replace(
            "<STRING_BUFFER_2>",
            &format!("\n{}", r.data().items[REPEL].name.replace('_', " ")),
        );
    assert_eq!(c.shell.field_notice.as_deref(), Some(expected.as_str()));
    assert!(!expected.contains('<'), "native text buffers resolved");
    assert_eq!(repel_steps(c), 250);
    assert_eq!(repel_quantity(c), before_quantity - 1);
    assert_eq!(
        c.shell.shell.session().state().active_repel_item.as_deref(),
        Some(REPEL)
    );
    assert_eq!(c.snapshot().expect("item preserves partner").party, party);
    let position = at(c);
    menu_press(c, GameButton::Down);
    assert_eq!(at(c), position, "item-use notice owns movement");
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("native item-use menu continuation");
        if c.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
            c.wait_frames(120).expect("print native item-use page");
            menu_press(c, GameButton::A);
        } else if snapshot.ui.menu.is_some()
            || c.shell.field_pack_target_mode.is_some()
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.bag_cursor.is_some()
            || c.shell.field_pack_pocket.is_some()
            || c.shell.start_menu_cursor.is_some()
        {
            menu_press(c, GameButton::B);
        } else {
            settle(c);
            assert_eq!(repel_steps(c), 250, "menus are not travel steps");
            return;
        }
    }
    panic!("native Max Repel notice/menu did not release input");
}

fn harbor_step(
    c: &mut VisibleShellController,
    button: GameButton,
    map: &str,
    point: Point,
    context: &str,
) -> Vec<String> {
    let before = repel_steps(c);
    assert!(before > 0, "native Max Repel must remain active: {context}");
    let text = super::step(c, button, map, point, context);
    assert_eq!(
        repel_steps(c),
        before - 1,
        "one actual harbor step: {context}"
    );
    text
}

fn assert_resources(before: &RuntimeShellSnapshot, after: &RuntimeShellSnapshot, context: &str) {
    assert_eq!(before.party, after.party, "{context}: party");
    assert_eq!(before.bag, after.bag, "{context}: bag");
    assert_eq!(
        before.progression.badges, after.progression.badges,
        "{context}: badges"
    );
    assert_eq!(
        before.progression.active_event_flags, after.progression.active_event_flags,
        "{context}: event flags"
    );
    assert_eq!(
        before.progression.active_engine_flags, after.progression.active_engine_flags,
        "{context}: engine flags"
    );
}

fn assert_native_row(r: &CrystalRuntime, c: &mut VisibleShellController) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/connections/grand-marais-harbor-closure-v01/closure.contract.json");
    let bytes = std::fs::read(path).expect("frozen accepted harbor contract");
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "2ae0af459e1a6c63bc8930646ba5fc12bb3071b9de62445daa631820d09c6cb3"
    );
    let contract: Value = serde_json::from_slice(&bytes).expect("accepted harbor source contract");
    let patches = contract["cell_patches"]
        .as_array()
        .expect("exact source cells");
    assert_eq!(patches.len(), 20);
    for (x, cell) in patches.iter().enumerate() {
        assert_eq!(cell["metatile"], serde_json::json!([x, 18]));
        let expected = u16::try_from(cell["after_block"].as_u64().expect("native byte block"))
            .expect("native block id");
        assert_eq!(
            r.data().maps[TOWN].blocks[360 + x],
            expected,
            "actual published town cell{x}"
        );
        assert_eq!(
            c.shell
                .shell
                .session()
                .overworld()
                .map
                .metatile_at(i16::try_from(x).expect("town metatile x"), 18),
            Some(expected),
            "actual active town row must retain the published buoy recipe"
        );
    }
    for x in 0..40 {
        assert_eq!(collision(r, c, (x, 37)), "BUOY", "outer native lane{x}");
    }
    for x in 1..=38 {
        assert_eq!(collision(r, c, (x, 36)), "WATER", "inner native lane{x}");
    }
    assert_eq!(collision(r, c, (0, 36)), "BUOY", "west turn-up cap");
    assert_eq!(collision(r, c, (39, 36)), "BUOY", "east turn-up cap");
}

fn blocked(c: &mut VisibleShellController, button: GameButton, point: Point, context: &str) {
    assert_at(c, TOWN, point, context);
    let before = c.snapshot().expect("blocked Surf preimage");
    let remaining = repel_steps(c);
    assert!(remaining > 0, "Max Repel remains active during buoy probes");
    assert_eq!(before.overworld.mode, MovementMode::Surf);
    // The first press may only change facing. Further presses must still fail.
    for _ in 0..3 {
        c.press(button)
            .expect("actual controller attempt against visible buoy");
        c.wait_frames(16)
            .expect("native rejected movement presentation");
        settle(c);
        assert_at(c, TOWN, point, context);
        let after = c.snapshot().expect("native blocked Surf result");
        assert_eq!(
            after.overworld.mode,
            MovementMode::Surf,
            "{context}: Surf retained"
        );
        assert_resources(&before, &after, context);
        assert_eq!(repel_steps(c), remaining, "blocked movement is not travel");
    }
}

fn probe_cap_from_north(c: &mut VisibleShellController, inner: i16, outer: i16) {
    let (outward, inward) = if outer < inner {
        (GameButton::Left, GameButton::Right)
    } else {
        (GameButton::Right, GameButton::Left)
    };
    for (button, point) in [(GameButton::Up, (inner, 35)), (outward, (outer, 35))] {
        harbor_step(
            c,
            button,
            TOWN,
            point,
            "ordinary Water approach above turn-up cap",
        );
    }
    blocked(
        c,
        GameButton::Down,
        (outer, 35),
        "turn-up cap also rejects northern approach",
    );
    for (button, point) in [(inward, (inner, 35)), (GameButton::Down, (inner, 36))] {
        harbor_step(
            c,
            button,
            TOWN,
            point,
            "ordinary Water return from cap probe",
        );
    }
}

fn capture(c: &mut VisibleShellController, label: &str) {
    if let Some(directory) = std::env::var_os("CRYSTAL_GRAND_MARAIS_CAPTURE_SAVES_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("ignored native harbor capture saves");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        c.save(directory.join(format!(
            "grand-marais-{label}-{}-{nonce}.crystalsave",
            std::process::id()
        )))
        .expect("production save for optional native GPU capture");
    }
}

#[test]
#[ignore = "requires the published Grand Marais buoy revision in CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK"]
fn grand_marais_harbor_controller_rejects_all_south_lanes_continues_and_dismounts() {
    assert!(
        std::env::var_os("CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK").is_some(),
        "choose the completed actual native buoy pack explicitly"
    );
    // The reused helper sets Fog and a legal Surf-capable Mew during setup.
    // Its optional booleans are Plain/Rising/Waterfall, all unnecessary here.
    let (root, r, mut c) = fixture_with_items(TOWN, (2, 16), false, false, false, &[(REPEL, 2)]);
    assert_native_row(&r, &mut c);
    assert_eq!(
        repel_quantity(&mut c),
        2,
        "original explicit harbor inventory"
    );
    use_max_repel(&mut c, &r);
    // Resource baseline follows the one real production item consumption.
    let initial = c.snapshot().expect("initial legal native bank");
    assert!(
        initial
            .progression
            .active_engine_flags
            .contains("ENGINE_FOGBADGE")
    );
    capture(&mut c, "bank-before-surf");
    mount(&mut c, GameButton::Left, TOWN, (1, 16));
    for y in 17..=36 {
        harbor_step(
            &mut c,
            GameButton::Down,
            TOWN,
            (1, y),
            "ordinary west harbor Water approach",
        );
        assert_eq!(
            c.snapshot().expect("native harbor Surf").overworld.mode,
            MovementMode::Surf
        );
    }
    blocked(
        &mut c,
        GameButton::Left,
        (1, 36),
        "west buoy turn-up cap closes the corner",
    );
    probe_cap_from_north(&mut c, 1, 0);
    capture(&mut c, "west-cap-surf");
    let mut continued = false;
    for x in 1..=38 {
        if x > 1 {
            harbor_step(
                &mut c,
                GameButton::Right,
                TOWN,
                (x, 36),
                "real inner harbor Water lane",
            );
        }
        blocked(
            &mut c,
            GameButton::Down,
            (x, 36),
            "south buoy rejects native Surf",
        );
        if x == 20 {
            capture(&mut c, "south-line-surf-before-continue");
            let save = std::env::temp_dir().join(format!(
                "grand-marais-harbor-{}-{}.crystalsave",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos()
            ));
            let before = c.snapshot().expect("native harbor save preimage");
            let saved_repel = repel_steps(&c);
            let saved_active_repel = c.shell.shell.session().state().active_repel_item.clone();
            c.save(&save)
                .expect("production Save while Surfing above buoy line");
            c = VisibleShellController::load_save(root.clone(), r.clone(), save.clone(), None)
                .expect("production Continue in actual harbor Water");
            let after = c.snapshot().expect("continued native harbor");
            assert_resources(&before, &after, "harbor Continue preserves resources/flags");
            assert_eq!(repel_steps(&c), saved_repel);
            assert_eq!(
                c.shell.shell.session().state().active_repel_item,
                saved_active_repel
            );
            assert_eq!(before.overworld.map_name, after.overworld.map_name);
            assert_eq!(before.overworld.tile, after.overworld.tile);
            assert_eq!(before.overworld.mode, after.overworld.mode);
            assert_eq!(before.overworld.facing, after.overworld.facing);
            assert_at(
                &mut c,
                TOWN,
                (20, 36),
                "Continue retains actual harbor Surf position",
            );
            assert_native_row(&r, &mut c);
            blocked(
                &mut c,
                GameButton::Down,
                (20, 36),
                "continued buoy line remains blocking",
            );
            std::fs::remove_file(save).expect("remove only this temporary gameplay save");
            continued = true;
        }
    }
    assert!(continued, "a real harbor Save and Continue is required");
    blocked(
        &mut c,
        GameButton::Right,
        (38, 36),
        "east buoy turn-up cap closes the corner",
    );
    probe_cap_from_north(&mut c, 38, 39);
    capture(&mut c, "east-cap-surf");
    for x in (1..38).rev() {
        harbor_step(
            &mut c,
            GameButton::Left,
            TOWN,
            (x, 36),
            "ordinary harbor return along inner lane",
        );
    }
    for y in (16..36).rev() {
        harbor_step(
            &mut c,
            GameButton::Up,
            TOWN,
            (1, y),
            "ordinary harbor return to original bank",
        );
    }
    harbor_step(
        &mut c,
        GameButton::Right,
        TOWN,
        (2, 16),
        "real Surf dismount onto unchanged native bank",
    );
    let returned = c.snapshot().expect("actual dry bank return");
    assert_eq!(returned.overworld.mode, MovementMode::Normal);
    assert_resources(
        &initial,
        &returned,
        "harbor round trip grants no resources or progress",
    );
    assert!(
        repel_steps(&c) > 0,
        "one native Max Repel covers the real harbor course"
    );
    assert_eq!(
        repel_quantity(&mut c),
        1,
        "only one native Max Repel consumed"
    );
    capture(&mut c, "bank-after-dismount");
}
