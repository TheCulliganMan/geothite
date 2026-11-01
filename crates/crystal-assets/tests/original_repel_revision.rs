//! Source-certified one-field repair against the actual completed geometry pack.
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    CompiledGamePack, parse_source_repel_duration, read_verified_compiled_game_pack,
    repair_original_repel_duration,
};
use serde_json::json;
use sha2::{Digest, Sha256};

const BASE_SHA: &str = "96a10295af34deeb57cd8ebdaf882d8eb80a70e49f63f8bc27ef84980dc1bcc1";
const SOURCE_SHA: &str = "95fe7b6c23c9b3db40259dc23d2ceea6c307349c1b01ae7d66d92cf0ae85e9a5";
struct Inputs {
    base: CompiledGamePack,
    repaired: CompiledGamePack,
    source: String,
}
fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let pack = std::env::var_os("CRYSTAL_ORIGINAL_REPEL_REVISION_PACK").map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/minnesota-northstar-rockcut-review.crystalpack"));
        let source = std::env::var_os("CRYSTAL_ORIGINAL_REPEL_SOURCE").map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/ryanculligan/.local/share/geothite/export-workspace/vendor/pokecrystal/engine/items/item_effects.asm"));
        let pack = pack.canonicalize().expect("completed pack path");
        let bytes = std::fs::read(&pack).expect("completed geometry-stage pack");
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), BASE_SHA, "exact source preimage pack");
        let source = std::fs::read_to_string(source).expect("actual pinned local source, never committed");
        assert_eq!(format!("{:x}", Sha256::digest(source.as_bytes())), SOURCE_SHA, "actual certified native source epoch");
        let base = read_verified_compiled_game_pack(pack).expect("strict geometry-stage base");
        let repaired = repair_original_repel_duration(&base, &source).expect("source-certified repair");
        Inputs { base, repaired, source }
    })
}

#[test]
fn original_repel_exact_source_labels_reject_suffix_missing_duplicate_and_nonliteral() {
    let source =
        "SuperRepelEffect:\nld b, 200\nMaxRepelEffect:\nld b, 250\nRepelEffect:\nld b, $64\n";
    for (label, expected) in [
        ("RepelEffect", 100),
        ("SuperRepelEffect", 200),
        ("MaxRepelEffect", 250),
    ] {
        assert_eq!(
            parse_source_repel_duration(source, label).expect("exact full label"),
            expected
        );
    }
    for bad in [
        "SuperRepelEffect:\nld b, 200",
        "; RepelEffect:\nld b, 100",
        "RepelEffect:\n",
        "RepelEffect:\nOtherEffect:\nld b, 100",
        "RepelEffect:\nld b, 100\nRepelEffect:\nld b, 100",
        "RepelEffect:\nld b, STEPS",
    ] {
        assert!(
            parse_source_repel_duration(bad, "RepelEffect").is_err(),
            "accepted malformed routine"
        );
    }
}

#[test]
#[ignore = "requires exact geometry96a pack and actual pinned external item source"]
fn original_repel_repair_retains_whole_pack_except_duration_and_derived_hash() {
    let i = inputs();
    let (format, mut data, audio, manifest, compression, files, report, mut identity) =
        i.repaired.clone().into_parts();
    let (
        old_format,
        old_data,
        old_audio,
        old_manifest,
        old_compression,
        old_files,
        old_report,
        old_identity,
    ) = i.base.clone().into_parts();
    assert_eq!(data.items["REPEL"].repel_steps, Some(100));
    assert_eq!(data.items["SUPER_REPEL"].repel_steps, Some(200));
    assert_eq!(data.items["MAX_REPEL"].repel_steps, Some(250));
    data.items
        .get_mut("REPEL")
        .expect("original item")
        .repel_steps = Some(200);
    assert!(data == old_data, "entire remaining GameDataSet unchanged");
    assert!(
        format == old_format
            && audio == old_audio
            && manifest == old_manifest
            && compression == old_compression
            && files == old_files
            && report == old_report,
        "all native storage/audio/manifests/reports unchanged"
    );
    assert_ne!(identity.content_hash, old_identity.content_hash);
    identity.content_hash = old_identity.content_hash.clone();
    assert_eq!(identity, old_identity, "only derived content hash changes");
}

#[test]
#[ignore = "requires exact geometry96a pack and actual pinned external item source"]
fn original_repel_repair_reads_actual_three_source_literals_and_rejects_source_drift() {
    let i = inputs();
    for (label, expected) in [
        ("RepelEffect", 100),
        ("SuperRepelEffect", 200),
        ("MaxRepelEffect", 250),
    ] {
        assert_eq!(
            parse_source_repel_duration(&i.source, label).expect("actual pinned source literal"),
            expected
        );
    }
    for (from, to) in [
        ("RepelEffect:", "OtherEffect:"),
        ("ld b, 100", "ld b, 101"),
        ("ld b, 200", "ld b, 201"),
        ("ld b, 250", "ld b, 251"),
    ] {
        let source = i.source.replace(from, to);
        assert_ne!(source, i.source, "meaningful changed-source fixture");
        assert!(
            repair_original_repel_duration(&i.base, &source).is_err(),
            "accepted native source drift"
        );
    }
    assert!(
        repair_original_repel_duration(
            &i.base,
            "SuperRepelEffect:\nld b, 200\nMaxRepelEffect:\nld b, 250"
        )
        .is_err(),
        "missing ordinary label must not borrow a suffix"
    );
}

#[test]
#[ignore = "requires exact geometry96a pack and actual pinned external item source"]
fn original_repel_repair_rejects_effect_duration_missing_and_already_repaired_preimages() {
    let i = inputs();
    for (item, field, value) in [
        ("REPEL", "repel_steps", json!(100)),
        ("REPEL", "effect", json!("SuperRepelEffect")),
        ("SUPER_REPEL", "repel_steps", json!(100)),
        ("MAX_REPEL", "repel_steps", json!(200)),
    ] {
        let mut raw = serde_json::to_value(&i.base).expect("native preimage fixture");
        raw["data"]["items"][item][field] = value;
        let bad: CompiledGamePack = serde_json::from_value(raw).expect("typed changed preimage");
        let error = repair_original_repel_duration(&bad, &i.source)
            .expect_err("reject specific native preimage")
            .to_string();
        assert!(
            error.contains(&format!("native {item} effect/duration preimage differs")),
            "must reject preimage before stale identity: {error}"
        );
    }
    let mut raw = serde_json::to_value(&i.base).expect("native missing-item fixture");
    raw["data"]["items"]
        .as_object_mut()
        .expect("items")
        .remove("REPEL");
    let missing: CompiledGamePack = serde_json::from_value(raw).expect("typed missing item");
    assert!(
        repair_original_repel_duration(&missing, &i.source)
            .expect_err("missing native item")
            .to_string()
            .contains("native REPEL is missing")
    );
    assert!(
        repair_original_repel_duration(&i.repaired, &i.source).is_err(),
        "repair must reject a repeated application"
    );
}

#[test]
#[ignore = "requires exact geometry96a pack and actual pinned external item source"]
fn original_repel_repair_uncached_maps_and_strict_written_roundtrip_are_exact() {
    let i = inputs();
    let mut before = i.base.data().clone();
    let mut after = i.repaired.data().clone();
    let names = before.maps.keys().cloned().collect::<Vec<_>>();
    before.maps.clear();
    after.maps.clear();
    for name in names {
        assert!(
            before
                .assemble_map_module_from_compiled_payloads(&name)
                .expect("uncached before map")
                == after
                    .assemble_map_module_from_compiled_payloads(&name)
                    .expect("uncached after map"),
            "uncached map changed: {name}"
        );
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-repel-{}-{nonce}.crystalpack",
        std::process::id()
    ));
    i.repaired
        .write_preserving_storage(&path)
        .expect("native repair write");
    let written = read_verified_compiled_game_pack(&path).expect("strict actual written readback");
    assert!(written == i.repaired, "entire written repair differs");
    std::fs::remove_file(path).expect("remove only own temporary pack");
}
