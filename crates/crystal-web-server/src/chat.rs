use super::*;
use std::time::{Duration, Instant};

fn validate_channel(channel: &str) -> Result<(), String> {
    if matches!(channel, "say" | "general" | "trade" | "lfg" | "whisper") {
        return Ok(());
    }
    let name = channel
        .strip_prefix("custom:")
        .ok_or("unknown chat channel")?;
    if name.is_empty()
        || name.len() > 24
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(
            "custom channel names must contain 1–24 lowercase letters, digits or hyphens".into(),
        );
    }
    Ok(())
}

impl Hub {
    pub(super) fn chat_membership(
        &mut self,
        id: Uuid,
        channel: String,
        join: bool,
    ) -> Result<Vec<Delivery>, String> {
        validate_channel(&channel)?;
        if matches!(channel.as_str(), "say" | "whisper") {
            return Err("say and whisper do not require channel membership".into());
        }
        let client = self.clients.get_mut(&id).expect("registered client");
        if join {
            if client.chat_channels.len() >= 12 && !client.chat_channels.contains(&channel) {
                return Err("you may join at most 12 channels".into());
            }
            client.chat_channels.insert(channel);
        } else {
            client.chat_channels.remove(&channel);
        }
        let mut channels = client.chat_channels.iter().cloned().collect::<Vec<_>>();
        channels.sort();
        Ok(vec![deliver(id, ServerMessage::ChatChannels { channels })])
    }

    pub(super) fn chat(
        &mut self,
        id: Uuid,
        channel: String,
        target: Option<String>,
        text: String,
    ) -> Result<Vec<Delivery>, String> {
        validate_channel(&channel)?;
        let text = text.trim();
        if text.is_empty() || text.chars().count() > 280 || text.chars().any(char::is_control) {
            return Err("chat must contain 1–280 characters without control characters".into());
        }
        if (channel == "whisper") != target.is_some() {
            return Err("only whispers require a target player ID".into());
        }
        let sender = self.clients.get(&id).expect("registered client");
        if !matches!(channel.as_str(), "say" | "whisper")
            && !sender.chat_channels.contains(&channel)
        {
            return Err("join that channel before speaking".into());
        }
        let whisper = if let Some(user) = &target {
            let peer = self
                .users
                .get(user)
                .copied()
                .ok_or("whisper target is offline")?;
            if self.clients[&peer].identity.world != sender.identity.world {
                return Err("whisper target is in a different world or modpack".into());
            }
            Some(peer)
        } else {
            None
        };
        if matches!(channel.as_str(), "say" | "general") && sender.presence.is_none() {
            return Err("enter the world before using local chat".into());
        }
        let recipients = self
            .clients
            .iter()
            .filter_map(|(peer_id, peer)| {
                if peer.identity.world != sender.identity.world {
                    return None;
                }
                let receives = match channel.as_str() {
                    "whisper" => *peer_id == id || Some(*peer_id) == whisper,
                    "say" => sender
                        .presence
                        .as_ref()
                        .zip(peer.presence.as_ref())
                        .is_some_and(|(a, b)| presences_are_visible(a, b)),
                    "general" => {
                        peer.chat_channels.contains(&channel)
                            && sender
                                .presence
                                .as_ref()
                                .zip(peer.presence.as_ref())
                                .is_some_and(|(a, b)| a.map == b.map)
                    }
                    _ => peer.chat_channels.contains(&channel),
                };
                receives.then_some(*peer_id)
            })
            .collect::<Vec<_>>();
        let message = ServerMessage::Chat {
            channel,
            from_user_id: sender.identity.user_id.clone(),
            from_display_name: sender.identity.display_name.clone(),
            target_user_id: target,
            text: text.into(),
        };
        let sender = self.clients.get_mut(&id).expect("registered client");
        let now = Instant::now();
        sender
            .chat_sent
            .retain(|sent| now.duration_since(*sent) < Duration::from_secs(5));
        if sender.chat_sent.len() >= 5 {
            return Err("chat is too fast; try again shortly".into());
        }
        sender.chat_sent.push_back(now);
        Ok(recipients
            .into_iter()
            .map(|peer| deliver(peer, message.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Hub, [Uuid; 4]) {
        let mut hub = Hub::default();
        let ids = std::array::from_fn(|_| Uuid::new_v4());
        for (n, id) in ids.iter().enumerate() {
            hub.connect(
                *id,
                ClientIdentity {
                    user_id: format!("player-{n}"),
                    display_name: format!("Trainer {n}"),
                    world: WorldIdentity {
                        world_id: if n == 3 { "other" } else { "world" }.into(),
                        modpack: ModpackIdentity {
                            id: "core".into(),
                            content_hash: "hash".into(),
                        },
                    },
                },
            )
            .unwrap();
            hub.handle(
                *id,
                ClientMessage::Presence {
                    map: if n == 2 { "elsewhere" } else { "town" }.into(),
                    tile_x: 0,
                    tile_y: 0,
                    direction: "down".into(),
                },
            );
        }
        (hub, ids)
    }

    fn send(
        hub: &mut Hub,
        id: Uuid,
        channel: &str,
        target: Option<&str>,
        text: &str,
    ) -> Vec<Delivery> {
        hub.handle(
            id,
            ClientMessage::Chat {
                channel: channel.into(),
                target_user_id: target.map(str::to_owned),
                text: text.into(),
            },
        )
    }

    #[test]
    fn chat_scopes_and_whisper_privacy_are_enforced() {
        let (mut hub, ids) = setup();
        for (channel, expected) in [("say", 2), ("general", 2), ("trade", 3), ("lfg", 3)] {
            let result = send(&mut hub, ids[0], channel, None, "Hello!");
            assert_eq!(result.len(), expected);
            assert!(!result.iter().any(|d| d.connection_id == ids[3]));
            assert!(result.iter().all(|d| matches!(&d.message, ServerMessage::Chat { from_user_id, .. } if from_user_id == "player-0")));
        }
        let whisper = send(&mut hub, ids[0], "whisper", Some("player-2"), "Private");
        assert_eq!(whisper.len(), 2);
        assert!(
            whisper
                .iter()
                .all(|d| [ids[0], ids[2]].contains(&d.connection_id))
        );
        assert!(matches!(
            send(&mut hub, ids[1], "whisper", Some("player-3"), "No")[0].message,
            ServerMessage::Error { .. }
        ));
    }

    #[test]
    fn custom_membership_leave_and_spam_limits_work() {
        let (mut hub, ids) = setup();
        assert!(matches!(
            send(&mut hub, ids[0], "custom:friends", None, "No")[0].message,
            ServerMessage::Error { .. }
        ));
        for id in &ids[..2] {
            hub.handle(
                *id,
                ClientMessage::ChatJoin {
                    channel: "custom:friends".into(),
                },
            );
        }
        assert_eq!(
            send(&mut hub, ids[0], "custom:friends", None, "Hello").len(),
            2
        );
        hub.handle(
            ids[1],
            ClientMessage::ChatLeave {
                channel: "custom:friends".into(),
            },
        );
        assert_eq!(
            send(&mut hub, ids[0], "custom:friends", None, "Hello").len(),
            1
        );
        for _ in 0..3 {
            send(&mut hub, ids[0], "say", None, "Hello");
        }
        assert!(matches!(
            send(&mut hub, ids[0], "say", None, "Too fast")[0].message,
            ServerMessage::Error { .. }
        ));
        for invalid in ["", " \t ", "hello\nworld", &"a".repeat(281)] {
            assert!(matches!(
                send(&mut hub, ids[1], "say", None, invalid)[0].message,
                ServerMessage::Error { .. }
            ));
        }
    }
}
