//! Export field-move rules from the pinned authored source tables.
use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn replacements(source: &str) -> Result<(Value, Value)> {
    let mut cut = BTreeMap::<String, BTreeMap<u16, Value>>::new();
    let mut whirlpool = BTreeMap::<String, BTreeMap<u16, Value>>::new();
    let mut group = "";
    let mut tileset = String::new();
    for line in source.lines().map(code_part) {
        if line == "CutTreeBlockPointers:" {
            group = "cut";
        }
        if line == "WhirlpoolBlockPointers:" {
            group = "whirlpool";
        }
        if line.starts_with('.') && line.ends_with(':') {
            tileset = line[1..line.len() - 1].to_owned();
        }
        let Some(raw) = line.strip_prefix("db ") else {
            continue;
        };
        let args = source_arguments(raw)?;
        if args == ["-1"] {
            continue;
        }
        ensure!(
            args.len() == 3 && !tileset.is_empty(),
            "invalid field-move replacement row"
        );
        let number = |s: &str| -> Result<u16> {
            Ok(if let Some(s) = s.strip_prefix('$') {
                u16::from_str_radix(s, 16)?
            } else {
                s.parse()?
            })
        };
        let variant = match (group, number(&args[2])?) {
            ("cut", 0) => "tree",
            ("cut", 1) => "grass",
            ("whirlpool", 0) => "whirlpool",
            _ => anyhow::bail!("unknown authored field-move animation"),
        };
        let target = match group {
            "cut" => &mut cut,
            "whirlpool" => &mut whirlpool,
            _ => anyhow::bail!("replacement outside authored table"),
        };
        ensure!(
            target
                .entry(tileset.clone())
                .or_default()
                .insert(
                    number(&args[0])?,
                    json!({"replacement_block_id":number(&args[1])?,"variant":variant})
                )
                .is_none(),
            "duplicate field-move replacement"
        );
    }
    ensure!(
        !cut.is_empty() && !whirlpool.is_empty(),
        "missing field-move tables"
    );
    Ok((serde_json::to_value(cut)?, serde_json::to_value(whirlpool)?))
}

pub(super) fn badge(source: &str, function: &str) -> Result<Value> {
    let section = source
        .split_once(&format!("{function}:"))
        .with_context(|| format!("missing {function}"))?
        .1;
    let flag = section
        .lines()
        .map(code_part)
        .take_while(|line| !line.ends_with(':') || line.starts_with('.'))
        .find_map(|line| line.strip_prefix("ld de, ENGINE_").map(str::to_owned))
        .with_context(|| format!("missing {function} badge check"))?;
    let names = [
        "ZEPHYRBADGE",
        "HIVEBADGE",
        "PLAINBADGE",
        "FOGBADGE",
        "STORMBADGE",
        "MINERALBADGE",
        "GLACIERBADGE",
        "RISINGBADGE",
    ];
    let index = names
        .iter()
        .position(|&name| name == flag)
        .context("unknown field-move badge")?;
    Ok(json!({"region":"johto","index":index}))
}

fn story_key(vendor: &Path, map: &str, script: &str, item: &str) -> Result<Value> {
    let source = std::fs::read_to_string(vendor.join("maps").join(format!("{map}.asm")))?;
    for line in source.lines().map(code_part) {
        let Some(raw) = line.strip_prefix("bg_event ") else {
            continue;
        };
        let args = source_arguments(raw)?;
        if args.len() != 4 || args[3] != script {
            continue;
        }
        let facing = match args[2].as_str() {
            "BGEVENT_READ" => None,
            "BGEVENT_UP" => Some("up"),
            "BGEVENT_DOWN" => Some("down"),
            "BGEVENT_LEFT" => Some("left"),
            "BGEVENT_RIGHT" => Some("right"),
            _ => anyhow::bail!("unsupported key facing"),
        };
        return Ok(
            json!({"item_id":item,"map_name":map,"required_facing":facing,
            "target_tile":{"x":args[0].parse::<i16>()?,"y":args[1].parse::<i16>()?},"target_script":script}),
        );
    }
    anyhow::bail!("missing authored story-key target {map}/{script}")
}

pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let constants: Value = serde_json::from_slice(&std::fs::read(
        data.join("story_event_script_constants.json"),
    )?)?;
    let collisions = |names: &[&str]| -> Result<Vec<u8>> {
        names
            .iter()
            .map(|name| {
                u8::try_from(
                    constants["global"][*name]
                        .as_i64()
                        .with_context(|| format!("missing source collision {name}"))?,
                )
                .context("collision byte range")
            })
            .collect()
    };
    let source = std::fs::read_to_string(vendor.join("engine/events/overworld.asm"))?;
    let (cut, whirlpool) = replacements(&std::fs::read_to_string(
        vendor.join("data/collision/field_move_blocks.asm"),
    )?)?;
    let mut value = json!({
        "cut":{"move_id":"CUT","badge":badge(&source,"CutFunction")?,
            "target_collisions":collisions(&["COLL_CUT_TREE","COLL_CUT_TREE_1A","COLL_TALL_GRASS","COLL_LONG_GRASS","COLL_LONG_GRASS_1C"] )?,"replacements":cut},
        "whirlpool":{"move_id":"WHIRLPOOL","badge":badge(&source,"WhirlpoolFunction")?,
            "target_collisions":collisions(&["COLL_WHIRLPOOL","COLL_WHIRLPOOL_2C"] )?,"replacements":whirlpool},
        "strength":{"move_id":"STRENGTH","badge":badge(&source,"StrengthFunction")?,"engine_flag":"ENGINE_STRENGTH_ACTIVE"},
        "flash":{"move_id":"FLASH","badge":badge(&source,"FlashFunction")?,"engine_flag":"STATUSFLAGS_FLASH"},
        "surf":{"move_id":"SURF","badge":badge(&source,"SurfFunction")?,"target_collisions":[],
            "blocked_collisions":collisions(&["COLL_WHIRLPOOL","COLL_WHIRLPOOL_2C","COLL_WATERFALL","COLL_WATERFALL_RIGHT","COLL_WATERFALL_LEFT","COLL_WATERFALL_UP"] )?},
        "waterfall":{"move_id":"WATERFALL","badge":badge(&source,"WaterfallFunction")?,"blocked_collisions":[],
            "target_collisions":collisions(&["COLL_WATERFALL","COLL_WATERFALL_RIGHT","COLL_WATERFALL_LEFT","COLL_WATERFALL_UP","COLL_CURRENT_DOWN"] )?},
        "fly":{"move_id":"FLY","badge":badge(&source,"FlyFunction")?},
        "headbutt":{"move_id":"HEADBUTT","target_collisions":collisions(&["COLL_HEADBUTT_TREE","COLL_HEADBUTT_TREE_1D"] )?},
        "escape_rope":{"item_id":"ESCAPE_ROPE","escape_rope_mode":"DIG_WARP"},"repel":{},
        "card_key":story_key(&vendor,"RadioTower3F","CardKeySlotScript","CARD_KEY")?,
        "basement_key":story_key(&vendor,"GoldenrodUnderground","BasementDoorScript","BASEMENT_KEY")?
    });
    for (key, id) in [
        ("dig", "DIG"),
        ("teleport", "TELEPORT"),
        ("rock_smash", "ROCK_SMASH"),
        ("sweet_scent", "SWEET_SCENT"),
    ] {
        value[key] = json!({"move_id":id,"target_collisions":[]});
    }
    for (key, id) in [
        ("bicycle", "BICYCLE"),
        ("itemfinder", "ITEMFINDER"),
        ("squirtbottle", "SQUIRTBOTTLE"),
        ("coin_case", "COIN_CASE"),
        ("blue_card", "BLUE_CARD"),
    ] {
        value[key] = json!({"item_id":id});
    }
    value["pokegear"] = Value::Null;
    value["town_map"] = Value::Null;
    let _: crystal_core::systems::field_moves::FieldMoveCatalog =
        serde_json::from_value(value.clone())?;
    let modular = data.join("content-packs/core-modular/field_moves");
    std::fs::create_dir_all(&modular)?;
    let bytes = serde_json::to_vec_pretty(&value)?;
    std::fs::write(data.join("field_moves.json"), &bytes)?;
    std::fs::write(modular.join("rules.json"), bytes)?;
    let effects = std::fs::read_to_string(vendor.join("engine/items/item_effects.asm"))?;
    let attributes = std::fs::read_to_string(vendor.join("data/decorations/attributes.asm"))?;
    let mut boxes = serde_json::Map::new();
    for (item, effect, decoration) in [("NORMAL_BOX", "NormalBoxEffect", "SILVER_TROPHY"),
        ("GORGEOUS_BOX", "GorgeousBoxEffect", "GOLD_TROPHY")] {
        ensure!(effects.contains(&format!("{effect}:\n\tld c, DECOFLAG_{decoration}_DOLL")),
            "authored box decoration changed");
        let flag = attributes.lines().map(code_part).filter_map(|line| line.strip_prefix("decoration ")
            .and_then(|raw| source_arguments(raw).ok()))
            .find(|args| args.len() == 5 && args[1] == decoration)
            .context("missing trophy decoration")?[3].clone();
        boxes.insert(item.into(), json!({"item_id":item,"effect":effect,"decoration_flag":flag}));
    }
    let modular = data.join("content-packs/core-modular/field_box_items");
    std::fs::create_dir_all(&modular)?;
    let bytes = serde_json::to_vec_pretty(&boxes)?;
    std::fs::write(data.join("field_box_items.json"), &bytes)?;
    std::fs::write(modular.join("rules.json"), bytes)?;
    Ok(())
}
