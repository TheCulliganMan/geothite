//! Register the opening maps and explicitly bind the production title spawn.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, PokegearLandmark, RuntimeMapMetadata,
    RuntimeSpawnPoint,
};

pub(super) fn publish(
    mut pack: CompiledGamePack,
    modules: Vec<(MapModule, [u16; 2])>,
) -> Result<CompiledGamePack> {
    const GROUP: &str = "MINNESOTA_WINONA";
    const LANDMARK: &str = "LANDMARK_MN_WINONA";
    const MANIFEST: &str = "minnesota-winona-opening-v1";
    ensure!(
        !pack
            .data
            .runtime_map_metadata
            .values()
            .any(|m| m.group_id == 73)
            && pack
                .data
                .story_event_script_constants
                .global
                .get("SPAWN_HOME")
                == Some(&0),
        "Winona group73 occupied or unsupported original HOME slot"
    );
    pack.data
        .story_event_script_constants
        .global
        .insert(format!("MAPGROUP_{GROUP}"), 73);
    let landmark_id = pack
        .data
        .pokegear_landmarks
        .landmarks
        .iter()
        .map(|l| l.id)
        .max()
        .context("landmark table")?
        .checked_add(1)
        .context("Winona landmark overflow")?;
    u8::try_from(landmark_id).context("Winona previous-landmark byte")?;
    pack.data
        .pokegear_landmarks
        .landmarks
        .push(PokegearLandmark {
            id: landmark_id,
            constant: LANDMARK.into(),
            label: "MN_WINONA".into(),
            name: "WINONA".into(),
            x: 112,
            y: 128,
            region: "JOHTO".into(),
        });
    let first = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|s| s.identifier)
        .max()
        .context("spawn table")?
        .checked_add(1)
        .context("Winona recovery overflow")?;
    let mut original_home = pack
        .data
        .runtime_spawn_points
        .remove("0")
        .context("original HOME destination")?;
    ensure!(
        original_home.map_constant == "PLAYERS_HOUSE_2F",
        "original HOME destination differs"
    );
    original_home.identifier = first;
    pack.data
        .runtime_spawn_points
        .insert(first.to_string(), original_home);
    let first = first
        .checked_add(1)
        .context("opening recovery identity overflow")?;
    for (index, (mut module, [x, y])) in modules.into_iter().enumerate() {
        let name = module.id.clone();
        let constant = module
            .attributes
            .map_constant
            .clone()
            .context("Winona map constant")?;
        let map_id = u16::try_from(index + 1)?;
        let identifier = if name == "MnWinonaHome2F" {
            0
        } else {
            first
                .checked_add(u16::try_from(index)?)
                .context("recovery identity overflow")?
        };
        module.attributes.location = Some(LANDMARK.into());
        module.attributes.map_group_constant = Some(format!("MAPGROUP_{GROUP}"));
        pack.data
            .pokegear_landmarks
            .map_to_landmark
            .insert(name.clone(), LANDMARK.into());
        let extension = CompiledMapExtension {
            manifest_id: format!("{MANIFEST}-{map_id}"),
            map_name: name.clone(),
            map_constant: constant.clone(),
            metadata: RuntimeMapMetadata {
                name: name.clone(),
                constant: constant.clone(),
                group_name: GROUP.into(),
                group_id: 73,
                map_id,
                width: module.attributes.width,
                height: module.attributes.height,
                environment: module
                    .attributes
                    .environment
                    .clone()
                    .context("Winona environment")?,
                phone_service: module.attributes.phone_service,
            },
            spawn_key: identifier.to_string(),
            spawn: RuntimeSpawnPoint {
                identifier,
                map_name: name.clone(),
                map_constant: constant,
                group_name: GROUP.into(),
                group_id: 73,
                map_id: i16::try_from(map_id)?,
                tile_x: i16::try_from(x)?,
                tile_y: i16::try_from(y)?,
                metatile_x: i16::try_from(x / 2)?,
                metatile_y: i16::try_from(y / 2)?,
                subtile_x: i16::try_from(x % 2)?,
                subtile_y: i16::try_from(y % 2)?,
            },
            module,
            wild_encounters: None,
            start_new_game_here: false,
        };
        crate::validate_compiled_map_extension(&extension)?;
        crate::insert_compiled_map_extension(&mut pack, extension)?;
        if !matches!(
            name.as_str(),
            "MnWinona" | "MnWinonaHome1F" | "MnWinonaHome2F" | "MnWinonaPokecenter1F"
        ) {
            pack.data
                .runtime_spawn_points
                .remove(&identifier.to_string());
        }
    }
    let entry = pack
        .data
        .runtime_title_screen
        .program
        .entrypoints
        .get("new_game")
        .context("production New Game entry")?
        .clone();
    let block = pack
        .data
        .runtime_title_screen
        .program
        .blocks
        .get_mut(&entry)
        .context("New Game block")?;
    for (target, value) in [
        ("wDefaultSpawnpoint", 0_u64),
        ("wPrevLandmark", u64::from(landmark_id)),
    ] {
        let operations = block
            .operations
            .iter_mut()
            .filter(|op| {
                op.op == "write_memory_byte"
                    && op.fields.get("target").and_then(|v| v.as_str()) == Some(target)
            })
            .collect::<Vec<_>>();
        ensure!(
            operations.len() == 1,
            "one production title write for {target} required"
        );
        operations
            .into_iter()
            .next()
            .context("validated title write")?
            .fields
            .insert("value".into(), json!(value));
    }
    pack.data
        .story_event_script_constants
        .global
        .insert("SPAWN_HOME".into(), 0);
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::RuntimeTitlePresentationParameters::from_program(
        &pack.data.runtime_title_screen.program,
    )?;
    Ok(pack)
}
