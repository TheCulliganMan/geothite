//! Visible shop objectives; purchases must change inventory and spend money.
use serde_json::Value;
fn menu(v:&Value)->Option<&Value>{v.pointer("/observe/menus")?.as_array()?.last().filter(|m|m["kind"]=="shop")}
fn money(v:&Value)->Option<u64>{v.pointer("/status/money")?.as_u64()}
fn ball(s:&str)->bool{let s=s.to_uppercase().replace('É',"E").replace([' ','_'],"").replace('#',"POKE");["POKEBALL","GREATBALL","ULTRABALL"].iter().any(|b|s.starts_with(b))}
fn stock(v:&Value)->Option<u64>{Some(v.pointer("/reward_state/items")?.as_array()?.iter().filter(|i|i["id"].as_str().is_some_and(ball)).map(|i|i["quantity"].as_u64().unwrap_or(0)).sum())}
pub(crate) fn needs_restock(v:&Value)->bool {
    matches!((stock(v),money(v)),(Some(n),Some(cash)) if n<3&&cash>=200)
}
fn label(s:&str)->String{s.trim().trim_start_matches('>').trim().to_uppercase()}
fn target(v:&Value)->Option<(&'static str,usize,usize)> {
 let m=menu(v)?;let rows=m["entries"].as_array()?;let cursor=rows.iter().position(|s|s.as_str().is_some_and(|s|s.trim().starts_with('>')))?;
 let need=stock(v)?<3;
 let wanted=match m["surface"].as_str()? {
  "top"=>if need&&money(v)?>=200{"BUY"}else{"QUIT"},
  "confirm"=>if need&&m["selling"]==false&&m["item"].as_str().is_some_and(ball)&&m["total_price"].as_u64()?<=money(v)?&&m["quantity"].as_u64()?<=3-stock(v)?{"YES"}else{"NO"},
  "buy"=>{
   if need {if let Some(i)=rows.iter().position(|s|s.as_str().is_some_and(|s|ball(&label(s)))) {return Some(("select_ball",i,cursor));}}
   "CANCEL"
  },
  _=>return None,
 };
 Some((if wanted=="BUY"{"buy"}else if wanted=="YES"{"confirm_purchase"}else{"leave"},rows.iter().position(|s|s.as_str().is_some_and(|s|label(s)==wanted))?,cursor))
}
fn quantity(v:&Value)->Option<(u64,u64)>{
 let m=menu(v)?;if m["surface"]!="quantity"||m["selling"]!=false||!m["item"].as_str().is_some_and(ball){return None;}
 let price=m["unit_price"].as_u64()?;if price==0{return None;}
 let desired=(3u64.saturating_sub(stock(v)?)).min(money(v)?/price);
 (desired>0).then_some((m["quantity"].as_u64()?,desired))
}
pub(crate) fn cues(v:&Value)->Vec<String>{
 if let Some((stage,target,cursor))=target(v){return vec![format!("objective:shop:{stage}"),format!("objective:menu:{}",if target==cursor{"aligned"}else if target<cursor{"north"}else{"south"})];}
 if let Some((current,desired))=quantity(v){return vec!["objective:shop:quantity".into(),format!("objective:quantity:{}",if current==desired{"aligned"}else if current<desired{"increase"}else{"decrease"})];}
 if menu(v).is_some(){vec!["objective:shop:read_or_cancel".into()]}else{vec![]}
}
pub(crate) fn context(v:&Value)->Option<String>{let m=menu(v)?;Some(format!("shop:{}:{}",m["surface"].as_str()?,cues(v).join("|")))}
fn potential(v:&Value)->f32{
 if let Some((stage,t,c))=target(v){let base=match stage{"buy"=>0.5,"select_ball"=>1.0,"confirm_purchase"=>2.0,_=>0.0};return base-0.1*t.abs_diff(c) as f32;}
 if let Some((q,w))=quantity(v){return 1.5-0.1*q.abs_diff(w) as f32;}
 0.0
}
pub(crate) fn feedback(before:&Value,after:&Value,button:&str)->f32{
 if menu(before).is_none()&&menu(after).is_none(){return 0.0;}
 let purchased=button=="a"&&target(before).is_some_and(|(s,t,c)|s=="confirm_purchase"&&t==c)
  &&matches!((stock(before),stock(after),money(before),money(after)),(Some(a),Some(b),Some(x),Some(y)) if b>a&&y<x&&Some(b-a)==menu(before).and_then(|m|m["quantity"].as_u64())&&Some(x-y)==menu(before).and_then(|m|m["total_price"].as_u64()));
 0.2*(potential(after)-potential(before))+if purchased{1.0}else{0.0}
}
#[cfg(test)] mod tests{
 use super::*;use serde_json::json;
 fn view(surface:&str,rows:Value)->Value{json!({"status":{"money":1000},"reward_state":{"items":[]},"observe":{"menus":[{"kind":"shop","surface":surface,"entries":rows,"item":"POKE_BALL","selling":false,"unit_price":200,"quantity":3,"total_price":600}]}})}
 #[test]fn purchase_requires_inventory_and_payment(){let a=view("confirm",json!([">YES"," NO"]));let mut b=view("notice",json!([]));b["status"]["money"]=json!(400);b["reward_state"]["items"]=json!([{"id":"POKE_BALL","quantity":3}]);assert!(feedback(&a,&b,"a")>0.0);assert!(feedback(&a,&b,"b")<0.0);b["status"]["money"]=json!(1000);assert!(feedback(&a,&b,"a")<0.0);}
 #[test]fn quantity_respects_budget_and_confirmation(){let mut a=view("quantity",json!([]));a["status"]["money"]=json!(400);assert_eq!(quantity(&a),Some((3,2)));a["observe"]["menus"][0]["surface"]=json!("confirm");a["observe"]["menus"][0]["entries"]=json!([">YES"," NO"]);assert_eq!(target(&a).unwrap().1,1);}
 #[test]fn menus_do_not_pay_for_round_trips(){let a=view("top",json!([">BUY"," SELL"," QUIT"]));let b=view("buy",json!([">POKE BALL ¥200"," POTION ¥300"," CANCEL"]));assert!((feedback(&a,&b,"a")+feedback(&b,&a,"b")).abs()<1e-6);}
}
