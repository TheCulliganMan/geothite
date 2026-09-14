//! Renderer-independent Hall of Fame presentation sequence.
//!
//! Timing follows engine/events/halloffame.asm. Frontpic animation and Oak's
//! rating are explicit completion boundaries; their duration belongs to the
//! existing animation and dialogue interpreters, not an estimated timer.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HallOfFamePhase {
    /// White display after the opening music/palette fade has completed.
    OpeningHold,
    /// PlayMusic(NONE), DelayFrame, then MUSIC_HALL_OF_FAME.
    MusicWait,
    PokemonBack {
        member: usize,
    },
    PokemonFront {
        member: usize,
    },
    PokemonAnimation {
        member: usize,
    },
    PokemonHold {
        member: usize,
    },
    PlayerBack,
    PlayerFront,
    OakRating,
    ClosingFade,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HallOfFameSequence {
    members: usize,
    phase: HallOfFamePhase,
    elapsed: u16,
}

impl HallOfFameSequence {
    /// Start after the opening fade. `members` counts recorded non-egg Pokémon.
    pub fn new(members: usize) -> Option<Self> {
        (members <= 6).then_some(Self {
            members,
            phase: HallOfFamePhase::OpeningHold,
            elapsed: 0,
        })
    }

    pub fn elapsed_frames(&self) -> u16 { self.elapsed }

    pub fn phase(&self) -> HallOfFamePhase {
        self.phase
    }

    /// Original eight-bit BG scroll registers for entrance pictures.
    /// Renderers must preserve wrapping at 256 rather than interpolate through
    /// zero as a signed/linear motion. Other phases have a stationary display.
    pub fn scroll(&self) -> (u8, u8) {
        match self.phase {
            HallOfFamePhase::PokemonBack { .. } | HallOfFamePhase::PlayerBack => {
                (0x90u8.wrapping_add((self.elapsed * 4) as u8), 0xd0)
            }
            HallOfFamePhase::PokemonFront { .. } => {
                (0x70u8.wrapping_sub((self.elapsed * 2) as u8), 0)
            }
            HallOfFamePhase::PlayerFront => (0xc0u8.wrapping_sub((self.elapsed * 2) as u8), 0),
            _ => (0, 0),
        }
    }

    fn enter(&mut self, phase: HallOfFamePhase) {
        self.phase = phase;
        self.elapsed = 0;
    }

    /// Advance one emulated frame. Returns true exactly when the phase changes.
    /// Ordinary A/B presses must not complete animation or rating boundaries.
    pub fn tick(&mut self) -> bool {
        use HallOfFamePhase::*;
        let (duration, next) = match self.phase {
            OpeningHold => (
                100,
                if self.members == 0 {
                    PlayerBack
                } else {
                    MusicWait
                },
            ),
            MusicWait => (1, PokemonBack { member: 0 }),
            PokemonBack { member } => (56, PokemonFront { member }),
            PokemonFront { member } => (56, PokemonAnimation { member }),
            PokemonHold { member } => (
                60,
                if member + 1 < self.members {
                    PokemonBack { member: member + 1 }
                } else {
                    PlayerBack
                },
            ),
            PlayerBack => (56, PlayerFront),
            PlayerFront => (96, OakRating),
            ClosingFade => (32, Complete),
            PokemonAnimation { .. } | OakRating | Complete => return false,
        };
        self.elapsed += 1;
        if self.elapsed < duration {
            return false;
        }
        self.enter(next);
        true
    }

    /// Called only after ANIM_MON_HOF has finished for the displayed member.
    pub fn complete_pokemon_animation(&mut self, member: usize) -> bool {
        if self.phase != (HallOfFamePhase::PokemonAnimation { member }) {
            return false;
        }
        self.enter(HallOfFamePhase::PokemonHold { member });
        true
    }

    /// Called after Oak's final text and sound boundary have both completed.
    pub fn complete_rating(&mut self) -> bool {
        if self.phase != HallOfFamePhase::OakRating {
            return false;
        }
        self.enter(HallOfFamePhase::ClosingFade);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wait(sequence: &mut HallOfFameSequence, frames: usize) {
        for _ in 1..frames {
            assert!(!sequence.tick());
        }
        assert!(sequence.tick());
    }

    #[test]
    fn ceremony_visits_all_six_members_and_waits_for_animation_and_rating() {
        use HallOfFamePhase::*;
        let mut sequence = HallOfFameSequence::new(6).unwrap();
        assert!(!sequence.complete_rating());
        wait(&mut sequence, 100);
        assert_eq!(sequence.phase(), MusicWait);
        wait(&mut sequence, 1);
        for member in 0..6 {
            assert_eq!(sequence.phase(), PokemonBack { member });
            assert_eq!(sequence.scroll(), (0x90, 0xd0));
            // Source SCX wraps after 28 of the 56 back-picture frames.
            for _ in 0..28 {
                assert!(!sequence.tick());
            }
            assert_eq!(sequence.scroll(), (0, 0xd0));
            wait(&mut sequence, 28);
            assert_eq!(sequence.phase(), PokemonFront { member });
            assert_eq!(sequence.scroll(), (0x70, 0));
            wait(&mut sequence, 56);
            assert_eq!(sequence.phase(), PokemonAnimation { member });
            for _ in 0..1000 {
                assert!(!sequence.tick());
            }
            assert!(!sequence.complete_pokemon_animation(member + 1));
            assert!(sequence.complete_pokemon_animation(member));
            wait(&mut sequence, 60);
        }
        assert_eq!(sequence.phase(), PlayerBack);
        wait(&mut sequence, 56);
        assert_eq!(sequence.phase(), PlayerFront);
        assert_eq!(sequence.scroll(), (0xc0, 0));
        wait(&mut sequence, 96);
        assert_eq!(sequence.phase(), OakRating);
        for _ in 0..1000 {
            assert!(!sequence.tick());
        }
        assert!(sequence.complete_rating());
        wait(&mut sequence, 32);
        assert_eq!(sequence.phase(), Complete);
        assert!(!sequence.tick());
        assert!(!sequence.complete_rating());
    }

    #[test]
    fn empty_record_goes_to_player_and_invalid_team_is_rejected() {
        assert!(HallOfFameSequence::new(7).is_none());
        let mut sequence = HallOfFameSequence::new(0).unwrap();
        wait(&mut sequence, 100);
        assert_eq!(sequence.phase(), HallOfFamePhase::PlayerBack);
    }
}
