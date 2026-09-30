//! Read-only battle presentation. These values describe the scene currently on
//! screen, not the runtime's already-resolved future turn. Renderers must never
//! use animation completion to advance a turn or infer a battle result.
use bevy::prelude::{Handle, Image, Resource, Vec2};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualBattleSide {
    Player,
    Enemy,
}
impl VisualBattleSide {
    pub const fn index(self) -> usize {
        match self {
            Self::Player => 0,
            Self::Enemy => 1,
        }
    }
    pub const fn opposite(self) -> Self {
        match self {
            Self::Player => Self::Enemy,
            Self::Enemy => Self::Player,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VisualBattleEnvironment {
    #[default]
    Meadow,
    Forest,
    Cave,
    Water,
    Interior,
    Ice,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualBattleBattler {
    pub side: VisualBattleSide,
    /// Exact currently presented identity, including visible Transform.
    pub species_id: Arc<str>,
    pub party_index: Option<usize>,
    pub texture: Handle<Image>,
    pub texture_size: Vec2,
    pub visible: bool,
    /// Substitute/minimize retain their honest source art instead of using a
    /// species mesh that would communicate the wrong visible battle state.
    pub allow_species_model: bool,
    pub shiny: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualBattleCueKind {
    Move,
    Impact,
    Faint,
    SendOut,
    Withdraw,
    Capture,
    CaptureDeflect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualBattleCue {
    pub kind: VisualBattleCueKind,
    pub side: VisualBattleSide,
    pub move_id: Arc<str>,
    /// From the verified move catalog; not guessed from move names.
    pub element: Arc<str>,
    /// Normalized progress of an existing visible animation, never a clock
    /// driving authoritative state. Capture success is deliberately absent.
    pub progress: f32,
    pub damaging: bool,
}

/// Presentation intensity only. This resource is never read by the controller
/// or the source animation interpreter. Both modes consume the same frames.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BattleFlashMode {
    #[default]
    Full,
    Reduced,
}
impl BattleFlashMode {
    pub const fn palette_strength(self) -> f32 {
        match self {
            Self::Full => 1.0,
            Self::Reduced => 0.12,
        }
    }
}

/// A currently visible source object, after source callbacks, framesets,
/// mirroring, clipping and palette writes. No future spawn event is exposed.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualBattleSourceObject {
    pub slot: usize,
    pub object_id: Arc<str>,
    pub texture: Handle<Image>,
    /// Same current OAM/frame with neutral OBP registers for reduced flashes.
    pub neutral_texture: Handle<Image>,
    /// Pixel coordinates in the original 160x144 battle display, Y down.
    pub center: Vec2,
    pub size: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualBattleSourceFrame {
    pub frame: u16,
    pub bgp: u8,
    pub battler_bgps: [u8; 2],
    /// Actual source four-shade palettes, including background shade zero.
    pub battler_palettes: [[[f32; 4]; 4]; 2],
    pub battler_textures: [Handle<Image>; 2],
    /// Current source displacements in pixels, Y up.
    pub battler_offsets: [Vec2; 2],
    pub screen_offset: Vec2,
    /// Current original horizontal scanline deformation, in source pixels.
    /// The arena bends its models from this buffer without advancing it.
    pub line_x_offsets: Option<[i8; 0x5f]>,
    pub objects: Vec<VisualBattleSourceObject>,
}

#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct VisualBattleFrame {
    pub active: bool,
    pub map_id: Arc<str>,
    pub environment: VisualBattleEnvironment,
    pub battlers: [Option<VisualBattleBattler>; 2],
    pub cues: Vec<VisualBattleCue>,
    pub source: Option<VisualBattleSourceFrame>,
}
impl VisualBattleFrame {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.active {
            return Ok(());
        }
        if self.map_id.is_empty() {
            return Err("battle map identity is missing");
        }
        for (i, battler) in self.battlers.iter().enumerate() {
            let Some(battler) = battler else {
                continue;
            };
            if battler.side.index() != i {
                return Err("battle side slot mismatch");
            }
            if battler.species_id.is_empty() {
                return Err("battle species is missing");
            }
            if battler.texture == Handle::default() {
                return Err("battle fallback texture is missing");
            }
            if !battler.texture_size.is_finite() || battler.texture_size.min_element() <= 0.0 {
                return Err("battle texture geometry is invalid");
            }
        }
        if self.cues.len() > 8 {
            return Err("unbounded battle cue frame");
        }
        for cue in &self.cues {
            if !cue.progress.is_finite() || !(0.0..=1.0).contains(&cue.progress) {
                return Err("battle cue progress is invalid");
            }
        }
        if let Some(source) = &self.source {
            if source.objects.len() > 10 {
                return Err("unbounded source battle objects");
            }
            if !source.screen_offset.is_finite()
                || source
                    .battler_offsets
                    .iter()
                    .any(|offset| !offset.is_finite())
                || source
                    .battler_palettes
                    .iter()
                    .flatten()
                    .flatten()
                    .any(|v| !v.is_finite())
            {
                return Err("invalid source battle presentation");
            }
            for object in &source.objects {
                if object.slot >= 10
                    || object.texture == Handle::default()
                    || object.neutral_texture == Handle::default()
                    || !object.center.is_finite()
                    || !object.size.is_finite()
                    || object.size.min_element() <= 0.0
                {
                    return Err("invalid source battle object");
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame() -> VisualBattleFrame {
        VisualBattleFrame {
            active: true,
            map_id: Arc::from("Route29"),
            battlers: [
                Some(VisualBattleBattler {
                    side: VisualBattleSide::Player,
                    species_id: Arc::from("CYNDAQUIL"),
                    party_index: Some(2),
                    texture: Handle::weak_from_u128(4),
                    texture_size: Vec2::splat(56.0),
                    visible: true,
                    allow_species_model: true,
                    shiny: false,
                }),
                None,
            ],
            ..Default::default()
        }
    }
    #[test]
    fn inactive_battle_is_valid_and_empty() {
        assert_eq!(VisualBattleFrame::default().validate(), Ok(()));
    }
    #[test]
    fn valid_battle_preserves_actual_party_identity() {
        let frame = frame();
        assert_eq!(frame.validate(), Ok(()));
        assert_eq!(frame.battlers[0].as_ref().unwrap().party_index, Some(2));
    }
    #[test]
    fn rejected_progress_cannot_reach_animation_math() {
        let mut frame = frame();
        frame.cues.push(VisualBattleCue {
            kind: VisualBattleCueKind::Move,
            side: VisualBattleSide::Player,
            move_id: Arc::from("TACKLE"),
            element: Arc::from("NORMAL"),
            progress: f32::NAN,
            damaging: true,
        });
        assert!(frame.validate().is_err());
    }
    #[test]
    fn side_and_source_art_are_required() {
        let mut frame = frame();
        frame.battlers[0].as_mut().unwrap().side = VisualBattleSide::Enemy;
        assert!(frame.validate().is_err());
        frame.battlers[0].as_mut().unwrap().side = VisualBattleSide::Player;
        frame.battlers[0].as_mut().unwrap().texture = Handle::default();
        assert!(frame.validate().is_err());
    }
}
