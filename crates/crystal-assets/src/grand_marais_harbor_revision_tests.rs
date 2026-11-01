//! Native external-pack coverage for the bounded Grand Marais revision.
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    MAP, compile_source, revise_grand_marais_harbor_boundary, validate_geometry,
    validate_preservation,
};
use crate::CompiledGamePack;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("existing repository root")
}

fn sources() -> (String, String) {
    let folder = root().join("planning/minnesota/connections/grand-marais-harbor-closure-v01");
    (
        std::fs::read_to_string(folder.join("closure.contract.json"))
            .expect("accepted harbor contract"),
        std::fs::read_to_string(folder.join("MnGrandMarais.candidate.layout.json"))
            .expect("accepted complete town layout"),
    )
}

fn base() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        crate::read_verified_compiled_game_pack(
            root().join("content-packs/minnesota-state-cohesion-v01.crystalpack"),
        )
        .expect("actual completed 681-map cohesion parent")
    })
}

fn revised() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let (contract, layout) = sources();
        revise_grand_marais_harbor_boundary(base(), &contract, &layout)
            .expect("accepted exact native buoy line")
    })
}

#[test]
#[ignore = "requires the completed external 681-map cohesion pack"]
fn grand_marais_harbor_revision_preserves_all_unowned_content_and_strict_wire() {
    let result = revised();
    let parent = base();
    validate_preservation(parent, result).expect("all 680 other maps/assets exact");
    assert_ne!(result.identity, parent.identity);
    let old = &parent.data.maps[MAP];
    let new = &result.data.maps[MAP];
    assert_eq!(
        new.blocks,
        compile_source(&sources().1).expect("native compiled source")
    );
    assert_eq!(
        old.blocks
            .iter()
            .zip(&new.blocks)
            .filter(|(a, b)| a != b)
            .count(),
        20
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-grand-marais-buoys-{}-{nonce}.crystalpack",
        std::process::id(),
    ));
    result
        .write_preserving_storage(&path)
        .expect("write actual native revision");
    let written = crate::read_verified_compiled_game_pack(&path).expect("strict written readback");
    assert_eq!(&written, result, "entire native wire survives write/read");
    validate_preservation(parent, &written).expect("written uncached geometry/preservation");
    std::fs::remove_file(path).expect("remove only this temporary verification pack");
}

#[test]
#[ignore = "requires the completed external 681-map cohesion pack"]
fn grand_marais_harbor_revision_rejects_source_parent_and_repeated_installation() {
    let (contract, layout) = sources();
    assert!(
        revise_grand_marais_harbor_boundary(base(), &(contract.clone() + " "), &layout).is_err()
    );
    assert!(
        revise_grand_marais_harbor_boundary(base(), &contract, &(layout.clone() + " ")).is_err()
    );
    assert!(revise_grand_marais_harbor_boundary(revised(), &contract, &layout).is_err());
    let mut wrong_parent = base().clone();
    wrong_parent.identity.content_hash = "not-the-accepted-parent".into();
    assert!(revise_grand_marais_harbor_boundary(&wrong_parent, &contract, &layout).is_err());
}

#[test]
#[ignore = "requires the completed external 681-map cohesion pack"]
fn grand_marais_harbor_revision_rejects_actual_water_gap_wrong_cap_and_road_edit() {
    let parent = base();
    let old = &parent.data.maps[MAP];
    for (index, block) in [(370, 0x35), (360, 0x39), (379, 0x39), (0, 0x35)] {
        let mut new = revised().data.maps[MAP].clone();
        new.blocks[index] = block;
        assert!(
            validate_geometry(parent, old, &new).is_err(),
            "invalid native boundary/art at metatile index {index} must fail"
        );
    }
}

#[test]
#[ignore = "requires the completed external 681-map cohesion pack"]
fn grand_marais_harbor_revision_rejects_unrelated_program_actor_art_and_audio_changes() {
    let parent = base();
    for variation in 0..4 {
        let mut changed = revised().clone();
        match variation {
            0 => changed.data.maps.get_mut(MAP).expect("town").objects[0].x += 1,
            1 => {
                changed
                    .data
                    .maps
                    .get_mut(MAP)
                    .expect("town")
                    .scripts
                    .clear();
            }
            2 => {
                changed.runtime_files.remove("gfx/tilesets/johto.png");
            }
            3 => {
                // This external pack stores no decoded PCM in compiled_audio;
                // insert a counterexample so the negative always changes data.
                changed
                    .compiled_audio
                    .insert("unexpected-audio".into(), vec![1]);
            }
            _ => unreachable!("four explicit unrelated mutations"),
        }
        assert!(
            validate_preservation(parent, &changed).is_err(),
            "unrelated mutation {variation} must not be normalized away"
        );
    }
}

#[test]
#[ignore = "requires the completed external 681-map cohesion pack"]
fn grand_marais_harbor_revision_rejects_encoded_geometry_disagreement() {
    let parent = base();
    let mut changed = revised().clone();
    let label = parent.data.maps[MAP]
        .attributes
        .blocks_label
        .as_ref()
        .expect("block label");
    changed
        .data
        .map_blocks
        .insert(label.clone(), parent.data.map_blocks[label].clone());
    assert!(
        validate_preservation(parent, &changed).is_err(),
        "actual uncached native blocks cannot retain the old open water edge"
    );
}
