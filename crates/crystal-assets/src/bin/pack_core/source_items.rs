//! Source-only item catalog generation. Outputs belong to the external workspace.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn code(line: &str) -> &str {
    line.split(';').next().unwrap_or("").trim()
}

fn integer(token: &str) -> Result<i64> {
    let token = token.trim();
    if let Some(hex) = token.strip_prefix('$') {
        Ok(i64::from_str_radix(hex, 16)?)
    } else {
        Ok(token.parse()?)
    }
}

fn rows(source: &str, instruction: &str) -> Vec<Vec<String>> {
    let prefix = format!("{instruction} ");
    source
        .lines()
        .filter_map(|line| code(line).strip_prefix(&prefix))
        .map(|args| args.split(',').map(|arg| arg.trim().to_owned()).collect())
        .collect()
}

fn repel_steps(source: &str, source_effect: &str) -> Result<i64> {
    crystal_assets::parse_source_repel_duration(source, source_effect).map(i64::from)
}

fn status_table(source: &str) -> Result<BTreeMap<String, Vec<String>>> {
    let mut result = BTreeMap::new();
    for row in rows(source, "db") {
        ensure!(row.len() == 3, "invalid StatusHealingActions row");
        if row[0] == "-1" {
            break;
        }
        let statuses: &[&str] = match row[2].as_str() {
            "1 << PSN" => &["POISON"],
            "1 << BRN" => &["BURN"],
            "1 << FRZ" => &["FREEZE"],
            "SLP_MASK" => &["SLEEP"],
            "1 << PAR" => &["PARALYSIS"],
            "%11111111" => &["POISON", "BURN", "FREEZE", "SLEEP", "PARALYSIS"],
            other => anyhow::bail!("unsupported source status mask {other}"),
        };
        ensure!(
            result
                .insert(
                    row[0].clone(),
                    statuses.iter().map(|s| s.to_string()).collect()
                )
                .is_none(),
            "duplicate status item"
        );
    }
    Ok(result)
}

pub(super) fn generate(
    repository: &Path,
    names: &[(String, String)],
    descriptions: &[String],
) -> Result<()> {
    let vendor = repository.join("vendor/pokecrystal");
    let read = |relative: &str| -> Result<String> {
        let path = vendor.join(relative);
        std::fs::read_to_string(&path)
            .with_context(|| format!("read item source {}", path.display()))
    };
    let attributes = rows(&read("data/items/attributes.asm")?, "item_attribute");
    ensure!(
        attributes.len() == 256,
        "ItemAttributes must contain 256 source slots"
    );
    ensure!(
        names.len() == 250 && descriptions.len() == 255,
        "incomplete item identity/text catalogs"
    );
    let effects_source = read("engine/items/item_effects.asm")?;
    let table = effects_source
        .split_once("ItemEffects:")
        .context("missing ItemEffects label")?
        .1
        .split_once("assert_table_length ITEM_B3")
        .context("missing ItemEffects table boundary")?
        .0;
    let effects = rows(table, "dw");
    ensure!(
        effects.len() == 179,
        "unexpected ItemEffects source slot count"
    );
    let statuses = status_table(&read("data/items/heal_status.asm")?)?;
    let mut healing = BTreeMap::new();
    for row in rows(&read("data/items/heal_hp.asm")?, "dbw") {
        ensure!(row.len() == 2, "invalid HealingHPAmounts row");
        if row[0] == "-1" {
            break;
        }
        let amount = if row[1] == "MAX_STAT_VALUE" {
            -1
        } else {
            integer(&row[1])?
        };
        ensure!(
            healing.insert(row[0].clone(), amount).is_none(),
            "duplicate HP healing item"
        );
    }
    let mut aggregate = Vec::new();
    let mut tmhm_index = 0usize;
    for (index, (id, name)) in names.iter().enumerate() {
        let attr = &attributes[index];
        ensure!(attr.len() == 7, "invalid ItemAttributes slot {}", index + 1);
        let tmhm_move = id.strip_prefix("TM_").or_else(|| id.strip_prefix("HM_"));
        let source_effect = if tmhm_move.is_some() {
            "TMHMEffect"
        } else if let Some(effect) = effects.get(index) {
            effect[0].as_str()
        } else {
            ensure!(
                attr[5] == "ITEMMENU_NOUSE" && attr[6] == "ITEMMENU_NOUSE",
                "item {id} lacks a source effect"
            );
            "NoEffect"
        };
        let mut item = json!({
            "name": name, "description": descriptions[index], "script_name": id,
            "effect": match source_effect { "CardKeyEffect" => "CARD_KEY", "BasementKeyEffect" => "BASEMENT_KEY", "EvoStoneEffect" => "EVO_STONE", effect => effect }, "status_heals": statuses.get(id).cloned().unwrap_or_default(),
            "price": integer(&attr[0])?, "held_effect": attr[1], "parameter": integer(&attr[2])?,
            "property": attr[3], "pocket": attr[4], "field_menu": attr[5], "battle_menu": attr[6],
            "field_usable": attr[5] != "ITEMMENU_NOUSE", "battle_usable": attr[6] != "ITEMMENU_NOUSE",
            "consumable": attr[4] != "KEY_ITEM" && !id.starts_with("HM_") && source_effect != "NoEffect",
            "tmhm_index": null, "tmhm_move": null,
            "revive_hp_percent": null, "party_revive_hp_percent": null,
            "pp_restore_scope": null, "pp_restore_points": null, "pp_up_stages": null,
            "vitamin_stat": null, "vitamin_stat_exp": null, "vitamin_max_stat_exp": null,
            "rare_candy_level_gain": null, "battle_stat_boost_stat": null, "battle_stat_boost_stages": null,
            "battle_escape_mode": null, "battle_capture_ball": null, "battle_focus_energy": null,
            "battle_stat_drop_guard": null, "confusion_heal": null, "repel_steps": null, "escape_rope_mode": null
        });
        if let Some(amount) = healing.get(id) {
            item["parameter"] = json!(amount);
        }
        if let Some(move_id) = tmhm_move {
            tmhm_index += 1;
            item["tmhm_index"] = json!(tmhm_index);
            item["tmhm_move"] = json!(move_id);
        }
        match source_effect {
            "PokeBallEffect" => item["battle_capture_ball"] = json!(true),
            "EscapeRopeEffect" => item["escape_rope_mode"] = json!("DIG_WARP"),
            "PokeDollEffect" => item["battle_escape_mode"] = json!("WILD_BATTLE"),
            "DireHitEffect" => item["battle_focus_energy"] = json!(true),
            "GuardSpecEffect" => item["battle_stat_drop_guard"] = json!(true),
            "BitterBerryEffect" => item["confusion_heal"] = json!(true),
            "FullRestoreEffect" | "HealPowderEffect" => item["confusion_heal"] = json!(true),
            "RepelEffect" | "SuperRepelEffect" | "MaxRepelEffect" => {
                item["repel_steps"] = json!(repel_steps(&effects_source, source_effect)?);
            }
            "RareCandyEffect" => item["rare_candy_level_gain"] = json!(1),
            "SacredAshEffect" => item["party_revive_hp_percent"] = json!(100),
            "ReviveEffect" | "RevivalHerbEffect" => {
                item["revive_hp_percent"] = json!(if id == "REVIVE" { 50 } else { 100 })
            }
            "VitaminEffect" => {
                let stat = match id.as_str() {
                    "HP_UP" => "HP",
                    "PROTEIN" => "ATTACK",
                    "IRON" => "DEFENSE",
                    "CARBOS" => "SPEED",
                    "CALCIUM" => "SPECIAL",
                    _ => anyhow::bail!("unknown source vitamin {id}"),
                };
                item["vitamin_stat"] = json!(stat);
                item["vitamin_stat_exp"] = json!(10 << 8);
                item["vitamin_max_stat_exp"] = json!(100 << 8);
            }
            "XItemEffect" | "XAccuracyEffect" => {
                let stat = match id.as_str() {
                    "X_ATTACK" => "ATTACK",
                    "X_DEFEND" => "DEFENSE",
                    "X_SPEED" => "SPEED",
                    "X_SPECIAL" => "SPECIAL_ATTACK",
                    "X_ACCURACY" => "ACCURACY",
                    _ => anyhow::bail!("unknown source battle stat item {id}"),
                };
                item["battle_stat_boost_stat"] = json!(stat);
                item["battle_stat_boost_stages"] = json!(1);
            }
            "RestorePPEffect" => {
                if id == "PP_UP" {
                    item["pp_up_stages"] = json!(1);
                } else {
                    item["pp_restore_scope"] = json!(if id == "ELIXER" || id == "MAX_ELIXER" {
                        "POKEMON"
                    } else {
                        "MOVE"
                    });
                    item["pp_restore_points"] = match id.as_str() {
                        "ETHER" | "ELIXER" => json!(10),
                        "MYSTERYBERRY" => json!(5),
                        "MAX_ETHER" | "MAX_ELIXER" => Value::Null,
                        _ => anyhow::bail!("unknown source PP item {id}"),
                    };
                }
            }
            _ => {}
        }
        if !item["status_heals"].as_array().unwrap().is_empty() && statuses[id].len() == 5 {
            item["confusion_heal"] = json!(true);
        }
        let typed: crystal_core::models::Item = serde_json::from_value(item.clone())
            .with_context(|| format!("decode generated source item {id}"))?;
        let issues = crystal_core::systems::battle_items::item_payload_issues(&typed);
        ensure!(
            issues.is_empty(),
            "source item {id} has invalid runtime payload: {issues:?}"
        );
        aggregate.push((id.clone(), item));
    }
    ensure!(tmhm_index == 57, "expected 50 TMs and seven HMs");
    let data = repository.join("apps/web/assets/data");
    let modular = data.join("content-packs/core-modular/items");
    std::fs::create_dir_all(&modular)?;
    for (id, item) in &aggregate {
        std::fs::write(
            modular.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&json!({id: item}))?,
        )?;
    }
    std::fs::write(
        data.join("items.json"),
        serde_json::to_vec_pretty(&aggregate.iter().map(|(_, item)| item).collect::<Vec<_>>())?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_repel_durations_match_exact_labels_despite_shared_suffix() {
        let source = "SuperRepelEffect:\n\tld b, 200\n\tret\n\
                      MaxRepelEffect:\n\tld b, 250\n\tret\n\
                      RepelEffect:\n\tld b, $64 ; duration\n\tret\n";
        for (effect, expected) in [
            ("RepelEffect", 100),
            ("SuperRepelEffect", 200),
            ("MaxRepelEffect", 250),
        ] {
            assert_eq!(
                repel_steps(source, effect).expect("exact synthetic routine duration"),
                expected,
                "{effect} must not match another routine's suffix"
            );
        }
    }

    #[test]
    fn source_repel_duration_ignores_comments_and_rejects_missing_or_empty_routine() {
        let source = "; RepelEffect:\nSuperRepelEffect:\n\tld b, 200\n";
        assert!(repel_steps(source, "RepelEffect").is_err());
        let source = "RepelEffect: ; exact declaration\n; ld b, 200\n\n\tld b, 101\n";
        assert_eq!(
            repel_steps(source, "RepelEffect").expect("actual first instruction literal"),
            101,
            "read the literal rather than hardcoding an item duration"
        );
        assert!(repel_steps("RepelEffect:\n", "RepelEffect").is_err());
        assert!(repel_steps("RepelEffect:\nOtherEffect:\nld b, 200", "RepelEffect").is_err());
    }

    #[test]
    fn source_status_masks_preserve_actual_burnt_and_ice_berry_actions() {
        let table = status_table("db BURNT_BERRY, PARTYMENUTEXT_HEAL_FRZ, 1 << FRZ\ndb ICE_BERRY, PARTYMENUTEXT_HEAL_BRN, 1 << BRN\ndb -1, 0, 0").unwrap();
        assert_eq!(table["BURNT_BERRY"], ["FREEZE"]);
        assert_eq!(table["ICE_BERRY"], ["BURN"]);
        assert!(status_table("db ITEM, ACTION, UNRECOGNIZED").is_err());
    }
}
