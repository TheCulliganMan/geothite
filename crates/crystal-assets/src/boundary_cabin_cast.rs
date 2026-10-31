//! Reconcile the cabin volunteer without duplicating the outdoor Root Beer Lady.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::northstar_rival_arc as arc;

const MAP: &str = "MnBoundaryRootBeerCabin";
const OLD: &str = "MN_INTERIOR_DOROTHY";
const NEW: &str = "MN_INTERIOR_CABIN_VOLUNTEER";
const OUTSIDE: &str = "BWS_ROOT_BEER_LADYRootBeerService";
const MANIFEST: &str = "boundary-cabin-volunteer-v1";

#[derive(Deserialize)]
struct Revision {
    schema_version: u8,
    id: String,
    map: String,
    canonical_exterior_service: String,
    old_script: String,
    old_sprite: String,
    new_script: String,
    new_sprite: String,
    tile: [u16; 2],
    approach: [u16; 2],
    preserved_paddler_script: String,
    lines: Vec<String>,
}

/// Replaces the duplicate indoor Dorothy with a distinct native cabin volunteer.
/// The outdoor Dorothy and root-beer service, existing indoor care operation,
/// paddler, room geometry and every unrelated map and catalog remain unchanged.
/// The original hash-pinned interior story source is not rewritten.
///
/// # Errors
/// Rejects duplicate installation, incompatible source/actors, absent or duplicate
/// exterior Dorothy, changed care semantics, occupied labels or invalid content.
pub fn revise_boundary_cabin_cast(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let revision: Revision = serde_json::from_str(source)?;
    ensure!(
        revision.schema_version == 1
            && revision.id == MANIFEST
            && revision.map == MAP
            && revision.canonical_exterior_service == OUTSIDE
            && revision.old_script == OLD
            && revision.old_sprite == "SPRITE_GRANNY"
            && revision.new_script == NEW
            && revision.new_sprite == "SPRITE_TEACHER"
            && revision.tile == [5, 3]
            && revision.approach == [5, 4]
            && revision.preserved_paddler_script == "MN_INTERIOR_CABIN_PADDLER",
        "canonical cabin cast revision required"
    );
    ensure!(
        !base.report.manifests.iter().any(|s| s == MANIFEST),
        "cabin cast already reconciled"
    );
    let exterior = base
        .data
        .maps
        .values()
        .flat_map(|m| m.objects.iter().map(move |o| (m, o)))
        .filter(|(_, o)| o.script == OUTSIDE)
        .collect::<Vec<_>>();
    ensure!(
        exterior.len() == 1 && exterior[0].0.id != MAP && exterior[0].1.sprite == "SPRITE_GRANNY",
        "one canonical outdoor Dorothy required"
    );
    let mut pack = base.clone();
    let mut room = arc::map(&pack, MAP)?;
    ensure!(
        room.objects.len() == 2
            && room.objects.iter().filter(|o| o.script == OLD).count() == 1
            && room
                .objects
                .iter()
                .filter(|o| o.script == revision.preserved_paddler_script)
                .count()
                == 1,
        "expected duplicate healer and preserved paddler required"
    );
    for at in [revision.tile, revision.approach] {
        ensure!(
            crate::authored_town_geometry::kind(&pack, &room, at)? == "FLOOR",
            "unsafe cabin care anchor {at:?}"
        );
    }
    ensure!(
        !room.scripts.contains_key(NEW) && !pack.data.map_scripts.contains_key(NEW),
        "occupied volunteer identity"
    );
    let old_text = format!("{OLD}InteriorText");
    let new_text = format!("{NEW}InteriorText");
    let mut program = room
        .scripts
        .remove(OLD)
        .context("existing indoor care")?
        .as_array()
        .context("native care program")?
        .clone();
    let commands = [
        "faceplayer",
        "opentext",
        "writetext",
        "waitbutton",
        "special",
        "closetext",
        "end",
    ];
    ensure!(
        program.len() == commands.len()
            && program
                .iter()
                .zip(commands)
                .all(|(row, command)| row["command"] == command),
        "preserve original care control flow"
    );
    for (index, row) in program.iter().enumerate() {
        let expected: Vec<&str> = match index {
            2 => vec![&old_text],
            4 => vec!["HealParty"],
            _ => Vec::new(),
        };
        ensure!(
            row["args"] == serde_json::json!(expected),
            "changed original care operands"
        );
    }
    room.scripts
        .remove(&old_text)
        .context("duplicate identity text")?;
    ensure!(
        !room
            .scripts
            .values()
            .any(|body| body.to_string().contains(OLD)),
        "another program references the old indoor identity"
    );
    program[2]["args"] = serde_json::json!([new_text]);
    arc::put(&mut room, NEW, program);
    ensure!(
        !revision.lines.is_empty()
            && revision
                .lines
                .iter()
                .all(|line| !line.contains("I am DOROTHY") && !line.contains("ROOT BEER LADY")),
        "volunteer must be a distinct person"
    );
    authoring::text(
        &mut room,
        &new_text,
        &revision
            .lines
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    let actor = room
        .objects
        .iter_mut()
        .find(|o| o.script == OLD)
        .context("indoor healer actor")?;
    ensure!(
        [actor.x, actor.y] == revision.tile && actor.sprite == revision.old_sprite,
        "original healer footing or art changed"
    );
    actor.script = NEW.into();
    actor.sprite = revision.new_sprite;
    arc::publish(&mut pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
