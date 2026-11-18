//! The public riverside walk joins the Route 4 picnic lawn through a garden gate.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::authored_route_surface::walking_map_distances;
use crate::{CompiledGamePack, northstar_regional_design};

const PARK: &str = "MnR04C00R00";
const TERRAIN: &[u8] =
    include_bytes!("../../../planning/minnesota/revisions/wayfinding-v01/river-park.json");

fn crossing(pack: &CompiledGamePack) -> Result<[u32; 2]> {
    let room = pack.data.maps.get(PARK).context("river park")?;
    let actors: BTreeSet<_> = room.objects.iter().map(|o| [o.x, o.y]).collect();
    let promenade = [35, 27];
    let picnic_lawn = [44, 30];
    let distance = |a, b| -> Result<u32> {
        walking_map_distances(pack, room, a, &actors)?
            .get(&b)
            .copied()
            .context("park paths must connect")
    };
    Ok([
        distance(promenade, picnic_lawn)?,
        distance(picnic_lawn, promenade)?,
    ])
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    ensure!(
        crossing(pack)? == [22, 22],
        "expected the old riverside detour"
    );
    northstar_regional_design::apply_rooms(pack, TERRAIN)?;
    ensure!(
        crossing(pack)? == [12, 12],
        "the picnic path must work in both directions"
    );
    Ok(())
}
