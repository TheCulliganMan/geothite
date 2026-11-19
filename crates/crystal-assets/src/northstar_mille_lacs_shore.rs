//! Fit a small fishing cove and a recoverable Lure Ball into the Mille Lacs picnic trail.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::json;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring::{command, refresh_event_section};
use crate::{
    CompiledGamePack, MapModule, authored_city_expansion_events, authored_town_publication,
};

const PARK: &str = "MnMilleLacs";
const BANK: &str = "mn_headwaters";
const ATLAS: &str = "data/tilesets/mn_headwaters_metatiles.bin";
const FLAG: &str = "EVENT_MN_HIDDEN_MILLE_LACS_PICNIC_LURE";
const PICNIC: &str = "MN_ML_PARK_PICNIC";
const PICKUP: &str = "MnMlPicnicLure";
const RETURN: &str = "MnMlPicnicPacked";
const GREETING: &str = "MnMlPicnicLunchText";
const HINT: &str = "MnMlPicnicSpareBallText";

#[derive(Deserialize)]
struct Recipe {
    tiles: [u8; 16],
    collision: [String; 4],
}

#[derive(Deserialize)]
struct Cove {
    recipes: Vec<Recipe>,
    placements: Vec<([u16; 2], u16, u16)>,
    picnic_pages: Vec<AuthoredDialoguePage>,
    clue_pages: Vec<AuthoredDialoguePage>,
}

/// Adds a sheltered walking and Surf destination south of the existing fishing pier.
///
/// Returns an error when the original park, native shore tiles, or item identity differs.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let cove: Cove = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/shore-watch-v01/picnic-cove.json"
    ))?;
    let mut park = pack.data.maps.get(PARK).context(PARK)?.clone();
    ensure!(
        park.attributes.tileset_name == BANK
            && (park.attributes.width, park.attributes.height) == (24, 20)
            && park
                .objects
                .iter()
                .any(|o| o.script == PICNIC && (o.x, o.y) == (11, 30))
            && park.scripts.get(PICNIC)
                == Some(&json!([command(
                    "jumptextfaceplayer",
                    &["MN_ML_PARK_PICNICText"]
                )]))
            && park
                .events
                .bg_events
                .iter()
                .all(|bg| (bg.x, bg.y) != (18, 34))
            && [PICKUP, RETURN, GREETING, HINT]
                .iter()
                .all(|label| !pack.data.map_scripts.contains_key(*label))
            && pack.data.items.contains_key("LURE_BALL")
            && pack.data.pokemon.contains_key("DARTDUCK"),
        "missing the Mille Lacs wooded picnic area"
    );
    shore(pack, &mut park, &cove)?;
    ensure!(
        kind(pack, &park, [18, 34])? == "FLOOR" && kind(pack, &park, [18, 35])? == "FLOOR",
        "flowers on the point must be searchable on foot"
    );
    authored_city_expansion_events::flags(pack, &[FLAG.to_owned()])?;
    park.events.bg_events.push(BackgroundEvent {
        x: 18,
        y: 34,
        event_type: "BGEVENT_ITEM".into(),
        script: PICKUP.into(),
    });
    park.scripts.insert(
        PICKUP.into(),
        json!([command("hiddenitem", &["LURE_BALL", FLAG])]),
    );
    park.scripts.insert(
        PICNIC.into(),
        json!([
            command("faceplayer", &[]),
            command("checkevent", &[FLAG]),
            command("iftrue", &[RETURN]),
            command("opentext", &[]),
            command("writetext", &[GREETING]),
            command("promptbutton", &[]),
            command("writetext", &[HINT]),
            command("waitbutton", &[]),
            command("closetext", &[]),
            command("end", &[])
        ]),
    );
    park.scripts
        .insert(RETURN.into(), json!([command("jumptext", &[GREETING])]));
    park.scripts.insert(
        GREETING.into(),
        authored_dialogue::compile(&cove.picnic_pages)?,
    );
    park.scripts
        .insert(HINT.into(), authored_dialogue::compile(&cove.clue_pages)?);
    park.script_field_pickups =
        crate::parse_script_field_pickups(PARK, &park.scripts, &park.objects)?;
    park.script_object_commands = crate::parse_script_object_commands(PARK, &park.scripts)?;
    park.script_flag_commands = crate::parse_script_flag_commands(PARK, &park.scripts)?;
    park.script_control_commands = crate::parse_script_control_commands(PARK, &park.scripts)?;
    park.script_text_commands = crate::parse_script_text_commands(PARK, &park.scripts)?;
    park.script_text_bodies = crate::parse_script_text_bodies(PARK, &park.scripts)?;
    park.script_runtime_commands = crate::parse_script_runtime_commands(PARK, &park.scripts)?;
    refresh_event_section(&mut park);
    authored_town_publication::replace(pack, park)
}

fn shore(pack: &mut CompiledGamePack, park: &mut MapModule, cove: &Cove) -> Result<()> {
    let atlas = pack.runtime_files.get_mut(ATLAS).context(ATLAS)?;
    let previous = atlas.len();
    let start = previous / 16;
    let bank = pack.data.tilesets.get_mut(BANK).context(BANK)?;
    ensure!(
        previous.is_multiple_of(16) && start + cove.recipes.len() <= 256,
        "incompatible headwaters metatiles"
    );
    for (i, recipe) in cove.recipes.iter().enumerate() {
        ensure!(
            recipe
                .tiles
                .iter()
                .all(|tile| atlas[..previous].contains(tile))
                && recipe
                    .collision
                    .iter()
                    .all(|s| matches!(s.as_str(), "WATER" | "FLOOR"))
                && bank
                    .collision
                    .insert(format!("{:02x}", start + i), recipe.collision.to_vec())
                    .is_none(),
            "unexpected picnic cove metatile"
        );
        atlas.extend_from_slice(&recipe.tiles);
    }
    pack.runtime_files.insert(
        "data/tilesets/mn_headwaters.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    for &([x, y], old, target) in &cove.placements {
        ensure!(
            (7..=10).contains(&x) && (12..=17).contains(&y),
            "placement outside the picnic cove"
        );
        let index = usize::from(y) * usize::from(park.attributes.width) + usize::from(x);
        let dest = if let Some(offset) = target.checked_sub(1000) {
            ensure!(
                usize::from(offset) < cove.recipes.len(),
                "unknown cove recipe"
            );
            u16::try_from(start)? + offset
        } else {
            ensure!(usize::from(target) < start, "unknown stock metatile");
            target
        };
        ensure!(
            park.blocks.get(index) == Some(&old),
            "unexpected old picnic bank at ({x},{y})"
        );
        park.blocks[index] = dest;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::{Context, Result, ensure};
    use crystal_core::input::GameButton;
    use crystal_core::models::{BALL_POCKET_CAPACITY, MAX_ITEM_STACK};
    use crystal_core::random::ReplayDivider;
    use crystal_core::save;
    use crystal_core::state::GameState;
    use crystal_core::systems::field_items::FieldItemPickupOutcome;
    use crystal_core::systems::map_context::SpawnMemoryUpdate;
    use crystal_core::world::map::{Direction, TilePosition};
    use crystal_core::world::movement::{MovementMode, StepOutcome};
    use crystal_core::world::session::OverworldInteractionTarget;

    use super::{FLAG, PARK, PICKUP, kind};

    #[test]
    #[ignore = "requires a published 'MN_SHORE_WATCH_PACK' archive"]
    fn the_flower_point_accepts_an_owned_canoe_in_both_directions_at_every_bank() -> Result<()> {
        let loaded = crate::read_loaded_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("MN_SHORE_WATCH_PACK").context("set MN_SHORE_WATCH_PACK")?,
        ))?;
        let pack = loaded.pack();
        let data = pack.data();
        let room = &data.maps[PARK];
        let music = pack.audio_manifest.music.keys().cloned().collect();
        let mut crossings = 0;
        for y in 23..36 {
            for x in 14..22 {
                let dry = TilePosition::new(x, y);
                if kind(pack, room, [x.try_into()?, y.try_into()?])? != "FLOOR" {
                    continue;
                }
                for (direction, launch, land) in [
                    (Direction::Right, GameButton::Right, GameButton::Left),
                    (Direction::Left, GameButton::Left, GameButton::Right),
                    (Direction::Down, GameButton::Down, GameButton::Up),
                    (Direction::Up, GameButton::Up, GameButton::Down),
                ] {
                    let wet = dry.moved(direction);
                    if !(14..22).contains(&wet.x)
                        || !(23..36).contains(&wet.y)
                        || kind(pack, room, [wet.x.try_into()?, wet.y.try_into()?])? != "WATER"
                    {
                        continue;
                    }
                    let mut session = data.overworld_session(PARK, dry, 0)?;
                    session.player.facing = direction;
                    let mut state = GameState::default();
                    let mut random = ReplayDivider::new([255; 256]);
                    let stopped = data.apply_overworld_input(
                        &mut state,
                        &mut session,
                        [launch],
                        &music,
                        &mut random,
                    )?;
                    ensure!(
                        matches!(stopped.movement, Some(StepOutcome::Blocked { .. }))
                            && session.player.tile == dry
                            && session.player.mode == MovementMode::Normal,
                        "walking entered the lake at {dry:?} toward {wet:?}"
                    );
                    data.apply_overworld_input(&mut state, &mut session, [], &music, &mut random)?;
                    ensure!(
                        state
                            .bag
                            .add_item(&data.items["CANOE"], 1)
                            .map_err(anyhow::Error::msg)?
                    );
                    let launch = data.apply_overworld_input(
                        &mut state,
                        &mut session,
                        [launch],
                        &music,
                        &mut random,
                    )?;
                    ensure!(
                        matches!(launch.movement, Some(StepOutcome::Moved { .. }))
                            && session.player.tile == wet
                            && data.is_canoe_mounted(&state, PARK, session.player.mode),
                        "the owned canoe failed to launch at {dry:?} toward {wet:?}"
                    );
                    // The first opposite press turns the rider. Give the second press
                    // a fresh edge if the normal controller needs it to step ashore.
                    for _ in 0..2 {
                        data.apply_overworld_input(
                            &mut state,
                            &mut session,
                            [],
                            &music,
                            &mut random,
                        )?;
                        data.apply_overworld_input(
                            &mut state,
                            &mut session,
                            [land],
                            &music,
                            &mut random,
                        )?;
                        if session.player.tile == dry {
                            break;
                        }
                    }
                    ensure!(
                        session.player.tile == dry
                            && session.player.mode == MovementMode::Normal
                            && !data.is_canoe_mounted(&state, PARK, session.player.mode)
                            && state.bag.quantity(&data.items["CANOE"]) == 1,
                        "the rider failed to come ashore at {dry:?} from {wet:?}"
                    );
                    crossings += 1;
                }
            }
        }
        ensure!(
            crossings == 20,
            "expected twenty legal two-way shoreline crossings, found {crossings}"
        );
        Ok(())
    }

    #[test]
    #[ignore = "requires a published 'MN_SHORE_WATCH_PACK' archive"]
    fn the_flower_point_holds_one_findable_lure_ball() -> Result<()> {
        let path = PathBuf::from(
            std::env::var_os("MN_SHORE_WATCH_PACK").context("set MN_SHORE_WATCH_PACK")?,
        );
        let loaded = crate::read_loaded_verified_compiled_game_pack(path)?;
        let pack = loaded.pack();
        let data = pack.data();
        let music = pack.audio_manifest.music.keys().cloned().collect();
        let approach = TilePosition::new(18, 35);
        let target = TilePosition::new(18, 34);
        let (mut state, mut session) =
            data.start_overworld_session_at_runtime_tile(PARK, approach, &music)?;
        session.player.facing = Direction::Up;
        let found = data
            .find_itemfinder_hidden_item(&state, PARK, approach)?
            .context("flowers should register")?;
        ensure!(found.tile == target && found.item_id == "LURE_BALL" && found.event_flag == FLAG);
        let hit = session
            .check_interaction_checked(1)
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .context("search the flowers")?;
        ensure!(
            hit.script == PICKUP
                && hit.target_tile == target
                && matches!(hit.target,
            OverworldInteractionTarget::Background { ref event_type } if event_type == "BGEVENT_ITEM")
        );
        ensure!(data.resolve_overworld_interaction(&state, &hit)?.is_some());
        let ordinary = &data.items["POKE_BALL"];
        for _ in 0..BALL_POCKET_CAPACITY {
            ensure!(
                state
                    .bag
                    .add_item(ordinary, MAX_ITEM_STACK)
                    .map_err(anyhow::Error::msg)?
            );
        }
        let full =
            data.pickup_script_field_item_in_session(&mut state, &mut session, PARK, PICKUP, 0)?;
        ensure!(
            matches!(full, FieldItemPickupOutcome::BagFull { event_flag, .. } if event_flag == FLAG)
        );
        ensure!(
            !state.flags.is_event_flag_set(FLAG)?
                && data.resolve_overworld_interaction(&state, &hit)?.is_some()
                && data
                    .find_itemfinder_hidden_item(&state, PARK, approach)?
                    .is_some()
        );
        ensure!(
            state
                .bag
                .remove_item(ordinary, MAX_ITEM_STACK)
                .map_err(anyhow::Error::msg)?
        );
        let got =
            data.pickup_script_field_item_in_session(&mut state, &mut session, PARK, PICKUP, 0)?;
        ensure!(
            matches!(got, FieldItemPickupOutcome::Collected {item_id, quantity: 1, event_flag, ..} if item_id == "LURE_BALL" && event_flag == FLAG)
        );
        ensure!(
            state.bag.quantity(&data.items["LURE_BALL"]) == 1
                && state.flags.is_event_flag_set(FLAG)?
        );
        data.commit_overworld_snapshot(&mut state, &session, SpawnMemoryUpdate::Preserve);
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let temp =
            std::env::temp_dir().join(format!("geothite-picnic-{}-{stamp}", std::process::id()));
        fs::create_dir(&temp)?;
        let file = temp.join("flowers.crystalsave");
        let restored = (|| -> Result<_> {
            let identity = loaded.save_modpack_identity()?;
            let hash = loaded.pack_content_hash()?;
            save::write_save_game_for_modpack(&file, state, &identity, &hash)?;
            Ok(save::read_save_game_for_modpack(&file, &identity, &hash)?.into_state())
        })();
        fs::remove_dir_all(temp)?;
        let (mut state, mut session) =
            data.resume_overworld_session_from_state(restored?, &music)?;
        let hit = session
            .check_interaction_checked(1)
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .context("face the flowers on Continue")?;
        ensure!(
            hit.player_tile == approach
                && hit.target_tile == target
                && hit.script == PICKUP
                && state.bag.quantity(&data.items["LURE_BALL"]) == 1
                && state.flags.is_event_flag_set(FLAG)?
        );
        let second =
            data.pickup_script_field_item_in_session(&mut state, &mut session, PARK, PICKUP, 0)?;
        ensure!(
            matches!(second, FieldItemPickupOutcome::AlreadyCollected {event_flag, ..} if event_flag == FLAG)
        );
        ensure!(
            data.find_itemfinder_hidden_item(&state, PARK, approach)?
                .is_none()
                && data.resolve_overworld_interaction(&state, &hit)?.is_none()
        );
        Ok(())
    }
}
