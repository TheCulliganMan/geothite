//! Minnesota's canoe shop and native, direction-specific paddling sprite banks.
use anyhow::{Context, Result, ensure};
use crystal_core::systems::field_moves::CANOE_ITEM;
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

const MANIFEST: &str = "minnesota-canoe-v1";

/// Add a reusable canoe at St. Paul's existing water-travel story handoff.
/// The Capitol Badge gates the gift. Existing fishing, Surf HM and story
/// scripts remain reachable after the canoe introduction and on repeat visits.
///
/// # Errors
/// Rejects invalid packs, duplicate installation, missing native source art,
/// missing shop actors, malformed programs or invalid resulting game data.
pub fn build_canoe_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::minnesota_interior_lineage::require_compact_coast(base)?;
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "canoe already installed"
    );
    ensure!(
        !base.data.items.contains_key(CANOE_ITEM),
        "CANOE identifier occupied"
    );
    let mut pack = base.clone();
    // A passive key item has the native map's no-use/no-toss contract.
    let mut item = pack
        .data
        .items
        .get("S_S_TICKET")
        .context("native passive key item")?
        .clone();
    item.name = "CANOE".into();
    item.script_name = CANOE_ITEM.into();
    item.description = "Walk into water\nto paddle. Step on\nland to portage.".into();
    item.price = 0;
    ensure!(
        item.pocket == "KEY_ITEM" && !item.consumable && !item.field_usable && !item.battle_usable,
        "canoe requires passive key-item semantics"
    );
    crate::insert_item(&mut pack.data.items, item)?;
    pack.report.items = pack.data.items.len();
    crate::canoe_sprites::install(&mut pack)?;
    let mut shop = pack
        .data
        .maps
        .get("MnStPaulAnglerHouse")
        .context("St Paul canoe shop")?
        .clone();
    let actor = shop
        .objects
        .iter_mut()
        .find(|actor| {
            actor.object_identifier.as_deref() == Some("MN_SPX_ANGLERHOUSE_HOST")
                || actor.script.starts_with("MN_SPX_ANGLERHOUSE_HOST")
        })
        .context("existing water-travel outfitter")?;
    let previous = actor.script.clone();
    actor.script = "MnCanoeShopOwner".into();
    let c = authoring::command;
    shop.scripts.insert(
        "MnCanoeShopOwner".into(),
        json!([
            c("faceplayer", &[]),
            c("checkitem", &[CANOE_ITEM]),
            c("iftrue", &["MnCanoeShopRepeat"]),
            c("checkflag", &["ENGINE_FOGBADGE"]),
            c("iffalse", &["MnCanoeShopBefore"]),
            c("opentext", &[]),
            c("writetext", &["MnCanoeShopWelcomeText"]),
            c("waitbutton", &[]),
            c("verbosegiveitem", &[CANOE_ITEM, "1"]),
            c("iffalse", &["MnCanoeShopFull"]),
            c("writetext", &["MnCanoeShopLaunchText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("sjump", &[&previous]),
        ]),
    );
    for (label, text) in [
        ("MnCanoeShopBefore", "MnCanoeShopBeforeText"),
        ("MnCanoeShopRepeat", "MnCanoeShopRepeatText"),
    ] {
        shop.scripts.insert(
            label.into(),
            json!([
                c("opentext", &[]),
                c("writetext", &[text]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("sjump", &[&previous]),
            ]),
        );
    }
    shop.scripts.insert(
        "MnCanoeShopFull".into(),
        json!([
            c("writetext", &["MnCanoeShopFullText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ]),
    );
    for (label, lines) in [
        (
            "MnCanoeShopWelcomeText",
            &[
                "ST PAUL CANOE SHOP",
                "A CAPITOL BADGE!",
                "The river awaits.",
                "I saved this cedar",
                "canoe for you.",
            ][..],
        ),
        (
            "MnCanoeShopLaunchText",
            &[
                "Walk toward water",
                "to launch. Step on",
                "land to portage.",
                "No move or PP is",
                "needed to paddle.",
                "SURF still works",
                "from your party.",
            ][..],
        ),
        (
            "MnCanoeShopBeforeText",
            &[
                "ST PAUL CANOE SHOP",
                "Earn the CAPITOL",
                "BADGE. Then I'll",
                "fit your first",
                "canoe for free.",
            ][..],
        ),
        (
            "MnCanoeShopRepeatText",
            &[
                "Carry your CANOE",
                "to any open shore.",
                "Watch for rocks",
                "and strong falls.",
            ][..],
        ),
        (
            "MnCanoeShopFullText",
            &["Your KEY POCKET", "is full. I'll keep", "your canoe here."][..],
        ),
    ] {
        ensure!(
            lines.iter().all(|line| line.chars().count() <= 18),
            "canoe text overflow: {label}"
        );
        authoring::text(&mut shop, label, lines);
    }
    authoring::speech(
        &pack,
        &mut shop,
        "CedarDisplay",
        "SPRITE_MN_CANOE_RACK",
        (11, 5),
        &["A cedar canoe.", "The ribs are bent", "and tied by hand."],
    )?;
    let display = shop.objects.last_mut().context("canoe display")?;
    display.sprite_has_facings = false;
    display.object_type = "OBJECTTYPE_SCRIPT".into();
    display.pal = 1;
    authoring::finish(&pack, &mut shop)?;
    shop.script_runtime_commands = crate::parse_script_runtime_commands(&shop.id, &shop.scripts)?;
    shop.script_object_commands = crate::parse_script_object_commands(&shop.id, &shop.scripts)?;
    crate::authored_town_publication::replace(&mut pack, shop)?;
    // Reuse the real exterior door. A nearby native sign names the shop
    // without introducing a second facade or changing traversal geometry.
    let mut street = pack
        .data
        .maps
        .get("MnStPaul")
        .context("St Paul exterior")?
        .clone();
    let door = street
        .events
        .warps
        .iter()
        .find(|warp| warp.target_map_constant == "MN_SPX_ANGLER_HOUSE")
        .context("canoe shop exterior entrance")?;
    ensure!(
        street.attributes.tileset_name == "kanto",
        "canoe shop needs native Kanto sign art"
    );
    let sign_tile = (door.x + 3, door.y);
    ensure!(
        sign_tile.0 % 2 == 1 && sign_tile.1 % 2 == 1,
        "Kanto sign must occupy the bottom-right metatile quadrant"
    );
    let sign_block = usize::from(sign_tile.1 / 2) * usize::from(street.attributes.width)
        + usize::from(sign_tile.0 / 2);
    ensure!(
        matches!(street.blocks[sign_block], 0x01 | 0x55),
        "canoe shop sign must replace ordinary lawn or paving"
    );
    street.blocks[sign_block] = 0x79;
    ensure!(
        !street
            .events
            .bg_events
            .iter()
            .any(|event| (event.x, event.y) == sign_tile),
        "canoe shop sign anchor is occupied"
    );
    street
        .events
        .bg_events
        .push(crystal_core::map::BackgroundEvent {
            x: sign_tile.0,
            y: sign_tile.1,
            event_type: "BGEVENT_READ".into(),
            script: "MnCanoeShopSign".into(),
        });
    authoring::text(
        &mut street,
        "MnCanoeShopSignText",
        &["ST PAUL CANOE SHOP", "Cedar boats and", "river advice."],
    );
    street.scripts.insert(
        "MnCanoeShopSign".into(),
        json!([c("jumptext", &["MnCanoeShopSignText"]),]),
    );
    authoring::finish(&pack, &mut street)?;
    crate::authored_town_publication::replace(&mut pack, street)?;
    crate::canoe_story_hints::install(&mut pack)?;
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
