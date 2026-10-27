//! Add the pinned native field-animation source to an existing game pack.
use anyhow::{Context, Result, ensure};
use serde_json::{Map, Value};

use crate::{
    CompiledGamePack, certify_synchronous_script_callasm_target,
    derive_compiled_game_pack_identity_from_manifest, verify_compiled_game_pack_for_runtime,
};

const MANIFEST_ID: &str = "native-field-scripts-source-repair";
// Actual Rust export of the pinned engine/events/field_moves.asm, SHA256
// f520f575b3abc4d6eb002dd26dcd8b89453e11132c4593f46c124070e6803495.
// Preserve these source bodies, including local branches and INCBIN references;
// this list is an import boundary, not a second native command catalog.
const FIELD_SOURCE_LABELS: [&str; 40] = [
    ".Coords@Cut_GetLeafSpawnCoords",
    ".Coords@Cut_Headbutt_GetPixelFacing",
    ".LoadCutGFX@OWCutAnimation",
    ".OAMloop@FlyToAnim",
    ".RestorePlayerSprite_DespawnLeaves@FlyToAnim",
    ".SpawnLeaf@FlyFunction_FrameTimer",
    ".done@ShakeHeadbuttTree",
    ".dw@OWCutJumptable",
    ".exit@FlyFromAnim",
    ".exit@FlyFunction_FrameTimer",
    ".exit@FlyToAnim",
    ".finish@OWCutAnimation",
    ".finished@Cut_WaitAnimSFX",
    ".left_side@Cut_GetLeafSpawnCoords",
    ".loop@FlyFromAnim",
    ".loop@FlyToAnim",
    ".loop@OWCutAnimation",
    ".loop@ShakeHeadbuttTree",
    ".top_side@Cut_GetLeafSpawnCoords",
    "BlindingFlash",
    "CutGrassGFX",
    "CutTreeGFX",
    "Cut_GetLeafSpawnCoords",
    "Cut_Headbutt_GetPixelFacing",
    "Cut_SpawnAnimateLeaves",
    "Cut_SpawnAnimateTree",
    "Cut_SpawnLeaf",
    "Cut_StartWaiting",
    "Cut_WaitAnimSFX",
    "FlyFromAnim",
    "FlyFunction_FrameTimer",
    "FlyFunction_InitGFX",
    "FlyToAnim",
    "HeadbuttTreeGFX",
    "HideHeadbuttTree",
    "OWCutAnimation",
    "OWCutJumptable",
    "PlayWhirlpoolSound",
    "ShakeHeadbuttTree",
    "TreeRelativeLocationTable",
];

/// Repairs retained native field-script source without replacing game content.
///
/// Both inputs must be verified packs. The fresh pinned source must have the
/// same 627 original OverworldEvents definitions and the 40 field-animation
/// definitions named above. Every overlapping body must be equal. Only missing
/// field definitions are copied into the existing catalog; existing standard
/// roots retain their order. Existing Headbutt entries and `Script_UsedWaterfall`
/// are added as roots when absent, including the deferred Waterfall loop.
/// Native Shake/Fly source certificates and executable Waterfall commands must
/// pass before the ordinary pack identity and manifest are published.
///
/// Maps, art, audio, encounters, items, event constants, HOME and recovery are
/// unchanged. This source repair does not implement new field-move gameplay.
/// Retaining the Headbutt menu root does not prove its physical source
/// fallthrough executes through the menu; contextual input remains a separate
/// controller contract.
/// Applying it twice is idempotent. Neither borrowed input is modified.
///
/// # Errors
/// Returns an error for invalid packs, ambiguous catalogs, a changed overlapping
/// body, incomplete/unexpected source imports, missing native field entries,
/// failed native source certificates or failed final pack verification.
pub fn build_native_field_scripts_modpack(
    base: &CompiledGamePack,
    source: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base).context("verify field-script repair base")?;
    verify_compiled_game_pack_for_runtime(source).context("verify fresh field-script source")?;
    let base_index = catalog_index(base, "OverworldEvents")?;
    let source_index = catalog_index(source, "OverworldEvents")?;
    let old = catalog(base, base_index, "OverworldEvents")?;
    let fresh = catalog(source, source_index, "OverworldEvents")?;
    ensure!(
        fresh.len() == 667,
        "fresh field source requires exactly 667 definitions"
    );
    for label in FIELD_SOURCE_LABELS {
        ensure!(
            fresh.contains_key(label),
            "missing native field source {label}"
        );
    }
    for (label, body) in old {
        ensure!(
            fresh.get(label) == Some(body),
            "native source overlap changed: {label}"
        );
    }
    for label in fresh
        .keys()
        .filter(|label| !FIELD_SOURCE_LABELS.contains(&label.as_str()))
    {
        ensure!(
            old.contains_key(label),
            "base lacks original native source {label}"
        );
    }
    let missing = FIELD_SOURCE_LABELS
        .iter()
        .filter(|label| !old.contains_key(**label))
        .count();
    ensure!(
        missing == 40 || missing == 0,
        "partial native field repair has {missing} missing definitions"
    );

    let mut pack = base.clone();
    let target = pack.data.story_events[base_index]["OverworldEvents"]
        .as_object_mut()
        .context("existing native OverworldEvents object")?;
    for label in FIELD_SOURCE_LABELS {
        if !target.contains_key(label) {
            target.insert(label.into(), fresh[label].clone());
        }
    }
    let standard_index = catalog_index(&pack, "StandardScripts")?;
    let roots = pack.data.story_events[standard_index]["StandardScripts"]["GlobalScriptRoots"]
        .as_array_mut()
        .context("existing native GlobalScriptRoots")?;
    for label in [
        "HeadbuttScript",
        "HeadbuttFromMenuScript",
        "Script_UsedWaterfall",
    ] {
        if !roots.iter().any(|root| root.as_str() == Some(label)) {
            roots.push(Value::String(label.into()));
        }
    }
    pack.data
        .materialize_global_scripts()
        .context("materialize repaired native globals")?;
    let globals = pack
        .data
        .global_scripts
        .as_ref()
        .context("repaired native global module")?;
    for label in ["ShakeHeadbuttTree", "FlyFromAnim", "FlyToAnim"] {
        certify_synchronous_script_callasm_target(&globals.definitions, label)
            .map_err(|error| anyhow::anyhow!("native visual source closure {label}: {error:?}"))?;
    }
    let map = "MnR11C00R00";
    let label = ".loop@Script_UsedWaterfall";
    let movement = pack.data.script_object_command(map, label, 0)?;
    ensure!(
        movement.command == "applymovement"
            && movement.object_id.as_deref() == Some("PLAYER")
            && movement.movement.as_deref() == Some(".WaterfallStep"),
        "native Waterfall object movement changed"
    );
    let check = pack.data.script_runtime_command(map, label, 1)?;
    ensure!(
        check.command == "callasm" && check.args == [".CheckContinueWaterfall"],
        "native Waterfall runtime check changed"
    );
    let repeat = pack.data.script_control_command(map, label, 2)?;
    ensure!(
        repeat.command == "iffalse" && repeat.resolved_target_script.as_deref() == Some(label),
        "native Waterfall loop branch changed"
    );
    ensure!(
        pack.data.script_control_command(map, label, 3)?.command == "end",
        "native Waterfall loop end changed"
    );
    let steps = pack
        .data
        .compiled_script_body(".WaterfallStep@Script_UsedWaterfall")
        .context("retained native Waterfall movement body")?;
    ensure!(
        steps[0]["command"] == "turn_waterfall" && steps[1]["command"] == "step_end",
        "native Waterfall movement body changed"
    );
    if !pack.report.manifests.iter().any(|id| id == MANIFEST_ID) {
        pack.report.manifests.push(MANIFEST_ID.into());
    }
    refresh_identity(&mut pack)?;
    verify_compiled_game_pack_for_runtime(&pack)
        .context("verify repaired native field-script pack")?;
    Ok(pack)
}

fn catalog_index(pack: &CompiledGamePack, key: &str) -> Result<usize> {
    let mut indices = pack
        .data
        .story_events
        .iter()
        .enumerate()
        .filter_map(|(index, value)| value.get(key).map(|_| index));
    let index = indices
        .next()
        .with_context(|| format!("missing existing {key} catalog"))?;
    ensure!(
        indices.next().is_none(),
        "ambiguous existing {key} catalogs"
    );
    Ok(index)
}

fn catalog<'a>(
    pack: &'a CompiledGamePack,
    index: usize,
    key: &str,
) -> Result<&'a Map<String, Value>> {
    pack.data.story_events[index][key]
        .as_object()
        .with_context(|| format!("native {key} object"))
}

fn refresh_identity(pack: &mut CompiledGamePack) -> Result<()> {
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use crystal_core::state::GameState;
    use crystal_core::world::map::{Direction, TilePosition};

    use super::*;
    use crate::ScriptRuntimeInputs;

    fn inputs() -> &'static (CompiledGamePack, CompiledGamePack) {
        static INPUTS: OnceLock<(CompiledGamePack, CompiledGamePack)> = OnceLock::new();
        INPUTS.get_or_init(|| {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            let base = std::env::var_os("CRYSTAL_NATIVE_FIELD_SCRIPTS_BASE_PACK")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    root.join("content-packs/minnesota-prairie-forest-link.crystalpack")
                });
            let source = PathBuf::from(
                std::env::var_os("CRYSTAL_NATIVE_FIELD_SCRIPTS_SOURCE_PACK")
                    .expect("fresh Rust-exported external source pack path"),
            );
            let read = |path: PathBuf| {
                crate::read_verified_compiled_game_pack(
                    path.canonicalize()
                        .expect("canonical completed external pack path"),
                )
                .expect("verified external pack")
            };
            (read(base), read(source))
        })
    }

    #[test]
    #[ignore = "requires current Minnesota and freshly Rust-exported source packs"]
    fn native_field_repair_preserves_content_and_is_idempotent() {
        let (base, source) = inputs();
        let pack = build_native_field_scripts_modpack(base, source)
            .expect("additive native source repair");
        assert_eq!(pack.runtime_files, base.runtime_files);
        assert_eq!(pack.audio_manifest, base.audio_manifest);
        assert_eq!(pack.compiled_audio, base.compiled_audio);
        assert_eq!(pack.audio_compression, base.audio_compression);
        let mut restored = pack.data.clone();
        restored.story_events = base.data.story_events.clone();
        restored.global_scripts = base.data.global_scripts.clone();
        assert_eq!(restored, base.data, "all unrelated game data must be exact");
        let old_index = catalog_index(base, "OverworldEvents").expect("old catalog");
        let old = catalog(base, old_index, "OverworldEvents").expect("old source");
        let new = catalog(&pack, old_index, "OverworldEvents").expect("repaired source");
        assert!(matches!(old.len(), 627 | 667));
        assert_eq!(new.len(), 667);
        for (label, body) in old {
            assert_eq!(new.get(label), Some(body));
        }
        let standard = catalog_index(base, "StandardScripts").expect("standard catalog");
        let old_roots = base.data.story_events[standard]["StandardScripts"]["GlobalScriptRoots"]
            .as_array()
            .expect("old roots");
        let new_roots = pack.data.story_events[standard]["StandardScripts"]["GlobalScriptRoots"]
            .as_array()
            .expect("new roots");
        assert_eq!(&new_roots[..old_roots.len()], old_roots);
        let mut expected = old_roots.clone();
        for label in [
            "HeadbuttScript",
            "HeadbuttFromMenuScript",
            "Script_UsedWaterfall",
        ] {
            if !expected.iter().any(|root| root.as_str() == Some(label)) {
                expected.push(Value::String(label.into()));
            }
        }
        assert_eq!(new_roots, &expected);
        let mut report = pack.report.clone();
        report.manifests = base.report.manifests.clone();
        assert_eq!(report, base.report);
        assert_ne!(pack.identity, base.identity);
        assert_eq!(
            build_native_field_scripts_modpack(&pack, source).expect("idempotent repair"),
            pack
        );
    }

    #[test]
    #[ignore = "requires current Minnesota and freshly Rust-exported source packs"]
    fn native_field_repair_executes_source_shake_and_compiles_waterfall_loop() {
        let (base, source) = inputs();
        let pack = build_native_field_scripts_modpack(base, source).expect("native repair");
        let data = &pack.data;
        let map = if data.maps.contains_key("MnR19C00R00") {
            "MnR19C00R00"
        } else {
            "Route29"
        };
        let tile = if map == "MnR19C00R00" {
            TilePosition::new(32, 20)
        } else {
            TilePosition::new(8, 8)
        };
        let mut state = GameState::default();
        let mut session = data
            .overworld_session(map, tile, 0)
            .expect("native source-command component location");
        session.player.facing = Direction::Left;
        let (command, _) = data
            .apply_script_runtime_command(
                &mut state,
                &mut session,
                map,
                map,
                "HeadbuttScript",
                3,
                ScriptRuntimeInputs::default(),
            )
            .expect("actual native ShakeHeadbuttTree command must execute synchronously");
        assert_eq!(command.command, "callasm");
        assert_eq!(command.args, ["ShakeHeadbuttTree"]);
        let label = ".loop@Script_UsedWaterfall";
        let object = data
            .script_object_command("MnR11C00R00", label, 0)
            .expect("retained native Waterfall object slot");
        assert_eq!(object.command, "applymovement");
        assert_eq!(object.object_id.as_deref(), Some("PLAYER"));
        assert_eq!(object.movement.as_deref(), Some(".WaterfallStep"));
        let check = data
            .script_runtime_command("MnR11C00R00", label, 1)
            .expect("retained native Waterfall runtime slot");
        assert_eq!(check.command, "callasm");
        assert_eq!(check.args, [".CheckContinueWaterfall"]);
        let repeat = data
            .script_control_command("MnR11C00R00", label, 2)
            .expect("retained native Waterfall branch slot");
        assert_eq!(repeat.command, "iffalse");
        assert_eq!(repeat.resolved_target_script.as_deref(), Some(label));
        assert_eq!(
            data.script_control_command("MnR11C00R00", label, 3)
                .expect("retained native Waterfall end slot")
                .command,
            "end"
        );
        let movement = data
            .compiled_script_body(".WaterfallStep@Script_UsedWaterfall")
            .expect("native movement body");
        assert_eq!(movement[0]["command"], "turn_waterfall");
    }

    fn rematerialize(pack: &mut CompiledGamePack) {
        pack.data
            .materialize_global_scripts()
            .expect("valid schema source mutation");
        refresh_identity(pack).expect("valid mutated identity");
        verify_compiled_game_pack_for_runtime(pack)
            .expect("mutation must pass pack schema before focused guard");
    }

    #[test]
    #[ignore = "requires current Minnesota and freshly Rust-exported source packs"]
    fn native_field_repair_rejects_changed_overlapping_source() {
        let (base, source) = inputs();
        let mut changed = source.clone();
        let index = catalog_index(&changed, "OverworldEvents").expect("source catalog");
        changed.data.story_events[index]["OverworldEvents"]["Script_UsedWaterfall"][0]["args"][0] =
            Value::String("UpdateTimePals".into());
        rematerialize(&mut changed);
        let error = build_native_field_scripts_modpack(base, &changed)
            .expect_err("changed native overlap must be rejected");
        assert!(
            error
                .to_string()
                .contains("native source overlap changed: Script_UsedWaterfall"),
            "{error:#}"
        );
    }

    #[test]
    #[ignore = "requires current Minnesota and freshly Rust-exported source packs"]
    fn native_field_repair_rejects_changed_visual_local_tail() {
        let (base, source) = inputs();
        let mut changed = source.clone();
        let index = catalog_index(&changed, "OverworldEvents").expect("source catalog");
        let body = changed.data.story_events[index]["OverworldEvents"][".SpawnLeaf@FlyFunction_FrameTimer"].as_array_mut().expect("actual native local tail");
        let last = body.last_mut().expect("native return");
        assert_eq!(last["command"], "ret");
        last["command"] = Value::String("nop".into());
        rematerialize(&mut changed);
        let error = build_native_field_scripts_modpack(base, &changed)
            .expect_err("changed visual tail must be rejected");
        assert!(
            error
                .to_string()
                .contains("native visual source closure ShakeHeadbuttTree"),
            "{error:#}"
        );
    }
}
