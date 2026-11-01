use anyhow::{Context,Result,ensure};
use serde_json::{Value,json};
use std::{collections::BTreeMap,path::Path};
use super::source_syntax::{code_part,source_arguments};
#[path = "source_decorations.rs"]
mod source_decorations;

pub(super) fn export(root:&Path)->Result<()> {
    let vendor=root.join("vendor/pokecrystal");let data=root.join("apps/web/assets/data");
    let constants:Value=serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let number=|name:&str|constants["global"][name].as_i64().with_context(||format!("missing source constant {name}"));
    let mut decorations=BTreeMap::new();let mut index=0i64;
    for line in std::fs::read_to_string(vendor.join("data/decorations/attributes.asm"))?.lines().map(code_part) {
        if let Some(raw)=line.strip_prefix("decoration ") {let args=source_arguments(raw)?;ensure!(args.len()==5,"source decoration fields");decorations.insert(index,args[3].clone());index+=1;}
    }
    let value = serde_json::to_value(source_decorations::export(&vendor, &constants["global"])?)?;
    write(&data,"decorations",&value)?;
    let mut random=Vec::new();let mut progression=Vec::new();let mut section="";
    let source=std::fs::read_to_string(vendor.join("data/items/mom_phone.asm"))?;
    for line in source.lines().map(code_part) {
        if line=="MomItems_1:" {section="random";}if line=="MomItems_2:" {section="progression";}
        let Some(raw)=line.strip_prefix("momitem ") else {continue};let args=source_arguments(raw)?;ensure!(args.len()==4,"source Mom purchase fields");
        let doll=args[2]=="MOM_DOLL";ensure!(doll||args[2]=="MOM_ITEM","source Mom purchase kind");
        let flag=if doll {Some(decorations.get(&number(&args[3])?).context("source doll event flag")?.clone())}else{None};
        let rule=json!({"trigger":args[0].parse::<u32>()?,"cost":args[1].parse::<u32>()?,"kind":if doll {"doll"}else{"item"},"target":args[3],"decoration_flag":flag});
        match section {"random"=>random.push(rule),"progression"=>progression.push(rule),_=>anyhow::bail!("Mom purchase without source table")}
    }
    let battle=std::fs::read_to_string(vendor.join("engine/battle/core.asm"))?;
    ensure!(battle.contains("\tld a, 7\n\tldh [hDivisor], a") && battle.contains("BoostExp:\n; Multiply experience by 1.5x"),"pinned source experience calculation changed");
    let reward=json!({"max_level":number("MAX_LEVEL")?,"wild_exp_divisor":7,"trainer_exp_numerator":3,"trainer_exp_denominator":2,"mom_money_increment":number("MOM_MONEY")?,"mom_random_items":random,"mom_progression_items":progression});
    let _:crystal_core::systems::battle_rewards::BattleRewardRules=serde_json::from_value(reward.clone())?;
    write(&data,"battle_reward_rules",&reward)?;
    ensure!(battle.contains("\tld a, 32\n\tldh [hMultiplier], a") && battle.contains("\tld b, 30\n\tldh a, [hQuotient + 3]"),"pinned source escape calculation changed");
    let escape=json!({"player_speed_multiplier":32,"enemy_speed_divisor":4,"failed_attempt_bonus":30,"rng_roll_values":256});
    let _:crystal_core::systems::battle_escape::BattleEscapeRules=serde_json::from_value(escape.clone())?;
    write(&data,"battle_escape_rules",&escape)?;
    let events=std::fs::read_to_string(vendor.join("engine/overworld/events.asm"))?;
    let happiness=std::fs::read_to_string(vendor.join("engine/events/happiness_egg.asm"))?;
    let breeding=std::fs::read_to_string(vendor.join("engine/pokemon/breeding.asm"))?;
    ensure!(events.contains("\tld hl, wPoisonStepCount\n\tld a, [hl]\n\tcp 4") && events.contains("\tld a, [wStepCount]\n\tcp $80") && happiness.contains("\tinc a\n\tand 1") && breeding.contains("\tld [hl], $78"),"pinned source step rules changed");
    let steps=json!({"poison_step_interval":4,"egg_step_trigger":128,"hatched_egg_happiness":120,"poison_status":"POISON","egg_nickname":"EGG","happiness_step_counter_mask":1,"happiness_step_counter_target":0});
    let _:crystal_core::systems::step_events::StepEventRules=serde_json::from_value(steps.clone())?;
    write(&data,"step_event_rules",&steps)?;
    Ok(())
}
fn write(data:&Path,category:&str,value:&Value)->Result<()> {
    let modular=data.join("content-packs/core-modular").join(category);std::fs::create_dir_all(&modular)?;
    let bytes=serde_json::to_vec_pretty(value)?;std::fs::write(data.join(format!("{category}.json")),&bytes)?;std::fs::write(modular.join("rules.json"),bytes)?;Ok(())
}
