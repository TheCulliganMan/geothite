//! Owning-module bridge for sampled tile art and deferred grove metadata.
use super::*;
use crate::terrain_worker::request::TextureBindings;
use crate::terrain_worker::{blobs::BlobStore, records::Records};
use bevy::platform::time::Instant;
use bevy::prelude::IVec2;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SamplesWire(Vec<(u32, Option<Vec<u8>>)>);

impl SamplesWire {
    pub(crate) fn capture(
        samples: &TerrainImageSamples,
        bindings: &mut TextureBindings,
        frame: &VisualWorldFrame,
    ) -> Self {
        let mut result = std::collections::BTreeMap::new();
        for tile in &frame.tiles {
            let slot = bindings.slot(&tile.texture);
            result
                .entry(slot)
                .or_insert_with(|| match samples.pixels.get(&tile.texture.id()) {
                    Some(TileImageSample::Rgba(pixels)) => Some(pixels.clone()),
                    Some(TileImageSample::Invalid) | None => None,
                });
        }
        Self(result.into_iter().collect())
    }

    pub(crate) fn restore(self, bindings: &TextureBindings) -> Result<TerrainImageSamples, String> {
        if self.0.len() > 262_144 {
            return Err("too many sampled terrain images".into());
        }
        let mut samples = TerrainImageSamples::default();
        for (slot, pixels) in self.0 {
            let image = bindings.handle(slot)?.id();
            let sample = match pixels {
                Some(pixels) if pixels.len() == SOURCE_TILE_PIXELS * SOURCE_TILE_PIXELS * 4 => {
                    TileImageSample::Rgba(pixels)
                }
                None => TileImageSample::Invalid,
                _ => return Err("invalid sampled terrain image size".into()),
            };
            if samples.pixels.insert(image, sample).is_some() {
                return Err("duplicate sampled terrain image".into());
            }
        }
        Ok(samples)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct StreamWire {
    bank_regions: Option<Vec<[u32; 4]>>,
    recipes: Records<RecipeWire>,
    geometry: Option<([u64; 2], [u32; 4])>,
    origin: [i32; 2],
    ledges: Vec<[[i32; 2]; 3]>,
    cells: crate::terrain_worker::request::Frame,
    shapes: Records<ShapeWire>,
    claims: Records<Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct RecipeWire {
    family: u8,
    bounds: [u32; 4],
    base: u32,
    tile_height: u32,
    coordinates: [i32; 2],
    neighbors: [bool; 4],
    understory: Option<([u32; 4], [u8; 2], u64)>,
    coverage: bool,
}

impl StreamWire {
    pub(crate) fn capture(
        stream: &StreamedGroves,
        frame: &VisualWorldFrame,
        bindings: &mut TextureBindings,
    ) -> Self {
        let source = &stream.source;
        let mut cells = frame.clone();
        cells.active = false;
        cells.tiles = source.cells.clone();
        cells.actors.clear();
        Self {
            bank_regions: source
                .bank_regions
                .get()
                .map(|regions| regions.iter().map(|r| r.map(f32::to_bits)).collect()),
            recipes: source
                .recipes
                .iter()
                .map(|r| RecipeWire {
                    family: match r.family {
                        crate::foliage::GroveFamily::JohtoPine => 0,
                        crate::foliage::GroveFamily::AncientForest => 1,
                        crate::foliage::GroveFamily::KantoDeciduous => 2,
                    },
                    bounds: r.bounds.map(f32::to_bits),
                    base: r.base_height.to_bits(),
                    tile_height: r.tile_height.to_bits(),
                    coordinates: r.coordinates,
                    neighbors: r.neighbors,
                    understory: r.understory_bounds.map(|p| (p.bounds.map(f32::to_bits),p.cells,p.occupied)),
                    coverage: r.coverage_fade,
                })
                .collect(),
            geometry: source.geometry.map(|g| {
                (
                    [g.width as u64, g.height as u64],
                    [
                        g.tile_width.to_bits(),
                        g.tile_height.to_bits(),
                        g.origin_x.to_bits(),
                        g.origin_z.to_bits(),
                    ],
                )
            }),
            origin: source.origin,
            ledges: source
                .jump_ledges
                .iter()
                .map(|l| [l.start.to_array(), l.end.to_array(), l.descent.to_array()])
                .collect(),
            cells: crate::terrain_worker::request::Frame::capture_with_bindings(&cells, bindings),
            shapes: source
                .shapes
                .iter()
                .copied()
                .map(ShapeWire::capture)
                .collect(),
            claims: match &source.claims {
                DeferredClaims::Static(claims) => claims
                    .iter()
                    .map(|claim| claim.map(str::to_owned))
                    .collect(),
                DeferredClaims::Owned(claims) => claims.clone().into(),
            },
        }
    }

    pub(crate) fn resolve(
        &mut self,
        blobs: &mut BlobStore,
        deadline: Instant,
        max_elements: usize,
    ) -> Result<bool, String> {
        let mut ready = self.cells.resolve(blobs, deadline, max_elements)?;
        ready &= self.recipes.resolve(blobs, deadline, max_elements)?;
        ready &= self.shapes.resolve(blobs, deadline, max_elements)?;
        ready &= self.claims.resolve(blobs, deadline, max_elements)?;
        Ok(ready)
    }

    pub(crate) fn restore(self, bindings: &TextureBindings) -> Result<Arc<StreamedGroves>, String> {
        let frame = self.cells.restore(bindings)?;
        if self.recipes.len() > 65_536
            || self.ledges.len() > 65_536
            || self.shapes.len() > 262_144
            || self.claims.len() > 262_144
        {
            return Err("deferred terrain source exceeds element limits".into());
        }
        let geometry = self
            .geometry
            .map(|(size, values)| -> Result<GridGeometry, String> {
                let [tile_width, tile_height, origin_x, origin_z] = values.map(f32::from_bits);
                if size[0] == 0
                    || size[1] == 0
                    || size[0].saturating_mul(size[1]) > 262_144
                    || !tile_width.is_finite()
                    || tile_width <= 0.
                    || !tile_height.is_finite()
                    || tile_height <= 0.
                    || !origin_x.is_finite()
                    || !origin_z.is_finite()
                {
                    return Err("invalid deferred terrain geometry".into());
                }
                Ok(GridGeometry {
                    width: size[0] as usize,
                    height: size[1] as usize,
                    tile_width,
                    tile_height,
                    origin_x,
                    origin_z,
                })
            })
            .transpose()?;
        if self.recipes.len() != 0 && geometry.is_none() {
            return Err("missing deferred terrain geometry".into());
        }
        let bank_regions = std::sync::OnceLock::new();
        if let Some(regions) = self.bank_regions {
            bank_regions
                .set(regions.into_iter().map(|r| r.map(f32::from_bits)).collect())
                .map_err(|_| "duplicate bank regions")?;
        }
        let recipes = self
            .recipes
            .into_vec()?
            .into_iter()
            .map(|r| {
                Ok(new_bark::GroveRecipe {
                    family: match r.family {
                        0 => crate::foliage::GroveFamily::JohtoPine,
                        1 => crate::foliage::GroveFamily::AncientForest,
                        2 => crate::foliage::GroveFamily::KantoDeciduous,
                        _ => return Err("invalid grove family".into()),
                    },
                    bounds: r.bounds.map(f32::from_bits),
                    base_height: f32::from_bits(r.base),
                    tile_height: f32::from_bits(r.tile_height),
                    coordinates: r.coordinates,
                    neighbors: r.neighbors,
                    understory_bounds: r.understory.map(|(bounds,cells,occupied)| crate::understory::Plot {bounds:bounds.map(f32::from_bits),cells,occupied}),
                    coverage_fade: r.coverage,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(Arc::new(StreamedGroves {
            source: DeferredGroves {
                terrace_datum: frame.terrace_datum.clone(),
                bank_regions,
                recipes,
                geometry,
                origin: self.origin,
                jump_ledges: self
                    .ledges
                    .into_iter()
                    .map(|l| crystal_render_api::VisualJumpLedge {
                        start: IVec2::from_array(l[0]),
                        end: IVec2::from_array(l[1]),
                        descent: IVec2::from_array(l[2]),
                    })
                    .collect(),
                cells: frame.tiles,
                shapes: self
                    .shapes
                    .into_vec()?
                    .into_iter()
                    .map(ShapeWire::restore)
                    .collect::<Result<_, _>>()?,
                claims: DeferredClaims::Owned(self.claims.into_vec()?),
            },
        }))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ShapeWire {
    Flat,
    Water,
    PlaneAt {
        height: f32,
    },
    Waterfall,
    Cutout {
        ground_tile_index: u16,
        solid: u8,
    },
    Relief {
        height: f32,
        ground_tile_index: u16,
        base_height: f32,
    },
    FloatingRelief {
        height: f32,
        ground_tile_index: u16,
        base_height: f32,
    },
    ShoreBand,
    RaisedTop {
        height: f32,
        solid: u8,
    },
    RampNorth {
        north_height: f32,
        south_height: f32,
    },
    RampEast {
        west_height: f32,
        east_height: f32,
    },
    FacadeBand {
        plane_subtile_row: u8,
        band_from_top: u8,
        band_count: u8,
        ground_tile_index: u16,
        solid: u8,
    },
    LedgeBand {
        face: u8,
        plane_subtile: u8,
        band_from_top: u8,
        band_count: u8,
        top_tile_index: u16,
        height: f32,
    },
}
fn solid_wire(v: SolidKind) -> u8 {
    match v {
        SolidKind::Building => 0,
        SolidKind::Tree => 1,
        SolidKind::Rock => 2,
        SolidKind::Ship => 3,
        SolidKind::FlatCard => 4,
        SolidKind::Prop => 5,
        SolidKind::CutTree => 6,
        SolidKind::Bank => 7,
        SolidKind::Flower => 8,
        SolidKind::Grass => 9,
        SolidKind::Fence => 10,
    }
}
fn restore_solid(v: u8) -> Result<SolidKind, String> {
    match v {
        0 => Ok(SolidKind::Building),
        1 => Ok(SolidKind::Tree),
        2 => Ok(SolidKind::Rock),
        3 => Ok(SolidKind::Ship),
        4 => Ok(SolidKind::FlatCard),
        5 => Ok(SolidKind::Prop),
        6 => Ok(SolidKind::CutTree),
        7 => Ok(SolidKind::Bank),
        8 => Ok(SolidKind::Flower),
        9 => Ok(SolidKind::Grass),
        10 => Ok(SolidKind::Fence),
        _ => Err("invalid solid kind".into()),
    }
}
impl ShapeWire {
    fn capture(v: CellShape) -> Self {
        match v {
            CellShape::Flat => Self::Flat,
            CellShape::Water => Self::Water,
            CellShape::PlaneAt { height } => Self::PlaneAt { height: height },
            CellShape::Waterfall => Self::Waterfall,
            CellShape::Cutout {
                ground_tile_index,
                solid,
            } => Self::Cutout {
                ground_tile_index: ground_tile_index,
                solid: solid_wire(solid),
            },
            CellShape::Relief {
                height,
                ground_tile_index,
                base_height,
            } => Self::Relief {
                height: height,
                ground_tile_index: ground_tile_index,
                base_height: base_height,
            },
            CellShape::FloatingRelief {
                height,
                ground_tile_index,
                base_height,
            } => Self::FloatingRelief {
                height: height,
                ground_tile_index: ground_tile_index,
                base_height: base_height,
            },
            CellShape::ShoreBand => Self::ShoreBand,
            CellShape::RaisedTop { height, solid } => Self::RaisedTop {
                height: height,
                solid: solid_wire(solid),
            },
            CellShape::RampNorth {
                north_height,
                south_height,
            } => Self::RampNorth {
                north_height: north_height,
                south_height: south_height,
            },
            CellShape::RampEast {
                west_height,
                east_height,
            } => Self::RampEast {
                west_height: west_height,
                east_height: east_height,
            },
            CellShape::FacadeBand {
                plane_subtile_row,
                band_from_top,
                band_count,
                ground_tile_index,
                solid,
            } => Self::FacadeBand {
                plane_subtile_row: plane_subtile_row,
                band_from_top: band_from_top,
                band_count: band_count,
                ground_tile_index: ground_tile_index,
                solid: solid_wire(solid),
            },
            CellShape::LedgeBand {
                face,
                plane_subtile,
                band_from_top,
                band_count,
                top_tile_index,
                height,
            } => Self::LedgeBand {
                face: match face {
                    LedgeFace::South => 0,
                    LedgeFace::West => 1,
                    LedgeFace::East => 2,
                },
                plane_subtile: plane_subtile,
                band_from_top: band_from_top,
                band_count: band_count,
                top_tile_index: top_tile_index,
                height: height,
            },
        }
    }
    fn restore(self) -> Result<CellShape, String> {
        Ok(match self {
            Self::Flat => CellShape::Flat,
            Self::Water => CellShape::Water,
            Self::PlaneAt { height } => CellShape::PlaneAt { height: height },
            Self::Waterfall => CellShape::Waterfall,
            Self::Cutout {
                ground_tile_index,
                solid,
            } => CellShape::Cutout {
                ground_tile_index: ground_tile_index,
                solid: restore_solid(solid)?,
            },
            Self::Relief {
                height,
                ground_tile_index,
                base_height,
            } => CellShape::Relief {
                height: height,
                ground_tile_index: ground_tile_index,
                base_height: base_height,
            },
            Self::FloatingRelief {
                height,
                ground_tile_index,
                base_height,
            } => CellShape::FloatingRelief {
                height: height,
                ground_tile_index: ground_tile_index,
                base_height: base_height,
            },
            Self::ShoreBand => CellShape::ShoreBand,
            Self::RaisedTop { height, solid } => CellShape::RaisedTop {
                height: height,
                solid: restore_solid(solid)?,
            },
            Self::RampNorth {
                north_height,
                south_height,
            } => CellShape::RampNorth {
                north_height: north_height,
                south_height: south_height,
            },
            Self::RampEast {
                west_height,
                east_height,
            } => CellShape::RampEast {
                west_height: west_height,
                east_height: east_height,
            },
            Self::FacadeBand {
                plane_subtile_row,
                band_from_top,
                band_count,
                ground_tile_index,
                solid,
            } => CellShape::FacadeBand {
                plane_subtile_row: plane_subtile_row,
                band_from_top: band_from_top,
                band_count: band_count,
                ground_tile_index: ground_tile_index,
                solid: restore_solid(solid)?,
            },
            Self::LedgeBand {
                face,
                plane_subtile,
                band_from_top,
                band_count,
                top_tile_index,
                height,
            } => CellShape::LedgeBand {
                face: match face {
                    0 => LedgeFace::South,
                    1 => LedgeFace::West,
                    2 => LedgeFace::East,
                    _ => return Err("invalid ledge face".into()),
                },
                plane_subtile: plane_subtile,
                band_from_top: band_from_top,
                band_count: band_count,
                top_tile_index: top_tile_index,
                height: height,
            },
        })
    }
}
