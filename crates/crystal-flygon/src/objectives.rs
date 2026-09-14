//! Explicit interaction objectives inferred from visible battle/menu telemetry.
//! These describe desired outcomes; they never submit an input or alter the game.
use serde_json::Value;

fn text<'a>(v: &'a Value, path: &str) -> &'a str {
    v.pointer(path).and_then(Value::as_str).unwrap_or("")
}
fn entries(v: &Value) -> Vec<&str> {
    v.pointer("/observe/menus").and_then(Value::as_array).and_then(|m| m.last())
        .and_then(|m| m.get("entries").or_else(|| m.get("options")))
        .and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect()
}
fn label(s: &str) -> String {
    s.trim().trim_start_matches('>').trim().to_uppercase()
}
fn ball(s: &str) -> bool {
    let s = s.to_uppercase().replace('É', "E").replace([' ', '_'], "").replace('#', "POKE");
    ["POKEBALL", "GREATBALL", "ULTRABALL", "MASTERBALL", "FASTBALL", "LEVELBALL", "LUREBALL", "HEAVYBALL", "LOVEBALL", "FRIENDBALL", "MOONBALL", "PARKBALL"].iter().any(|b| s.starts_with(b))
}
/// An interpretable target and relative cursor position. No target button score.
#[derive(Debug, PartialEq)]
pub(crate) struct MenuGoal {
    pub stage: &'static str,
    pub target: usize,
    pub cursor: usize,
    pub columns: usize,
}
impl MenuGoal {
    pub fn distance(&self) -> usize {
        (self.target / self.columns).abs_diff(self.cursor / self.columns)
            + (self.target % self.columns).abs_diff(self.cursor % self.columns)
    }
    pub fn cues(&self) -> Vec<String> {
        let mut cues = vec![format!("objective:capture:{}", self.stage)];
        if self.target == self.cursor { cues.push("objective:menu:aligned".into()); }
        if self.target / self.columns < self.cursor / self.columns { cues.push("objective:menu:north".into()); }
        if self.target / self.columns > self.cursor / self.columns { cues.push("objective:menu:south".into()); }
        if self.target % self.columns < self.cursor % self.columns { cues.push("objective:menu:west".into()); }
        if self.target % self.columns > self.cursor % self.columns { cues.push("objective:menu:east".into()); }
        cues
    }
}
/// Capture only an uncaught wild species when actual inventory contains a ball.
/// The displayed battle header distinguishes wild encounters from trainers.
pub(crate) fn capture_species(v: &Value) -> Option<&str> {
    if text(v,"/status/screen") != "battle" { return None; }
    let overlay=text(v,"/observe/battle");
    if overlay.split_whitespace().next()? != "Wild" { return None; }
    let species=overlay.lines().find_map(|l| l.strip_prefix("Enemy "))?.split_whitespace().next()?;
    let needed_unowned=crate::field_objectives::capture_hm(v).is_some() && v.pointer("/reward_state/owned_species_counts").and_then(Value::as_object).is_some_and(|counts|counts.get(species).and_then(Value::as_u64).unwrap_or(0)==0);
    if !needed_unowned && v.pointer("/reward_state/caught_species").and_then(Value::as_array).is_some_and(|a| a.iter().any(|s| s.as_str()==Some(species))) { return None; }
    let has_ball=v.pointer("/reward_state/items").and_then(Value::as_array).is_some_and(|a| a.iter().any(|i| i["quantity"].as_u64().unwrap_or(0)>0 && i["id"].as_str().is_some_and(ball)));
    has_ball.then_some(species)
}
/// Penalize an observed voluntary escape only when a viable capture was abandoned.
/// Do not infer intent from battle disappearance alone (enemy escape, loss, etc.).
pub(crate) fn abandoned_capture(before:&Value,after:&Value,button:&str)->bool {
    if button!="a" || capture_species(before).is_none() || text(after,"/status/screen")!="overworld"
        || after.pointer("/reward_state/battle_result").and_then(Value::as_u64)!=Some(2) {return false;}
    let menu=before.pointer("/observe/menus").and_then(Value::as_array).and_then(|m|m.last());
    if !menu.is_some_and(|m|m["surface"]=="commands") || !entries(before).iter().any(|s|s.trim().starts_with('>')&&label(s)=="RUN") {return false;}
    let Some(active)=before.pointer("/reward_state/battle/active_player").and_then(Value::as_u64) else{return false;};
    before.pointer("/status/party").and_then(Value::as_array).and_then(|party|party.iter().find(|p|p["slot"].as_u64()==Some(active)))
        .is_some_and(|p|matches!((p["hp"].as_u64(),p["max_hp"].as_u64()),(Some(hp),Some(max)) if max>0&&hp>max/3))
}
pub(crate) fn capture_menu(v: &Value) -> Option<MenuGoal> {
    capture_species(v)?;
    let rows=entries(v);
    let labels:Vec<_>=rows.iter().map(|s| label(s)).collect();
    let cursor=rows.iter().position(|s| s.trim().starts_with('>'))?;
    let hp=v.pointer("/reward_state/battle/enemy_hp")?.as_u64()?;
    let _max=v.pointer("/reward_state/battle/enemy_max_hp")?.as_u64()?.max(1);
    if hp==0 { return None; }
    // Recover from the visible party-action submenu before pursuing a throw.
    if labels.iter().any(|s| s=="SWITCH") && labels.iter().any(|s| s=="STATS") {
        let active=v.pointer("/reward_state/battle/active_player")?.as_u64()?;
        if !v.pointer("/status/party")?.as_array()?.iter().any(|p|p["slot"].as_u64()==Some(active)&&p["hp"].as_u64().unwrap_or(0)>0) {return None;}
        return Some(MenuGoal{stage:"leave_party_actions",target:labels.iter().position(|s|s=="CANCEL")?,cursor,columns:1});
    }
    let menu=v.pointer("/observe/menus")?.as_array()?.last()?;
    if menu["surface"]=="party" {
        let active=v.pointer("/reward_state/battle/active_player")?.as_u64()?;
        let party=v.pointer("/status/party")?.as_array()?;
        let healthy=party.iter().filter(|p|p["is_egg"]!=true&&p["hp"].as_u64().unwrap_or(0)>0).collect::<Vec<_>>();
        if healthy.len()==1 && healthy[0]["slot"].as_u64()==Some(active) {
            return Some(MenuGoal{stage:"leave_party_actions",target:labels.iter().position(|s|s=="CANCEL")?,cursor,columns:1});
        }
    }
    // Real 2x2 main menu, not a vertical list. Move selection remains learned
    // while weakening; once ready to throw, leave the move list for Pack.
    if labels.len()==4 && labels.iter().any(|s| s=="FIGHT") && labels.iter().any(|s| s=="PACK") {
        let (stage,wanted)=if ready_to_throw(v) {("open_pack","PACK")} else {("weaken","FIGHT")};
        return Some(MenuGoal{stage,target:labels.iter().position(|s| s==wanted)?,cursor,columns:2});
    }
    if ready_to_throw(v) {
        if menu["surface"]=="moves" {
            return Some(MenuGoal{stage:"leave_moves",target:labels.iter().position(|s|s=="CANCEL")?,cursor,columns:1});
        }
        if menu["surface"]=="item_actions" && labels.first().is_some_and(|s|s.strip_prefix("ACTION ").is_some_and(ball)) {
            return Some(MenuGoal{stage:"throw_ball",target:labels.iter().position(|s|s=="USE")?,cursor,columns:1});
        }
        if let Some(target)=labels.iter().position(|s| ball(s)) {
            return Some(MenuGoal{stage:"select_ball",target,cursor,columns:1});
        }

    }
    None
}

// Prioritize a finite healing resource when the active battler is critically
// hurt. This describes visible menu goals, never presses a button.
fn healing_item(v: &Value) -> Option<&str> {
    if text(v,"/status/screen")!="battle" { return None; }
    let active=v.pointer("/reward_state/battle/active_player")?.as_u64()?;
    let p=v.pointer("/status/party")?.as_array()?.iter().find(|p|p["slot"].as_u64()==Some(active))?;
    let hp=p["hp"].as_u64()?;let max=p["max_hp"].as_u64()?;
    if hp==0 || max==0 || hp*3>max {return None;}
    let items=v.pointer("/reward_state/items")?.as_array()?;
    ["POTION","SUPER_POTION","HYPER_POTION","MAX_POTION","FULL_RESTORE","FRESH_WATER","SODA_POP","LEMONADE","MOOMOO_MILK"]
        .into_iter().find(|wanted|items.iter().any(|i|i["id"]==*wanted&&i["quantity"].as_u64().unwrap_or(0)>0))
}
fn healing_menu(v:&Value)->Option<MenuGoal> {
    let item=healing_item(v)?;
    let menu=v.pointer("/observe/menus")?.as_array()?.last()?;
    let rows=entries(v);let labels=rows.iter().map(|s|label(s)).collect::<Vec<_>>();
    let cursor=rows.iter().position(|s|s.trim().starts_with('>'))?;
    let (stage,target,columns)=match menu["surface"].as_str()? {
        "commands"=>("heal_open_pack",labels.iter().position(|s|s=="PACK")?,2),
        "moves"|"party"|"party_actions"=>("heal_return_to_commands",labels.iter().position(|s|s=="CANCEL")?,1),
        "items"=>{
            let name=item.replace('_',"");
            let target=labels.iter().position(|s|s.replace([' ','_'],"").starts_with(&name))?;
            ("heal_select_item",target,1)
        },
        "item_actions"=>{
            let selected=labels.first().and_then(|s|s.strip_prefix("ACTION "))
                .is_some_and(|s|s.replace([' ','_'],"").starts_with(&item.replace('_',"")));
            if selected {("heal_use_item",labels.iter().position(|s|s=="USE")?,1)}
            else {("heal_leave_wrong_item",labels.iter().position(|s|s=="QUIT"||s=="CANCEL")?,1)}
        },
        "item_target"=>{
            let active=v.pointer("/reward_state/battle/active_player")?.as_u64()?;
            let target=v.pointer("/status/party")?.as_array()?.iter().position(|p|p["slot"].as_u64()==Some(active))?;
            if target>=rows.len(){return None;}
            ("heal_target",target,1)
        },
        _=>return None,
    };
    Some(MenuGoal{stage,target,cursor,columns})
}
fn healing_potential(v:&Value)->Option<f32> {
    healing_item(v)?;
    if !text(v,"/observe/battle_message").is_empty(){return None;}
    if let Some(g)=healing_menu(v) {
        let stage=match g.stage {"heal_return_to_commands"=>-0.5,"heal_leave_wrong_item"=>0.3,"heal_open_pack"=>0.0,"heal_select_item"=>1.0,"heal_use_item"=>1.5,"heal_target"=>2.0,_=>0.0};
        return Some(stage-0.1*g.distance() as f32);
    }
    crate::pocket_objectives::goal(v,"Items").map(|(distance,_)|0.8-0.1*distance as f32)
}
fn healing_feedback(before:&Value,after:&Value)->Option<f32> {
    let item=healing_item(before)?;
    let active=before.pointer("/reward_state/battle/active_player")?.as_u64()?;
    let hp=|v:&Value|v.pointer("/status/party")?.as_array()?.iter().find(|p|p["slot"].as_u64()==Some(active))?["hp"].as_u64();
    let quantity=|v:&Value|v.pointer("/reward_state/items").and_then(Value::as_array).and_then(|is|is.iter().find(|i|i["id"]==item)).and_then(|i|i["quantity"].as_u64()).unwrap_or(0);
    if hp(after)>hp(before) && quantity(after)<quantity(before) {return Some(0.5);}
    Some(match (healing_potential(before),healing_potential(after)) {(Some(a),Some(b))=>0.3*(b-a),_=>0.0})
}

/// Guard only repeated trainer Pack openings; B and directional controls remain
/// available, as do item selection within an already-open Pack and wild captures.
pub(crate) fn trainer_pack_selected(v:&Value)->bool {
    text(v,"/status/screen")=="battle" && text(v,"/observe/battle").starts_with("Trainer ")
        && entries(v).iter().any(|s|label(s)=="FIGHT")
        && entries(v).iter().any(|s|s.trim().starts_with('>')&&label(s)=="PACK")
}

/// A ball cannot capture a trainer's Pokemon. Keep the Pack usable for healing
/// and cancellation, but do not spend inventory on this rejected action.
pub(crate) fn trainer_ball_selected(v: &Value) -> bool {
    text(v, "/status/screen") == "battle"
        && text(v, "/observe/battle").starts_with("Trainer ")
        && v.pointer("/observe/menus").and_then(Value::as_array).and_then(|m| m.last())
            .is_some_and(|m| m["surface"] == "balls")
        && entries(v).iter().any(|s| s.trim().starts_with('>') && ball(&label(s)))
}

// A trainer battle has a different objective from a capture. These cues
// describe menu progress and never replace the neural action selection.
fn trainer_menu(v: &Value) -> Option<MenuGoal> {
    if text(v,"/status/screen") != "battle" || !text(v,"/observe/battle").starts_with("Trainer ") { return None; }
    if v.pointer("/reward_state/battle/enemy_hp")?.as_u64()? == 0 { return None; }
    let active=v.pointer("/reward_state/battle/active_player")?.as_u64()?;
    let party=v.pointer("/status/party")?.as_array()?;
    if !party.iter().any(|p|p["slot"].as_u64()==Some(active)&&p["hp"].as_u64().unwrap_or(0)>0) {return None;}
    let menu=v.pointer("/observe/menus")?.as_array()?.last()?;
    let rows=entries(v);let labels:Vec<_>=rows.iter().map(|s|label(s)).collect();
    let cursor=rows.iter().position(|s|s.trim().starts_with('>'))?;
    let (stage,target,columns)=match menu["surface"].as_str()? {
        "commands"=>("fight",labels.iter().position(|s|s=="FIGHT")?,2),
        // Only cancel a voluntary party choice when there is no usable
        // alternative. A fainted active Pokemon and forced prompts are excluded.
        "party"|"party_actions" if party.iter().filter(|p|p["is_egg"]!=true&&p["hp"].as_u64().unwrap_or(0)>0).count()==1
            =>("leave_party",labels.iter().position(|s|s=="CANCEL")?,1),
        _=>return None,
    };
    Some(MenuGoal{stage,target,cursor,columns})
}
fn trainer_potential(v:&Value)->Option<f32> {
    if !text(v,"/observe/battle_message").is_empty(){return None;}
    if let Some(g)=trainer_menu(v){return Some(if g.stage=="fight"{0.0}else{-0.2}-0.1*g.distance() as f32);}
    let menu=v.pointer("/observe/menus")?.as_array()?.last()?;
    (menu["surface"]=="moves").then_some(1.0)
}
fn trainer_feedback(before:&Value,after:&Value)->f32 {
    if trainer_menu(before).is_none() && trainer_menu(after).is_none(){return 0.0;}
    if text(before,"/observe/battle") .split('\n').next()!=text(after,"/observe/battle").split('\n').next(){return 0.0;}
    match (trainer_potential(before),trainer_potential(after)){
        (Some(a),Some(b))=>0.3*(b-a),_=>0.0
    }
}

// Visible level disparity is a conservative knockout-risk heuristic, not a
// damage estimate. Do not insist on weakening a much lower-level target.
fn capture_level_risk(v: &Value) -> bool {
    let Some(active)=v.pointer("/reward_state/battle/active_player").and_then(Value::as_u64) else {return false;};
    let level=v.pointer("/status/party").and_then(Value::as_array)
        .and_then(|party|party.iter().find(|p|p["slot"].as_u64()==Some(active)))
        .and_then(|p|p["level"].as_u64());
    let enemy=text(v,"/observe/battle").lines().find_map(|line|line.strip_prefix("Enemy "))
        .and_then(|line|line.split_whitespace().nth(1))
        .and_then(|token|token.strip_prefix('L')).and_then(|level|level.parse::<u64>().ok());
    matches!((level,enemy),(Some(a),Some(b)) if b>0 && a>=b.saturating_mul(2) && a.saturating_sub(b)>=5)
}
fn ready_to_throw(v: &Value) -> bool {
    let hp=v.pointer("/reward_state/battle/enemy_hp").and_then(Value::as_u64).unwrap_or(0);
    let max=v.pointer("/reward_state/battle/enemy_max_hp").and_then(Value::as_u64).unwrap_or(0);
    hp>0 && max>0 && (hp<=max/2 || capture_level_risk(v))
}
fn pocket_cue(v: &Value) -> Option<String> {
    if healing_item(v).is_some() { return crate::pocket_objectives::cue(v,"Items"); }
    (capture_species(v).is_some() && ready_to_throw(v)).then(||crate::pocket_objectives::cue(v,"Balls")).flatten()
}
// Utility browsing does not advance a declared story objective. Keep its
// potential negative so entering and leaving the same page cannot farm reward.
fn unneeded_utility_menu(v: &Value) -> bool {
    text(v,"/status/screen")=="overworld" && crate::curriculum::goal(v).is_some()
        && v.pointer("/observe/menus").and_then(Value::as_array).and_then(|m|m.last()).is_some_and(|m|matches!(m["kind"].as_str(),Some("pokedex"|"options"|"trainer_card")) || (m["kind"]=="start" && crate::field_objectives::teaching_target(v).is_none()) || (m["kind"]=="pack" && idle_field_pack(v)))
}
// A ball-only field inventory has no current field interaction to complete.
// Require explicit inventory telemetry; unknown items and key items stay usable.
pub(crate) fn idle_field_pack(v: &Value) -> bool {
    !crate::curriculum::needs_healing(v)
        && crate::field_objectives::teaching_target(v).is_none()
        && v.pointer("/reward_state/key_items").and_then(Value::as_array).is_some_and(|items|items.is_empty())
        && v.pointer("/reward_state/items").and_then(Value::as_array).is_some_and(|items|items.iter().all(|i|i["id"].as_str().is_some_and(ball)))
}
fn utility_exit(v: &Value) -> Option<MenuGoal> {
    if !unneeded_utility_menu(v){return None;}
    let rows=entries(v);
    Some(MenuGoal{stage:"return_to_story",target:rows.iter().position(|s|matches!(label(s).as_str(),"CANCEL"|"EXIT"))?,cursor:rows.iter().position(|s|s.trim().starts_with('>'))?,columns:1})
}
fn utility_potential(v: &Value) -> f32 {
    if !unneeded_utility_menu(v){return 0.0;}
    -1.0-utility_exit(v).map_or(0.0,|g|0.1*g.distance() as f32)
}
pub(crate) fn cues(v: &Value) -> Vec<String> {
    if let Some(cues)=crate::recovery_objectives::cues(v){return cues;}
    if unneeded_utility_menu(v){return utility_exit(v).map(|g|g.cues().into_iter().map(|s|s.replace("objective:capture:","objective:overworld:")).collect()).unwrap_or_else(||vec!["objective:overworld:return_to_story".into()]);}
    if let Some(item)=healing_item(v) {
        let mut cues=vec![format!("objective:battle:heal:{item}")];
        if let Some(g)=healing_menu(v){cues.extend(g.cues().into_iter().map(|s|s.replace("objective:capture:","objective:battle:")));}
        if let Some(c)=crate::pocket_objectives::cue(v,"Items"){cues.push(c);}
        return cues;
    }
    let Some(species)=capture_species(v) else {
        return trainer_menu(v).map(|g|g.cues().into_iter().map(|s|s.replace("objective:capture:","objective:battle:")).collect()).unwrap_or_default();
    };
    let mut cues=vec![format!("objective:capture:species:{species}")];
    if let Some(hm)=crate::field_objectives::capture_hm(v){cues.push(format!("objective:capture:needed_for:{hm}"));}
    if let Some(cue)=pocket_cue(v){cues.push(cue);}
    if let Some(goal)=capture_menu(v) { cues.extend(goal.cues()); }
    else if !entries(v).is_empty() { cues.push(if ready_to_throw(v){"objective:capture:find_ball_pocket"}else{"objective:capture:weaken"}.into()); }
    cues
}
/// Foreground menu identity excludes cursor, changing HP and inventory counts.
/// A battle move list, Pack and main commands must not share one learned head.
pub(crate) fn menu_context(v: &Value) -> Option<String> {
    if let Some(cues)=crate::recovery_objectives::cues(v){return Some(cues.join("|"));}
    let menu=v.pointer("/observe/menus")?.as_array()?.last()?;
    let kind=menu["kind"].as_str()?;
    let rows=entries(v);
    let labels=rows.iter().map(|s| label(s)).collect::<Vec<_>>();
    let stage=if let Some(surface)=menu["surface"].as_str().filter(|s| matches!(*s,"commands"|"moves"|"party"|"party_actions"|"items"|"balls"|"key_items"|"machines"|"item_actions"|"item_target"|"faint_prompt"|"shift_prompt")) {surface}
        else if labels.iter().any(|s| s=="FIGHT") && labels.iter().any(|s| s=="RUN") {"commands"}
        else if labels.iter().any(|s| ball(s)) {"balls"}
        else if labels.iter().any(|s| s.contains("PP")) {"moves"}
        else if labels.iter().any(|s| s.starts_with("TEACH ")) {"teach_confirmation"}
        else if labels.iter().any(|s| s.contains("ABLE")) {"teach_target"}
        else if labels.iter().any(|s| s.starts_with("POCKET:")) {"pocket"}
        else {"choices"};
    let intent=utility_exit(v).or_else(||healing_menu(v)).or_else(||capture_menu(v)).or_else(||trainer_menu(v)).map(|g| g.cues().join("|")).unwrap_or_else(|| if healing_item(v).is_some(){"heal".into()}else if capture_species(v).is_some(){"capture".into()}else{"ordinary".into()});
    // A learned response for Tackle must not become the response for Leer
    // after a switch or move replacement. PP values are bucketed to usable /
    // exhausted so spending one PP does not discard the learned context.
    let moves = if stage == "moves" {
        let active=v.pointer("/reward_state/battle/active_player").and_then(Value::as_u64);
        let party=v.pointer("/status/party").and_then(Value::as_array);
        let learned=party.and_then(|p|p.iter().find(|p|p["slot"].as_u64()==active))
            .and_then(|p|p["moves"].as_array());
        labels.iter().map(|name| {
            let pp=learned.and_then(|ms|ms.iter().find(|m|m["name"].as_str().is_some_and(|n|label(n)==*name)))
                .and_then(|m|m["current_pp"].as_u64());
            format!("{name}:{}",match pp {Some(0)=>"empty",Some(_)=>"usable",None=>"unknown"})
        }).collect::<Vec<_>>().join("|")
    } else {String::new()};
    let base=format!("menu:{kind}:{stage}:{intent}:{}",pocket_cue(v).unwrap_or_default());
    Some(if moves.is_empty(){base}else{
        let selected=rows.iter().position(|s|s.trim().starts_with('>'))
            .and_then(|i|labels.get(i)).map(String::as_str).unwrap_or("unknown");
        let disabled=menu.pointer("/move_info/disabled").and_then(Value::as_bool);
        format!("{base}:{moves}:selected:{selected}:disabled:{disabled:?}")
    })
}
fn balls(v: &Value)->u64 {
    v.pointer("/reward_state/items").and_then(Value::as_array).into_iter().flatten().filter(|i| i["id"].as_str().is_some_and(ball)).map(|i| i["quantity"].as_u64().unwrap_or(0)).sum()
}
fn potential(v:&Value)->Option<f32> {
    if text(v,"/status/screen")!="battle" || !text(v,"/observe/battle_message").is_empty(){return None;}
    let goal=capture_menu(v);
    if let Some(g)=goal {
        let stage=match g.stage {"weaken"=>0.0,"open_pack"=>1.0,"select_ball"=>2.0,"throw_ball"=>2.5,"leave_moves"=>0.5,"leave_party_actions"=>-0.2,_=>0.0};
        return Some(stage-0.1*g.distance() as f32);
    }
    if let Some((distance,_))=crate::pocket_objectives::goal(v,"Balls") {
        return Some(if ready_to_throw(v){1.8-0.1*distance as f32}else{-0.3});
    }
    let rows=entries(v);
    (!rows.is_empty()).then_some(0.0)
}
/// Signed progress cancels on menu loops. Consuming a ball is actual gameplay,
/// not a reward for repeatedly highlighting it. Capture completion keeps the
/// existing one-time species/event reward. Knocking out this target is failure.
pub(crate) fn feedback(before:&Value,after:&Value,rewards:&mut Vec<String>,aversions:&mut Vec<String>)->f32 {
    if let Some(progress)=crate::recovery_objectives::feedback(before,after){return progress;}
    if unneeded_utility_menu(before)||unneeded_utility_menu(after){return 0.25*(utility_potential(after)-utility_potential(before));}
    let healing=healing_feedback(before,after);
    let Some(species)=capture_species(before) else{return healing.unwrap_or_else(||trainer_feedback(before,after));};
    let caught=after.pointer("/reward_state/caught_species").and_then(Value::as_array).is_some_and(|a| a.iter().any(|s| s.as_str()==Some(species)));
    let previously_caught=before.pointer("/reward_state/caught_species").and_then(Value::as_array).is_some_and(|a|a.iter().any(|s|s.as_str()==Some(species)));
    let count=|v:&Value|v.pointer("/reward_state/owned_species_counts").and_then(Value::as_object).map(|m|m.get(species).and_then(Value::as_u64).unwrap_or(0));
    let acquired=matches!((count(before),count(after)),(Some(a),Some(b)) if b>a);
    if (caught && !previously_caught) || acquired {return 1.5;}
    let knocked_out=after.pointer("/reward_state/battle/enemy_hp").and_then(Value::as_u64)==Some(0) || rewards.iter().any(|s|s=="battle:victory");
    if knocked_out {
        rewards.retain(|s| !s.starts_with("battle:damage") && s!="battle:victory");
        aversions.push("battle:capture_target_knocked_out".into());return 0.0;
    }
    if ready_to_throw(before) {rewards.retain(|s|!s.starts_with("battle:damage"));}
    if let Some(progress)=healing {return progress;}
    if balls(after)<balls(before) && text(after,"/status/screen")=="battle" {return 0.25;}
    if capture_species(after)!=Some(species) {return 0.0;}
    match (potential(before),potential(after)) {
        (Some(a),Some(b)) if a!=b=>0.3*(b-a),
        _=>0.0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn utility_exit_progress_is_signed_and_leaves_hm_menus_alone() {
        let closed=serde_json::json!({"status":{"screen":"overworld","party":[{"hp":46,"max_hp":46}]},"map_info":{"name":"VioletCity"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]},"observe":{"menus":[]}});
        let mut open=closed.clone();open["observe"]["menus"]=serde_json::json!([{"kind":"pokedex","entries":[">TYPE1 FIRE","TYPE2 ----","BEGIN SEARCH!!","CANCEL"]}]);
        let mut near=open.clone();near["observe"]["menus"][0]["entries"]=serde_json::json!(["TYPE1 FIRE","TYPE2 ----","BEGIN SEARCH!!",">CANCEL"]);
        let feedback_value=|a:&Value,b:&Value|feedback(a,b,&mut vec![],&mut vec![]);
        assert!(feedback_value(&open,&near)>0.0);
        assert_eq!(feedback_value(&open,&near)+feedback_value(&near,&open),0.0);
        assert_eq!(feedback_value(&closed,&open)+feedback_value(&open,&closed),0.0);
        assert_eq!(feedback_value(&open,&open),0.0);
        assert!(cues(&near).iter().any(|s|s.contains("return_to_story")));
        for kind in ["options", "trainer_card"] {
            let mut utility = open.clone();
            utility["observe"]["menus"][0]["kind"] = serde_json::json!(kind);
            assert!(cues(&utility).iter().any(|s|s.contains("return_to_story")));
            assert!(feedback_value(&utility,&closed)>0.0);
            assert_eq!(feedback_value(&closed,&utility)+feedback_value(&utility,&closed),0.0);
        }
        let mut hm=near.clone();hm["observe"]["menus"][0]["kind"]=serde_json::json!("pack");
        assert!(!unneeded_utility_menu(&hm));assert_eq!(utility_potential(&hm),0.0);
        let mut no_story=open.clone();no_story["reward_state"]["event_flags"]=serde_json::json!([]);
        assert!(!unneeded_utility_menu(&no_story));
    }
    #[test]
    fn ball_only_field_pack_returns_to_story_without_blocking_item_tasks() {
        let mut v=json!({"status":{"screen":"overworld","party":[{"hp":46,"max_hp":46,"moves":[]}]},"map_info":{"name":"VioletPokecenter1F"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"],"items":[{"id":"POKE_BALL","quantity":2}],"key_items":[],"machines":["TM_MUD_SLAP"]},"observe":{"menus":[{"kind":"pack","entries":["POCKET: TM/HM"," TM31 x01",">CANCEL"]}]}});
        assert!(cues(&v).contains(&"objective:overworld:return_to_story".into()));
        let mut closed=v.clone();closed["observe"]["menus"]=json!([]);
        assert!(feedback(&v,&closed,&mut vec![],&mut vec![])>0.0);
        assert_eq!(feedback(&v,&closed,&mut vec![],&mut vec![])+feedback(&closed,&v,&mut vec![],&mut vec![]),0.0);
        for (key,value) in [("key_items",json!(["SQUIRTBOTTLE"])),("items",json!([{"id":"POTION","quantity":1}])),("items",Value::Null)] {
            let mut needed=v.clone();needed["reward_state"][key]=value;assert!(!unneeded_utility_menu(&needed));
        }
        v["reward_state"]["machines"]=json!(["HM_CUT"]);
        v["map_info"]["hm_compatibility"]=json!({"HM_CUT":[{"slot":0,"able":true}]});
        assert!(!unneeded_utility_menu(&v));
        v["status"]["screen"]=json!("battle");assert!(!unneeded_utility_menu(&v));
    }
    #[test]
    fn start_exit_guides_story_interactions_but_preserves_hm_teaching() {
        let mut v=json!({"status":{"screen":"overworld","party":[{"hp":40,"max_hp":40,"moves":[]}]},"map_info":{"name":"VioletPokecenter1F"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]},"observe":{"menus":[{"kind":"start","entries":["PACK",">OPTION","EXIT"]}]}});
        assert!(unneeded_utility_menu(&v));
        assert_eq!(utility_exit(&v).unwrap().target,2);
        let mut near=v.clone();near["observe"]["menus"][0]["entries"]=json!(["PACK","OPTION",">EXIT"]);
        let mut closed=v.clone();closed["observe"]["menus"]=json!([]);
        assert!(utility_potential(&near)>utility_potential(&v));
        let feedback_value=|a:&Value,b:&Value|feedback(a,b,&mut vec![],&mut vec![]);
        assert_eq!(feedback_value(&closed,&v)+feedback_value(&v,&near)+feedback_value(&near,&closed),0.0);
        v["reward_state"]["machines"]=json!(["HM_CUT"]);
        v["map_info"]["hm_compatibility"]=json!({"HM_CUT":[{"slot":0,"able":true}]});
        assert!(crate::field_objectives::teaching_target(&v).is_some());
        assert!(!unneeded_utility_menu(&v));
    }
    #[test]
    fn critical_healing_context_tracks_the_observed_pocket_direction() {
        let mut v = serde_json::json!({"status":{"screen":"battle","party":[{"slot":0,"hp":2,"max_hp":43}]},"reward_state":{"battle":{"active_player":0},"items":[{"id":"POTION","quantity":1}]},"observe":{"battle":"Trainer FALKNER","menus":[{"kind":"battle","surface":"balls","entries":["># BALL x2","CANCEL"],"pack_pockets":["Items","Balls","Key","TM/HM"],"pack_pocket":"Balls"}]}});
        let west = menu_context(&v).unwrap();
        assert!(west.contains("objective:pocket:Items:west"));
        v["observe"]["menus"][0]["pack_pockets"] = serde_json::json!(["Balls","Items","Key","TM/HM"]);
        let east = menu_context(&v).unwrap();
        assert!(east.contains("objective:pocket:Items:east"));
        assert_ne!(west, east);
        v["status"]["party"][0]["hp"] = serde_json::json!(43);
        assert!(!menu_context(&v).unwrap().contains("objective:pocket:Items:"));
    }
    use super::*;use serde_json::json;
    fn wild()->Value { json!({"status":{"screen":"battle","party":[{"slot":0,"hp":19,"is_egg":false}]},"observe":{"battle":"Wild BATTLETYPE_NORMAL\nEnemy PIDGEY L3 HP 4/12","menus":[{"kind":"battle","entries":[">FIGHT","<PKMN>","PACK","RUN"]}]},"reward_state":{"caught_species":[],"items":[{"id":"POKE_BALL","quantity":5}],"battle":{"active_player":0,"enemy_hp":4,"enemy_max_hp":12}}}) }
    #[test] fn healing_exits_moves_and_rejects_using_the_wrong_item() {
        let mut v=wild();v["status"]["party"][0]["hp"]=json!(10);v["status"]["party"][0]["max_hp"]=json!(43);
        v["reward_state"]["items"].as_array_mut().unwrap().push(json!({"id":"POTION","quantity":1}));
        let mut commands=v.clone();commands["observe"]["menus"][0]["surface"]=json!("commands");
        v["observe"]["menus"][0]=json!({"surface":"moves","entries":[">TACKLE","LEER","CANCEL"]});
        assert_eq!(healing_menu(&v).unwrap().target,2);
        assert!(healing_feedback(&v,&commands).unwrap()>0.0);
        assert!((healing_feedback(&v,&commands).unwrap()+healing_feedback(&commands,&v).unwrap()).abs()<1e-6);
        v["observe"]["menus"][0]=json!({"surface":"item_actions","entries":["ACTION # BALL x3",">USE","QUIT"]});
        assert_eq!(healing_menu(&v).unwrap().stage,"heal_leave_wrong_item");
        assert_eq!(healing_menu(&v).unwrap().target,2);
        v["observe"]["menus"][0]["entries"]=json!(["ACTION POTION x1",">USE","QUIT"]);
        assert_eq!(healing_menu(&v).unwrap().stage,"heal_use_item");
        v["status"]["party"][0]["hp"]=json!(43);assert!(healing_menu(&v).is_none());
    }
    #[test] fn ready_capture_leaves_move_list_but_weakening_keeps_moves_available() {
        let mut a=wild();a["observe"]["menus"][0]=json!({"kind":"battle","surface":"moves","entries":[">TACKLE","LEER","CANCEL"]});
        let g=capture_menu(&a).unwrap();assert_eq!(g.stage,"leave_moves");assert_eq!(g.target,2);
        assert!(cues(&a).contains(&"objective:menu:south".into()));assert!(!cues(&a).contains(&"objective:capture:find_ball_pocket".into()));
        let mut b=a.clone();b["observe"]["menus"][0]["entries"]=json!(["TACKLE","LEER",">CANCEL"]);
        assert!(feedback(&a,&b,&mut vec![],&mut vec![])>0.0);
        assert_eq!(feedback(&a,&b,&mut vec![],&mut vec![])+feedback(&b,&a,&mut vec![],&mut vec![]),0.0);
        a["reward_state"]["battle"]["enemy_hp"]=json!(12);assert!(capture_menu(&a).is_none());
        a["reward_state"]["battle"]["enemy_hp"]=json!(4);a["observe"]["battle"]=json!("Trainer\nEnemy PIDGEY L3 HP 4/12");assert!(capture_menu(&a).is_none());
    }
    #[test] fn capture_escape_requires_confirmed_player_intent_and_healthy_active() {
        let mut a=wild();a["status"]["party"][0]["max_hp"]=json!(20);
        a["observe"]["menus"][0]=json!({"kind":"battle","surface":"commands","entries":["FIGHT","<PKMN>","PACK",">RUN"]});
        let mut b=a.clone();b["status"]["screen"]=json!("overworld");b["reward_state"]["battle"]=Value::Null;b["reward_state"]["battle_result"]=json!(2);
        assert!(abandoned_capture(&a,&b,"a"));assert!(!abandoned_capture(&a,&b,"right"));
        for result in [0,1,64] {b["reward_state"]["battle_result"]=json!(result);assert!(!abandoned_capture(&a,&b,"a"));}
        b["reward_state"]["battle_result"]=json!(2);b["status"]["screen"]=json!("battle");assert!(!abandoned_capture(&a,&b,"a"));b["status"]["screen"]=json!("overworld");
        a["status"]["party"][0]["hp"]=json!(6);assert!(!abandoned_capture(&a,&b,"a"));a["status"]["party"][0]["hp"]=json!(19);
        a["observe"]["menus"][0]["entries"]=json!([">FIGHT","<PKMN>","PACK","RUN"]);assert!(!abandoned_capture(&a,&b,"a"));
        a["observe"]["menus"][0]["entries"]=json!(["FIGHT","<PKMN>","PACK",">RUN"]);a["reward_state"]["items"]=json!([]);assert!(!abandoned_capture(&a,&b,"a"));
    }
    #[test] fn capture_needed_hm_species_requires_a_new_acquisition_not_old_dex_credit() {
        let mut v=wild();v["reward_state"]["caught_species"]=json!(["PIDGEY"]);
        v["reward_state"]["machines"]=json!(["HM_FLY"]);
        v["map_info"]=json!({"hm_compatibility":{"HM_FLY":[{"slot":0,"able":false}]}});
        v["reward_state"]["enemy_hm_compatibility"]=json!({"HM_FLY":true});
        v["reward_state"]["owned_species_counts"]=json!({});
        assert_eq!(capture_species(&v),Some("PIDGEY"));
        assert!(cues(&v).contains(&"objective:capture:needed_for:FLY".into()));
        assert_eq!(feedback(&v,&v,&mut vec![],&mut vec![]),0.0);
        let mut after=v.clone();after["reward_state"]["owned_species_counts"]["PIDGEY"]=json!(1);
        assert_eq!(feedback(&v,&after,&mut vec![],&mut vec![]),1.5);
        assert!(capture_species(&after).is_none(),"Already-owned boxed candidates should not trigger duplicate catching");
        v["reward_state"]["enemy_hm_compatibility"]["HM_FLY"]=Value::Null;
        assert!(capture_species(&v).is_none());
        v["reward_state"]["enemy_hm_compatibility"]["HM_FLY"]=json!(true);
        v["observe"]["battle"]=json!("Trainer PIDGEY");assert!(capture_species(&v).is_none());
    }
    #[test] fn capture_avoids_forced_weakening_with_large_visible_level_gap() {
        let mut v=wild();
        v["status"]["party"][0]["level"]=json!(13);
        v["reward_state"]["battle"]["enemy_hp"]=json!(12);
        assert_eq!(capture_menu(&v).unwrap().stage,"open_pack");
        v["observe"]["menus"][0]=json!({"surface":"balls","entries":["># BALL x3","CANCEL"]});
        assert_eq!(capture_menu(&v).unwrap().stage,"select_ball");
        v["status"]["party"][0]["level"]=json!(4);
        assert!(!ready_to_throw(&v));
        v["status"]["party"][0]["level"]=Value::Null;
        assert!(!ready_to_throw(&v));
        v["status"]["party"][0]["level"]=json!(13);
        v["reward_state"]["battle"]["enemy_hp"]=json!(0);
        assert!(!ready_to_throw(&v));
    }
    #[test] fn capture_recognizes_pinned_binary_poke_ball_glyph() {
        let mut v=wild();
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"balls","entries":["># BALL x3","CANCEL"],"pack_pocket":"Balls","pack_pockets":["Items","Balls","Key","TM/HM"]});
        let goal=capture_menu(&v).unwrap();
        assert_eq!(goal.stage,"select_ball");assert_eq!(goal.target,0);
        assert!(cues(&v).contains(&"objective:capture:select_ball".into()));
        assert!(!ball("CANCEL"));assert!(!ball("# DOLL"));
        let list=v.clone();
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"item_actions","entries":["ACTION # BALL",">USE"," QUIT"]});
        assert_eq!(capture_menu(&v).unwrap().stage,"throw_ball");
        assert_eq!(capture_menu(&v).unwrap().target,1);
        let mut rewards=vec![];let mut aversions=vec![];
        let round_trip=feedback(&list,&v,&mut rewards,&mut aversions)+feedback(&v,&list,&mut rewards,&mut aversions);
        assert!(round_trip.abs()<1e-6);
    }
    #[test] fn critical_battle_healing_connects_pack_item_and_active_target() {
        let mut v=wild();v["observe"]["battle"]=json!("Trainer ABE");
        v["status"]["party"][0]["hp"]=json!(8);v["status"]["party"][0]["max_hp"]=json!(36);
        v["reward_state"]["items"].as_array_mut().unwrap().push(json!({"id":"POTION","quantity":1}));
        v["observe"]["menus"][0]["surface"]=json!("commands");
        assert_eq!(healing_menu(&v).unwrap().target,2);
        assert!(cues(&v).contains(&"objective:battle:heal:POTION".into()));
        let main=v.clone();
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"items","entries":[">ANTIDOTE","POTION ×1","CANCEL"],"pack_pockets":["Items","Balls","Key","TM/HM"],"pack_pocket":"Items"});
        assert_eq!(healing_menu(&v).unwrap().target,1);
        let delta=healing_feedback(&main,&v).unwrap()+healing_feedback(&v,&main).unwrap();
        assert!(delta.abs()<1e-6,"Menu round trips cannot farm healing progress");
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"item_actions","entries":["ACTION POTION",">USE"," QUIT"]});
        assert_eq!(healing_menu(&v).unwrap().stage,"heal_use_item");assert_eq!(healing_menu(&v).unwrap().target,1);
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"item_target","entries":[">CYNDAQUIL","CANCEL"]});
        assert_eq!(healing_menu(&v).unwrap().target,0);
        let mut healed=v.clone();healed["status"]["party"][0]["hp"]=json!(28);healed["reward_state"]["items"][1]["quantity"]=json!(0);
        assert_eq!(healing_feedback(&v,&healed),Some(0.5));
        assert!(healing_item(&healed).is_none());
        // HP changes without consumption and mere consumption without recovery
        // are not proof of successful healing.
        healed["reward_state"]["items"][1]["quantity"]=json!(1);
        assert_ne!(healing_feedback(&v,&healed),Some(0.5));
        healed["status"]["party"][0]["hp"]=json!(8);healed["reward_state"]["items"][1]["quantity"]=json!(0);
        assert_ne!(healing_feedback(&v,&healed),Some(0.5));
        v["status"]["party"][0]["hp"]=json!(0);assert!(healing_item(&v).is_none());
        v["status"]["party"][0]["hp"]=json!(13);assert!(healing_item(&v).is_none());
        v["status"]["party"][0]["hp"]=json!(8);v["reward_state"]["items"][1]["quantity"]=json!(0);assert!(healing_item(&v).is_none());
    }
    #[test] fn critical_healing_selects_active_slot_and_still_limits_empty_pack_visits() {
        let mut v=wild();v["observe"]["battle"]=json!("Trainer ABE");
        v["status"]["party"]=json!([{"slot":0,"hp":30,"max_hp":30},{"slot":1,"hp":4,"max_hp":30}]);
        v["reward_state"]["battle"]["active_player"]=json!(1);
        v["reward_state"]["items"]=json!([{"id":"SUPER_POTION","quantity":1}]);
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"item_target","entries":[">FIRST","SECOND","CANCEL"]});
        assert_eq!(healing_menu(&v).unwrap().target,1);
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"commands","entries":["FIGHT","<PKMN>",">PACK","RUN"]});
        assert!(trainer_pack_selected(&v));
        assert!(menu_context(&v).unwrap().contains("heal_open_pack"));
    }
    #[test] fn pack_guard_does_not_block_captures_items_or_other_commands() {
        let mut v=wild();v["observe"]["menus"][0]["entries"]=json!(["FIGHT","<PKMN>",">PACK","RUN"]);
        assert!(!trainer_pack_selected(&v));
        v["observe"]["battle"]=json!("Trainer ABE");assert!(trainer_pack_selected(&v));
        v["observe"]["menus"][0]["entries"]=json!([">POTION","CANCEL"]);assert!(!trainer_pack_selected(&v));
        v["observe"]["menus"][0]["entries"]=json!([">FIGHT","<PKMN>","PACK","RUN"]);assert!(!trainer_pack_selected(&v));
    }
    #[test] fn move_context_tracks_move_identity_and_exhaustion_without_pp_churn() {
        let mut v=wild();
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"moves","entries":[">TACKLE","LEER","CANCEL"]});
        v["status"]["party"][0]["moves"]=json!([{"name":"TACKLE","current_pp":10},{"name":"LEER","current_pp":20}]);
        let initial=menu_context(&v);
        v["status"]["party"][0]["moves"][0]["current_pp"]=json!(9);
        assert_eq!(initial,menu_context(&v));
        v["status"]["party"][0]["moves"][0]["current_pp"]=json!(0);
        assert_ne!(initial,menu_context(&v));
        v["status"]["party"][0]["moves"][0]["current_pp"]=json!(10);
        v["observe"]["menus"][0]["entries"]=json!(["TACKLE",">LEER","CANCEL"]);
        assert_ne!(initial,menu_context(&v));
        v["observe"]["menus"][0]["entries"]=json!([">TACKLE","LEER","CANCEL"]);
        assert_eq!(initial,menu_context(&v));
        v["observe"]["menus"][0]["move_info"]=json!({"disabled":true});
        assert_ne!(initial,menu_context(&v));
        v["observe"]["menus"][0]["entries"]=json!([">EMBER","LEER","CANCEL"]);
        assert_ne!(initial,menu_context(&v));
    }
    #[test] fn capture_leaves_unnecessary_party_list_but_not_a_required_replacement(){
        let mut v=wild();v["reward_state"]["battle"]["active_player"]=json!(0);
        v["status"]["party"]=json!([{"slot":0,"hp":19,"is_egg":false}]);
        v["observe"]["menus"][0]=json!({"kind":"battle","surface":"party","entries":[">CYNDAQUIL","CANCEL"]});
        assert_eq!(capture_menu(&v).unwrap().target,1);
        v["status"]["party"][0]["hp"]=json!(0);assert!(capture_menu(&v).is_none());
        v["status"]["party"].as_array_mut().unwrap().push(json!({"slot":1,"hp":20,"is_egg":false}));
        assert!(capture_menu(&v).is_none());
    }
    #[test] fn capture_recovers_from_party_actions_without_rewarding_reentry(){
        let mut a=wild();a["observe"]["menus"][0]["entries"]=json!([">SWITCH","STATS","CANCEL"]);
        let goal=capture_menu(&a).unwrap();assert_eq!(goal.stage,"leave_party_actions");assert_eq!(goal.target,2);
        let mut b=a.clone();b["observe"]["menus"][0]["entries"]=json!(["SWITCH","STATS",">CANCEL"]);
        let forward=feedback(&a,&b,&mut vec![],&mut vec![]);
        assert!(forward>0.0);assert!((forward+feedback(&b,&a,&mut vec![],&mut vec![])).abs()<1e-6);
        a["status"]["party"][0]["hp"]=json!(0);assert!(capture_menu(&a).is_none());
    }
    #[test] fn trainer_fight_progress_cancels_on_backtracking_and_preserves_forced_switches(){
        let mut a=wild();a["observe"]["battle"]=json!("Trainer ABE\nEnemy SPEAROW L9 HP27/27");
        a["observe"]["menus"][0]["surface"]=json!("commands");
        a["reward_state"]["battle"]["active_player"]=json!(0);
        a["status"]["party"]=json!([{"slot":0,"hp":19,"is_egg":false}]);
        assert_eq!(trainer_menu(&a).unwrap().stage,"fight");
        let mut b=a.clone();b["observe"]["menus"][0]=json!({"kind":"battle","surface":"moves","entries":[">TACKLE","LEER","CANCEL"]});
        let f=feedback(&a,&b,&mut vec![],&mut vec![]);assert!(f>0.0);
        assert!((f+feedback(&b,&a,&mut vec![],&mut vec![])).abs()<1e-6);
        b["observe"]["menus"][0]=json!({"kind":"battle","surface":"party","entries":[">CYNDAQUIL","CANCEL"]});
        assert_eq!(trainer_menu(&b).unwrap().target,1);
        b["status"]["party"][0]["hp"]=json!(0);assert!(trainer_menu(&b).is_none());
        b["status"]["party"][0]["hp"]=json!(19);b["observe"]["menus"][0]["surface"]=json!("faint_prompt");assert!(trainer_menu(&b).is_none());
    }
    #[test] fn main_menu_geometry_and_health_change_the_target(){
        let mut v=wild();let g=capture_menu(&v).unwrap();assert_eq!(g.stage,"open_pack");assert_eq!(g.distance(),1);assert!(g.cues().contains(&"objective:menu:south".into()));
        v["reward_state"]["battle"]["enemy_hp"]=json!(12);assert_eq!(capture_menu(&v).unwrap().stage,"weaken");assert_eq!(capture_menu(&v).unwrap().distance(),0);
    }
    #[test] fn trainer_caught_species_and_empty_inventory_are_not_capture_targets(){
        let mut v=wild();v["observe"]["battle"]=json!("Trainer BATTLETYPE_NORMAL\nEnemy PIDGEY L3 HP 4/12");assert!(capture_species(&v).is_none());
        let mut v=wild();v["reward_state"]["caught_species"]=json!(["PIDGEY"]);assert!(capture_species(&v).is_none());
        let mut v=wild();v["reward_state"]["items"][0]["quantity"]=json!(0);assert!(capture_species(&v).is_none());
    }
    #[test] fn nested_menu_and_ball_labels_use_the_visible_foreground(){
        let mut v=wild();v["observe"]["menus"]=json!([{"kind":"start","entries":[">PACK"]},{"kind":"battle","entries":[">GREAT BALL x2","POKé BALL x5","CANCEL"]}]);
        let g=capture_menu(&v).unwrap();assert_eq!(g.stage,"select_ball");assert_eq!(g.target,0);
    }
    #[test] fn capture_cursor_loop_cannot_farm_progress_and_knockout_is_failure(){
        let a=wild();let mut b=a.clone();b["observe"]["menus"][0]["entries"]=json!(["FIGHT","<PKMN>",">PACK","RUN"]);
        let mut rewards=vec![];let mut aversions=vec![];
        let forward=feedback(&a,&b,&mut rewards,&mut aversions);
        let backward=feedback(&b,&a,&mut rewards,&mut aversions);
        assert!(forward>0.0);assert!((forward+backward).abs()<1e-6);
        assert_eq!(feedback(&b,&b,&mut rewards,&mut aversions),0.0);
        let mut dead=a.clone();dead["reward_state"]["battle"]["enemy_hp"]=json!(0);
        rewards=vec!["battle:damage:0".into(),"battle:victory".into()];
        assert_eq!(feedback(&a,&dead,&mut rewards,&mut aversions),0.0);
        assert!(rewards.is_empty());assert!(aversions.contains(&"battle:capture_target_knocked_out".into()));
    }
    #[test] fn final_ball_throw_and_capture_are_observed_outcomes(){
        let mut a=wild();a["reward_state"]["items"][0]["quantity"]=json!(1);
        a["observe"]["menus"][0]["entries"]=json!([">POKé BALL x1","CANCEL"]);
        assert_eq!(capture_menu(&a).unwrap().stage,"select_ball");
        let mut b=a.clone();b["reward_state"]["items"][0]["quantity"]=json!(0);
        assert_eq!(feedback(&a,&b,&mut vec![],&mut vec![]),0.25);
        b["reward_state"]["caught_species"]=json!(["PIDGEY"]);
        assert_eq!(feedback(&a,&b,&mut vec![],&mut vec![]),1.5);
    }
    #[test] fn entering_pack_before_weakening_does_not_earn_capture_progress(){
        let mut a=wild();a["reward_state"]["battle"]["enemy_hp"]=json!(12);
        let mut b=a.clone();b["observe"]["menus"]=json!([{"kind":"battle","entries":[">POTION x1","CANCEL"],"pack_pocket":"Items","pack_pockets":["Items","Balls","Key","TM/HM"]}]);
        assert!(feedback(&a,&b,&mut vec![],&mut vec![])<=0.0);
        assert!(!cues(&b).iter().any(|s|s.starts_with("objective:pocket:Balls:")));
    }
    #[test] fn pocket_round_trip_has_zero_net_capture_reward(){
        let mut a=wild();a["observe"]["menus"]=json!([{"kind":"battle","entries":[">POTION x1","CANCEL"],"pack_pocket":"Items","pack_pockets":["Items","Balls","Key","TM/HM"]}]);
        let mut b=a.clone();b["observe"]["menus"][0]["pack_pocket"]=json!("Balls");b["observe"]["menus"][0]["entries"]=json!([">POKE BALL x5","CANCEL"]);
        let f=feedback(&a,&b,&mut vec![],&mut vec![]);let r=feedback(&b,&a,&mut vec![],&mut vec![]);
        assert!(f>0.0);assert!((f+r).abs()<1e-6);assert!(cues(&a).contains(&"objective:pocket:Balls:east".into()));
    }
    #[test] fn explicit_battle_surface_distinguishes_name_only_move_rows(){
        let mut moves=wild();moves["observe"]["menus"][0]=json!({"kind":"battle","surface":"moves","entries":[">TACKLE","LEER","CANCEL"]});
        let mut party=moves.clone();party["observe"]["menus"][0]["surface"]=json!("party");
        assert_ne!(menu_context(&moves),menu_context(&party));
        assert!(menu_context(&moves).unwrap().contains(":moves:"));
        assert!(crate::interface::sensory_features_with_menus(&moves,true).iter().any(|s|s.contains("menu:surface:")&&s.contains("moves")));
    }
    #[test] fn contexts_separate_commands_from_ball_selection(){
        let a=wild();let mut b=a.clone();b["observe"]["menus"][0]["entries"]=json!([">POKE BALL x5","CANCEL"]);
        assert_ne!(menu_context(&a),menu_context(&b));
        let mut c=b.clone();c["observe"]["menus"][0]["entries"]=json!([">POKE BALL x4","CANCEL"]);
        assert_eq!(menu_context(&b),menu_context(&c));
    }

}

#[cfg(test)]
mod perception_tests {
    use serde_json::json;
    #[test]
    fn detailed_menus_retain_enemy_health_and_encounter_kind() {
        let v=json!({"status":{"screen":"battle"},"observe":{"battle":"Wild BATTLETYPE_NORMAL\nEnemy PIDGEY L3 HP 3/12","menus":[{"kind":"battle","entries":[">FIGHT","PACK"]}]},"reward_state":{"battle":{"enemy_hp":3,"enemy_max_hp":12,"enemy_status":null}}});
        let features=crate::interface::sensory_features_with_menus(&v,true);
        assert!(features.contains(&"battle:kind:Wild".into()));
        assert!(features.contains(&"battle:enemy_health:1".into()));
        assert!(features.iter().any(|s| s.starts_with("menu:0:row:")));
    }
}
