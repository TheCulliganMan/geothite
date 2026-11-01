//! Catalog rejection and real-base preservation regressions; no pack publication.
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::MinnesotaPokemonCatalog;
use crate::minnesota_pokemon_pixel_art;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    pokemon: PathBuf,
    native: PathBuf,
    profiles: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("remove owned species-test fixture");
    }
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "geothite-species-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("create owned fixture");
        let fixture = Self {
            pokemon: root.join("pokemon"),
            native: root.join("native"),
            profiles: root.join("profiles.json"),
            root,
        };
        write_json(
            &fixture.pokemon.join("roster.json"),
            &json!({"families":[{"id":"MN_F001","members":[{"id":"MN_001","name":"Newleaf","types":["GRASS"],"evolves_to":null}]}]}),
        );
        write_json(
            &fixture.pokemon.join("sprite-generation.json"),
            &json!({"families":[{"id":"MN_F001","manifest":"sprites/mn-f001/manifest.json"}]}),
        );
        write_json(
            &fixture.pokemon.join("palettes.json"),
            &json!({"pokemon":{"MN_001":{"gbc_rgb555_words":[32767,26319,12517,0]}}}),
        );
        fixture.write_profiles(profile());
        let palette = [32767, 26319, 12517, 0];
        let mut imports = Vec::new();
        let mut source_png = Vec::new();
        for (view, size) in [("front", 56_usize), ("back", 48)] {
            let mut pixels = vec![0_u8; size * size];
            for y in 10..size - 10 {
                for x in 10..size - 10 {
                    pixels[y * size + x] = 1;
                }
            }
            pixels[15 * size + 15] = 3;
            let rgb = minnesota_pokemon_pixel_art::colored(&pixels, palette);
            let png = minnesota_pokemon_pixel_art::png_rgb(size, size, &rgb).expect("fixture PNG");
            if view == "front" {
                source_png = png.clone();
            }
            let directory = fixture.native.join("MN_F001");
            fs::create_dir_all(&directory).expect("native directory");
            fs::write(directory.join(format!("newleaf.{view}.png")), png).expect("native PNG");
            let mut tiles = minnesota_pokemon_pixel_art::bitplanes(&pixels, size, size);
            if view == "back" {
                let mut columns = Vec::new();
                for x in 0..6 {
                    for y in 0..6 {
                        let offset = (y * 6 + x) * 16;
                        columns.extend_from_slice(&tiles[offset..offset + 16]);
                    }
                }
                tiles = columns;
            }
            fs::write(directory.join(format!("newleaf.{view}.2bpp")), tiles).expect("native bits");
            let rows: Vec<String> = pixels
                .chunks_exact(size)
                .map(|row| {
                    row.iter()
                        .map(|index| ['.', '1', '2', '3'][usize::from(*index)])
                        .collect()
                })
                .collect();
            let fit = if view == "front" { [54, 54] } else { [46, 47] };
            write_json(
                &directory.join(format!("newleaf.{view}.json")),
                &json!({"view":view,"size":[size,size],"rows":rows,"palette_rgb555":palette,"source_sha256":hash(&source_png),"outline_profile":"source-contour-v1","import_crop":[0,0,56,56],"import_fit":fit,"white_detail_min_samples":16,"interior_black_min_samples":32,"mirror_horizontal":false}),
            );
            imports.push(json!({"id":"newleaf","species_id":"MN_001","view":view,"source":"sprites/mn-f001/source.png","crop":[0,0,56,56],"fit":fit,"normal_palette_rgb555":palette}));
        }
        let source = fixture.pokemon.join("sprites/mn-f001/source.png");
        fs::create_dir_all(source.parent().expect("source parent")).expect("create source parent");
        fs::write(source, &source_png).expect("source PNG");
        fs::write(fixture.native.join("MN_F001/line.review.png"), &source_png).expect("review PNG");
        write_json(
            &fixture.pokemon.join("sprites/mn-f001/manifest.json"),
            &json!({"family_id":"MN_F001","visual_review":{"status":"reviewed","outline_profile":"source-contour-v1","sheet_sha256":hash(&source_png)},"sprites":imports}),
        );
        fixture
    }

    fn write_profiles(&self, value: Value) {
        write_json(
            &self.profiles,
            &json!({"schema_version":1,"species":[value]}),
        );
    }
    fn load(&self) -> anyhow::Result<MinnesotaPokemonCatalog> {
        MinnesotaPokemonCatalog::load(&self.pokemon, &self.native, &self.profiles)
    }
}

fn profile() -> Value {
    json!({"id":"MN_001","classification":"LEAF","height_inches":35,"weight_tenths_lb":140,"dex_pages":["It gathers rain @ inside its leaves. @ It shares the dew.","Its roots can feel @ the gentle steps @ of its companions."],"base_stats":{"hp":45,"attack":49,"defense":65,"speed":45,"special_attack":49,"special_defense":65},"catch_rate":45,"base_exp":64,"gender_ratio":31,"hatch_cycles":20,"growth_rate":"GROWTH_MEDIUM_SLOW","egg_groups":["EGG_MONSTER","EGG_PLANT"],"level_moves":[[1,"TACKLE"],[1,"GROWL"],[10,"RAZOR_LEAF"]],"tmhm_moves":["CUT"],"egg_moves":["VINE_WHIP"],"evolution":null,"held_items":[null,null],"menu_icon":"ICON_ODDISH","cry":{"cry":"CRY_CHIKORITA","pitch":-16,"length":176},"shiny_rgb5":[[27,20,9],[9,15,6]],"footprint_species":"CHIKORITA"})
}

fn write_json(path: &Path, value: &Value) {
    fs::create_dir_all(path.parent().expect("fixture JSON parent"))
        .expect("fixture JSON directory");
    fs::write(
        path,
        serde_json::to_vec(value).expect("fixture JSON encode"),
    )
    .expect("fixture JSON write");
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn catalog_rejects_missing_duplicate_profiles_and_text_overflow() {
    let fixture = Fixture::new();
    fixture.load().expect("valid reviewed fixture");
    for change in [
        "missing",
        "duplicate",
        "classification",
        "page",
        "footprint",
    ] {
        let mut value = profile();
        match change {
            "missing" => {
                value["id"] = json!("MN_002");
                fixture.write_profiles(value);
            }
            "duplicate" => write_json(
                &fixture.profiles,
                &json!({"schema_version":1,"species":[value.clone(),value]}),
            ),
            "classification" => {
                value["classification"] = json!("TOO LONG CLASS");
                fixture.write_profiles(value);
            }
            "page" => {
                value["dex_pages"][0] = json!("A line far too long to fit @ second @ third");
                fixture.write_profiles(value);
            }
            "footprint" => {
                value
                    .as_object_mut()
                    .expect("profile object")
                    .remove("footprint_species");
                fixture.write_profiles(value);
            }
            _ => unreachable!("known test case"),
        }
        assert!(fixture.load().is_err(), "must reject {change}");
    }
}

#[test]
fn catalog_rejects_stale_review_import_and_bitplanes() {
    for change in ["review", "fit", "bits", "source", "png"] {
        let fixture = Fixture::new();
        let native = fixture.native.join("MN_F001");
        match change {
            "review" => {
                fs::write(native.join("line.review.png"), b"changed")
                    .expect("alter fixture review");
            }
            "fit" => {
                let path = fixture.pokemon.join("sprites/mn-f001/manifest.json");
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&path).expect("read fixture manifest"))
                        .expect("parse fixture manifest");
                value["sprites"][0]["fit"] = json!([40, 40]);
                write_json(&path, &value);
            }
            "bits" => {
                fs::write(native.join("newleaf.back.2bpp"), vec![0; 576])
                    .expect("alter fixture bits");
            }
            "source" => {
                fs::write(
                    fixture.pokemon.join("sprites/mn-f001/source.png"),
                    b"changed",
                )
                .expect("alter fixture source");
            }
            "png" => {
                let pixels = vec![[255, 255, 255]; 56 * 56];
                let png = minnesota_pokemon_pixel_art::png_rgb(56, 56, &pixels).expect("empty PNG");
                fs::write(native.join("newleaf.front.png"), png).expect("alter fixture PNG");
            }
            _ => unreachable!("known test case"),
        }
        assert!(fixture.load().is_err(), "must reject stale {change}");
    }
}

#[test]
fn catalog_accepts_legacy_metadata_with_unchanged_converter_defaults() {
    let fixture = Fixture::new();
    for view in ["front", "back"] {
        let path = fixture.native.join(format!("MN_F001/newleaf.{view}.json"));
        let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("native metadata"))
            .expect("parse native metadata");
        let fields = value.as_object_mut().expect("native metadata object");
        fields.remove("white_detail_min_samples");
        fields.remove("interior_black_min_samples");
        fields.remove("mirror_horizontal");
        write_json(&path, &value);
    }
    fixture
        .load()
        .expect("legacy metadata retains identical converter defaults");
}

#[test]
#[ignore = "requires the user-supplied verified local TUI content pack"]
fn species_overlay_preserves_real_base_and_rejects_unknown_moves() {
    let fixture = Fixture::new();
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .expect("set external CRYSTAL_RENDER_TEST_PACK");
    let base = crate::read_verified_compiled_game_pack(path).expect("verified external base");
    let catalog = fixture.load().expect("authored catalog");
    let pack = catalog
        .build(&base, None)
        .expect("aggregate species extension");
    assert_eq!(pack.data.pokemon.len(), base.data.pokemon.len() + 1);
    assert_eq!(pack.data.pokemon["NEWLEAF"].int_id, 252);
    assert_eq!(pack.data.pokedex_entries["NEWLEAF"].height_digits, 211);
    assert_eq!(pack.compiled_audio, base.compiled_audio);
    assert_eq!(pack.audio_manifest, base.audio_manifest);
    assert_eq!(pack.audio_compression, base.audio_compression);
    for (key, bytes) in &base.runtime_files {
        assert_eq!(
            pack.runtime_files.get(key),
            Some(bytes),
            "original runtime file {key}"
        );
    }
    let mut restored = pack.data.clone();
    restored.pokemon.remove("NEWLEAF");
    restored.learnsets.remove("NEWLEAF");
    restored.evolutions.0.remove("NEWLEAF");
    restored.egg_moves.remove("NEWLEAF");
    restored.menu_icons.remove("NEWLEAF");
    restored.pokedex_entries.remove("NEWLEAF");
    restored.pokemon_cries.remove("NEWLEAF");
    restored.pokemon_frontpic_anim.remove("NEWLEAF");
    restored.pokemon_frontpic_anim.remove("NEWLEAF_IDLE");
    restored
        .capture_rules
        .heavy_ball_modifiers
        .remove("NEWLEAF");
    restored.oak_ratings = base.data.oak_ratings.clone();
    assert_eq!(restored, base.data, "all unrelated game data preserved");
    assert!(
        catalog.build(&pack, None).is_err(),
        "repeated installation rejected"
    );
    let family = catalog
        .build(&base, Some("MN_F001"))
        .expect("family closure");
    assert_eq!(family.data.pokemon, pack.data.pokemon);
    for (field, value) in [
        ("level_moves", json!([[1, "DOES_NOT_EXIST"]])),
        ("tmhm_moves", json!(["TACKLE"])),
        ("menu_icon", json!("ICON_UNKNOWN")),
        ("footprint_species", json!("UNKNOWN")),
    ] {
        let mut author = profile();
        author[field] = value;
        fixture.write_profiles(author);
        let invalid = fixture.load().expect("valid profile shape");
        assert!(
            invalid.build(&base, None).is_err(),
            "unknown reference {field}"
        );
    }
}

#[test]
fn catalog_rejects_conflicting_ids_names_types_and_evolution_targets() {
    for change in ["id", "name", "type", "target"] {
        let fixture = Fixture::new();
        let path = fixture.pokemon.join("roster.json");
        let mut roster: Value =
            serde_json::from_slice(&fs::read(&path).expect("read fixture roster"))
                .expect("parse fixture roster");
        let original = roster["families"][0]["members"][0].clone();
        match change {
            "id" => roster["families"][0]["members"]
                .as_array_mut()
                .expect("members array")
                .push(original),
            "name" => {
                let mut repeated = original;
                repeated["id"] = json!("MN_002");
                roster["families"][0]["members"]
                    .as_array_mut()
                    .expect("members array")
                    .push(repeated);
            }
            "type" => roster["families"][0]["members"][0]["types"] = json!(["SOUND"]),
            "target" => roster["families"][0]["members"][0]["evolves_to"] = json!("MN_999"),
            _ => unreachable!("known test case"),
        }
        write_json(&path, &roster);
        assert!(fixture.load().is_err(), "must reject conflicting {change}");
    }
}

#[test]
fn catalog_accepts_pure_and_dual_fairy_species() {
    for types in [
        json!(["FAIRY"]),
        json!(["GRASS", "FAIRY"]),
        json!(["FAIRY", "FLYING"]),
    ] {
        let fixture = Fixture::new();
        let path = fixture.pokemon.join("roster.json");
        let mut roster: Value =
            serde_json::from_slice(&fs::read(&path).expect("read fixture roster"))
                .expect("parse fixture roster");
        roster["families"][0]["members"][0]["types"] = types;
        write_json(&path, &roster);
        fixture.load().expect("declared Fairy species is accepted");
    }
}
