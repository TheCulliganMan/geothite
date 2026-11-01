//! Frozen continuous R17 cedar-marsh and R11 frosted-gorge source contracts.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::authored_route_layout::Layout;

pub(crate) const CONTRACT_HASH: &str =
    "c901add54e8ec698b72f563bcd2fa93ef0383e95bd399db8f6bb70f048032735";

#[derive(Deserialize)]
pub(crate) struct Plan {
    pub schema_version: u16,
    pub id: String,
    pub art: Vec<Art>,
    pub routes: Vec<Route>,
}

#[derive(Deserialize)]
pub(crate) struct Art {
    pub alias: String,
    pub definition_sha256: String,
    pub files: BTreeMap<String, String>,
}

#[derive(Deserialize)]
pub(crate) struct Route {
    pub id: String,
    pub dimensions_metatiles: [u16; 2],
    pub old_tileset: String,
    pub new_tileset: String,
    pub sources: BTreeMap<String, String>,
    pub previous_layout: String,
    pub candidate_layout: String,
    pub previous_dossier: String,
    pub candidate_dossier: String,
    pub native_modules: Vec<Module>,
    pub native_cap_overlays: Vec<Cap>,
    pub compiled_before_sha256: String,
    pub compiled_after_sha256: String,
    pub snowline_exclusive_y_by_meta_x: Vec<u16>,
    pub changes: Vec<Cell>,
    pub water_to_winter: Vec<[u16; 2]>,
    pub slide_courses: Vec<Course>,
    pub endpoints_core: [[u16; 2]; 2],
    pub cut: Option<Cut>,
    pub protected_bridge_metatiles: Vec<[u16; 2]>,
    pub required_grass_bounds_core: Option<[u16; 4]>,
}

#[derive(Deserialize)]
pub(crate) struct Module {
    pub map: String,
    pub column: u16,
    pub row: u16,
    pub module_sha256: String,
    pub retarget: bool,
}

#[derive(Deserialize)]
pub(crate) struct Cap {
    pub at: [u16; 2],
    pub source: u16,
    pub native: u16,
}

#[derive(Deserialize)]
pub(crate) struct Cell {
    pub at: [u16; 2],
    pub before: u16,
    pub after: u16,
}

#[derive(Deserialize)]
pub(crate) struct Course {
    pub from_core: [u16; 2],
    pub direction: [i16; 2],
    pub ice_path_core: Vec<[u16; 2]>,
    pub stop_core: [u16; 2],
    pub stop_kind: String,
    pub blocked_target_core: Option<[u16; 2]>,
    pub ordinary_turn_exits: Vec<Turn>,
}

#[derive(Deserialize)]
pub(crate) struct Turn {
    pub direction: [i16; 2],
    pub ice_path_core: Vec<[u16; 2]>,
    pub dry_stop_core: [u16; 2],
}

#[derive(Deserialize)]
pub(crate) struct Cut {
    pub tile: [u16; 2],
    pub source_block: u16,
    pub replacement_block: u16,
    pub compare_from: [u16; 2],
    pub compare_to: [u16; 2],
    pub minimum_saved_steps: usize,
}

pub(crate) fn hash(raw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(raw))
}

pub(crate) fn parse(text: &str) -> Result<Plan> {
    ensure!(
        hash(text.as_bytes()) == CONTRACT_HASH,
        "exact frozen winter-section contract required"
    );
    let plan: Plan = serde_json::from_str(text)?;
    ensure!(
        plan.schema_version == 1
            && plan.id == "MN_WINTER_SECTIONS"
            && plan.art.len() == 2
            && plan.routes.len() == 2,
        "bounded winter section contract extent changed"
    );
    for (route, (id, dimensions, source, target, changes, water, courses)) in
        plan.routes.iter().zip([
            (
                "MN_R17",
                [32, 56],
                "mn_border_pines",
                "mn_winter_outdoor",
                1232,
                40,
                56,
            ),
            (
                "MN_R11",
                [56, 96],
                "mn_north_shore",
                "mn_winter_north_shore",
                826,
                0,
                26,
            ),
        ])
    {
        ensure!(
            route.id == id
                && route.dimensions_metatiles == dimensions
                && route.old_tileset == source
                && route.new_tileset == target
                && route.changes.len() == changes
                && route.water_to_winter.len() == water
                && route.slide_courses.len() == courses,
            "unreviewed winter geography/source extent"
        );
    }
    Ok(plan)
}

pub(crate) fn source<'a>(
    sources: &'a BTreeMap<String, String>,
    route: &Route,
    name: &str,
) -> Result<&'a str> {
    let text = sources
        .get(name)
        .context("winter-section source input missing")?;
    ensure!(
        route.sources.get(name) == Some(&hash(text.as_bytes())),
        "winter-section source preimage changed: {name}"
    );
    Ok(text)
}

pub(crate) fn compile(
    sources: &BTreeMap<String, String>,
    route: &Route,
    previous: bool,
) -> Result<Vec<u16>> {
    let name = if previous {
        &route.previous_layout
    } else {
        &route.candidate_layout
    };
    let text = source(sources, route, name)?;
    let layout: Layout = serde_json::from_str(text)?;
    ensure!(
        layout.schema_version == 2
            && layout.id == route.id
            && layout.dimensions_metatiles == route.dimensions_metatiles
            && layout.compiled_tileset
                == if previous {
                    route.old_tileset.as_str()
                } else {
                    route.new_tileset.as_str()
                },
        "winter layout identity/extent differs"
    );
    let mut blocks = layout.compile()?;
    if previous {
        for cap in &route.native_cap_overlays {
            let index = usize::from(cap.at[1]) * usize::from(route.dimensions_metatiles[0])
                + usize::from(cap.at[0]);
            ensure!(
                blocks.get(index) == Some(&cap.source),
                "original source/native cap preimage changed"
            );
            blocks[index] = cap.native;
        }
    }
    let bytes = blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        hash(&bytes)
            == if previous {
                route.compiled_before_sha256.as_str()
            } else {
                route.compiled_after_sha256.as_str()
            },
        "native whole-layout compilation differs from frozen source pixels"
    );
    Ok(blocks)
}
