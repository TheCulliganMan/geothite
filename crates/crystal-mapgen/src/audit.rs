use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{GeneratedGrid, H3Facility};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapAudit {
    pub passed: bool,
    pub district_counts: BTreeMap<String, usize>,
    pub structures: usize,
    pub encounter_cells: usize,
    pub walkable_reach_percent: f64,
    pub errors: Vec<String>,
    pub notes: Vec<String>,
}

pub fn audit_grid(grid: &GeneratedGrid) -> MapAudit {
    let expected_pokecenter = grid
        .source
        .h3
        .as_ref()
        .is_none_or(|plan| plan.requests_facility(H3Facility::PokemonCenter));
    let expected_mart = grid
        .source
        .h3
        .as_ref()
        .is_none_or(|plan| plan.requests_facility(H3Facility::Mart));
    audit_grid_with_facilities(grid, expected_pokecenter, expected_mart)
}

/// Audits a grid against the facility allocation chosen by a regional batch.
///
/// [`audit_grid`] derives these expectations from the grid's H3 regional plan;
/// this explicit form is useful when validating a planner before attaching its
/// directives. Standalone generation still expects one Center and one Mart.
pub fn audit_grid_with_facilities(
    grid: &GeneratedGrid,
    expected_pokecenter: bool,
    expected_mart: bool,
) -> MapAudit {
    let expected = usize::from(grid.width) * usize::from(grid.height);
    let invalid = grid.width == 0
        || grid.height == 0
        || grid.cells.len() != expected
        || grid.scene.as_ref().is_none_or(|scene| {
            scene.generator_version != crate::GENERATOR_VERSION
                || scene.districts.len() != expected
                || scene.reserved.len() != expected
                || scene.islands.len() != expected
                || scene.courtyards.len() != expected
                || scene.spawn.0 >= grid.width
                || scene.spawn.1 >= grid.height
                || scene.structures.iter().any(|s| {
                    let (w, h) = s.recipe.dimensions();
                    u32::from(s.origin.0) + u32::from(w) > u32::from(grid.width)
                        || u32::from(s.origin.1) + u32::from(h) > u32::from(grid.height)
                        || s.door.0 / 2 >= grid.width
                        || s.door.1 / 2 >= grid.height
                })
                || scene
                    .destinations
                    .iter()
                    .any(|d| d.entrance.0 >= grid.width || d.entrance.1 >= grid.height)
        });
    if invalid {
        return MapAudit {
            passed: false,
            district_counts: BTreeMap::new(),
            structures: 0,
            encounter_cells: 0,
            walkable_reach_percent: 0.0,
            errors: vec![
                "grid dimensions, scene layers, or generator version are invalid; regenerate it"
                    .into(),
            ],
            notes: Vec::new(),
        };
    }
    crate::scene::audit(grid, expected_pokecenter, expected_mart)
}
