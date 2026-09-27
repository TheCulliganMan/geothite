//! HM teaching goals read the actual visible menus. No button is prescribed.
use serde_json::Value;
const HMS:[(&str,&str,usize);7]=[("HM_CUT","CUT",1),("HM_SURF","SURF",3),("HM_STRENGTH","STRENGTH",2),("HM_WHIRLPOOL","WHIRLPOOL",6),("HM_WATERFALL","WATERFALL",7),("HM_FLY","FLY",5),("HM_FLASH","FLASH",0)];
pub(crate) fn known(v:&Value,name:&str)->bool {
    v.pointer("/status/party").and_then(Value::as_array).into_iter().flatten()
        .filter(|p|p["is_egg"]!=true).any(|p|p["moves"].as_array().is_some_and(|m|m.iter().any(|m|m["name"].as_str()==Some(name))))
}
fn incompatible(v:&Value,item:&str)->bool {
    v.pointer("/map_info/hm_compatibility").and_then(|m|m.get(item)).and_then(Value::as_array)
        .is_some_and(|slots|!slots.is_empty() && slots.iter().all(|p|p["able"]==false))
}
/// Connect a verified party capability gap to the currently encountered species.
pub(crate) fn capture_hm(v:&Value)->Option<&'static str> {
    HMS.iter().find(|(item,name,_)|!known(v,name) && incompatible(v,item)
        && v.pointer("/reward_state/machines").and_then(Value::as_array).is_some_and(|a|a.iter().any(|i|i==item))
        && v.pointer("/reward_state/enemy_hm_compatibility").and_then(|m|m.get(*item))==Some(&Value::Bool(true)))
        .map(|(_,name,_)|*name)
}
fn boxed_candidate(v:&Value)->Option<(&'static str,&'static str,usize,usize)> {
    if v.pointer("/status/screen")?.as_str()?!="overworld" {return None;}
    HMS.iter().find_map(|(item,name,_)|{
        if known(v,name)||!incompatible(v,item){return None;}
        let candidate=v.pointer("/reward_state/boxed_hm_candidates")?.get(*item)?;
        Some((*item,*name,candidate["box"].as_u64()? as usize,candidate["position"].as_u64()? as usize))
    })
}
pub(crate) fn needs_boxed_hm(v:&Value)->bool {boxed_candidate(v).is_some()}
fn deposit_candidate(v:&Value)->Option<usize> {
    let party=v.pointer("/status/party")?.as_array()?;
    if party.len()<6{return None;}
    party.iter().enumerate().filter(|(i,p)| {
        if p["item"].as_str().is_some_and(|s|s.contains("MAIL")){return false;}
        let others=party.iter().enumerate().filter(|(j,_)|j!=i).map(|(_,p)|p).collect::<Vec<_>>();
        if !others.iter().any(|p|p["is_egg"]!=true&&p["hp"].as_u64().unwrap_or(0)>0){return false;}
        HMS.iter().all(|(_,hm,_)|!p["moves"].as_array().is_some_and(|ms|ms.iter().any(|m|m["name"]==*hm))
            || others.iter().any(|p|p["is_egg"]!=true&&p["moves"].as_array().is_some_and(|ms|ms.iter().any(|m|m["name"]==*hm))))
    }).min_by_key(|(_,p)|if p["is_egg"]==true{0}else{p["level"].as_u64().unwrap_or(u64::MAX)})
        .map(|(i,_)|i)
}
fn withdrawal(v:&Value)->Option<(String,f32)> {
    let egg=crate::curriculum::pending_togepi(v) && v.pointer("/map_info/name").and_then(Value::as_str)==Some("VioletPokecenter1F");
    let candidate=boxed_candidate(v);
    if !egg && candidate.is_none(){return None;}
    let menu=v.pointer("/observe/menus")?.as_array()?.last()?;
    if menu["kind"]!="pc" && !(menu["kind"]=="yes_no" && menu["purpose"]=="pc_change_box") {return None;}
    let rows=menu["entries"].as_array()?;
    let labels=rows.iter().filter_map(Value::as_str).map(|s|s.trim().trim_start_matches('>').trim().to_uppercase()).collect::<Vec<_>>();
    let cursor=rows.iter().position(|s|s.as_str().is_some_and(|s|s.trim().starts_with('>')))?;
    let current_box=menu["box_index"].as_u64()? as usize;
    let (_,name,target_box,target_slot)=if egg{("EGG","TOGEPI",current_box,0)}else{candidate?};
    let full=v.pointer("/status/party")?.as_array()?.len()>=6;
    let deposit=deposit_candidate(v);
    let free_box=menu["box_counts"].as_array().and_then(|boxes|boxes.iter()
        .filter(|b|matches!((b["count"].as_u64(),b["capacity"].as_u64()),(Some(n),Some(c)) if n<c))
        .min_by_key(|b|if b["box"].as_u64()==Some(current_box as u64){0}else{1})
        .and_then(|b|b["box"].as_u64())).map(|b|b as usize);
    let can_deposit=full&&deposit.is_some()&&free_box.is_some();
    let wanted_box=if can_deposit{free_box.unwrap()}else{target_box};
    if menu["kind"]=="yes_no" {
        let correct=menu["target_box"].as_u64()==Some(wanted_box as u64);
        let target=labels.iter().position(|s|s==if correct{"YES"}else{"NO"})?;
        let direction=if target<cursor{"north"}else if target>cursor{"south"}else{"aligned"};
        return Some((format!("objective:pc:{name}:{}|objective:pc:cursor:{direction}",if correct{"confirm_box_save"}else{"cancel_wrong_box_save"}),2.8-0.1*target.abs_diff(cursor) as f32));
    }
    let (stage,rank,target,cursor)=match menu["surface"].as_str()? {
        "hub" if egg && !full=>("return_to_aide",0.0,labels.iter().position(|s|s=="TURN OFF")?,cursor),
        "hub"=>("open_storage",1.0,labels.iter().position(|s|s.contains("BILL")||s.contains("SOMEONE"))?,cursor),
        "storage_actions"=>{
            let label=if egg&&!full || full&&!can_deposit{"SEE YA"}else if current_box!=wanted_box{"CHANGE BOX"}else if full{"DEPOSIT"}else{"WITHDRAW"};
            (if full{"make_party_space"}else{"choose_storage_action"},2.0,labels.iter().position(|s|s.starts_with(label))?,cursor)
        },
        "boxes"=>("choose_box",2.5,wanted_box,menu["selected_box"].as_u64()? as usize),
        "box_actions"=>{
            let correct=menu["selected_box"].as_u64()==Some(wanted_box as u64);
            (if correct{"switch_box"}else{"return_to_boxes"},2.7,labels.iter().position(|s|s==if correct{"SWITCH"}else{"QUIT"})?,cursor)
        },
        "deposit" if can_deposit && current_box==wanted_box =>("choose_deposit",2.8,deposit?,menu["selected_slot"].as_u64()? as usize),
        "pokemon_actions" if can_deposit && current_box==wanted_box && menu["selected_slot"].as_u64()==deposit.map(|i|i as u64) && labels.iter().any(|s|s=="DEPOSIT") =>("deposit_for_hm_space",2.9,labels.iter().position(|s|s=="DEPOSIT")?,cursor),
        "withdraw" if !egg && current_box==target_box && !full =>("choose_candidate",3.0,target_slot,menu["selected_slot"].as_u64()? as usize),
        "pokemon_actions" if !egg && current_box==target_box && menu["selected_slot"].as_u64()==Some(target_slot as u64) && !full =>("withdraw_candidate",4.0,labels.iter().position(|s|s=="WITHDRAW")?,cursor),
        "release_confirmation"=>("cancel_release",0.0,labels.iter().position(|s|s=="NO")?,cursor),
        _=>("return_to_storage",0.0,labels.iter().position(|s|s=="CANCEL"||s=="QUIT")?,cursor),
    };
    let direction=if target<cursor{"north"}else if target>cursor{"south"}else{"aligned"};
    Some((format!("objective:pc:{name}:{stage}|objective:pc:cursor:{direction}"),rank-0.1*target.abs_diff(cursor) as f32))
}
fn withdrawal_feedback(before:&Value,after:&Value)->Option<f32> {
    if [before,after].iter().any(|v|v.pointer("/observe/menus").and_then(Value::as_array).is_some_and(|ms|ms.iter().any(|m|m["kind"]=="pc"||m["purpose"]=="pc_change_box")))
        && [before,after].iter().all(|v|crate::curriculum::pending_togepi(v) && v.pointer("/map_info/name").and_then(Value::as_str)==Some("VioletPokecenter1F")){
        let full=|v:&Value|if crate::curriculum::needs_egg_space(v){1.0}else{0.0};
        let a=withdrawal(before).map_or(0.0,|g|g.1);let b=withdrawal(after).map_or(0.0,|g|g.1);
        return Some(0.5*(full(before)-full(after))+0.2*(b-a));
    }
    let (item,_,_,_)=boxed_candidate(before)?;
    // Menu progress is signed and cancels on a round trip. Withdrawal itself
    // changes capability; do not award repeatable deposit/withdraw bonuses.
    if boxed_candidate(after).map(|c|c.0)!=Some(item){return Some(0.0);}
    let a=withdrawal(before).map_or(0.0,|g|g.1);let b=withdrawal(after).map_or(0.0,|g|g.1);
    Some(0.2*(b-a))
}
fn desired(v:&Value)->Option<(&'static str,&'static str,usize)> {
    if v.pointer("/status/screen").and_then(Value::as_str)!=Some("overworld"){return None;}
    HMS.into_iter().find(|(item,name,_)| !known(v,name) && !incompatible(v,item) && v.pointer("/reward_state/machines").and_then(Value::as_array).is_some_and(|m|m.iter().any(|i|i.as_str()==Some(item))))
}
/// A field teaching task starts before opening Start/Pack. Do not let walking
/// cues or a walking policy head compete with it, but finish visible dialogue.
pub(crate) fn teaching_target(v:&Value)->Option<&'static str>{
    if ["/observe/visible_dialogue","/observe/battle_message"].iter().any(|p|v.pointer(p).and_then(Value::as_str).is_some_and(|s|!s.trim().is_empty())){return None;}
    desired(v).map(|(_,name,_)|name)
}
#[derive(Debug)]
struct Teaching {
    move_name:&'static str,
    stage:&'static str,
    rank:f32,
    target:Option<usize>,
    cursor:Option<usize>,
    badge:usize,
}
fn teaching(v:&Value)->Option<Teaching> {
    let (item,name,badge)=desired(v)?;
    let display=match name {"CUT"=>"HM01","FLY"=>"HM02","SURF"=>"HM03","STRENGTH"=>"HM04","FLASH"=>"HM05","WHIRLPOOL"=>"HM06",_=>"HM07"};
    let menu=v.pointer("/observe/menus").and_then(Value::as_array).and_then(|m|m.last());
    let rows:Vec<_>=menu.and_then(|m|m.get("entries").or_else(||m.get("options"))).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
    let labels:Vec<_>=rows.iter().map(|s|s.trim().trim_start_matches('>').trim().to_uppercase()).collect();
    let cursor=rows.iter().position(|s|s.trim().starts_with('>')).or_else(||menu.and_then(|m|m["selected"].as_u64()).map(|i|i as usize));
    let desired_header=labels.first().is_some_and(|s|s.split_whitespace().any(|t|t==name||t==display));
    let selected=menu.and_then(|m|m["selected_machine"].as_str());
    let selected_desired=selected==Some(item)||desired_header;
    let (stage,rank,target)=if labels.is_empty(){("open_pack",0.0,None)}
        else if labels.iter().any(|s|s=="CHOOSE A MOVE TO FORGET") && desired_header {
            let target=labels.iter().enumerate().skip(2).find(|(_,s)|s.as_str()!="CANCEL" && !HMS.iter().any(|(_,hm,_)|s.split_whitespace().any(|t|t==*hm))).map(|(i,_)|i);
            ("replace_non_hm",5.0,target)
        }else if labels.first().is_some_and(|s|s.starts_with("DELETE A MOVE FOR ")) && selected_desired {
            ("allow_replacement",4.5,labels.iter().position(|s|s=="YES"))
        }else if labels.first().is_some_and(|s|s=="STOP LEARNING THIS MOVE?") && selected_desired {
            ("continue_learning",4.5,labels.iter().position(|s|s=="NO"))
        }else if labels.first().is_some_and(|s|s.starts_with("TEACH ")&&s.contains(name)) {
            ("confirm_teaching",3.0,labels.iter().position(|s|s=="YES"))
        }else if desired_header && labels.iter().any(|s|s.ends_with("ABLE")) {
            let target=labels.iter().position(|s|s.ends_with(" ABLE")&&!s.ends_with("NOT ABLE"));
            (if target.is_some(){"choose_compatible"}else{"need_compatible_pokemon"},4.0,target)
        }else if labels.first().is_some_and(|s|s.starts_with("ACTION ")&&(s.contains(name)||s.contains(display))) {
            ("use_machine",2.0,labels.iter().position(|s|s=="USE"))
        }else if labels.first().is_some_and(|s|s.starts_with("POCKET:")) {
            let target=labels.iter().enumerate().skip(1).find(|(_,s)|s.split_whitespace().any(|t|t==name||t==display)).map(|(i,_)|i);
            (if target.is_some(){"select_machine"}else{"find_machine_pocket"},if target.is_some(){1.5}else{1.0},target)
        }else {("open_pack",0.5,labels.iter().position(|s|s=="PACK"))};
    Some(Teaching{move_name:name,stage,rank,target,cursor,badge})
}
fn describe(t:&Teaching)->Vec<String> {
    let mut cues=vec![format!("objective:hm:{}:{}",t.move_name,t.stage)];
    if let (Some(target),Some(cursor))=(t.target,t.cursor){
        cues.push(format!("objective:hm:cursor:{}",if target<cursor{"north"}else if target>cursor{"south"}else{"aligned"}));
    }
    cues
}
// A healing trip has no Pokedex or Pokegear task. Keep purposeful item,
// party, dialogue and HM interactions outside this return-to-field potential.
fn recovery_menu_potential(v:&Value)->Option<f32> {
    if desired(v).is_some() || !crate::curriculum::needs_healing(v) || crate::curriculum::goal(v).is_none()
        || v.pointer("/status/screen")?.as_str()? != "overworld" {return None;}
    if !matches!(v.pointer("/map_info/name")?.as_str()?,"VioletGym"|"VioletCity"|"VioletPokecenter1F"|"AzaleaGym"|"AzaleaTown"|"AzaleaPokecenter1F"|"GoldenrodGym"|"GoldenrodCity"|"GoldenrodPokecenter1F"|"EcruteakGym"|"EcruteakCity"|"EcruteakPokecenter1F"){return None;}
    if v.pointer("/observe/visible_dialogue").and_then(Value::as_str).is_some_and(|s|!s.trim().is_empty()){return None;}
    match v.pointer("/observe/menus")?.as_array()?.last() {
        None=>Some(0.0),
        Some(m)=>match m["kind"].as_str()? {"start"=>Some(-0.2),"pokedex"|"pokegear"|"options"|"trainer_card"=>Some(-0.6),_=>None},
    }
}
pub(crate) fn cues(v:&Value)->Vec<String> {
    if crate::curriculum::goal(v)==Some(crate::curriculum::Goal::CutTree) {
        if let Some(menu)=v.pointer("/observe/menus").and_then(Value::as_array).and_then(|ms|ms.last()) {
            if menu["kind"]=="yes_no" && v.pointer("/observe/visible_dialogue").and_then(Value::as_str).is_some_and(|s|s.to_uppercase().contains("CUT")) {
                return vec!["objective:field:CUT:confirm".into(),format!("objective:menu:{}",if menu["selected"].as_u64()==Some(0){"aligned"}else{"north"})];
            }
        }
    }
    if let Some((cues,_))=withdrawal(v){return cues.split('|').map(str::to_owned).collect();}
    if recovery_menu_potential(v).is_some_and(|p|p<0.0){return vec!["objective:recovery:return_to_field".into()];}
    let missing=HMS.iter().filter(|(item,name,_)|!known(v,name)&&incompatible(v,item))
        .map(|(_,name,_)|format!("objective:hm:{name}:acquire_compatible_pokemon")).collect::<Vec<_>>();
    let Some(t)=teaching(v)else{return missing;};let mut cues=describe(&t);
    cues.extend(missing);
    if let Some(cue)=crate::pocket_objectives::cue(v,"TM/HM"){cues.push(cue);}
    if v.pointer(&format!("/status/badges/johto/{}",t.badge)).and_then(Value::as_bool)!=Some(true){cues.push(format!("objective:hm:field_use_requires_badge:{}",t.badge+1));}
    cues
}
pub(crate) fn context(v:&Value)->Option<String>{if let Some((cues,_))=withdrawal(v){return Some(cues);} teaching(v).map(|_|cues(v).join("|"))}
pub(crate) fn feedback(before:&Value,after:&Value)->f32 {
    if let Some(progress)=withdrawal_feedback(before,after){return progress;}
    if let (Some(a),Some(b))=(recovery_menu_potential(before),recovery_menu_potential(after)){return b-a;}
    let Some(a)=teaching(before)else{return 0.0;};
    if known(after,a.move_name){return 1.5;}
    if [before,after].iter().any(|v|v.pointer("/observe/visible_dialogue").and_then(Value::as_str).is_some_and(|s|!s.trim().is_empty())) {return 0.0;}
    let Some(b)=teaching(after).filter(|b|b.move_name==a.move_name)else{return 0.0;};
    // Do not invent cursor progress across unknown pocket layouts. Signed stage
    // differences cancel on reopening loops; merely staying aligned pays zero.
    let potential=|t:&Teaching|t.rank-match(t.target,t.cursor){(Some(a),Some(b))=>0.1*a.abs_diff(b).min(4) as f32,_=>0.0};
    let pocket=|v:&Value|crate::pocket_objectives::goal(v,"TM/HM").map_or(0.0,|(distance,_)|0.1*distance as f32);
    0.2*((potential(&b)-pocket(after))-(potential(&a)-pocket(before)))
}
#[cfg(test)] mod tests {
    use super::*;use serde_json::json;
    fn state(rows:Value)->Value{json!({"status":{"screen":"overworld","badges":{"johto":vec![false;8]},"party":[{"is_egg":false,"moves":[{"name":"TACKLE"}]}]},"reward_state":{"machines":["HM_CUT"]},"observe":{"menus":[{"kind":"pack","entries":rows}]}})}
    #[test] fn egg_space_uses_protected_deposit_then_exits_without_withdrawing(){
        let mut field=state(json!([]));field["map_info"]=json!({"name":"VioletPokecenter1F"});
        field["reward_state"]=json!({"machines":[],"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]});field["status"]["badges"]["johto"][0]=json!(true);
        field["status"]["party"]=json!((0..6).map(|i|json!({"level":i+1,"hp":10,"max_hp":10,"moves":[{"name":if i==0{"CUT"}else{"TACKLE"}}]})).collect::<Vec<_>>());field["observe"]["menus"]=json!([]);
        assert_eq!(crate::curriculum::goal(&field),Some(crate::curriculum::Goal::Pc));assert_eq!(deposit_candidate(&field),Some(1));
        let mut pc=field.clone();pc["observe"]["menus"]=json!([{"kind":"pc","surface":"storage_actions","box_index":0,"box_counts":[{"box":0,"count":3,"capacity":20}],"entries":[">WITHDRAW <PK><MN>","DEPOSIT <PK><MN>","CHANGE BOX","SEE YA!"]}]);
        assert!(context(&pc).unwrap().contains("TOGEPI:make_party_space"));assert!(context(&pc).unwrap().contains("cursor:south"));
        let mut space=pc.clone();space["status"]["party"].as_array_mut().unwrap().remove(1);
        assert!(feedback(&pc,&space)>0.0);assert!((feedback(&pc,&space)+feedback(&space,&pc)).abs()<1e-6);
        assert!(matches!(crate::curriculum::goal(&space),Some(crate::curriculum::Goal::Talk("ElmsAide",_))));
        let mut outside=space.clone();outside["observe"]["menus"]=json!([]);
        assert!((feedback(&field,&pc)+feedback(&pc,&space)+feedback(&space,&outside)-0.5).abs()<1e-6);
        space["observe"]["menus"][0]["surface"]=json!("pokemon_actions");space["observe"]["menus"][0]["selected_slot"]=json!(0);space["observe"]["menus"][0]["entries"]=json!([">WITHDRAW","STATS","RELEASE","CANCEL"]);
        assert!(!context(&space).unwrap().contains("withdraw_candidate"));
        space["observe"]["menus"][0]["surface"]=json!("hub");space["observe"]["menus"][0]["entries"]=json!(["SOMEONE'S PC",">TURN OFF"]);
        assert!(context(&space).unwrap().contains("return_to_aide"));assert!(context(&space).unwrap().contains("cursor:aligned"));
        let mut injured=field.clone();injured["status"]["party"][0]["hp"]=json!(1);assert!(matches!(crate::curriculum::goal(&injured),Some(crate::curriculum::Goal::Talk("NURSE",_))));
        let mut dex=injured.clone();dex["observe"]["menus"]=json!([{"kind":"pokedex","entries":[">153 -----"]}]);
        assert!(feedback(&injured,&dex)<0.0);assert!((feedback(&injured,&dex)+feedback(&dex,&injured)).abs()<1e-6);
    }
    #[test] fn teaching_starts_before_pack_and_yields_to_dialogue_or_missing_capability(){
        let mut v=state(json!([]));v["observe"]["menus"]=json!([]);
        assert_eq!(teaching_target(&v),Some("CUT"));assert!(context(&v).unwrap().contains("open_pack"));
        v["observe"]["visible_dialogue"]=json!("Here is the HM.");assert_eq!(teaching_target(&v),None);
        v["observe"]["visible_dialogue"]=Value::Null;
        v["map_info"]=json!({"hm_compatibility":{"HM_CUT":[{"able":false}]}});assert_eq!(teaching_target(&v),None);
        v["map_info"]["hm_compatibility"]["HM_CUT"][0]["able"]=json!(true);
        v["status"]["party"][0]["moves"]=json!([{"name":"CUT"}]);assert_eq!(teaching_target(&v),None);
    }
    #[test] fn healing_detours_reward_return_without_menu_farming_or_hm_interference() {
        let mut field=json!({"status":{"screen":"overworld","party":[{"hp":15,"max_hp":43}]},"map_info":{"name":"VioletGym"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"],"machines":[]},"observe":{"menus":[]}});
        let mut start=field.clone();start["observe"]["menus"]=json!([{"kind":"start","entries":[">POKEDEX","PACK"]}]);
        let mut dex=field.clone();dex["observe"]["menus"]=json!([{"kind":"pokedex","entries":[">153 -----"]}]);
        assert!(feedback(&dex,&start)>0.0);assert!(feedback(&start,&field)>0.0);
        assert!((feedback(&field,&start)+feedback(&start,&dex)+feedback(&dex,&start)+feedback(&start,&field)).abs()<1e-6);
        assert_eq!(feedback(&dex,&dex),0.0);
        assert!(cues(&dex).contains(&"objective:recovery:return_to_field".into()));
        for kind in ["pack","party","yes_no"] {let mut useful=field.clone();useful["observe"]["menus"]=json!([{"kind":kind}]);assert!(recovery_menu_potential(&useful).is_none());}
        field["reward_state"]["machines"]=json!(["HM_CUT"]);assert!(recovery_menu_potential(&field).is_none());
        dex["status"]["party"][0]["hp"]=json!(43);assert!(recovery_menu_potential(&dex).is_none());
    }
    #[test] fn hm_space_deposit_preserves_unique_field_moves_and_a_healthy_battler() {
        let mut v=state(json!([]));
        v["status"]["party"]=json!([
            {"level":30,"hp":90,"moves":[{"name":"TACKLE"}]},
            {"level":2,"hp":10,"moves":[{"name":"SURF"}]},
            {"level":3,"hp":10,"moves":[{"name":"CUT"}]},
            {"level":4,"hp":10,"moves":[{"name":"TACKLE"}],"item":"FLOWER_MAIL"},
            {"level":5,"hp":10,"moves":[{"name":"TACKLE"}]},
            {"level":6,"hp":10,"moves":[{"name":"TACKLE"}]}]);
        assert_eq!(deposit_candidate(&v),Some(4));
        v["status"]["party"][5]["is_egg"]=json!(true);assert_eq!(deposit_candidate(&v),Some(5));
        for p in v["status"]["party"].as_array_mut().unwrap(){p["hp"]=json!(0);}
        v["status"]["party"][4]["hp"]=json!(10);
        assert_ne!(deposit_candidate(&v),Some(4));
    }
    #[test] fn boxed_hm_withdrawal_uses_identity_and_cancels_menu_round_trips() {
        let mut v=state(json!([]));v["map_info"]=json!({"hm_compatibility":{"HM_CUT":[{"able":false}]}});
        v["reward_state"]["boxed_hm_candidates"]=json!({"HM_CUT":{"box":2,"position":4,"species":"PARAS"}});
        let field=v.clone();
        v["observe"]["menus"]=json!([{"kind":"pc","surface":"storage_actions","box_index":0,"entries":[">WITHDRAW <PK><MN>","DEPOSIT <PK><MN>","CHANGE BOX","SEE YA!"]}]);
        assert!(context(&v).unwrap().contains("cursor:south"));
        assert!((feedback(&field,&v)+feedback(&v,&field)).abs()<1e-6);
        v["observe"]["menus"][0]=json!({"kind":"pc","surface":"withdraw","box_index":2,"selected_slot":3,"entries":["BOX3",">SAME NAME","SAME NAME","CANCEL"]});
        assert!(context(&v).unwrap().contains("cursor:south"));
        v["observe"]["menus"][0]=json!({"kind":"pc","surface":"pokemon_actions","box_index":2,"selected_slot":4,"entries":[">WITHDRAW","STATS","RELEASE","CANCEL"]});
        assert!(context(&v).unwrap().contains("withdraw_candidate"));
        v["observe"]["menus"][0]["selected_slot"]=json!(3);
        assert!(context(&v).unwrap().contains("return_to_storage"));
        v["observe"]["menus"][0]=json!({"kind":"pc","surface":"release_confirmation","box_index":2,"entries":[">YES","NO"]});
        assert!(context(&v).unwrap().contains("cancel_release"));
        v["observe"]["menus"][0]=json!({"kind":"pc","surface":"box_actions","box_index":0,"selected_box":1,"entries":[">SWITCH","NAME","PRINT","QUIT"]});
        assert!(context(&v).unwrap().contains("return_to_boxes"));
        v["observe"]["menus"][0]["selected_box"]=json!(2);
        assert!(context(&v).unwrap().contains("switch_box"));
        v["observe"]["menus"][0]=json!({"kind":"yes_no","purpose":"pc_change_box","target_box":2,"box_index":0,"entries":[">YES","NO"]});
        assert!(context(&v).unwrap().contains("confirm_box_save"));
        v["observe"]["menus"][0]["target_box"]=json!(1);
        assert!(context(&v).unwrap().contains("cancel_wrong_box_save"));
        v["observe"]["menus"][0]["purpose"]=json!("release");
        assert!(withdrawal(&v).is_none());
    }
    #[test] fn verified_incompatibility_skips_impossible_teaching_but_unknown_does_not() {
        let mut v=state(json!([]));
        v["map_info"]=json!({"hm_compatibility":{"HM_CUT":[{"slot":0,"able":false}]}});
        assert!(teaching(&v).is_none());
        assert!(cues(&v).contains(&"objective:hm:CUT:acquire_compatible_pokemon".into()));
        v["reward_state"]["machines"]=json!(["HM_CUT","HM_SURF"]);
        v["map_info"]["hm_compatibility"]["HM_SURF"]=json!([{"slot":0,"able":true}]);
        assert_eq!(teaching(&v).unwrap().move_name,"SURF");
        v["map_info"]["hm_compatibility"]["HM_CUT"][0]["able"]=Value::Null;
        assert_eq!(teaching(&v).unwrap().move_name,"CUT");
        v["map_info"]["hm_compatibility"]["HM_CUT"]=json!([]);
        assert_eq!(teaching(&v).unwrap().move_name,"CUT");
    }
    #[test] fn compatibility_never_confuses_not_able_with_able(){let v=state(json!(["ITEM HM01 x01",">CHIKORITA NOT ABLE","TOTODILE ABLE"]));let t=teaching(&v).unwrap();assert_eq!(t.stage,"choose_compatible");assert_eq!(t.target,Some(2));assert!(cues(&v).iter().any(|s|s.contains("requires_badge:2")));}
    #[test] fn already_known_moves_and_unowned_machines_are_not_taught(){let mut v=state(json!([]));v["status"]["party"][0]["moves"]=json!([{"name":"CUT"}]);assert!(teaching(&v).is_none());v["status"]["party"][0]["moves"]=json!([]);v["reward_state"]["machines"]=json!([]);assert!(teaching(&v).is_none());}
    #[test] fn teaching_menu_loop_has_zero_net_reward(){let a=state(json!(["POCKET: TM/HM",">HM01 x01","CANCEL"]));let b=state(json!(["ACTION HM01 x01",">USE","CANCEL"]));assert!(feedback(&a,&b)>0.0);assert!((feedback(&a,&b)+feedback(&b,&a)).abs()<1e-6);assert_eq!(feedback(&b,&b),0.0);}
    #[test] fn hm_pocket_progress_uses_wrapping_tabs_and_cancels_on_return(){
        let mut a=state(json!(["POCKET: ITEMS",">POTION x1","CANCEL"]));
        a["observe"]["menus"][0]["pack_pockets"]=json!(["Items","Balls","Key","TM/HM"]);
        a["observe"]["menus"][0]["pack_pocket"]=json!("Items");
        let mut b=a.clone();b["observe"]["menus"][0]["pack_pocket"]=json!("TM/HM");b["observe"]["menus"][0]["entries"]=json!(["POCKET: TM/HM",">HM01 x1","CANCEL"]);
        assert!(cues(&a).contains(&"objective:pocket:TM/HM:west".into()));
        assert!(feedback(&a,&b)>0.0);assert!((feedback(&a,&b)+feedback(&b,&a)).abs()<1e-6);
        assert_ne!(context(&a),context(&b));
    }
    #[test] fn replacing_a_move_preserves_existing_hms(){let v=state(json!(["ITEM HM01 x01","CHOOSE A MOVE TO FORGET",">SURF","TACKLE","CANCEL"]));assert_eq!(teaching(&v).unwrap().target,Some(3));let mut after=v.clone();after["status"]["party"][0]["moves"]=json!([{"name":"CUT"}]);assert_eq!(feedback(&v,&after),1.5);}
}
