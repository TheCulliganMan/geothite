//! Declared story assistance. Prerequisites select an outcome, never a button.
//! Exit coordinates come from authored map telemetry, not guessed door positions.
use serde_json::Value;
fn has(v:&Value,flag:&str)->bool {v.pointer("/reward_state/event_flags").and_then(Value::as_array).is_some_and(|a|a.iter().any(|f|f.as_str()==Some(flag)))}
fn badge(v:&Value,i:usize)->bool {v.pointer(&format!("/status/badges/johto/{i}")).and_then(Value::as_bool)==Some(true)}
#[derive(Debug,PartialEq)]
pub(crate) enum Goal { Exit(&'static str), Talk(&'static str,&'static str), Visit(&'static str,&'static str), Pc, CutTree }
pub(crate) fn needs_healing(v:&Value)->bool {
    v.pointer("/status/party").and_then(Value::as_array).is_some_and(|party|party.iter().filter(|p|p["is_egg"]!=true).any(|p|{
        let hp=p["hp"].as_u64().unwrap_or(0);let max=p["max_hp"].as_u64().unwrap_or(0);
        (max>0&&hp*4<=max*3) || p["status"].as_str().is_some_and(|s|!s.is_empty()&&s!="None")
            || p["moves"].as_array().is_some_and(|moves|moves.iter().any(|m|m["current_pp"].as_u64()==Some(0)))
    }))
}
pub(crate) fn pending_togepi(v:&Value)->bool {
    has(v,"EVENT_GAVE_MYSTERY_EGG_TO_ELM") && badge(v,0) && !has(v,"EVENT_GOT_TOGEPI_EGG_FROM_ELMS_AIDE")
}
pub(crate) fn needs_egg_space(v:&Value)->bool {
    pending_togepi(v) && v.pointer("/status/party").and_then(Value::as_array).is_some_and(|p|p.len()>=6)
}
pub(crate) fn goal(v:&Value)->Option<Goal> {
    use Goal::*;
    if !has(v,"EVENT_GAVE_MYSTERY_EGG_TO_ELM") {return None;}
    let map=v.pointer("/map_info/name")?.as_str()?;
    for (city,center,gym) in [("VioletCity","VioletPokecenter1F","VioletGym"),("AzaleaTown","AzaleaPokecenter1F","AzaleaGym"),("GoldenrodCity","GoldenrodPokecenter1F","GoldenrodGym"),("EcruteakCity","EcruteakPokecenter1F","EcruteakGym"),("OlivineCity","OlivinePokecenter1F","OlivineGym")] {
        if needs_healing(v) {
            if map==gym{return Some(Exit(city));}
            if map==city{return Some(Exit(center));}
            if map==center{return Some(Talk("NURSE","Heal the party before continuing the story"));}
        }
        if map=="VioletPokecenter1F" && needs_egg_space(v){return Some(Pc);}
        if crate::field_objectives::needs_boxed_hm(v) {
            if map==gym {return Some(Exit(city));}
            if map==city {return Some(Exit(center));}
            if map==center {return Some(Pc);}
        }
        if !needs_healing(v) && map==center && center!="VioletPokecenter1F" {return Some(Exit(city));}
    }
    // Replenish in towns already on the story route, without sending a healthy
    // run backwards through a dungeon. Authored exits provide door coordinates.
    for (city,mart) in [("CherrygroveCity","CherrygroveMart"),("VioletCity","VioletMart"),("AzaleaTown","AzaleaMart"),("EcruteakCity","EcruteakMart"),("OlivineCity","OlivineMart")] {
        if map==mart {
            return Some(if crate::shop_objectives::needs_restock(v) {Talk("CLERK","Replenish Poke Balls for captures")}else{Exit(city)});
        }
        if map==city && crate::shop_objectives::needs_restock(v) {return Some(Exit(mart));}
    }
    let return_for_bottle = has(v,"EVENT_MET_FLORIA") && !has(v,"EVENT_GOT_SQUIRTBOTTLE") && !has(v,"EVENT_FOUGHT_SUDOWOODO");
    Some(match map {
        "DarkCaveVioletEntrance"=>Exit("Route31"),
        "Route31"=>Exit("Route31VioletGate"),
        "Route31VioletGate"=>Exit("VioletCity"),
        "VioletCity" if !badge(v,0)=>Exit("VioletGym"),
        "VioletGym" if !badge(v,0)=>Talk("FALKNER","Challenge Falkner for the Zephyr Badge"),
        "VioletGym"=>Exit("VioletCity"),
        "VioletCity" if !has(v,"EVENT_GOT_TOGEPI_EGG_FROM_ELMS_AIDE")=>Exit("VioletPokecenter1F"),
        "VioletCity"=>Exit("Route32"),
        "VioletPokecenter1F" if badge(v,0) && !has(v,"EVENT_GOT_TOGEPI_EGG_FROM_ELMS_AIDE")=>Talk("ElmsAide","Receive the Togepi Egg before Route 32"),
        "VioletPokecenter1F"=>Exit("VioletCity"),
        "Route32"=>Exit("UnionCave1F"),
        "UnionCave1F"=>Exit("Route33"),
        "Route33"=>Exit("AzaleaTown"),
        "AzaleaTown" if !has(v,"EVENT_CLEARED_SLOWPOKE_WELL") && !has(v,"EVENT_KURTS_HOUSE_KURT_1")=>Exit("KurtsHouse"),
        "KurtsHouse" if !has(v,"EVENT_CLEARED_SLOWPOKE_WELL")=>Talk("KURT","Ask Kurt about the Slowpoke Well"),
        "KurtsHouse"=>Exit("AzaleaTown"),
        "AzaleaTown" if !has(v,"EVENT_CLEARED_SLOWPOKE_WELL")=>Exit("SlowpokeWellB1F"),
        "SlowpokeWellB1F" if !has(v,"EVENT_CLEARED_SLOWPOKE_WELL")=>Talk("TrainerGruntM1","Clear Team Rocket from Slowpoke Well"),
        "SlowpokeWellB1F"=>Exit("AzaleaTown"),
        "AzaleaTown" if !badge(v,1)=>Exit("AzaleaGym"),
        "AzaleaGym" if !badge(v,1)=>Talk("BUGSY","Challenge Bugsy for the Hive Badge"),
        "AzaleaGym"=>Exit("AzaleaTown"),
        "AzaleaTown"=>Exit("IlexForestAzaleaGate"),
        "IlexForestAzaleaGate"=>Exit("IlexForest"),
        "IlexForest" if !has(v,"EVENT_HERDED_FARFETCHD")=>Talk("FARFETCHD","Return Farfetch'd to the charcoal maker"),
        "IlexForest" if !has(v,"EVENT_GOT_HM01_CUT")=>Talk("CHARCOAL_MASTER","Receive Cut after returning Farfetch'd"),
        "IlexForest" if badge(v,1) && crate::field_objectives::known(v,"CUT") && !cut_tiles(v).is_empty()=>CutTree,
        "IlexForest"=>Exit("Route34IlexForestGate"),
        "Route34IlexForestGate"=>Exit("Route34"),
        "Route34"=>Exit("GoldenrodCity"),
        "GoldenrodCity" if !badge(v,2)=>Exit("GoldenrodGym"),
        // Winning starts a coordinate-triggered conversation before Whitney
        // awards the badge. Repeatedly talking to her cannot clear the crying flag.
        "GoldenrodGym" if !badge(v,2) && has(v,"EVENT_BEAT_WHITNEY") && has(v,"EVENT_MADE_WHITNEY_CRY")=>Visit("WhitneyCriesScript","Walk away so Bridget explains Whitney's crying"),
        "GoldenrodGym" if !badge(v,2)=>Talk("WHITNEY","Challenge Whitney and receive the Plain Badge"),
        "GoldenrodGym"=>Exit("GoldenrodCity"),
        "GoldenrodCity" if return_for_bottle=>Exit("GoldenrodFlowerShop"),
        "GoldenrodCity"=>Exit("Route35GoldenrodGate"),
        "GoldenrodFlowerShop" if return_for_bottle && !has(v,"EVENT_TALKED_TO_FLORIA_AT_FLOWER_SHOP")=>Talk("FlowerShopFloriaScript","Speak to Floria after meeting her at the tree"),
        "GoldenrodFlowerShop" if return_for_bottle=>Talk("FlowerShopTeacherScript","Receive the SquirtBottle with the Plain Badge"),
        "GoldenrodFlowerShop"=>Exit("GoldenrodCity"),
        "Route35GoldenrodGate" if return_for_bottle=>Exit("GoldenrodCity"),
        "Route35GoldenrodGate"=>Exit("Route35"),
        "Route35" if return_for_bottle=>Exit("Route35GoldenrodGate"),
        "Route35"=>Exit("Route35NationalParkGate"),
        "Route35NationalParkGate" if return_for_bottle=>Exit("Route35"),
        "Route35NationalParkGate"=>Exit("NationalPark"),
        "NationalPark" if return_for_bottle=>Exit("Route35NationalParkGate"),
        "NationalPark"=>Exit("Route36NationalParkGate"),
        "Route36NationalParkGate" if return_for_bottle=>Exit("NationalPark"),
        "Route36NationalParkGate"=>Exit("Route36"),
        "Route36" if !has(v,"EVENT_MET_FLORIA") && !has(v,"EVENT_FOUGHT_SUDOWOODO")=>Talk("Route36FloriaScript","Meet Floria beside the strange tree"),
        "Route36" if return_for_bottle=>Exit("Route36NationalParkGate"),
        "Route36" if !has(v,"EVENT_FOUGHT_SUDOWOODO")=>Talk("SudowoodoScript","Water the strange tree to open the northern route"),
        "Route36"=>Exit("Route37"),
        "Route37"=>Exit("EcruteakCity"),
        "EcruteakCity" if !has(v,"EVENT_GOT_HM03_SURF")=>Exit("DanceTheater"),
        "DanceTheater" if !has(v,"EVENT_GOT_HM03_SURF")=>{
            let next=[
                ("EVENT_BEAT_KIMONO_GIRL_NAOKO","TrainerKimonoGirlNaoko"),
                ("EVENT_BEAT_KIMONO_GIRL_SAYO","TrainerKimonoGirlSayo"),
                ("EVENT_BEAT_KIMONO_GIRL_ZUKI","TrainerKimonoGirlZuki"),
                ("EVENT_BEAT_KIMONO_GIRL_KUNI","TrainerKimonoGirlKuni"),
                ("EVENT_BEAT_KIMONO_GIRL_MIKI","TrainerKimonoGirlMiki"),
            ].into_iter().find(|(flag,_)|!has(v,flag));
            match next {
                Some((_,role))=>Talk(role,"Defeat the Kimono Girls to earn Surf"),
                None=>Talk("DanceTheaterSurfGuy","Receive Surf after defeating all five Kimono Girls"),
            }
        },
        "DanceTheater"=>Exit("EcruteakCity"),
        "EcruteakCity" if !has(v,"EVENT_RELEASED_THE_BEASTS")=>Exit("BurnedTower1F"),
        "BurnedTower1F" if !has(v,"EVENT_RELEASED_THE_BEASTS")=>Exit("BurnedTowerB1F"),
        "BurnedTowerB1F" if !has(v,"EVENT_RELEASED_THE_BEASTS")=>Visit("ReleaseTheBeasts","Approach the legendary beasts to reopen Morty's gym"),
        "BurnedTowerB1F"=>Exit("BurnedTower1F"),
        "BurnedTower1F"=>Exit("EcruteakCity"),
        "EcruteakCity" if !badge(v,3)=>Exit("EcruteakGym"),
        "EcruteakGym" if !has(v,"EVENT_RELEASED_THE_BEASTS")=>Exit("EcruteakCity"),
        "EcruteakGym" if !badge(v,3)=>Talk("MORTY","Defeat Morty for permission to use Surf"),
        "EcruteakGym"=>Exit("EcruteakCity"),
        // Continue west only after obtaining Surf and its field permission.
        // Door/boundary positions still come from the current map observation.
        "EcruteakCity"=>Exit("Route38EcruteakGate"),
        "Route38EcruteakGate" if !badge(v,3) || !has(v,"EVENT_GOT_HM03_SURF")=>Exit("EcruteakCity"),
        "Route38EcruteakGate"=>Exit("Route38"),
        "Route38" if !badge(v,3) || !has(v,"EVENT_GOT_HM03_SURF")=>Exit("Route38EcruteakGate"),
        "Route38"=>Exit("Route39"),
        "Route39"=>Exit("OlivineCity"),
        "OlivineCity" if !has(v,"EVENT_GOT_HM04_STRENGTH")=>Exit("OlivineCafe"),
        "OlivineCafe" if !has(v,"EVENT_GOT_HM04_STRENGTH")=>Talk("OlivineCafeStrengthSailorScript","Receive Strength from the sailor in Olivine Cafe"),
        "OlivineCafe"=>Exit("OlivineCity"),
        "OlivineCity" if has(v,"EVENT_JASMINE_RETURNED_TO_GYM") && !badge(v,4)=>Exit("OlivineGym"),
        "OlivineGym" if has(v,"EVENT_JASMINE_RETURNED_TO_GYM") && !badge(v,4)=>Talk("JASMINE","Defeat Jasmine after healing Amphy"),
        "OlivineGym"=>Exit("OlivineCity"),
        "OlivineLighthouse6F" if !has(v,"EVENT_JASMINE_RETURNED_TO_GYM") &&
            (!has(v,"EVENT_JASMINE_EXPLAINED_AMPHYS_SICKNESS") || v.pointer("/reward_state/items").and_then(Value::as_array).is_some_and(|items|items.iter().any(|i|i["id"]=="SECRETPOTION"&&i["quantity"].as_u64().unwrap_or(0)>0)))=>
            Talk("OlivineLighthouseJasmine","Speak to Jasmine about Amphy and deliver the medicine"),
        "OlivineLighthouse6F"=>Exit("OlivineLighthouse5F"),
        "CianwoodCity" if has(v,"EVENT_JASMINE_EXPLAINED_AMPHYS_SICKNESS") && !has(v,"EVENT_GOT_SECRETPOTION_FROM_PHARMACY")=>Exit("CianwoodPharmacy"),
        "CianwoodPharmacy" if has(v,"EVENT_JASMINE_EXPLAINED_AMPHYS_SICKNESS") && !has(v,"EVENT_GOT_SECRETPOTION_FROM_PHARMACY")=>Talk("CianwoodPharmacist","Collect Amphy's medicine after Jasmine's request"),
        "CianwoodPharmacy"=>Exit("CianwoodCity"),
        _=>return None,
    })
}
fn clean(s:&str)->String { s.chars().filter(|c|c.is_ascii_alphanumeric()).flat_map(char::to_uppercase).collect() }
pub(crate) fn matches_object(o:&Value,needle:&str)->bool {
    ["name","curriculum_role"].iter().any(|k|o[*k].as_str().is_some_and(|s|clean(s).contains(&clean(needle))))
}
/// Only observed counter permissions permit speaking across an intervening tile.
pub(crate) fn counter_at(v:&Value,x:i64,y:i64)->bool {
    let Some(t)=v.pointer("/map_info/terrain") else{return false;};
    let (Some(ox),Some(oy))=(t["origin_x"].as_i64(),t["origin_y"].as_i64()) else{return false;};
    let (Ok(col),Ok(row))=(usize::try_from(x-ox),usize::try_from(y-oy)) else{return false;};
    t["rows"].as_array().and_then(|rs|rs.get(row)).and_then(Value::as_array).and_then(|cs|cs.get(col))
        .and_then(|c|c["permission"].as_u64()).is_some_and(|p|p==0x90||p==0x98)
}
pub(crate) fn pc_tiles(v:&Value)->Vec<(i64,i64)> {interaction_tiles(v,&[0x93])}
pub(crate) fn cut_tiles(v:&Value)->Vec<(i64,i64)> {interaction_tiles(v,&[0x12,0x1a])}
fn interaction_tiles(v:&Value,permissions:&[u64])->Vec<(i64,i64)> {
    let Some(t)=v.pointer("/map_info/terrain") else{return vec![];};
    let (Some(x),Some(y),Some(rows))=(t["origin_x"].as_i64(),t["origin_y"].as_i64(),t["rows"].as_array()) else{return vec![];};
    rows.iter().enumerate().flat_map(|(dy,row)|row.as_array().into_iter().flatten().enumerate()
        .filter(|(_,c)|c["permission"].as_u64().is_some_and(|p|permissions.contains(&p))).map(move |(dx,_)|(x+dx as i64,y+dy as i64))).collect()
}
// Declared puzzle assistance from the visible bird position. These sides
// follow the authored chase branches; no script memory or inputs are changed.
// Reference: pret/pokecrystal maps/IlexForest.asm, validated by the engine's
// farfetchd_chase_handles_a_wrong_approach_then_cut_and_charcoal_rewards test.
pub(crate) fn farfetchd_phase(o:&Value)->Option<usize>{
    let point=(o["x"].as_i64()?,o["y"].as_i64()?);
    [(14,31),(15,25),(20,24),(29,22),(28,31),(24,35),(22,31),(15,29),(10,35),(6,28)]
        .iter().position(|p|*p==point).map(|i|i+1)
}
pub(crate) fn farfetchd_approaches(o:&Value)->Option<Vec<(i64,i64)>>{
    let phase=farfetchd_phase(o)?;let x=o["x"].as_i64()?;let y=o["y"].as_i64()?;
    let offsets=[(-1,0),(1,0),(0,-1),(0,1)];
    Some(offsets.into_iter().filter(|&(dx,dy)|match phase{
        3=>(dx,dy)!=(1,0), 4=>(dx,dy)!=(0,1),
        5=>[(1,0),(0,-1)].contains(&(dx,dy)), 6=>(dx,dy)!=(-1,0),
        7=>[(-1,0),(0,1)].contains(&(dx,dy)), 8=>(dx,dy)==(0,-1),
        9=>[(1,0),(0,1)].contains(&(dx,dy)), _=>true,
    }).map(|(dx,dy)|(x+dx,y+dy)).collect())
}
pub(crate) fn chase_progress(before:&Value,after:&Value)->f32{
    if before.pointer("/map_info/name").and_then(Value::as_str)!=Some("IlexForest")
        || after.pointer("/map_info/name").and_then(Value::as_str)!=Some("IlexForest")
        || after.pointer("/flow_state/animating")==Some(&Value::Bool(true)){return 0.0;}
    let phase=|v:&Value|v.pointer("/map_info/objects").and_then(Value::as_array)
        .and_then(|a|a.iter().find(|o|matches_object(o,"FARFETCHD"))).and_then(farfetchd_phase);
    match (phase(before),phase(after)){(Some(a),Some(b))=>0.2*(b as f32-a as f32),_=>0.0}
}
pub(crate) fn target(v:&Value)->Option<(String,Vec<(i64,i64)>)> {
    match goal(v)? {
        Goal::CutTree=>{
            let points=cut_tiles(v).into_iter().flat_map(|(x,y)|[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]).collect::<Vec<_>>();
            (!points.is_empty()).then(||("Use Cut to open the forest route".into(),points))
        }
        Goal::Pc=>{
            let points=pc_tiles(v).into_iter().flat_map(|(x,y)|[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]).collect::<Vec<_>>();
            (!points.is_empty()).then(||((if needs_egg_space(v){"Make room for the Togepi Egg"}else{"Withdraw a compatible HM user from the PC"}).into(),points))
        }
        Goal::Visit(script,label)=>{
            let points=v.pointer("/map_info/curriculum_events")?.as_array()?.iter()
                .filter(|e|e["script"].as_str()==Some(script))
                .filter_map(|e|Some((e["x"].as_i64()?,e["y"].as_i64()?)))
                .collect::<Vec<_>>();
            (!points.is_empty()).then(||(label.into(),points))
        }
        Goal::Talk(needle,label)=>{
            if needle=="FARFETCHD" && v.pointer("/map_info/name").and_then(Value::as_str)==Some("IlexForest") {
                let bird=v.pointer("/map_info/objects")?.as_array()?.iter().find(|o|matches_object(o,needle))?;
                if let Some(points)=farfetchd_approaches(bird){return Some((format!("{label}: chase stage {}",farfetchd_phase(bird)?),points));}
            }
            let points=v.pointer("/map_info/objects")?.as_array()?.iter()
                .filter(|o|matches_object(o,needle))
                .filter_map(|o|Some((o["x"].as_i64()?,o["y"].as_i64()?)))
                .flat_map(|(x,y)|{
                    let mut points=vec![(x-1,y),(x+1,y),(x,y-1),(x,y+1)];
                    for (dx,dy) in [(-1,0),(1,0),(0,-1),(0,1)] {
                        if counter_at(v,x+dx,y+dy){points.push((x+2*dx,y+2*dy));}
                    }
                    points
                }).collect::<Vec<_>>();
            (!points.is_empty()).then(||(label.into(),points))
        }
        Goal::Exit(destination)=>{
            let width=v.pointer("/map_info/dimensions/0")?.as_i64()?;
            let height=v.pointer("/map_info/dimensions/1")?.as_i64()?;
            if width<=0||height<=0||width*height>20000{return None;}
            let mut points=Vec::new();
            for e in v.pointer("/map_info/curriculum_exits")?.as_array()?.iter().filter(|e|e["target"].as_str().is_some_and(|s|clean(s)==clean(destination))) {
                if e["kind"]=="warp" {if let (Some(x),Some(y))=(e["x"].as_i64(),e["y"].as_i64()){points.push((x,y));}}
                else {match e["direction"].as_str().unwrap_or("").to_lowercase().as_str(){
                    "north"=>points.extend((0..width).map(|x|(x,0))),
                    "south"=>points.extend((0..width).map(|x|(x,height-1))),
                    "west"=>points.extend((0..height).map(|y|(0,y))),
                    "east"=>points.extend((0..height).map(|y|(width-1,y))),_=>{}
                }}
            }
            (!points.is_empty()).then(||(format!("Travel to {destination}"),points))
        }
    }
}
/// Continue across the boundary after the path distance reaches zero.
/// Only an authored connection or edge doorway for the active destination supplies this cue.
pub(crate) fn outward(v: &Value) -> Option<&'static str> {
    let Goal::Exit(destination) = goal(v)? else { return None; };
    let x = v.pointer("/map_info/player/x")?.as_i64()?;
    let y = v.pointer("/map_info/player/y")?.as_i64()?;
    let width = v.pointer("/map_info/dimensions/0")?.as_i64()?;
    let height = v.pointer("/map_info/dimensions/1")?.as_i64()?;
    if x < 0 || y < 0 || x >= width || y >= height { return None; }
    v.pointer("/map_info/curriculum_exits")?.as_array()?.iter()
        .filter(|e| e["target"].as_str().is_some_and(|s| clean(s) == clean(destination)))
        .find_map(|e| {
            if e["kind"]=="warp" && e["x"].as_i64()==Some(x) && e["y"].as_i64()==Some(y) {
                return if y==height-1{Some("south")}else if y==0{Some("north")}else if x==0{Some("west")}else if x==width-1{Some("east")}else{None};
            }
            if e["kind"]!="connection"{return None;}
            match e["direction"].as_str()?.to_ascii_lowercase().as_str() {
            "north" if y == 0 => Some("north"),
            "south" if y == height - 1 => Some("south"),
            "west" if x == 0 => Some("west"),
            "east" if x == width - 1 => Some("east"),
            _ => None,
        }})
}
#[cfg(test)] mod tests {
    use super::*;use serde_json::json;
    fn state(map:&str)->Value {json!({"map_info":{"name":map,"dimensions":[20,20],"objects":[],"curriculum_exits":[]},"status":{"badges":{"johto":vec![false;8]}},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]}})}
    #[test]
    fn medicine_route_requires_request_and_returns_to_jasmine_only_with_medicine() {
        let mut v=serde_json::json!({"map_info":{"name":"CianwoodPharmacy"},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]}});
        assert_eq!(goal(&v),Some(Goal::Exit("CianwoodCity")));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(serde_json::json!("EVENT_JASMINE_EXPLAINED_AMPHYS_SICKNESS"));
        assert!(matches!(goal(&v),Some(Goal::Talk("CianwoodPharmacist",_))));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(serde_json::json!("EVENT_GOT_SECRETPOTION_FROM_PHARMACY"));
        assert_eq!(goal(&v),Some(Goal::Exit("CianwoodCity")));
        v["map_info"]["name"]=serde_json::json!("OlivineLighthouse6F");
        assert_eq!(goal(&v),Some(Goal::Exit("OlivineLighthouse5F")));
        v["reward_state"]["items"]=serde_json::json!([{"id":"SECRETPOTION","quantity":1}]);
        assert!(matches!(goal(&v),Some(Goal::Talk("OlivineLighthouseJasmine",_))));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(serde_json::json!("EVENT_JASMINE_RETURNED_TO_GYM"));
        assert_eq!(goal(&v),Some(Goal::Exit("OlivineLighthouse5F")));
        v["map_info"]["name"]=serde_json::json!("OlivineGym");
        assert!(matches!(goal(&v),Some(Goal::Talk("JASMINE",_))));
        v["status"]=serde_json::json!({"badges":{"johto":[true,true,true,true,true,false]}});
        assert_eq!(goal(&v),Some(Goal::Exit("OlivineCity")));
    }
    #[test]
    fn westward_hm_journey_preserves_prerequisites_and_finishes_the_handoff() {
        let mut v=serde_json::json!({"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM","EVENT_GOT_HM03_SURF","EVENT_RELEASED_THE_BEASTS"]},"status":{"badges":{"johto":[true,true,true,false]},"party":[]},"map_info":{"name":"Route38EcruteakGate"}});
        assert_eq!(goal(&v),Some(Goal::Exit("EcruteakCity")));
        v["status"]["badges"]["johto"][3]=serde_json::json!(true);
        assert_eq!(goal(&v),Some(Goal::Exit("Route38")));
        v["map_info"]=serde_json::json!({"name":"OlivineCafe","dimensions":[10,8],"objects":[{"name":"SAILOR_1","curriculum_role":"OlivineCafeStrengthSailorScript","x":4,"y":3}],"curriculum_exits":[{"kind":"warp","target":"OLIVINE_CITY","x":2,"y":7}]});
        let (label,tiles)=target(&v).unwrap();
        assert!(label.contains("Strength"));assert!(tiles.contains(&(3,3)));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(serde_json::json!("EVENT_GOT_HM04_STRENGTH"));
        assert_eq!(target(&v),Some(("Travel to OlivineCity".into(),vec![(2,7)])));
        v["map_info"]["name"]=serde_json::json!("OlivineCity");
        v["status"]["party"]=serde_json::json!([{"hp":5,"max_hp":40}]);
        assert_eq!(goal(&v),Some(Goal::Exit("OlivinePokecenter1F")));
    }
    #[test]
    fn chase_sides_exclude_backtracking_and_reward_only_net_progress(){
        let observation=|x,y|json!({"map_info":{"name":"IlexForest","objects":[{"name":"ILEXFOREST_FARFETCHD","x":x,"y":y}]},"flow_state":{"animating":false},"reward_state":{"event_flags":["EVENT_GAVE_MYSTERY_EGG_TO_ELM"]}});
        for (x,y,forbidden) in [(20,24,vec![(21,24)]),(29,22,vec![(29,23)]),(28,31,vec![(27,31),(28,32)]),(24,35,vec![(23,35)]),(22,31,vec![(23,31),(22,30)]),(15,29,vec![(14,29),(16,29),(15,30)]),(10,35,vec![(9,35),(10,34)])]{
            let v=observation(x,y);let (_,points)=target(&v).unwrap();
            for point in forbidden {assert!(!points.contains(&point),"wrong side {point:?} at {x},{y}");}
        }
        let a=observation(29,22);let b=observation(28,31);
        assert!(chase_progress(&a,&b)>0.0);assert_eq!(chase_progress(&a,&b)+chase_progress(&b,&a),0.0);
        assert_eq!(chase_progress(&a,&a),0.0);
        let mut moving=b.clone();moving["flow_state"]["animating"]=json!(true);assert_eq!(chase_progress(&a,&moving),0.0);
        let start=observation(14,31);let returned=observation(6,28);assert!((chase_progress(&start,&returned)-1.8).abs()<0.00001);
        assert!(target(&a).unwrap().0.ends_with("stage 4"));
        assert_ne!(target(&a).unwrap().0,target(&b).unwrap().0);
    }
    #[test] fn edge_doorway_keeps_outward_cue_until_the_real_map_transition() {
        let mut v=state("VioletGym");
        v["status"]["party"]=json!([{"hp":15,"max_hp":43}]);
        v["map_info"]["dimensions"]=json!([10,16]);v["map_info"]["player"]=json!({"x":5,"y":15});
        v["map_info"]["curriculum_exits"]=json!([{"kind":"warp","target":"VIOLET_CITY","x":5,"y":15}]);
        assert_eq!(outward(&v),Some("south"));
        v["map_info"]["player"]["x"]=json!(4);assert_eq!(outward(&v),None);
        v["map_info"]["player"]["x"]=json!(5);v["map_info"]["curriculum_exits"][0]["target"]=json!("OTHER_CITY");assert_eq!(outward(&v),None);
        v["map_info"]["curriculum_exits"][0]["target"]=json!("VIOLET_CITY");
        v["map_info"]["player"]["y"]=json!(14);v["map_info"]["curriculum_exits"][0]["y"]=json!(14);assert_eq!(outward(&v),None);
    }
    #[test] fn injured_gym_party_returns_to_nurse_before_another_challenge() {
        let mut v=state("VioletGym");
        v["status"]["party"]=serde_json::json!([{"slot":0,"hp":15,"max_hp":43,"is_egg":false}]);
        for (gym,city,center) in [("VioletGym","VioletCity","VioletPokecenter1F"),("AzaleaGym","AzaleaTown","AzaleaPokecenter1F"),("GoldenrodGym","GoldenrodCity","GoldenrodPokecenter1F"),("EcruteakGym","EcruteakCity","EcruteakPokecenter1F")] {
            v["map_info"]["name"]=serde_json::json!(gym);assert_eq!(goal(&v),Some(Goal::Exit(city)));
            v["map_info"]["name"]=serde_json::json!(city);assert_eq!(goal(&v),Some(Goal::Exit(center)));
            v["map_info"]["name"]=serde_json::json!(center);assert!(matches!(goal(&v),Some(Goal::Talk("NURSE",_))));
        }
        v["status"]["party"][0]["hp"]=serde_json::json!(43);v["map_info"]["name"]=serde_json::json!("VioletGym");
        assert!(matches!(goal(&v),Some(Goal::Talk("FALKNER",_))));
    }
    #[test] fn heal_before_gym_then_resume_story_without_premature_egg_request() {
        let mut v=state("VioletCity");v["status"]["party"]=json!([{"hp":19,"max_hp":36,"is_egg":false,"moves":[{"current_pp":16}]}]);
        assert_eq!(goal(&v),Some(Goal::Exit("VioletPokecenter1F")));
        v["map_info"]["name"]=json!("VioletPokecenter1F");assert!(matches!(goal(&v),Some(Goal::Talk("NURSE",_))));
        v["status"]["party"][0]["hp"]=json!(36);assert_eq!(goal(&v),Some(Goal::Exit("VioletCity")));
        v["map_info"]["name"]=json!("VioletCity");assert_eq!(goal(&v),Some(Goal::Exit("VioletGym")));
        v["status"]["party"][0]["moves"][0]["current_pp"]=json!(0);assert!(needs_healing(&v));
        v["status"]["party"][0]["is_egg"]=json!(true);assert!(!needs_healing(&v));
    }
    #[test] fn bottle_quest_returns_to_floria_then_resumes_north() {
        let mut v=state("GoldenrodCity");
        v["status"]["badges"]["johto"][2]=json!(true);
        assert_eq!(goal(&v),Some(Goal::Exit("Route35GoldenrodGate")));
        v["map_info"]["name"]=json!("Route36");
        assert!(matches!(goal(&v),Some(Goal::Talk("Route36FloriaScript",_))));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_MET_FLORIA"));
        for (map,destination) in [("Route36","Route36NationalParkGate"),("Route36NationalParkGate","NationalPark"),("NationalPark","Route35NationalParkGate"),("Route35NationalParkGate","Route35"),("Route35","Route35GoldenrodGate"),("Route35GoldenrodGate","GoldenrodCity"),("GoldenrodCity","GoldenrodFlowerShop")] {
            v["map_info"]["name"]=json!(map);
            assert_eq!(goal(&v),Some(Goal::Exit(destination)),"{map}");
        }
        v["map_info"]["name"]=json!("GoldenrodFlowerShop");
        assert!(matches!(goal(&v),Some(Goal::Talk("FlowerShopFloriaScript",_))));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_TALKED_TO_FLORIA_AT_FLOWER_SHOP"));
        assert!(matches!(goal(&v),Some(Goal::Talk("FlowerShopTeacherScript",_))));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_GOT_SQUIRTBOTTLE"));
        assert_eq!(goal(&v),Some(Goal::Exit("GoldenrodCity")));
        v["map_info"]["name"]=json!("Route36");
        assert!(matches!(goal(&v),Some(Goal::Talk("SudowoodoScript",_))));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_FOUGHT_SUDOWOODO"));
        assert_eq!(goal(&v),Some(Goal::Exit("Route37")));
    }
    #[test] fn forest_cut_requires_badge_known_move_and_an_observed_tree() {
        let mut v=state("IlexForest");
        v["reward_state"]["event_flags"].as_array_mut().unwrap().extend([json!("EVENT_HERDED_FARFETCHD"),json!("EVENT_GOT_HM01_CUT")]);
        v["map_info"]["terrain"]=json!({"origin_x":2,"origin_y":3,"rows":[[{"permission":18}]]});
        v["status"]["party"]=json!([{"moves":[{"name":"CUT"}]}]);
        assert_eq!(goal(&v),Some(Goal::Exit("Route34IlexForestGate")));
        v["status"]["badges"]["johto"][1]=json!(true);assert_eq!(goal(&v),Some(Goal::CutTree));
        assert!(target(&v).unwrap().1.contains(&(2,4)));
        v["map_info"]["terrain"]["rows"][0][0]["permission"]=json!(0);
        assert_eq!(goal(&v),Some(Goal::Exit("Route34IlexForestGate")));
    }
    #[test] fn boxed_hm_detour_uses_observed_pc_and_preserves_healing_priority() {
        let mut v=state("VioletCity");
        v["status"]["screen"]=json!("overworld");
        v["map_info"]["hm_compatibility"]=json!({"HM_CUT":[{"able":false}]});
        v["reward_state"]["boxed_hm_candidates"]=json!({"HM_CUT":{"box":0,"position":0}});
        assert_eq!(goal(&v),Some(Goal::Exit("VioletPokecenter1F")));
        v["map_info"]["name"]=json!("VioletPokecenter1F");
        assert_eq!(goal(&v),Some(Goal::Pc));assert!(target(&v).is_none());
        v["map_info"]["terrain"]=json!({"origin_x":8,"origin_y":1,"rows":[[{"permission":147}]]});
        assert!(target(&v).unwrap().1.contains(&(8,2)));
        v["status"]["party"]=json!([{"hp":1,"max_hp":40}]);
        assert!(matches!(goal(&v),Some(Goal::Talk("NURSE",_))));
    }
    #[test] fn whitney_requires_walking_away_then_returning_for_the_badge() {
        let mut v=state("GoldenrodGym");
        assert!(matches!(goal(&v),Some(Goal::Talk("WHITNEY",_))));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().extend([json!("EVENT_BEAT_WHITNEY"),json!("EVENT_MADE_WHITNEY_CRY")]);
        assert!(matches!(goal(&v),Some(Goal::Visit("WhitneyCriesScript",_))));
        assert!(target(&v).is_none(),"Require authored event coordinates");
        v["map_info"]["curriculum_events"]=json!([{"script":"WhitneyCriesScript","x":8,"y":5}]);
        assert_eq!(target(&v).unwrap().1,vec![(8,5)]);
        v["reward_state"]["event_flags"].as_array_mut().unwrap().retain(|f|f!="EVENT_MADE_WHITNEY_CRY");
        assert!(matches!(goal(&v),Some(Goal::Talk("WHITNEY",_))));
        v["status"]["badges"]["johto"]=json!([true,true,true,false,false,false,false,false]);
        assert_eq!(goal(&v),Some(Goal::Exit("GoldenrodCity")));
    }
    #[test] fn surf_permission_requires_beasts_then_morty_badge() {
        let mut v=state("EcruteakCity");
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_GOT_HM03_SURF"));
        assert_eq!(goal(&v),Some(Goal::Exit("BurnedTower1F")));
        v["map_info"]["name"]=json!("BurnedTowerB1F");
        assert!(matches!(goal(&v),Some(Goal::Visit("ReleaseTheBeasts",_))));
        assert!(target(&v).is_none(),"Do not invent coordinates without authored telemetry");
        v["map_info"]["curriculum_events"]=json!([{"script":"ReleaseTheBeasts","x":10,"y":6}]);
        assert_eq!(target(&v).unwrap().1,vec![(10,6)]);
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_RELEASED_THE_BEASTS"));
        assert_eq!(goal(&v),Some(Goal::Exit("BurnedTower1F")));
        v["map_info"]["name"]=json!("EcruteakCity");
        assert_eq!(goal(&v),Some(Goal::Exit("EcruteakGym")));
        v["map_info"]["name"]=json!("EcruteakGym");
        assert!(matches!(goal(&v),Some(Goal::Talk("MORTY",_))));
        v["status"]["badges"]["johto"][3]=json!(true);
        assert_eq!(goal(&v),Some(Goal::Exit("EcruteakCity")));
    }
    #[test] fn surf_reward_requires_all_five_distinct_trainers() {
        let mut v=state("DanceTheater");
        for name in ["NAOKO","SAYO","ZUKI","KUNI","MIKI"] {
            assert!(matches!(goal(&v),Some(Goal::Talk(role,_)) if role.to_uppercase().ends_with(name)));
            v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!(format!("EVENT_BEAT_KIMONO_GIRL_{name}")));
        }
        assert!(matches!(goal(&v),Some(Goal::Talk("DanceTheaterSurfGuy",_))));
        v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_GOT_HM03_SURF"));
        assert_eq!(goal(&v),Some(Goal::Exit("EcruteakCity")));
    }
    #[test] fn later_route_boundary_keeps_the_exit_cue_until_transition() {
        let mut v=state("Route33");
        v["map_info"]["player"]=json!({"x":0,"y":8});
        v["map_info"]["curriculum_exits"]=json!([
            {"kind":"connection","direction":"East","target":"UNION_CAVE_1F"},
            {"kind":"connection","direction":"West","target":"AZALEA_TOWN"}
        ]);
        assert_eq!(outward(&v),Some("west"));
        let cues=crate::breadcrumbs::Breadcrumbs::default().cues(&v);
        assert!(cues.contains(&"story-scent:west".into()));
        v["map_info"]["player"]["x"]=json!(1);
        assert_eq!(outward(&v),None);
        v["map_info"]["player"]["x"]=json!(19);
        assert_eq!(outward(&v),None);
        v["map_info"]["name"]=json!("AzaleaTown");
        assert_eq!(outward(&v),None);
    }
    #[test] fn gym_completion_changes_the_route(){let mut v=state("VioletCity");assert_eq!(goal(&v),Some(Goal::Exit("VioletGym")));v["status"]["badges"]["johto"][0]=json!(true);assert_eq!(goal(&v),Some(Goal::Exit("VioletPokecenter1F")));v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_GOT_TOGEPI_EGG_FROM_ELMS_AIDE"));assert_eq!(goal(&v),Some(Goal::Exit("Route32")));}
    #[test] fn restocking_returns_to_story_and_preserves_healing_priority(){
        let mut v=state("AzaleaTown");v["status"]["money"]=json!(1000);v["reward_state"]["items"]=json!([]);
        assert_eq!(goal(&v),Some(Goal::Exit("AzaleaMart")));
        v["map_info"]["name"]=json!("AzaleaMart");assert!(matches!(goal(&v),Some(Goal::Talk("CLERK",_))));
        v["reward_state"]["items"]=json!([{"id":"POKE_BALL","quantity":3}]);assert_eq!(goal(&v),Some(Goal::Exit("AzaleaTown")));
        v["map_info"]["name"]=json!("AzaleaTown");assert_ne!(goal(&v),Some(Goal::Exit("AzaleaMart")));
        v["reward_state"]["items"]=json!([]);v["status"]["money"]=json!(199);assert_ne!(goal(&v),Some(Goal::Exit("AzaleaMart")));
        v["status"]["money"]=json!(1000);v["status"]["party"]=json!([{"hp":5,"max_hp":40,"is_egg":false}]);assert_eq!(goal(&v),Some(Goal::Exit("AzaleaPokecenter1F")));
        v["map_info"]["name"]=json!("UnionCave1F");assert_eq!(goal(&v),Some(Goal::Exit("Route33")));
    }
    #[test] fn cut_requires_the_completed_chase(){let mut v=state("IlexForest");assert!(matches!(goal(&v),Some(Goal::Talk("FARFETCHD",_))));v["reward_state"]["event_flags"].as_array_mut().unwrap().push(json!("EVENT_HERDED_FARFETCHD"));assert!(matches!(goal(&v),Some(Goal::Talk("CHARCOAL_MASTER",_))));}
    #[test] fn authored_exits_supply_coordinates_and_missing_exits_are_not_guessed(){let mut v=state("Route31");assert!(target(&v).is_none());v["map_info"]["curriculum_exits"]=json!([{"kind":"warp","target":"ROUTE_31_VIOLET_GATE","x":4,"y":7}]);assert_eq!(target(&v).unwrap().1,vec![(4,7)]);}
}
