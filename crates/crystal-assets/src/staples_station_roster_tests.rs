//! All touched encounter species are active; original balance/source tables survive.
use std::collections::BTreeSet;

use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_regional_approach;
use crate::staples_station::ROAD;
use crate::staples_station_roster::FISHING;

pub(super) fn verify(base: &CompiledGamePack, child: &CompiledGamePack) {
    let (before, after) = (base.data(), child.data());
    let policy = authored_regional_approach::policy().expect("active regional policy");
    assert_eq!(
        before.trainers.trainers.len(),
        after.trainers.trainers.len()
    );
    for (id, old) in &before.trainers.trainers {
        let new = &after.trainers.trainers[id];
        if id != "MN_R15_T03" {
            assert_eq!(
                new, old,
                "unrelated trainer, including original source regions: {id}"
            );
            continue;
        }
        assert_eq!(
            new.party
                .iter()
                .map(|m| m.species.as_str())
                .collect::<Vec<_>>(),
            ["BRISTUB", "CLINKLAW"]
        );
        let mut restored = new.clone();
        for (member, old) in restored.party.iter_mut().zip(&old.party) {
            assert!(policy.allowed.contains(&member.species));
            assert_eq!(
                policy
                    .target(child, &member.species, Some(member.level))
                    .expect("legal level"),
                member.species
            );
            assert_eq!(
                member.moves,
                after
                    .create_pokemon(&member.species, member.level, member.dvs)
                    .expect("native moves")
                    .moves
            );
            member.species.clone_from(&old.species);
            member.moves.clone_from(&old.moves);
        }
        assert_eq!(
            &restored, old,
            "trainer levels, DVs, held items, rewards and all other data stay exact"
        );
    }
    for (name, old) in &before.wild_encounters {
        let new = &after.wild_encounters[name];
        if name == ROAD {
            let old = serde_json::to_value(old).expect("original encounter table");
            let mut new = serde_json::to_value(new).expect("regional encounter table");
            restore_species(&old, &mut new, &policy.allowed);
            assert_eq!(
                new, old,
                "only species change; levels, rates, times, slots and zones remain exact"
            );
        } else {
            assert_eq!(new, old, "unrelated wild table {name}");
        }
    }
    assert_eq!(before.wild_encounters.len(), after.wild_encounters.len());
    assert_eq!(
        before.field_encounters, after.field_encounters,
        "no field table exists on the original R15"
    );
    let (old, new) = (&before.fishing, &after.fishing);
    assert_eq!(old.rod_items, new.rod_items);
    for (id, group) in &old.groups {
        assert_eq!(
            new.groups.get(id),
            Some(group),
            "shared original rod table {id}"
        );
    }
    for (id, time) in &old.time_groups {
        assert_eq!(
            new.time_groups.get(id),
            Some(time),
            "shared original time fish {id}"
        );
    }
    for (id, rule) in &old.swarm_rules {
        assert_eq!(
            new.swarm_rules.get(id),
            Some(rule),
            "original swarm rule {id}"
        );
    }
    assert_eq!(new.groups.len(), old.groups.len() + 2);
    let source_group = before.maps[ROAD]
        .attributes
        .fishing_group
        .as_deref()
        .expect("original approach fish");
    let source_swarm = old
        .swarm_rules
        .values()
        .find(|rule| rule.base_group == source_group)
        .expect("original approach swarm")
        .swarm_group
        .as_str();
    let mut source_times = BTreeSet::new();
    for name in [source_group, source_swarm] {
        for table in old.groups[name].rod_tables.values() {
            for slot in &table.slots {
                if let Some(time) = &slot.time_group {
                    source_times.insert(time);
                }
            }
        }
    }
    assert_eq!(
        new.time_groups.len(),
        old.time_groups.len() + source_times.len()
    );
    assert_eq!(new.swarm_rules.len(), old.swarm_rules.len() + 1);
    for (source, target) in [
        (source_group, FISHING),
        (source_swarm, "FISHGROUP_MN_STAPLES_R15_SWARM"),
    ] {
        let mut restored = new.groups[target].clone();
        assert!(
            !old.groups
                .values()
                .any(|g| g.source_index == restored.source_index)
        );
        restored.source_index = old.groups[source].source_index;
        for (rod, table) in &mut restored.rod_tables {
            for (slot, old_slot) in table
                .slots
                .iter_mut()
                .zip(&old.groups[source].rod_tables[rod].slots)
            {
                if let Some(species) = &slot.species {
                    assert!(policy.allowed.contains(species));
                    assert_eq!(
                        policy
                            .target(child, species, Some(slot.level))
                            .expect("legal fish level"),
                        *species
                    );
                }
                slot.species.clone_from(&old_slot.species);
                if let Some(time) = &old_slot.time_group {
                    assert_eq!(
                        slot.time_group.as_deref(),
                        Some(format!("MN_STAPLES_R15_TIME_{time}").as_str())
                    );
                    let old_time =
                        serde_json::to_value(&old.time_groups[time]).expect("old timed fish");
                    let mut new_time = serde_json::to_value(
                        &new.time_groups[slot.time_group.as_ref().expect("local time")],
                    )
                    .expect("local time fish");
                    restore_species(&old_time, &mut new_time, &policy.allowed);
                    assert_eq!(new_time, old_time, "timed fish preserve native levels");
                }
                slot.time_group.clone_from(&old_slot.time_group);
            }
        }
        assert_eq!(
            restored, old.groups[source],
            "rod thresholds, levels and bite rates stay exact"
        );
    }
    let mut swarm = new.swarm_rules["FISHSWARM_MN_STAPLES_R15"].clone();
    assert_eq!(swarm.base_group, FISHING);
    assert_eq!(swarm.swarm_group, "FISHGROUP_MN_STAPLES_R15_SWARM");
    swarm
        .base_group
        .clone_from(&old.swarm_rules["FISHSWARM_QWILFISH"].base_group);
    swarm
        .swarm_group
        .clone_from(&old.swarm_rules["FISHSWARM_QWILFISH"].swarm_group);
    assert_eq!(
        swarm, old.swarm_rules["FISHSWARM_QWILFISH"],
        "native daily swarm behavior survives"
    );
}

fn restore_species(old: &Value, new: &mut Value, allowed: &BTreeSet<String>) {
    match (old, new) {
        (Value::Object(old), Value::Object(new)) => {
            for (key, value) in old {
                let actual = new.get_mut(key).expect("same encounter schema");
                if ["species", "day_species", "night_species"].contains(&key.as_str())
                    && value.is_string()
                {
                    assert!(
                        allowed.contains(actual.as_str().expect("actual species")),
                        "non-Minnesota encounter {actual}"
                    );
                    *actual = value.clone();
                } else {
                    restore_species(value, actual, allowed);
                }
            }
        }
        (Value::Array(old), Value::Array(new)) => {
            assert_eq!(old.len(), new.len());
            for (old, new) in old.iter().zip(new) {
                restore_species(old, new, allowed);
            }
        }
        _ => {}
    }
}
