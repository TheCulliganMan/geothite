//! Native Cut trees in authored gyms, with explicit sealed course targets.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::authored_route_surface::walking_map_distances;
use crate::gym_design::Gym;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn guide_gift_event(gym: &Gym) -> String {
    format!("EVENT_MN_GYM_{}_CUT_HM_RECEIVED", gym.slot)
}

pub(crate) fn install_guide(module: &mut MapModule, gym: &Gym, guide: &str, guide_text: &str) {
    use crate::grand_marais_modpack::authoring;
    use serde_json::json;

    if gym.cut_gates.is_empty() {
        return;
    }
    let repeat = format!("{guide}Repeat");
    let gift = format!("{guide}CutGiftText");
    let full = format!("{guide}Full");
    let full_text = format!("{full}Text");
    authoring::text(
        module,
        &gift,
        &["Take this HM.", "Teach CUT to a", "partner from PACK."],
    );
    authoring::text(
        module,
        &full_text,
        &["Make room in PACK,", "then talk to me."],
    );
    module.scripts.insert(
        repeat.clone(),
        json!([authoring::command("jumptextfaceplayer", &[guide_text]),]),
    );
    module.scripts.insert(
        full.clone(),
        json!([authoring::command("jumptextfaceplayer", &[&full_text]),]),
    );
    module.scripts.insert(
        guide.into(),
        json!([
            authoring::command("faceplayer", &[]),
            authoring::command("checkevent", &[&guide_gift_event(gym)]),
            authoring::command("iftrue", &[&repeat]),
            authoring::command("checkitem", &["HM_CUT"]),
            authoring::command("iftrue", &[&repeat]),
            authoring::command("giveitem", &["HM_CUT", "1"]),
            authoring::command("iffalse", &[&full]),
            authoring::command("setevent", &[&guide_gift_event(gym)]),
            authoring::command("opentext", &[]),
            authoring::command("writetext", &[&gift]),
            authoring::command("waitbutton", &[]),
            authoring::command("closetext", &[]),
            authoring::command("sjump", &[&repeat]),
        ]),
    );
}

pub(crate) fn validate_design(gym: &Gym) -> Result<()> {
    let [width, height] = gym.dimensions_metatiles;
    let inside = |[x, y]: [u16; 2]| x < width * 2 && y < height * 2;
    let mut ids = BTreeSet::new();
    let mut blocks = BTreeSet::new();
    let mut events = BTreeSet::new();
    for gate in &gym.cut_gates {
        ensure!(
            !gate.id.is_empty()
                && ids.insert(&gate.id)
                && gate
                    .event
                    .starts_with(&format!("EVENT_MN_GYM_{}_CUT_", gym.slot))
                && events.insert(&gate.event)
                && gate.event != guide_gift_event(gym)
                && inside(gate.tree_core)
                && inside(gate.approach_core)
                && blocks.insert([gate.tree_core[0] / 2, gate.tree_core[1] / 2])
                && gym.art.contains_key(&gate.replacement_symbol)
                && !gate.targets_core.is_empty()
                && gate.targets_core.iter().copied().all(inside)
                && gate.tree_core[0].abs_diff(gate.approach_core[0])
                    + gate.tree_core[1].abs_diff(gate.approach_core[1])
                    == 1
                && gate
                    .gated_trainers
                    .iter()
                    .all(|id| gym.trainers.iter().any(|t| &t.id == id)),
            "{}: invalid or duplicate Cut gate {}",
            gym.id,
            gate.id
        );
    }
    ensure!(
        gym.cut_gates.is_empty()
            || (gym.only_victory_gates() && gym.lifts.is_empty() && gym.winds.is_empty()),
        "{}: Cut gardens permit only victory return gates",
        gym.id
    );
    Ok(())
}

pub(crate) fn install_scripts(
    module: &mut MapModule,
    gym: &Gym,
    symbols: &BTreeMap<char, u16>,
    refresh: &str,
    refresh_body: &mut Vec<serde_json::Value>,
) {
    use crate::grand_marais_modpack::authoring::command;
    use crystal_core::map::CoordEvent;
    use serde_json::json;

    for gate in &gym.cut_gates {
        let root = format!("{}{}Pruned", gym.id, gate.id);
        let unchanged = format!("{root}Unchanged");
        refresh_body.push(command("scall", &[&unchanged]));
        let done = format!("{unchanged}Done");
        module.scripts.insert(
            unchanged,
            json!([
                command("checkevent", &[&gate.event]),
                command("iffalse", &[&done]),
                command(
                    "changeblock",
                    &[
                        &(gate.tree_core[0] / 2 * 2).to_string(),
                        &(gate.tree_core[1] / 2 * 2).to_string(),
                        &symbols[&gate.replacement_symbol].to_string(),
                    ]
                ),
                command("end", &[]),
            ]),
        );
        module.scripts.insert(done, json!([command("end", &[])]));
        module.scripts.insert(
            root.clone(),
            json!([
                command("setevent", &[&gate.event]),
                command("scall", &[refresh]),
                command("end", &[]),
            ]),
        );
        module.events.coord_events.push(CoordEvent {
            x: gate.tree_core[0],
            y: gate.tree_core[1],
            scene_id: String::new(),
            script_name: root,
        });
    }
}

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    gym: &Gym,
    symbols: &BTreeMap<char, u16>,
) -> Result<()> {
    if gym.cut_gates.is_empty() {
        return Ok(());
    }
    let mut replacements = BTreeMap::new();
    for gate in &gym.cut_gates {
        let symbol = gym.rows[usize::from(gate.tree_core[1] / 2)]
            .chars()
            .nth(usize::from(gate.tree_core[0] / 2))
            .context("Cut tree source glyph")?;
        let [source, block] = &gym.art[&symbol];
        let source_id = u16::from_str_radix(block, 16).context("Cut source block")?;
        let native = pack
            .data
            .field_moves
            .cut
            .replacements
            .get(source)
            .and_then(|entries| entries.get(&source_id))
            .context("Cut tree requires a supplied native replacement")?;
        let [replacement_source, replacement_block] = &gym.art[&gate.replacement_symbol];
        ensure!(
            native.variant == "tree"
                && replacement_source == source
                && u16::from_str_radix(replacement_block, 16)? == native.replacement_block_id,
            "{}: Cut replacement differs from native source for {}",
            gym.id,
            gate.id
        );
        let mut relocated = native.clone();
        relocated.replacement_block_id = symbols[&gate.replacement_symbol];
        replacements.insert(symbols[&symbol], relocated);
    }
    ensure!(
        pack.data
            .field_moves
            .cut
            .replacements
            .insert(gym.tileset(), replacements)
            .is_none(),
        "gym Cut behavior already installed"
    );
    Ok(())
}

pub(crate) fn validate_geometry(
    pack: &CompiledGamePack,
    module: &MapModule,
    gym: &Gym,
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    let declared = gym
        .cut_gates
        .iter()
        .map(|gate| gate.tree_core)
        .collect::<BTreeSet<_>>();
    let kind = |at: [u16; 2]| {
        let [x, y] = at;
        let block = module.blocks
            [usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2)];
        pack.data.tilesets[&module.attributes.tileset_name].collision[&format!("{block:02x}")]
            [usize::from(y % 2 * 2 + x % 2)]
        .as_str()
    };
    for y in 0..module.attributes.height * 2 {
        for x in 0..module.attributes.width * 2 {
            ensure!(
                (kind([x, y]) == "CUT_TREE") == declared.contains(&[x, y]),
                "{}: undeclared or missing native Cut tree at {x},{y}",
                gym.id
            );
        }
    }
    if gym.cut_gates.is_empty() {
        return Ok(());
    }
    let native = &pack.data.field_moves.cut.replacements[&gym.tileset()];
    let mut opened = module.clone();
    for gate in &gym.cut_gates {
        let index = usize::from(gate.tree_core[1] / 2) * usize::from(module.attributes.width)
            + usize::from(gate.tree_core[0] / 2);
        opened.blocks[index] = native[&module.blocks[index]].replacement_block_id;
    }
    // NPC bodies must never be the permanent boundary of a Cut-gated court.
    let after = walking_map_distances(pack, &opened, gym.arrival_core, &BTreeSet::new())?;
    for gate in &gym.cut_gates {
        let index = usize::from(gate.tree_core[1] / 2) * usize::from(module.attributes.width)
            + usize::from(gate.tree_core[0] / 2);
        let mut closed = opened.clone();
        closed.blocks[index] = module.blocks[index];
        let before = walking_map_distances(pack, &closed, gym.arrival_core, &BTreeSet::new())?;
        for target in &gate.targets_core {
            ensure!(
                !before.contains_key(target) && after.contains_key(target),
                "{}: Cut target {target:?} for {} is bypassable or remains blocked",
                gym.id,
                gate.id
            );
        }
        for id in &gate.gated_trainers {
            let actor = gym
                .trainers
                .iter()
                .find(|t| &t.id == id)
                .context("Cut-gated trainer")?;
            ensure!(
                !before
                    .keys()
                    .any(|at| at[0].abs_diff(actor.at[0]) + at[1].abs_diff(actor.at[1]) == 1)
                    && after
                        .keys()
                        .any(|at| at[0].abs_diff(actor.at[0]) + at[1].abs_diff(actor.at[1]) == 1),
                "{}: trainer {id} is not gated by {}",
                gym.id,
                gate.id
            );
        }
        ensure!(
            !occupied.contains(&gate.approach_core),
            "occupied Cut approach"
        );
    }
    Ok(())
}
