//! Fair stamps and small activities, lowered by the ordinary native parsers.
use anyhow::{Context, Result};
use serde_json::json;

use crate::MapModule;
use crate::grand_marais_modpack::authoring;

pub(crate) fn install(modules: &mut [(MapModule, [u16; 2])]) -> Result<()> {
    for (map, _) in modules.iter_mut() {
        match map.id.as_str() {
            "MnFairBarn" => stamp(
                map,
                "MnFairBarnVolunteer",
                "EVENT_MN_FAIR_BARN_STAMP",
                &[
                    "Good care wins more than ribbons.",
                    "Here is your livestock stamp.",
                ],
            ),
            "MnFairDairy" => stamp(
                map,
                "MnFairDairyDemonstrator",
                "EVENT_MN_FAIR_DAIRY_STAMP",
                &[
                    "BUTTERMUSE carves butter while the room stays cool.",
                    "Here is your dairy stamp.",
                ],
            ),
            _ => {}
        }
    }
    let map = &mut modules
        .iter_mut()
        .find(|(m, _)| m.id == "MnStateFairgrounds")
        .context("fair exterior activities")?
        .0;
    quiz(map);
    game(map);
    prize(map);
    care(map);
    Ok(())
}

fn stamp(map: &mut MapModule, script: &str, flag: &str, lines: &[&str]) {
    let c = authoring::command;
    let intro = format!("{script}StampText");
    let repeat = format!("{script}Repeat");
    let repeat_text = format!("{repeat}Text");
    authoring::text(map, &intro, lines);
    authoring::text(
        map,
        &repeat_text,
        &["Your card already has our stamp. Enjoy the fair!"],
    );
    map.scripts.insert(
        script.into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &[flag]),
            c("iftrue", &[&repeat]),
            c("opentext", &[]),
            c("writetext", &[&intro]),
            c("waitbutton", &[]),
            c("setevent", &[flag]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scripts
        .insert(repeat, json!([c("jumptextfaceplayer", &[&repeat_text])]));
}

fn quiz(map: &mut MapModule) {
    let c = authoring::command;
    for (suffix, lines) in [
        (
            "Question",
            vec!["Does BUTTERMUSE carve sculptures out of butter?"],
        ),
        ("Correct", vec!["Right! Here is your MIDWAY stamp."]),
        (
            "Hint",
            vec!["Visit the dairy exhibit for a hint. You can try again."],
        ),
        ("Repeat", vec!["You already earned our stamp. Nice work!"]),
    ] {
        authoring::text(map, &format!("MnFairQuiz{suffix}Text"), &lines);
    }
    map.scripts.insert(
        "MnFairMidwayHost".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_FAIR_MIDWAY_STAMP"]),
            c("iftrue", &["MnFairQuizRepeat"]),
            c("opentext", &[]),
            c("writetext", &["MnFairQuizQuestionText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnFairQuizHint"]),
            c("setevent", &["EVENT_MN_FAIR_MIDWAY_STAMP"]),
            c("writetext", &["MnFairQuizCorrectText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnFairQuizHint".into(),
        json!([
            c("writetext", &["MnFairQuizHintText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    map.scripts.insert(
        "MnFairQuizRepeat".into(),
        json!([c("jumptextfaceplayer", &["MnFairQuizRepeatText"])]),
    );
}

fn game(map: &mut MapModule) {
    let c = authoring::command;
    for (suffix, lines) in [
        (
            "Invite",
            vec!["Free practice toss! Land a ring and win a CORN STICK. Try?"],
        ),
        (
            "Miss",
            vec!["Just missed! Practice is free. Come try again."],
        ),
        (
            "Win",
            vec!["A perfect toss! One CORN STICK for your party."],
        ),
        (
            "Bag",
            vec!["Your bag is full. Make room and you can claim your prize."],
        ),
        (
            "Repeat",
            vec!["You won our prize already. Cheer for the next player!"],
        ),
    ] {
        authoring::text(map, &format!("MnFairGame{suffix}Text"), &lines);
    }
    map.scripts.insert(
        "MnFairGameHost".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_FAIR_GAME_PRIZE"]),
            c("iftrue", &["MnFairGameRepeat"]),
            c("opentext", &[]),
            c("checkevent", &["EVENT_MN_FAIR_GAME_PENDING"]),
            c("iftrue", &["MnFairGameAward"]),
            c("writetext", &["MnFairGameInviteText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnFairGameEnd"]),
            c("random", &["2"]),
            c("iftrue", &["MnFairGameMiss"]),
            c("setevent", &["EVENT_MN_FAIR_GAME_PENDING"]),
            c("sjump", &["MnFairGameAward"])
        ]),
    );
    map.scripts.insert(
        "MnFairGameAward".into(),
        json!([
            c("giveitem", &["FAIR_CORN_STICK", "1"]),
            c("iffalse", &["MnFairGameBag"]),
            c("setevent", &["EVENT_MN_FAIR_GAME_PRIZE"]),
            c("clearevent", &["EVENT_MN_FAIR_GAME_PENDING"]),
            c("writetext", &["MnFairGameWinText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for suffix in ["Miss", "Bag"] {
        map.scripts.insert(
            format!("MnFairGame{suffix}"),
            json!([
                c("writetext", &[&format!("MnFairGame{suffix}Text")]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    map.scripts.insert(
        "MnFairGameEnd".into(),
        json!([c("closetext", &[]), c("end", &[])]),
    );
    map.scripts.insert(
        "MnFairGameRepeat".into(),
        json!([c("jumptextfaceplayer", &["MnFairGameRepeatText"])]),
    );
}

fn prize(map: &mut MapModule) {
    let c = authoring::command;
    for (suffix, lines) in [
        (
            "Need",
            vec!["Collect stamps at the livestock barn, dairy exhibit and MIDWAY quiz."],
        ),
        (
            "Got",
            vec!["All three stamps! Here is one RARE CANDY. Thanks for visiting!"],
        ),
        (
            "Bag",
            vec!["Your bag is full. Your prize will be waiting right here."],
        ),
        (
            "Repeat",
            vec!["You claimed your stamp prize. Enjoy another lap of the fair!"],
        ),
    ] {
        authoring::text(map, &format!("MnFairPrize{suffix}Text"), &lines);
    }
    map.scripts.insert(
        "MnFairPrizeClerk".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_FAIR_PRIZE_CLAIMED"]),
            c("iftrue", &["MnFairPrizeRepeat"]),
            c("opentext", &[]),
            c("checkevent", &["EVENT_MN_FAIR_BARN_STAMP"]),
            c("iffalse", &["MnFairPrizeNeed"]),
            c("checkevent", &["EVENT_MN_FAIR_DAIRY_STAMP"]),
            c("iffalse", &["MnFairPrizeNeed"]),
            c("checkevent", &["EVENT_MN_FAIR_MIDWAY_STAMP"]),
            c("iffalse", &["MnFairPrizeNeed"]),
            c("giveitem", &["RARE_CANDY", "1"]),
            c("iffalse", &["MnFairPrizeBag"]),
            c("setevent", &["EVENT_MN_FAIR_PRIZE_CLAIMED"]),
            c("writetext", &["MnFairPrizeGotText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for suffix in ["Need", "Bag"] {
        map.scripts.insert(
            format!("MnFairPrize{suffix}"),
            json!([
                c("writetext", &[&format!("MnFairPrize{suffix}Text")]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    map.scripts.insert(
        "MnFairPrizeRepeat".into(),
        json!([c("jumptextfaceplayer", &["MnFairPrizeRepeatText"])]),
    );
}

fn care(map: &mut MapModule) {
    let c = authoring::command;
    for (suffix, text) in [
        (
            "Question",
            "Before a show, should you check that your partner is healthy?",
        ),
        (
            "Correct",
            "Good care comes first! Here is a BERRY from our4-H project.",
        ),
        (
            "Hint",
            "A healthy, rested partner can enjoy the fair too. Try again!",
        ),
        (
            "Bag",
            "Make room in your bag and try the care question again.",
        ),
        (
            "Repeat",
            "You already earned our care prize. Keep looking after your partner!",
        ),
    ] {
        authoring::text(map, &format!("MnFairCare{suffix}Text"), &[text]);
    }
    map.scripts.insert(
        "MnFairCareHost".into(),
        json!([
            c("faceplayer", &[]),
            c("checkevent", &["EVENT_MN_FAIR_CARE_PRIZE"]),
            c("iftrue", &["MnFairCareRepeat"]),
            c("opentext", &[]),
            c("writetext", &["MnFairCareQuestionText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnFairCareHint"]),
            c("giveitem", &["BERRY", "1"]),
            c("iffalse", &["MnFairCareBag"]),
            c("setevent", &["EVENT_MN_FAIR_CARE_PRIZE"]),
            c("writetext", &["MnFairCareCorrectText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[])
        ]),
    );
    for suffix in ["Hint", "Bag"] {
        map.scripts.insert(
            format!("MnFairCare{suffix}"),
            json!([
                c("writetext", &[&format!("MnFairCare{suffix}Text")]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
    }
    map.scripts.insert(
        "MnFairCareRepeat".into(),
        json!([c("jumptextfaceplayer", &["MnFairCareRepeatText"])]),
    );
}
