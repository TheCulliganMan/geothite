// Save data stays in Rust; the browser only handles files, links, and confirmation UI.
#[derive(Default)]
struct SaveManagementBridge {
    open: bool,
    pending: Option<(String, Vec<u8>)>,
    result: Option<serde_json::Value>,
    exported: Vec<u8>,
}
thread_local! {
    static SAVE_MANAGEMENT: std::cell::RefCell<SaveManagementBridge> = std::cell::RefCell::new(SaveManagementBridge::default());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_save_manager_open() {
    SAVE_MANAGEMENT.with_borrow_mut(|bridge| {
        bridge.open = true;
        bridge.result = None;
    });
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_save_manager_close() {
    SAVE_MANAGEMENT.with_borrow_mut(|bridge| *bridge = SaveManagementBridge::default());
}
fn save_manager_is_open() -> bool {
    SAVE_MANAGEMENT.with_borrow(|bridge| bridge.open)
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_save_manager_request(action: &str, bytes: &[u8]) -> std::result::Result<(), String> {
    if !matches!(
        action,
        "status" | "save" | "export" | "inspect" | "import" | "delete"
    ) {
        return Err("Unknown save action.".into());
    }
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("Save files must be smaller than 4 MB.".into());
    }
    SAVE_MANAGEMENT.with_borrow_mut(|bridge| {
        if !bridge.open || bridge.pending.is_some() {
            return Err("Open Save games and wait for the current action.".into());
        }
        bridge.result = None;
        bridge.exported.clear();
        bridge.pending = Some((action.into(), bytes.to_vec()));
        Ok(())
    })
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_save_manager_poll() -> String {
    SAVE_MANAGEMENT.with_borrow(|bridge| {
        serde_json::json!({
            "pending": bridge.pending.is_some(), "result": bridge.result,
        })
        .to_string()
    })
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_save_manager_take_bytes() -> Vec<u8> {
    SAVE_MANAGEMENT.with_borrow_mut(|bridge| std::mem::take(&mut bridge.exported))
}

fn apply_save_management(
    mut runtime: ResMut<BevyRuntimeShell>,
    multiplayer: Option<NonSend<MultiplayerRuntime>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    SAVE_MANAGEMENT.with_borrow_mut(|bridge| {
        if !bridge.open { return; }
        keys.reset_all();
        let Some((action, bytes)) = bridge.pending.take() else { return; };
        let result = (|| -> Result<serde_json::Value> {
            use crystal_core::save::{read_save_game_for_modpack, read_save_game_bytes_for_modpack, encode_save_game_bytes, erase_save_game};
            let path = runtime.quick_save_path.clone().context("No save slot is available.")?;
            let can_save = customization_can_edit(&runtime, multiplayer.as_deref()).unwrap_or(false);
            let online_safe = multiplayer.as_ref().is_none_or(|online| {
                online.session.is_none() && online.queued_mode.is_none()
                    && online.direct_mode.is_none() && online.pending_interaction.is_none()
            });
            let can_replace = online_safe && (can_save || runtime.title_menu.is_some() || runtime.intro_screen.is_some());
            let current = || read_save_game_for_modpack(&path, runtime.shell.runtime().modpack(), &runtime.shell.runtime().pack_identity().content_hash);
            let summary = |state: &GameState| serde_json::json!({
                "trainer": state.player_name, "sprite": state.player_gender,
                "trainer_id": state.player_id,
            });
            match action.as_str() {
                "status" => Ok(serde_json::json!({"save": current().ok().map(|s| summary(s.state())), "can_save": can_save, "can_replace": can_replace})),
                "save" => {
                    anyhow::ensure!(can_save, "Return to the overworld and finish dialogue or online interactions before saving.");
                    runtime.shell.save(&path)?;
                    mark_runtime_snapshot_dirty(&mut runtime);
                    Ok(serde_json::json!({"message": "Progress saved."}))
                }
                "export" => {
                    let save = current().context("No valid saved game yet. Save your progress first.")?;
                    bridge.exported = encode_save_game_bytes(&save)?;
                    Ok(serde_json::json!({"message": "Saved progress is ready.", "save": summary(save.state())}))
                }
                "inspect" | "import" => {
                    let save = read_save_game_bytes_for_modpack(&bytes, "shared.crystalsave", runtime.shell.runtime().modpack(), &runtime.shell.runtime().pack_identity().content_hash)
                        .context("This save is damaged or belongs to a different game pack.")?;
                    runtime.shell.runtime().validate_save_state_for_runtime_pack(save.state())?;
                    let preview = summary(save.state());
                    if action == "import" {
                        anyhow::ensure!(can_replace, "Return to the title screen or overworld and finish online interactions before restoring.");
                        runtime.shell.runtime().save_game(&path, save.into_state())?;
                    }
                    Ok(serde_json::json!({"save": preview, "reload": action == "import"}))
                }
                "delete" => {
                    anyhow::ensure!(can_replace, "Return to the title screen or overworld and finish online interactions before deleting.");
                    erase_save_game(&path)?;
                    Ok(serde_json::json!({"reload": true}))
                }
                _ => unreachable!(),
            }
        })();
        bridge.result = Some(match result {
            Ok(value) => value,
            Err(error) => serde_json::json!({"error": format!("{error:#}")}),
        });
    });
}
