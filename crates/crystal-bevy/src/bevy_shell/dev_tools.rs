//! Optional presentation diagnostics. These never dispatch gameplay input.

#[cfg(all(not(target_arch = "wasm32"), feature = "dev-diagrams"))]
mod diagrams;

use bevy::{
    dev_tools::{
        fps_overlay::{
            FpsOverlayConfig, FpsOverlayPlugin, FpsOverlaySystems, FrameTimeGraphConfig,
        },
        render_debug::RenderDebugOverlayKeybindings,
    },
    prelude::*,
    ui_render::GlobalUiDebugOptions,
};

pub(super) fn install(app: &mut App) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "dev-diagrams"))]
    diagrams::install(app);
    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            enabled: false,
            text_config: TextFont::from_font_size(18.0),
            frame_time_graph_config: FrameTimeGraphConfig {
                // The graph starts hidden and remains unavailable on WebGL2.
                enabled: false,
                ..default()
            },
            ..default()
        },
    })
    .insert_resource(RenderDebugOverlayKeybindings {
        enable_keybindings: true,
        ..default()
    })
    .add_systems(Update, toggle_overlays.before(FpsOverlaySystems::Customize));
    #[cfg(target_arch = "wasm32")]
    app.add_systems(
        Update,
        sync_browser_fps_visibility.after(FpsOverlaySystems::Customize),
    );
}

// Bevy 0.20's display system requires a graph entity even though WebGL omits
// that entity. Control its public overlay root so the counter still toggles.
#[cfg(any(target_arch = "wasm32", test))]
fn sync_browser_fps_visibility(
    fps: Res<FpsOverlayConfig>,
    mut roots: Query<(&mut Node, &GlobalZIndex)>,
) {
    for (mut node, z_index) in &mut roots {
        if z_index.0 == bevy::dev_tools::fps_overlay::FPS_OVERLAY_ZINDEX {
            node.display = if fps.enabled {
                Display::DEFAULT
            } else {
                Display::None
            };
        }
    }
}

fn toggle_overlays(
    keys: Res<ButtonInput<KeyCode>>,
    mut fps: ResMut<FpsOverlayConfig>,
    mut ui: ResMut<GlobalUiDebugOptions>,
) {
    if keys.just_pressed(KeyCode::F8) {
        fps.enabled = !fps.enabled;
    }
    if cfg!(not(target_arch = "wasm32")) && keys.just_pressed(KeyCode::F9) {
        fps.frame_time_graph_config.enabled = !fps.frame_time_graph_config.enabled;
    }
    if keys.just_pressed(KeyCode::F10) {
        ui.toggle();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_fps_toggles_without_a_graph_and_preserves_other_ui() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(FpsOverlayConfig {
                enabled: false,
                ..default()
            })
            .init_resource::<GlobalUiDebugOptions>()
            .add_systems(
                Update,
                (toggle_overlays, sync_browser_fps_visibility).chain(),
            );
        let fps_root = app
            .world_mut()
            .spawn((
                Node::default(),
                GlobalZIndex(bevy::dev_tools::fps_overlay::FPS_OVERLAY_ZINDEX),
            ))
            .id();
        let game_ui = app
            .world_mut()
            .spawn((Node::default(), GlobalZIndex(1)))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Node>(fps_root).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(game_ui).unwrap().display,
            Display::DEFAULT
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F8);
        app.update();
        assert_eq!(
            app.world().get::<Node>(fps_root).unwrap().display,
            Display::DEFAULT
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F8);
        app.update();
        assert_eq!(
            app.world().get::<Node>(fps_root).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(game_ui).unwrap().display,
            Display::DEFAULT
        );
    }

    #[test]
    fn development_plugins_install_hidden_overlays_and_frame_diagnostics() {
        use bevy::{
            asset::AssetPlugin,
            diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
            render::storage::ShaderBuffer,
            shader::Shader,
            text::{RemSize, TextIterScratch},
        };

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), DiagnosticsPlugin))
            .init_asset::<Shader>()
            .init_asset::<ShaderBuffer>()
            .init_resource::<RemSize>()
            .init_resource::<TextIterScratch>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<GlobalUiDebugOptions>();
        install(&mut app);
        app.update();
        assert!(app.is_plugin_added::<FpsOverlayPlugin>());
        assert!(app.is_plugin_added::<FrameTimeDiagnosticsPlugin>());
        assert!(!app.world().resource::<FpsOverlayConfig>().enabled);
        assert!(
            !app.world()
                .resource::<FpsOverlayConfig>()
                .frame_time_graph_config
                .enabled
        );
        assert!(!app.world().resource::<GlobalUiDebugOptions>().enabled);
        assert!(
            app.world()
                .resource::<RenderDebugOverlayKeybindings>()
                .enable_keybindings
        );
    }

    #[test]
    fn development_hotkeys_toggle_independent_overlays_without_game_buttons() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(FpsOverlayConfig {
                enabled: false,
                ..default()
            })
            .init_resource::<GlobalUiDebugOptions>()
            .add_systems(Update, toggle_overlays);

        // Production game buttons and the existing view/fullscreen/capture
        // shortcuts must leave diagnostic state untouched.
        for key in [
            KeyCode::KeyZ,
            KeyCode::Enter,
            KeyCode::F3,
            KeyCode::F6,
            KeyCode::F11,
        ] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
        }
        app.update();
        assert!(!app.world().resource::<FpsOverlayConfig>().enabled);
        assert!(!app.world().resource::<GlobalUiDebugOptions>().enabled);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        let initial_graph = app
            .world()
            .resource::<FpsOverlayConfig>()
            .frame_time_graph_config
            .enabled;
        for key in [KeyCode::F8, KeyCode::F9, KeyCode::F10] {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
        }
        app.update();
        assert!(app.world().resource::<FpsOverlayConfig>().enabled);
        assert_eq!(
            app.world()
                .resource::<FpsOverlayConfig>()
                .frame_time_graph_config
                .enabled,
            if cfg!(target_arch = "wasm32") {
                initial_graph
            } else {
                !initial_graph
            }
        );
        assert!(app.world().resource::<GlobalUiDebugOptions>().enabled);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert!(app.world().resource::<FpsOverlayConfig>().enabled);
        assert!(app.world().resource::<GlobalUiDebugOptions>().enabled);
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "dev-diagrams"))]
fn observe_controller_flow(
    shell: Option<Res<super::BevyRuntimeShell>>,
    mut flow: ResMut<diagrams::ObservedFlow>,
    output: Res<diagrams::DiagramOutput>,
) {
    use super::RuntimeShellPhase;
    let Some(shell) = shell else {
        return;
    };
    let Some((revision, snapshot)) = shell.cached_snapshot.as_ref() else {
        return;
    };
    if *revision != shell.snapshot_revision {
        return;
    }
    // Read the controller's presentation state only; do not take a new mutable
    // snapshot or insert Bevy States that could become a second dispatcher.
    let state = if shell.intro_screen.is_some() {
        "Intro"
    } else if shell.title_menu.is_some() {
        "TitleMenu"
    } else if !shell.battle_messages.is_empty() || snapshot.ui.text_window_open {
        "Text"
    } else if shell.start_menu_cursor.is_some() {
        "StartMenu"
    } else if snapshot.battle.is_none()
        && super::visible_field_command_entries(snapshot, &shell).is_ok_and(|entries| {
            entries
                .iter()
                .any(|entry| entry.trim_start().starts_with('>'))
        })
    {
        "Menu"
    } else {
        match snapshot.phase {
            RuntimeShellPhase::Overworld => "Overworld",
            RuntimeShellPhase::WildBattle => "WildBattle",
            RuntimeShellPhase::StaticWildBattle => "StaticWildBattle",
            RuntimeShellPhase::TrainerBattle => "TrainerBattle",
            RuntimeShellPhase::Text => "Text",
            RuntimeShellPhase::YesNo => "YesNo",
            RuntimeShellPhase::Menu => "Menu",
            RuntimeShellPhase::Shop => "Shop",
        }
    };
    if flow.observe(state) {
        let result = (|| -> std::io::Result<()> {
            std::fs::create_dir_all(&output.directory)?;
            std::fs::write(
                output.directory.join("observed-state-flow.mmd"),
                flow.mermaid(),
            )?;
            std::fs::write(
                output.directory.join("observed-state-flow.md"),
                format!(
                    "# Observed controller transitions\n\nOnly transitions seen in this session are shown; this is not a complete game state machine.\n\n```mermaid\n{}```\n",
                    flow.mermaid()
                ),
            )
        })();
        if let Err(error) = result {
            warn!("State flow export failed: {error}");
        }
    }
}
