//! Verified overworld contact for an already committed battle. Optional
//! evidence is deliberately absent when no matching rendered scene was seen.
use crate::{VisualActorId, VisualTile, VisualWorldFrame};
use bevy::prelude::{Handle, IVec2, Image, Resource, UVec2, Vec2};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualBattleSourceMovement {
    Normal,
    Bike,
    Skate,
    Surf,
    SurfPika,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisualBattleSourcePose {
    pub map_id: Arc<str>,
    pub source_frame: u64,
    /// Authoritative map coordinates, +X east and +Y south, 16 source pixels per tile.
    pub core_tile: IVec2,
    /// Unit direction in core coordinates, +X east and +Y south.
    pub facing: IVec2,
    pub movement: VisualBattleSourceMovement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisualBattleObjectTarget {
    pub object_identifier: Arc<str>,
    /// Authored interaction script of the checked object; a field item can
    /// dispatch a different script, recorded separately in trigger_script.
    pub object_script: Arc<str>,
    pub core_tile: IVec2,
    pub trigger_script: Arc<str>,
    pub battle_source_script: Arc<str>,
    pub startbattle_command_index: usize,
}

/// Authoritative contact supplied by the encounter's checked production path.
/// A water target is a map contact, not a rendered actor or a claim that the
/// selected species can swim. Missing target evidence is never synthesized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VisualBattleTarget {
    Object(VisualBattleObjectTarget),
    FishingWater { core_tile: IVec2 },
}

impl VisualBattleTarget {
    pub fn core_tile(&self) -> IVec2 {
        match self {
            Self::Object(target) => target.core_tile,
            Self::FishingWater { core_tile } => *core_tile,
        }
    }
}

/// A frozen terrain-only frame. Retain only once per checked candidate, not
/// once per tick. Texture handles and exact cells identify the rendered inputs.
/// Handles do not freeze asset pixels: the classic compositor may update the
/// same image later. Consumers may pin already-built geometry/materials after
/// exact correspondence; this value does not authorize rebuilding from mutable
/// map_texture pixels as though they were a frozen image.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualBattleTerrainEvidence {
    pub map_id: Arc<str>,
    /// Source extent carried by the exact rendered terrain, in core tiles.
    /// None is unknown; the rendered border/connection halo is never substituted.
    pub source_map_size_core_tiles: Option<UVec2>,
    pub terrain_revision: u64,
    /// Coordinates are source 8x8 tiles, NOT core 16x16 tiles.
    pub grid_origin: IVec2,
    pub grid_size: UVec2,
    pub center: Vec2,
    pub viewport_size: Vec2,
    pub tile_size: Vec2,
    pub map_texture: Handle<Image>,
    pub tiles: Arc<[VisualTile]>,
}

impl VisualBattleTerrainEvidence {
    /// Compare the actual built frame, never a desired/pending renderer key.
    /// Actor animation is intentionally excluded from terrain correspondence.
    pub fn matches_built_frame(&self, frame: &VisualWorldFrame) -> bool {
        frame.active
            && self.map_id == frame.map_id
            && self.source_map_size_core_tiles == frame.source_map_size_core_tiles
            && self.terrain_revision == frame.terrain_revision
            && self.grid_origin == frame.grid_origin
            && self.grid_size == frame.grid_size
            && self.center == frame.center
            && self.viewport_size == frame.viewport_size
            && self.tile_size == frame.tile_size
            && self.map_texture == frame.map_texture
            && self.tiles.as_ref() == frame.tiles.as_slice()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualBattleAnchorFrame {
    pub terrain: VisualBattleTerrainEvidence,
    pub source_actor: VisualActorId,
    /// None for checked map contact (for example fishing water). Never invent
    /// an actor slot for an encounter that had no visible overworld enemy.
    pub target_actor: Option<VisualActorId>,
    /// Frozen feet in VisualWorldFrame world-pixel coordinates (+Y north).
    /// The source is verified sprite footing. An object target uses its sprite
    /// footing; fishing uses the checked water tile's south-center support
    /// point. Neither is a promise of built terrain elevation.
    pub source_foot: Vec2,
    pub target_foot: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VisualBattleLocation {
    /// Same controller-owned serial as BattlePresentationOrigin; not an RNG,
    /// save, replay, or gameplay identity.
    pub generation: u64,
    pub source: VisualBattleSourcePose,
    pub target: VisualBattleTarget,
    /// Actual source map extent in core tiles, northwest origin (0, 0),
    /// exclusive southeast bound. Border/connection halo is not map acreage.
    pub source_map_size_core_tiles: Option<UVec2>,
    /// None for a cold, missing, stale, or unverified preceding world frame.
    pub anchors: Option<Arc<VisualBattleAnchorFrame>>,
}

/// Only a real encounter can populate this resource. Field-action candidates
/// stay private to the controller. Lifetime follows retained battle narration.
#[derive(Resource, Clone, Debug, Default)]
pub struct BattleLocationFrame {
    pub location: Option<Arc<VisualBattleLocation>>,
}
