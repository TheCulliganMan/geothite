use crystal_assets::modpack::GameDataSet;
use crystal_core::{state::GameState, world::movement::MovementMode};
use std::{collections::BTreeSet, path::PathBuf};

fn source_data() -> Option<GameDataSet> {
    let root=PathBuf::from(std::env::var_os("CRYSTAL_EXPORT_WORKSPACE")?);
    let read=|name:&str| serde_json::from_slice::<serde_json::Value>(&std::fs::read(root.join("apps/web/assets/data").join(name)).unwrap()).unwrap();
    let mut data=GameDataSet::default();
    data.map_attributes=serde_json::from_value(read("map_attributes.json")).unwrap();
    data.map_blocks=serde_json::from_value(read("map_blocks.json")).unwrap();
    data.map_dimensions=serde_json::from_value(read("map_dimensions.json")).unwrap();
    data.map_scripts=serde_json::from_value(read("map_scripts.json")).unwrap();
    data.runtime_map_metadata=serde_json::from_value(read("runtime_map_metadata.json")).unwrap();
    data.npcs=serde_json::from_value(read("npcs.json")).unwrap();
    data.tilesets=serde_json::from_value(read("tilesets.json")).unwrap();
    data.story_event_script_constants=serde_json::from_value(read("story_event_script_constants.json")).unwrap();
    for name in data.map_attributes.keys() {data.maps.insert(name.clone(),data.assemble_map_module_from_compiled_payloads(name).unwrap());}
    Some(data)
}

#[test]
fn authored_rocket_music_changes_after_story_flags_and_retains_surf_priority() {
    let Some(data)=source_data() else{return};
    for (name,flag,base,takeover) in [
        ("RadioTower1F","ENGINE_ROCKETS_IN_RADIO_TOWER","MUSIC_GOLDENROD_CITY","MUSIC_ROCKET_OVERTURE"),
        ("MahoganyMart1F","ENGINE_ROCKETS_IN_MAHOGANY","MUSIC_CHERRYGROVE_CITY","MUSIC_ROCKET_HIDEOUT"),
    ] {
        assert_eq!(data.maps[name].attributes.music.as_deref(),Some(base));
        let mut state=GameState::default();
        let music=BTreeSet::from([base.to_owned(),takeover.to_owned(),"MUSIC_SURF".to_owned()]);
        for active in [false,true,false] {
            state.flags.set_engine_flag(flag,active).unwrap();
            data.sync_current_map_music(&mut state,name,MovementMode::Normal,&music).unwrap();
            assert_eq!(state.script_runtime.current_music.as_deref(),Some(if active{takeover}else{base}));
        }
        state.flags.set_engine_flag(flag,true).unwrap();
        data.sync_current_map_music(&mut state,name,MovementMode::Surf,&music).unwrap();
        assert_eq!(state.script_runtime.current_music.as_deref(),Some("MUSIC_SURF"));
        assert!(data.sync_current_map_music(&mut state,name,MovementMode::Normal,&BTreeSet::from([base.to_owned()])).is_err());
    }
}

#[test]
fn authored_trainers_keep_visibility_separate_from_defeat_flags() {
    let Some(data)=source_data() else{return};
    let gym=&data.maps["CeruleanGym"];
    let object=gym.objects.iter().find(|object|object.script=="TrainerSwimmerfBriana").unwrap();
    assert_eq!(object.event_flag,"EVENT_TRAINERS_IN_CERULEAN_GYM");
    assert_eq!(gym.trainer_scripts[&object.script].event_flag,"EVENT_BEAT_SWIMMERF_BRIANA");
    let route=&data.maps["Route30"];
    let object=route.objects.iter().find(|object|object.script=="TrainerYoungsterMikey").unwrap();
    assert_eq!(object.event_flag,"-1");
    assert_eq!(route.trainer_scripts[&object.script].event_flag,"EVENT_BEAT_YOUNGSTER_MIKEY");
    let report=crystal_assets::modpack::ModpackCompiler::new(&crystal_assets::AssetRoot::new(".")).audit(&data);
    assert!(!report.diagnostics.iter().any(|diagnostic|diagnostic.code=="invalid_trainer_object_visibility_flag"));
}
