//! Give the new Champion a lead on Reed and honor the border watch's ledger.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::install_canonical;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command as c;
use crate::international_falls_story::{ARRIVED, GIVEN, REPORT};
use crate::{CompiledGamePack, MapModule};

const CHAMPION: &str = "EVENT_MN_LEAGUE_CHAMPION";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const REED_WON: &str = "EVENT_MN_REED_CHAMPION_DEFEATED";

/// Updates the Ely and Boundary rangers and the guide on the shore pass.
///
/// Returns an error if an original talker or its expected script has changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    shore_guide(pack)?;
    let mut ely = pack.data.maps.get("MnEly").context("Ely ranger")?.clone();
    let ranger = "MnElyRanger";
    let snow = c("jumptextfaceplayer", &["MnElyRangerText"]);
    ensure!(
        ely.objects.iter().any(|o| o.script == ranger)
            && ely.scripts.get(ranger) == Some(&json!([snow])),
        "expected the public Ely ranger and winter greeting"
    );
    add(&mut ely, "MnAfterChampionSnow", json!([snow]))?;
    ely.scripts.insert(
        ranger.into(),
        json!([
            c("checkevent", &[CHAMPION]),
            c("iffalse", &["MnAfterChampionSnow"]),
            c("checkevent", &[RESTORED]),
            c("iffalse", &["MnAfterChampionSnow"]),
            c("checkevent", &[REED_WON]),
            c("iftrue", &["MnAfterChampionSnow"]),
            c("jumptextfaceplayer", &["MnAfterChampionReedText"])
        ]),
    );
    publish(pack, ely, &["MnAfterChampionReedText"])?;

    let mut camp = pack
        .data
        .maps
        .get("MnBoundaryWaters")
        .context("Boundary west watch")?
        .clone();
    let watch = "MnWayNorthWatch";
    let border_ranger = "MN_BOUNDARY_BORDER_RANGER";
    ensure!(
        camp.objects.iter().any(|o| o.script == border_ranger)
            && camp
                .scripts
                .get(border_ranger)
                .and_then(Value::as_array)
                .is_some_and(|r| {
                    r.starts_with(&[
                        c("checkevent", &[ARRIVED]),
                        c("iffalse", &[watch]),
                        c("checkevent", &[GIVEN]),
                        c("iftrue", &[watch]),
                    ])
                }),
        "expected the active west watch and its unclaimed crossing receipt"
    );
    let original = c("jumptextfaceplayer", &["MnWayNorthWatchText"]);
    let rows = camp
        .scripts
        .get_mut(watch)
        .and_then(Value::as_array_mut)
        .context("Champion's border directions")?;
    ensure!(
        rows.as_slice()
            == [
                c("checkevent", &[CHAMPION]),
                c("iffalse", &["MnWayLeagueTrail"]),
                original,
            ],
        "expected the watch's Champion and League split"
    );
    // An arrival awaiting the clinic reward is already handled by the actor's
    // entry script. Its other visits reach this branch.
    rows.splice(
        2..2,
        [
            c("checkevent", &[GIVEN]),
            c("iftrue", &["MnAfterChampionWatchHome"]),
            c("checkevent", &[REPORT]),
            c("iftrue", &["MnAfterChampionWatchCrossing"]),
        ],
    );
    for (entry, label) in [
        ("MnAfterChampionWatchHome", "MnAfterChampionWatchHomeText"),
        (
            "MnAfterChampionWatchCrossing",
            "MnAfterChampionWatchCrossingText",
        ),
    ] {
        add(&mut camp, entry, json!([c("jumptextfaceplayer", &[label])]))?;
    }
    publish(
        pack,
        camp,
        &[
            "MnAfterChampionWatchHomeText",
            "MnAfterChampionWatchCrossingText",
        ],
    )
}

fn shore_guide(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnElyShorePass")
        .context("Ely shore pass")?
        .clone();
    let guide = "MnElyShorePassTrailGuide";
    let label = "MnAfterChampionShoreGuideText";
    ensure!(
        room.objects
            .iter()
            .any(|o| o.script == guide && (o.x, o.y) == (8, 14))
            && room.scripts.get(guide)
                == Some(&json!([c(
                    "jumptextfaceplayer",
                    &["MnElyShorePassTrailGuideText"]
                )])),
        "expected the trail guide beside the snow and grass"
    );
    room.scripts
        .insert(guide.into(), json!([c("jumptextfaceplayer", &[label])]));
    publish(pack, room, &[label])
}

fn add(room: &mut MapModule, label: &str, body: Value) -> Result<()> {
    ensure!(
        room.scripts.insert(label.into(), body).is_none(),
        "occupied ranger script {label}"
    );
    Ok(())
}

fn publish(pack: &mut CompiledGamePack, mut room: MapModule, labels: &[&str]) -> Result<()> {
    for label in labels {
        ensure!(
            !room.scripts.contains_key(*label),
            "occupied ranger text {label}"
        );
        install_canonical(&mut room, label)?;
    }
    room.script_flag_commands = crate::parse_script_flag_commands(&room.id, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(&room.id, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(&room.id, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(&room.id, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
