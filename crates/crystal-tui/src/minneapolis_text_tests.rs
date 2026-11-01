//! External-pack regression for a declined and retried native yes/no prompt.

use super::*;

fn call(
    game: &mut RuntimeGameShell,
    renderer: &mut RuntimeTextRenderer,
    name: &str,
    arguments: Value,
) -> Value {
    call_mcp_tool(game, renderer, &mut None, Some(name), &arguments, None)
        .expect("production MCP tool")
}

fn press(game: &mut RuntimeGameShell, renderer: &mut RuntimeTextRenderer, button: &str) {
    call(game, renderer, "press", json!({"button": button}));
}

fn finish_dialogue(game: &mut RuntimeGameShell, renderer: &mut RuntimeTextRenderer) -> String {
    let mut pages = String::new();
    for _ in 0..64 {
        let status = call(game, renderer, "status", json!({}));
        if status["structuredContent"]["dialogue"] == false {
            return pages;
        }
        let observed = call(game, renderer, "observe", json!({}));
        pages.push_str(
            observed["content"][0]["text"]
                .as_str()
                .expect("visible text"),
        );
        press(game, renderer, "a");
    }
    panic!("dialogue did not return control");
}

fn choose(game: &mut RuntimeGameShell, renderer: &mut RuntimeTextRenderer, yes: bool) {
    let label = if yes { "> YES" } else { "> NO" };
    for _ in 0..64 {
        let observed = call(game, renderer, "observe", json!({}));
        if observed["structuredContent"]["prompt"]
            .as_array()
            .expect("prompt lines")
            .iter()
            .any(|line| {
                line["text"]
                    .as_str()
                    .is_some_and(|text| text.trim() == label)
            })
        {
            press(game, renderer, "a");
            return;
        }
        let status = call(game, renderer, "status", json!({}));
        press(
            game,
            renderer,
            if status["structuredContent"]["mode"] == "yesno" {
                if yes { "up" } else { "down" }
            } else {
                "a"
            },
        );
    }
    panic!("displayed choice was never selectable");
}

#[test]
#[ignore = "requires the external Minneapolis pack and an earned enforcer-win save"]
fn mcp_yes_no_highlight_follows_controller_after_decline_and_retry() {
    let pack =
        PathBuf::from(env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external chapter pack"));
    let save = PathBuf::from(
        env::var_os("CRYSTAL_MINNEAPOLIS_ENFORCER_WIN_SAVE").expect("earned enforcer-win save"),
    );
    let root = AssetRoot::new(pack.parent().expect("pack parent"));
    let loaded = read_loaded_verified_compiled_game_pack(pack).expect("verified chapter pack");
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("native runtime");
    let mut game = RuntimeGameShell::load_save(root, runtime, save, None).expect("Continue");
    let mut renderer = RuntimeTextRenderer::default();
    let initial = call(&mut game, &mut renderer, "status", json!({}));
    assert_eq!(
        initial["structuredContent"]["map"],
        "MnMinneapolisDispatchOffice"
    );
    assert_eq!(
        initial["structuredContent"]["position"],
        json!({"x": 9, "y": 9})
    );
    for (direction, steps) in [("right", 2), ("up", 3)] {
        call(
            &mut game,
            &mut renderer,
            "move",
            json!({"direction": direction, "steps": steps}),
        );
    }
    for (yes, expected) in [(false, "The latch is shut."), (true, "The clerk is free.")] {
        press(&mut game, &mut renderer, "right");
        press(&mut game, &mut renderer, "a");
        choose(&mut game, &mut renderer, yes);
        let pages = finish_dialogue(&mut game, &mut renderer);
        assert!(
            pages.contains(expected),
            "selected choice disagreed with display: {pages}"
        );
    }
    let opened = call(
        &mut game,
        &mut renderer,
        "move",
        json!({"direction": "right", "steps": 3}),
    );
    assert_eq!(
        opened["structuredContent"]["position"],
        json!({"x": 14, "y": 6})
    );
    let map = call(&mut game, &mut renderer, "map_info", json!({}));
    assert!(!map.to_string().contains("MN_ICE_CAPTIVE_DISPATCHER"));
    assert_eq!(
        opened["structuredContent"]["money"],
        initial["structuredContent"]["money"]
    );
    assert_eq!(
        opened["structuredContent"]["badges"],
        initial["structuredContent"]["badges"]
    );
}
