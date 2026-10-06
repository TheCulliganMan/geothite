//! Read-only battle presentation. These values describe the scene currently on
//! screen, not the runtime's already-resolved future turn. Renderers must never
//! use animation completion to advance a turn or infer a battle result.
use bevy::prelude::{Handle, Image, Rect, Resource, SystemSet, UVec2, Vec2};
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
    /// Listed physical dimension from the currently presented species' Pokédex
    /// entry, in metres. Some long-bodied species are measured along the body.
    /// None means the content does not establish a physical size.
    pub pokedex_size_m: Option<f32>,
    pub party_index: Option<usize>,
    pub texture: Handle<Image>,
    pub texture_size: Vec2,
    /// Full sprite rectangle in the original 160x144 LCD, in pixels, Y down.
    pub source_rect: Rect,
    /// Nontransparent source-pixel bounds retained for attack/appearance mapping.
    pub source_opaque_rect: Rect,
    pub visible: bool,
    /// Substitute/minimize retain their honest source art instead of using a
    /// species mesh that would communicate the wrong visible battle state.
    pub allow_species_model: bool,
    pub shiny: bool,
}

/// Current source picture operation. These phases never advance a clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualCapturePicture {
    Full,
    /// ReturnMon/EnterMon's intact 8x8 tiles in the fixed 7x7 front slot.
    Tiles(u8),
    Hidden,
}

/// Present capture state only: no RNG, predicted result, or completion command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisualBattleCapture {
    pub frame: u16,
    pub ball_id: Arc<str>,
    /// False while the existing narration still owns the start boundary.
    pub presented: bool,
    pub enemy_picture: VisualCapturePicture,
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

/// Identity of one source-proven complete effect assembly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VisualBattleSourceAssembly {
    /// Source spawn event identity; a reused OAM slot is not an assembly.
    pub event_index: usize,
    pub spawn_frame: u16,
}

/// Placement is admitted only by verified source callback semantics. All
/// pieces of an assembly retain one mapping; this data never owns a clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VisualBattleSourcePlacement {
    BattlerLocal {
        side: VisualBattleSide,
    },
    IndependentTransport {
        from: VisualBattleSide,
        assembly: VisualBattleSourceAssembly,
        /// Current, unclipped displayed LCD pivot, after the original turn
        /// coordinate fix. Clipping must not move this registration point.
        pivot: Vec2,
    },
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
    /// Normalized source texture crop after clipping to the original LCD.
    /// Texture handles remain shared; this never resizes/reuploads their pixels.
    pub uv_rect: Rect,
    /// None retains the complete existing source-canvas projection.
    pub placement: Option<VisualBattleSourcePlacement>,
}

/// A source BATTLEROBJ copy. This describes presentation only; it never owns
/// a clock or advances the animation. Crops use the same LCD attack plane as SCX/SCY.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualBattleBattlerRows {
    /// Half-open LCD Y interval copied to stationary OAM.
    pub source_y: Vec2,
    /// The source clears the corresponding BG tiles one tick after allocation.
    pub bg_cleared: bool,
    /// Native HUD depth derived from the same source slot and whole-frame
    /// OBJ/BG-priority decision as explicit source OAM, never inferred in 3D.
    pub oam_depth: f32,
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
    /// Original SCX sampling offsets in source LCD pixels (X right).
    /// Output column x samples column x + offset for that output row.
    pub line_x_offsets: Option<[i8; 0x5f]>,
    /// Original vertical background sampling in source LCD pixels (Y down).
    /// Output row y samples row y + offset; OAM objects and HUD are unwarped.
    pub line_y_offsets: Option<[i8; 0x5f]>,
    pub objects: Vec<VisualBattleSourceObject>,
    /// Opt-in, bounded Tackle/Water Gun prototype. None preserves the normal
    /// scene path and avoids all extra actor rendering.
    pub battler_rows: [Option<VisualBattleBattlerRows>; 2],
}

/// The immersive viewport size in the native 2D pass and physical pixels.
/// The bridge publishes this after normal fullscreen layout.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct VisualBattleCanvas {
    pub size: Vec2,
    pub physical_size: UVec2,
}
impl Default for VisualBattleCanvas {
    fn default() -> Self {
        Self {
            size: Vec2::new(640.0, 576.0),
            physical_size: UVec2::ZERO,
        }
    }
}

/// PostUpdate boundary: extract the viewport before sizing its 3D target.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BattleCanvasExtract;

#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct VisualBattleFrame {
    pub active: bool,
    /// Preserve exact original clipping/row sampling/transition sequences until
    /// the 3D renderer can reproduce them. Source data remains available to QA.
    pub use_source_scene: bool,
    pub map_id: Arc<str>,
    pub environment: VisualBattleEnvironment,
    pub battlers: [Option<VisualBattleBattler>; 2],
    pub cues: Vec<VisualBattleCue>,
    pub source: Option<VisualBattleSourceFrame>,
    /// The entire visible capture lifetime, independent of OAM or cue presence.
    pub capture: Option<VisualBattleCapture>,
}
impl VisualBattleFrame {
    /// Source LCD scroll applies to the rendered BG battlers, never OBJ/HUD.
    pub fn uses_source_scanlines(&self) -> bool {
        self.source.as_ref().is_some_and(|source| {
            source.line_x_offsets.is_some() || source.line_y_offsets.is_some()
        })
    }
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
            if !battler.source_rect.min.is_finite()
                || !battler.source_rect.max.is_finite()
                || battler.source_rect.size().min_element() <= 0.0
                || !battler.source_opaque_rect.min.is_finite()
                || !battler.source_opaque_rect.max.is_finite()
                || battler.source_opaque_rect.size().min_element() <= 0.0
            {
                return Err("battle source geometry is invalid");
            }
            if !battler.texture_size.is_finite() || battler.texture_size.min_element() <= 0.0 {
                return Err("battle texture geometry is invalid");
            }
            if battler
                .pokedex_size_m
                .is_some_and(|size| !size.is_finite() || size <= 0.0)
            {
                return Err("battle physical size is invalid");
            }
        }
        if let Some(capture) = &self.capture {
            if capture.ball_id.is_empty()
                || matches!(capture.enemy_picture, VisualCapturePicture::Tiles(n) if ![3, 5, 7].contains(&n))
                || (!capture.presented && capture.enemy_picture != VisualCapturePicture::Full)
            {
                return Err("invalid current capture picture");
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
            for rows in source.battler_rows.iter().flatten() {
                if !rows.oam_depth.is_finite() || !rows.source_y.is_finite()
                    || rows.source_y.x < 0.0
                    || rows.source_y.y > 144.0
                    || rows.source_y.x >= rows.source_y.y
                {
                    return Err("invalid extracted battler rows");
                }
            }
            if let [Some(player), Some(enemy)] = source.battler_rows {
                if player.oam_depth != enemy.oam_depth {
                    return Err("mixed extracted row depths require separate overlays");
                }
            }
            for object in &source.objects {
                if object.slot >= 10
                    || object.texture == Handle::default()
                    || object.neutral_texture == Handle::default()
                    || !object.center.is_finite()
                    || !object.size.is_finite()
                    || object.size.min_element() <= 0.0
                    || !object.uv_rect.min.is_finite()
                    || !object.uv_rect.max.is_finite()
                    || object.uv_rect.min.min_element() < 0.0
                    || object.uv_rect.max.max_element() > 1.0
                    || object.uv_rect.size().min_element() <= 0.0
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
    #[test]
    fn physical_dimensions_reject_invalid_values_without_inventing_missing_data() {
        let mut valid = frame();
        assert_eq!(valid.validate(), Ok(()));
        for size in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            valid.battlers[0].as_mut().unwrap().pokedex_size_m = Some(size);
            assert_eq!(valid.validate(), Err("battle physical size is invalid"));
        }
        valid.battlers[0].as_mut().unwrap().pokedex_size_m = None;
        valid.battlers[0].as_mut().unwrap().allow_species_model = false;
        assert_eq!(valid.validate(), Ok(()));
    }
    fn frame() -> VisualBattleFrame {
        VisualBattleFrame {
            active: true,
            map_id: Arc::from("Route29"),
            battlers: [
                Some(VisualBattleBattler {
                    side: VisualBattleSide::Player,
                    species_id: Arc::from("CYNDAQUIL"),
                    pokedex_size_m: Some(0.508),
                    party_index: Some(2),
                    texture: Handle::weak_from_u128(4),
                    texture_size: Vec2::splat(56.0),
                    source_rect: Rect::new(16.0, 48.0, 72.0, 104.0),
                    source_opaque_rect: Rect::new(16.0, 48.0, 72.0, 104.0),
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
