use super::*;
use crystal_net::hosted::{LeaderboardEntry, LeaderboardStats};

impl Hub {
    pub fn standings_snapshot(&self) -> HashMap<String, LeaderboardStats> {
        self.standings.clone()
    }

    pub fn replace_standings(
        &mut self,
        standings: HashMap<String, LeaderboardStats>,
    ) -> Result<(), String> {
        for (id, stats) in &standings {
            validate_token("leaderboard user id", id)?;
            if stats.pvp_wins > stats.pvp_battles
                || stats.pve_wins > stats.pve_battles
                || stats.party_level > 600
            {
                return Err("invalid leaderboard totals".into());
            }
        }
        self.standings = standings;
        Ok(())
    }

    pub(super) fn update_game_stats(
        &mut self,
        connection: Uuid,
        battles: u64,
        wins: u64,
        level: u16,
    ) -> Result<Vec<Delivery>, String> {
        if wins > battles || battles > u64::from(u32::MAX) || level > 600 {
            return Err("invalid game statistics".into());
        }
        let user = &self.clients[&connection].identity.user_id;
        let stats = self.standings.entry(user.clone()).or_default();
        // Save restores and reconnects must not double count cumulative progress.
        stats.pve_battles = stats.pve_battles.max(battles);
        stats.pve_wins = stats.pve_wins.max(wins);
        stats.party_level = stats.party_level.max(level);
        Ok(Vec::new())
    }

    pub(super) fn leaderboard(
        &self,
        connection: Uuid,
        metric: String,
        offset: usize,
    ) -> Result<Vec<Delivery>, String> {
        if LeaderboardStats::default().score(&metric).is_none() {
            return Err("unknown leaderboard".into());
        }
        let mut entries = self
            .standings
            .iter()
            .filter_map(|(id, stats)| {
                let name = self.directory.get(id)?;
                let value = stats.score(&metric)?;
                (value > 0).then(|| LeaderboardEntry {
                    rank: 0,
                    user_id: id.clone(),
                    display_name: name.clone(),
                    online: self.users.contains_key(id),
                    value,
                })
            })
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| {
            b.value
                .cmp(&a.value)
                .then_with(|| {
                    a.display_name
                        .to_lowercase()
                        .cmp(&b.display_name.to_lowercase())
                })
                .then_with(|| a.user_id.cmp(&b.user_id))
        });
        let mut rank = 0;
        let mut previous = None;
        for (index, entry) in entries.iter_mut().enumerate() {
            if previous != Some(entry.value) {
                rank = index + 1;
            }
            previous = Some(entry.value);
            entry.rank = rank;
        }
        let total = entries.len();
        let offset = offset.min(total.saturating_sub(1) / 100 * 100);
        let entries = entries.into_iter().skip(offset).take(100).collect();
        Ok(vec![deliver(
            connection,
            ServerMessage::Leaderboard {
                metric,
                offset,
                total,
                entries,
            },
        )])
    }

    pub(super) fn confirm_trade(
        &mut self,
        connection: Uuid,
        trade_id: String,
    ) -> Result<Vec<Delivery>, String> {
        let session_id = self
            .active_session_for(connection)
            .ok_or("no active trade session")?;
        let session = self.sessions.get_mut(&session_id).expect("active session");
        if !matches!(session.mode, MatchMode::Trade | MatchMode::TimeCapsule)
            || !trade_id.starts_with(&format!("{session_id}-trade-"))
            || trade_id.len() > 80
            || trade_id
                .strip_prefix(&format!("{session_id}-trade-"))
                .and_then(|s| s.parse::<u64>().ok())
                .is_none()
        {
            return Err("invalid completed trade".into());
        }
        if !session.trades.contains_key(&trade_id) && session.trades.len() >= 1024 {
            return Err("start a new trade session".into());
        }
        let confirmations = session.trades.entry(trade_id).or_default();
        if !confirmations.insert(connection) || confirmations.len() != 2 {
            return Ok(Vec::new());
        }
        for id in session.players {
            let user = &self.clients[&id].identity.user_id;
            let stats = self.standings.entry(user.clone()).or_default();
            stats.trades = stats.trades.saturating_add(1);
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn connect(hub: &mut Hub, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        hub.connect(
            id,
            ClientIdentity {
                user_id: name.into(),
                display_name: name.into(),
                world: WorldIdentity {
                    world_id: "world".into(),
                    modpack: ModpackIdentity {
                        id: "core".into(),
                        content_hash: "hash".into(),
                    },
                },
            },
        )
        .unwrap();
        id
    }
    fn pair(hub: &mut Hub, mode: MatchMode) -> (Uuid, Uuid, Uuid) {
        let a = connect(hub, "Ash");
        let b = connect(hub, "Blue");
        let session = Uuid::new_v4();
        hub.sessions.insert(
            session,
            Session {
                players: [a, b],
                mode,
                ranked: false,
                reports: HashMap::new(),
                trades: HashMap::new(),
                settled: false,
            },
        );
        (a, b, session)
    }
    #[test]
    fn community_directory_keeps_offline_users_and_refreshes_names() {
        let mut hub = Hub::default();
        let a = connect(&mut hub, "Ash");
        let b = connect(&mut hub, "Blue");
        hub.handle(
            b,
            ClientMessage::SetProfile {
                display_name: "Gold".into(),
                player_gender: 0,
            },
        );
        hub.disconnect(b);
        let ServerMessage::SocialUsers { users, total, .. } =
            hub.social_list(a, "".into(), 0).unwrap().remove(0).message
        else {
            panic!()
        };
        assert_eq!(total, 2);
        assert!(users[0].online);
        assert_eq!(users[1].display_name, "Gold");
        assert!(!users[1].online);
        let mut restarted = Hub::default();
        restarted
            .replace_directory(
                serde_json::from_slice(&serde_json::to_vec(&hub.directory_snapshot()).unwrap())
                    .unwrap(),
            )
            .unwrap();
        let a = connect(&mut restarted, "Ash");
        let ServerMessage::SocialUsers { users, .. } = restarted
            .social_list(a, "GOLD".into(), 0)
            .unwrap()
            .remove(0)
            .message
        else {
            panic!()
        };
        assert_eq!(users.len(), 1);
        assert!(!users[0].online);
    }
    #[test]
    fn community_directory_pages_are_bounded_and_do_not_omit_users() {
        let mut hub = Hub::default();
        let a = connect(&mut hub, "Ash");
        for i in 0..204 {
            hub.directory
                .insert(format!("player-{i}"), format!("Trainer{i:03}"));
        }
        let mut count = 0;
        for offset in [0, 100, 200] {
            let ServerMessage::SocialUsers {
                users,
                total,
                offset: returned,
                ..
            } = hub
                .social_list(a, "".into(), offset)
                .unwrap()
                .remove(0)
                .message
            else {
                panic!()
            };
            assert_eq!(returned, offset);
            assert_eq!(total, 205);
            assert!(users.len() <= 100);
            count += users.len();
        }
        assert_eq!(count, 205);
        assert!(matches!(
            hub.handle(
                Uuid::new_v4(),
                ClientMessage::SocialList {
                    query: "".into(),
                    offset: 0
                }
            )[0]
            .message,
            ServerMessage::Error { .. }
        ));
    }
    #[test]
    fn community_pvp_requires_agreement_and_does_not_count_twice_or_count_cancels() {
        let mut hub = Hub::default();
        let (a, b, session_id) = pair(&mut hub, MatchMode::Battle);
        hub.handle(
            a,
            ClientMessage::Result {
                session_id,
                outcome: MatchOutcome::Local,
            },
        );
        assert!(hub.standings.is_empty());
        hub.handle(
            b,
            ClientMessage::Result {
                session_id,
                outcome: MatchOutcome::Remote,
            },
        );
        assert_eq!(hub.standings["Ash"].pvp_wins, 1);
        assert_eq!(hub.standings["Blue"].pvp_battles, 1);
        hub.handle(
            a,
            ClientMessage::Result {
                session_id,
                outcome: MatchOutcome::Local,
            },
        );
        assert_eq!(hub.standings["Ash"].pvp_battles, 1);
        let mut cancelled = Hub::default();
        let (a, _, session_id) = pair(&mut cancelled, MatchMode::Battle);
        cancelled.handle(
            a,
            ClientMessage::Result {
                session_id,
                outcome: MatchOutcome::Cancelled,
            },
        );
        assert!(cancelled.standings.is_empty());
    }
    #[test]
    fn community_trades_require_two_confirmations_and_count_each_trade_once() {
        let mut hub = Hub::default();
        let (a, b, session) = pair(&mut hub, MatchMode::Trade);
        for sequence in 0..2 {
            let trade_id = format!("{session}-trade-{sequence}");
            hub.handle(
                a,
                ClientMessage::TradeCompleted {
                    trade_id: trade_id.clone(),
                },
            );
            assert_eq!(hub.standings.get("Ash").map_or(0, |s| s.trades), sequence);
            hub.handle(
                b,
                ClientMessage::TradeCompleted {
                    trade_id: trade_id.clone(),
                },
            );
            hub.handle(a, ClientMessage::TradeCompleted { trade_id });
        }
        assert_eq!(hub.standings["Ash"].trades, 2);
        assert_eq!(hub.standings["Blue"].trades, 2);
        assert!(hub.confirm_trade(a, "fake-trade-3".into()).is_err());
    }
    #[test]
    fn community_game_stats_are_high_water_marks_and_rank_ties_stably() {
        let mut hub = Hub::default();
        let a = connect(&mut hub, "Ash");
        let b = connect(&mut hub, "Blue");
        hub.update_game_stats(a, 10, 8, 200).unwrap();
        hub.update_game_stats(a, 5, 3, 100).unwrap();
        hub.update_game_stats(b, 10, 8, 190).unwrap();
        assert_eq!(hub.standings["Ash"].pve_battles, 10);
        assert_eq!(hub.standings["Ash"].party_level, 200);
        assert!(hub.update_game_stats(a, 2, 3, 200).is_err());
        assert!(hub.update_game_stats(a, 10, 8, 601).is_err());
        let ServerMessage::Leaderboard { entries, .. } = hub
            .leaderboard(a, "pve_wins".into(), 0)
            .unwrap()
            .remove(0)
            .message
        else {
            panic!()
        };
        assert_eq!(entries.len(), 2);
        assert_eq!((entries[0].rank, entries[1].rank), (1, 1));
        assert_eq!(entries[0].display_name, "Ash");
        let restored =
            serde_json::from_slice(&serde_json::to_vec(&hub.standings_snapshot()).unwrap())
                .unwrap();
        hub.replace_standings(restored).unwrap();
        assert_eq!(hub.standings["Ash"].pve_wins, 8);
    }
}
