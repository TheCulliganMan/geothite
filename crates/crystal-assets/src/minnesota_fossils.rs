//! Paired Driftless fossils and retryable Rochester restoration.
use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::northstar_lair_authoring::flag;
use crate::northstar_rival_arc::{actor, map, publish, put};
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "minnesota-fossil-research-v1";
const CHOSEN: &str = "EVENT_MN_FOSSIL_CHOSEN";
const LENS: &str = "EVENT_MN_FOSSIL_CHOSE_LENS";
const EXCHANGED: &str = "EVENT_MN_FOSSIL_EXCHANGED";

/// Installs the optional paired specimen choice and a first-badge restoration
/// service. The other specimen becomes available after the regional Champion.
/// Failed item delivery leaves the choice open. A full party retains the fossil;
/// specimens are consumed only after a successful native Pokémon gift.
/// Existing terrain, encounters, trainers, source art and audio are preserved.
///
/// # Errors
/// Rejects duplicate installation, occupied identities, missing native species,
/// items or maps, occupied actor positions, and invalid compiled output.
pub fn build_minnesota_fossils(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "fossils already installed"
    );
    for name in ["TRILUME", "SIPHLET"] {
        ensure!(
            base.data.pokemon.contains_key(name),
            "missing Minnesota fossil {name}"
        );
        ensure!(
            base.data
                .story_event_script_constants
                .global
                .contains_key(&format!("MN_DEX_MEMBER_{name}")),
            "inactive Minnesota fossil {name}"
        );
    }
    let mut pack = base.clone();
    for name in [
        CHOSEN,
        LENS,
        EXCHANGED,
        "EVENT_MN_TRILUME_RESTORED",
        "EVENT_MN_SIPHLET_RESTORED",
    ] {
        flag(&mut pack, name)?;
    }
    for (id, name, description) in [
        (
            "LENS_FOSSIL",
            "LENS FOSSIL",
            "A fossil with many\ntiny eye lenses.",
        ),
        (
            "SPIRE_FOSSIL",
            "SPIRE FOSSIL",
            "A chambered fossil\nfrom an old sea.",
        ),
    ] {
        let mut item = pack
            .data
            .items
            .get("MACHINE_PART")
            .context("native key item template")?
            .clone();
        item.name = name.into();
        item.description = description.into();
        item.script_name = id.into();
        ensure!(
            pack.data.items.insert(id.into(), item).is_none(),
            "occupied fossil item {id}"
        );
    }
    let mut bluff = map(&pack, "MnR01C00R00")?;
    add_actor(&pack, &mut bluff, "MnFossilBluff", [39, 21])?;
    put(
        &mut bluff,
        "MnFossilBluff",
        vec![
            c("faceplayer", &[]),
            c("checkevent", &[CHOSEN]),
            c("iftrue", &["MnFossilExchange"]),
            c("opentext", &[]),
            c("writetext", &["MnFossilSlabText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnFossilClose"]),
            c("writetext", &["MnFossilLensChoiceText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnFossilSpireConfirm"]),
            c("writetext", &["MnFossilLensConfirmText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnFossilClose"]),
            c("verbosegiveitem", &["LENS_FOSSIL", "1"]),
            c("iffalse", &["MnFossilBagFull"]),
            c("setevent", &[LENS]),
            c("sjump", &["MnFossilChoiceDone"]),
        ],
    );
    put(
        &mut bluff,
        "MnFossilSpireConfirm",
        vec![
            c("writetext", &["MnFossilSpireConfirmText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnFossilClose"]),
            c("verbosegiveitem", &["SPIRE_FOSSIL", "1"]),
            c("iffalse", &["MnFossilBagFull"]),
            c("sjump", &["MnFossilChoiceDone"]),
        ],
    );
    put(
        &mut bluff,
        "MnFossilChoiceDone",
        vec![
            c("setevent", &[CHOSEN]),
            c("writetext", &["MnFossilDirectionsText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnFossilClose"]),
        ],
    );
    put(
        &mut bluff,
        "MnFossilExchange",
        vec![
            c("checkevent", &[EXCHANGED]),
            c("iftrue", &["MnFossilBluffRepeat"]),
            c("checkevent", &["EVENT_MN_LEAGUE_CHAMPION"]),
            c("iffalse", &["MnFossilBluffRepeat"]),
            c("opentext", &[]),
            c("writetext", &["MnFossilExchangeText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnFossilClose"]),
            c("checkevent", &[LENS]),
            c("iftrue", &["MnFossilExchangeSpire"]),
            c("verbosegiveitem", &["LENS_FOSSIL", "1"]),
            c("sjump", &["MnFossilExchangeResult"]),
        ],
    );
    put(
        &mut bluff,
        "MnFossilExchangeSpire",
        vec![
            c("verbosegiveitem", &["SPIRE_FOSSIL", "1"]),
            c("sjump", &["MnFossilExchangeResult"]),
        ],
    );
    put(
        &mut bluff,
        "MnFossilExchangeResult",
        vec![
            c("iffalse", &["MnFossilBagFull"]),
            c("setevent", &[EXCHANGED]),
            c("writetext", &["MnFossilDirectionsText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnFossilClose"]),
        ],
    );
    put(
        &mut bluff,
        "MnFossilBluffRepeat",
        vec![c("jumptext", &["MnFossilDirectionsText"])],
    );
    common_text(&mut bluff);
    for (label, lines) in [
        (
            "MnFossilSlabText",
            vec![
                "This bluff was once",
                "the floor of a sea.",
                "The slab holds two",
                "fossil specimens.",
                "Study one with me?",
            ],
        ),
        (
            "MnFossilLensChoiceText",
            vec!["Take LENS FOSSIL?", "YES: many-eyed", "NO: spire shell"],
        ),
        (
            "MnFossilLensConfirmText",
            vec!["Choose LENS FOSSIL", "for this trip?"],
        ),
        (
            "MnFossilSpireConfirmText",
            vec!["Take SPIRE FOSSIL", "for this trip?"],
        ),
        (
            "MnFossilDirectionsText",
            vec![
                "Bring your fossil",
                "to our researcher",
                "by ROCHESTER's",
                "clinic courtyard.",
                "Earn the CARE BADGE",
                "and leave room for",
                "your new friend.",
            ],
        ),
        (
            "MnFossilExchangeText",
            vec![
                "Your League record",
                "secured a research",
                "exchange. We can",
                "offer the specimen",
                "you left behind.",
                "Would you like it?",
            ],
        ),
    ] {
        text(&mut bluff, label, &lines);
    }
    publish(&mut pack, bluff)?;

    let mut city = map(&pack, "MnRochester")?;
    add_actor(&pack, &mut city, "MnFossilResearcher", [20, 10])?;
    put(
        &mut city,
        "MnFossilResearcher",
        vec![
            c("faceplayer", &[]),
            c("opentext", &[]),
            c("checkflag", &["ENGINE_ZEPHYRBADGE"]),
            c("iffalse", &["MnFossilNeedBadge"]),
            c("checkitem", &["LENS_FOSSIL"]),
            c("iftrue", &["MnFossilRestoreTRILUME"]),
            c("checkitem", &["SPIRE_FOSSIL"]),
            c("iftrue", &["MnFossilRestoreSIPHLET"]),
            c("writetext", &["MnFossilNeedSpecimenText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnFossilClose"]),
        ],
    );
    for (species, item) in [("TRILUME", "LENS_FOSSIL"), ("SIPHLET", "SPIRE_FOSSIL")] {
        let root = format!("MnFossilRestore{species}");
        let restored = format!("EVENT_MN_{species}_RESTORED");
        let label = format!("{root}Text");
        put(
            &mut city,
            &root,
            vec![
                c("checkevent", &[&restored]),
                c("iftrue", &["MnFossilAlreadyRestored"]),
                c("writetext", &[&label]),
                c("yesorno", &[]),
                c("iffalse", &["MnFossilClose"]),
                c("readvar", &["VAR_PARTYCOUNT"]),
                c("ifequal", &["6", "MnFossilPartyFull"]),
                c("closetext", &[]),
                c("givepoke", &[species, "10"]),
                c("ifequal", &["2", "MnFossilGiftFailed"]),
                c("takeitem", &[item]),
                c("setevent", &[&restored]),
                c("opentext", &[]),
                c("writetext", &["MnFossilRestoredText"]),
                c("waitbutton", &[]),
                c("sjump", &["MnFossilClose"]),
            ],
        );
        text(
            &mut city,
            &label,
            &[
                &format!("Restore {species}"),
                "at level 10?",
                "The fossil is used",
                "only when it lives.",
            ],
        );
    }
    for (label, target) in [
        ("MnFossilNeedBadge", "MnFossilNeedBadgeText"),
        ("MnFossilPartyFull", "MnFossilPartyFullText"),
        ("MnFossilAlreadyRestored", "MnFossilAlreadyRestoredText"),
    ] {
        put(
            &mut city,
            label,
            vec![
                c("writetext", &[target]),
                c("waitbutton", &[]),
                c("sjump", &["MnFossilClose"]),
            ],
        );
    }
    put(
        &mut city,
        "MnFossilGiftFailed",
        vec![c("opentext", &[]), c("sjump", &["MnFossilPartyFull"])],
    );
    common_text(&mut city);
    for (label, lines) in [
        (
            "MnFossilNeedBadgeText",
            vec![
                "Earn the CARE BADGE",
                "before we restore",
                "a fossil together.",
            ],
        ),
        (
            "MnFossilNeedSpecimenText",
            vec![
                "A researcher on",
                "DRIFTLESS TRAIL",
                "studies a paired",
                "fossil slab. Bring",
                "one specimen here.",
            ],
        ),
        (
            "MnFossilPartyFullText",
            vec![
                "Your party is full",
                "Make room and",
                "return. Keep your",
                "fossil until then.",
            ],
        ),
        (
            "MnFossilAlreadyRestoredText",
            vec!["We have restored", "that specimen's", "that ancient line."],
        ),
        (
            "MnFossilRestoredText",
            vec!["A living window", "into the old sea!", "Care for it well."],
        ),
    ] {
        text(&mut city, label, &lines);
    }
    publish(&mut pack, city)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.report.items = pack.data.items.len();
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

fn add_actor(pack: &CompiledGamePack, room: &mut MapModule, id: &str, at: [u16; 2]) -> Result<()> {
    ensure!(
        !room.scripts.contains_key(id),
        "occupied fossil script {id}"
    );
    ensure!(
        !room.objects.iter().any(|o| [o.x, o.y] == at),
        "occupied fossil researcher"
    );
    let kind = crate::authored_town_geometry::kind(pack, room, at)?;
    ensure!(
        matches!(kind, "FLOOR" | "TALL_GRASS"),
        "fossil researcher needs native walkable ground"
    );
    actor(pack, room, id, "SPRITE_SCIENTIST", at)
}

fn common_text(room: &mut MapModule) {
    put(
        room,
        "MnFossilClose",
        vec![c("closetext", &[]), c("end", &[])],
    );
    put(
        room,
        "MnFossilBagFull",
        vec![
            c("writetext", &["MnFossilBagFullText"]),
            c("waitbutton", &[]),
            c("sjump", &["MnFossilClose"]),
        ],
    );
    text(
        room,
        "MnFossilBagFullText",
        &[
            "Your key pocket is",
            "full. Make room and",
            "ask again. We will",
            "keep the specimen.",
        ],
    );
}

fn text(room: &mut MapModule, label: &str, lines: &[&str]) {
    authoring::text(room, label, lines);
}
