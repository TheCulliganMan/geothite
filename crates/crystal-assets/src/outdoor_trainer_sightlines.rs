//! Align a trainer's facing with a reachable lane rather than a cliff face.

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrainerTurn {
    pub map: String,
    pub script: String,
    pub tile: [u16; 2],
    pub before: String,
    pub after: String,
}

pub(super) fn apply(pack: &mut CompiledGamePack, turn: &TrainerTurn) -> Result<()> {
    let (dx, dy) = match turn.after.as_str() {
        "SPRITEMOVEDATA_STANDING_RIGHT" => (1, 0),
        "SPRITEMOVEDATA_STANDING_LEFT" => (-1, 0),
        "SPRITEMOVEDATA_STANDING_DOWN" => (0, 1),
        "SPRITEMOVEDATA_STANDING_UP" => (0, -1),
        _ => anyhow::bail!("trainer sightline requires a native standing direction"),
    };
    let map = pack
        .data
        .maps
        .get_mut(&turn.map)
        .context("trainer exterior")?;
    let index = map
        .objects
        .iter()
        .position(|o| o.script == turn.script)
        .context("existing trainer object")?;
    let object = &map.objects[index];
    ensure!(
        object.object_type == "OBJECTTYPE_TRAINER"
            && [object.x, object.y] == turn.tile
            && object.spritemovedata == turn.before
            && turn.before != turn.after
            && object.sightline_direction_override.is_none()
            && object.radius > 0,
        "trainer source identity, coordinates or facing differ"
    );
    let width = usize::from(map.attributes.width);
    let height = usize::from(map.attributes.height);
    let collision = &pack.data.tilesets[&map.attributes.tileset_name].collision;
    // Crystal's sight test is geometric. Author the complete challenge lane so
    // the trainer can actually approach instead of challenging across a wall.
    for distance in 1..=object.radius {
        let x = i32::from(object.x) + dx * i32::from(distance);
        let y = i32::from(object.y) + dy * i32::from(distance);
        let (x, y) = usize::try_from(x)
            .ok()
            .zip(usize::try_from(y).ok())
            .filter(|(x, y)| *x < width * 2 && *y < height * 2)
            .context("trainer sightline leaves its exterior")?;
        let block = map.blocks[y / 2 * width + x / 2];
        let kind = &collision[&format!("{block:02x}")][y % 2 * 2 + x % 2];
        ensure!(
            matches!(kind.as_str(), "FLOOR" | "TALL_GRASS")
                && !map
                    .objects
                    .iter()
                    .any(|o| usize::from(o.x) == x && usize::from(o.y) == y),
            "trainer must face an unobstructed walkable approach lane"
        );
    }
    map.objects[index].spritemovedata.clone_from(&turn.after);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires the external sightline candidate; reads current roster/curation at runtime"]
    fn mina_every_scaled_team_uses_active_minnesota_families() {
        let pack = crate::read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_OUTDOOR_SIGHTLINE_PACK")
                .expect("exact external sightline pack"),
        ))
        .expect("verified candidate");
        let directory =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/pokemon");
        let read = |name| {
            serde_json::from_slice::<serde_json::Value>(
                &std::fs::read(directory.join(name)).expect("canonical roster source"),
            )
            .expect("canonical roster JSON")
        };
        let roster = read("roster.json");
        let curation = read("roster-curation.json");
        let retired: BTreeSet<_> = curation["retired_families"]
            .as_array()
            .expect("retired families")
            .iter()
            .map(|f| f["id"].as_str().expect("family ID"))
            .collect();
        let active: BTreeSet<_> = roster["families"]
            .as_array()
            .expect("roster families")
            .iter()
            .filter(|f| !retired.contains(f["id"].as_str().expect("family ID")))
            .flat_map(|f| f["members"].as_array().expect("family members"))
            .map(|m| {
                m["name"]
                    .as_str()
                    .expect("species name")
                    .to_ascii_uppercase()
            })
            .collect();
        let trainer = &pack.data.trainers.trainers["MN_R09_T03"];
        let first = trainer
            .party
            .iter()
            .map(|p| p.level)
            .max()
            .expect("Mina team");
        for ace in first..=100 {
            let team = pack
                .data
                .rematch_trainer_at_ace(trainer, ace)
                .expect("actual production evolved team");
            for member in team.party {
                assert!(
                    active.contains(&member.species),
                    "ace{ace}: {}",
                    member.species
                );
            }
        }
    }
}
