//! Explicit incomplete source checkpoint; never used by the production compiler.
use anyhow::{Context, Result, ensure};
use crystal_assets::{
    COMPILED_GAME_PACK_FORMAT_VERSION,
    modpack::{CompiledGamePackIdentity, GameDataSet, ModpackAudioManifest, ModpackCompileReport},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Serialize)]
struct Checkpoint<'a> {
    format_version: u16,
    data: &'a GameDataSet,
    compiled_audio: BTreeMap<String, Vec<u8>>,
    audio_manifest: &'a ModpackAudioManifest,
    audio_compression: Option<String>,
    runtime_files: &'a BTreeMap<String, Vec<u8>>,
    report: &'a ModpackCompileReport,
    identity: CompiledGamePackIdentity,
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().context(
        "usage: pack_source_checkpoint <external-workspace> <output.unvalidated.crystalpack>",
    )?);
    let output = PathBuf::from(args.next().context("missing checkpoint output")?);
    let mut audit = false;
    let mut title_unvalidated = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--audit" => audit = true,
            "--title-unvalidated" => title_unvalidated = Some(PathBuf::from(args.next().context("missing unvalidated title catalog path")?)),
            _ => anyhow::bail!("unexpected argument {argument}"),
        }
    }
    ensure!(
        output
            .to_string_lossy()
            .ends_with(".unvalidated.crystalpack"),
        "checkpoint output must end with .unvalidated.crystalpack"
    );
    let assets = root.join("apps/web/assets");
    let json = |name: &str| -> Result<serde_json::Value> {
        Ok(serde_json::from_slice(&std::fs::read(
            assets.join("data").join(name),
        )?)?)
    };
    let mut data = GameDataSet::default();
    if let Some(path) = title_unvalidated {
        data.runtime_title_screen = serde_json::from_slice(&std::fs::read(path)?)?;
    } else if assets.join("data/runtime_title_screen.json").exists() {
        data.runtime_title_screen = serde_json::from_value(json("runtime_title_screen.json")?)?;
    }
    if assets.join("data/field_encounters.json").exists() {
        data.field_encounters = serde_json::from_value(json("field_encounters.json")?)?;
    }
    if assets.join("data/decorations.json").exists() {
        data.decorations = serde_json::from_value(json("decorations.json")?)?;
    }
    if assets.join("data/special_routines.json").exists() {
        data.special_routines = serde_json::from_value(json("special_routines.json")?)?;
    }
    if assets.join("data/npc_trades.json").exists() {
        data.npc_trades = serde_json::from_value(json("npc_trades.json")?)?;
    }
    if assets.join("data/battle_tower_rules.json").exists() {
        data.battle_tower_rules = Some(serde_json::from_value(json("battle_tower_rules.json")?)?);
    }
    if assets.join("data/odd_egg_definitions.json").exists() {
        data.odd_egg_definitions = serde_json::from_value(json("odd_egg_definitions.json")?)?;
    }
    if assets.join("data/wild_encounters.json").exists() {
        data.wild_encounters = serde_json::from_value(json("wild_encounters.json")?)?;
        data.encounter_slot_tables = serde_json::from_value(json("encounter_slot_tables.json")?)?;
        data.encounter_music_modifiers = serde_json::from_value(json("encounter_music_modifiers.json")?)?;
    }
    if assets.join("data/phone_contacts.json").exists() {
        data.phone_contacts = serde_json::from_value(json("phone_contacts.json")?)?;
        data.permanent_phone_numbers = serde_json::from_value(json("permanent_phone_numbers.json")?)?;
        data.special_phone_calls = serde_json::from_value(json("special_phone_calls.json")?)?;
    }
    if assets.join("data/sprite_palette_defaults.json").exists() {
        data.sprite_palette_defaults = serde_json::from_value(json("sprite_palette_defaults.json")?)?;
        data.pokegear_town_map_palette_map = serde_json::from_value(json("pokegear_town_map_palette_map.json")?)?;
        data.pokegear_landmarks = serde_json::from_value(json("pokegear_landmarks.json")?)?;
    }
    if assets.join("data/battle_animations.json").exists() {
        data.battle_animations = serde_json::from_value(json("battle_animations.json")?)?;
        data.battle_animation_table = serde_json::from_value(json("battle_animation_table.json")?)?;
        data.sprite_anim_bundle = serde_json::to_string(&json("sprite_anim_bundle.json")?)?;
    }
    if assets.join("data/trainers.json").exists() {
        data.trainers = serde_json::from_value(json("trainers.json")?)?;
        data.trainer_class_names = serde_json::from_value(json("trainer_class_names.json")?)?;
    }
    if assets.join("data/asm_text.json").exists() {
        data.asm_text = serde_json::from_value(json("asm_text.json")?)?;
    }
    if assets.join("data/pc_strings.json").exists() {
        data.pc_strings = serde_json::from_value(json("pc_strings.json")?)?;
        data.menu_icons = serde_json::from_value(json("menu_icons.json")?)?;
        data.move_names = serde_json::from_value(json("move_names.json")?)?;
        data.pokedex_entries = serde_json::from_value(json("pokedex_entries.json")?)?;
    }
    if assets.join("data/capture_rules.json").exists() {
        data.capture_rules = serde_json::from_value(json("capture_rules.json")?)?;
    }
    if assets.join("data/happiness_data.json").exists() {
        data.happiness_data = Some(serde_json::from_value(json("happiness_data.json")?)?);
        data.magikarp_lengths = serde_json::from_value(json("magikarp_lengths.json")?)?;
        data.dratini_move_sets = serde_json::from_value(json("dratini_move_sets.json")?)?;
    }
    if assets.join("data/shuckie_gift.json").exists() {
        data.shuckie_gift = Some(serde_json::from_value(json("shuckie_gift.json")?)?);
        data.bug_contest_config = Some(serde_json::from_value(json("bug_contest_config.json")?)?);
        data.oak_ratings = serde_json::from_value(json("oak_ratings.json")?)?;
    }
    if assets.join("data/fishing.json").exists() {
        data.fishing = serde_json::from_value(json("fishing.json")?)?;
    }
    if assets.join("data/currency_constants.json").exists() {
        data.currency_constants = serde_json::from_value(json("currency_constants.json")?)?;
        data.marts = serde_json::from_value(json("marts.json")?)?;
    }
    if assets.join("data/battle_stat_multipliers.json").exists() {
        data.battle_stat_multipliers = serde_json::from_value(json("battle_stat_multipliers.json")?)?;
    }
    if assets.join("data/move_priorities.json").exists() {
        data.move_priorities = serde_json::from_value(json("move_priorities.json")?)?;
    }
    if assets.join("data/type_categories.json").exists() {
        data.type_categories = serde_json::from_value(json("type_categories.json")?)?;
        data.type_effectiveness = serde_json::from_value(json("type_effectiveness.json")?)?;
    }
    if assets.join("data/weather_modifiers.json").exists() {
        data.weather_modifiers = serde_json::from_value(json("weather_modifiers.json")?)?;
    }
    if assets.join("data/capture_wobble_probabilities.json").exists() {
        data.capture_wobble_probabilities = serde_json::from_value(json("capture_wobble_probabilities.json")?)?;
    }
    if assets.join("data/pokemon_cries.json").exists() {
        data.pokemon_cries = serde_json::from_value(json("pokemon_cries.json")?)?;
        data.initialize_events = serde_json::from_value(json("initialize_events.json")?)?;
    }
    if assets.join("data/roaming_pokemon.json").exists() {
        data.roaming_pokemon = serde_json::from_value(json("roaming_pokemon.json")?)?;
    }
    if assets.join("data/playability.json").exists() {
        data.playability = serde_json::from_value(json("playability.json")?)?;
    }
    if assets.join("data/story_event_script_constants.json").exists() { data.story_event_script_constants=serde_json::from_value(json("story_event_script_constants.json")?)?; }
    for value in json("items.json")?
        .as_array()
        .context("source items array")?
    {
        let item: crystal_core::models::Item = serde_json::from_value(value.clone())?;
        ensure!(
            data.items.insert(item.script_name.clone(), item).is_none(),
            "duplicate item"
        );
    }
    data.story_events
        .push(json("story_events/StandardScripts.json")?);
    if assets.join("data/story_events/OverworldEvents.json").exists() {
        data.story_events.push(json("story_events/OverworldEvents.json")?);
        data.phone_scripts.push(json("phone_scripts/Global.json")?);
    }
    data.pokemon_frontpic_anim = serde_json::from_value(json("pokemon_frontpic_anim.json")?)?;
    if assets.join("data/story_event_script_constants.json").exists() {
        data.story_event_script_constants = serde_json::from_value(json("story_event_script_constants.json")?)?;
    }
    if assets.join("data/battle_reward_rules.json").exists() {
        data.battle_reward_rules = serde_json::from_value(json("battle_reward_rules.json")?)?;
        data.step_event_rules = serde_json::from_value(json("step_event_rules.json")?)?;
        data.battle_escape_rules = serde_json::from_value(json("battle_escape_rules.json")?)?;
    }
    if assets.join("data/pokemon.json").exists() {
        data.pokemon = serde_json::from_value(json("pokemon.json")?)?;
    }
    if assets.join("data/moves.json").exists() {
        data.moves = serde_json::from_value(json("moves.json")?)?;
    }
    if assets.join("data/growth_rates.json").exists() {
        data.growth_rates = serde_json::from_value(json("growth_rates.json")?)?;
        data.learnsets = serde_json::from_value(json("learnsets.json")?)?;
        data.level_up_moves = serde_json::from_value(json("level_up_moves.json")?)?;
        data.egg_moves = serde_json::from_value(json("egg_moves.json")?)?;
        data.evolutions = serde_json::from_value(json("evolutions.json")?)?;
    }
    if assets.join("data/buena_password_categories.json").exists() {
        data.buena_password_categories = serde_json::from_value(json("buena_password_categories.json")?)?;
        data.buena_prizes = serde_json::from_value(json("buena_prizes.json")?)?;
        data.kurt_apricorn_recipes = serde_json::from_value(json("kurt_apricorn_recipes.json")?)?;
    }
    if assets.join("data/fruit_trees.json").exists() {
        data.fruit_trees = serde_json::from_value(json("fruit_trees.json")?)?;
        data.flee_mons = serde_json::from_value(json("flee_mons.json")?)?;
    }
    if assets.join("data/battle_anim_bundle.json").exists() {
        data.battle_anim_bundle = serde_json::to_string(&json("battle_anim_bundle.json")?)?;
    }
    if assets.join("data/audio.json").exists() {
        data.audio = serde_json::from_value(json("audio.json")?)?;
        for asset in &mut data.audio {
            if matches!(asset.source, crystal_assets::ModpackAudioSource::Pcm) {
                asset.source = crystal_assets::ModpackAudioSource::Midi;
                asset.path = asset.path.strip_suffix(".pcm").context("source PCM asset path")?.to_owned() + ".mid";
                asset.validate()?;
            }
        }
    }
    if assets.join("data/field_moves.json").exists() {
        data.field_moves = serde_json::from_value(json("field_moves.json")?)?;
        data.field_box_items = serde_json::from_value(json("field_box_items.json")?)?;
    }
    if assets.join("data/map_attributes.json").exists() {
        data.map_attributes = serde_json::from_value(json("map_attributes.json")?)?;
        data.map_blocks = serde_json::from_value(json("map_blocks.json")?)?;
        data.map_dimensions = serde_json::from_value(json("map_dimensions.json")?)?;
        data.map_scripts = serde_json::from_value(json("map_scripts.json")?)?;
        data.runtime_map_metadata = serde_json::from_value(json("runtime_map_metadata.json")?)?;
        if assets.join("data/runtime_spawn_points.json").exists() {
            data.runtime_spawn_points = serde_json::from_value(json("runtime_spawn_points.json")?)?;
        }
        data.npcs = serde_json::from_value(json("npcs.json")?)?;
        if assets.join("data/tilesets.json").exists() { data.tilesets = serde_json::from_value(json("tilesets.json")?)?; }
        let mut failures = Vec::new();
        for name in data.map_attributes.keys() {
            match data.assemble_map_module_from_compiled_payloads(name) {
                Ok(module) => { data.maps.insert(name.clone(),module); },
                Err(error) => { failures.push(format!("{name}: {error:#}")); }
            }
        }
        for failure in failures.iter().take(8) { eprintln!("incomplete map module: {failure}"); }
        eprintln!("assembled {} source map modules; {} need further export",data.maps.len(),failures.len());
    }
    // The serialized checkpoint must contain the same executable standard
    // scripts that the audit sees. Materializing only after writing left
    // jumpstd targets absent from otherwise successfully audited packs.
    data.materialize_global_scripts().context("materialize authored global scripts before checkpoint serialization")?;
    let mut files = BTreeMap::new();
    for &name in crystal_assets::REQUIRED_POKEGEAR_RUNTIME_FILE_KEYS {
        files.insert(name.to_string(), std::fs::read(assets.join(name))?);
    }
    collect_files(&assets,&assets,&mut files)?;
    for resource in &data.runtime_title_screen.program.resources {
        if resource.kind == "graphics" && resource.path.ends_with(".lz") {
            anyhow::ensure!(!resource.path.starts_with('/') && !resource.path.split('/').any(|part| part == ".."), "invalid compressed resource path");
            files.insert(resource.path.clone(), std::fs::read(assets.join(&resource.path))?);
        }
    }
    for &name in crystal_assets::REQUIRED_VENDOR_RUNTIME_FILE_KEYS { files.insert(name.to_string(), std::fs::read(root.join(name))?); }
    let manifest = ModpackAudioManifest::from_assets(&data.audio, &BTreeMap::new())?;
    let report = ModpackCompileReport {
        manifests: vec!["core-modular".into()],
        pokemon: data.pokemon.len(),
        moves: data.moves.len(),
        maps: data.maps.len(),
        items: data.items.len(),
        ..Default::default()
    };
    let version = COMPILED_GAME_PACK_FORMAT_VERSION;
    let mut identity_bytes = Vec::new();
    ciborium::into_writer(
        &(version, &data, &manifest, &files, &report),
        &mut identity_bytes,
    )?;
    let identity = CompiledGamePackIdentity {
        format_version: version,
        runtime_modpack_id: "core-modular".into(),
        pokemon_species: data.pokemon.len(),
        maps: data.maps.len(),
        moves: data.moves.len(),
        content_hash: format!("{:x}", Sha256::digest(identity_bytes)),
        items: data.items.len(),
        music: manifest.music.len(),
        sound_effects: manifest.sound_effects.len(),
        cries: manifest.cries.len(),
    };
    let pack = Checkpoint {
        format_version: version,
        data: &data,
        compiled_audio: BTreeMap::new(),
        audio_manifest: &manifest,
        audio_compression: Some(crystal_assets::modpack::PACK_AUDIO_COMPRESSION_MIDI.to_owned()),
        runtime_files: &files,
        report: &report,
        identity,
    };
    let mut payload = Vec::new();
    ciborium::into_writer(&pack, &mut payload)?;
    let length = u32::try_from(payload.len())?;
    let hash = payload.iter().fold(0x811c9dc5u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x01000193)
    });
    let mut bytes = b"CRYSTALPACK\0".to_vec();
    bytes.extend_from_slice(&version.to_be_bytes());
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(&hash.to_be_bytes());
    bytes.extend_from_slice(&payload);
    std::fs::create_dir_all(output.parent().context("checkpoint parent")?)?;
    let temporary = output.with_extension("crystalpack.tmp");
    std::fs::write(&temporary, &bytes)?;
    std::fs::rename(&temporary, &output)?;
    println!(
        "wrote incomplete checkpoint {} ({} bytes, {} items, {} species, {} moves, {} map modules; full content and audio audit still pending)",
        output.display(),
        bytes.len(),
        data.items.len(),
        data.pokemon.len(),
        data.moves.len(), data.maps.len()
    );
    if audit {
        data.audio = serde_json::from_value(json("audio.json")?)?;
        let root = crystal_assets::AssetRoot::new(root);
        let report = crystal_assets::modpack::ModpackCompiler::new(&root).audit(&data);
        for diagnostic in &report.diagnostics {
            println!("{:?} {} [{}]: {}", diagnostic.severity, diagnostic.subject, diagnostic.code, diagnostic.message);
        }
        ensure!(!report.has_errors(), "checkpoint complete-source audit failed");
    }
    Ok(())
}

fn collect_files(root: &std::path::Path, directory: &std::path::Path, output: &mut BTreeMap<String,Vec<u8>>) -> Result<()> {
    for entry in std::fs::read_dir(directory)? {
        let path=entry?.path();
        if path.is_dir() {collect_files(root,&path,output)?;}
        else if path.extension().and_then(|ext|ext.to_str()).is_some_and(|ext| matches!(ext,"png"|"2bpp"|"1bpp"|"gbcpal"|"pal"|"tilemap"|"rle"|"bin"|"attrmap"|"dimensions")) {
            output.insert(path.strip_prefix(root)?.to_string_lossy().replace('\\',"/"),std::fs::read(&path)?);
        }
    }
    Ok(())
}
