//! Strict reachability policy rooted in the authored new-game spawn and flags.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

pub(super) fn read(root: &Path) -> Result<Value> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let read = |name: &str| -> Result<Value> {
        Ok(serde_json::from_slice(&std::fs::read(data.join(name))?)?)
    };
    let constants = read("story_event_script_constants.json")?;
    let id = constants["global"]["SPAWN_HOME"]
        .as_u64()
        .context("authored SPAWN_HOME constant")?;
    let spawns = read("runtime_spawn_points.json")?;
    let spawn = spawns.get(id.to_string()).context("authored home spawn")?;
    let source = std::fs::read_to_string(vendor.join("engine/menus/intro_menu.asm"))?;
    let mut entry = String::from("NewGame:\n");
    let mut started = false;
    for line in source.lines().map(super::source_syntax::code_part) {
        if line == "NewGame:" {
            started = true;
            continue;
        }
        if !started {
            continue;
        }
        if line.ends_with(':') && !line.starts_with('.') {
            break;
        }
        entry.push_str(line);
        entry.push('\n');
    }
    ensure!(started, "authored NewGame entry is missing");
    let menu = super::source_scripts::parse(&entry)?;
    let new_game = menu
        .get("NewGame")
        .and_then(Value::as_array)
        .context("source NewGame")?;
    ensure!(
        new_game
            .iter()
            .any(|entry| entry == &json!({"command":"ld", "args":["a","SPAWN_HOME"]})),
        "new game does not select authored home spawn"
    );
    let scripts = super::source_scripts::parse(&std::fs::read_to_string(
        vendor.join("engine/events/std_scripts.asm"),
    )?)?;
    let initial = scripts
        .get("InitializeEventsScript")
        .and_then(Value::as_array)
        .context("source initial events")?;
    let mut flags = initial
        .iter()
        .filter(|entry| entry["command"] == "setevent")
        .map(|entry| {
            entry["args"][0]
                .as_str()
                .context("source initial event name")
        })
        .collect::<Result<Vec<_>>>()?;
    // Source setevent is idempotent; the playability schema declares a set of
    // initially active flags rather than a tape of repeated source commands.
    let mut seen_flags = std::collections::BTreeSet::new();
    flags.retain(|flag| seen_flags.insert(*flag));
    ensure!(!flags.is_empty(), "source initial event list is empty");
    let surf_source = super::source_scripts::parse(&std::fs::read_to_string(
        vendor.join("maps/DanceTheater.asm"),
    )?)?;
    ensure!(
        surf_source
            .values()
            .filter_map(Value::as_array)
            .flatten()
            .any(|command| command["command"] == "verbosegiveitem"
                && command["args"][0] == "HM_SURF"),
        "source topology Surf capability has no authored HM grant"
    );
    let field_moves = read("field_moves.json")?;
    ensure!(
        field_moves["surf"]["move_id"] == "SURF",
        "source topology Surf capability has no production travel rule"
    );
    for (file, item, command, key, move_id) in [
        ("maps/IlexForest.asm", "HM_CUT", "verbosegiveitem", "cut", "CUT"),
        (
            "maps/IcePath1F.asm",
            "HM_WATERFALL",
            "itemball",
            "waterfall",
            "WATERFALL",
        ),
        (
            "maps/TeamRocketBaseB2F.asm",
            "HM_WHIRLPOOL",
            "verbosegiveitem",
            "whirlpool",
            "WHIRLPOOL",
        ),
    ] {
        let scripts = super::source_scripts::parse(&std::fs::read_to_string(vendor.join(file))?)?;
        ensure!(
            scripts
                .values()
                .filter_map(Value::as_array)
                .flatten()
                .any(|entry| entry["command"] == command && entry["args"][0] == item),
            "source topology {move_id} capability has no authored HM grant"
        );
        ensure!(
            field_moves[key]["move_id"] == move_id,
            "source topology {move_id} capability has no production field rule"
        );
    }
    let map_attributes = read("map_attributes.json")?;
    let dormant = map_attributes
        .as_object()
        .context("source map attributes")?
        .keys()
        .filter(|name| name.ends_with("Beta"))
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        dormant.len() == 13,
        "pinned source inactive Beta map count changed"
    );
    let mut tokens = Vec::new();
    for map in &dormant {
        let source = std::fs::read_to_string(vendor.join(format!("maps/{map}.asm")))?;
        ensure!(
            source
                .lines()
                .any(|line| super::source_syntax::code_part(line) == format!("{map}_MapEvents:")),
            "inactive map lacks authored events {map}"
        );
        let constant = map_attributes[map]["map_constant"]
            .as_str()
            .with_context(|| format!("inactive map identity {map}"))?;
        tokens.push((map.clone(), constant.to_owned()));
    }
    let mut inaccessible = std::collections::BTreeMap::<String, Vec<i32>>::new();
    for entry in std::fs::read_dir(vendor.join("maps"))? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("asm") {
            continue;
        }
        let origin = path
            .file_stem()
            .and_then(|value| value.to_str())
            .context("source map filename")?;
        if dormant.iter().any(|name| name == origin) {
            continue;
        }
        let raw = std::fs::read_to_string(&path)?;
        let mut warp_index = 0;
        let mut certified_commands = Vec::new();
        for line in raw.lines() {
            let code = super::source_syntax::code_part(line);
            if code.trim_start().starts_with("warp_event ") {
                warp_index += 1;
                if line.split_once(';').is_some_and(|(_, comment)| comment.trim() == "inaccessible") {
                    let parsed = super::source_scripts::parse(&format!("Certificate:\n{code}\n"))?;
                    for command in parsed.values().filter_map(Value::as_array).flatten() {
                        if tokens.iter().any(|(_, constant)| command["args"][2].as_str() == Some(constant)) {
                            inaccessible.entry(origin.to_owned()).or_default().push(warp_index);
                            certified_commands.push(command.clone());
                        }
                    }
                }
            }
        }
        let source = super::source_scripts::parse(&raw)?;
        for command in source.values().filter_map(Value::as_array).flatten() {
            if certified_commands.contains(command) { continue; }
            for argument in command["args"].as_array().into_iter().flatten() {
                ensure!(
                    !tokens
                        .iter()
                        .any(|(name, constant)| argument.as_str() == Some(name)
                            || argument.as_str() == Some(constant)),
                    "active source map {origin} references inactive Beta map {argument}"
                );
            }
        }
    }
    let value = json!({
        "topology_traversal_modes":["Walk","Surf","Waterfall","Whirlpool","Cut"],
        "topology_script_block_changes":true,
        "dormant_maps":dormant,
        "inaccessible_dormant_warps":inaccessible,
        "retired_gs_warps":retired_gs_warps(root)?,
        "connection_warp_routes":connection_warp_routes(root)?,
        "start_maps":[spawn["mapName"]],
        "start_tiles":[{"map":spawn["mapName"],"tile":{"x":spawn["tileX"],"y":spawn["tileY"]}}],
        "initial_events":flags,"initial_items":[],
        "goal_maps":[],"goal_events":[],"goal_items":[],
        "progression_rules":[],"map_access":[],
        "require_all_maps_reachable":true,"require_walkable_maps":true
    });
    let _: crystal_assets::modpack::PlayabilityRules = serde_json::from_value(value.clone())?;
    Ok(value)
}

pub fn export(root: &Path) -> Result<()> {
    let value = read(root)?;
    let data = root.join("apps/web/assets/data");
    for path in [
        data.join("playability.json"),
        data.join("content-packs/core-modular/playability/rules.json"),
    ] {
        std::fs::create_dir_all(path.parent().context("playability output parent")?)?;
        std::fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    }
    Ok(())
}

fn connection_warp_routes(root: &Path) -> Result<std::collections::BTreeMap<String, std::collections::BTreeMap<String, Vec<String>>>> {
    let vendor = root.join("vendor/pokecrystal");
    let geometry = super::source_map_geometry::read(&vendor)?;
    let routes: &[&[&str]] = &[
        &["AzaleaTown", "IlexForestAzaleaGate", "IlexForest", "Route34IlexForestGate", "Route34"],
        &["EcruteakCity", "Route38EcruteakGate", "Route38"],
        &["EcruteakCity", "Route42EcruteakGate", "Route42"],
        &["FuchsiaCity", "Route15FuchsiaGate", "Route15"],
        &["Route29", "Route29Route46Gate", "Route46"],
        &["VioletCity", "Route31VioletGate", "Route31"],
        &["Route5", "Route5SaffronGate", "SaffronCity"],
        &["Route6", "Route6SaffronGate", "SaffronCity"],
        &["Route8", "Route8SaffronGate", "SaffronCity"],
    ];
    let mut result = std::collections::BTreeMap::<String, std::collections::BTreeMap<String, Vec<String>>>::new();
    for route in routes {
        for names in [route.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>(),
            route.iter().rev().map(|name| (*name).to_owned()).collect::<Vec<_>>()] {
            let origin = &names[0];
            let destination = names.last().context("connection route destination")?;
            ensure!(geometry.get(origin).is_some_and(|map| map.attributes.connections.iter()
                .any(|connection| &connection.target_map == destination)),
                "authored backdrop connection {origin} to {destination} is missing");
            for pair in names.windows(2) {
                let target = geometry.get(&pair[1]).and_then(|map| map.attributes.map_constant.as_deref())
                    .context("warp route target identity")?;
                let source = std::fs::read_to_string(vendor.join(format!("maps/{}.asm", pair[0])))?;
                ensure!(source.lines().any(|line| super::source_syntax::code_part(line)
                    .strip_prefix("warp_event ").is_some_and(|args|
                        super::source_syntax::source_arguments(args).is_ok_and(|args|
                            args.get(2).is_some_and(|value| value == target)))),
                    "authored connection detour lacks warp {} to {}", pair[0], pair[1]);
            }
            result.entry(origin.clone()).or_default().insert(destination.clone(), names);
        }
    }
    Ok(result)
}

fn retired_gs_warps(root: &Path) -> Result<std::collections::BTreeMap<String, Vec<i32>>> {
    use std::collections::{BTreeMap, BTreeSet};
    #[derive(Clone)]
    struct Warp { x: usize, y: usize, target: String, destination: usize, retired: bool }
    let vendor = root.join("vendor/pokecrystal");
    let geometry = super::source_map_geometry::read(&vendor)?;
    let tilesets: Value = serde_json::from_slice(&std::fs::read(root.join("apps/web/assets/data/tilesets.json"))?)?;
    let identities = geometry.iter().filter_map(|(name, map)| map.attributes.map_constant.as_ref()
        .map(|constant| (constant.clone(), name.clone()))).collect::<BTreeMap<_, _>>();
    let mut warps = BTreeMap::<String, Vec<Warp>>::new();
    for name in geometry.keys() {
        let source = std::fs::read_to_string(vendor.join(format!("maps/{name}.asm")))?;
        let mut rows = Vec::new();
        for line in source.lines() {
            let Some(args) = super::source_syntax::code_part(line).trim().strip_prefix("warp_event ") else { continue; };
            let args = super::source_syntax::source_arguments(args)?;
            ensure!(args.len() == 4, "retired source warp arity");
            let destination = args[3].parse::<i32>()?;
            rows.push(Warp { x: args[0].parse()?, y: args[1].parse()?,
                target: identities.get(&args[2]).cloned().unwrap_or(args[2].clone()),
                destination: destination.max(0) as usize,
                retired: line.split_once(';').is_some_and(|(_, comment)| comment.trim() == "inaccessible, left over from G/S") });
        }
        warps.insert(name.clone(), rows);
    }
    let blocked = |name: &str, warp: &Warp| -> Result<bool> {
        let map = geometry.get(name).context("retired source map geometry")?;
        let index = (warp.y / 2) * usize::from(map.attributes.width) + warp.x / 2;
        let block = *map.blocks.get(index).context("retired warp block bounds")?;
        let collision = tilesets[&map.attributes.tileset_name]["collision"].as_object().context("retired warp collision table")?;
        let quadrants = collision.iter().find(|(key, _)| u16::from_str_radix(key, 16).ok() == Some(block))
            .map(|(_, value)| value).context("retired warp collision block")?;
        Ok(quadrants[(warp.y % 2) * 2 + warp.x % 2] == "WALL")
    };
    let mut certified = BTreeMap::<String, BTreeSet<i32>>::new();
    for (name, rows) in &warps {
        for (index, warp) in rows.iter().enumerate().filter(|(_, warp)| warp.retired) {
            let target = warps.get(&warp.target).context("retired warp target source")?;
            let destination = target.get(warp.destination.checked_sub(1).context("retired warp requires destination")?)
                .context("retired warp target index")?;
            if !blocked(name, warp)? && !blocked(&warp.target, destination)? { continue; }
            certified.entry(name.clone()).or_default().insert((index + 1) as i32);
            if destination.target == *name && destination.destination == index + 1 {
                certified.entry(warp.target.clone()).or_default().insert(warp.destination as i32);
            }
        }
    }
    Ok(certified.into_iter().map(|(name, indices)| (name, indices.into_iter().collect())).collect())
}
