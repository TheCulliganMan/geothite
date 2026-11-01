//! Exact current-world preservation for the two-cell western R10 stair revision.
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_r10_rock_stairs};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const ALIAS: &str = "mn_r10_rockstairs";
const EXTERIORS: [&str; 2] = ["MnR10C00R00", "MnR10C01R00"];
struct Inputs {
    base: CompiledGamePack,
    revised: CompiledGamePack,
    before: String,
    after: String,
    dossier: String,
}
fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("repository");
        let path = std::env::var_os("CRYSTAL_R10_STAIRS_PACK").map(PathBuf::from)
            .unwrap_or_else(||root.join("content-packs/minnesota-northstar-repel-review.crystalpack"))
            .canonicalize().expect("completed current world pack");
        assert_eq!(format!("{:x}",Sha256::digest(std::fs::read(&path).expect("pack bytes"))),
            "31258a1788a986b158d0679e4f3a26c4dd476964311f707a6e09c9640cad48c6");
        let folder = std::env::var_os("CRYSTAL_R10_STAIRS_SOURCE").map(PathBuf::from)
            .unwrap_or_else(||root.join("planning/minnesota/renders/iron-range-descent/rock-stairs-candidate/v01-right-lane"));
        let read=|name|std::fs::read_to_string(folder.join(name)).expect("frozen source input");
        let before=read("baseline-MN_R10.layout.json");let after=read("MN_R10.layout.candidate.json");let dossier=read("baseline-MN_R10.json");
        let base=read_verified_compiled_game_pack(path).expect("strict original world");
        let revised=revise_r10_rock_stairs(&base,&before,&after,&dossier).expect("bounded native stairs");
        Inputs{base,revised,before,after,dossier}
    })
}

#[test]
#[ignore = "requires exact completed31258 world and frozen R10 source candidate"]
fn r10_stairs_revision_whole_data_and_every_other_module_remain_exact() {
    let i = inputs();
    let a = i.revised.data();
    let b = i.base.data();
    assert_eq!(a.maps.len(), 510);
    for (name, module) in &b.maps {
        if !EXTERIORS.contains(&name.as_str()) {
            assert_eq!(&a.maps[name], module, "changed unrelated map {name}");
        }
    }
    let mut normalized = a.clone();
    for name in EXTERIORS {
        let old = &b.maps[name];
        let new = &a.maps[name];
        let mut module = new.clone();
        assert_eq!(new.attributes.tileset_name, ALIAS);
        module.blocks = old.blocks.clone();
        module.attributes.tileset_name = old.attributes.tileset_name.clone();
        assert_eq!(&module, old, "changed nonterrain map record {name}");
        let label = old.attributes.blocks_label.as_ref().expect("block payload");
        normalized.maps.insert(name.into(), old.clone());
        normalized
            .map_attributes
            .insert(name.into(), b.map_attributes[name].clone());
        normalized
            .map_blocks
            .insert(label.clone(), b.map_blocks[label].clone());
    }
    normalized.tilesets.remove(ALIAS);
    assert!(
        normalized == *b,
        "entire remaining GameDataSet must remain exact"
    );
    assert_eq!(a.field_moves, b.field_moves);
    assert_eq!(&a.maps["MnR10Cave1F"], &b.maps["MnR10Cave1F"]);
}

#[test]
#[ignore = "requires exact completed31258 world and frozen R10 source candidate"]
fn r10_stairs_revision_only_new_alias_art_and_two_manifest_ids_change_storage() {
    let i = inputs();
    let old = i.base.runtime_files();
    let new = i.revised.runtime_files();
    for (name, bytes) in old {
        assert_eq!(new.get(name), Some(bytes), "changed source file {name}");
    }
    let mut added = new
        .keys()
        .filter(|name| !old.contains_key(*name))
        .map(String::as_str)
        .collect::<Vec<_>>();
    added.sort_unstable();
    assert_eq!(
        added,
        [
            "data/tilesets/mn_r10_rockstairs.json",
            "data/tilesets/mn_r10_rockstairs_metatiles.bin",
            "data/tilesets/mn_r10_rockstairs_palette_map.json",
            "gfx/tilesets/mn_r10_rockstairs.2bpp",
            "gfx/tilesets/mn_r10_rockstairs.png"
        ]
    );
    let native = &old["data/tilesets/johto_metatiles.bin"];
    let alias = &new["data/tilesets/mn_r10_rockstairs_metatiles.bin"];
    assert_eq!(&alias[..native.len()], native);
    assert_eq!(alias.len(), 129 * 16);
    let ts = &i.revised.data().tilesets[ALIAS];
    let source = &i.base.data().tilesets["johto"];
    for (id, collision) in &source.collision {
        assert_eq!(&ts.collision[id], collision);
    }
    assert_eq!(ts.collision["80"], ["FLOOR", "FLOOR", "WALL", "FLOOR"]);
    // Compare original blocks' exact physical glyph bytes and palette indices,
    // including banked references; allocating a logical slot must not repaint
    // any original address through a shared physical bank.
    let physical = |glyph: u8, palette: u8| {
        if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 127) + 96
        }
    };
    let old_gfx = &old["gfx/tilesets/johto.2bpp"];
    let new_gfx = &new["gfx/tilesets/mn_r10_rockstairs.2bpp"];
    for &glyph in native {
        let before = source.palette_map[usize::from(glyph)];
        let after = ts.palette_map[usize::from(glyph)];
        assert_eq!(before, after);
        let slot = physical(glyph, before) * 16;
        assert_eq!(&old_gfx[slot..slot + 16], &new_gfx[slot..slot + 16]);
    }
    let cave = &old["data/tilesets/cave_metatiles.bin"];
    let cave_gfx = &old["gfx/tilesets/cave.2bpp"];
    let cave_ts = &i.base.data().tilesets["cave"];
    for (&from, &to) in cave[0x12 * 16..0x13 * 16]
        .iter()
        .zip(&alias[128 * 16..129 * 16])
    {
        let from_palette = cave_ts.palette_map[usize::from(from)];
        let to_palette = ts.palette_map[usize::from(to)];
        assert_eq!(from_palette & 7, to_palette & 7);
        let a = physical(from, from_palette) * 16;
        let b = physical(to, to_palette) * 16;
        assert_eq!(
            &cave_gfx[a..a + 16],
            &new_gfx[b..b + 16],
            "stair source glyph changed"
        );
    }
    let (_, _, audio, manifest, compression, files, mut report, mut identity) =
        i.revised.clone().into_parts();
    let (_, _, old_audio, old_manifest, old_compression, old_files, old_report, old_identity) =
        i.base.clone().into_parts();
    assert_eq!(
        &report.manifests[old_report.manifests.len()..],
        [
            "minnesota-r10-rockstairs-art",
            "minnesota-r10-rockstairs-native-stair"
        ]
    );
    report.manifests.truncate(old_report.manifests.len());
    assert_eq!(report, old_report);
    assert!(audio == old_audio && manifest == old_manifest && compression == old_compression);
    let mut files = files;
    for key in added {
        files.remove(key);
    }
    assert_eq!(files, old_files);
    assert_ne!(identity.content_hash, old_identity.content_hash);
    assert_eq!(
        identity.runtime_modpack_id,
        format!(
            "{}+minnesota-r10-rockstairs-art+minnesota-r10-rockstairs-native-stair",
            old_identity.runtime_modpack_id
        ),
        "runtime identity derives exactly the two appended art manifests"
    );
    identity.content_hash = old_identity.content_hash.clone();
    identity.runtime_modpack_id = old_identity.runtime_modpack_id.clone();
    assert_eq!(identity, old_identity);
}

#[test]
#[ignore = "requires exact completed31258 world and frozen R10 source candidate"]
fn r10_stairs_revision_rejects_wrong_recipe_extra_strokes_and_retained_source_drift() {
    let i = inputs();
    for label in [
        "stair orientation",
        "transformed source",
        "other alias",
        "extra stroke",
        "missing shoulder",
        "retained field",
    ] {
        let mut v: Value = serde_json::from_str(&i.after).expect("source candidate");
        match label {
            "stair orientation" => v["derived_blocks"][0]["source_block"] = json!("30"),
            "transformed source" => v["derived_blocks"][0]["tile_row_order"] = json!([3, 2, 1, 0]),
            "other alias" => v["compiled_tileset"] = json!("johto"),
            "extra stroke" => v["layers"]
                .as_array_mut()
                .expect("layers")
                .push(json!({"kind":"point","name":"bad","position":[9,11],"block":"0a"})),
            "missing shoulder" => {
                v["layers"]
                    .as_array_mut()
                    .expect("layers")
                    .last_mut()
                    .expect("shoulder")["block"] = json!("71")
            }
            _ => v["reject_cliff_cuts"] = json!(false),
        }
        assert!(
            revise_r10_rock_stairs(&i.base, &i.before, &v.to_string(), &i.dossier).is_err(),
            "accepted {label}"
        );
    }
    assert!(revise_r10_rock_stairs(&i.revised, &i.before, &i.after, &i.dossier).is_err());
    assert!(
        revise_r10_rock_stairs(&i.base, &format!("{}\n", i.before), &i.after, &i.dossier).is_err()
    );
    assert!(
        revise_r10_rock_stairs(&i.base, &i.before, &i.after, &format!("{}\n", i.dossier)).is_err()
    );
}

#[test]
#[ignore = "requires exact completed31258 world and frozen R10 source candidate"]
fn r10_stairs_revision_rejects_missing_city_seam_or_changed_source_art() {
    let i = inputs();
    for (field, label) in [("seam", "MnR10C00R00"), ("art", "gfx/tilesets/cave.2bpp")] {
        let mut v = serde_json::to_value(&i.base).expect("native mutation fixture");
        if field == "seam" {
            v["data"]["maps"][label]["blocks"][5 * 32] = json!(0x68);
        } else {
            let original = v["runtime_files"][label][0]
                .as_u64()
                .expect("native source glyph byte");
            v["runtime_files"][label][0] = json!(original ^ 255);
        }
        let bad: CompiledGamePack = serde_json::from_value(v).expect("typed preimage mutation");
        let error = revise_r10_rock_stairs(&bad, &i.before, &i.after, &i.dossier)
            .expect_err("reject native drift")
            .to_string();
        assert!(
            error.contains(if field == "seam" {
                "installed blocks"
            } else {
                "certified native art"
            }),
            "specific guard must reject drift: {error}"
        );
    }
}

#[test]
#[ignore = "requires exact completed31258 world and frozen R10 source candidate"]
fn r10_stairs_revision_uncached_assembly_and_strict_written_roundtrip_are_exact() {
    let i = inputs();
    let mut before = i.base.data().clone();
    let mut data = i.revised.data().clone();
    let names = data.maps.keys().cloned().collect::<Vec<_>>();
    before.maps.clear();
    data.maps.clear();
    for name in names {
        let old = before
            .assemble_map_module_from_compiled_payloads(&name)
            .expect("uncached original map");
        let mut new = data
            .assemble_map_module_from_compiled_payloads(&name)
            .expect("uncached revised map");
        if EXTERIORS.contains(&name.as_str()) {
            let cached = &i.revised.data().maps[&name];
            assert_eq!(
                new.attributes, cached.attributes,
                "stale owned attributes {name}"
            );
            assert_eq!(new.blocks, cached.blocks, "stale owned terrain {name}");
            assert_eq!(new.attributes.tileset_name, ALIAS);
            new.attributes.tileset_name = old.attributes.tileset_name.clone();
            new.blocks = old.blocks.clone();
            assert_eq!(new, old, "changed owned uncached nonterrain record {name}");
        } else {
            // Preserve unrelated split/cache drift in the supplied world. This
            // revision owns only the two R10 exteriors; it must not repair or
            // normalize another map merely to make a cached comparison pass.
            assert_eq!(new, old, "changed unrelated uncached module {name}");
        }
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r10-stairs-{}-{nonce}.crystalpack",
        std::process::id()
    ));
    i.revised
        .write_preserving_storage(&path)
        .expect("native write");
    assert_eq!(
        read_verified_compiled_game_pack(&path).expect("strict written readback"),
        i.revised
    );
    std::fs::remove_file(path).expect("remove only own scratch pack");
}
