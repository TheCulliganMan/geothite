// The web form queues edits for the Rust game loop. Authentication IDs are never editable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PlayerCustomization {
    name: String,
    handle: String,
    sprite: u8,
}

impl PlayerCustomization {
    fn validate(&self) -> Result<()> {
        anyhow::ensure!(!self.name.is_empty() && self.name.len() <= 8
            && self.name.trim() == self.name
            && self.name.bytes().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || b" .'-".contains(&c)),
            "Trainer name must be 1–8 uppercase letters, numbers, spaces, periods, apostrophes or hyphens.");
        anyhow::ensure!(!self.handle.is_empty() && self.handle.len() <= 24
            && self.handle.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'),
            "Handle must be 1–24 letters, numbers or underscores.");
        anyhow::ensure!(self.sprite <= 1, "Choose Chris or Kris.");
        Ok(())
    }
}

#[derive(Default)]
struct CustomizationBridge {
    enabled: bool,
    open: bool,
    request_open: bool,
    pending: Option<PlayerCustomization>,
    profile: Option<PlayerCustomization>,
    error: Option<String>,
    saved: bool,
    can_edit: bool,
}

thread_local! {
    static CUSTOMIZATION: std::cell::RefCell<CustomizationBridge> = std::cell::RefCell::new(CustomizationBridge::default());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_customization_open() {
    CUSTOMIZATION.with_borrow_mut(|bridge| { bridge.request_open = true; bridge.error = None; bridge.saved = false; });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_customization_close() {
    CUSTOMIZATION.with_borrow_mut(|bridge| { bridge.open = false; bridge.pending = None; });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_customization_save(json: &str) -> std::result::Result<(), String> {
    if json.len() > 256 { return Err("Profile is too large".into()); }
    let profile: PlayerCustomization = serde_json::from_str(json).map_err(|error| error.to_string())?;
    profile.validate().map_err(|error| error.to_string())?;
    CUSTOMIZATION.with_borrow_mut(|bridge| {
        if !bridge.enabled || !bridge.open || bridge.pending.is_some() { return Err("Open Personalization before saving.".into()); }
        bridge.pending = Some(profile);
        bridge.error = None;
        bridge.saved = false;
        Ok(())
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_customization_poll() -> String {
    CUSTOMIZATION.with_borrow(|bridge| serde_json::json!({
        "enabled": bridge.enabled, "open": bridge.open, "can_edit": bridge.can_edit,
        "profile": bridge.profile, "error": bridge.error, "saved": bridge.saved,
        "pending": bridge.pending.is_some(),
    }).to_string())
}

fn customization_is_open() -> bool { CUSTOMIZATION.with_borrow(|bridge| bridge.open) }

#[cfg(target_arch = "wasm32")]
fn customization_storage() -> Result<(web_sys::Storage, String)> {
    let window = web_sys::window().context("Browser is unavailable")?;
    let storage = window.local_storage().map_err(|_| anyhow::anyhow!("Profile storage is unavailable"))?
        .context("Profile storage is unavailable")?;
    let session = window.session_storage().map_err(|_| anyhow::anyhow!("Player session is unavailable"))?
        .context("Player session is unavailable")?;
    let identity = session.get_item("crystal.multiplayer.player_id").ok().flatten().unwrap_or_else(|| "local".into());
    Ok((storage, format!("geothite.profile.{identity}")))
}

fn stored_customization() -> Option<PlayerCustomization> {
    #[cfg(target_arch = "wasm32")]
    {
        let (storage, key) = customization_storage().ok()?;
        let profile: PlayerCustomization = serde_json::from_str(&storage.get_item(&key).ok()??).ok()?;
        profile.validate().ok()?;
        Some(profile)
    }
    #[cfg(not(target_arch = "wasm32"))]
    { None }
}

fn persist_customization(profile: &PlayerCustomization) -> Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let (storage, key) = customization_storage()?;
        storage.set_item(&key, &serde_json::to_string(profile)?)
            .map_err(|_| anyhow::anyhow!("Unable to save your profile. Check browser storage space."))?;
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = profile;
    Ok(())
}

fn customization_can_edit(runtime: &BevyRuntimeShell, multiplayer: Option<&MultiplayerRuntime>) -> Result<bool> {
    let snapshot = runtime.shell.snapshot()?;
    let safe = !snapshot.trainer.player_name.is_empty()
        && visible_quick_save_blockers(runtime, &snapshot, false, false, false)
            .iter().all(|reason| *reason == "start_menu");
    let online_safe = multiplayer.is_none_or(|online| !online.failed && online.session.is_none()
        && online.queued_mode.is_none() && online.direct_mode.is_none() && online.pending_interaction.is_none());
    Ok(safe && online_safe)
}

fn apply_player_customization(
    mut runtime: ResMut<BevyRuntimeShell>,
    mut multiplayer: Option<NonSendMut<MultiplayerRuntime>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut elapsed: Local<f32>,
) {
    *elapsed += time.delta_seconds();
    let urgent = CUSTOMIZATION.with_borrow(|bridge| bridge.request_open || bridge.pending.is_some());
    if !urgent && *elapsed < 0.1 {
        if customization_is_open() { keys.reset_all(); }
        return;
    }
    *elapsed = 0.0;
    let enabled = runtime.shell.runtime().data().player_customization;
    let can_edit = enabled && customization_can_edit(&runtime, multiplayer.as_deref()).unwrap_or(false);
    CUSTOMIZATION.with_borrow_mut(|bridge| {
        bridge.enabled = enabled;
        bridge.can_edit = can_edit;
        if !bridge.open {
            let state = runtime.shell.session().state();
            let handle = multiplayer.as_ref().map(|online| online.config.display_name.clone())
                .or_else(|| stored_customization().map(|profile| profile.handle)).unwrap_or_else(|| "PLAYER".into());
            bridge.profile = Some(PlayerCustomization { name: state.player_name.clone(), handle, sprite: state.player_gender });
        }
        if bridge.request_open {
            bridge.request_open = false;
            if can_edit {
                runtime.start_menu_cursor = None;
                bridge.open = true;
                mark_runtime_presentation_dirty(&mut runtime);
            } else { bridge.error = Some("Return to the overworld and finish any dialogue or online interaction first.".into()); }
        }
        if bridge.open { keys.reset_all(); }
        let Some(profile) = bridge.pending.take() else { return };
        let result = (|| -> Result<()> {
            anyhow::ensure!(can_edit, "Finish the current dialogue or online interaction before saving.");
            profile.validate()?;
            let before = runtime.shell.session().state().clone();
            let path = runtime.quick_save_path.clone().context("No save slot is available")?;
            // Validation and pack checks run through the same runtime commands as the naming flow.
            let update = (|| -> Result<()> {
                runtime.shell.set_trainer_identity(profile.name.clone(), before.player_id)?;
                runtime.shell.set_player_gender(profile.sprite)?;
                persist_customization(&profile)?;
                runtime.shell.save(&path)?;
                Ok(())
            })();
            if let Err(error) = update {
                if let Some(old) = bridge.profile.as_ref() { let _ = persist_customization(old); }
                runtime.shell.session.state = before;
                mark_runtime_snapshot_dirty(&mut runtime);
                return Err(error);
            }
            if let Some(online) = multiplayer.as_mut() {
                online.config.display_name = profile.handle.clone();
                online.last_profile = None;
                online.last_presence = None;
            }
            mark_runtime_snapshot_dirty(&mut runtime);
            Ok(())
        })();
        match result {
            Ok(()) => { bridge.profile = Some(profile); bridge.saved = true; bridge.error = None; }
            Err(error) => { bridge.error = Some(error.to_string()); bridge.saved = false; }
        }
    });
}

#[cfg(test)]
mod customization_tests {
    use super::*;
    #[test]
    fn profile_rejects_invalid_names_handles_and_sprites() {
        let mut profile = PlayerCustomization { name: "CHRIS".into(), handle: "chris_24".into(), sprite: 1 };
        assert!(profile.validate().is_ok());
        profile.sprite = 2; assert!(profile.validate().is_err()); profile.sprite = 0;
        profile.handle = "space name".into(); assert!(profile.validate().is_err());
        profile.handle = "ok".into(); profile.name = "NINECHARS".into(); assert!(profile.validate().is_err());
        profile.name = "<script>".into(); assert!(profile.validate().is_err());
    }
    #[test]
    fn profile_bridge_requires_open_menu_and_bounds_pending_edits() {
        CUSTOMIZATION.with_borrow_mut(|bridge| *bridge = CustomizationBridge::default());
        let json = r#"{"name":"KRIS","handle":"kris","sprite":1}"#;
        assert!(crystal_customization_save(json).is_err());
        CUSTOMIZATION.with_borrow_mut(|bridge| { bridge.enabled = true; bridge.open = true; });
        assert!(crystal_customization_save(json).is_ok());
        assert!(crystal_customization_save(json).is_err());
        crystal_customization_close();
        assert!(!customization_is_open());
        CUSTOMIZATION.with_borrow_mut(|bridge| *bridge = CustomizationBridge::default());
    }
}
