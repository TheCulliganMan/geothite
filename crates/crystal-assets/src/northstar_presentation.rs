//! Restore accepted leader identities and distinguish the river teacher from Reed.

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue::AuthoredDialoguePage;

const PADDLER: &str = "MN_RIVER_FUN_LOWER_REED";
const PADDLER_MAP: &str = "MnMississippiLowerWoodsC00R00";
const MANIFEST: &str = "northstar-native-presentation-v1";

// Cast manifests accept these battle sources. The later field repair acceptance
// records preserve those exact portraits/palettes and replace only field poses.
const LEADERS: [(&str, &str, &str); 8] = [
    (
        include_str!("../../../planning/minnesota/npcs/cast/02-varel/native-pixels-v2.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/02-varel/native-pixels-v2.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/02-varel.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/03-torven/native-pixels-v3.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/03-torven/native-pixels-v3.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/03-torven.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/04-avrin/native-pixels-v2.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/04-avrin/native-pixels-v3.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/04-avrin.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/05-brannic/native-pixels-v2.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/05-brannic/native-pixels-v3.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/05-brannic.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/06-saven/native-pixels-v3.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/06-saven/native-pixels-v3.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/06-saven.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/07-sivren/native-pixels-v2.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/07-sivren/native-pixels-v2.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/07-sivren.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/08-orlis/native-pixels-v3.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/08-orlis/native-pixels-v3.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/08-orlis.json"
        ),
    ),
    (
        include_str!("../../../planning/minnesota/npcs/cast/09-norek/native-pixels-v2.json"),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/09-norek/native-pixels-v2.json"
        ),
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/09-norek.json"
        ),
    ),
];

/// Bind accepted native leaders before creating their rematch variants.
///
/// Keeps trainer IDs, teams, rewards, actor positions and original shared art.
/// The existing private leader classes also retain leader music and happiness
/// behavior. The caller must derive the final pack identity after this step.
///
/// # Errors
/// Rejects repeated installation, missing/ambiguous actors, stale art preimages,
/// conflicting private classes, invalid native art or a reused teacher name.
pub(crate) fn install(pack: &mut CompiledGamePack, story_source: &str) -> Result<()> {
    ensure!(
        !pack.report.manifests.iter().any(|entry| entry == MANIFEST),
        "native presentation already installed"
    );
    for (battle_source, field_source, binding_source) in LEADERS {
        let art = crate::minnesota_native_npc::compile(battle_source, Some(field_source))?;
        let binding = current_binding(pack, binding_source)?;
        let id = binding["trainer_ids"][0]
            .as_str()
            .context("accepted leader trainer ID")?;
        let class = binding["trainer_class"]
            .as_str()
            .context("accepted leader class")?;
        ensure!(
            crate::is_gym_leader_class(class),
            "native leader role required"
        );
        let mut expected = pack
            .data
            .trainers
            .trainers
            .get(id)
            .context("current leader trainer")?
            .clone();
        expected.trainer_class = class.into();
        let map_name = binding["actors"][0]["map"]
            .as_str()
            .context("accepted leader room")?;
        let room = pack
            .data
            .maps
            .get(map_name)
            .context("current leader room")?
            .clone();
        *pack = crate::bind_minnesota_trainer_art(pack, &serde_json::to_string(&binding)?, &art)?;
        ensure!(
            pack.data.trainers.trainers.get(id) == Some(&expected),
            "native leader binding changed team or battle rules: {id}"
        );
        let revised = pack
            .data
            .maps
            .get(map_name)
            .context("revised leader room")?;
        ensure!(
            room.attributes == revised.attributes
                && room.blocks == revised.blocks
                && room.events == revised.events,
            "native leader binding changed geometry or progression events: {map_name}"
        );
    }
    rename_paddling_teacher(pack, story_source)?;
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

fn current_binding(pack: &CompiledGamePack, source: &str) -> Result<Value> {
    let mut binding: Value = serde_json::from_str(source)?;
    for actor in binding["actors"]
        .as_array_mut()
        .context("accepted leader actors")?
    {
        let map_name = actor["map"].as_str().context("leader map")?;
        let script = actor["script"].as_str().context("leader script")?;
        let map = pack.data.maps.get(map_name).context("current leader map")?;
        let mut matches = map.objects.iter().filter(|object| object.script == script);
        let object = matches.next().context("current leader actor")?;
        ensure!(matches.next().is_none(), "ambiguous leader actor: {script}");
        ensure!(
            Some(object.sprite.as_str()) == actor["previous_sprite"].as_str(),
            "stale leader sprite: {script}"
        );
        // Reviewed gym revisions moved Avrin and Brannic. Bind the same actor
        // at its current position; never transplant the old room coordinates.
        actor["tile"] = serde_json::json!([object.x, object.y]);
    }
    Ok(binding)
}

fn rename_paddling_teacher(pack: &mut CompiledGamePack, story_source: &str) -> Result<()> {
    let story: Value = serde_json::from_str(story_source)?;
    let name = story["native_presentation"]["trainer_names"][PADDLER]
        .as_str()
        .context("canonical paddling teacher name")?;
    ensure!(
        !name.is_empty()
            && name.len() <= 10
            && name.bytes().all(|byte| byte.is_ascii_uppercase())
            && name != "REED",
        "distinct native paddling teacher name required"
    );
    ensure!(
        !pack.data.trainers.trainers.iter().any(|(id, trainer)| {
            id.starts_with("MN_") && id != PADDLER && trainer.name == name
        }),
        "paddling teacher name already belongs to another regional trainer"
    );
    let trainer = pack
        .data
        .trainers
        .trainers
        .get_mut(PADDLER)
        .context("existing paddling teacher")?;
    ensure!(trainer.name == "REED", "stale paddling teacher name");
    trainer.name = name.into();
    let mut map = pack
        .data
        .maps
        .get(PADDLER_MAP)
        .context("paddling teacher map")?
        .clone();
    ensure!(
        map.objects
            .iter()
            .filter(|actor| actor.script == PADDLER)
            .count()
            == 1,
        "one exact paddling teacher actor required"
    );
    for suffix in ["Seen", "Won", "Lost"] {
        let label = format!("{PADDLER}{suffix}");
        ensure!(
            map.scripts.contains_key(&label),
            "missing teacher text {label}"
        );
        let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(
            story["dialogue_overhaul"]["texts"][PADDLER_MAP][&label].clone(),
        )
        .with_context(|| format!("canonical paddling teacher dialogue: {label}"))?;
        map.scripts
            .insert(label, crate::authored_dialogue::compile(&pages)?);
    }
    crate::gym_scripts::finish(pack, &mut map)?;
    crate::authored_town_publication::replace(pack, map)
}
