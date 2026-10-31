// A purchase and its random result commit together. The shop remains the input
// owner until its receipt/reveal is acknowledged; only then may battle begin.
impl GameDataSet {
    /// Complete a shop transaction and resolve its Fair-only surprise once.
    /// Errors leave money, inventory, random state and pending encounter intact.
    pub fn buy_shop_item_with_surprise<S>(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
        item_id: &str,
        quantity: u16,
        divider: &mut S,
    ) -> Result<ShopResult>
    where
        S: DividerSource + ?Sized,
        S::Error: std::fmt::Display,
    {
        use crystal_core::systems::fair_food::{self, PendingFairFood};
        anyhow::ensure!(
            PendingFairFood::read(state)
                .map_err(anyhow::Error::msg)?
                .is_none(),
            "acknowledge the pending food surprise before another purchase"
        );
        let mut next = state.clone();
        let result = self.buy_shop_item(&mut next, item_id, quantity)?;
        if result.success {
            let shop = next
                .script_runtime
                .pending_shop
                .as_ref()
                .context("purchase requires its shop")?;
            if let Some(species) =
                fair_food::species_for_purchase(&session.map.name, &shop.mart_id, item_id)
            {
                let mut rng = CrystalRandom::new(next.random_state, divider);
                let surprise = fair_food::roll_surprise(&mut rng)
                    .map_err(|error| anyhow::anyhow!("Fair food roll: {error}"))?;
                if surprise && fair_food::usable_party_index(&next).is_some() {
                    let encounter = WildEncounterRoll {
                        map_name: session.map.name.clone(),
                        tile: session.player.tile,
                        surface: EncounterSurface::Grass,
                        time: next.time.time_of_day,
                        threshold: 20,
                        encounter_roll: 0,
                        slot_percent_roll: Some(0),
                        level_roll: None,
                        roaming_slot: None,
                        repelled_by: None,
                        resolved: Some(ResolvedWildEncounter {
                            encounter: crystal_core::world::encounters::WildEncounter {
                                species: species.into(),
                                level: fair_food::SURPRISE_LEVEL,
                            },
                            slot: 0,
                            level: fair_food::SURPRISE_LEVEL,
                        }),
                    };
                    let metadata = self.runtime_map_metadata_for_name(&session.map.name)?;
                    let current_map = (
                        u8::try_from(metadata.group_id)?,
                        u8::try_from(metadata.map_id)?,
                    );
                    let enemy = materialize_non_roaming_wild_battle_with_rng(
                        &encounter,
                        "BATTLETYPE_NORMAL",
                        self.pokemon
                            .get(species)
                            .context("Fair food species missing")?,
                        &self.learnsets,
                        &self.moves,
                        &self.growth_rates,
                        self.unlocked_unown_sets(&next)?,
                        next.player_id,
                        current_map,
                        current_map,
                        &self.magikarp_lengths,
                        &mut rng,
                    )
                    .map_err(|error| anyhow::anyhow!("materialize Fair food surprise: {error}"))?;
                    let pending = PendingFairFood {
                        item_id: item_id.into(),
                        battle: WildBattleStart {
                            battle_type: "BATTLETYPE_NORMAL".into(),
                            battle_music: self.wild_battle_music_for_map_time(
                                &session.map.name,
                                next.time.time_of_day,
                            )?,
                            encounter,
                            enemy_party: vec![enemy.clone()],
                            enemy_pokemon: enemy,
                        },
                    };
                    pending.write(&mut next).map_err(anyhow::Error::msg)?;
                }
                next.random_state = rng.state();
            }
        }
        *state = next;
        Ok(result)
    }

    /// Consume a saved encounter after all shop and dialogue owners have closed.
    /// Repeated calls after consumption are harmless and do not charge or roll.
    pub fn start_pending_fair_food_battle(
        &self,
        state: &mut GameState,
        session: &OverworldSession,
    ) -> Result<bool> {
        use crystal_core::systems::fair_food::PendingFairFood;
        let Some(pending) = PendingFairFood::read(state).map_err(anyhow::Error::msg)? else {
            return Ok(false);
        };
        pending.validate(state).map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            state.script_runtime.pending_shop.is_none()
                && !state.script_runtime.text_window_open
                && state.script_runtime.active_menu.is_none()
                && state.script_runtime.next_script.is_none(),
            "close the shop and its dialogue before starting the food surprise"
        );
        anyhow::ensure!(
            session.map.name == crystal_core::systems::fair_food::FAIR_MAP,
            "food surprise left the Fair"
        );
        if crystal_core::systems::fair_food::usable_party_index(state).is_none() {
            PendingFairFood::clear(state);
            return Ok(false);
        }
        let mut next = state.clone();
        PendingFairFood::clear(&mut next);
        next.script_runtime
            .memory
            .insert("wBattleScriptFlags".into(), "0".into());
        activate_wild_battle_start(&mut next, &pending.battle, &self.items)
            .context("activate Fair food surprise")?;
        next.battle_active_party_index =
            crystal_core::systems::fair_food::usable_party_index(&next);
        next.battle_active_enemy_party_index = Some(0);
        next.battle_rewarded_enemy_party_indices.clear();
        next.battle_evolvable_party_indices.clear();
        next.battle_escape_attempts = 0;
        next.battle_pay_day_money = 0;
        crate::nuzlocke::register_wild_encounter(
            self.nuzlocke_rules,
            &mut next,
            &session.map.name,
            "BATTLETYPE_NORMAL",
        );
        *state = next;
        Ok(true)
    }
}

#[cfg(test)]
#[path = "fair_food_purchase_tests.rs"]
mod fair_food_purchase_tests;
