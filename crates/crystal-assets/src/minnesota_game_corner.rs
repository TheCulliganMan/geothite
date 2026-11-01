//! Minneapolis's native coin games and retryable prize counters.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::Value;

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

const ROOM: &str = "MnMinneapolisCornerHome";
const CONSTANT: &str = "MN_MINNEAPOLIS_CORNER_HOME";
const MANIFEST: &str = "minnesota-game-corner-v1";
const WELCOME: &str = "EVENT_MN_GAME_CORNER_WELCOME";

#[derive(Deserialize)]
struct Plan {
    id: String,
    prizes: Vec<Prize>,
}

#[derive(Deserialize)]
struct Prize {
    item: String,
    name: String,
    coins: u16,
}

/// Installs slots, Card Flip, a Coin Case attendant and six item/TM prizes.
///
/// Reuses the supplied pack's native Game Corner art and machine events. The
/// former Corner Home's map identity and exterior door are retained.
/// Its two residents become the attendant and a patron. No
/// creatures, encounters, battles, source art or audio are changed.
///
/// # Errors
/// Rejects duplicate installation, missing native inputs, invalid prize prices,
/// changed facade/return warps, inaccessible services or invalid compiled data.
pub fn build_minnesota_game_corner(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let plan: Plan = serde_json::from_str(source).context("Game Corner contract")?;
    ensure!(
        plan.id == MANIFEST
            && plan.prizes.len() == 6
            && !base.report.manifests.iter().any(|m| m == MANIFEST),
        "six-prize uninstalled Minnesota Game Corner required"
    );
    for prize in &plan.prizes {
        ensure!(
            base.data.items.contains_key(&prize.item)
                && (1..=9999).contains(&prize.coins)
                && prize.name.chars().count() <= 13,
            "invalid Game Corner prize"
        );
    }
    let original = base
        .data
        .maps
        .get(ROOM)
        .context("Minneapolis Corner Home")?;
    let outside = &base.data.maps["MnMinneapolis"];
    ensure!(
        outside
            .events
            .warps
            .iter()
            .any(|w| w.index == 71 && [w.x, w.y] == [56, 51] && w.target_map_constant == CONSTANT)
            && original
                .events
                .warps
                .iter()
                .take(2)
                .all(|w| w.target_map_constant == "MN_MINNEAPOLIS" && w.target_warp_id == 72),
        "retain the current Corner Home facade and outside return"
    );
    let mut pack = base.clone();
    crate::northstar_lair_authoring::flag(&mut pack, WELCOME)?;
    let mut room = authoring::blank(base, "GoldenrodGameCorner", ROOM, CONSTANT)?;
    room.attributes.location = original.attributes.location.clone();
    room.attributes.map_group_constant = original.attributes.map_group_constant.clone();
    room.attributes.fishing_group = original.attributes.fishing_group.clone();
    for at in [(2, 13), (3, 13), (3, 12)] {
        authoring::warp(&mut room, at, "MN_MINNEAPOLIS", 72);
    }
    // Runtime art/events come from the local pack, never another exported catalog.
    room.events.bg_events = base.data.maps["GoldenrodGameCorner"]
        .events
        .bg_events
        .iter()
        .filter(|e| e.script.contains("MachineScript"))
        .cloned()
        .collect();
    for event in &mut room.events.bg_events {
        event.script = if event.script.contains("CardFlip") {
            "MnCornerCards"
        } else {
            "MnCornerSlots"
        }
        .into();
    }
    put(
        &mut room,
        "MnCornerSlots",
        vec![
            c("reanchormap", &[]),
            c("setval", &["FALSE"]),
            c("special", &["SlotMachine"]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        &mut room,
        "MnCornerCards",
        vec![
            c("reanchormap", &[]),
            c("special", &["CardFlip"]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    coin_vendor(&mut room);
    authoring::npc(&pack, &mut room, "MnCornerCoins", "SPRITE_CLERK", 3, 2)?;
    authoring::npc(
        &pack,
        &mut room,
        "MnCornerItems",
        "SPRITE_RECEPTIONIST",
        16,
        2,
    )?;
    authoring::npc(
        &pack,
        &mut room,
        "MnCornerTms",
        "SPRITE_RECEPTIONIST",
        18,
        2,
    )?;
    authoring::npc(&pack, &mut room, "MnCornerWelcome", "SPRITE_TEACHER", 2, 9)?;
    authoring::speech(
        &pack,
        &mut room,
        "Patron",
        "SPRITE_TWIN",
        (10, 10),
        &[
            "Slots on the left!",
            "Card Flip on the right!",
            "Trade coins at the back counters.",
        ],
    )?;
    welcome(&mut room);
    prize_counter(&mut room, "MnCornerItems", &plan.prizes[..3]);
    prize_counter(&mut room, "MnCornerTms", &plan.prizes[3..]);
    for (label, lines) in [
        (
            "MnCornerNeedCase",
            vec!["Ask the attendant", "for a COIN CASE."],
        ),
        (
            "MnCornerPoor",
            vec!["You need more coins", "for that prize."],
        ),
        (
            "MnCornerFull",
            vec![
                "Your PACK is full.",
                "Keep your coins!",
                "Make room and",
                "come back.",
            ],
        ),
        ("MnCornerThanks", vec!["Enjoy your prize!"]),
    ] {
        authoring::text(&mut room, label, &lines);
    }
    for (label, text) in [
        ("MnCornerNoCase", "MnCornerNeedCase"),
        ("MnCornerNotEnough", "MnCornerPoor"),
        ("MnCornerNoRoom", "MnCornerFull"),
        ("MnCornerReceived", "MnCornerThanks"),
    ] {
        put(
            &mut room,
            label,
            vec![
                c("writetext", &[text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[]),
            ],
        );
    }
    put(
        &mut room,
        "MnCornerClose",
        vec![c("closetext", &[]), c("end", &[])],
    );
    finish(&pack, &mut room)?;
    let reachable = crate::authored_town_geometry::reachable(&pack, &room, [3, 12], false)?;
    for at in [
        [3, 4],
        [16, 4],
        [18, 4],
        [3, 9],
        [5, 6],
        [14, 6],
        [2, 13],
        [3, 13],
    ] {
        ensure!(
            reachable.contains(&at),
            "Game Corner approach blocked: {at:?}"
        );
    }
    crate::authored_town_publication::replace(&mut pack, room)?;
    install_sign(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn put(room: &mut MapModule, label: &str, commands: Vec<Value>) {
    room.scripts.insert(label.into(), Value::Array(commands));
}

fn finish(pack: &CompiledGamePack, room: &mut MapModule) -> Result<()> {
    authoring::finish(pack, room)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(&room.id, &room.scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(&room.id, &room.scripts)?;
    let maps = pack
        .data
        .runtime_map_metadata
        .values()
        .map(|m| (m.constant.clone(), m.name.clone()))
        .collect::<BTreeMap<_, _>>();
    room.script_map_commands = crate::parse_script_map_commands(&room.id, &room.scripts, &maps)?;
    room.script_variable_commands = crate::parse_script_variable_commands(&room.id, &room.scripts)?;
    room.script_menu_definitions = crate::parse_script_menu_definitions(&room.id, &room.scripts)?;
    room.script_vertical_menus =
        crate::parse_script_vertical_menus(&room.id, &room.scripts, &room.script_menu_definitions)?;
    Ok(())
}

fn welcome(room: &mut MapModule) {
    authoring::text(
        room,
        "MnCornerWelcomeText",
        &[
            "MINNEAPOLIS",
            "GAME CORNER!",
            "Here is a free",
            "COIN CASE and",
            "20 starter coins.",
        ],
    );
    authoring::text(
        room,
        "MnCornerRulesText",
        &[
            "Buy coins at the",
            "left counter.",
            "Play slots or",
            "Card Flip below.",
            "The two counters",
            "on the right have",
            "prizes for coins!",
        ],
    );
    put(
        room,
        "MnCornerWelcome",
        vec![
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkevent", &[WELCOME]),
            c("iftrue", &["MnCornerRules"]),
            c("checkitem", &["COIN_CASE"]),
            c("iftrue", &["MnCornerStarter"]),
            c("giveitem", &["COIN_CASE", "1"]),
            c("iffalse", &["MnCornerNoRoom"]),
            c("sjump", &["MnCornerStarter"]),
        ],
    );
    put(
        room,
        "MnCornerStarter",
        vec![
            c("givecoins", &["20"]),
            c("setevent", &[WELCOME]),
            c("writetext", &["MnCornerWelcomeText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnCornerRules"]),
        ],
    );
    put(
        room,
        "MnCornerRules",
        vec![
            c("writetext", &["MnCornerRulesText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
}

fn prize_counter(room: &mut MapModule, label: &str, prizes: &[Prize]) {
    let menu = format!("{label}Menu");
    let data = format!("{label}Data");
    let intro = format!("{label}Text");
    authoring::text(room, &intro, &["Exchange coins", "for which prize?"]);
    let mut commands = vec![
        c("faceplayer", &[]),
        c("opentext", &[]),
        c("checkitem", &["COIN_CASE"]),
        c("iffalse", &["MnCornerNoCase"]),
        c("writetext", &[&intro]),
        c("special", &["DisplayCoinCaseBalance"]),
        c("loadmenu", &[&menu]),
        c("verticalmenu", &[]),
        c("closewindow", &[]),
    ];
    for (i, _) in prizes.iter().enumerate() {
        commands.push(c(
            "ifequal",
            &[&(i + 1).to_string(), &format!("{label}Prize{i}")],
        ));
    }
    commands.push(c("sjump", &["MnCornerClose"]));
    put(room, label, commands);
    put(
        room,
        &menu,
        vec![
            c("db", &["MENU_BACKUP_TILES"]),
            c("menu_coords", &["0", "0", "19", "11"]),
            c("dw", &[&data]),
            c("db", &["1"]),
        ],
    );
    let mut entries = vec![c("db", &["STATICMENU_CURSOR"]), c("db", &["4"])];
    for (i, prize) in prizes.iter().enumerate() {
        entries.push(c("db", &[&format!("\"{} {}@\"", prize.name, prize.coins)]));
        let confirm = format!("{label}Confirm{i}");
        authoring::text(
            room,
            &confirm,
            &[
                &format!("{} for", prize.name),
                &format!("{} coins?", prize.coins),
            ],
        );
        put(
            room,
            &format!("{label}Prize{i}"),
            vec![
                c("checkcoins", &[&prize.coins.to_string()]),
                c("ifequal", &["HAVE_LESS", "MnCornerNotEnough"]),
                c("writetext", &[&confirm]),
                c("yesorno", &[]),
                c("iffalse", &["MnCornerClose"]),
                c("giveitem", &[&prize.item, "1"]),
                c("iffalse", &["MnCornerNoRoom"]),
                c("takecoins", &[&prize.coins.to_string()]),
                c("sjump", &["MnCornerReceived"]),
            ],
        );
    }
    entries.push(c("db", &["\"CANCEL@\""]));
    put(room, &data, entries);
}

fn coin_vendor(room: &mut MapModule) {
    authoring::text(
        room,
        "MnCornerBuyCoinsText",
        &["Buy coins for", "which amount?"],
    );
    put(
        room,
        "MnCornerCoins",
        vec![
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkitem", &["COIN_CASE"]),
            c("iffalse", &["MnCornerNoCase"]),
            c("writetext", &["MnCornerBuyCoinsText"]),
            c("loadmenu", &["MnCornerCoinMenu"]),
            c("verticalmenu", &[]),
            c("closewindow", &[]),
            c("ifequal", &["1", "MnCornerBuy50"]),
            c("ifequal", &["2", "MnCornerBuy500"]),
            c("sjump", &["MnCornerClose"]),
        ],
    );
    put(
        room,
        "MnCornerCoinMenu",
        vec![
            c("db", &["MENU_BACKUP_TILES"]),
            c("menu_coords", &["0", "0", "19", "9"]),
            c("dw", &["MnCornerCoinData"]),
            c("db", &["1"]),
        ],
    );
    put(
        room,
        "MnCornerCoinData",
        vec![
            c("db", &["STATICMENU_CURSOR"]),
            c("db", &["3"]),
            c("db", &["\"50 COINS 1000@\""]),
            c("db", &["\"500 COINS 10000@\""]),
            c("db", &["\"CANCEL@\""]),
        ],
    );
    for (label, coins, money, limit) in [
        ("MnCornerBuy50", "50", "1000", "9949"),
        ("MnCornerBuy500", "500", "10000", "9499"),
    ] {
        let confirm = format!("{label}Text");
        authoring::text(
            room,
            &confirm,
            &[&format!("{coins} coins for"), &format!("{money} money?")],
        );
        put(
            room,
            label,
            vec![
                c("checkcoins", &[limit]),
                c("ifequal", &["HAVE_MORE", "MnCornerCoinLimit"]),
                c("checkmoney", &["YOUR_MONEY", money]),
                c("ifequal", &["HAVE_LESS", "MnCornerMoneyShort"]),
                c("writetext", &[&confirm]),
                c("yesorno", &[]),
                c("iffalse", &["MnCornerClose"]),
                c("takemoney", &["YOUR_MONEY", money]),
                c("givecoins", &[coins]),
                c("sjump", &["MnCornerCoinsBought"]),
            ],
        );
    }
    for (label, lines) in [
        (
            "MnCornerCoinLimit",
            vec!["Your COIN CASE", "can hold 9999."],
        ),
        (
            "MnCornerMoneyShort",
            vec!["You need more", "money for coins."],
        ),
        (
            "MnCornerCoinsBought",
            vec!["Here are your coins!", "Enjoy the games!"],
        ),
    ] {
        let text = format!("{label}Text");
        authoring::text(room, &text, &lines);
        put(
            room,
            label,
            vec![
                c("writetext", &[&text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[]),
            ],
        );
    }
}

fn install_sign(pack: &mut CompiledGamePack) -> Result<()> {
    let mut outside = pack.data.maps["MnMinneapolis"].clone();
    ensure!(
        crate::authored_town_geometry::kind(pack, &outside, [55, 51])? == "WALL"
            && crate::authored_town_geometry::kind(pack, &outside, [55, 52])? == "FLOOR"
            && !outside
                .events
                .bg_events
                .iter()
                .any(|e| [e.x, e.y] == [55, 51]),
        "Game Corner sign site occupied"
    );
    ensure!(
        crate::authored_town_geometry::reachable(pack, &outside, [63, 26], false)?
            .contains(&[55, 52]),
        "Game Corner must be reachable on foot from the Minneapolis clinic"
    );
    let mut added = authoring::blank(pack, "MnMinneapolis", "MnMinneapolis", "MN_MINNEAPOLIS")?;
    authoring::text(
        &mut added,
        "MnCornerSignText",
        &[
            "MINNEAPOLIS",
            "GAME CORNER",
            "Slots! Card Flip!",
            "Coin prizes!",
        ],
    );
    put(
        &mut added,
        "MnCornerSign",
        vec![c("jumptext", &["MnCornerSignText"])],
    );
    finish(pack, &mut added)?;
    outside.scripts.extend(added.scripts);
    outside
        .script_text_commands
        .extend(added.script_text_commands);
    outside.script_text_bodies.extend(added.script_text_bodies);
    outside.events.bg_events.push(BackgroundEvent {
        x: 55,
        y: 51,
        event_type: "BGEVENT_READ".into(),
        script: "MnCornerSign".into(),
    });
    authoring::refresh_event_section(&mut outside);
    crate::authored_town_publication::replace(pack, outside)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ROOM, build_minnesota_game_corner};

    #[test]
    #[ignore = "requires CRYSTAL_GAME_CORNER_PARENT with current compact Minnesota"]
    fn minnesota_game_corner_preserves_region_and_rejects_duplicate_installation() {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_GAME_CORNER_PARENT").expect("completed parent"),
        );
        let base = crate::read_verified_compiled_game_pack(path).expect("verified parent");
        let source = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/game-corner.json"),
        )
        .expect("runtime-loaded contract");
        let pack = build_minnesota_game_corner(&base, &source).expect("playable Game Corner");
        assert_eq!(pack.runtime_files, base.runtime_files);
        assert_eq!(pack.compiled_audio, base.compiled_audio);
        assert_eq!(pack.audio_manifest, base.audio_manifest);
        assert_eq!(pack.data.maps.len(), base.data.maps.len());
        for (id, map) in &base.data.maps {
            if ![ROOM, "MnMinneapolis"].contains(&id.as_str()) {
                assert_eq!(
                    serde_json::to_value(&pack.data.maps[id]).expect("map"),
                    serde_json::to_value(map).expect("original map"),
                    "unrelated map {id}"
                );
            }
        }
        let mut before = serde_json::to_value(&base.data).expect("parent data");
        let mut after = serde_json::to_value(&pack.data).expect("child data");
        for key in [
            "maps",
            "map_scripts",
            "map_attributes",
            "map_dimensions",
            "map_blocks",
            "npcs",
            "runtime_map_metadata",
            "story_event_script_constants",
        ] {
            before.as_object_mut().expect("data object").remove(key);
            after.as_object_mut().expect("data object").remove(key);
        }
        assert_eq!(after, before, "unrelated gameplay and roster data");
        let room = &pack.data.maps[ROOM];
        assert!(room.gift_pokemon_scripts.is_empty());
        assert!(
            room.script_runtime_commands
                .iter()
                .any(|c| c.command == "special")
        );
        assert_eq!(room.script_vertical_menus.len(), 3);
        assert!(build_minnesota_game_corner(&pack, &source).is_err());
    }
}
