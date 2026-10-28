//! Rasterize explicit editor strokes. This module never chooses geography.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

#[cfg(test)]
#[path = "superior_inland_source_tests.rs"]
mod superior_inland_source_tests;

#[derive(Deserialize)]
pub(crate) struct Layout {
    pub id: String,
    pub schema_version: u16,
    pub dimensions_metatiles: [u16; 2],
    pub compiled_tileset: String,
    shore_fitting: String,
    reject_cliff_cuts: bool,
    layers: Vec<Layer>,
    #[serde(default)]
    materials: BTreeMap<Material, String>,
    #[serde(default)]
    cliff_blocks: BTreeMap<String, String>,
    #[serde(default)]
    additional_water_blocks: Vec<String>,
}

#[derive(Deserialize)]
struct Layer {
    name: String,
    #[serde(flatten)]
    shape: Shape,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Shape {
    Polygon {
        points: Vec<[i32; 2]>,
        material: Material,
    },
    CliffPolygon {
        points: Vec<[i32; 2]>,
    },
    Path {
        points: Vec<[i32; 2]>,
        width_metatiles: u16,
        material: Material,
    },
    Rect {
        bounds: [i32; 4],
        material: Material,
    },
    BlockRect {
        bounds: [i32; 4],
        block: String,
    },
    BlockRemap {
        bounds: [i32; 4],
        from: String,
        to: String,
    },
    Point {
        position: [i32; 2],
        block: String,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material {
    Floor,
    Lawn,
    Encounter,
    Flowers,
    Forest,
    Oak,
    Water,
    Ice,
    Boulder,
    Sign,
}

impl Material {
    fn block(self) -> u16 {
        match self {
            Self::Floor => 1,
            Self::Lawn => 2,
            Self::Encounter => 3,
            Self::Flowers => 4,
            Self::Forest => 5,
            Self::Oak => 0x61,
            Self::Water => 0x35,
            Self::Ice => 0x3e,
            Self::Boulder => 0x0a,
            Self::Sign => 0x45,
        }
    }
}

impl Layout {
    pub fn parse(text: &str) -> Result<Self> {
        let layout: Self = serde_json::from_str(text)?;
        ensure!(layout.schema_version == 2, "unsupported editor schema");
        let expected = match layout.id.as_str() {
            "MN_R01" => ([32, 20], "johto", "johto_8_neighbors"),
            "MN_R02" => ([30, 22], "kanto", "kanto_8_neighbors"),
            "MN_R03" => ([24, 32], "mn_headwaters", "johto_8_neighbors"),
            "MN_R04" => ([28, 18], "mn_river_park", "explicit_source_edges"),
            "MN_R05" => ([32, 56], "mn_headwaters", "johto_8_neighbors"),
            "MN_R06" => ([32, 44], "mn_headwaters", "johto_8_neighbors"),
            "MN_R07" => ([32, 48], "mn_headwaters", "johto_8_neighbors"),
            "MN_R08" => ([40, 48], "mn_headwaters", "johto_8_neighbors"),
            "MN_BOUNDARY_WATERS_SAFARI" => (
                if layout.dimensions_metatiles == [64, 56] {
                    [64, 56]
                } else {
                    [48, 40]
                },
                "mn_boundary_safari",
                "johto_8_neighbors",
            ),
            "MN_R09" => ([64, 36], "mn_iron_range", "johto_8_neighbors"),
            "MN_R10" => ([48, 20], "johto", "johto_8_neighbors"),
            "MN_R16" => ([48, 24], "mn_headwaters", "johto_8_neighbors"),
            "MN_R11" => (
                if layout.dimensions_metatiles == [56, 96] {
                    [56, 96]
                } else {
                    [32, 72]
                },
                "mn_north_shore",
                "johto_8_neighbors",
            ),
            "MN_R12" => (
                if layout.dimensions_metatiles == [32, 32] {
                    [32, 32]
                } else {
                    [64, 56]
                },
                "mn_gunflint",
                "forest_8_neighbors",
            ),
            "MN_R13" => ([32, 64], "johto", "johto_8_neighbors"),
            "MN_R14" => ([48, 32], "mn_headwaters", "johto_8_neighbors"),
            "MN_R18" => ([40, 24], "mn_headwaters", "johto_8_neighbors"),
            "MN_R19" => ([32, 30], "johto", "johto_8_neighbors"),
            "MN_R20" => ([64, 32], "mn_headwaters", "johto_8_neighbors"),
            "MN_R17" => ([32, 56], "mn_border_pines", "forest_8_neighbors"),
            "MN_R15" => ([32, 18], "kanto", "kanto_8_neighbors"),
            "MnBoundaryWaters" => ([32, 24], "mn_boundary_safari", "johto_8_neighbors"),
            "MnGrandMarais" => ([20, 19], "mn_marais_shore", "explicit_source_edges"),
            "MnMaraisCedarApproach" => (
                if layout.dimensions_metatiles == [20, 12] {
                    [20, 12]
                } else {
                    [32, 19]
                },
                "mn_winter_north_shore",
                "explicit_source_edges",
            ),
            "MnMaraisInlandNeck" => ([65, 19], "mn_winter_north_shore", "explicit_source_edges"),
            _ => anyhow::bail!("unsupported authored route layout {}", layout.id),
        };
        ensure!(
            layout.dimensions_metatiles == expected.0
                && (layout.compiled_tileset == expected.1
                    || matches!(
                        (layout.id.as_str(), layout.compiled_tileset.as_str()),
                        ("MN_R02", "mn_prairie_kanto")
                            | ("MN_R16", "mn_prairie_headwaters")
                            | ("MN_R10", "mn_r10_rockstairs")
                            | ("MN_R12", "mn_winter_gunflint")
                            | ("MN_R13", "mn_winter_north_shore")
                    ))
                && (layout.shore_fitting == expected.2
                    || (matches!(
                        layout.id.as_str(),
                        "MN_R03" | "MN_R11" | "MN_R12" | "MN_R13"
                    ) && layout.shore_fitting == "explicit_source_edges")),
            "retain the complete authored route and native art fitting"
        );
        Ok(layout)
    }

    fn material(&self, material: Material) -> Result<u16> {
        ensure!(
            material != Material::Ice || self.materials.contains_key(&Material::Ice),
            "ice requires an explicit native tileset block"
        );
        self.materials
            .get(&material)
            .map_or(Ok(material.block()), |value| parse_block(value))
    }

    pub fn compile(&self) -> Result<Vec<u16>> {
        let [width, height] = self.dimensions_metatiles.map(i32::from);
        let index = |x: i32, y: i32| (y * width + x) as usize;
        let inside = |x: i32, y: i32| x >= 0 && y >= 0 && x < width && y < height;
        let mut blocks = vec![self.material(Material::Forest)?; (width * height) as usize];
        let cliffs = (0x68..=0x73)
            .map(|id| {
                let mapped = self.cliff_blocks.get(&format!("{id:02x}"));
                Ok((id, mapped.map_or(Ok(id), |value| parse_block(value))?))
            })
            .collect::<Result<BTreeMap<u16, u16>>>()?;
        for layer in &self.layers {
            let mut painted = Vec::new();
            let block = match &layer.shape {
                Shape::Polygon { points, material } => {
                    painted = polygon(points, width, height).with_context(|| layer.name.clone())?;
                    self.material(*material)?
                }
                Shape::CliffPolygon { points } => {
                    let mask =
                        polygon(points, width, height).with_context(|| layer.name.clone())?;
                    let mut occupied = vec![false; blocks.len()];
                    for &[x, y] in &mask {
                        occupied[index(x, y)] = true;
                    }
                    let rock = |x, y| inside(x, y) && occupied[index(x, y)];
                    for [x, y] in mask {
                        let (n, s, w, e) = (
                            rock(x, y - 1),
                            rock(x, y + 1),
                            rock(x - 1, y),
                            rock(x + 1, y),
                        );
                        let edge = if !n && !w {
                            0x6a
                        } else if !n && !e {
                            0x6b
                        } else if !s && !w {
                            0x6c
                        } else if !s && !e {
                            0x6d
                        } else if !n {
                            0x70
                        } else if !s {
                            0x72
                        } else if !w {
                            0x68
                        } else if !e {
                            0x69
                        } else if !rock(x - 1, y + 1) {
                            0x6e
                        } else if !rock(x + 1, y + 1) {
                            0x6f
                        } else {
                            0x71
                        };
                        blocks[index(x, y)] = cliffs[&edge];
                    }
                    continue;
                }
                Shape::Path {
                    points,
                    width_metatiles,
                    material,
                } => {
                    ensure!(
                        [1, 2, 3].contains(width_metatiles),
                        "{}: unsupported brush width",
                        layer.name
                    );
                    ensure!(points.len() >= 2, "{}: empty path", layer.name);
                    ensure!(
                        points.iter().all(|&[x, y]| inside(x, y)),
                        "{}: authored path point outside route",
                        layer.name
                    );
                    for segment in points.windows(2) {
                        let [a, b] = [segment[0], segment[1]];
                        ensure!(
                            a[0] == b[0] || a[1] == b[1],
                            "{}: path must be orthogonal",
                            layer.name
                        );
                        let dx = (b[0] - a[0]).signum();
                        let dy = (b[1] - a[1]).signum();
                        let length = (b[0] - a[0]).abs() + (b[1] - a[1]).abs();
                        for step in 0..=length {
                            let [x, y] = [a[0] + dx * step, a[1] + dy * step];
                            painted.push([x, y]);
                            // Match the existing editor's directed two-cell brush.
                            if *width_metatiles == 2 {
                                painted.push([x + dy, y + dx]);
                            } else if *width_metatiles == 3 {
                                painted.push([x + dy, y + dx]);
                                painted.push([x - dy, y - dx]);
                            }
                        }
                    }
                    if self.reject_cliff_cuts {
                        ensure!(
                            !painted.iter().any(|&[x, y]| inside(x, y)
                                && cliffs.values().any(|&block| block == blocks[index(x, y)])),
                            "{}: path cuts a cliff face",
                            layer.name
                        );
                    }
                    self.material(*material)?
                }
                Shape::Rect { bounds, material } => {
                    painted = rectangle(*bounds, width, height)?;
                    self.material(*material)?
                }
                Shape::BlockRect { bounds, block } => {
                    painted = rectangle(*bounds, width, height)?;
                    parse_block(block)?
                }
                Shape::BlockRemap { bounds, from, to } => {
                    let from = parse_block(from)?;
                    let to = parse_block(to)?;
                    for [x, y] in rectangle(*bounds, width, height)? {
                        if blocks[index(x, y)] == from {
                            blocks[index(x, y)] = to;
                        }
                    }
                    continue;
                }
                Shape::Point { position, block } => {
                    painted.push(*position);
                    parse_block(block)?
                }
            };
            for [x, y] in painted {
                // A wide edge brush can extend one cell beyond the canvas.
                if inside(x, y) {
                    blocks[index(x, y)] = block;
                } else {
                    ensure!(
                        matches!(layer.shape, Shape::Path { .. }),
                        "{}: point outside route",
                        layer.name
                    );
                }
            }
        }
        if self.shore_fitting == "explicit_source_edges" {
            return Ok(blocks);
        }
        let original = blocks.clone();
        let forest = self.shore_fitting == "forest_8_neighbors";
        let kanto = self.shore_fitting == "kanto_8_neighbors";
        let water = self.material(Material::Water)?;
        let mut excluded = if forest {
            vec![water, 0x35]
        } else if kanto {
            vec![water, 0x78]
        } else {
            vec![water, 6, 0x80]
        };
        for block in &self.additional_water_blocks {
            excluded.push(parse_block(block)?);
        }
        let edges = if forest {
            [0x07, 0x0d, 0x0a, 0x0b, 0x03, 0x38, 0x39]
        } else if kanto {
            [0x1d, 0x1f, 0x65, 0x64, 0x1e, 0x2e, 0x2d]
        } else {
            [0x54, 0x55, 0x58, 0x59, 0x76, 0x79, 0x7a]
        };
        let land = |x, y| inside(x, y) && !excluded.contains(&original[index(x, y)]);
        for y in 0..height {
            for x in 0..width {
                if original[index(x, y)] != water {
                    continue;
                }
                let (n, w, e) = (land(x, y - 1), land(x - 1, y), land(x + 1, y));
                blocks[index(x, y)] = if n && w {
                    edges[0]
                } else if n && e {
                    edges[1]
                } else if w {
                    edges[2]
                } else if e {
                    edges[3]
                } else if n {
                    edges[4]
                } else if land(x - 1, y - 1) {
                    edges[5]
                } else if land(x + 1, y - 1) {
                    edges[6]
                } else {
                    water
                };
            }
        }
        Ok(blocks)
    }
}

#[cfg(test)]
#[path = "authored_safari_layout_tests.rs"]
mod safari_tests;

fn parse_block(text: &str) -> Result<u16> {
    let block = u16::from_str_radix(text, 16).context("source block identifier")?;
    ensure!(
        block <= u16::from(u8::MAX),
        "source block exceeds native byte addressing"
    );
    Ok(block)
}

fn rectangle([x0, y0, x1, y1]: [i32; 4], width: i32, height: i32) -> Result<Vec<[i32; 2]>> {
    ensure!(
        0 <= x0 && x0 < x1 && x1 <= width && 0 <= y0 && y0 < y1 && y1 <= height,
        "invalid editor rectangle"
    );
    Ok((y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| [x, y]))
        .collect())
}

fn polygon(points: &[[i32; 2]], width: i32, height: i32) -> Result<Vec<[i32; 2]>> {
    ensure!(points.len() >= 3, "polygon needs three points");
    for (a, b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        ensure!(a[0] == b[0] || a[1] == b[1], "polygon must be orthogonal");
        ensure!(
            a[0] >= 0 && a[1] >= 0 && a[0] < width && a[1] < height,
            "polygon outside canvas"
        );
    }
    let mut filled = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let mut inside = false;
            let mut boundary = false;
            for (a, b) in points
                .iter()
                .zip(points.iter().cycle().skip(1))
                .take(points.len())
            {
                if x >= a[0].min(b[0])
                    && x <= a[0].max(b[0])
                    && y >= a[1].min(b[1])
                    && y <= a[1].max(b[1])
                {
                    boundary = true;
                    break;
                }
                if a[0] == b[0] && (a[1] > y) != (b[1] > y) && x < a[0] {
                    inside = !inside;
                }
            }
            if inside || boundary {
                filled.push([x, y]);
            }
        }
    }
    Ok(filled)
}
