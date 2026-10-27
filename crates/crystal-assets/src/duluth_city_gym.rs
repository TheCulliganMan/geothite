//! Validate and rebind an existing canonical Gym8 without regenerating it.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::design::Room;
use crate::gym_design::{Gym, Member};
use crate::{CompiledGamePack, MapModule};

fn ordered(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted = map
                .into_iter()
                .map(|(key, value)| (key, ordered(value)))
                .collect::<BTreeMap<_, _>>();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(ordered).collect()),
        scalar => scalar,
    }
}

pub(super) fn validate(pack: &CompiledGamePack, room: &Room, canonical_json: &str) -> Result<()> {
    let reference = room
        .source_reference
        .as_ref()
        .context("Duluth must reference canonical Gym8")?;
    ensure!(
        room.tileset == "mn_gym_8"
            && room.passive_arrival.index == 3
            && room.exits.len() == 2
            && room.exits.iter().zip([1, 2]).all(|(exit, index)| {
                exit.index == index && exit.target == "MnDuluth" && exit.target_warp == 4
            }),
        "canonical Gym8 retains carpets1/2, passive3 and exterior passive4"
    );
    ensure!(
        reference.path == "planning/minnesota/gyms/interiors.json"
            && reference.gym_id == "MnDuluthGym"
            && room.rows.is_empty()
            && room.residents.is_empty()
            && room.signs.is_empty(),
        "no alternative city-owned Gym8 rows or actors"
    );
    let collection: Value =
        serde_json::from_str(canonical_json).context("canonical gym collection")?;
    let selected = collection["gyms"]
        .as_array()
        .context("canonical rooms")?
        .iter()
        .filter(|g| g["id"] == reference.gym_id)
        .collect::<Vec<_>>();
    ensure!(selected.len() == 1, "one canonical Gym8 object required");
    let bytes = serde_json::to_vec(&ordered(selected[0].clone()))?;
    let hash = format!("{:x}", Sha256::digest(bytes));
    ensure!(
        hash == reference.room_sha256,
        "canonical Gym8 object digest differs"
    );
    // Other gyms and collection-file hashes are deliberately outside this scope.
    let gym: Gym = serde_json::from_value(selected[0].clone()).context("typed canonical Gym8")?;
    let native = pack
        .data
        .maps
        .get(&room.id)
        .context("canonical native Gym8 must already be installed; no silent regeneration")?;
    ensure!(
        gym.slot == 8
            && gym.id == room.id
            && gym.constant == room.native_constant
            && room.dimensions_metatiles == gym.dimensions_metatiles
            && room.passive_arrival.tile == gym.arrival_core
            && room.exits.iter().map(|exit| exit.tile).collect::<Vec<_>>() == gym.exit_tiles_core
            && [native.attributes.width, native.attributes.height] == gym.dimensions_metatiles
            && native.attributes.tileset_name == gym.tileset()
            && native.attributes.map_constant.as_deref() == Some(gym.constant.as_str()),
        "canonical native Gym8 identity differs"
    );
    let needed = gym
        .rows
        .iter()
        .flat_map(|r| r.chars())
        .chain(gym.gates.iter().map(|g| g.open_symbol))
        .chain(['#'])
        .collect::<BTreeSet<_>>();
    let symbols = needed
        .into_iter()
        .enumerate()
        .map(|(index, symbol)| Ok((symbol, u16::try_from(index)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    ensure!(
        reference.symbol_blocks
            == symbols
                .iter()
                .map(|(s, b)| (*s, format!("{b:02x}")))
                .collect(),
        "native Gym8 glyph allocation differs"
    );
    ensure!(
        native.blocks
            == gym
                .rows
                .iter()
                .flat_map(|r| r.chars().map(|s| symbols[&s]))
                .collect::<Vec<_>>(),
        "native Gym8 closed terrain differs from canonical source"
    );
    validate_art(pack, &gym, &symbols)?;
    let mut actors = BTreeMap::from([
        (
            format!("{}Guide", gym.id),
            ("SPRITE_GYM_GUIDE", gym.guide_at),
        ),
        (
            format!("{}Leader", gym.id),
            (gym.leader_sprite.as_str(), gym.leader_at),
        ),
    ]);
    for trainer in &gym.trainers {
        actors.insert(trainer.id.clone(), (trainer.sprite.as_str(), trainer.at));
        party(pack, &trainer.id, &trainer.trainer_class, &trainer.party)?;
    }
    ensure!(
        actors.len() == native.objects.len(),
        "canonical Gym8 actor count differs"
    );
    for actor in &native.objects {
        ensure!(
            actors.get(&actor.script) == Some(&(actor.sprite.as_str(), [actor.x, actor.y])),
            "canonical Gym8 actor sprite/anchor differs"
        );
    }
    party(pack, &gym.battle_id(), &gym.trainer_class, &gym.party)?;
    let leader = format!("{}Leader", gym.id);
    let reward = format!("{leader}Reward");
    ensure!(
        native
            .scripts
            .get(&leader)
            .and_then(Value::as_array)
            .is_some_and(
                |body| body.contains(&json!({"command":"setflag","args":[gym.badge_flag]}))
            ),
        "canonical Gym8 badge script differs"
    );
    ensure!(
        native
            .scripts
            .get(&reward)
            .and_then(Value::as_array)
            .is_some_and(
                |body| body.contains(&json!({"command":"giveitem","args":[gym.reward,"1"]}))
            ),
        "canonical Gym8 reward script differs"
    );
    ensure!(
        native.events.warps.len() == 3,
        "canonical Gym8 must retain carpets1/2 and passive3"
    );
    for (warp, (index, tile)) in native.events.warps.iter().zip([
        (1, gym.exit_tiles_core[0]),
        (2, gym.exit_tiles_core[1]),
        (3, gym.arrival_core),
    ]) {
        ensure!(
            warp.index == index && [warp.x, warp.y] == tile,
            "canonical Gym8 warp anchors differ"
        );
    }
    validate_complete_module(pack, &gym, &symbols, native)?;
    Ok(())
}

fn validate_complete_module(
    pack: &CompiledGamePack,
    gym: &Gym,
    symbols: &BTreeMap<char, u16>,
    native: &MapModule,
) -> Result<()> {
    // Reconstruct an expected value solely for comparison. No generated module,
    // trainer or event from this private fixture can reach the returned pack.
    // Requiring the supplied native module first prevents silent repair of an
    // absent or stale canonical installation, including dialogue-only changes.
    let mut comparison = pack.clone();
    let battles = gym
        .trainers
        .iter()
        .map(|t| t.id.clone())
        .chain([gym.battle_id()])
        .collect::<Vec<_>>();
    for id in &battles {
        comparison.data.trainers.trainers.remove(id);
        comparison
            .data
            .story_event_script_constants
            .global
            .remove(id);
    }
    let index = match native.events.warps.first().map(|warp| warp.target_warp_id) {
        Some(3) => 0,
        Some(24) => 7,
        _ => anyhow::bail!("canonical Gym8 has no review-hall return"),
    };
    let mut expected = crate::gym_publication::room(&comparison, gym, symbols, index)?;
    crate::gym_scripts::install(&mut comparison, &mut expected, gym, symbols)?;
    ensure!(
        expected == *native,
        "canonical native Gym8 complete module/program differs; install the captured canonical gym pack first"
    );
    for id in battles {
        ensure!(
            pack.data.trainers.trainers.get(&id) == comparison.data.trainers.trainers.get(&id),
            "canonical native Gym8 complete trainer differs: {id}"
        );
    }
    Ok(())
}

fn party(pack: &CompiledGamePack, id: &str, class: &str, source: &[Member]) -> Result<()> {
    let native = pack
        .data
        .trainers
        .trainers
        .get(id)
        .context("canonical Gym8 native party")?;
    ensure!(
        native.trainer_class == class && native.party.len() == source.len(),
        "canonical Gym8 party/class differs"
    );
    for (mon, source) in native.party.iter().zip(source) {
        ensure!(
            mon.species == source.species && mon.level == source.level,
            "canonical Gym8 member differs"
        );
        if !source.moves.is_empty() {
            ensure!(
                mon.moves
                    .iter()
                    .map(|m| m.name.as_str())
                    .eq(source.moves.iter().map(String::as_str)),
                "canonical Gym8 explicit moves differ"
            );
        }
    }
    Ok(())
}

fn validate_art(pack: &CompiledGamePack, gym: &Gym, symbols: &BTreeMap<char, u16>) -> Result<()> {
    let target = pack
        .data
        .tilesets
        .get(&gym.tileset())
        .context("canonical Gym8 tileset")?;
    let files = pack.runtime_files();
    let target_blocks = files
        .get(&format!("data/tilesets/{}_metatiles.bin", gym.tileset()))
        .context("native Gym8 metatiles")?;
    let target_pixels = files
        .get(&format!("gfx/tilesets/{}.2bpp", gym.tileset()))
        .context("native Gym8 graphics")?;
    for (symbol, index) in symbols {
        let [source, block] = gym
            .art
            .get(symbol)
            .context("canonical Gym8 symbol recipe")?;
        let id = usize::from(u16::from_str_radix(block, 16)?);
        let definition = pack
            .data
            .tilesets
            .get(source)
            .context("canonical Gym8 source tileset")?;
        ensure!(
            target.collision.get(&format!("{index:02x}")) == definition.collision.get(block),
            "canonical Gym8 collision differs"
        );
        let source_blocks = files
            .get(&format!("data/tilesets/{source}_metatiles.bin"))
            .context("source gym metatiles")?;
        let source_pixels = files
            .get(&format!("gfx/tilesets/{source}.2bpp"))
            .context("source gym graphics")?;
        for offset in 0..16 {
            let original = *source_blocks
                .get(id * 16 + offset)
                .context("source metatile bounds")?;
            let actual = *target_blocks
                .get(usize::from(*index) * 16 + offset)
                .context("Gym8 metatile bounds")?;
            let a = definition.palette_map[usize::from(original)];
            let b = target.palette_map[usize::from(actual)];
            let physical = |glyph: u8, palette: u8, length: usize| {
                if palette & 8 == 0 {
                    usize::from(glyph)
                } else {
                    usize::from(glyph & 0x7f) + length / 32
                }
            };
            let a_at = physical(original, a, source_pixels.len()) * 16;
            let b_at = physical(actual, b, target_pixels.len()) * 16;
            ensure!(
                a & 7 == b & 7
                    && source_pixels.get(a_at..a_at + 16) == target_pixels.get(b_at..b_at + 16)
                    && source_pixels.get(a_at..a_at + 16).is_some(),
                "canonical Gym8 native pixels/palette differ"
            );
        }
    }
    Ok(())
}

pub(super) fn rebind(pack: &mut CompiledGamePack) -> Result<()> {
    let mut module = pack
        .data
        .maps
        .get("MnDuluthGym")
        .context("installed canonical Gym8")?
        .clone();
    for warp in &mut module.events.warps {
        warp.target_map_constant = "MN_DULUTH".into();
        warp.target_map = "MN_DULUTH".into();
        warp.target_warp_id = 4;
    }
    let mut count = 0;
    for command in &mut module.map_event_section_commands {
        if command.command == "warp_event" {
            ensure!(command.args.len() == 4, "canonical raw warp shape");
            command.args[2] = "MN_DULUTH".into();
            command.args[3] = "4".into();
            count += 1;
        }
    }
    ensure!(count == 3, "canonical raw warp records differ");
    install_existing(pack, module)
}

fn install_existing(pack: &mut CompiledGamePack, module: MapModule) -> Result<()> {
    crate::insert_compiled_map_split_payloads(
        &mut pack.data.map_scripts,
        &mut pack.data.npcs,
        &module.id,
        module
            .attributes
            .map_scripts_label
            .as_deref()
            .context("Gym8 scripts label")?,
        module
            .attributes
            .map_events_label
            .as_deref()
            .context("Gym8 events label")?,
        &module,
    )?;
    pack.data.maps.insert(module.id.clone(), module);
    Ok(())
}
