#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCompetitiveBattleSnapshot {
    pub turn: u32,
    pub weather: crystal_core::battle::damage::Weather,
    pub weather_turns: u8,
    /// The engine's ordinary-move priority; presentation reports its offset.
    pub base_priority: i8,
    pub moves: Vec<crystal_core::battle::turn::BattleMovePreview>,
    pub player_boosts: BTreeMap<Stat, i8>,
    pub enemy_boosts: BTreeMap<Stat, i8>,
    /// Reflect, Light Screen, Safeguard turns; player then enemy.
    pub screens: [[u8; 3]; 2],
    pub spikes: [bool; 2],
}

impl RuntimeCompetitiveBattleSnapshot {
    pub fn effectiveness_text(value: crystal_core::battle::damage::TypeMultiplier) -> String {
        if value.denominator == 1 {
            format!("x{}", value.numerator)
        } else {
            format!("x{}/{}", value.numerator, value.denominator)
        }
    }

    pub fn move_detail_lines(&self, index: usize) -> Vec<String> {
        let Some(m) = self.moves.get(index) else {
            return Vec::new();
        };
        let power = m
            .power
            .map_or_else(|| "conditional".into(), |v| v.to_string());
        let category = match m.category {
            crystal_core::battle::damage::MoveCategory::Physical => "PHYSICAL / ATK vs DEF",
            crystal_core::battle::damage::MoveCategory::Special => "SPECIAL / SPA vs SPD",
            crystal_core::battle::damage::MoveCategory::Status => "STATUS",
        };
        let mut lines = vec![
            format!("{}  {category}", m.move_type.replace("_TYPE", "")),
            format!(
                "Power {power}  Base accuracy {}%  Priority {:+}",
                m.accuracy,
                i16::from(m.priority) - i16::from(self.base_priority)
            ),
        ];
        if m.category != crystal_core::battle::damage::MoveCategory::Status {
            lines.push(format!(
                "{}Effectiveness {}",
                if m.stab { "STAB  " } else { "" },
                Self::effectiveness_text(m.effectiveness)
            ));
            lines.push(m.damage_on_hit.map_or_else(
                || "Damage: conditional".into(),
                |[min, max]| {
                    format!(
                        "Damage on hit: {min}-{max}{} (non-critical)",
                        if m.per_hit { " per hit" } else { "" }
                    )
                },
            ));
        }
        lines
    }

    pub fn field_detail_lines(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "Turn {}  {:?} ({} turns)",
            self.turn + 1,
            self.weather,
            self.weather_turns
        )];
        for (side, boosts, screens, spikes) in [
            ("You", &self.player_boosts, self.screens[0], self.spikes[0]),
            ("Foe", &self.enemy_boosts, self.screens[1], self.spikes[1]),
        ] {
            let stats = boosts
                .iter()
                .filter(|(_, v)| **v != 0)
                .map(|(s, v)| format!("{s:?} {v:+}"))
                .collect::<Vec<_>>()
                .join(" ");
            if !stats.is_empty() {
                lines.push(format!("{side}: {stats}"));
            }
            if screens.iter().any(|n| *n > 0) || spikes {
                lines.push(format!(
                    "{side}: Reflect {} Screen {} Safeguard {} Spikes {}",
                    screens[0], screens[1], screens[2], spikes
                ));
            }
        }
        lines
    }

    fn from_state(data: &GameDataSet, state: &GameState) -> Result<Option<Self>> {
        if !data.competitive_battles
            || battle_tower_opponent_is_staged(state)
            || matches!(state.battle, crystal_core::state::BattleMemory::Inactive)
        {
            return Ok(None);
        }
        // The first command menu precedes the retained combat record. Use the
        // production projection for both fresh and ongoing battles so move
        // inspection is available before the first turn, without mutating it.
        let projected = crystal_core::battle::turn::active_battle_combat_state(state)
            .map_err(|e| anyhow::anyhow!("competitive battle projection: {e:?}"))?;
        let combat = &projected;
        let moves = crystal_core::battle::turn::battle_moves(
            combat,
            crystal_core::battle::turn::BattleSide::Player,
        )
        .iter()
        .map(|learned| {
            let move_data = data
                .moves
                .get(&learned.name)
                .context("competitive preview missing move")?;
            crystal_core::battle::turn::preview_battle_move(
                combat,
                crystal_core::battle::turn::BattleSide::Player,
                move_data,
                &data.items,
                &data.move_priorities,
                &data.battle_stat_multipliers,
                &data.type_categories,
                &data.type_effectiveness,
                &data.weather_modifiers,
            )
            .map_err(|e| anyhow::anyhow!("competitive move preview: {e:?}"))
        })
        .collect::<Result<Vec<_>>>()?;
        Ok(Some(Self {
            turn: combat.turn,
            weather: combat.weather,
            weather_turns: combat.weather_turns,
            base_priority: data.move_priorities.base_priority,
            moves,
            player_boosts: combat.player_transform.as_ref().map_or_else(
                || combat.player.stat_boosts.clone(),
                |p| p.stat_boosts.clone(),
            ),
            enemy_boosts: combat.enemy_transform.as_ref().map_or_else(
                || combat.enemy.stat_boosts.clone(),
                |p| p.stat_boosts.clone(),
            ),
            screens: [
                [
                    combat.player_reflect_turns,
                    combat.player_light_screen_turns,
                    combat.player_safeguard_turns,
                ],
                [
                    combat.enemy_reflect_turns,
                    combat.enemy_light_screen_turns,
                    combat.enemy_safeguard_turns,
                ],
            ],
            spikes: [combat.player_spikes, combat.enemy_spikes],
        }))
    }
}
