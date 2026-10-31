//! Earned research, bicycle and restoration rewards on existing native actors.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_rival_arc::{map, publish, put};

const MANIFEST: &str = "northstar-progression-rewards-v1";
pub(crate) const DEX_MEMBER_PREFIX: &str = "MN_DEX_MEMBER_";
pub(crate) const DEX_CAUGHT_VARIABLE: &str = "VAR_MINNESOTA_DEX_CAUGHT";

/// Adds once-only native rewards without changing terrain, teams or encounters.
/// The field-station aide counts caught active Minnesota species, not seen
/// entries or foreign species. Failed bag delivery never consumes a reward.
/// Tamarack awards a Master Ball after restoration; the Minneapolis workshop
/// loans a free bicycle after the first badge. Original conversations survive.
///
/// # Errors
/// Rejects duplicate installation, missing actors/items, malformed roster data,
/// retired/missing species, occupied script identities or invalid native output.
pub fn build_northstar_progression_rewards(
    base: &CompiledGamePack,
    roster: &str,
    curation: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "rewards already installed"
    );
    let roster: Value = serde_json::from_str(roster)?;
    let curation: Value = serde_json::from_str(curation)?;
    let retired: BTreeSet<_> = curation["retired_families"]
        .as_array()
        .context("curation families")?
        .iter()
        .filter_map(|f| f["id"].as_str())
        .collect();
    let mut species = BTreeSet::new();
    for family in roster["families"].as_array().context("roster families")? {
        let id = family["id"].as_str().context("family identity")?;
        if retired.contains(id) {
            continue;
        }
        for member in family["members"].as_array().context("family members")? {
            let name = member["name"]
                .as_str()
                .context("species name")?
                .to_ascii_uppercase();
            ensure!(
                base.data.pokemon.contains_key(&name),
                "active species absent: {name}"
            );
            ensure!(species.insert(name), "duplicate active species");
        }
    }
    ensure!(
        species.len() >= 100,
        "complete active regional roster required"
    );
    let mut pack = base.clone();
    for name in &species {
        let key = format!("{DEX_MEMBER_PREFIX}{name}");
        ensure!(
            pack.data
                .story_event_script_constants
                .global
                .insert(key, 1)
                .is_none(),
            "occupied Dex member"
        );
    }
    for item in [
        "MASTER_BALL",
        "BICYCLE",
        "ITEMFINDER",
        "GREAT_BALL",
        "ULTRA_BALL",
        "LUCKY_EGG",
        "RARE_CANDY",
    ] {
        ensure!(
            pack.data.items.contains_key(item),
            "native reward missing: {item}"
        );
    }
    let mut station = map(&pack, "MnWinonaFieldStation")?;
    wrap(&mut station, "MN_W_TAMARACK", "MnRewardTamarack")?;
    put(
        &mut station,
        "MnRewardTamarack",
        vec![
            c("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
            c("iffalse", &["MnRewardTamarackOriginal"]),
            c("checkevent", &["EVENT_MN_MASTER_BALL_RECEIVED"]),
            c("iftrue", &["MnRewardTamarackOriginal"]),
            c("opentext", &[]),
            c("writetext", &["MnRewardMasterIntroText"]),
            c("waitbutton", &[]),
            // Commit the saved claim before the next visible boundary. A
            // verbose grant yields while its item has already entered the bag.
            c("giveitem", &["MASTER_BALL", "1"]),
            c("iffalse", &["MnRewardStationFull"]),
            c("setevent", &["EVENT_MN_MASTER_BALL_RECEIVED"]),
            c("writetext", &["MnRewardMasterGivenText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    authoring::text(
        &mut station,
        "MnRewardMasterIntroText",
        &[
            "TAMARACK: The water is moving again. You brought people home.",
            "The lab kept one MASTER BALL for an emergency. I trust your judgment.",
        ],
    );
    authoring::text(
        &mut station,
        "MnRewardMasterGivenText",
        &[
            "You received a MASTER BALL!",
            "It catches without fail. You decide when to use it. The wetlands still hold mysteries.",
        ],
    );
    wrap(&mut station, "MN_W_AIDE", "MnRewardDex")?;
    // Keep existing reward flag identities so saved claims remain valid.
    // The 50-species supplies replace EXP Share while retaining its claim flag.
    crate::northstar_lair_authoring::flag(&mut pack, "EVENT_MN_DEX_REWARD_2")?;
    let tiers = [
        (0, 10, "GREAT_BALL", 10),
        (1, 25, "ITEMFINDER", 1),
        (2, 50, "ULTRA_BALL", 10),
        (3, 100, "LUCKY_EGG", 1),
        (4, species.len(), "RARE_CANDY", 10),
    ];
    let mut root = vec![c("faceplayer", &[])];
    for (i, (flag_index, count, item, quantity)) in tiers.iter().enumerate() {
        let flag = format!("EVENT_MN_DEX_REWARD_{flag_index}");
        if *flag_index != 2 {
            crate::northstar_lair_authoring::flag(&mut pack, &flag)?;
        }
        let next = format!("MnRewardDexTier{}", i + 1);
        let claim = format!("MnRewardDexClaim{i}");
        let text = format!("MnRewardDexClaim{i}Text");
        let mut check = vec![
            c("checkevent", &[&flag]),
            c("iftrue", &[&next]),
            c("readvar", &[DEX_CAUGHT_VARIABLE]),
            c("ifless", &[&count.to_string(), "MnRewardDexNeed"]),
            c("sjump", &[&claim]),
        ];
        if i == 0 {
            root.append(&mut check);
        } else {
            put(&mut station, &format!("MnRewardDexTier{i}"), check);
        }
        put(
            &mut station,
            &claim,
            vec![
                c("opentext", &[]),
                c("writetext", &[&text]),
                c("waitbutton", &[]),
                c("verbosegiveitem", &[item, &quantity.to_string()]),
                c("iffalse", &["MnRewardStationFull"]),
                c("setevent", &[&flag]),
                c("closetext", &[]),
                c("end", &[]),
            ],
        );
        authoring::text(
            &mut station,
            &text,
            &[
                &format!(
                    "You've caught {count} Minnesota species! Your field notes earned this research award."
                ),
                match *item {
                    "GREAT_BALL" => {
                        "GREAT BALLs help catch wild Pokemon. Keep a few ready on the trail."
                    }
                    "ULTRA_BALL" => "ULTRA BALLs help catch elusive Pokemon. Keep exploring!",
                    "ITEMFINDER" => {
                        "Use ITEMFINDER outdoors to search for hidden items. Register it for SELECT, too."
                    }
                    "LUCKY_EGG" => "A partner holding LUCKY EGG earns extra experience in battle.",
                    _ => "Use RARE CANDY on a partner to raise its level.",
                },
                "Each award is given once. Keep exploring!",
            ],
        );
    }
    put(&mut station, "MnRewardDex", root);
    put(
        &mut station,
        &format!("MnRewardDexTier{}", tiers.len()),
        vec![c("jumptextfaceplayer", &["MnRewardDexCompleteText"])],
    );
    put(
        &mut station,
        "MnRewardDexNeed",
        vec![
            c("opentext", &[]),
            c("writetext", &["MnRewardDexNeedText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("sjump", &["MnRewardDexOriginal"]),
        ],
    );
    authoring::text(
        &mut station,
        "MnRewardDexNeedText",
        &[
            "I award research supplies for 10, 25, 50, 100 and all Minnesota species caught.",
            "Seen entries don't count. Your partners may stay in the PC. Come back as your records grow!",
            "All healthy party Pokemon already share battle experience. No item is needed.",
        ],
    );
    authoring::text(
        &mut station,
        "MnRewardDexCompleteText",
        &[
            "Every Minnesota species recorded! These notes will guide our field crews for years. Thank you.",
        ],
    );
    put(
        &mut station,
        "MnRewardStationFull",
        vec![
            c("writetext", &["MnRewardStationFullText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    authoring::text(
        &mut station,
        "MnRewardStationFullText",
        &["Make room in that PACK pocket, then ask again. I'll keep your award here."],
    );
    crate::northstar_lair_authoring::flag(&mut pack, "EVENT_MN_MASTER_BALL_RECEIVED")?;
    publish(&mut pack, station)?;
    let mut workshop = map(&pack, "MnMinneapolisToolWorkshop")?;
    wrap(&mut workshop, "MN_MP_TOOLWORKSHOP_HOST", "MnRewardBike")?;
    crate::northstar_lair_authoring::flag(&mut pack, "EVENT_MN_BICYCLE_RECEIVED")?;
    put(
        &mut workshop,
        "MnRewardBike",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_BICYCLE_RECEIVED"]),
            c("iftrue", &["MnRewardBikeAfter"]),
            c("checkflag", &["ENGINE_ZEPHYRBADGE"]),
            c("iffalse", &["MnRewardBikeDenied"]),
            c("opentext", &[]),
            c("writetext", &["MnRewardBikeIntroText"]),
            c("waitbutton", &[]),
            c("checkitem", &["BICYCLE"]),
            c("iftrue", &["MnRewardBikeOwned"]),
            c("verbosegiveitem", &["BICYCLE", "1"]),
            c("iffalse", &["MnRewardBikeFull"]),
            c("sjump", &["MnRewardBikeOwned"]),
        ],
    );
    put(
        &mut workshop,
        "MnRewardBikeOwned",
        vec![
            c("setevent", &["EVENT_MN_BICYCLE_RECEIVED"]),
            c("writetext", &["MnRewardBikeUseText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    for (label, text) in [
        ("MnRewardBikeAfter", "MnRewardBikeUseText"),
        ("MnRewardBikeDenied", "MnRewardBikeDeniedText"),
        ("MnRewardBikeFull", "MnRewardBikeFullText"),
    ] {
        put(&mut workshop, label, vec![c("jumptextfaceplayer", &[text])]);
    }
    put(
        &mut workshop,
        "MnRewardBikeAfter",
        vec![
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("writetext", &["MnRewardBikeUseText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("sjump", &["MnRewardBikeOriginal"]),
        ],
    );
    authoring::text(
        &mut workshop,
        "MnRewardBikeIntroText",
        &[
            "I repair bicycles here, too. You earned a badge? Then you've got places to go!",
            "Take this rebuilt BICYCLE. No charge. Bring it back for repairs anytime.",
        ],
    );
    authoring::text(
        &mut workshop,
        "MnRewardBikeUseText",
        &[
            "Choose BICYCLE in PACK's KEY ITEMS. Register it to ride with SELECT outdoors.",
            "Some narrow paths and indoor floors are for walking. Ride safely!",
        ],
    );
    authoring::text(
        &mut workshop,
        "MnRewardBikeDeniedText",
        &[
            "This workshop also repairs bicycles. Earn your first BADGE and I'll have one ready for you.",
        ],
    );
    authoring::text(
        &mut workshop,
        "MnRewardBikeFullText",
        &["Your KEY ITEMS pocket is full. Make room and I'll keep the bike ready."],
    );
    publish(&mut pack, workshop)?;
    let mut city = map(&pack, "MnMinneapolis")?;
    ensure!(
        city.scripts.contains_key("MN_MP_TOOLWORKSHOP_SIGN"),
        "existing workshop sign"
    );
    authoring::text(
        &mut city,
        "MN_MP_TOOLWORKSHOP_SIGNText",
        &[
            "BICYCLE & TOOL WORKSHOP",
            "Rebuilt bicycles for badge holders. Keep our river trails rolling!",
        ],
    );
    publish(&mut pack, city)?;
    repair_machine_part_handoff(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn wrap(room: &mut crate::MapModule, actor: &str, label: &str) -> Result<()> {
    ensure!(!room.scripts.contains_key(label), "occupied reward script");
    let original = room
        .scripts
        .get(actor)
        .context("reward actor program")?
        .clone();
    ensure!(
        room.objects.iter().any(|o| o.script == actor),
        "reward actor missing"
    );
    put(
        room,
        &format!("{label}Original"),
        original.as_array().context("native program")?.clone(),
    );
    put(room, actor, vec![c("sjump", &[label])]);
    Ok(())
}

/// Reads caught records against the authored active-region membership constants.
/// This is recomputed on every query; save variables cannot spoof completion.
pub(crate) fn caught_count(
    data: &crate::GameDataSet,
    state: &crystal_core::state::GameState,
) -> usize {
    state
        .pokedex
        .caught_species
        .iter()
        .filter(|id| {
            data.story_event_script_constants
                .global
                .contains_key(&format!("{DEX_MEMBER_PREFIX}{id}"))
        })
        .count()
}

fn repair_machine_part_handoff(pack: &mut CompiledGamePack) -> Result<()> {
    let mut station = map(pack, "MnGrandMaraisFieldStation")?;
    wrap(&mut station, "MnGrandMaraisWelcome", "MnRewardMachinePart")?;
    put(
        &mut station,
        "MnRewardMachinePart",
        vec![
            c("checkevent", &["EVENT_MN_GRAND_MARAIS_STARTER"]),
            c("iffalse", &["MnRewardMachinePartOriginal"]),
            c("checkevent", &["EVENT_MN_GRAND_MARAIS_SIGNAL"]),
            c("iftrue", &["MnRewardMachinePartOriginal"]),
            c("checkitem", &["MACHINE_PART"]),
            c("iftrue", &["MnRewardMachinePartOriginal"]),
            c("opentext", &[]),
            c("verbosegiveitem", &["MACHINE_PART", "1"]),
            c("iffalse", &["MnRewardMachinePartFull"]),
            c("writetext", &["MnRewardMachinePartGivenText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    put(
        &mut station,
        "MnRewardMachinePartFull",
        vec![
            c("writetext", &["MnRewardMachinePartFullText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ],
    );
    authoring::text(
        &mut station,
        "MnRewardMachinePartGivenText",
        &[
            "The lighthouse keeper needs this MACHINE PART. Bring it to the lighthouse here in GRAND MARAIS.",
        ],
    );
    authoring::text(
        &mut station,
        "MnRewardMachinePartFullText",
        &["Make room in KEY ITEMS and return. The keeper still needs this MACHINE PART."],
    );
    publish(pack, station)
}
