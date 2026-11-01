//! Frozen connection identities and conservative native walking/Surf geometry.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, ensure};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;

use super::{Source, TOWN};
use crate::{CompiledGamePack, MapModule};

pub(super) fn validate_source(s: &Source) -> Result<()> {
    let patches = s
        .patches
        .iter()
        .map(|p| {
            (
                p.map.as_str(),
                p.metatile,
                p.expected.as_str(),
                p.block.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    ensure!(
        s.patches.len() == 6
            && patches
                == BTreeSet::from([
                    ("MnR10C01R00", [15, 10], "02", "05"),
                    ("MnR10C01R00", [15, 11], "02", "05"),
                    ("MnR10C01R00", [15, 12], "02", "01"),
                    ("MnR10C01R00", [15, 13], "02", "05"),
                    (TOWN, [2, 11], "05", "45"),
                    (TOWN, [2, 12], "45", "01"),
                ]),
        "retain the six exact Duluth mouth and sign patches"
    );
    ensure!(
        s.sign_relocation.script == "MnDuluth_DULUTH_SIGN"
            && s.sign_relocation.original == [4, 24]
            && s.sign_relocation.destination == [4, 22],
        "retain the relocated readable west sign"
    );
    ensure!(
        s.passages.len() == 2,
        "Duluth needs exactly two walking links"
    );
    for (p, north) in s.passages.iter().zip([false, true]) {
        let (
            route,
            outward,
            reverse,
            offset,
            town_edge,
            route_edge,
            bodies,
            inside,
            outside,
            ids,
            root,
            inward,
            event,
        ) = if north {
            (
                "MnR11C00R03",
                "north",
                "south",
                -7,
                [[32, 0], [33, 0]],
                [[46, 47], [47, 47]],
                [[32, 2], [33, 2]],
                [[32, 3], [33, 3]],
                [[32, 1], [33, 1]],
                ["MN_DULUTH_SHORE_GUARD_WEST", "MN_DULUTH_SHORE_GUARD_EAST"],
                "MnDuluthShoreCheck",
                "DOWN",
                "EVENT_MN_LIGHTHOUSE_SIGNAL",
            )
        } else {
            (
                "MnR10C01R00",
                "west",
                "east",
                0,
                [[0, 24], [0, 25]],
                [[31, 24], [31, 25]],
                [[2, 24], [2, 25]],
                [[3, 24], [3, 25]],
                [[1, 24], [1, 25]],
                ["MN_DULUTH_RANGE_GUARD_NORTH", "MN_DULUTH_RANGE_GUARD_SOUTH"],
                "MnDuluthRangeCheck",
                "RIGHT",
                "EVENT_MN_DEPOT_CLEARED",
            )
        };
        let badges = if north {
            &BADGE_ENGINE_FLAGS[..8]
        } else {
            &BADGE_ENGINE_FLAGS[6..7]
        };
        ensure!(
            p.route == route
                && p.outward == outward
                && p.reverse == reverse
                && p.offset == offset
                && p.town_edge == town_edge
                && p.route_edge == route_edge
                && p.bodies == bodies
                && p.inside == inside
                && p.outside == outside
                && p.identifiers == ids
                && p.root == root
                && p.incoming_facing == inward
                && p.required_event == event
                && p.badge_flags.iter().map(String::as_str).collect::<Vec<_>>() == badges,
            "retain exact Duluth {outward} native checkpoint, lane offsets and prerequisites"
        );
        for lines in [&p.badge_lines, &p.evidence_lines] {
            ensure!(
                !lines.is_empty()
                    && lines.iter().all(|s| !s.is_empty()
                        && s.len() <= 18
                        && s.chars()
                            .all(|c| c.is_ascii() && !c.is_control() && c != '"')),
                "Duluth checkpoint text must fit native pages"
            );
        }
    }
    Ok(())
}

fn collision<'a>(pack: &'a CompiledGamePack, map: &MapModule, at: [u16; 2]) -> Result<&'a str> {
    crate::authored_route_modpack::collision(
        pack,
        &map.blocks,
        map.attributes.width,
        at,
        &map.attributes.tileset_name,
    )
}
fn floor(pack: &CompiledGamePack, map: &MapModule, at: [u16; 2]) -> Result<()> {
    ensure!(
        collision(pack, map, at)? == "FLOOR",
        "{} passage {at:?} is not FLOOR",
        map.id
    );
    Ok(())
}

pub(super) fn validate_maps(
    pack: &CompiledGamePack,
    maps: &BTreeMap<String, MapModule>,
    s: &Source,
) -> Result<()> {
    for (name, extent) in [
        (TOWN, [32, 24]),
        ("MnR10C01R00", [16, 20]),
        ("MnR11C00R03", [32, 24]),
    ] {
        ensure!(
            [maps[name].attributes.width, maps[name].attributes.height] == extent,
            "changed Duluth/route extent"
        );
    }
    let town = &maps[TOWN];
    let mut blocked = BTreeSet::new();
    for p in &s.passages {
        for at in p
            .town_edge
            .into_iter()
            .chain(p.bodies)
            .chain(p.inside)
            .chain(p.outside)
        {
            floor(pack, town, at)?;
            ensure!(
                !town.objects.iter().any(|a| [a.x, a.y] == at)
                    && !town.events.coord_events.iter().any(|e| [e.x, e.y] == at)
                    && !town.events.warps.iter().any(|w| [w.x, w.y] == at),
                "Duluth checkpoint overlaps original interaction"
            );
        }
        for at in p.inside {
            let retreat = if p.outward == "west" {
                [at[0] + 1, at[1]]
            } else {
                [at[0], at[1] + 1]
            };
            floor(pack, town, retreat)?;
            ensure!(
                !town.objects.iter().any(|a| [a.x, a.y] == retreat),
                "Duluth refusal hits original actor"
            );
        }
        for at in p.route_edge {
            floor(pack, &maps[&p.route], at)?;
        }
        let closed = BTreeSet::from(p.inside);
        let seen =
            crate::authored_route_surface::reachable_map_with_surf(pack, town, [6, 24], &closed)?;
        ensure!(
            p.town_edge.iter().all(|at| !seen.contains(at)),
            "Duluth {} walking/Surf checkpoint bypass",
            p.outward
        );
        blocked.extend(p.inside);
        // Only the two paired walking cells may open on the newly linked route edge.
        let route = &maps[&p.route];
        let edge = if p.reverse == "east" {
            (0..route.attributes.height * 2)
                .map(|y| [route.attributes.width * 2 - 1, y])
                .collect::<Vec<_>>()
        } else {
            (0..route.attributes.width * 2)
                .map(|x| [x, route.attributes.height * 2 - 1])
                .collect::<Vec<_>>()
        };
        for at in edge {
            ensure!(
                p.route_edge.contains(&at)
                    || !matches!(
                        collision(pack, route, at)?,
                        "FLOOR" | "TALL_GRASS" | "WATER"
                    ),
                "unpaired Duluth route walking/Surf edge {} {at:?}",
                route.id
            );
        }
    }
    let seen =
        crate::authored_route_surface::reachable_map_with_surf(pack, town, [6, 24], &blocked)?;
    ensure!(
        seen.contains(&[4, 23]),
        "relocated Duluth sign has no public approach"
    );
    ensure!(
        !town.objects.iter().any(|a| [a.x, a.y] == [4, 23]),
        "relocated sign approach occupied"
    );
    // Existing passive city returns are actual FLOOR warp records, unlike
    // the active facade doors. Derive the service anchors from installed native
    // content rather than copying the city author's door catalog here.
    for warp in &town.events.warps {
        let at = [warp.x, warp.y];
        if collision(pack, town, at)? == "FLOOR" {
            ensure!(
                seen.contains(&at),
                "closed checkpoints deny Duluth service return {at:?}"
            );
        }
    }
    for y in 0..48 {
        for x in 0..64 {
            if x != 0 && x != 63 && y != 0 && y != 47 {
                continue;
            }
            let at = [x, y];
            ensure!(
                s.passages.iter().any(|p| p.town_edge.contains(&at))
                    || !matches!(collision(pack, town, at)?, "FLOOR" | "TALL_GRASS" | "WATER"),
                "unpaired Duluth walking/Surf boundary {at:?}"
            );
        }
    }
    Ok(())
}
