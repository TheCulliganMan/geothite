//! GetJoypad/JoyTextDelay mirrors and VBlank's text-delay counter.
//! Masks use the core's GameButton bits; the algorithm is bit-position agnostic.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct JoyTextDelay {
    pub down: u8,
    pub pressed: u8,
    pub released: u8,
    pub last: u8,
    pub text_delay_frames: u8,
}

impl JoyTextDelay {
    pub fn advance_vblanks(&mut self, frames: u32) {
        self.text_delay_frames = self.text_delay_frames
            .saturating_sub(frames.min(u32::from(u8::MAX)) as u8);
    }

    /// GetJoypad updates mirrors without applying JoyTextDelay's repeat gate.
    pub fn get_joypad(&mut self, physical_down: u8) {
        let changed = self.down ^ physical_down;
        self.released = changed & self.down;
        self.pressed = changed & physical_down;
        self.down = physical_down;
    }

    /// Call only where ASM calls JoyTextDelay. Blocking text/graphics calls
    /// continue VBlank but do not update these GetJoypad mirrors.
    pub fn sample(&mut self, physical_down: u8, in_menu: bool) {
        self.get_joypad(physical_down);
        self.last = if in_menu { self.down } else { self.pressed };
        if self.pressed != 0 {
            self.text_delay_frames = 15;
        } else if self.text_delay_frames != 0 {
            self.last = 0;
        } else {
            self.text_delay_frames = 5;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{B_PAD_A, B_PAD_B, B_PAD_RIGHT};

    #[test]
    fn source_initial_fifteen_and_repeat_five_vblanks() {
        let mut joy = JoyTextDelay::default();
        joy.sample(B_PAD_RIGHT, true);
        assert_eq!(joy.last, B_PAD_RIGHT);
        for period in [15, 5, 5] {
            for _ in 1..period {
                joy.advance_vblanks(1);
                joy.sample(B_PAD_RIGHT, true);
                assert_eq!(joy.last, 0);
                assert_eq!(joy.pressed, 0);
            }
            joy.advance_vblanks(1);
            joy.sample(B_PAD_RIGHT, true);
            assert_eq!(joy.last, B_PAD_RIGHT);
            assert_eq!(joy.pressed, 0);
        }
    }

    #[test]
    fn blocked_call_keeps_mirrors_but_vblank_expires_the_delay() {
        let mut joy = JoyTextDelay::default();
        joy.sample(B_PAD_A, true);
        joy.advance_vblanks(30);
        assert_eq!(joy.down, B_PAD_A);
        joy.sample(B_PAD_B, true);
        assert_eq!((joy.pressed, joy.released, joy.last), (B_PAD_B, B_PAD_A, B_PAD_B));
    }

    #[test]
    fn conversation_getjoypad_does_not_restart_menu_repeat_or_create_a_second_press() {
        let mut joy = JoyTextDelay::default();
        joy.sample(B_PAD_A, true);
        joy.advance_vblanks(3);
        let before = (joy.last, joy.text_delay_frames);
        joy.get_joypad(B_PAD_B);
        assert_eq!((joy.pressed, joy.released), (B_PAD_B, B_PAD_A));
        assert_eq!((joy.last, joy.text_delay_frames), before);
        joy.advance_vblanks(10);
        joy.sample(B_PAD_B, true);
        assert_eq!(joy.pressed, 0, "FinishPhoneCall must not see a new B after the conversation's GetJoypad");
    }

    #[test]
    fn fresh_press_exposes_all_held_menu_buttons_but_non_menu_is_edge_only() {
        let mut joy = JoyTextDelay::default();
        joy.sample(B_PAD_A, true);
        joy.advance_vblanks(1);
        joy.sample(B_PAD_A | B_PAD_RIGHT, true);
        assert_eq!(joy.pressed, B_PAD_RIGHT);
        assert_eq!(joy.last, B_PAD_A | B_PAD_RIGHT);
        joy.advance_vblanks(15);
        joy.sample(B_PAD_A | B_PAD_RIGHT, false);
        assert_eq!(joy.last, 0);
    }
}
