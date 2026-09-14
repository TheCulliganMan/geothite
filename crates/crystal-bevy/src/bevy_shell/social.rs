// Browser social controls use the game's authenticated socket, including during a link session.
#[derive(Default)]
struct SocialBridge {
    pending: VecDeque<crystal_net::hosted::ClientMessage>,
    events: VecDeque<crystal_net::hosted::ServerMessage>,
    connected: bool,
    focused: bool,
    players: Vec<serde_json::Value>,
    selected_player: Option<String>,
}

thread_local! {
    static SOCIAL_BRIDGE: std::cell::RefCell<SocialBridge> = std::cell::RefCell::new(SocialBridge::default());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_social_focus(focused: bool) {
    SOCIAL_BRIDGE.with_borrow_mut(|bridge| bridge.focused = focused);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_social_send(json: &str) -> std::result::Result<(), String> {
    if json.len() > 2048 {
        return Err("Social command is too large".into());
    }
    let message: crystal_net::hosted::ClientMessage =
        serde_json::from_str(json).map_err(|e| e.to_string())?;
    if !matches!(
        &message,
        crystal_net::hosted::ClientMessage::Chat { .. }
            | crystal_net::hosted::ClientMessage::ChatJoin { .. }
            | crystal_net::hosted::ClientMessage::ChatLeave { .. }
            | crystal_net::hosted::ClientMessage::InteractionRequest { .. }
            | crystal_net::hosted::ClientMessage::InteractionResponse { .. }
            | crystal_net::hosted::ClientMessage::InteractionCancel
    ) {
        return Err("Unsupported social command".into());
    }
    SOCIAL_BRIDGE.with_borrow_mut(|bridge| {
        if !bridge.connected {
            return Err("Multiplayer is reconnecting".into());
        }
        if bridge.pending.len() >= 32 {
            return Err("Too many pending social commands".into());
        }
        bridge.pending.push_back(message);
        Ok(())
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn crystal_social_poll() -> String {
    SOCIAL_BRIDGE.with_borrow_mut(|bridge| {
        serde_json::json!({
            "connected": bridge.connected,
            "events": bridge.events.drain(..).collect::<Vec<_>>(),
            "players": bridge.players,
            "selected_player": bridge.selected_player.take(),
        })
        .to_string()
    })
}

fn social_event(message: &crystal_net::hosted::ServerMessage) {
    use crystal_net::hosted::ServerMessage;
    if !matches!(
        message,
        ServerMessage::Welcome { .. }
            | ServerMessage::Chat { .. }
            | ServerMessage::ChatChannels { .. }
            | ServerMessage::InteractionRequest { .. }
            | ServerMessage::InteractionResponse { .. }
            | ServerMessage::MatchFound { .. }
            | ServerMessage::Error { .. }
    ) {
        return;
    }
    SOCIAL_BRIDGE.with_borrow_mut(|bridge| {
        if bridge.events.len() >= 200 {
            bridge.events.pop_front();
        }
        bridge.events.push_back(message.clone());
    });
}

impl MultiplayerRuntime {
    // Invitations must update the same lobby state as keyboard requests before
    // MatchFound arrives. Sending them directly would reject a valid match.
    fn prepare_social_interaction(
        &mut self,
        message: &crystal_net::hosted::ClientMessage,
        runtime_shell: &mut BevyRuntimeShell,
    ) -> Result<()> {
        use crystal_net::hosted::{ClientMessage, ServerMessage};
        match message {
            ClientMessage::InteractionRequest { target_user_id, kind } => {
                anyhow::ensure!(self.connection.is_some() && self.session.is_none() && self.queued_mode.is_none() && self.direct_mode.is_none() && self.pending_interaction.is_none(), "Finish or cancel your current invitation or link session first.");
                if let Some(reason) = direct_interaction_block_reason(runtime_shell.shell.session().state()) { anyhow::bail!(reason); }
                anyhow::ensure!(!has_visible_shell_a_action(runtime_shell)?, "Close the current dialogue or menu first.");
                let map = runtime_shell.shell.session().snapshot().map_name;
                anyhow::ensure!(self.remote_presences.get(target_user_id).is_some_and(|player| player.map == map), "That trainer is no longer nearby.");
                self.direct_mode = Some(*kind);
            }
            ClientMessage::InteractionResponse { request_id, target_user_id, accepted } => {
                let request = self.pending_interaction.as_ref().context("That invitation has expired.")?;
                anyhow::ensure!(request.request_id == *request_id && request.from_user_id == *target_user_id, "That invitation has expired.");
                let kind = request.kind;
                if *accepted {
                    if let Some(reason) = direct_interaction_block_reason(runtime_shell.shell.session().state()) { anyhow::bail!(reason); }
                    anyhow::ensure!(!has_visible_shell_a_action(runtime_shell)?, "Close the current dialogue or menu first.");
                }
                self.direct_mode = accepted.then_some(kind);
                self.pending_interaction = None;
                social_event(&ServerMessage::InteractionResponse { request_id: *request_id, from_user_id: target_user_id.clone(), accepted: *accepted });
            }
            ClientMessage::InteractionCancel => {
                anyhow::ensure!(self.connection.is_some() && self.session.is_none(), "You are already in a link session.");
                // Keep direct_mode until the server confirms cancellation: an
                // acceptance may already be in flight ahead of our cancel.
            }
            _ => {}
        }
        Ok(())
    }

    fn poll_social(&mut self, runtime_shell: &mut BevyRuntimeShell) -> Result<()> {
        let messages = SOCIAL_BRIDGE.with_borrow_mut(|bridge| {
            if self.failed {
                bridge.connected = false;
                bridge.pending.clear();
            }
            bridge.pending.drain(..).collect::<Vec<_>>()
        });
        for message in messages {
            if let Err(error) = self.prepare_social_interaction(&message, runtime_shell) {
                social_event(&crystal_net::hosted::ServerMessage::Error {
                    code: "social_error".into(), message: error.to_string(),
                });
                continue;
            }
            if let Some(connection) = self.connection.as_mut() {
                connection.send(message)?;
            } else if let Some(session) = self.session.as_mut() {
                session.send_social(message)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod social_bridge_tests {
    use super::*;

    #[test]
    fn social_bridge_rejects_gameplay_commands_and_offline_chat() {
        SOCIAL_BRIDGE.with_borrow_mut(|bridge| *bridge = SocialBridge::default());
        assert!(crystal_social_send(r#"{"type":"queue_leave"}"#).is_err());
        let chat = r#"{"type":"chat","channel":"say","target_user_id":null,"text":"Hello"}"#;
        assert!(crystal_social_send(chat).is_err());
        SOCIAL_BRIDGE.with_borrow_mut(|bridge| bridge.connected = true);
        crystal_social_send(r#"{"type":"interaction_request","target_user_id":"player-2","kind":"battle"}"#).unwrap();
        SOCIAL_BRIDGE.with_borrow_mut(|bridge| bridge.pending.clear());
        crystal_social_send(chat).unwrap();
        assert_eq!(SOCIAL_BRIDGE.with_borrow(|bridge| bridge.pending.len()), 1);
        crystal_social_focus(true);
        assert!(SOCIAL_BRIDGE.with_borrow(|bridge| bridge.focused));
        crystal_social_focus(false);
        assert!(!SOCIAL_BRIDGE.with_borrow(|bridge| bridge.focused));
        SOCIAL_BRIDGE.with_borrow_mut(|bridge| *bridge = SocialBridge::default());
    }
}
