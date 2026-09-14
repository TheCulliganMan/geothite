//! One live radio program, including text, source RNG and cartridge effects.
use crate::{
    GameDataSet,
    radio_catalog::RadioCatalog,
    radio_host::{RadioHost, RadioHostState, radio_textbox},
};
use anyhow::{Context, Result};
use crystal_core::{
    random::{CrystalRandom, DividerSource},
    state::GameState,
    systems::{
        radio_playback::{RadioPlayback, RadioPlaybackError, RadioPlaybackHost},
        radio_program::{
            RadioBuenaContext, RadioOakContext, RadioPeoplePlacesContext, RadioProgramContext,
            RadioProgramEffect, RadioProgramError, RadioProgramTextSource,
        },
        radio_text::{RADIO_SCROLL, RadioTextEnvironment, RadioTextError, RadioTextWindow},
    },
};
use std::{cell::RefCell, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioBroadcast {
    pub playback: RadioPlayback,
    pub host: RadioHostState,
    catalog: Arc<RadioCatalog>,
}

impl RadioBroadcast {
    pub fn new(data: &GameDataSet, line: u8, weekday: u8) -> Result<Self> {
        anyhow::ensure!(
            line <= 10,
            "radio station entry {line} is outside the source station table"
        );
        Ok(Self {
            playback: RadioPlayback::new(line, radio_textbox()),
            host: RadioHostState::new(weekday)?,
            catalog: Arc::new(RadioCatalog::new(data)?),
        })
    }

    pub fn advance_frame<S: DividerSource + Clone>(
        &mut self,
        data: &GameDataSet,
        game: &mut GameState,
        divider: &mut S,
        in_johto: bool,
        held_ab: bool,
    ) -> Result<()>
    where
        S::Error: std::fmt::Display,
    {
        // Scroll and suspended text cannot mutate cartridge state or sample
        // Random. Save the larger game only on calls that may execute a program.
        let game_before = (self.playback.state.current_line != RADIO_SCROLL
            && !self.playback.printing())
        .then(|| game.clone());
        let divider_before = divider.clone();
        let before = self.clone();
        let result = self.advance_frame_inner(data, game, divider, in_johto, held_ab);
        if result.is_err() {
            *self = before;
            if let Some(before) = game_before {
                *game = before;
            }
            *divider = divider_before;
        }
        result
    }

    fn advance_frame_inner<S: DividerSource>(
        &mut self,
        data: &GameDataSet,
        game: &mut GameState,
        divider: &mut S,
        in_johto: bool,
        held_ab: bool,
    ) -> Result<()>
    where
        S::Error: std::fmt::Display,
    {
        self.host.game_state_changed = false;
        let weekday = game.time.current_day % 7;
        self.host.memory.set_weekday(weekday)?;
        let category = u8::try_from(game.buenas_password.category_index)?;
        let option = u8::try_from(game.buenas_password.option_index)?;
        anyhow::ensure!(
            category < 16 && option < 16,
            "Buena password does not fit its source nibbles"
        );
        let buena = RadioBuenaContext {
            hour: game.time.registers.hours,
            password: (category << 4) | option,
            password_generated: game.flags.is_engine_flag_set("ENGINE_BUENAS_PASSWORD")?,
        };
        let oak = RadioOakContext {
            segment_counter: self.host.oak_segment_counter,
            delay: self.playback.state.delay,
            grass_routes: &self.catalog.oak_grass_routes,
        };
        let people = RadioPeoplePlacesContext {
            hall_of_fame: game
                .flags
                .engine_flags
                .get("STATUSFLAGS_HALL_OF_FAME_F")
                .copied()
                .unwrap_or(false),
            kanto_badges: game
                .badges
                .kanto
                .iter()
                .enumerate()
                .fold(0, |bits, (index, set)| bits | (u8::from(*set) << index)),
            trainer_class_count: self.catalog.trainer_class_count,
            place_count: self.catalog.place_count,
            hidden_people: &self.catalog.hidden_people,
            hidden_people_beat_e4: &self.catalog.hidden_people_beat_e4,
            hidden_people_beat_kanto: &self.catalog.hidden_people_beat_kanto,
        };
        let mut caught = [false; 251];
        for species in &game.pokedex.caught_species {
            let index = data
                .pokemon
                .get(species)
                .with_context(|| format!("caught radio species {species} is missing"))?
                .int_id;
            let slot = index
                .checked_sub(1)
                .and_then(|index| caught.get_mut(usize::from(index)))
                .context("caught radio species is outside Crystal's Pokédex")?;
            *slot = true;
        }
        let takeover = game
            .flags
            .is_engine_flag_set("ENGINE_ROCKETS_IN_RADIO_TOWER")?;
        let context = RadioProgramContext {
            buena: Some(&buena),
            oak: Some(&oak),
            printed: self.playback.state.printed,
            weekday,
            caught_pokemon: Some(&caught),
            people_places: Some(&people),
        };
        // Selection consumes Random before effects. Both use the very same
        // injected divider and state, so Lucky's later calls continue the stream.
        let host = RefCell::new(RadioHost {
            data,
            game,
            divider,
            state: &mut self.host,
            held_ab,
        });
        let mut random = |carry| {
            let mut host = host.borrow_mut();
            let RadioHost {
                game,
                divider,
                state,
                ..
            } = &mut *host;
            let mut rng = CrystalRandom::new(game.random_state, &mut **divider);
            let output = rng
                .random(carry)
                .map_err(|error| RadioProgramError::Random(error.to_string()))?;
            game.random_state = rng.state();
            state.game_state_changed = true;
            Ok(output.value)
        };
        self.playback.advance_frame(
            context,
            takeover,
            in_johto,
            held_ab,
            &mut random,
            &mut SharedHost(&host),
        )?;
        Ok(())
    }
}

struct SharedHost<'a, 'b, S: DividerSource>(&'a RefCell<RadioHost<'b, S>>);
impl<S: DividerSource> RadioTextEnvironment for SharedHost<'_, '_, S> {
    fn ram_text(&self, address: u16) -> Result<Vec<u8>, RadioTextError> {
        self.0.borrow().ram_text(address)
    }
    fn weekday(&self) -> u8 {
        self.0.borrow().weekday()
    }
}
impl<S: DividerSource> RadioPlaybackHost for SharedHost<'_, '_, S>
where
    S::Error: std::fmt::Display,
{
    fn effect(
        &mut self,
        effect: &RadioProgramEffect,
        window: &mut RadioTextWindow,
    ) -> Result<bool, RadioPlaybackError> {
        self.0.borrow_mut().effect(effect, window)
    }
    fn text(&mut self, source: &RadioProgramTextSource) -> Result<Vec<u8>, RadioPlaybackError> {
        self.0.borrow_mut().text(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_core::random::ReplayDivider;

    #[test]
    fn radio_broadcast_replays_rocket_rom_tiles_and_rolls_back_failed_random() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        let pack_path = root.join("content-packs/core-modular.crystalpack");
        let pack = crate::read_verified_compiled_game_pack(pack_path).unwrap();
        let data = pack.data();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tools/asm-oracle/fixtures/radio-program-rocket.json"
        ))
        .unwrap();
        let mut game = GameState::default();
        game.flags
            .set_engine_flag("ENGINE_ROCKETS_IN_RADIO_TOWER", true)
            .unwrap();
        let mut divider = ReplayDivider::new([]);
        // A normal music station is taken over by Rocket at PlayRadioShow.
        let mut broadcast = RadioBroadcast::new(data, 6, 0).unwrap();
        for record in fixture["prints"].as_array().unwrap() {
            let mut finished = false;
            for _ in 0..150 {
                broadcast
                    .advance_frame(data, &mut game, &mut divider, true, false)
                    .unwrap();
                if broadcast.playback.state.current_line == RADIO_SCROLL
                    && broadcast.playback.state.delay == 100
                {
                    finished = true;
                    break;
                }
            }
            assert!(finished, "source print did not complete: {record}");
            let actual = broadcast
                .playback
                .window
                .tiles
                .iter()
                .flatten()
                .map(|tile| format!("{tile:02x}"))
                .collect::<String>();
            assert_eq!(actual, record["tiles_after"].as_str().unwrap());
            assert_eq!(
                u64::from(broadcast.playback.state.printed),
                record["printed_after"].as_u64().unwrap()
            );
            // Leave the completion frame before waiting for the next print.
            broadcast
                .advance_frame(data, &mut game, &mut divider, true, false)
                .unwrap();
        }
        assert_eq!(divider.consumed(), 0);
        game.flags
            .set_engine_flag("ENGINE_ROCKETS_IN_RADIO_TOWER", false)
            .unwrap();
        broadcast.playback.state.current_line = 13;
        let game_before = game.clone();
        let before = broadcast.clone();
        assert!(
            broadcast
                .advance_frame(data, &mut game, &mut divider, true, false)
                .is_err()
        );
        assert_eq!(game, game_before);
        assert_eq!(broadcast, before);
        assert_eq!(divider.consumed(), 0);
    }
}
