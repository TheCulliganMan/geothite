//! Resolve accepted story art through the production native loaders.
use bevy::prelude::{Assets, Image};

use super::super::{load_gbcpal_palette, load_oak_intro_frame, load_sprite_art};
use super::winona::load;

#[test]
#[ignore = "requires revised external pack and isolated native scene saves"]
fn overhaul_new_game_cousin_is_a_brief_family_visit_without_progression() {
    use crystal_core::input::GameButton;

    let (root, runtime) = load();
    let mut controller =
        super::super::VisibleShellController::new_game(root, runtime, "CHRIS", None)
            .expect("production New Game");
    for _ in 0..8 {
        if controller.snapshot().expect("bedroom").overworld.tile.x == 2 {
            break;
        }
        super::winona::press(&mut controller, GameButton::Left);
    }
    super::winona::at(&mut controller, "MnWinonaHome2F", (2, 3));
    super::winona::press(&mut controller, GameButton::A);
    let before = controller.snapshot().expect("actual cousin conversation");
    assert!(before.party.slots.is_empty());
    assert!(before.progression.badges.johto.iter().all(|badge| !badge));
    assert!(
        before
            .ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_ref())
            .is_some_and(|text| text.contains("COUSIN") && text.contains("game"))
    );
    let directory = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR")
            .expect("isolated actual scene capture directory"),
    );
    std::fs::create_dir_all(&directory).expect("capture directory");
    controller
        .save(directory.join("cousin-speaking.crystalsave"))
        .expect("preserve the actual New Game dialogue for native review");
    super::winona::press(&mut controller, GameButton::Right);
    let owned = controller.snapshot().expect("owned input");
    assert_eq!(owned.overworld.tile, before.overworld.tile);
    assert_eq!(owned.overworld.facing, before.overworld.facing);
    let mut pages = Vec::new();
    for _ in 0..16 {
        let snapshot = controller.snapshot().expect("cousin pages");
        let Some(text) = snapshot.ui.text.and_then(|text| text.asm_text) else {
            break;
        };
        pages.push(text);
        super::winona::press(&mut controller, GameButton::A);
    }
    assert_eq!(
        pages.len(),
        2,
        "a brief visit, without the old town-story monologue"
    );
    assert!(pages[1].contains("mom"));
    let after = controller.snapshot().expect("conversation finished");
    assert!(after.party.slots.is_empty());
    assert_eq!(
        after.progression.badges.johto,
        before.progression.badges.johto
    );
    assert!(after.ui.text.is_none());
}

#[test]
#[ignore = "requires the revised cast in CRYSTAL_WINONA_TEST_PACK"]
fn overhaul_story_cast_resolves_accepted_native_art_and_private_classes() {
    let (_, runtime) = load();
    let root = runtime
        .materialize_runtime_files()
        .expect("actual pack art");
    let mut images = Assets::<Image>::default();
    for stem in ["mn_prof_tamarack", "mn_rival_reed"] {
        let art = load_sprite_art(&root, stem, 0, "day", &mut images)
            .expect("private native actor art resolves");
        let palette = load_gbcpal_palette(
            &root
                .runtime_assets()
                .join(format!("gfx/sprites/{stem}.gbcpal")),
        )
        .expect("the actor's own native palette");
        for direction in [&art.down, &art.up, &art.left, &art.right] {
            let standing = images
                .get(&direction.standing.handle)
                .expect("standing image");
            let walking = images
                .get(&direction.walking.as_ref().expect("walking pose").handle)
                .expect("walking image");
            assert_ne!(
                standing.data, walking.data,
                "actual lifted-foot walking pose"
            );
            assert!(
                standing
                    .data
                    .as_ref()
                    .expect("native pixels")
                    .as_chunks::<4>()
                    .0
                    .contains(&[palette[1][0], palette[1][1], palette[1][2], 255])
            );
        }
    }
    let front = load_oak_intro_frame(&root, "battle-trainer:mn_rival_reed", &mut images)
        .expect("adult Reed battle art resolves");
    assert_eq!(
        images.get(&front.handle).expect("battle image").size().x,
        56
    );
    for source in [
        include_str!(
            "../../../../../planning/minnesota/npcs/tamarack-ice/01-tamarack/binding-v1.json"
        ),
        include_str!(
            "../../../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/01-reed.json"
        ),
    ] {
        let binding: serde_json::Value =
            serde_json::from_str(source).expect("accepted cast binding");
        let class = binding["trainer_class"].as_str().expect("private class");
        for actor in binding["actors"].as_array().expect("field appearances") {
            let map = &runtime.data().maps[actor["map"].as_str().expect("map")];
            let object = map
                .objects
                .iter()
                .find(|object| Some(object.script.as_str()) == actor["script"].as_str())
                .expect("existing story actor");
            assert_eq!(object.sprite, format!("SPRITE_{class}"));
            assert_eq!(u64::from(object.x), actor["tile"][0].as_u64().expect("x"));
            assert_eq!(u64::from(object.y), actor["tile"][1].as_u64().expect("y"));
        }
        for id in binding["trainer_ids"].as_array().expect("battle variants") {
            assert_eq!(
                runtime.data().trainers.trainers[id.as_str().expect("trainer identity")]
                    .trainer_class,
                class
            );
        }
    }
}

#[test]
#[ignore = "requires the actual New Game case checkpoint in CRYSTAL_STORY_CAST_CASE_SAVE and a render directory"]
fn overhaul_actual_reed_case_continues_into_a_live_native_battle_capture() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("POKEGEAR_PC_RENDER_DIR").expect("isolated native LCD capture directory"),
    );
    let (_, runtime) = load();
    let root = runtime
        .materialize_runtime_files()
        .expect("actual native cast art");
    let source = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_STORY_CAST_CASE_SAVE")
            .expect("actual same-pack New Game case scene"),
    );
    let mut controller =
        super::super::VisibleShellController::load_save(root, runtime.clone(), source, None)
            .expect("Continue the genuine case exchange");
    assert!(
        controller
            .snapshot()
            .expect("actual case")
            .progression
            .badges
            .johto
            .iter()
            .all(|badge| !badge)
    );
    for _ in 0..64 {
        if controller
            .snapshot()
            .expect("actual conversation")
            .battle
            .is_some()
        {
            break;
        }
        super::winona::press(&mut controller, crystal_core::input::GameButton::A);
    }
    let before = controller.snapshot().expect("native trainer intro");
    assert!(matches!(before.battle.expect("Reed battle").kind,
        crystal_runtime::RuntimeBattleKind::Trainer { ref trainer_id, .. } if trainer_id == "MN_REED_WINONA"));
    let mut app = super::menu_render_test_app(controller.shell);
    app.update();
    super::save_live_battle_canvas_for_test(app.world_mut(), "reed-live-trainer-intro.png");
    assert!(directory.join("reed-live-trainer-intro.png").is_file());
    let mut controller = super::super::VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<super::super::BevyRuntimeShell>()
            .expect("retain the live production controller"),
    };
    super::northstar_story_events::play_reed(&mut controller);
}
