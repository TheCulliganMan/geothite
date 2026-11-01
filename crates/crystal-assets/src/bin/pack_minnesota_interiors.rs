//! Publish authored room life over a completed local Minnesota pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_minnesota_interior_life_modpack, read_verified_compiled_game_pack};
use serde_json::{Value, json};

const STORE_TEXTS: [(&str, &str); 7] = [
    ("MnBemidjiMart", "MN_BEMIDJI_CUSTOMERText"),
    ("MnBrainerdMart", "MN_BRAINERD_MART_CUSTOMERText"),
    ("MnGrandRapidsMart", "MN_GR_MART_CUSTOMERText"),
    ("MnMankatoMart", "MN_MK_SHOPPERText"),
    ("MnRochesterMart", "MN_RC_SHOPPERText"),
    ("MnStCloudMart", "MN_ST_CLOUD_MART_CUSTOMERText"),
    ("MnWinonaMart", "MN_W_CUSTOMERText"),
];

fn store_dialogue_source(source: &str) -> Result<String> {
    let source: Value = serde_json::from_str(source).context("interior story JSON")?;
    let rooms = source["rooms"].as_array().context("interior rooms")?;
    let mut selected = Vec::new();
    for (map, label) in STORE_TEXTS {
        let matching_rooms = rooms
            .iter()
            .filter(|room| room["map"] == map)
            .collect::<Vec<_>>();
        ensure!(matching_rooms.len() == 1, "expected one room {map}");
        let texts = matching_rooms[0]["texts"]
            .as_array()
            .context("room texts")?;
        let matching_texts = texts
            .iter()
            .filter(|text| text["label"] == label)
            .collect::<Vec<_>>();
        ensure!(matching_texts.len() == 1, "expected one text {map}/{label}");
        selected.push(json!({
            "map": map,
            "purpose": "Short customer conversation in the existing Mart.",
            "texts": [matching_texts[0]],
            "conversations": []
        }));
    }
    Ok(serde_json::to_string(&json!({
        "schema_version": 1,
        "manifest": "minnesota-interior-life-store-dialogue-v01",
        "all_installed_interiors": false,
        "rooms": selected
    }))?)
}

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3 || (paths.len() == 4 && paths[3].as_os_str() == "--store-dialogue"),
        "usage: pack_minnesota_interiors BASE STORIES_JSON OUTPUT [--store-dialogue]"
    );
    ensure!(
        paths[0].canonicalize()? != paths[2].canonicalize().unwrap_or_else(|_| paths[2].clone()),
        "interior output must differ from its completed base"
    );
    let mut source =
        std::fs::read_to_string(&paths[1]).context("read authored interior stories")?;
    if paths.len() == 4 {
        source = store_dialogue_source(&source)?;
    }
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let pack = build_minnesota_interior_life_modpack(&base, &source)?;
    pack.write_preserving_storage(&paths[2])?;
    read_verified_compiled_game_pack(&paths[2]).context("verify published interiors")?;
    println!("published interior stories: {}", paths[2].display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{STORE_TEXTS, store_dialogue_source};
    use crystal_assets::{build_minnesota_interior_life_modpack, read_verified_compiled_game_pack};
    use serde_json::Value;

    #[test]
    fn store_dialogue_selection_uses_only_existing_customer_texts() {
        let source = include_str!("../../../../planning/minnesota/interior-life/stories.json");
        let selected = store_dialogue_source(source).expect("complete store dialogue source");
        let selected: Value = serde_json::from_str(&selected).expect("selected source JSON");
        let rooms = selected["rooms"].as_array().expect("selected rooms");
        assert_eq!(rooms.len(), 7);
        assert_eq!(selected["all_installed_interiors"], false);
        for room in rooms {
            assert_eq!(room["texts"].as_array().expect("texts").len(), 1);
            assert!(room["conversations"].as_array().expect("actors").is_empty());
        }

        let mut missing: Value = serde_json::from_str(source).expect("authored source JSON");
        missing["rooms"]
            .as_array_mut()
            .expect("rooms")
            .retain(|room| room["map"] != "MnWinonaMart");
        assert!(store_dialogue_source(&missing.to_string()).is_err());
    }

    #[test]
    #[ignore = "requires external parent pack in CRYSTAL_STORE_REVIEW_PARENT"]
    fn store_dialogue_overlay_preserves_customer_and_shop_programs() {
        let parent = std::env::var_os("CRYSTAL_STORE_REVIEW_PARENT")
            .expect("explicit external review parent");
        let parent = read_verified_compiled_game_pack(std::path::PathBuf::from(parent))
            .expect("verified parent pack");
        let source = include_str!("../../../../planning/minnesota/interior-life/stories.json");
        let source = store_dialogue_source(source).expect("bounded store source");
        let revised =
            build_minnesota_interior_life_modpack(&parent, &source).expect("bounded store overlay");
        for (map, label) in STORE_TEXTS {
            let before = &parent.data().maps[map];
            let after = &revised.data().maps[map];
            assert_eq!(before.objects, after.objects, "{map} actors");
            assert_eq!(
                before.script_shop_commands, after.script_shop_commands,
                "{map} shop"
            );
            assert_ne!(
                before.script_text_bodies[label], after.script_text_bodies[label],
                "{map} revised text"
            );
            for (name, script) in &before.scripts {
                if name != label {
                    assert_eq!(after.scripts.get(name), Some(script), "{map}/{name}");
                }
            }
        }
        for (map, before) in &parent.data().maps {
            if !STORE_TEXTS.iter().any(|(name, _)| *name == map.as_str()) {
                assert_eq!(&revised.data().maps[map], before, "unrelated map {map}");
            }
        }
        assert_eq!(
            parent.data().marts,
            revised.data().marts,
            "shop inventories"
        );
        assert_eq!(parent.data().audio, revised.data().audio, "audio programs");
    }
}
