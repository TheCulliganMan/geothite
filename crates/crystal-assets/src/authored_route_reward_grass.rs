//! Enforce optional cache habitat using the same native surface as publication.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_design::RouteDesign;
use crate::authored_route_modpack::collision;
use crate::authored_route_surface;

pub(crate) fn validate(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
) -> Result<()> {
    let [width, height] = design.dimensions_metatiles.map(|value| value * 2);
    let mut ids = BTreeSet::new();
    for reward in &design.optional_grass_rewards {
        ensure!(
            ids.insert(&reward.actor_id) && reward.minimum_grass_steps > 0,
            "reward habitat needs a unique pickup and nonzero encounter threshold"
        );
        let actor = design
            .items
            .iter()
            .find(|item| item.id == reward.actor_id)
            .context("grass reward does not identify a native pickup")?;
        let mut grass = BTreeSet::new();
        for [x0, y0, x1, y1] in
            std::iter::once(reward.bounds_core).chain(reward.additional_bounds_core.iter().copied())
        {
            ensure!(
                x0 < x1 && y0 < y1 && x1 <= width && y1 <= height,
                "{}: reward grass bounds exceed the route",
                actor.id
            );
            let prior = grass.len();
            for y in y0..y1 {
                for x in x0..x1 {
                    match collision(pack, blocks, width / 2, [x, y], tileset)? {
                        "TALL_GRASS" => {
                            grass.insert([x, y]);
                        }
                        "WALL" => {}
                        _ => anyhow::bail!(
                            "{}: reward threshold contains a non-grass opening",
                            actor.id
                        ),
                    }
                }
            }
            ensure!(
                grass.len() > prior,
                "{}: empty or redundant reward habitat",
                actor.id
            );
        }
        // A trainer, resident or another collectible is not a permanent terrain
        // barrier. Clearing their bodies conservatively checks eventual access.
        let occupied = BTreeSet::from([actor.tile]);
        let costs = authored_route_surface::walking_grass_costs(
            pack, design, blocks, tileset, &occupied, &grass,
        )?;
        let [x, y] = actor.tile;
        let minimum = [
            [x.saturating_sub(1), y],
            [x + 1, y],
            [x, y.saturating_sub(1)],
            [x, y + 1],
        ]
        .iter()
        .filter(|tile| {
            crate::authored_route_events::chunk(&design.map_prefix, **tile)
                == crate::authored_route_events::chunk(&design.map_prefix, actor.tile)
        })
        .filter_map(|tile| costs.get(tile))
        .min();
        ensure!(
            minimum.is_some_and(|steps| *steps >= u32::from(reward.minimum_grass_steps)),
            "{}: optional reward grass is bypassable, too short or disconnected (minimum {minimum:?})",
            actor.id
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::validate;
    use crate::authored_route_design::RouteDesign;
    use crate::authored_route_layout::Layout;

    #[test]
    #[ignore = "requires CRYSTAL_R10_CACHE_REWARD_PACK and CRYSTAL_R10_CACHE_SOURCE_ROOT; reads native pack and review layouts at runtime"]
    fn r10_reward_grass_rejects_dry_forecourt_and_accepts_four_step_native_barrier() {
        let pack = PathBuf::from(
            std::env::var_os("CRYSTAL_R10_CACHE_REWARD_PACK")
                .expect("explicit native north-stairs parent"),
        );
        let root = PathBuf::from(
            std::env::var_os("CRYSTAL_R10_CACHE_SOURCE_ROOT")
                .expect("Minnesota revisions directory"),
        );
        let read = |revision: &str, file: &str| {
            std::fs::read_to_string(root.join(revision).join(file))
                .expect("authored review prerequisite")
        };
        let base = crate::read_verified_compiled_game_pack(&pack).expect("verified native parent");
        let rejected = "r10-cache-cliff-v01";
        let rejected_design = RouteDesign::parse(&read(rejected, "dossier.json"))
            .expect("explicit closed carrier face");
        let baseline = Layout::parse(&read(rejected, "before.layout.json"))
            .expect("exact native preimage")
            .compile()
            .expect("baseline terrain");
        validate(&base, &rejected_design, &baseline, "mn_r10_rockstairs")
            .expect("original cache still requires encounter grass");
        let dry = Layout::parse(&read(rejected, "after.layout.json"))
            .expect("retained failed contour")
            .compile()
            .expect("rejected terrain");
        let error = validate(&base, &rejected_design, &dry, "mn_r10_rockstairs")
            .expect_err("a source render cannot certify an avoidable habitat");
        assert!(
            error.to_string().contains("optional reward grass"),
            "{error:#}"
        );
        let art = crate::add_r10_cache_art(&base).expect("two exact native glyph joins");
        let current = "r10-cache-cliff-v02";
        let revised = crate::revise_authored_route_terrain(
            &art,
            &read(current, "before.layout.json"),
            &read(current, "after.layout.json"),
            &read(current, "dossier.json"),
        )
        .expect("complete native terrain/anchor/habitat validation");
        assert_eq!(revised.data.maps["MnR10C00R00"].blocks[10 * 32 + 5], 0x82);
        assert_eq!(revised.data.maps["MnR10C00R00"].blocks[4 * 32 + 20], 0x81);
        assert_eq!(
            revised.data.maps["MnR10C00R00"].objects,
            base.data.maps["MnR10C00R00"].objects
        );
    }

    #[test]
    #[ignore = "requires CRYSTAL_R10_WEST_TERRACE_PACK and CRYSTAL_R10_CACHE_SOURCE_ROOT; reads native pack and authored layouts at runtime"]
    fn r10_closed_west_terrace_rejects_dry_front_lip() {
        let pack = PathBuf::from(
            std::env::var_os("CRYSTAL_R10_WEST_TERRACE_PACK")
                .expect("explicit native cache terrace parent"),
        );
        let root = PathBuf::from(
            std::env::var_os("CRYSTAL_R10_CACHE_SOURCE_ROOT")
                .expect("Minnesota revisions directory"),
        )
        .join("r10-west-terrace-v01");
        let read = |file: &str| {
            std::fs::read_to_string(root.join(file)).expect("authored west terrace prerequisite")
        };
        let base = crate::read_verified_compiled_game_pack(&pack).expect("verified native parent");
        let art = crate::add_r10_west_terrace_art(&base).expect("exact native cliff/grass joins");
        let revised = crate::revise_authored_route_terrain(
            &art,
            &read("before.layout.json"),
            &read("after.layout.json"),
            &read("dossier.json"),
        )
        .expect("closed terrace preserves anchors, returns and four-step habitat");
        assert_eq!(
            revised.data.maps["MnR10C00R00"].objects,
            base.data.maps["MnR10C00R00"].objects
        );
        let mut dry = Layout::parse(&read("after.layout.json"))
            .expect("closed west terrace")
            .compile()
            .expect("exact terrain");
        for x in [12, 13, 14, 15] {
            assert_eq!(dry[6 * 48 + x], 0x84);
            dry[6 * 48 + x] = 0x72;
        }
        let mut design =
            RouteDesign::parse(&read("dossier.json")).expect("native west terrace contracts");
        // The rejected early pass omitted these front bands. Its brown lip
        // reaches the cache after only two actual grass steps, even though the
        // staircase is farther away. Test the native minimum, not band syntax.
        design.optional_grass_rewards[0]
            .additional_bounds_core
            .truncate(2);
        let error = validate(&art, &design, &dry, "mn_r10_rockstairs")
            .expect_err("a dry front lip must not bypass the reward habitat");
        assert!(
            error.to_string().contains("optional reward grass"),
            "{error:#}"
        );
    }
}
