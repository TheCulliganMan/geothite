//! Additive native healing drink for the planned Isle of Pines hospitality stop.
use anyhow::{Context, Result, ensure};
use crystal_core::systems::battle_items::{
    ITEM_EFFECT_BEHAVIOR_RESTORE_HP, active_battle_item_effect_plan, battle_pp_item_effect_plan,
    party_wide_item_effect_plan,
};
use serde_json::json;

use crate::grand_marais_modpack::authoring;
use crate::{
    CompiledGamePack, CompiledMapExtension, MapModule, RuntimeMapMetadata, RuntimeSpawnPoint,
    derive_compiled_game_pack_identity_from_manifest, insert_item,
    verify_compiled_game_pack_for_runtime,
};

/// Manifest for the additive Root Beer item; it does not publish an NPC or shop.
pub const ROOT_BEER_MANIFEST_ID: &str = "minnesota-root-beer-v1";
/// Exact catalog, bag, shop and item-use identifier for the new drink.
pub const ROOT_BEER_ITEM_ID: &str = "ROOT_BEER";
/// Saved only after Dorothy's first free bottle is actually delivered.
pub const ROOT_BEER_FIRST_GIFT_EVENT: &str = "EVENT_MN_ROOT_BEER_FIRST_GIFT";
/// Dedicated standard-mart stock, independent from every existing shop.
pub const ROOT_BEER_MART_ID: &str = "MART_MN_ROOT_BEER";
/// Service metadata and script authoring, separate from the item-only overlay.
pub const ROOT_BEER_SERVICE_MANIFEST_ID: &str = "minnesota-root-beer-lady-service-v1";

/// Whether Dorothy also offers the native nursing-care rest routine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootBeerRestPolicy {
    /// Offer an optional Yes/No rest before the optional shop.
    Offer,
    /// Offer the gift and shop without a party-rest choice.
    Omit,
}

/// Native service material ready for the caller's actual map publication.
pub struct PreparedRootBeerLadyService {
    /// Verified item/mart/event metadata; all existing maps remain unchanged.
    pub pack: CompiledGamePack,
    /// The caller's authored map with its existing Dorothy actor retargeted.
    pub module: MapModule,
    /// Root interaction script attached to that existing actor.
    pub script_name: String,
}

/// Adds ROOT BEER using the supplied pack's native Lemonade healing behavior.
///
/// The new consumable restores at most 60 HP, costs 300, and targets a party
/// Pokémon in both field and battle item menus. `script_name` becomes the new
/// catalog identifier; `effect` remains the existing `RestoreHPEffect`. The core
/// item adapter derives its RESTORE_HP behavior from that native HP payload, so
/// no invented script entry or renamed existing drink is required.
///
/// All existing items, maps, runtime files, audio and unrelated data are retained.
/// Dorothy's first gift, purchases, dialogue, cabin and nursing service are not
/// installed by this item-only overlay.
///
/// # Errors
/// Returns an error if the base pack is invalid, the item or manifest already
/// exists, native Lemonade is missing or has incompatible behavior, or the new
/// item, identity or resulting pack fails production validation.
pub fn build_root_beer_lady_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.data.items.contains_key(ROOT_BEER_ITEM_ID),
        "ROOT_BEER item already exists"
    );
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == ROOT_BEER_MANIFEST_ID),
        "Root Beer overlay already applied"
    );
    let lemonade = base
        .data
        .items
        .get("LEMONADE")
        .context("native Lemonade item")?;
    let plan = active_battle_item_effect_plan(lemonade)
        .context("native Lemonade HP restoration behavior")?;
    ensure!(
        lemonade.script_name == "LEMONADE"
            && lemonade.effect == "RestoreHPEffect"
            && plan.behavior_id == ITEM_EFFECT_BEHAVIOR_RESTORE_HP
            && battle_pp_item_effect_plan(lemonade).is_none()
            && party_wide_item_effect_plan(lemonade).is_none()
            && lemonade.rare_candy_level_gain.is_none()
            && lemonade.tmhm_index.is_none()
            && lemonade.tmhm_move.is_none()
            && lemonade.pocket == "ITEM"
            && lemonade.field_menu == "ITEMMENU_PARTY"
            && lemonade.field_usable
            && lemonade.battle_menu == "ITEMMENU_PARTY"
            && lemonade.battle_usable
            && lemonade.consumable,
        "Lemonade must retain native consumable party HP healing semantics"
    );
    let mut drink = lemonade.clone();
    drink.script_name = ROOT_BEER_ITEM_ID.into();
    drink.name = "ROOT BEER".into();
    drink.description = "Restores #MON\nHP by 60.".into();
    drink.parameter = 60;
    drink.price = 300;
    let mut pack = base.clone();
    insert_item(&mut pack.data.items, drink).context("add Root Beer item")?;
    pack.report.items = pack.data.items.len();
    pack.report.manifests.push(ROOT_BEER_MANIFEST_ID.into());
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

/// Authors Dorothy's native gift, standard shop and optional rest on an existing actor.
///
/// The caller supplies the real authored exterior or room and its exact actor
/// identifier (or current script identifier). This function creates no map,
/// facade, warp, actor, Safari lifecycle or alternative interaction dispatcher.
/// It returns a separate compiled module for the caller to publish through its
/// normal map-extension path. Existing published maps in the returned pack are
/// unchanged until that explicit publication.
///
/// Apply [`build_root_beer_lady_modpack`] first. The first bottle uses ordinary
/// `giveitem`, branches on delivery failure, and sets the saved gift event only
/// on success. Declining or a full bag preserves the offer. Later visits skip
/// the gift and use `pokemart MARTTYPE_STANDARD`; the native quantity/money/bag
/// transaction owns purchases at the item's price of 300. Optional rest uses
/// only `special HealParty`. No command renews Safari steps, balls or admission.
///
/// This prepares parsed native commands. Actual dialogue ownership, menu
/// navigation, first-gift retry, purchases and rest still require controller
/// verification after the caller publishes the actual map.
///
/// # Errors
/// Returns an error for an invalid base, missing/incompatible ROOT_BEER,
/// ambiguous or unsuitable actor, invalid anchor, conflicting script or stock,
/// unavailable native rest routine, parser failure, or invalid resulting pack.
/// Neither input is changed on failure.
pub fn prepare_root_beer_lady_service(
    base: &CompiledGamePack,
    module: &MapModule,
    actor_id: &str,
    rest: RootBeerRestPolicy,
) -> Result<PreparedRootBeerLadyService> {
    verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !actor_id.is_empty()
            && actor_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
        "Dorothy requires an exact authored actor identifier"
    );
    let drink = base
        .data
        .items
        .get(ROOT_BEER_ITEM_ID)
        .context("install ROOT_BEER first")?;
    ensure!(
        drink.script_name == ROOT_BEER_ITEM_ID
            && drink.effect == "RestoreHPEffect"
            && drink.parameter == 60
            && drink.price == 300
            && drink.pocket == "ITEM"
            && drink.field_usable
            && drink.field_menu == "ITEMMENU_PARTY"
            && drink.battle_usable
            && drink.battle_menu == "ITEMMENU_PARTY"
            && drink.consumable,
        "Dorothy requires the native ROOT_BEER item contract"
    );
    let matches = module
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.object_identifier.as_deref() == Some(actor_id) || object.script == actor_id
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    ensure!(
        matches.len() == 1,
        "Dorothy actor is missing or ambiguous on {}",
        module.id
    );
    let actor_index = matches[0];
    let actor = &module.objects[actor_index];
    ensure!(
        actor.sprite == "SPRITE_GRANNY" && actor.object_type == "OBJECTTYPE_SCRIPT",
        "Dorothy must be the authored native Granny script actor"
    );
    ensure!(
        base.runtime_files.contains_key("gfx/sprites/granny.png"),
        "native Granny source sprite is missing"
    );
    let width = module
        .attributes
        .width
        .checked_mul(2)
        .context("Dorothy map width overflow")?;
    let height = module
        .attributes
        .height
        .checked_mul(2)
        .context("Dorothy map height overflow")?;
    ensure!(
        width > 0
            && height > 0
            && module.blocks.len()
                == usize::from(module.attributes.width) * usize::from(module.attributes.height),
        "Dorothy map block extent must match its dimensions"
    );
    ensure!(
        actor.x < width && actor.y < height,
        "Dorothy anchor is outside its map"
    );
    ensure!(
        !module.objects.iter().enumerate().any(|(index, other)| {
            index != actor_index && other.x == actor.x && other.y == actor.y
        }) && !module
            .events
            .warps
            .iter()
            .any(|warp| warp.x == actor.x && warp.y == actor.y),
        "Dorothy anchor overlaps another actor or warp"
    );
    let collision = &base
        .data
        .tilesets
        .get(&module.attributes.tileset_name)
        .context("Dorothy map tileset")?
        .collision;
    let kind = |x: u16, y: u16| -> Result<&str> {
        let index = usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2);
        let block = module
            .blocks
            .get(index)
            .context("Dorothy map block extent")?;
        let quadrants = collision
            .get(&format!("{block:02x}"))
            .context("Dorothy anchor collision")?;
        Ok(quadrants[usize::from(y % 2 * 2 + x % 2)].as_str())
    };
    ensure!(
        kind(actor.x, actor.y)? == "FLOOR",
        "Dorothy must stand on native FLOOR"
    );
    let approaches = [
        actor.x.checked_sub(1).map(|x| (x, actor.y)),
        (actor.x + 1 < width).then_some((actor.x + 1, actor.y)),
        actor.y.checked_sub(1).map(|y| (actor.x, y)),
        (actor.y + 1 < height).then_some((actor.x, actor.y + 1)),
    ];
    let mut approachable = false;
    for (x, y) in approaches.into_iter().flatten() {
        if kind(x, y)? == "FLOOR"
            && !module
                .objects
                .iter()
                .any(|other| other.x == x && other.y == y)
            && !module
                .events
                .warps
                .iter()
                .any(|warp| warp.x == x && warp.y == y)
        {
            approachable = true;
        }
    }
    ensure!(
        approachable,
        "Dorothy needs an unoccupied native walking approach"
    );
    if rest == RootBeerRestPolicy::Offer {
        base.data.require_special_routine("HealParty")?;
    }
    let script = format!("{actor_id}RootBeerService");
    ensure!(
        !module
            .scripts
            .keys()
            .any(|label| label.starts_with(&script)),
        "Dorothy service script namespace already exists"
    );
    let mut pack = base.clone();
    if let Some(stock) = pack.data.marts.0.get(ROOT_BEER_MART_ID) {
        ensure!(
            stock == &[ROOT_BEER_ITEM_ID.to_owned()],
            "conflicting Root Beer mart stock"
        );
    } else {
        pack.data
            .marts
            .0
            .insert(ROOT_BEER_MART_ID.into(), vec![ROOT_BEER_ITEM_ID.into()]);
    }
    let constants = &mut pack.data.story_event_script_constants.global;
    if !constants.contains_key(ROOT_BEER_FIRST_GIFT_EVENT) {
        let next = constants
            .iter()
            .filter(|(name, _)| name.starts_with("EVENT_"))
            .map(|(_, value)| *value)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .context("Dorothy event identifier overflow")?;
        constants.insert(ROOT_BEER_FIRST_GIFT_EVENT.into(), next);
    }
    if !pack
        .report
        .manifests
        .iter()
        .any(|id| id == ROOT_BEER_SERVICE_MANIFEST_ID)
    {
        pack.report
            .manifests
            .push(ROOT_BEER_SERVICE_MANIFEST_ID.into());
    }
    let mut authored = module.clone();
    authored.objects[actor_index].script = script.clone();
    let choices = format!("{script}Choices");
    let shop = format!("{script}Shop");
    let bag_full = format!("{script}BagFull");
    let farewell = format!("{script}Farewell");
    let first_text = format!("{script}FirstText");
    let gift_text = format!("{script}GiftText");
    let full_text = format!("{script}FullText");
    let rest_text = format!("{script}RestText");
    let rested_text = format!("{script}RestedText");
    let buy_text = format!("{script}BuyText");
    let goodbye_text = format!("{script}GoodbyeText");
    authored.scripts.insert(
        script.clone(),
        json!([
            authoring::command("faceplayer", &[]),
            authoring::command("opentext", &[]),
            authoring::command("checkevent", &[ROOT_BEER_FIRST_GIFT_EVENT]),
            authoring::command("iftrue", &[&choices]),
            authoring::command("writetext", &[&first_text]),
            authoring::command("yesorno", &[]),
            authoring::command("iffalse", &[&choices]),
            authoring::command("giveitem", &[ROOT_BEER_ITEM_ID, "1"]),
            authoring::command("iffalse", &[&bag_full]),
            authoring::command("setevent", &[ROOT_BEER_FIRST_GIFT_EVENT]),
            authoring::command("writetext", &[&gift_text]),
            authoring::command("waitbutton", &[]),
            authoring::command("sjump", &[&choices])
        ]),
    );
    authored.scripts.insert(
        bag_full,
        json!([
            authoring::command("writetext", &[&full_text]),
            authoring::command("waitbutton", &[]),
            authoring::command("sjump", &[&choices])
        ]),
    );
    authored.scripts.insert(
        choices,
        if rest == RootBeerRestPolicy::Offer {
            json!([
                authoring::command("writetext", &[&rest_text]),
                authoring::command("yesorno", &[]),
                authoring::command("iffalse", &[&shop]),
                authoring::command("special", &["HealParty"]),
                authoring::command("writetext", &[&rested_text]),
                authoring::command("waitbutton", &[]),
                authoring::command("sjump", &[&shop])
            ])
        } else {
            json!([authoring::command("sjump", &[&shop])])
        },
    );
    authored.scripts.insert(
        shop,
        json!([
            authoring::command("writetext", &[&buy_text]),
            authoring::command("yesorno", &[]),
            authoring::command("iffalse", &[&farewell]),
            // End the consumed Yes/No printer before handing ownership to
            // the ordinary mart. Its native entry opens a fresh text surface.
            authoring::command("closetext", &[]),
            authoring::command("opentext", &[]),
            authoring::command("pokemart", &["MARTTYPE_STANDARD", ROOT_BEER_MART_ID]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[])
        ]),
    );
    authored.scripts.insert(
        farewell,
        json!([
            authoring::command("writetext", &[&goodbye_text]),
            authoring::command("waitbutton", &[]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[])
        ]),
    );
    for (label, lines) in [
        (
            first_text,
            vec![
                "Welcome, paddler!",
                "Try my root beer.",
                "The first is free.",
                "Would you like one?",
            ],
        ),
        (gift_text, vec!["Enjoy your bottle!", "Rest your feet."]),
        (
            full_text,
            vec![
                "Your PACK is full.",
                "I'll keep a bottle",
                "for your next stop.",
            ],
        ),
        (rest_text, vec!["May I help your", "tired partners rest?"]),
        (
            rested_text,
            vec!["Your partners are", "ready for the lake."],
        ),
        (buy_text, vec!["More ROOT BEER?", "Bottles cost 300."]),
        (goodbye_text, vec!["Safe paddling!", "Visit again soon."]),
    ] {
        authoring::text(&mut authored, &label, &lines);
    }
    authoring::finish(&pack, &mut authored)?;
    authored.script_runtime_commands =
        crate::parse_script_runtime_commands(&authored.id, &authored.scripts)?;
    authored.script_object_commands =
        crate::parse_script_object_commands(&authored.id, &authored.scripts)?;
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(PreparedRootBeerLadyService {
        pack,
        module: authored,
        script_name: script,
    })
}

/// Publishes precisely the supplied prepared service map as a new native map.
///
/// The caller supplies its actual authored map metadata and an unoccupied FLOOR
/// arrival. No terrain, cabin facade, return warp, regional connection or Safari
/// lifecycle is invented. Existing maps and the complete original recovery
/// spawn table, including HOME, remain unchanged. This map does not become a new
/// recovery destination or a new-game start. The caller must author real access
/// and verify the published service through the production controller.
///
/// # Errors
/// Returns an error if metadata/arrival does not match the prepared map, the
/// map already exists, the temporary insertion cannot be represented, or the
/// resulting native map, identity or pack fails production validation.
pub fn publish_root_beer_lady_service_map(
    prepared: PreparedRootBeerLadyService,
    metadata: RuntimeMapMetadata,
    arrival: [u16; 2],
) -> Result<CompiledGamePack> {
    let PreparedRootBeerLadyService {
        mut pack,
        module,
        script_name,
    } = prepared;
    verify_compiled_game_pack_for_runtime(&pack)?;
    let constant = module
        .attributes
        .map_constant
        .clone()
        .context("service map constant")?;
    let landmark = module
        .attributes
        .location
        .as_deref()
        .context("service map requires an existing native landmark")?;
    ensure!(
        pack.data
            .pokegear_landmarks
            .landmarks
            .iter()
            .any(|entry| entry.constant == landmark),
        "service map landmark must already exist in the supplied pack"
    );
    ensure!(
        metadata.name == module.id
            && metadata.constant == constant
            && metadata.width == module.attributes.width
            && metadata.height == module.attributes.height,
        "service metadata must match the supplied authored map"
    );
    ensure!(
        module
            .objects
            .iter()
            .any(|actor| actor.script == script_name)
            && module.scripts.contains_key(&script_name),
        "prepared Dorothy service actor and script are missing"
    );
    let [x, y] = arrival;
    ensure!(
        u32::from(x) < u32::from(module.attributes.width) * 2
            && u32::from(y) < u32::from(module.attributes.height) * 2
            && !module
                .objects
                .iter()
                .any(|actor| actor.x == x && actor.y == y)
            && !module
                .events
                .warps
                .iter()
                .any(|warp| warp.x == x && warp.y == y),
        "service arrival must be inside the map and off actors/warps"
    );
    ensure!(
        crate::authored_route_modpack::collision(
            &pack,
            &module.blocks,
            module.attributes.width,
            arrival,
            &module.attributes.tileset_name,
        )? == "FLOOR",
        "service arrival must use native FLOOR"
    );
    // The existing extension lowerer materializes map/split/NPC payloads with
    // a temporary spawn. Native Crystal's bounded recovery table has no spare
    // slot; remove this temporary entry before final verification, exactly as
    // the authored-route publisher does. No existing spawn is overwritten.
    let temporary_spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .map(|spawn| spawn.identifier)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("temporary service spawn identifier overflow")?;
    let name = module.id.clone();
    pack.data
        .pokegear_landmarks
        .map_to_landmark
        .insert(name.clone(), landmark.into());
    let group_id = i16::try_from(metadata.group_id).context("service group identifier")?;
    let map_id = i16::try_from(metadata.map_id).context("service map identifier")?;
    let tile_x = i16::try_from(x).context("service arrival x")?;
    let tile_y = i16::try_from(y).context("service arrival y")?;
    let extension = CompiledMapExtension {
        manifest_id: format!("minnesota-root-beer-map-{}", name.to_ascii_lowercase()),
        map_name: name.clone(),
        map_constant: constant.clone(),
        spawn_key: temporary_spawn.to_string(),
        spawn: RuntimeSpawnPoint {
            identifier: temporary_spawn,
            map_constant: constant,
            map_name: name,
            group_id,
            map_id,
            tile_x,
            tile_y,
            group_name: metadata.group_name.clone(),
            metatile_x: tile_x / 2,
            metatile_y: tile_y / 2,
            subtile_x: tile_x % 2,
            subtile_y: tile_y % 2,
        },
        metadata,
        module,
        wild_encounters: None,
        start_new_game_here: false,
    };
    crate::validate_compiled_map_extension(&extension)?;
    crate::insert_compiled_map_extension(&mut pack, extension)?;
    pack.data
        .runtime_spawn_points
        .remove(&temporary_spawn.to_string());
    pack.report.maps = pack.data.maps.len();
    pack.report.reachable_maps.sort();
    pack.report.solvable_maps.sort();
    pack.identity = derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
