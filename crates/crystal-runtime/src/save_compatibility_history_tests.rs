//! Completed presentation records may move when a release edits dialogue.

use crystal_core::state::{
    GameState, ItemUseRuntimeEvent, OverworldMemory, ScriptAudioRuntimeEvent,
    ScriptAudioRuntimeKind, ScriptControlRuntimeEvent, ScriptControlRuntimeKind, ScriptEndState,
    ScriptGraphicsRuntimeEvent, ScriptGraphicsRuntimeKind, ScriptMapRuntimeEvent,
    ScriptMapRuntimeKind, ScriptMoneyRuntimeEvent, ScriptMoneyRuntimeKind, ScriptMusicFade,
    ScriptShopRuntimeEvent, ScriptTextRuntimeEvent, ScriptTextRuntimeKind,
};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

use super::normalize_upgrade_checkpoint;

fn completed_conversation() -> GameState {
    let mut state = GameState {
        overworld: OverworldMemory::Active {
            map_name: "MnWinona".into(),
            tile: TilePosition::new(3, 3),
            facing: Direction::Down,
            mode: MovementMode::Normal,
        },
        money: 4321,
        ..GameState::default()
    };
    state.script_runtime.current_music = Some("MUSIC_NEW_BARK_TOWN".into());
    state
        .script_runtime
        .variables
        .insert("regional_progress".into(), "earned".into());
    state
        .script_runtime
        .memory
        .insert("story_receipt".into(), "preserved".into());
    state
        .script_runtime
        .named_buffers
        .insert("STRING_BUFFER_1".into(), "REED".into());
    state.script_runtime.script_ended = Some(ScriptEndState {
        callback: false,
        just_battled_guard: false,
        source_script: "OldNpcScript".into(),
        command_index: 8,
    });
    state
        .script_runtime
        .text_events
        .push(ScriptTextRuntimeEvent {
            command: "writetext".into(),
            kind: ScriptTextRuntimeKind::Write,
            text_label: Some("OldNpcText".into()),
            face_player: false,
            closes_text: false,
            source_script: "OldNpcScript".into(),
            command_index: 4,
        });
    state.script_runtime.active_text_label = Some("OldNpcText".into());
    state
}

#[test]
fn completed_history_is_removed_only_from_the_upgrade_copy() {
    let source = completed_conversation();
    let original_bytes = serde_json::to_vec(&source).expect("source snapshot");
    let upgraded = normalize_upgrade_checkpoint(source.clone()).expect("idle upgrade copy");
    assert_eq!(
        serde_json::to_vec(&source).expect("unchanged source"),
        original_bytes
    );
    let mut expected = serde_json::to_value(&source).expect("expected snapshot");
    expected["script_runtime"]["script_ended"] = serde_json::Value::Null;
    expected["script_runtime"]["text_events"] = serde_json::json!([]);
    expected["script_runtime"]["active_text_label"] = serde_json::Value::Null;
    assert_eq!(
        serde_json::to_value(upgraded).expect("normalized copy"),
        expected
    );
}

#[test]
fn pending_work_is_rejected_instead_of_becoming_completed_history() {
    type PendingWork = (&'static str, fn(&mut GameState));
    let cases: &[PendingWork] = &[
        ("open text", |state| {
            state.script_runtime.text_window_open = true
        }),
        ("sound wait", |state| {
            state.script_runtime.waiting_for_sound_effect = true
        }),
        ("music fade", |state| {
            state.script_runtime.pending_music_fade = Some(ScriptMusicFade {
                audio_id: "MUSIC_NONE".into(),
                fade_frames: 2,
                source_script: "OldNpcScript".into(),
                command_index: 2,
            })
        }),
        ("audio", |state| {
            state
                .script_runtime
                .audio_events
                .push(ScriptAudioRuntimeEvent {
                    command: "playsound".into(),
                    kind: ScriptAudioRuntimeKind::SoundEffect,
                    audio_id: Some("SFX_READ_TEXT_2".into()),
                    fade_frames: None,
                    source_script: "OldNpcScript".into(),
                    command_index: 2,
                })
        }),
        ("graphics", |state| {
            state
                .script_runtime
                .graphics_events
                .push(ScriptGraphicsRuntimeEvent {
                    command: "refreshscreen".into(),
                    kind: ScriptGraphicsRuntimeKind::RefreshSprites,
                    color: None,
                    direction: None,
                    frames: None,
                    source_script: "OldNpcScript".into(),
                    command_index: 2,
                })
        }),
        ("money", |state| {
            state
                .script_runtime
                .money_events
                .push(ScriptMoneyRuntimeEvent {
                    command: "placemoney".into(),
                    kind: ScriptMoneyRuntimeKind::PlaceMoneyTopRight,
                    money: 4321,
                    coins: None,
                    source_script: "OldNpcScript".into(),
                    command_index: 2,
                })
        }),
        ("map", |state| {
            state.script_runtime.map_events.push(ScriptMapRuntimeEvent {
                command: "refreshmap".into(),
                kind: ScriptMapRuntimeKind::RefreshMap,
                target_map: None,
                tile: None,
                facing: None,
                map_setup: None,
                source_script: "OldNpcScript".into(),
                command_index: 2,
            })
        }),
        ("control", |state| {
            state
                .script_runtime
                .control_events
                .push(ScriptControlRuntimeEvent {
                    kind: ScriptControlRuntimeKind::End,
                    target_script: None,
                    source_script: "OldNpcScript".into(),
                    command_index: 8,
                })
        }),
        ("shop", |state| {
            state
                .script_runtime
                .shop_events
                .push(ScriptShopRuntimeEvent {
                    mart_type: "MARTTYPE_STANDARD".into(),
                    mart_id: "MART_CHERRYGROVE".into(),
                    inventory: vec!["POTION".into()],
                    source_script: "OldNpcScript".into(),
                    command_index: 2,
                })
        }),
        ("item use", |state| {
            state
                .script_runtime
                .item_use_events
                .push(ItemUseRuntimeEvent {
                    item_id: "POTION".into(),
                    context: "field".into(),
                    consumed: true,
                })
        }),
    ];
    for (label, install_pending) in cases {
        let mut source = completed_conversation();
        install_pending(&mut source);
        let before = serde_json::to_vec(&source).expect("pending source snapshot");
        assert!(
            normalize_upgrade_checkpoint(source.clone()).is_err(),
            "{label}"
        );
        assert_eq!(
            serde_json::to_vec(&source).expect("pending source retained"),
            before,
            "{label}"
        );
    }
}
