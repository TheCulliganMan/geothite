#![forbid(unsafe_code)]

//! Read-only presentation data shared by the classic world renderer and
//! optional world-rendering mods.
//!
//! The game runtime remains authoritative. Render mods should consume
//! [`VisualWorldFrame`] through `Res<VisualWorldFrame>` during
//! [`WorldRenderSet::RenderSync`] and must not feed presentation state back
//! into simulation.

mod battle;
mod battle_location;
mod streamed_images;
pub use battle::*;
pub use battle_location::*;
pub use streamed_images::{stream_composed_image, is_streamed_image};

use std::{collections::HashSet, sync::Arc};

use bevy::prelude::{
    App, Handle, Image, IntoScheduleConfigs, Plugin, Resource, SystemSet, UVec2, UVec4, Update, Vec2, Vec4,
};

/// Ordered phases for publishing and consuming one visual world frame.
///
/// [`WorldRenderSet::ClassicWorld`] updates the faithful 2D presentation,
/// [`WorldRenderSet::PresentationExtract`] publishes its read-only frame, and
/// [`WorldRenderSet::RenderSync`] lets optional renderers synchronize their
/// own presentation from that frame.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorldRenderSet {
    ClassicWorld,
    PresentationExtract,
    RenderSync,
}

/// Installs the render-phase ordering into Bevy's `Update` schedule.
pub fn configure_world_render_sets(app: &mut App) -> &mut App {
    app.configure_sets(
        Update,
        (
            WorldRenderSet::ClassicWorld,
            WorldRenderSet::PresentationExtract,
            WorldRenderSet::RenderSync,
        )
            .chain(),
    )
}

/// Initializes the visual frame resource and installs the render-phase order.
#[derive(Clone, Copy, Debug, Default)]
pub struct VisualWorldRenderPlugin;

impl Plugin for VisualWorldRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualSurfRider>()
            .init_resource::<VisualPokemonDimensions>()
            .init_resource::<VisualPlayerTraversal>()
            .init_resource::<VisualWorldLighting>()
            .init_resource::<VisualWorldClock>()
            .init_resource::<VisualWorldEntryEffect>()
            .init_resource::<VisualTileAnimationFrame>();
        configure_world_render_sets(app);
        streamed_images::install(app);
    }
}

/// Controller-authored battle-entry effects applied to the selected 3D field.
/// Packed masks use one bit per native 20x18 cell; offsets are native pixels.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct VisualWorldEntryEffect {
    pub active: bool,
    pub dark: f32,
    pub light: f32,
    pub line_x_offsets: [i8; 144],
    pub black_cells: [UVec4; 3],
    pub trainer_cells: [UVec4; 3],
    pub trainer_pixels: UVec4,
    pub trainer_palette: [Vec4; 4],
}
impl Default for VisualWorldEntryEffect {
    fn default() -> Self {
        Self { active: false, dark: 0., light: 0., line_x_offsets: [0; 144],
            black_cells: [UVec4::ZERO; 3], trainer_cells: [UVec4::ZERO; 3],
            trainer_pixels: UVec4::ZERO, trainer_palette: [Vec4::ZERO; 4] }
    }
}

/// The palette period of the successfully rendered classic map. This is
/// presentation metadata; optional renderers never advance a game clock.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VisualWorldLighting {
    Morning,
    #[default]
    Day,
    Night,
    /// Authored unlit map without Flash; terrain is hidden, sprites remain visible.
    Dark,
}

impl VisualWorldLighting {
    pub fn from_source_palette(period: &str) -> Self {
        match period {
            "morn" => Self::Morning,
            "nite" => Self::Night,
            "dark" | "darkness" => Self::Dark,
            _ => Self::Day,
        }
    }
}

/// Known open-air terraces sharing otherwise enclosed source atlases.
/// Their 3D sky can use the RTC while native fixed-palette pixels stay exact.
pub fn is_open_air_rooftop(map:&str,tileset:&str)->bool {
    matches!((map,tileset),("CeladonMansionRoof","mansion")
        |("GoldenrodDeptStoreRoof","mart")|("TinTowerRoof","tower"))
}

/// Read-only local clock for PALETTE_AUTO maps and known open-air terraces.
/// `None` keeps enclosed fixed-palette scenes on their source period.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisualWorldClock {
    pub minute_of_day: Option<u16>,
    pub second_of_minute: u8,
}

impl VisualWorldClock {
    pub fn seconds_of_day(self) -> Option<u32> {
        self.minute_of_day
            .map(|minute| u32::from(minute) * 60 + u32::from(self.second_of_minute.min(59)))
    }
}

/// Exact source identity for one 8x8 tile in Crystal's 4x4 metatile layout.
///
/// Optional renderers may use this stable, presentation-only identity to look
/// up an authored shape profile. The host deliberately does not derive shape
/// or height from gameplay collision permissions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VisualTileSource {
    pub tileset_id: Arc<str>,
    pub metatile_id: u16,
    pub subtile_column: u8,
    pub subtile_row: u8,
    pub tile_index: u16,
}

/// One tile in the visual map grid.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VisualTile {
    pub column: u32,
    pub row: u32,
    pub source: VisualTileSource,
    /// Exact 8x8 image sampled from the classic renderer when this cell was
    /// published. A retained grid may keep an earlier authenticated animation
    /// phase. This gives optional renderers non-stretched top/edge source art
    /// without exposing asset-root paths or gameplay data.
    pub texture: Handle<Image>,
    /// Complete immutable authored texture-phase family, shared by every cell
    /// using this animation from the same host art-cache entry. `texture` is
    /// always a member. Static or otherwise unproven art has no family.
    ///
    /// This authenticates texture provenance only, never geometry. Consumers
    /// may accept different member phases only when their validated source
    /// profile proves geometry invariant across this exact family. Overlapping
    /// families do not establish correspondence, and palette/art-cache changes
    /// must publish a distinct family. Phase changes do not change geometry
    /// revisions or require cloning the complete published grid.
    pub animation_frames: Option<Arc<[Handle<Image>]>>,
    /// Whether the tile belongs to the classic map foreground-priority layer.
    /// This is compositing metadata, not a height or shape signal.
    pub priority: bool,
}

/// Current authenticated phases of animated cells, independent of geometry.
/// Consumers must match map, source identity and immutable phase family before
/// updating a retained atlas. No simulation clock is exposed or advanced.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct VisualTileAnimationFrame {
    pub map_id: Arc<str>,
    pub grid_origin: bevy::prelude::IVec2,
    pub tiles: Vec<VisualTile>,
}

/// Stable identity for an actor across visual frame publications.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualActorId {
    Player,
    /// First party member, presented only by the optional 3D overworld.
    PartyFollower,
    /// The actual Surf user, underneath the separately posed trainer.
    SurfMount,
    RemotePlayer(u64),
    Object(u32),
    /// Presentation-only card emitted by the faithful world renderer. These
    /// IDs never enter gameplay state or object-slot identity.
    Effect(VisualEffectId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualEffectId {
    GrassRustle,
    HealMachineBall(u8),
    HealMachineLamp(u8),
}

/// 3D rider appearance without replacing the source Surf actor identity.
#[derive(Resource, Default, Clone, Copy)]
pub struct VisualSurfRider {
    pub female: bool,
}

/// Physical dimensions from the loaded pack's Pokédex, shared by every
/// overworld role. A carrying Pokémon keeps the same dimension as a follower.
#[derive(Resource, Default)]
pub struct VisualPokemonDimensions {
    pub meters: std::collections::BTreeMap<String, f32>,
}

/// Read-only endpoints and timing of the player's currently presented step.
/// Coordinates are ground feet in the current visual frame, without sprite lift.
#[derive(Resource, Default, Clone, Debug)]
pub struct VisualPlayerTraversal {
    pub map_id: Arc<str>,
    pub step: Option<VisualTraversalStep>,
}

#[derive(Clone, Copy, Debug)]
pub struct VisualTraversalStep {
    pub from: Vec2,
    pub to: Vec2,
    pub progress: f32,
    pub ledge_jump: bool,
}

/// Presentation state for one actor visible in the world pass.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualActor {
    pub id: VisualActorId,
    /// Stable presentation-art identity (for example `player`, `rock`, or
    /// `effect_grass_rustle`). Optional renderers may use this to choose
    /// geometry; it carries no gameplay or collision semantics.
    pub source_id: Arc<str>,
    /// Pack-backed Pokemon appearance, independent of shared sprite identity.
    pub model_palette: Option<VisualPokemonModelPalette>,
    pub texture: Handle<Image>,
    /// Actor center in the same world-pixel coordinate system as the map.
    pub center: Vec2,
    pub size: Vec2,
    pub flip_x: bool,
    /// Whether this actor is drawn above foreground-priority map tiles.
    pub above_priority: bool,
    /// Authoritative world-facing vector (+X east, +Y north), independent of camera orbit.
    /// None for presentation effects and legacy remote actors.
    pub facing: Option<Vec2>,
}

/// Authored one-way jump edge in map-aligned 8x8 visual-tile coordinates.
/// `descent` points from the jumpable upper side toward the lower landing side.
/// These edges describe presentation only; the runtime owns movement permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VisualJumpLedge {
    pub start: bevy::prelude::IVec2,
    pub end: bevy::prelude::IVec2,
    pub descent: bevy::prelude::IVec2,
}

/// Immutable-by-convention snapshot consumed by optional world renderers.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualTerraceDatum {
    /// Complete source-map/connection extent, independent of the camera crop.
    pub origin: bevy::prelude::IVec2,
    pub size: UVec2,
    /// Calibrated NW/SW/SE/NE corner levels, in authored ledge-rise units.
    /// Standing-water entries stay flat and never carry an inland slope.
    pub levels: Arc<[[f32;4]]>,
    /// Source-complete water-backed barrier cells, sorted by (y,x), not camera crop.
    /// Empty preserves the legacy authored-underlay classification.
    pub water_barriers: Arc<[bevy::prelude::IVec2]>,
    /// Source-proven inland river level and its cells, including gaps in the
    /// connected-map rectangle. Rendered border water continues this plane.
    pub open_water: Option<VisualOpenWaterDatum>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct VisualOpenWaterDatum {
    pub level: f32,
    pub cells: Arc<[bevy::prelude::IVec2]>,
}
impl VisualTerraceDatum {
    pub fn water_barrier_at(&self,point:bevy::prelude::IVec2)->bool {
        self.water_barriers.binary_search_by_key(&(point.y,point.x),|p|(p.y,p.x)).is_ok()
    }
    pub fn open_water_at(&self,point:bevy::prelude::IVec2)->Option<f32> {
        let river=self.open_water.as_ref()?;
        let p=point-self.origin;
        (p.x<0 || p.y<0 || p.x>=self.size.x as i32 || p.y>=self.size.y as i32
            || river.cells.binary_search_by_key(&(point.y,point.x),|p|(p.y,p.x)).is_ok())
            .then_some(river.level)
    }
    pub fn valid_open_water(&self)->bool {
        self.open_water.as_ref().is_none_or(|river|river.level.is_finite() && river.level>=0.
            && river.cells.len()as u64<=u64::from(self.size.x)*u64::from(self.size.y)
            && river.cells.iter().all(|p| {
                let x=i64::from(p.x)-i64::from(self.origin.x);
                let y=i64::from(p.y)-i64::from(self.origin.y);
                x>=0 && y>=0 && x<i64::from(self.size.x) && y<i64::from(self.size.y)
            }) && river.cells.windows(2).all(|p|(p[0].y,p[0].x)<(p[1].y,p[1].x)))
    }
    pub fn valid_water_barriers(&self)->bool {
        self.water_barriers.len() as u64<=u64::from(self.size.x)*u64::from(self.size.y)
            && self.water_barriers.iter().all(|p| {
                let x=i64::from(p.x)-i64::from(self.origin.x);
                let y=i64::from(p.y)-i64::from(self.origin.y);
                x>=0 && y>=0 && x<i64::from(self.size.x) && y<i64::from(self.size.y)
            }) && self.water_barriers.windows(2).all(|p|(p[0].y,p[0].x)<(p[1].y,p[1].x))
    }
}

/// A source map rectangle in the active map's 8x8 tile coordinate space.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VisualSourceRegion {
    pub map_id: Arc<str>,
    pub origin: bevy::prelude::IVec2,
    pub size: UVec2,
}

/// Immutable-by-convention snapshot consumed by optional world renderers.
/// Inactive frames are empty and valid; active frames require real source art
/// and valid finite geometry before a renderer can consume them.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct VisualWorldFrame {
    pub active: bool,
    /// Stable compiled map identifier for presentation-profile selection.
    pub map_id: Arc<str>,
    /// Authenticated map placements in the active map's 8x8 source-tile space.
    /// Base map first; connected rectangles follow host resolution order.
    pub source_regions: Arc<[VisualSourceRegion]>,
    /// Actual source-map extent captured with this rendered terrain grid, in
    /// core 16x16 source-pixel tiles (+X east, +Y south). Northwest is (0, 0);
    /// the southeast bound is exclusive. Border/connection halo is excluded.
    /// None preserves legacy/unknown evidence; never infer bounds from grid_size.
    /// Consumers must include this value in built-terrain cache compatibility.
    pub source_map_size_core_tiles: Option<UVec2>,
    /// Read-only source warp positions in core tiles. Interior architecture
    /// leaves boundary exits open; these never grant movement permissions.
    pub boundary_warps: Arc<[bevy::prelude::IVec2]>,
    /// Complete active-map/connection ledges, retained independently of the
    /// camera crop so terrace elevation cannot reset while walking.
    pub jump_ledges: Arc<[VisualJumpLedge]>,
    /// Source-complete coastal land-domain calibration. Legacy frames omit it.
    pub terrace_datum: Option<Arc<VisualTerraceDatum>>,
    /// Changes whenever the visible tile sources or their live art change.
    pub terrain_revision: u64,
    /// Map tile coordinate of the grid northwest corner; independent of camera interpolation.
    pub grid_origin: bevy::prelude::IVec2,
    pub map_texture: Handle<Image>,
    /// Center of the composited map surface / visual viewport in world pixels.
    pub center: Vec2,
    /// Horizontal camera-rig origin in the same coordinates as terrain/actors.
    /// Zero preserves screen-centred producers. Map-anchored producers move
    /// this origin while retaining static scenery transforms.
    pub view_center: Vec2,
    /// Visible world extent in world pixels.
    pub viewport_size: Vec2,
    /// Width and height of a visual tile in world pixels.
    pub tile_size: Vec2,
    pub grid_size: UVec2,
    pub tiles: Vec<VisualTile>,
    pub actors: Vec<VisualActor>,
}

impl VisualWorldFrame {
    /// Translate source-cell coordinates into a reciprocally placed connected
    /// map. This proves a coordinate basis only, not retained geometry coverage.
    pub fn connected_source_shift(&self, next: &Self) -> Option<bevy::prelude::IVec2> {
        use bevy::prelude::IVec2;
        if self.map_id == next.map_id { return Some(IVec2::ZERO); }
        let a = self.source_regions.first()?;
        let b = next.source_regions.first()?;
        if a.map_id != self.map_id || b.map_id != next.map_id
            || a.origin != IVec2::ZERO || b.origin != IVec2::ZERO { return None; }
        let target = self.source_regions.iter().find(|r| r.map_id == next.map_id)?;
        let back = next.source_regions.iter().find(|r| r.map_id == self.map_id)?;
        let add = |point: IVec2, shift: IVec2| Some(IVec2::new(
            point.x.checked_add(shift.x)?, point.y.checked_add(shift.y)?));
        let shift = back.origin;
        if target.size != b.size || back.size != a.size
            || add(target.origin, shift)? != IVec2::ZERO { return None; }
        for old in self.source_regions.iter() {
            for new in next.source_regions.iter().filter(|r| r.map_id == old.map_id) {
                if old.size != new.size || add(old.origin, shift)? != new.origin { return None; }
            }
        }
        Some(shift)
    }

    pub fn source_region_at(&self, column: u32, row: u32) -> Option<&VisualSourceRegion> {
        let point = self.grid_origin + bevy::prelude::IVec2::new(column as i32, row as i32);
        let contains = |region: &&VisualSourceRegion| {
            let local = point - region.origin;
            local.cmpge(bevy::prelude::IVec2::ZERO).all()
                && local.cmplt(region.size.as_ivec2()).all()
        };
        let base = self.source_regions.first()?;
        if contains(&base) { return Some(base); }
        self.source_regions.iter().skip(1).rev().find(contains).or(Some(base))
    }
    /// Presentation-only border fill follows the nearest native rectangle.
    /// Authored ownership remains available through `source_region_at`.
    fn scenery_region_at(&self, column: u32, row: u32) -> Option<&VisualSourceRegion> {
        let point=self.grid_origin.as_i64vec2()+bevy::math::I64Vec2::new(i64::from(column),i64::from(row));
        let source=self.source_region_at(column,row)?;
        let local=point-source.origin.as_i64vec2();
        if local.cmpge(bevy::math::I64Vec2::ZERO).all()
            && local.cmplt(source.size.as_i64vec2()).all() { return Some(source); }
        self.source_regions.iter().min_by_key(|region| {
            let lo=region.origin.as_i64vec2();
            let hi=lo+region.size.as_i64vec2()-bevy::math::I64Vec2::ONE;
            let d=(lo-point).max(bevy::math::I64Vec2::ZERO)+(point-hi).max(bevy::math::I64Vec2::ZERO);
            (d.element_sum(),region.map_id.as_ref())
        })
    }
    pub fn scenery_map_at(&self, column: u32, row: u32) -> &str {
        self.scenery_region_at(column,row).map_or(self.map_id.as_ref(),|r|r.map_id.as_ref())
    }
    pub fn scenery_grid_origin_at(&self, column: u32, row: u32) -> bevy::prelude::IVec2 {
        self.grid_origin-self.scenery_region_at(column,row).map_or(bevy::prelude::IVec2::ZERO,|r|r.origin)
    }
    pub fn source_map_at(&self, column: u32, row: u32) -> &str {
        self.source_region_at(column, row).map_or(self.map_id.as_ref(), |region| region.map_id.as_ref())
    }
    pub fn source_grid_origin_at(&self, column: u32, row: u32) -> bevy::prelude::IVec2 {
        self.grid_origin - self.source_region_at(column, row)
            .map_or(bevy::prelude::IVec2::ZERO, |region| region.origin)
    }
    /// Rebase presentation coordinates into stable source-map space without
    /// changing source tiles, actor offsets, geometry or gameplay state.
    /// Returns the translation to apply to external presentation endpoints.
    pub fn anchor_to_map(&mut self) -> Vec2 {
        if !self.active { return Vec2::ZERO; }
        let center = self.grid_origin.as_vec2() * self.tile_size * Vec2::new(1., -1.);
        let shift = center - self.center;
        self.center = center;
        self.view_center += shift;
        for actor in &mut self.actors { actor.center += shift; }
        shift
    }

    /// Checks the geometry and identity invariants required by render mods.
    ///
    /// Inactive frames are always accepted because consumers must ignore all
    /// remaining fields when `active` is false.
    pub fn validate(&self) -> Result<(), VisualWorldFrameError> {
        if !self.active {
            return Ok(());
        }
        if self.source_regions.len() > 512
            || self.source_regions.first().is_some_and(|base|
                base.map_id != self.map_id || base.origin != bevy::prelude::IVec2::ZERO)
            || self.source_regions.iter().any(|region| region.map_id.is_empty()
                || region.size.min_element() == 0 || region.size.max_element() > 1_000_000
                || region.origin.as_i64vec2().abs().max_element() > 1_000_000) {
            return Err(VisualWorldFrameError::InvalidSourceRegions);
        }

        if self.map_id.is_empty() {
            return Err(VisualWorldFrameError::EmptyMapId);
        }
        if self
            .source_map_size_core_tiles
            .is_some_and(|size| size.x == 0 || size.y == 0)
        {
            return Err(VisualWorldFrameError::InvalidSourceMapSize);
        }
        if self.map_texture == Handle::<Image>::default() {
            return Err(VisualWorldFrameError::MissingMapTexture);
        }
        if self.terrace_datum.as_ref().is_some_and(|d|d.size.x==0 || d.size.y==0
            || u64::from(d.size.x)*u64::from(d.size.y)>2_000_000
            || d.levels.len()as u64!=u64::from(d.size.x)*u64::from(d.size.y)
            || d.levels.iter().flatten().any(|&v|!v.is_finite() || v<0.) || !d.valid_water_barriers() || !d.valid_open_water()) {
            return Err(VisualWorldFrameError::InvalidTerraceDatum);
        }
        if !self.center.is_finite() || !self.view_center.is_finite() {
            return Err(VisualWorldFrameError::NonFiniteCenter);
        }
        if !is_positive_finite(self.viewport_size) {
            return Err(VisualWorldFrameError::InvalidViewportSize);
        }
        if !is_positive_finite(self.tile_size) {
            return Err(VisualWorldFrameError::InvalidTileSize);
        }
        if self.grid_size.x == 0 || self.grid_size.y == 0 {
            return Err(VisualWorldFrameError::EmptyGrid);
        }
        let terrain_size = self.tile_size * self.grid_size.as_vec2();
        if self.viewport_size.x > terrain_size.x || self.viewport_size.y > terrain_size.y {
            return Err(VisualWorldFrameError::ViewportGridMismatch);
        }

        let expected_tile_count = u64::from(self.grid_size.x) * u64::from(self.grid_size.y);
        let actual_tile_count = self.tiles.len() as u64;
        if actual_tile_count != expected_tile_count {
            return Err(VisualWorldFrameError::TileCountMismatch {
                expected: expected_tile_count,
                actual: actual_tile_count,
            });
        }

        let mut tile_coordinates = HashSet::with_capacity(self.tiles.len());
        for tile in &self.tiles {
            if tile.column >= self.grid_size.x || tile.row >= self.grid_size.y {
                return Err(VisualWorldFrameError::TileOutsideGrid {
                    column: tile.column,
                    row: tile.row,
                });
            }
            if !tile_coordinates.insert((tile.column, tile.row)) {
                return Err(VisualWorldFrameError::DuplicateTile {
                    column: tile.column,
                    row: tile.row,
                });
            }
            if tile.texture == Handle::<Image>::default() {
                return Err(VisualWorldFrameError::MissingTileTexture {
                    column: tile.column,
                    row: tile.row,
                });
            }
            if tile.source.tileset_id.is_empty() {
                return Err(VisualWorldFrameError::EmptyTileSourceTileset {
                    column: tile.column,
                    row: tile.row,
                });
            }
            if tile.source.subtile_column >= 4 || tile.source.subtile_row >= 4 {
                return Err(VisualWorldFrameError::TileSourceOutsideMetatile {
                    column: tile.column,
                    row: tile.row,
                    subtile_column: tile.source.subtile_column,
                    subtile_row: tile.source.subtile_row,
                });
            }
        }

        let mut actor_ids = HashSet::with_capacity(self.actors.len());
        for actor in &self.actors {
            if actor.source_id.is_empty() {
                return Err(VisualWorldFrameError::EmptyActorSource(actor.id));
            }
            if actor.texture == Handle::<Image>::default() {
                return Err(VisualWorldFrameError::MissingActorTexture(actor.id));
            }
            if !actor.center.is_finite() {
                return Err(VisualWorldFrameError::NonFiniteActorCenter(actor.id));
            }
            if !is_positive_finite(actor.size) {
                return Err(VisualWorldFrameError::InvalidActorSize(actor.id));
            }
            if actor.facing.is_some_and(|direction| {
                !direction.is_finite() || direction.length_squared() < 0.0001
            }) {
                return Err(VisualWorldFrameError::InvalidActorFacing(actor.id));
            }
            if !actor_ids.insert(actor.id) {
                return Err(VisualWorldFrameError::DuplicateActor(actor.id));
            }
        }

        Ok(())
    }
}

fn is_positive_finite(value: Vec2) -> bool {
    value.is_finite() && value.x > 0.0 && value.y > 0.0
}

/// Why an active [`VisualWorldFrame`] cannot be consumed safely.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualWorldFrameError {
    EmptyMapId,
    InvalidSourceRegions,
    InvalidSourceMapSize,
    InvalidTerraceDatum,
    MissingMapTexture,
    NonFiniteCenter,
    InvalidViewportSize,
    InvalidTileSize,
    EmptyGrid,
    ViewportGridMismatch,
    TileCountMismatch {
        expected: u64,
        actual: u64,
    },
    TileOutsideGrid {
        column: u32,
        row: u32,
    },
    DuplicateTile {
        column: u32,
        row: u32,
    },
    MissingTileTexture {
        column: u32,
        row: u32,
    },
    EmptyTileSourceTileset {
        column: u32,
        row: u32,
    },
    TileSourceOutsideMetatile {
        column: u32,
        row: u32,
        subtile_column: u8,
        subtile_row: u8,
    },
    EmptyActorSource(VisualActorId),
    MissingActorTexture(VisualActorId),
    NonFiniteActorCenter(VisualActorId),
    InvalidActorSize(VisualActorId),
    InvalidActorFacing(VisualActorId),
    DuplicateActor(VisualActorId),
}

#[cfg(test)]
mod tests {
    #[test]
    fn connected_owner_and_local_seed_coordinates_survive_map_handoff() {
        use bevy::prelude::IVec2;
        let region = |map: &str, x, width| VisualSourceRegion {
            map_id: Arc::from(map), origin: IVec2::new(x, 0), size: UVec2::new(width, 40),
        };
        let mut before = active_frame();
        before.map_id = "Route29".into();
        before.grid_origin = IVec2::new(72, 0);
        before.source_regions = vec![region("Route29", 0, 80), region("NewBarkTown", 80, 40)].into();
        let mut after = before.clone();
        after.map_id = "NewBarkTown".into();
        after.grid_origin -= IVec2::new(80, 0);
        after.source_regions = vec![region("NewBarkTown", 0, 40), region("Route29", -80, 80)].into();
        for (column, owner) in [(4, "Route29"), (12, "NewBarkTown")] {
            assert_eq!(before.source_map_at(column, 8), owner);
            assert_eq!(after.source_map_at(column, 8), owner);
            let position = IVec2::new(column as i32, 8);
            assert_eq!(before.source_grid_origin_at(column, 8) + position,
                after.source_grid_origin_at(column, 8) + position);
        }
        for (column,owner) in [(4,"Route29"),(12,"NewBarkTown")] {
            assert_eq!(before.scenery_map_at(column,48),owner);
            assert_eq!(after.scenery_map_at(column,48),owner,
                "off-map 3D fill must not switch with the active map");
            let p=IVec2::new(column as i32,48);
            assert_eq!(before.scenery_grid_origin_at(column,48)+p,
                after.scenery_grid_origin_at(column,48)+p);
        }
        assert_eq!(before.connected_source_shift(&after), Some(IVec2::new(-80, 0)));
        assert_eq!(after.connected_source_shift(&before), Some(IVec2::new(80, 0)));
        let mut inconsistent = after.clone();
        Arc::make_mut(&mut inconsistent.source_regions)[1].origin.x += 1;
        assert_eq!(before.connected_source_shift(&inconsistent), None);
        assert_eq!(before.source_map_at(1000, 1000), "Route29", "off-map borders belong to the base");
        before.source_regions = Arc::from([]);
        assert_eq!(before.source_map_at(12, 8), "Route29", "legacy frames keep their map identity");
    }

    #[test]
    fn connected_regions_require_a_valid_base_and_bounded_dimensions() {
        let mut frame = active_frame();
        let valid = VisualSourceRegion { map_id: frame.map_id.clone(),
            origin: bevy::prelude::IVec2::ZERO, size: UVec2::splat(20) };
        frame.source_regions = vec![valid.clone()].into();
        assert!(frame.validate().is_ok());
        for invalid in [VisualSourceRegion { map_id: "other".into(), ..valid.clone() },
            VisualSourceRegion { size: UVec2::ZERO, ..valid.clone() },
            VisualSourceRegion { origin: bevy::prelude::IVec2::splat(i32::MIN), ..valid }] {
            frame.source_regions = vec![invalid].into();
            assert_eq!(frame.validate(), Err(VisualWorldFrameError::InvalidSourceRegions));
        }
    }

    use bevy::prelude::{App, IntoScheduleConfigs, ResMut, Resource, Update};

    use super::*;

    fn visual_tile(column: u32, row: u32, tile_index: u16) -> VisualTile {
        VisualTile {
            column,
            row,
            source: VisualTileSource {
                tileset_id: Arc::from("johto"),
                metatile_id: 3,
                subtile_column: u8::try_from(column).expect("test column fits u8"),
                subtile_row: u8::try_from(row).expect("test row fits u8"),
                tile_index,
            },
            texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(100 + u128::from(tile_index)), std::marker::PhantomData),
            animation_frames: None,
            priority: false,
        }
    }

    #[test]
    fn map_anchoring_preserves_source_and_screen_correspondence() {
        let mut frame = active_frame();
        frame.grid_origin = bevy::prelude::IVec2::new(13, -7);
        let source = frame.clone();
        let shift = frame.anchor_to_map();
        assert_eq!(frame.center, Vec2::new(13., 7.) * source.tile_size);
        assert_eq!(frame.center - frame.view_center, source.center);
        assert_eq!(frame.tiles, source.tiles);
        assert_eq!(frame.terrain_revision, source.terrain_revision);
        assert_eq!(frame.grid_origin, source.grid_origin);
        for (actor, old) in frame.actors.iter().zip(&source.actors) {
            assert_eq!(actor.center, old.center + shift);
            assert_eq!(actor.center-frame.center, old.center-source.center);
        }
        let anchored = frame.clone();
        assert_eq!(frame.anchor_to_map(), Vec2::ZERO);
        assert_eq!(frame, anchored);
        assert!(frame.validate().is_ok());
        frame.view_center.x = f32::NAN;
        assert!(frame.validate().is_err());
    }

    fn active_frame() -> VisualWorldFrame {
        VisualWorldFrame {
            jump_ledges: Default::default(),
            terrace_datum: None,
            boundary_warps: Default::default(),
            active: true,
            map_id: Arc::from("NewBarkTown"),
            source_regions: Arc::from([]),
            source_map_size_core_tiles: None,
            terrain_revision: 7,
            grid_origin: bevy::prelude::IVec2::ZERO,
            map_texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(1), std::marker::PhantomData),
            center: Vec2::new(8.0, 4.0),
            view_center: Vec2::ZERO,
            viewport_size: Vec2::new(16.0, 8.0),
            tile_size: Vec2::splat(8.0),
            grid_size: UVec2::new(2, 1),
            tiles: vec![
                visual_tile(0, 0, 7),
                VisualTile {
                    priority: true,
                    ..visual_tile(1, 0, 9)
                },
            ],
            actors: vec![VisualActor { model_palette: None,
                id: VisualActorId::Player,
                source_id: Arc::from("player"),
                texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(2), std::marker::PhantomData),
                center: Vec2::new(8.0, 4.0),
                size: Vec2::splat(16.0),
                flip_x: false,
                above_priority: false,
                facing: None,
            }],
        }
    }

    #[test]
    fn default_frame_is_inactive_and_valid() {
        let frame = VisualWorldFrame::default();

        assert!(!frame.active);
        assert!(frame.map_id.is_empty());
        assert_eq!(frame.source_map_size_core_tiles, None);
        assert_eq!(frame.terrain_revision, 0);
        assert_eq!(frame.map_texture, Handle::<Image>::default());
        assert!(frame.tiles.is_empty());
        assert!(frame.actors.is_empty());
        assert_eq!(frame.validate(), Ok(()));
    }

    #[test]
    fn active_frame_with_valid_geometry_is_accepted() {
        assert_eq!(active_frame().validate(), Ok(()));
    }

    #[test]
    fn active_frame_requires_a_stable_map_identity() {
        let mut frame = active_frame();
        frame.map_id = Arc::from("");

        assert_eq!(frame.validate(), Err(VisualWorldFrameError::EmptyMapId));
    }

    #[test]
    fn active_frame_requires_terrain_to_cover_the_viewport() {
        let mut frame = active_frame();
        frame.viewport_size.x += 1.0;

        assert_eq!(
            frame.validate(),
            Err(VisualWorldFrameError::ViewportGridMismatch)
        );
    }

    #[test]
    fn active_frame_allows_terrain_beyond_the_game_boy_viewport() {
        let mut frame = active_frame();
        frame.grid_size = UVec2::new(3, 1);
        frame.tiles.push(visual_tile(2, 0, 11));

        assert_eq!(frame.validate(), Ok(()));
    }

    #[test]
    fn active_frame_rejects_tiles_outside_the_grid() {
        let mut frame = active_frame();
        frame.tiles[1].column = frame.grid_size.x;

        assert_eq!(
            frame.validate(),
            Err(VisualWorldFrameError::TileOutsideGrid { column: 2, row: 0 })
        );
    }

    #[test]
    fn active_frame_requires_complete_unique_tile_coverage() {
        let mut incomplete = active_frame();
        incomplete.tiles.pop();
        assert_eq!(
            incomplete.validate(),
            Err(VisualWorldFrameError::TileCountMismatch {
                expected: 2,
                actual: 1,
            })
        );

        let mut duplicate = active_frame();
        duplicate.tiles[1].column = duplicate.tiles[0].column;
        assert_eq!(
            duplicate.validate(),
            Err(VisualWorldFrameError::DuplicateTile { column: 0, row: 0 })
        );
    }

    #[test]
    fn active_frame_rejects_default_texture_handles() {
        let mut missing_map = active_frame();
        missing_map.map_texture = Handle::default();
        assert_eq!(
            missing_map.validate(),
            Err(VisualWorldFrameError::MissingMapTexture)
        );

        let mut missing_actor = active_frame();
        missing_actor.actors[0].texture = Handle::default();

        assert_eq!(
            missing_actor.validate(),
            Err(VisualWorldFrameError::MissingActorTexture(
                VisualActorId::Player
            ))
        );

        let mut missing_tile = active_frame();
        missing_tile.tiles[0].texture = Handle::default();
        assert_eq!(
            missing_tile.validate(),
            Err(VisualWorldFrameError::MissingTileTexture { column: 0, row: 0 })
        );
    }

    #[test]
    fn active_frame_rejects_incomplete_source_identity() {
        let mut empty_tileset = active_frame();
        empty_tileset.tiles[0].source.tileset_id = Arc::from("");
        assert_eq!(
            empty_tileset.validate(),
            Err(VisualWorldFrameError::EmptyTileSourceTileset { column: 0, row: 0 })
        );

        let mut outside_metatile = active_frame();
        outside_metatile.tiles[0].source.subtile_column = 4;
        assert_eq!(
            outside_metatile.validate(),
            Err(VisualWorldFrameError::TileSourceOutsideMetatile {
                column: 0,
                row: 0,
                subtile_column: 4,
                subtile_row: 0,
            })
        );
    }

    #[test]
    fn active_frame_rejects_duplicate_stable_actor_ids() {
        let mut frame = active_frame();
        frame.actors.push(VisualActor { model_palette: None,
            id: VisualActorId::Player,
            source_id: Arc::from("player"),
            texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(3), std::marker::PhantomData),
            center: Vec2::new(96.0, 72.0),
            size: Vec2::splat(16.0),
            flip_x: true,
            above_priority: true,
            facing: None,
        });

        assert_eq!(
            frame.validate(),
            Err(VisualWorldFrameError::DuplicateActor(VisualActorId::Player))
        );
    }

    #[test]
    fn active_frame_rejects_invalid_model_facing() {
        let mut frame = active_frame();
        for facing in [Vec2::ZERO, Vec2::new(f32::NAN, 0.0)] {
            frame.actors[0].facing = Some(facing);
            assert_eq!(
                frame.validate(),
                Err(VisualWorldFrameError::InvalidActorFacing(
                    VisualActorId::Player
                ))
            );
        }
        frame.actors[0].facing = Some(Vec2::NEG_Y);
        assert_eq!(frame.validate(), Ok(()));
    }

    #[test]
    fn active_frame_requires_actor_source_identity() {
        let mut frame = active_frame();
        frame.actors[0].source_id = Arc::from("");

        assert_eq!(
            frame.validate(),
            Err(VisualWorldFrameError::EmptyActorSource(
                VisualActorId::Player
            ))
        );
    }

    #[test]
    fn presentation_effect_identity_is_distinct_and_stable() {
        let mut frame = active_frame();
        let grass_rustle = VisualActor { model_palette: None,
            id: VisualActorId::Effect(VisualEffectId::GrassRustle),
            source_id: Arc::from("effect_grass_rustle"),
            texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(3), std::marker::PhantomData),
            center: Vec2::new(8.0, 4.0),
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: true,
            facing: None,
        };
        frame.actors.push(grass_rustle.clone());

        assert_eq!(frame.validate(), Ok(()));

        frame.actors.push(grass_rustle);
        assert_eq!(
            frame.validate(),
            Err(VisualWorldFrameError::DuplicateActor(
                VisualActorId::Effect(VisualEffectId::GrassRustle)
            ))
        );
    }

    #[test]
    fn source_map_extent_is_optional_positive_and_independent_of_rendered_halo() {
        let mut frame = active_frame();
        assert_eq!(frame.source_map_size_core_tiles, None);
        assert!(
            frame.validate().is_ok(),
            "legacy unknown bounds remain valid"
        );
        frame.source_map_size_core_tiles = Some(UVec2::ONE);
        frame.grid_origin = bevy::prelude::IVec2::new(-4, -4);
        assert!(
            frame.validate().is_ok(),
            "rendered halo may lie outside actual map bounds"
        );
        for size in [UVec2::ZERO, UVec2::new(1, 0), UVec2::new(0, 1)] {
            frame.source_map_size_core_tiles = Some(size);
            assert_eq!(
                frame.validate(),
                Err(VisualWorldFrameError::InvalidSourceMapSize)
            );
        }
    }

    #[test]
    fn coastal_terrace_field_rejects_invalid_corner_evidence() {
        let mut frame=active_frame();
        for (size,levels) in [(UVec2::ZERO,vec![]),(UVec2::new(2,1),vec![[0.;4]]),
            (UVec2::ONE,vec![[f32::NAN;4]]),(UVec2::ONE,vec![[-1.;4]])] {
            frame.terrace_datum=Some(Arc::new(VisualTerraceDatum {origin:bevy::prelude::IVec2::ZERO,size,levels:levels.into(),open_water:None,water_barriers:Arc::from([])}));
            assert_eq!(frame.validate(),Err(VisualWorldFrameError::InvalidTerraceDatum));
        }
        frame.terrace_datum=Some(Arc::new(VisualTerraceDatum {
            open_water:None,water_barriers:Arc::from([]),
            origin:bevy::prelude::IVec2::new(-40,20),size:UVec2::ONE,levels:vec![[0.,1.,2.,1.]].into(),
        }));
        assert!(frame.validate().is_ok());
        let mut valid=frame.terrace_datum.as_ref().unwrap().as_ref().clone();
        valid.size=UVec2::splat(2);valid.levels=vec![[0.;4];4].into();
        for cells in [vec![bevy::prelude::IVec2::ZERO],vec![valid.origin,valid.origin],
            vec![valid.origin+bevy::prelude::IVec2::X,valid.origin]] {
            let mut invalid=valid.clone();invalid.water_barriers=cells.into();
            frame.terrace_datum=Some(Arc::new(invalid));
            assert_eq!(frame.validate(),Err(VisualWorldFrameError::InvalidTerraceDatum));
        }
        let mut valid=valid;valid.water_barriers=vec![valid.origin].into();
        assert!(valid.water_barrier_at(valid.origin));
        for (level,cells) in [(f32::NAN,vec![valid.origin]),(-1.,vec![valid.origin]),
            (1.,vec![valid.origin,valid.origin]),(1.,vec![valid.origin-bevy::prelude::IVec2::ONE])] {
            let mut invalid=valid.clone();invalid.open_water=Some(VisualOpenWaterDatum {level,cells:cells.into()});
            frame.terrace_datum=Some(Arc::new(invalid));
            assert_eq!(frame.validate(),Err(VisualWorldFrameError::InvalidTerraceDatum));
        }
        frame.terrace_datum=Some(Arc::new(valid));assert!(frame.validate().is_ok());
    }

    #[test]
    fn battle_terrain_correspondence_rejects_missing_or_different_source_extent() {
        let mut frame = active_frame();
        frame.source_map_size_core_tiles = Some(UVec2::new(30, 18));
        let evidence = VisualBattleTerrainEvidence {
            map_id: frame.map_id.clone(),
            source_map_size_core_tiles: frame.source_map_size_core_tiles,
            terrain_revision: frame.terrain_revision,
            grid_origin: frame.grid_origin,
            grid_size: frame.grid_size,
            center: frame.center,
            viewport_size: frame.viewport_size,
            tile_size: frame.tile_size,
            map_texture: frame.map_texture.clone(),
            tiles: frame.tiles.clone().into(),
        };
        assert!(evidence.matches_built_frame(&frame));
        frame.source_map_size_core_tiles = Some(UVec2::new(30, 20));
        assert!(!evidence.matches_built_frame(&frame));
        frame.source_map_size_core_tiles = None;
        assert!(!evidence.matches_built_frame(&frame));
    }

    #[derive(Resource, Default)]
    struct PhaseLog(Vec<WorldRenderSet>);

    fn record_classic(mut log: ResMut<PhaseLog>) {
        log.0.push(WorldRenderSet::ClassicWorld);
    }

    fn record_extract(mut log: ResMut<PhaseLog>) {
        log.0.push(WorldRenderSet::PresentationExtract);
    }

    fn record_sync(mut log: ResMut<PhaseLog>) {
        log.0.push(WorldRenderSet::RenderSync);
    }

    #[test]
    fn plugin_orders_render_phases_and_initializes_frame() {
        let mut app = App::new();
        app.add_plugins(VisualWorldRenderPlugin)
            .init_resource::<PhaseLog>()
            .add_systems(Update, record_sync.in_set(WorldRenderSet::RenderSync))
            .add_systems(Update, record_classic.in_set(WorldRenderSet::ClassicWorld))
            .add_systems(
                Update,
                record_extract.in_set(WorldRenderSet::PresentationExtract),
            );

        app.update();

        assert!(!app.world().resource::<VisualWorldFrame>().active);
        assert_eq!(
            app.world().resource::<PhaseLog>().0,
            vec![
                WorldRenderSet::ClassicWorld,
                WorldRenderSet::PresentationExtract,
                WorldRenderSet::RenderSync,
            ]
        );
    }
    #[test]
    fn open_air_rooftop_identity_does_not_reclassify_roof_houses_or_other_atlases() {
        for (map,atlas) in [("CeladonMansionRoof","mansion"),("GoldenrodDeptStoreRoof","mart"),("TinTowerRoof","tower")] {
            assert!(is_open_air_rooftop(map,atlas));
            assert!(!is_open_air_rooftop(map,"house"));
        }
        assert!(!is_open_air_rooftop("CeladonMansionRoofHouse","mansion"));
        assert!(!is_open_air_rooftop("CeladonMansion1F","mansion"));
        assert!(!is_open_air_rooftop("GoldenrodDeptStore1F","mart"));
        assert!(!is_open_air_rooftop("TinTower1F","tower"));
    }

}
