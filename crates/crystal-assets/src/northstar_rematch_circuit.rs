//! Persistent eight-leader circuit progress, preserving adventure reward receipts.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, MapModule};

const CLAIMED: &str = "EVENT_MN_REMATCH_CIRCUIT_PRIZE";
const GYMS: [(&str, &str); 8] = [
    ("MnRochesterGym", "BLACKGLASSES"),
    ("MnMankatoGym", "SOFT_SAND"),
    ("MnMinneapolisGym", "MAGNET"),
    ("MnStPaulGym", "METAL_COAT"),
    ("MnStCloudGym", "SHARP_BEAK"),
    ("MnBrainerdGym", "MYSTIC_WATER"),
    ("MnBemidjiGym", "MIRACLE_SEED"),
    ("MnDuluthGym", "NEVERMELTICE"),
];

/// Retains all existing win and gift flags, including already claimed candies.
pub(crate) fn install(pack: &mut CompiledGamePack, story_source: &str) -> Result<()> {
    let source: Value = serde_json::from_str(story_source)?;
    let circuit: BTreeMap<String, Vec<AuthoredDialoguePage>> =
        serde_json::from_value(source["refinement_services"]["circuit"].clone())?;
    let dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>> =
        serde_json::from_value(source["refinement_services"]["dialogue"].clone())?;
    ensure!(
        pack.data.items.contains_key("LEFTOVERS"),
        "circuit prize exists"
    );
    crate::authored_city_expansion_events::flags(pack, &[CLAIMED.into()])?;
    for (index, (map, item)) in GYMS.iter().enumerate() {
        let number = index + 1;
        let won = format!("EVENT_MN_GYM_{number}_REMATCH_WON");
        let gift = format!("EVENT_MN_GYM_{number}_REMATCH_GIFT");
        ensure!(
            [&won, &gift].iter().all(|flag| pack
                .data
                .story_event_script_constants
                .global
                .contains_key(*flag))
                && pack.data.items.contains_key(*item),
            "installed rematch receipts and themed prize"
        );
        let mut room = pack
            .data
            .maps
            .get(*map)
            .context("installed rematch room")?
            .clone();
        let after = format!("{map}RematchAfter");
        let full = format!("{map}RematchFull");
        let reward = format!("{map}RematchReward");
        ensure!(
            room.scripts.get(&reward)
                == Some(&json!([
                    c("checkevent", &[&gift]),
                    c("iftrue", &[&after]),
                    c("giveitem", &["RARE_CANDY", "1"]),
                    c("iffalse", &[&full]),
                    c("setevent", &[&gift]),
                    c("jumptext", &[&format!("{map}RematchGiftText")])
                ])),
            "installed first-win reward preimage"
        );
        // Reuse the old receipt: players who already took a candy keep it and
        // cannot collect a second first-win prize after upgrading.
        room.scripts.insert(
            reward,
            json!([
                c("checkevent", &[&gift]),
                c("iftrue", &[&after]),
                c("giveitem", &[item, "1"]),
                c("iffalse", &[&full]),
                c("setevent", &[&gift]),
                c("jumptext", &[&format!("{map}RematchGiftText")])
            ]),
        );
        for kind in ["Gift", "Full"] {
            let label = format!("{map}Rematch{kind}Text");
            let pages = dialogue
                .get(*map)
                .and_then(|texts| texts.get(&label))
                .context("themed first-win gift and pending-claim prose")?;
            room.scripts
                .insert(label, authored_dialogue::compile(pages)?);
        }
        install_guide(&mut room, &circuit)?;
        crate::northstar_story_revision::publish(pack, room)?;
    }
    Ok(())
}

fn install_guide(
    room: &mut MapModule,
    prose: &BTreeMap<String, Vec<AuthoredDialoguePage>>,
) -> Result<()> {
    let map = room.id.clone();
    let label = |suffix: &str| format!("{map}Circuit{suffix}");
    // Bemidji's outer guide script owns the existing retryable CUT gift.
    // Hook its repeat conversation so that the HM claim keeps priority.
    let guide = if map == "MnBemidjiGym" {
        format!("{map}GuideRepeat")
    } else {
        format!("{map}Guide")
    };
    let original = room
        .scripts
        .get(&guide)
        .context("existing gym guide")?
        .clone();
    ensure!(
        original == json!([c("jumptextfaceplayer", &[&format!("{map}GuideText")])]),
        "gym guide preimage"
    );
    let legacy = format!("{map}CircuitBeforeChampion");
    ensure!(
        !room.scripts.contains_key(&legacy),
        "circuit already installed"
    );
    room.scripts.insert(legacy.clone(), original);
    for key in ["Intro", "Complete", "Full", "Prize"]
        .into_iter()
        .map(str::to_owned)
        .chain((1..=8).map(|number| format!("Missing{number}")))
    {
        room.scripts.insert(
            label(&format!("{key}Text")),
            authored_dialogue::compile(prose.get(&key).context("complete circuit prose")?)?,
        );
    }
    room.scripts.insert(
        guide,
        json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_LEAGUE_CHAMPION"]),
            c("iffalse", &[&legacy]),
            c("checkevent", &[CLAIMED]),
            c("iftrue", &[&label("Complete")]),
            c("sjump", &[&label("Eligibility")])
        ]),
    );
    room.scripts.insert(
        label("Progress"),
        json!([
            c("opentext", &[]),
            c("writetext", &[&label("IntroText")]),
            c("waitbutton", &[]),
            c("sjump", &[&label("Check1")])
        ]),
    );
    for number in 1..=8 {
        let next = if number == 8 {
            label("End")
        } else {
            label(&format!("Check{}", number + 1))
        };
        room.scripts.insert(
            label(&format!("Check{number}")),
            json!([
                c(
                    "checkevent",
                    &[&format!("EVENT_MN_GYM_{number}_REMATCH_WON")]
                ),
                c("iftrue", &[&next]),
                c("writetext", &[&label(&format!("Missing{number}Text"))]),
                c("waitbutton", &[]),
                c("sjump", &[&next])
            ]),
        );
    }
    room.scripts
        .insert(label("Eligibility"), eligibility_program(&map));
    room.scripts
        .insert(label("End"), json!([c("closetext", &[]), c("end", &[])]));
    room.scripts.insert(label("Claim"), claim_program(&map));
    room.scripts.insert(
        label("Complete"),
        json!([c("jumptext", &[&label("CompleteText")])]),
    );
    room.scripts
        .insert(label("Full"), json!([c("jumptext", &[&label("FullText")])]));
    Ok(())
}

fn eligibility_program(map: &str) -> Value {
    let mut commands = Vec::new();
    for number in 1..=8 {
        commands.push(c(
            "checkevent",
            &[&format!("EVENT_MN_GYM_{number}_REMATCH_WON")],
        ));
        commands.push(c("iffalse", &[&format!("{map}CircuitProgress")]));
    }
    commands.push(c("sjump", &[&format!("{map}CircuitClaim")]));
    Value::Array(commands)
}

fn claim_program(map: &str) -> Value {
    json!([
        c("checkevent", &[CLAIMED]),
        c("iftrue", &[&format!("{map}CircuitComplete")]),
        c("giveitem", &["LEFTOVERS", "1"]),
        c("iffalse", &[&format!("{map}CircuitFull")]),
        c("setevent", &[CLAIMED]),
        c("jumptext", &[&format!("{map}CircuitPrizeText")])
    ])
}

#[cfg(test)]
mod tests {
    use super::{CLAIMED, claim_program, eligibility_program};

    #[test]
    fn every_leader_is_required_and_prize_receipt_follows_bag_success() {
        let eligibility = eligibility_program("MnDuluthGym");
        let checks = eligibility.as_array().expect("eligibility");
        for number in 1..=8 {
            assert!(checks.iter().any(|entry| entry["command"] == "checkevent"
                && entry["args"][0] == format!("EVENT_MN_GYM_{number}_REMATCH_WON")));
        }
        assert_eq!(
            checks
                .iter()
                .filter(|entry| entry["command"] == "iffalse")
                .count(),
            8
        );
        let claim = claim_program("MnDuluthGym");
        let commands = claim.as_array().expect("claim");
        let give = commands
            .iter()
            .position(|entry| entry["command"] == "giveitem")
            .expect("prize");
        assert_eq!(commands[0]["args"][0], CLAIMED);
        assert_eq!(commands[give + 1]["command"], "iffalse");
        assert_eq!(commands[give + 1]["args"][0], "MnDuluthGymCircuitFull");
        assert_eq!(commands[give + 2]["command"], "setevent");
        assert_eq!(commands[give + 2]["args"][0], CLAIMED);
    }
}
