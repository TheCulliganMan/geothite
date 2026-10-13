//! Authored forced entry and exit paths may reach a door that ordinary walking
//! cannot enter. This certificate applies only to warp graph endpoints.
use super::*;

fn movement_delta(module: &MapModule, script: &str, label: &str) -> Option<(i16, i16)> {
    let mut matches = module.script_movements.iter().filter(|movement|
        movement.label == label && movement.source_script.as_deref() == Some(script));
    let movement = matches.next()?;
    if matches.next().is_some() { return None; }
    let mut delta = (0_i16, 0_i16);
    let mut ended = false;
    for (index, step) in movement.steps.iter().enumerate() {
        if step.index != index { return None; }
        if ended { return None; }
        if step.command == "step_end" { ended = true; continue; }
        if !matches!(step.command.as_str(), "step" | "slow_step" | "fast_step") { return None; }
        let direction = match step.direction.as_deref()? {
            "UP" => (0, -PLAYABILITY_RUNTIME_TILE_STRIDE),
            "DOWN" => (0, PLAYABILITY_RUNTIME_TILE_STRIDE),
            "LEFT" => (-PLAYABILITY_RUNTIME_TILE_STRIDE, 0),
            "RIGHT" => (PLAYABILITY_RUNTIME_TILE_STRIDE, 0),
            _ => return None,
        };
        delta.0 = delta.0.checked_add(direction.0)?;
        delta.1 = delta.1.checked_add(direction.1)?;
    }
    ended.then_some(delta)
}

fn simple_player_movement<'a>(body: &'a Value) -> Option<&'a str> {
    let commands = body.as_array()?;
    if commands.last()?["command"] != "end" { return None; }
    let mut player = None;
    for command in &commands[..commands.len() - 1] {
        match command["command"].as_str()? {
            "applymovement" if command["args"][0] == "PLAYER" => {
                if player.is_some() { return None; }
                player = Some(command["args"][1].as_str()?);
            }
            "applymovement" | "applymovementlasttalked" => {},
            _ => return None,
        }
    }
    player
}

pub(super) fn component_at(
    module: &MapModule, context: &MapPlayabilityContext, endpoint: TilePosition,
) -> Option<usize> {
    if let Some(component) = context.component_at(endpoint) { return Some(component); }
    let (width, height) = context.map.tile_bounds();
    if endpoint.x < 0 || endpoint.y < 0 || endpoint.x as u16 >= width || endpoint.y as u16 >= height {
        return None;
    }
    // A normal actor interaction must lead to a simple forced movement call
    // immediately followed by warpcheck. Merely having movement data is not
    // enough to make a wall traversable.
    for object in &module.objects {
        let callable = runtime_module_script_subset(&module.scripts, [object.script.as_str()], false);
        let actor = checked_runtime_map_event_tile(object.x, object.y)?;
        for body in callable.values().filter_map(Value::as_array) {
            for pair in body.windows(2) {
                if pair[0]["command"] != "scall" || pair[1]["command"] != "warpcheck" { continue; }
                let Some(callee) = pair[0]["args"][0].as_str() else { continue; };
                let Some(called) = callable.get(callee) else { continue; };
                let Some(label) = simple_player_movement(called) else { continue; };
                let Some(delta) = movement_delta(module, callee, label) else { continue; };
                let Some(x) = endpoint.x.checked_sub(delta.0) else { continue; };
                let Some(y) = endpoint.y.checked_sub(delta.1) else { continue; };
                let origin = TilePosition::new(x, y);
                if i32::from(origin.x).abs_diff(i32::from(actor.x))
                    + i32::from(origin.y).abs_diff(i32::from(actor.y))
                    != PLAYABILITY_RUNTIME_TILE_STRIDE as u32 { continue; }
                let Some(component) = context.component_at(origin) else { continue; };
                // The landing is usable only when an authored scene also
                // provides the inverse forced exit into that same component.
                for scene in &module.scenes.scenes {
                    let Some(root) = scene.script_name.as_deref() else { continue; };
                    let exit = runtime_module_script_subset(&module.scripts, [root], false);
                    if exit.iter().any(|(script, body)| simple_player_movement(body)
                        .and_then(|label| movement_delta(module, script, label))
                        .is_some_and(|back| i32::from(back.0) == -i32::from(delta.0)
                            && i32::from(back.1) == -i32::from(delta.1))) {
                        return Some(component);
                    }
                }
            }
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    use crystal_core::map::{MapScene, MapSceneTable, MapEvents, MapConnection};
    use crystal_core::systems::script_objects::{ScriptMovement, ScriptMovementStep};
    use serde_json::json;fn test_map_module(id: &str, map_constant: &str, connection_target: Option<&str>) -> MapModule {
    MapModule {
        id: id.to_string(),
        attributes: MapAttributes {
            tileset_name: "johto".to_string(),
            border_block: 0,
            width: 1,
            height: 1,
            connections: connection_target
                .map(|target| {
                    vec![MapConnection {
                        direction: "east".to_string(),
                        target_map: target.to_string(),
                        offset: 0,
                    }]
                })
                .unwrap_or_default(),
            time_of_day: None,
            phone_service: 0,
            phone_flag: false,
            environment: Some("route".to_string()),
            location: Some("johto".to_string()),
            music: None,
            palette: None,
            fishing_group: None,
            map_constant: Some(map_constant.to_string()),
            map_group_constant: None,
            blocks_label: None,
            map_scripts_label: None,
            map_events_label: None,
            connection_flags: None,
        },
        scripts: BTreeMap::new(),
        trainer_scripts: BTreeMap::new(),
        scripted_trainer_battles: Vec::new(),
        scripted_wild_battles: Vec::new(),
        script_item_grants: Vec::new(),
        script_item_checks: Vec::new(),
        script_item_takes: Vec::new(),
        script_economy_commands: Vec::new(),
        gift_pokemon_scripts: Vec::new(),
        script_flag_commands: Vec::new(),
        script_scene_commands: Vec::new(),
        script_audio_commands: Vec::new(),
        script_block_changes: Vec::new(),
        script_object_commands: Vec::new(),
        script_movements: Vec::new(),
        script_map_commands: Vec::new(),
        script_text_commands: Vec::new(),
        script_text_bodies: BTreeMap::new(),
        script_menu_definitions: BTreeMap::new(),
        script_vertical_menus: BTreeMap::new(),
        script_elevators: BTreeMap::new(),
        script_variable_commands: Vec::new(),
        script_control_commands: Vec::new(),
        script_field_pickups: Vec::new(),
        script_shop_commands: Vec::new(),
        script_phone_commands: Vec::new(),
        script_runtime_commands: Vec::new(),
        script_swarm_commands: Vec::new(),
        map_script_section_commands: Vec::new(),
        map_event_section_commands: Vec::new(),
        scenes: MapSceneTable::default(),
        events: MapEvents::default(),
        objects: Vec::new(),
        blocks: vec![1],
    }
}

    fn fixture() -> (MapModule, MapPlayabilityContext) {
        let mut module=test_map_module("Reception", "RECEPTION", None);
        module.attributes.width=8;module.attributes.height=4;module.blocks=vec![0;32];
        module.objects.push(ObjectEvent {
            sprite:"SPRITE_RECEPTIONIST".into(),sprite_has_facings:true,x:5,y:2,
            spritemovedata:"SPRITEMOVEDATA_STANDING_DOWN".into(),move_range_x:0,move_range_y:0,
            hram_x:-1,hram_y:-1,pal:0,object_type:"OBJECTTYPE_SCRIPT".into(),radius:0,
            script:"Reception".into(),label:None,event_flag:String::new(),
            object_identifier:Some("RECEPTIONIST".into()),sightline_direction_override:None,
        });
        module.scripts.insert("Reception".into(),json!([
            {"command":"scall","args":["Enter"]},{"command":"warpcheck","args":[]},
            {"command":"end","args":[]} ]));
        module.scripts.insert("Enter".into(),json!([
            {"command":"applymovementlasttalked","args":["NpcMoves"]},
            {"command":"applymovement","args":["PLAYER","In"]},{"command":"end","args":[]} ]));
        module.scripts.insert("LeaveScene".into(),json!([
            {"command":"sdefer","args":["Leave"]},{"command":"end","args":[]} ]));
        module.scripts.insert("Leave".into(),json!([
            {"command":"applymovement","args":["PLAYER","Out"]},{"command":"end","args":[]} ]));
        module.scenes.scenes.push(MapScene {scene_id:"LEAVE".into(),script_name:Some("LeaveScene".into())});
        for (script,label,directions) in [("Enter","In",["UP","UP","RIGHT","UP"]),
            ("Leave","Out",["DOWN","LEFT","DOWN","DOWN"])] {
            let mut steps=directions.into_iter().enumerate().map(|(index,direction)| ScriptMovementStep {
                command:"step".into(),direction:Some(direction.into()),duration:None,index,
            }).collect::<Vec<_>>();
            steps.push(ScriptMovementStep {command:"step_end".into(),direction:None,duration:None,index:4});
            module.script_movements.push(ScriptMovement {label:label.into(),source_script:Some(script.into()),steps});
        }
        let context=MapPlayabilityContext {
            map:OverworldMapData::from_attributes("Reception",&module.attributes,module.blocks.clone()),
            component_by_tile:BTreeMap::from([((10,6),0)]),component_count:1,
            ledge_component_edges:BTreeSet::new(),
        };
        (module,context)
    }
    #[test]
    fn forced_reception_entry_and_scene_exit_certify_only_the_warp_endpoint() {
        let (module,context)=fixture();
        assert_eq!(context.component_at(TilePosition::new(12,0)),None,"ordinary wall collision stays blocked");
        assert_eq!(component_at(&module,&context,TilePosition::new(12,0)),Some(0));
        assert_eq!(component_at(&module,&context,TilePosition::new(10,6)),Some(0));
        assert_eq!(component_at(&module,&context,TilePosition::new(8,0)),None,"unrelated walls stay blocked");
        assert_eq!(component_at(&module,&context,TilePosition::new(12,-2)),None);
    }
    #[test]
    fn unsupported_or_unreachable_script_paths_cannot_certify_a_wall() {
        let endpoint=TilePosition::new(12,0);
        for mutation in 0..6 {
            let (mut module,context)=fixture();
            match mutation {
                0=>module.scripts.get_mut("Reception").unwrap()[1]["command"]=json!("end"),
                1=>module.objects[0].script="Uncalled".into(),
                2=>module.script_movements[0].steps.pop().map(|_| ()).unwrap(),
                3=>module.script_movements[1].steps[0].direction=Some("UP".into()),
                4=>module.scenes.scenes.clear(),
                _=>module.objects[0].x=1,
            }
            assert_eq!(component_at(&module,&context,endpoint),None,"mutation {mutation}");
        }
    }
}
