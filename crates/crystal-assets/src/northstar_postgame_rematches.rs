//! Optional Champion rematches with persistent, retryable first-win gifts.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::northstar_battle_balance::{self, AuthoredMember};
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "northstar-champion-gym-rematches-v1";
const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
const GYMS: [&str; 8] = [
    "MnRochesterGym",
    "MnMankatoGym",
    "MnMinneapolisGym",
    "MnStPaulGym",
    "MnStCloudGym",
    "MnBrainerdGym",
    "MnBemidjiGym",
    "MnDuluthGym",
];
const TEXT_KINDS: [&str; 7] = ["Offer", "Won", "Lost", "After", "Gift", "Full", "Declined"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rematch {
    gym_index: u8,
    map: String,
    level_range: [u8; 2],
    party: Vec<AuthoredMember>,
}

/// Adds repeat battles without changing the badge battle or its pending TM claim.
pub(crate) fn install(pack: &mut CompiledGamePack, story_source: &str) -> Result<()> {
    ensure!(
        !pack.report.manifests.iter().any(|name| name == MANIFEST),
        "gym rematches already installed"
    );
    ensure!(
        pack.data
            .story_event_script_constants
            .global
            .contains_key(CHAMPION)
            && pack.data.items.contains_key("RARE_CANDY"),
        "installed Champion event and rematch prize required"
    );
    let teams: Value = serde_json::from_str(include_str!(
        "../../../planning/minnesota/story-pokemon.json"
    ))?;
    let rematches: Vec<Rematch> =
        serde_json::from_value(teams["postgame_gym_rematches"]["trainers"].clone())?;
    let story: Value = serde_json::from_str(story_source)?;
    let dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>> =
        serde_json::from_value(story["postgame_gym_rematches"]["dialogue"].clone())?;
    ensure!(
        rematches.len() == GYMS.len() && dialogue.len() == GYMS.len(),
        "complete eight-leader rematch circuit required"
    );
    let policy = RegionalRoster::current()?;
    for (index, rematch) in rematches.iter().enumerate() {
        ensure!(
            usize::from(rematch.gym_index) == index + 1 && rematch.map == GYMS[index],
            "rematch Gym identity differs"
        );
        let expected_range = match index {
            0..=3 => [55, 57],
            4..=6 => [56, 58],
            _ => [58, 60],
        };
        ensure!(
            rematch.level_range == expected_range
                && rematch.party.len() == 5
                && rematch.party.iter().all(|member| {
                    (expected_range[0]..=expected_range[1]).contains(&member.level)
                })
                && rematch
                    .party
                    .iter()
                    .map(|member| &member.species)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == 5,
            "five distinct partners in the advertised level range required"
        );
        let trainer_id = format!("MN_GYM_{}_REMATCH", rematch.gym_index);
        let trainer_class = register_trainer(pack, &policy, rematch, &trainer_id)?;
        let won = format!("EVENT_MN_GYM_{}_REMATCH_WON", rematch.gym_index);
        let gift = format!("EVENT_MN_GYM_{}_REMATCH_GIFT", rematch.gym_index);
        crate::authored_city_expansion_events::flags(pack, &[won.clone(), gift.clone()])?;
        let mut room = pack
            .data
            .maps
            .get(&rematch.map)
            .context("installed rematch Gym")?
            .clone();
        let texts = dialogue
            .get(&rematch.map)
            .context("canonical rematch dialogue")?;
        ensure!(
            texts.keys().map(String::as_str).collect::<BTreeSet<_>>()
                == TEXT_KINDS.into_iter().collect(),
            "complete rematch dialogue required"
        );
        let offer_lines: Vec<_> = texts["Offer"]
            .iter()
            .flat_map(|page| page.lines.iter().map(String::as_str))
            .collect();
        let level_notice = format!("LV {}-{}.", expected_range[0], expected_range[1]);
        ensure!(
            offer_lines.contains(&"Five partners.") && offer_lines.contains(&level_notice.as_str()),
            "rematch invitation must disclose the actual party size and levels"
        );
        for (kind, pages) in texts {
            let label = format!("{}Rematch{kind}Text", rematch.map);
            ensure!(
                !room.scripts.contains_key(&label),
                "occupied rematch text {label}"
            );
            room.scripts
                .insert(label, authored_dialogue::compile(pages)?);
        }
        install_scripts(&mut room, rematch, &trainer_id, &trainer_class, &won, &gift)?;
        crate::northstar_story_revision::publish(pack, room)?;
    }
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

fn register_trainer(
    pack: &mut CompiledGamePack,
    policy: &RegionalRoster,
    rematch: &Rematch,
    id: &str,
) -> Result<String> {
    ensure!(
        !pack.data.trainers.trainers.contains_key(id)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(id),
        "occupied rematch trainer {id}"
    );
    let original_id = format!("MN_GYM_{}", rematch.gym_index);
    let mut trainer = pack
        .data
        .trainers
        .get(&original_id)
        .context("original Gym leader")?
        .clone();
    let dvs = trainer.party.first().context("original leader DVs")?.dvs;
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|entry| entry.trainer_class == trainer.trainer_class)
        .filter_map(|entry| {
            pack.data
                .story_event_script_constants
                .global
                .get(&entry.trainer_id)
        })
        .copied()
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("rematch trainer constant overflow")?;
    trainer.trainer_id = id.into();
    trainer.items.clear();
    trainer.win_quote.clear();
    trainer.lose_quote.clear();
    trainer.party = rematch
        .party
        .iter()
        .map(|member| northstar_battle_balance::build_member(pack, policy, member, dvs))
        .collect::<Result<Vec<_>>>()?;
    let class = trainer.trainer_class.clone();
    pack.data.trainers.insert(trainer)?;
    pack.data
        .story_event_script_constants
        .global
        .insert(id.into(), next);
    Ok(class)
}

fn install_scripts(
    room: &mut MapModule,
    rematch: &Rematch,
    trainer_id: &str,
    trainer_class: &str,
    won: &str,
    gift: &str,
) -> Result<()> {
    let map = &rematch.map;
    let after = format!("{map}LeaderAfter");
    let old_after = format!("{map}RematchBeforeChampion");
    let offer = format!("{map}RematchOffer");
    let reward = format!("{map}RematchReward");
    let after_win = format!("{map}RematchAfter");
    let full = format!("{map}RematchFull");
    let declined = format!("{map}RematchDeclined");
    let original = json!([c("jumptext", &[&format!("{after}Text")])]);
    ensure!(
        room.scripts.get(&after) == Some(&original),
        "changed leader return program: {map}"
    );
    let leader = room
        .scripts
        .get(&format!("{map}Leader"))
        .and_then(Value::as_array)
        .context("original Gym leader program")?;
    ensure!(
        leader.starts_with(&[
            c("faceplayer", &[]),
            c(
                "checkevent",
                &[&format!("EVENT_MN_GYM_{}_DEFEATED", rematch.gym_index)]
            ),
            c("iftrue", &[&format!("{map}LeaderReward")])
        ]),
        "earned Gym victory must route through its original reward"
    );
    let first_reward = room
        .scripts
        .get(&format!("{map}LeaderReward"))
        .and_then(Value::as_array)
        .context("original Gym reward program")?;
    ensure!(
        first_reward.first()
            == Some(&c(
                "checkevent",
                &[&format!("EVENT_MN_GYM_{}_TM_RECEIVED", rematch.gym_index)]
            ))
            && first_reward.get(1) == Some(&c("iftrue", &[&after])),
        "original TM reward must precede rematches"
    );
    for label in [&old_after, &offer, &reward, &after_win, &full, &declined] {
        ensure!(
            !room.scripts.contains_key(label),
            "occupied rematch program {label}"
        );
    }
    room.scripts.insert(old_after.clone(), original);
    room.scripts.insert(
        after,
        json!([
            c("checkevent", &[CHAMPION]),
            c("iffalse", &[&old_after]),
            c("checkevent", &[won]),
            c("iffalse", &[&offer]),
            c("checkevent", &[gift]),
            c("iffalse", &[&reward]),
            c("sjump", &[&offer])
        ]),
    );
    room.scripts.insert(
        offer,
        json!([
            c("opentext", &[]),
            c("writetext", &[&format!("{map}RematchOfferText")]),
            c("yesorno", &[]),
            c("iffalse", &[&declined]),
            c("closetext", &[]),
            c(
                "winlosstext",
                &[
                    &format!("{map}RematchWonText"),
                    &format!("{map}RematchLostText")
                ]
            ),
            c("loadtrainer", &[trainer_class, trainer_id]),
            c("startbattle", &[]),
            c("reloadmapafterbattle", &[]),
            c("setevent", &[won]),
            c("sjump", &[&reward])
        ]),
    );
    room.scripts.insert(
        reward,
        json!([
            c("checkevent", &[gift]),
            c("iftrue", &[&after_win]),
            c("giveitem", &["RARE_CANDY", "1"]),
            c("iffalse", &[&full]),
            c("setevent", &[gift]),
            c("jumptext", &[&format!("{map}RematchGiftText")])
        ]),
    );
    room.scripts.insert(
        after_win,
        json!([c("jumptext", &[&format!("{map}RematchAfterText")])]),
    );
    room.scripts.insert(
        full,
        json!([c("jumptext", &[&format!("{map}RematchFullText")])]),
    );
    room.scripts.insert(
        declined,
        json!([
            c("writetext", &[&format!("{map}RematchDeclinedText")]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    Ok(())
}
