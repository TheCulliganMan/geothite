//! Identify the two through-cave mouths and the ice cavern on Route 20's existing posts.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_route_design::RouteDesign;
use crate::authored_route_events::{chunk, local};
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring;

/// Publishes the two signs from the same route dossier used for a fresh regional build.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let route: RouteDesign = serde_json::from_str(include_str!(
        "../../../planning/minnesota/routes/MN_R20.json"
    ))?;
    for id in ["MN_R20_SIGN_GRANITE", "MN_R20_SIGN_MEADOW"] {
        let sign = route
            .signs
            .iter()
            .find(|s| s.id == id)
            .with_context(|| format!("Route 20 sign {id}"))?;
        let name = chunk(&route.map_prefix, sign.tile);
        let at = local(sign.tile);
        let mut room = pack
            .data
            .maps
            .get(&name)
            .with_context(|| format!("Route 20 map {name}"))?
            .clone();
        let label = format!("{id}Text");
        ensure!(
            room.events
                .bg_events
                .iter()
                .any(|e| e.script == id && e.event_type == "BGEVENT_READ" && [e.x, e.y] == at)
                && room.scripts.contains_key(&label),
            "expected the original readable {id} post in {name}"
        );
        let lines: Vec<_> = sign.lines.iter().map(String::as_str).collect();
        authoring::text(&mut room, &label, &lines);
        room.script_text_bodies = crate::parse_script_text_bodies(&name, &room.scripts)?;
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}
