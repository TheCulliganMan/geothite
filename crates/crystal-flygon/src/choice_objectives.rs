//! Visible story choices have distinct meanings, even with identical YES/NO rows.
use serde_json::Value;
fn text<'a>(v:&'a Value,path:&str)->&'a str {v.pointer(path).and_then(Value::as_str).unwrap_or("")}
fn menu(v:&Value)->Option<&Value>{v.pointer("/observe/menus")?.as_array()?.last().filter(|m|m["kind"]=="yes_no")}
pub(crate) fn context(v:&Value)->Option<String>{
    menu(v)?;
    let question=text(v,"/observe/visible_dialogue").split_whitespace().collect::<Vec<_>>().join(" ");
    Some(format!("choice:{}:{}:{}",text(v,"/map_info/name"),text(v,"/reward_state/last_talked_object"),question.chars().take(180).collect::<String>()))
}
fn yes_cursor(v:&Value)->Option<usize>{
    let m=menu(v)?;let rows=m["entries"].as_array()?;
    if rows.len()!=2 || rows[0].as_str()?.trim().trim_start_matches('>').trim()!="YES" || rows[1].as_str()?.trim().trim_start_matches('>').trim()!="NO" {return None;}
    rows.iter().position(|s|s.as_str().is_some_and(|s|s.trim().starts_with('>')))
        .or_else(||m["selected"].as_u64().and_then(|n|(n<2).then_some(n as usize)))
}
fn starter(v:&Value)->Option<usize>{
    if text(v,"/map_info/name")!="ElmsLab" || !text(v,"/reward_state/last_talked_object").contains("POKE_BALL") {return None;}
    if !v.pointer("/status/party")?.as_array()?.is_empty(){return None;}
    yes_cursor(v)
}
fn healing(v:&Value)->Option<usize>{
    let question=text(v,"/observe/visible_dialogue").split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    if !text(v,"/map_info/name").contains("Pokecenter") || !text(v,"/reward_state/last_talked_object").contains("NURSE")
        || !question.contains("shall we heal your") || !crate::curriculum::needs_healing(v) {return None;}
    yes_cursor(v)
}
fn togepi(v:&Value)->Option<usize>{
    if text(v,"/map_info/name")!="VioletPokecenter1F" || !text(v,"/reward_state/last_talked_object").contains("ELMS_AIDE")
        || !crate::curriculum::pending_togepi(v) || v.pointer("/status/party")?.as_array()?.len()>=6 {return None;}
    let q=text(v,"/observe/visible_dialogue").split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    if !(q.contains("take the")&&q.contains("egg?")&&(q.contains("would you")||q.contains("will you"))){return None;}
    yes_cursor(v)
}
fn medicine(v:&Value)->Option<usize>{
    if text(v,"/map_info/name")!="OlivineLighthouse6F"
        || !text(v,"/reward_state/last_talked_object").contains("JASMINE")
        || !v.pointer("/reward_state/items")?.as_array()?.iter().any(|i|i["id"]=="SECRETPOTION"&&i["quantity"].as_u64().unwrap_or(0)>0)
        || v.pointer("/reward_state/event_flags").and_then(Value::as_array).is_some_and(|a|a.iter().any(|f|f=="EVENT_JASMINE_RETURNED_TO_GYM")){return None;}
    let q=text(v,"/observe/visible_dialogue").split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    if !q.contains("will that medicine cure amphy?"){return None;}
    yes_cursor(v)
}
fn field_move(v:&Value)->Option<(usize,&str)>{
    if text(v,"/status/screen")!="overworld" {return None;}
    let m=menu(v)?;
    if m["purpose"]!="field_move" {return None;}
    let name=m["field_move"].as_str()?;
    let badge=match name {"CUT"=>1,"SURF"=>3,"WHIRLPOOL"=>6,"WATERFALL"=>7,_=>return None};
    if v.pointer(&format!("/status/badges/johto/{badge}")).and_then(Value::as_bool)!=Some(true)
        || !crate::field_objectives::known(v,name){return None;}
    Some((yes_cursor(v)?,name))
}
pub(crate) fn cues(v:&Value)->Vec<String>{
    if let Some((cursor,name))=field_move(v){return vec![format!("objective:hm:use:{name}"),format!("objective:menu:{}",if cursor==0{"aligned"}else{"north"})];}
    let offer=medicine(v).map(|i|(i,"give_amphy_medicine")).or_else(||starter(v).map(|i|(i,"accept_starter"))).or_else(||healing(v).map(|i|(i,"heal_party"))).or_else(||togepi(v).map(|i|(i,"accept_togepi_egg")));
    offer.map(|(cursor,goal)|vec![format!("objective:story:{goal}"),format!("objective:menu:{}",if cursor==0{"aligned"}else{"north"})]).unwrap_or_default()
}
pub(crate) fn feedback(before:&Value,after:&Value,button:&str)->f32{
    if let Some((cursor,name))=field_move(before){
        if let Some((next,other))=field_move(after){return if name==other{0.1*(cursor as f32-next as f32)}else{0.0};}
        if menu(after).is_none() && (button=="b" || (button=="a" && cursor==1)){return -0.3;}
        return 0.0; // The observed field action, not pressing YES, earns completion.
    }

    if let Some(cursor)=medicine(before){
        if let Some(next)=medicine(after){return 0.1*(cursor as f32-next as f32);}
        // Only the actual story event earns completion. Waiting on a page or
        // accepting the handoff is not a repeatable source of reward.
        return 0.0;
    }
    if let Some(cursor)=togepi(before){
        if let Some(next)=togepi(after){return 0.1*(cursor as f32-next as f32);}
        if (button=="b" || button=="a"&&cursor==1) && menu(after).is_none()
            && text(before,"/map_info/name")==text(after,"/map_info/name") && crate::curriculum::pending_togepi(after){return -0.3;}
        return 0.0; // Only the observed Egg/story event pays completion credit.
    }
    if let Some(cursor)=healing(before){
        if let Some(next)=healing(after){return 0.1*(cursor as f32-next as f32);}
        if (button=="b" || button=="a"&&cursor==1) && menu(after).is_none()
            && text(before,"/map_info/name")==text(after,"/map_info/name") && crate::curriculum::needs_healing(after){return -0.3;}
        return 0.0; // The actual restoration is rewarded by the existing ledger.
    }
    let Some(cursor)=starter(before)else{return 0.0;};
    if let Some(next)=starter(after){return 0.1*(cursor as f32-next as f32);}
    // Only charge a visible decline transition, never an unchanged input or a
    // required nickname/other question after the Pokemon has been received.
    let declined=button=="b" || (button=="a" && cursor==1);
    if declined && text(before,"/map_info/name")==text(after,"/map_info/name")
        && after.pointer("/status/party").and_then(Value::as_array).is_some_and(|p|p.is_empty())
        && menu(after).is_none(){return -0.3;}
    0.0 // Receiving the actual starter is rewarded by the story event ledger.
}
#[cfg(test)]mod tests{
 use super::*;use serde_json::json;
 fn offer()->Value{json!({"map_info":{"name":"ElmsLab"},"status":{"party":[]},"reward_state":{"last_talked_object":"ELMSLAB_POKE_BALL1"},"observe":{"visible_dialogue":"CYNDAQUIL, the fire POKéMON?","menus":[{"kind":"yes_no","selected":0,"entries":[">YES","NO"]}]}})}
 #[test]fn medicine_choice_requires_the_item_speaker_question_and_unfinished_event(){
  let mut v=offer();v["map_info"]["name"]=json!("OlivineLighthouse6F");v["reward_state"]=json!({"last_talked_object":"OLIVINELIGHTHOUSE6F_JASMINE","items":[{"id":"SECRETPOTION","quantity":1}],"event_flags":[]});v["observe"]["visible_dialogue"]=json!("JASMINE: …Will
that medicine cure
AMPHY?");
  assert!(cues(&v).contains(&"objective:story:give_amphy_medicine".into()));
  let mut no=v.clone();no["observe"]["menus"][0]["entries"]=json!(["YES",">NO"]);
  assert!(feedback(&no,&v,"up")>0.0);assert_eq!(feedback(&no,&v,"up")+feedback(&v,&no,"down"),0.0);
  let mut done=v.clone();done["observe"]["menus"]=json!([]);assert_eq!(feedback(&v,&done,"a"),0.0);
  for path in ["/reward_state/last_talked_object","/observe/visible_dialogue"]{let mut wrong=v.clone();*wrong.pointer_mut(path).unwrap()=json!("unrelated");assert!(cues(&wrong).is_empty());}
  v["reward_state"]["items"][0]["quantity"]=json!(0);assert!(cues(&v).is_empty());
  v["reward_state"]["items"][0]["quantity"]=json!(1);v["reward_state"]["event_flags"]=json!(["EVENT_JASMINE_RETURNED_TO_GYM"]);assert!(cues(&v).is_empty());
 }
 #[test]fn field_confirmation_requires_visible_purpose_move_and_badge(){
  for (name,badge) in [("CUT",1),("SURF",3),("WHIRLPOOL",6),("WATERFALL",7)]{
   let mut v=offer();v["status"]=json!({"screen":"overworld","party":[{"is_egg":false,"moves":[{"name":name}]}],"badges":{"johto":vec![false;8]}});
   v["observe"]["menus"][0]["purpose"]=json!("field_move");v["observe"]["menus"][0]["field_move"]=json!(name);
   assert!(field_move(&v).is_none());v["status"]["badges"]["johto"][badge]=json!(true);
   assert!(cues(&v).contains(&format!("objective:hm:use:{name}")));
   let mut no=v.clone();no["observe"]["menus"][0]["entries"]=json!(["YES",">NO"]);
   assert!(feedback(&no,&v,"up")>0.0);assert_eq!(feedback(&no,&v,"up")+feedback(&v,&no,"down"),0.0);
   let mut done=v.clone();done["observe"]["menus"]=json!([]);
   assert_eq!(feedback(&v,&done,"a"),0.0);assert!(feedback(&v,&done,"b")<0.0);
   assert_eq!(feedback(&v,&v,"b"),0.0);
   v["observe"]["menus"][0]["purpose"]=json!("unrelated");assert!(field_move(&v).is_none());
  }
 }
 #[test]fn togepi_offers_require_room_and_match_both_authored_questions(){
  let mut v=offer();v["map_info"]["name"]=json!("VioletPokecenter1F");v["reward_state"]=json!({"last_talked_object":"VIOLETPOKECENTER1F_ELMS_AIDE","event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]});
  v["status"]=json!({"party":[{"nickname":"QUILAVA"}],"badges":{"johto":[true,false,false,false,false,false,false,false]}});
  for q in ["Would you take the\nPOKéMON EGG?","CHRIS, will you\ntake the EGG?"]{
   v["observe"]["visible_dialogue"]=json!(q);assert!(cues(&v).contains(&"objective:story:accept_togepi_egg".into()));
  }
  let mut no=v.clone();no["observe"]["menus"][0]["entries"]=json!(["YES",">NO"]);
  assert!(feedback(&no,&v,"up")>0.0);assert_eq!(feedback(&no,&v,"up")+feedback(&v,&no,"down"),0.0);
  let mut closed=v.clone();closed["observe"]["menus"]=json!([]);assert_eq!(feedback(&v,&v,"b"),0.0);assert!(feedback(&v,&closed,"b")<0.0);
  closed["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_GOT_TOGEPI_EGG_FROM_ELMS_AIDE"));assert_eq!(feedback(&v,&closed,"a"),0.0);
  v["status"]["party"]=json!(vec![json!({});6]);assert!(cues(&v).is_empty());
  v["status"]["party"]=json!([{}]);v["observe"]["visible_dialogue"]=json!("Would you like to trade?");assert!(cues(&v).is_empty());
 }
 #[test]fn nurse_offer_requires_injury_and_the_actual_healing_question(){
  let mut v=offer();v["map_info"]["name"]=json!("VioletPokecenter1F");v["reward_state"]["last_talked_object"]=json!("VIOLETPOKECENTER1F_NURSE");
  v["status"]["party"]=json!([{"hp":19,"max_hp":36}]);v["observe"]["visible_dialogue"]=json!("Shall we heal your\n#MON?");
  assert!(cues(&v).contains(&"objective:story:heal_party".into()));
  let mut after=v.clone();after["observe"]["menus"]=json!([]);assert!(feedback(&v,&after,"b")<0.0);assert_eq!(feedback(&v,&v,"b"),0.0);
  after["status"]["party"][0]["hp"]=json!(36);assert_eq!(feedback(&v,&after,"a"),0.0);
  v["status"]["party"][0]["hp"]=json!(36);assert!(cues(&v).is_empty());
  v["status"]["party"][0]["hp"]=json!(19);v["observe"]["visible_dialogue"]=json!("Would you like to trade?");assert!(cues(&v).is_empty());
 }
 #[test]fn question_contexts_do_not_share_clock_and_starter_choices(){let a=offer();let mut b=a.clone();b["map_info"]["name"]=json!("PlayersHouse1F");b["observe"]["visible_dialogue"]=json!("Is that OK?");assert_ne!(context(&a),context(&b));assert!(cues(&b).is_empty());}
 #[test]fn declines_cost_and_cursor_round_trips_do_not_pay(){let a=offer();let mut no=a.clone();no["observe"]["menus"][0]["entries"]=json!(["YES",">NO"]);let f=feedback(&no,&a,"up");assert!(f>0.0);assert_eq!(f+feedback(&a,&no,"down"),0.0);let mut closed=a.clone();closed["observe"]["menus"]=json!([]);assert!(feedback(&a,&closed,"b")<0.0);assert_eq!(feedback(&a,&a,"b"),0.0);assert_eq!(feedback(&a,&closed,"a"),0.0);}
 #[test]fn nickname_and_unknown_choices_are_not_starter_offers(){let mut v=offer();v["status"]["party"]=json!([{"nickname":"CYNDAQUIL"}]);assert!(cues(&v).is_empty());let mut v=offer();v["reward_state"]["last_talked_object"]=json!("ELMSLAB_ELM");assert!(cues(&v).is_empty());}
}
