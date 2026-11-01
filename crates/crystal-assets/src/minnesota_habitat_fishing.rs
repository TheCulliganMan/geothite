//! Private local rod habitats; original groups, timers and swarm rules stay intact.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

pub(super) fn install(
    pack: &mut CompiledGamePack,
    map_name: &str,
    species: &str,
    level: u8,
) -> Result<()> {
    let mut map = crate::northstar_rival_arc::map(pack, map_name)?;
    let original = map
        .attributes
        .fishing_group
        .as_ref()
        .context("native fishing group")?
        .clone();
    let group_name = format!("FISHGROUP_MN_HABITAT_{}", map_name.to_ascii_uppercase());
    let swarms = pack
        .data
        .fishing
        .swarm_rules
        .values()
        .filter(|s| s.base_group == original)
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        swarms.len() <= 1,
        "one native swarm binding per local group required"
    );
    let mut groups = vec![(original, group_name.clone())];
    if let Some(swarm) = swarms.first() {
        groups.push((swarm.swarm_group.clone(), format!("{group_name}_SWARM")));
    }
    for (before, after) in groups {
        let mut group = pack
            .data
            .fishing
            .groups
            .get(&before)
            .context("original native rod tables")?
            .clone();
        group.source_index = pack
            .data
            .fishing
            .groups
            .values()
            .map(|g| g.source_index)
            .max()
            .context("native fishing indices")?
            .checked_add(1)
            .context("fishing index overflow")?;
        for rod in ["GOOD_ROD", "SUPER_ROD"] {
            let table = group.rod_tables.get(rod).context("native upgraded rod")?;
            let first = table.slots.first().context("native first rod slot")?;
            let old_species = first
                .species
                .as_ref()
                .context("direct common rod slot required")?;
            ensure!(
                group
                    .rod_tables
                    .values()
                    .flat_map(|t| &t.slots)
                    .filter(|s| s.species.as_ref() == Some(old_species))
                    .count()
                    > 1,
                "cannot remove a unique old rod species"
            );
            let slot = group
                .rod_tables
                .get_mut(rod)
                .context("upgraded rod")?
                .slots
                .first_mut()
                .context("rod slot")?;
            slot.species = Some(species.into());
            slot.level = if rod == "SUPER_ROD" {
                level.max(slot.level)
            } else {
                level
            };
            ensure!(
                slot.time_group.is_none(),
                "time-group selection must remain intact"
            );
        }
        ensure!(
            pack.data.fishing.groups.insert(after, group).is_none(),
            "occupied local fishing group"
        );
    }
    if let Some(mut swarm) = swarms.into_iter().next() {
        swarm.base_group = group_name.clone();
        swarm.swarm_group = format!("{group_name}_SWARM");
        ensure!(
            pack.data
                .fishing
                .swarm_rules
                .insert(
                    format!("MN_HABITAT_SWARM_{}", map_name.to_ascii_uppercase()),
                    swarm
                )
                .is_none(),
            "occupied local fishing swarm"
        );
    }
    pack.data
        .map_attributes
        .get_mut(map_name)
        .context("native map-attribute mirror")?
        .fishing_group = Some(group_name.clone());
    map.attributes.fishing_group = Some(group_name);
    // Header-only changes must not reparse unrelated authored programs. In
    // particular, reparsing can reorder a parent's compiled command vectors.
    pack.data.maps.insert(map_name.into(), map);
    Ok(())
}
