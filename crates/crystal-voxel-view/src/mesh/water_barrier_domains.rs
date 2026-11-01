//! Resolve ambiguous barrier backing from complete native source components.
//! Consumers receive immutable cells so camera crops cannot flip water to lawn.
use bevy::prelude::IVec2;
use crystal_render_api::VisualTileSource;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

pub(super) fn resolve(native: &BTreeMap<(i32, i32), (&VisualTileSource, bool)>) -> Arc<[IVec2]> {
    if !native.values().any(|&(s, _)| {
        s.tileset_id.as_ref() == "kanto"
            && s.tile_index == 0x14
            && super::super::shape_for_source(s) == crate::profile::CellShape::Water
    }) {
        return Arc::from([]);
    }
    let mut remaining = BTreeSet::new();
    for (&(y, x), &(s, _)) in native {
        if crate::kanto_cliff::ambiguous_water_barrier_local(s) != Some((0, 0)) {
            continue;
        }
        let complete = (0..2).all(|dy| {
            (0..2).all(|dx| {
                native.get(&(y + dy, x + dx)).is_some_and(|&(other, _)| {
                    other.tileset_id == s.tileset_id
                        && other.metatile_id == s.metatile_id
                        && other.subtile_column == s.subtile_column + dx as u8
                        && other.subtile_row == s.subtile_row + dy as u8
                        && crate::kanto_cliff::ambiguous_water_barrier_local(other)
                            == Some((dx as u8, dy as u8))
                })
            })
        });
        if complete {
            for dy in 0..2 {
                for dx in 0..2 {
                    remaining.insert((y + dy, x + dx));
                }
            }
        }
    }
    let eligible = remaining.clone();
    let mut wet_cells = Vec::new();
    while let Some(start) = remaining.pop_first() {
        let mut pending = VecDeque::from([start]);
        let mut component = vec![start];
        let (mut wet, mut dry) = (0usize, 0usize);
        while let Some((y, x)) = pending.pop_front() {
            for p in [(y, x - 1), (y, x + 1), (y - 1, x), (y + 1, x)] {
                if eligible.contains(&p) {
                    if remaining.remove(&p) {
                        pending.push_back(p);
                        component.push(p);
                    }
                    continue;
                }
                let Some(&(s, _)) = native.get(&p) else {
                    continue;
                };
                if s.tileset_id.as_ref() != "kanto" {
                    continue;
                }
                if (s.tile_index == 0x14
                    && super::super::shape_for_source(s) == crate::profile::CellShape::Water)
                    || crate::kanto_cliff::shoreline_round_barrier_local(s).is_some()
                {
                    wet += 1;
                } else if super::super::shape_for_source(s) == crate::profile::CellShape::Flat
                    && matches!(
                        super::super::new_bark::ground_material(s),
                        Some(
                            super::super::new_bark::GroundMaterial::Lawn
                                | super::super::new_bark::GroundMaterial::Path
                                | super::super::new_bark::GroundMaterial::Pavers
                        )
                    )
                {
                    dry += 1;
                }
            }
        }
        // Unknown/cropped evidence never authorizes a water underlay. A dry
        // boundary beside one wet edge retains its established land treatment.
        if wet > dry {
            wet_cells.extend(component.into_iter().map(|(y, x)| IVec2::new(x, y)));
        }
    }
    wet_cells.sort_unstable_by_key(|p| (p.y, p.x));
    wet_cells.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sources(water: bool) -> Vec<(IVec2, VisualTileSource, bool)> {
        (0..36)
            .map(|i| {
                let (x, y) = (i % 6, i / 6);
                let rock = (2..4).contains(&x) && (2..4).contains(&y);
                (
                    IVec2::new(x, y),
                    VisualTileSource {
                        tileset_id: "kanto".into(),
                        metatile_id: if rock {
                            0x13
                        } else if water {
                            0x35
                        } else {
                            1
                        },
                        subtile_column: if rock { (x - 2) as u8 } else { (x % 4) as u8 },
                        subtile_row: if rock { (y - 2) as u8 } else { (y % 4) as u8 },
                        tile_index: if rock {
                            [[0x2a, 0x2b], [0x3a, 0x3b]][(y - 2) as usize][(x - 2) as usize]
                        } else if water {
                            0x14
                        } else {
                            0x2c
                        },
                    },
                    false,
                )
            })
            .collect()
    }
    #[test]
    fn complete_source_component_keeps_dry_stones_and_rejects_partial_art() {
        let get = |s: &[(IVec2, VisualTileSource, bool)]| {
            resolve(&s.iter().map(|(p, s, b)| ((p.y, p.x), (s, *b))).collect())
        };
        let mut wet = sources(true);
        assert_eq!(get(&wet).len(), 4);
        assert!(get(&sources(false)).is_empty());
        wet[2 * 6 + 2].1.tile_index = 0xffff;
        assert!(get(&wet).is_empty());
        let mut dry = sources(false);
        dry[1 * 6 + 2].1.tile_index = 0x14;
        dry[1 * 6 + 2].1.metatile_id = 0x35;
        assert!(
            get(&dry).is_empty(),
            "one wet edge must not turn a dry fence into ocean"
        );
    }
}
