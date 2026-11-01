//! Authored ordinary opposition between the rival and executive encounters.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_lair_authoring as a;
use crate::northstar_rival_arc as arc;

#[derive(Deserialize)]
pub(crate) struct Guard {
    map: String,
    id: String,
    at: [u16; 2],
    facing: String,
    sight: u8,
    party: Vec<(String, u8)>,
    seen: String,
}

pub(crate) fn install(pack: &mut CompiledGamePack, guards: &[Guard]) -> Result<()> {
    ensure!(guards.len() == 24, "complete authored grunt coverage");
    for guard in guards {
        ensure!(
            matches!(guard.facing.as_str(), "UP" | "DOWN" | "LEFT" | "RIGHT")
                && (1..=4).contains(&guard.sight),
            "native sight direction/range"
        );
        let mut room = arc::map(pack, &guard.map)?;
        ensure!(
            !room.objects.iter().any(|o| [o.x, o.y] == guard.at),
            "occupied guard station {}",
            guard.id
        );
        let flag = format!("EVENT_{}_DEFEATED", guard.id);
        a::flag(pack, &flag)?;
        let party = guard
            .party
            .iter()
            .map(|(s, l)| (s.as_str(), *l))
            .collect::<Vec<_>>();
        let ice = guard.id.starts_with("MN_ICE_");
        a::trainer(
            pack,
            &guard.id,
            "GRUNTM",
            if ice { "ICE" } else { "LODE" },
            &party,
        )?;
        let seen = format!("{}SeenText", guard.id);
        let won = format!("{}WonText", guard.id);
        let lost = format!("{}LostText", guard.id);
        let after = format!("{}After", guard.id);
        authoring::text(&mut room, &seen, &[&guard.seen]);
        authoring::text(
            &mut room,
            &won,
            &[if ice {
                "My orders came from upstairs. I still carried them out."
            } else {
                "I knew what we were taking."
            }],
        );
        authoring::text(&mut room, &lost, &["You should have kept walking."]);
        arc::put(
            &mut room,
            &guard.id,
            vec![c(
                "trainer",
                &["GRUNTM", &guard.id, &flag, &seen, &won, &lost, &after],
            )],
        );
        arc::put(
            &mut room,
            &after,
            vec![c("endifjustbattled", &[]), c("jumptextfaceplayer", &[&won])],
        );
        a::actor(
            pack,
            &mut room,
            &guard.id,
            if ice {
                "SPRITE_OFFICER"
            } else {
                "SPRITE_ROCKET"
            },
            guard.at,
        )?;
        let actor = room.objects.last_mut().context("native grunt")?;
        actor.object_type = "OBJECTTYPE_TRAINER".into();
        actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{}", guard.facing);
        actor.radius = u16::from(guard.sight);
        arc::publish(pack, room)?;
    }
    Ok(())
}
