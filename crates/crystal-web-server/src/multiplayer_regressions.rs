use super::*;

fn identity(user: &str) -> ClientIdentity {
    ClientIdentity {
        user_id: user.into(),
        display_name: user.into(),
        world: WorldIdentity {
            world_id: "world".into(),
            modpack: ModpackIdentity {
                id: "core".into(),
                content_hash: "hash".into(),
            },
        },
    }
}

fn pair(mode: MatchMode) -> (Hub, Uuid, Uuid, Uuid) {
    let mut hub = Hub::default();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    hub.connect(a, identity("Alice")).unwrap();
    hub.connect(b, identity("Bob")).unwrap();
    hub.join_queue(a, mode, 1000, 100).unwrap();
    let found = hub.join_queue(b, mode, 1000, 100).unwrap();
    let ServerMessage::MatchFound { session_id, .. } = found[0].message else {
        panic!("match")
    };
    (hub, a, b, session_id)
}

fn presence(x: i32) -> ClientMessage {
    ClientMessage::Presence {
        map: "town".into(),
        tile_x: x,
        tile_y: 0,
        direction: "down".into(),
    }
}

#[test]
fn pending_and_disputed_results_can_be_cancelled_in_every_mode() {
    for mode in [MatchMode::Battle, MatchMode::Trade, MatchMode::TimeCapsule] {
        for disputed in [false, true] {
            let (mut hub, a, b, session) = pair(mode);
            hub.report_result(a, session, MatchOutcome::Local).unwrap();
            if disputed {
                assert!(hub.report_result(b, session, MatchOutcome::Local).is_err());
            }
            let cancelled = hub
                .report_result(a, session, MatchOutcome::Cancelled)
                .unwrap();
            assert_eq!(cancelled.len(), 2);
            assert!(cancelled.iter().all(|d| matches!(
                d.message,
                ServerMessage::ResultSettled {
                    winner_user_id: None,
                    ..
                }
            )));
            assert_eq!(hub.session_count(), 0);
            assert!(hub.standings.is_empty());
            assert!(hub.ratings.is_empty());
            assert!(hub.binary_relay_target(a, session).is_err());
            // Both players can match again immediately.
            hub.join_queue(a, mode, 1000, 100).unwrap();
            hub.join_queue(b, mode, 1000, 100).unwrap();
            assert_eq!(hub.session_count(), 1);
        }
    }
}

#[test]
fn result_retry_cannot_rewrite_a_declared_winner() {
    let (mut hub, a, b, session) = pair(MatchMode::Battle);
    hub.report_result(a, session, MatchOutcome::Local).unwrap();
    assert!(hub.report_result(a, session, MatchOutcome::Remote).is_err());
    hub.report_result(b, session, MatchOutcome::Remote).unwrap();
    assert_eq!(hub.standings["Alice"].pvp_wins, 1);
    assert_eq!(hub.standings["Bob"].pvp_wins, 0);
}

#[test]
fn repeated_queue_join_preserves_fifo_and_mode_changes_still_work() {
    let mut hub = Hub::default();
    let ids = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    for (id, name) in ids.into_iter().zip(["Alice", "Bob", "Chris"]) {
        hub.connect(id, identity(name)).unwrap();
    }
    hub.ratings.insert("Bob".into(), 1200);
    hub.join_queue(ids[0], MatchMode::Battle, 1000, 100)
        .unwrap();
    hub.join_queue(ids[1], MatchMode::Battle, 1200, 100)
        .unwrap();
    hub.join_queue(ids[0], MatchMode::Battle, 5000, 100)
        .unwrap();
    assert_eq!(hub.queued_count(), 2);
    let matched = hub
        .join_queue(ids[2], MatchMode::Battle, 1000, 1000)
        .unwrap();
    assert_eq!(matched[0].connection_id, ids[0]);
    hub.join_queue(ids[1], MatchMode::TimeCapsule, 1200, 100)
        .unwrap();
    assert_eq!(hub.queued_count(), 1);
    assert_eq!(
        hub.queues.values().next().unwrap()[0].mode,
        MatchMode::TimeCapsule
    );
}

#[test]
fn trade_sequence_aliases_cannot_inflate_either_trade_leaderboard() {
    for mode in [MatchMode::Trade, MatchMode::TimeCapsule] {
        let (mut hub, a, b, session) = pair(mode);
        for suffix in ["01", "+1", "-1", "", "18446744073709551616"] {
            for player in [a, b] {
                assert!(
                    hub.confirm_trade(player, format!("{session}-trade-{suffix}"))
                        .is_err()
                );
            }
        }
        assert!(hub.standings.is_empty());
        for _ in 0..2 {
            hub.confirm_trade(a, format!("{session}-trade-1")).unwrap();
            hub.confirm_trade(b, format!("{session}-trade-1")).unwrap();
        }
        assert_eq!(hub.standings["Alice"].trades, 1);
        assert_eq!(hub.standings["Bob"].trades, 1);
    }
}

#[test]
fn duplicate_connection_and_invalid_names_leave_existing_identity_intact() {
    let mut hub = Hub::default();
    let a = Uuid::new_v4();
    hub.connect(a, identity("Alice")).unwrap();
    assert!(hub.connect(a, identity("Bob")).is_err());
    assert_eq!(hub.clients[&a].identity.user_id, "Alice");
    assert!(!hub.users.contains_key("Bob"));
    for name in [" \t", "trainer\nspoof", "\u{001b}[31m"] {
        let mut bad = identity("Bob");
        bad.display_name = name.into();
        assert!(hub.connect(Uuid::new_v4(), bad).is_err());
        assert!(
            hub.replace_directory(HashMap::from([("Bob".into(), name.into())]))
                .is_err()
        );
    }
    assert_eq!(hub.directory["Alice"], "Alice");
    hub.disconnect(a);
    assert!(hub.users.is_empty());
    assert!(hub.clients.is_empty());
}

#[test]
fn unchanged_presence_is_quiet_but_still_expires_invitations() {
    let mut hub = Hub::default();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    hub.connect(a, identity("Alice")).unwrap();
    hub.connect(b, identity("Bob")).unwrap();
    hub.handle(a, presence(0));
    hub.handle(b, presence(1));
    assert!(hub.handle(a, presence(0)).is_empty());
    let invitation = hub
        .interaction_request(a, "Bob".into(), MatchMode::Battle)
        .unwrap();
    let ServerMessage::InteractionRequest { request_id, .. } = invitation[0].message else {
        panic!("invite")
    };
    hub.interactions.get_mut(&request_id).unwrap().created -= std::time::Duration::from_secs(31);
    let expired = hub.handle(a, presence(0));
    assert_eq!(expired.len(), 2);
    assert!(hub.interactions.is_empty());
    assert_eq!(hub.handle(a, presence(2)).len(), 1);
}

#[test]
fn extreme_ranked_ratings_remain_reloadable_after_a_win() {
    let (mut hub, a, b, session) = pair(MatchMode::Battle);
    hub.replace_ratings(HashMap::from([
        ("Alice".into(), 5000),
        ("Bob".into(), 5000),
    ]))
    .unwrap();
    hub.report_result(a, session, MatchOutcome::Local).unwrap();
    hub.report_result(b, session, MatchOutcome::Remote).unwrap();
    assert_eq!(hub.ratings["Alice"], 5000);
    let mut restored = Hub::default();
    restored.replace_ratings(hub.ratings_snapshot()).unwrap();
    restored
        .replace_standings(hub.standings_snapshot())
        .unwrap();
}
