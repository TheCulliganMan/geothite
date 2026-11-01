//! Three-way composition and rejection of changes hidden behind new hashes.
use serde_json::json;

use super::{ART, CITY, FILES, MANIFEST, compose_duluth_story_terrain};
use crate::CompiledGamePack;

fn read(name: &str) -> CompiledGamePack {
    crate::read_verified_compiled_game_pack(
        std::env::var_os(name).expect("explicit composition fixture"),
    )
    .expect("verified composition fixture")
}

fn identity(pack: &mut CompiledGamePack) {
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )
    .expect("changed fixture identity");
}

fn contract(
    current: &CompiledGamePack,
    before: &CompiledGamePack,
    child: &CompiledGamePack,
) -> String {
    json!({
        "schema_version": 1,
        "current_content_hash": current.identity().expect("current identity").content_hash,
        "accepted_base_content_hash": before.identity().expect("base identity").content_hash,
        "accepted_child_content_hash": child.identity().expect("child identity").content_hash,
    })
    .to_string()
}

#[test]
#[ignore = "requires CRYSTAL_DULUTH_COMPOSITION_CURRENT/BASE/CHILD/SOURCE/WRITTEN packs and contract"]
fn duluth_story_composition_preserves_released_guards_and_rejects_conflicts() {
    let current = read("CRYSTAL_DULUTH_COMPOSITION_CURRENT");
    let before = read("CRYSTAL_DULUTH_COMPOSITION_BASE");
    let accepted = read("CRYSTAL_DULUTH_COMPOSITION_CHILD");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_DULUTH_COMPOSITION_SOURCE").expect("explicit composition source"),
    )
    .expect("composition source JSON");
    let result = compose_duluth_story_terrain(&current, &before, &accepted, &source)
        .expect("reviewed stairs over released guard presentation");
    assert!(
        result == read("CRYSTAL_DULUTH_COMPOSITION_WRITTEN"),
        "published pack equals Rust composition"
    );
    let mut expected = serde_json::to_value(&current.data).expect("current catalogs");
    let actual = serde_json::to_value(&result.data).expect("composed catalogs");
    expected["maps"][CITY]["blocks"] = actual["maps"][CITY]["blocks"].clone();
    let label = current.data.maps[CITY]
        .attributes
        .blocks_label
        .as_ref()
        .expect("block label");
    expected["map_blocks"][label] = actual["map_blocks"][label].clone();
    expected["tilesets"][ART] = actual["tilesets"][ART].clone();
    assert!(
        expected == actual,
        "all story programs, mirrors and unrelated catalogs remain current"
    );
    assert_eq!(
        result.data.maps[CITY].blocks,
        accepted.data.maps[CITY].blocks
    );
    assert_eq!(result.data.tilesets[ART], accepted.data.tilesets[ART]);
    assert!(result.runtime_files.keys().eq(current.runtime_files.keys()));
    for (path, bytes) in &current.runtime_files {
        if path.contains("mn_duluth_expanded") {
            assert_eq!(
                result.runtime_files.get(path),
                accepted.runtime_files.get(path)
            );
        } else {
            assert_eq!(
                result.runtime_files.get(path),
                Some(bytes),
                "unrelated runtime file {path}"
            );
        }
    }
    assert_eq!(result.compiled_audio(), current.compiled_audio());
    let mut report = current.report.clone();
    report.manifests.push(MANIFEST.into());
    assert_eq!(result.report, report);
    assert!(
        compose_duluth_story_terrain(
            &result,
            &before,
            &accepted,
            &contract(&result, &before, &accepted)
        )
        .is_err(),
        "reject repeat installation"
    );
    {
        let mut changed = current.clone();
        let mut city = changed.data.maps[CITY].clone();
        city.scripts
            .get_mut("MnDuluthRangeCheckBadges")
            .expect("released guard")
            .as_array_mut()
            .expect("guard instructions")[4] = json!({"command":"end","args":[]});
        crate::authored_town_publication::replace(&mut changed, city)
            .expect("valid changed program mirrors");
        identity(&mut changed);
        let error = match compose_duluth_story_terrain(
            &changed,
            &before,
            &accepted,
            &contract(&changed, &before, &accepted),
        ) {
            Err(error) => error,
            Ok(_) => panic!("never omit the forced backstep"),
        };
        assert!(
            error.to_string().contains("beyond its text boundary"),
            "{error:#}"
        );
    }
    {
        let mut changed = current.clone();
        *changed
            .runtime_files
            .get_mut(FILES[0])
            .expect("native metatiles")
            .first_mut()
            .expect("first native tile") ^= 1;
        identity(&mut changed);
        let error = match compose_duluth_story_terrain(
            &changed,
            &before,
            &accepted,
            &contract(&changed, &before, &accepted),
        ) {
            Err(error) => error,
            Ok(_) => panic!("never overwrite newer native art"),
        };
        assert!(
            error.to_string().contains("current Duluth art differs"),
            "{error:#}"
        );
    }
    {
        let mut changed = accepted.clone();
        let mut map = changed.data.maps["NewBarkTown"].clone();
        map.blocks[0] ^= 1;
        crate::authored_town_publication::replace(&mut changed, map)
            .expect("changed donor mirrors");
        identity(&mut changed);
        let error = match compose_duluth_story_terrain(
            &current,
            &before,
            &changed,
            &contract(&current, &before, &changed),
        ) {
            Err(error) => error,
            Ok(_) => panic!("reject unrelated donor terrain"),
        };
        assert!(
            error.to_string().contains("unrelated content changes"),
            "{error:#}"
        );
    }
    let mut stale: serde_json::Value = serde_json::from_str(&source).expect("source contract");
    stale["current_content_hash"] = "0".repeat(64).into();
    assert!(
        compose_duluth_story_terrain(&current, &before, &accepted, &stale.to_string()).is_err()
    );
}
