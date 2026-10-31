//! Register the existing Grand Marais clinic as an actual recovery service.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::RuntimeSpawnPoint;
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_rival_arc::{self as arc, put};

const ROOM: &str = "MnGrandMaraisPokecenter";
const CONSTANT: &str = "MN_GRAND_MARAIS_POKECENTER";
const NURSE: &str = "MnGrandMaraisNurse";
const MANIFEST: &str = "minnesota-marais-clinic-recovery-v1";

pub(super) fn wire(pack: &mut CompiledGamePack) -> Result<()> {
    let mut clinic = arc::map(pack, ROOM)?;
    let expected = json!([
        c("blackoutmod", &[CONSTANT]),
        c("jumpstd", &["PokecenterNurseScript"])
    ]);
    let existing = pack
        .data
        .runtime_spawn_points
        .values()
        .filter(|s| s.map_constant == CONSTANT)
        .collect::<Vec<_>>();
    if !existing.is_empty() {
        ensure!(
            existing.len() == 1
                && existing[0].map_name == ROOM
                && (existing[0].tile_x, existing[0].tile_y) == (3, 5)
                && clinic.scripts.get(NURSE) == Some(&expected),
            "incompatible existing Marais recovery"
        );
        return Ok(());
    }
    ensure!(
        clinic.scripts.get(NURSE) == Some(&json!([c("jumpstd", &["PokecenterNurseScript"])])),
        "Marais nurse changed before recovery wiring"
    );
    ensure!(
        crate::authored_town_geometry::kind(pack, &clinic, [3, 5])? == "FLOOR"
            && !clinic.objects.iter().any(|o| [o.x, o.y] == [3, 5])
            && !clinic.events.warps.iter().any(|w| [w.x, w.y] == [3, 5]),
        "recovery requires unoccupied passive clinic floor"
    );
    let metadata = pack
        .data
        .runtime_map_metadata
        .get(CONSTANT)
        .context("native Marais clinic metadata")?
        .clone();
    let identifier = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("recovery catalog")?
        .checked_add(1)
        .context("recovery identifier overflow")?;
    let spawn = RuntimeSpawnPoint {
        identifier,
        map_name: ROOM.into(),
        map_constant: CONSTANT.into(),
        group_name: metadata.group_name,
        group_id: i16::try_from(metadata.group_id)?,
        map_id: i16::try_from(metadata.map_id)?,
        tile_x: 3,
        tile_y: 5,
        metatile_x: 1,
        metatile_y: 2,
        subtile_x: 1,
        subtile_y: 1,
    };
    put(
        &mut clinic,
        NURSE,
        expected.as_array().context("nurse program")?.clone(),
    );
    arc::publish(pack, clinic)?;
    ensure!(
        pack.data
            .runtime_spawn_points
            .insert(identifier.to_string(), spawn)
            .is_none(),
        "occupied recovery identifier"
    );
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}
