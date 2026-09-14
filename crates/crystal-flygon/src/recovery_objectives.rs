//! Visible keyboard and forced replacement goals. Cursor loops have zero net value.
use serde_json::Value;
fn menu(v: &Value) -> Option<&Value> { v.pointer("/observe/menus")?.as_array()?.last() }
fn active(v: &Value) -> Option<u64> { v.pointer("/reward_state/battle/active_player")?.as_u64() }
fn fainted(v: &Value) -> Option<u64> {
    if v.pointer("/status/screen")?.as_str()? != "battle" { return None; }
    let slot=active(v)?;
    v.pointer("/status/party")?.as_array()?.iter().any(|p|p["slot"].as_u64()==Some(slot)&&p["hp"].as_u64()==Some(0)).then_some(slot)
}
fn usable(p: &Value) -> bool { p["is_egg"]!=true && p["hp"].as_u64().is_some_and(|hp|hp>0) }
// Match the engine's wrapping 9x5 keyboard and three grouped bottom keys.
fn next(row:usize,col:usize,dir:usize)->(usize,usize){
    match dir {0=>((row+4)%5,col),1=>((row+1)%5,col),2=>(row,if row==4{((col/3+2)%3)*3}else{(col+8)%9}),_=>(row,if row==4{((col/3+1)%3)*3}else{(col+1)%9})}
}
fn keyboard_distance(row:usize,col:usize,filled:bool)->usize {
    let mut queue=std::collections::VecDeque::from([(row,col,0)]);let mut seen=[[false;9];5];
    while let Some((r,c,d))=queue.pop_front(){
        if (filled && r==4 && c>=6)||(!filled && r<2){return d;}
        if seen[r][c]{continue;}seen[r][c]=true;
        for dir in 0..4 {let (nr,nc)=next(r,c,dir);queue.push_back((nr,nc,d+1));}
    } 0
}
fn goal(v:&Value)->Option<(String,f32,Vec<String>)>{
    let m=menu(v)?;
    if m["kind"]=="name_choices" {
        let rows=m["options"].as_array()?;
        if rows.first()?.as_str()?.trim().to_uppercase()!="NEW NAME" {return None;}
        // Prefer an offered preset over reopening the custom keyboard.
        let target=rows.iter().position(|r|r.as_str().is_some_and(|s| !matches!(s.trim().to_uppercase().as_str(), "NEW NAME"|"NEW NAME?")))?;
        let cursor=m["selected"].as_u64()? as usize;
        if cursor>=rows.len(){return None;}
        let g=crate::objectives::MenuGoal{stage:"choose_name",target,cursor,columns:1};
        return Some((format!("name_choices:{rows:?}"),-0.1*g.distance() as f32,g.cues().into_iter().map(|s|s.replace("objective:capture:","objective:naming:")).collect()));
    }
    if m["kind"]=="name_input" {
        let r=m["cursor_row"].as_u64()? as usize;let c=m["cursor_column"].as_u64()? as usize;
        if r>=5||c>=9||m["keyboard"].as_array()?.len()!=5{return None;}
        let filled=!m["value"].as_str()?.trim().is_empty();
        let d=keyboard_distance(r,c,filled);
        let stage=if filled{"finish_name"}else{"enter_name"};
        let mut cues=vec![format!("objective:naming:{stage}")];
        if d==0{cues.push("objective:menu:aligned".into());}
        for (dir,label) in ["north","south","west","east"].iter().enumerate(){let(nr,nc)=next(r,c,dir);if keyboard_distance(nr,nc,filled)<d{cues.push(format!("objective:menu:{label}"));}}
        return Some((format!("name:{}",m["label"]),if filled{1.0}else{0.0}-0.1*d as f32,cues));
    }
    let old=fainted(v)?;
    let party=v.pointer("/status/party")?.as_array()?;
    let target=party.iter().position(usable)?;
    let rows=m["entries"].as_array()?;
    let cursor=rows.iter().position(|r|r.as_str().is_some_and(|s|s.trim().starts_with('>')))?;
    let (stage,target,base)=match m["surface"].as_str()? {
        "party"=>("select_replacement",target,1.0),
        "faint_prompt"=>("continue_battle",rows.iter().position(|r|r.as_str().is_some_and(|s|s.trim().trim_start_matches('>').trim()=="YES"))?,0.0),
        _=>return None,
    };
    if target>=rows.len(){return None;}
    let g=crate::objectives::MenuGoal{stage,target,cursor,columns:1};
    Some((format!("replacement:{old}"),base-0.1*g.distance() as f32,g.cues().into_iter().map(|s|s.replace("objective:capture:","objective:battle:")).collect()))
}
pub(crate) fn cues(v:&Value)->Option<Vec<String>>{Some(goal(v)?.2)}
pub(crate) fn feedback(before:&Value,after:&Value)->Option<f32>{
    if let Some(old)=fainted(before){
        if after.pointer("/status/screen").and_then(Value::as_str)==Some("battle") {
            if let Some(new)=active(after).filter(|new|*new!=old){
                let healthy=after.pointer("/status/party").and_then(Value::as_array).is_some_and(|ps|ps.iter().any(|p|p["slot"].as_u64()==Some(new)&&usable(p)));
                if healthy{return Some(1.0);}
            }
        }
    }
    let (id,a,_)=goal(before)?;
    Some(match goal(after){Some((other,b,_)) if id==other=>0.3*(b-a),_=>0.0})
}
#[cfg(test)] mod tests {
    use super::*;use serde_json::json;
    #[test] fn keyboard_wraps_and_does_not_pay_for_typing_or_cursor_loops(){
        let mut a=json!({"observe":{"menus":[{"kind":"name_input","label":"NAME","value":"A","keyboard":["a","b","c","d","end"],"cursor_row":0,"cursor_column":8}]}});
        assert!(cues(&a).unwrap().contains(&"objective:menu:north".into()));
        let mut b=a.clone();b["observe"]["menus"][0]["cursor_row"]=json!(4);
        assert!(feedback(&a,&b).unwrap()>0.0);assert_eq!(feedback(&a,&b).unwrap()+feedback(&b,&a).unwrap(),0.0);
        a["observe"]["menus"][0]["value"]=json!("ABC");assert_eq!(feedback(&a,&a),Some(0.0));
        b=a.clone();b["observe"]["menus"][0]["value"]=json!("ABCD");assert_eq!(feedback(&a,&b),Some(0.0));
    }
    #[test] fn preset_choices_have_signed_progress_and_unknown_choices_are_ignored(){
        let a=json!({"observe":{"menus":[{"kind":"name_choices","options":["NEW NAME","CHRIS","MAT"],"selected":0}]}});
        let mut b=a.clone();b["observe"]["menus"][0]["selected"]=json!(1);
        assert!(feedback(&a,&b).unwrap()>0.0);
        assert_eq!(feedback(&a,&b).unwrap()+feedback(&b,&a).unwrap(),0.0);
        b["observe"]["menus"][0]["options"]=json!(["YES","NO"]);assert!(cues(&b).is_none());
    }
    #[test] fn replacement_skips_eggs_and_fainted_members_and_pays_actual_switch(){
        let a=json!({"status":{"screen":"battle","party":[{"slot":0,"hp":0},{"slot":1,"hp":5,"is_egg":true},{"slot":2,"hp":20}]},"reward_state":{"battle":{"active_player":0}},"observe":{"menus":[{"kind":"battle","surface":"party","entries":[">FAINTED","EGG","HEALTHY","CANCEL"]}]}});
        assert!(cues(&a).unwrap().contains(&"objective:menu:south".into()));
        let mut b=a.clone();b["observe"]["menus"][0]["entries"]=json!(["FAINTED","EGG",">HEALTHY","CANCEL"]);
        assert!(feedback(&a,&b).unwrap()>0.0);assert_eq!(feedback(&a,&b).unwrap()+feedback(&b,&a).unwrap(),0.0);
        b["reward_state"]["battle"]["active_player"]=json!(2);assert_eq!(feedback(&a,&b),Some(1.0));assert_eq!(feedback(&b,&b),None);
        b["reward_state"]["battle"]["active_player"]=json!(1);assert_ne!(feedback(&a,&b),Some(1.0));
        b["status"]["screen"]=json!("overworld");assert_ne!(feedback(&a,&b),Some(1.0));
    }
}
