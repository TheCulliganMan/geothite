//! Static move descriptions and production-rendered move previews.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_core::models::Move;
use serde_json::Value;

use super::{array, badge, esc, href, read, string, title};

pub(super) fn load(root: &Path, moves: &BTreeMap<String, Move>) -> Result<Value> {
    let mut catalog = read(&root.join("northwoods-moves.json"))?;
    for filename in ["fairy-moves.json", "starter-signature-moves.json"] {
        let path = root.join(filename);
        if path.try_exists()? {
            let supplement = read(&path)?;
            merge_supplement(&mut catalog, &supplement, moves)?;
        }
    }
    catalog["moves"]
        .as_array_mut()
        .context("missing move definitions")?
        .retain(|entry| moves.contains_key(string(&entry["id"])));
    let mut ids = BTreeSet::new();
    for entry in array(&catalog["moves"]) {
        ensure!(
            ids.insert(string(&entry["id"])),
            "duplicate exported move ID"
        );
        let actual = serde_json::to_value(&moves[string(&entry["id"])])?;
        ensure!(
            actual == entry["battle"],
            "move description differs from packed battle definition"
        );
    }
    Ok(catalog)
}

fn merge_supplement(
    catalog: &mut Value,
    supplement: &Value,
    moves: &BTreeMap<String, Move>,
) -> Result<()> {
    let additions: Vec<_> = array(&supplement["moves"])
        .iter()
        .filter(|entry| moves.contains_key(string(&entry["id"])))
        .cloned()
        .collect();
    if additions.is_empty() {
        return Ok(());
    }
    catalog["moves"]
        .as_array_mut()
        .context("missing move definitions")?
        .extend(additions);
    if let Some(rules) = supplement["rules"]
        .as_str()
        .filter(|rules| !rules.is_empty())
    {
        catalog["rules"] = Value::String(format!("{} {rules}", string(&catalog["rules"])));
    }
    Ok(())
}

pub(super) fn copy_previews(catalog: &Value, source: &Path, output: &Path) -> Result<()> {
    fs::create_dir_all(output.join("animations"))?;
    for entry in array(&catalog["moves"]) {
        let id = string(&entry["id"]).to_ascii_lowercase();
        for side in ["player", "enemy"] {
            for palette in ["normal", "shiny"] {
                let variant = format!("{side}-{palette}");
                let folder = source.join(&id);
                let animation = folder.join(format!("{variant}.png"));
                let bytes = fs::read(&animation).with_context(|| {
                    format!(
                        "missing production-rendered preview {}",
                        animation.display()
                    )
                })?;
                ensure!(
                    bytes.windows(4).any(|chunk| chunk == b"acTL"),
                    "move preview is not an APNG"
                );
                fs::write(
                    output
                        .join("animations")
                        .join(format!("{id}-{variant}.png")),
                    bytes,
                )?;
                let keyframes = folder.join(format!("{variant}-keyframes.json"));
                let metadata = keyframes
                    .try_exists()?
                    .then(|| read(&keyframes))
                    .transpose()?;
                let index = still_frame(metadata.as_ref(), string(&entry["id"]), &variant)?;
                fs::copy(
                    folder.join(&variant).join(format!("frame-{index:03}.png")),
                    output
                        .join("animations")
                        .join(format!("{id}-{variant}-still.png")),
                )?;
            }
        }
    }
    Ok(())
}

// Older capture sets have no contact metadata and retain the original opening still.
fn still_frame(metadata: Option<&Value>, move_id: &str, variant: &str) -> Result<u64> {
    let Some(metadata) = metadata else {
        return Ok(0);
    };
    ensure!(
        metadata["move_id"].as_str() == Some(move_id),
        "preview contact metadata move mismatch"
    );
    ensure!(
        metadata["variant"].as_str() == Some(variant),
        "preview contact metadata variant mismatch"
    );
    let frames = metadata["frames"].as_u64().context("preview frame count")?;
    let indices = metadata["contact_indices"]
        .as_array()
        .context("preview contact indices")?;
    ensure!(
        indices.len() == 6,
        "preview contact metadata requires six indices"
    );
    for index in indices {
        ensure!(
            index.as_u64().is_some_and(|index| index < frames),
            "preview contact frame is outside its timeline"
        );
    }
    indices[3].as_u64().context("preview target-contact frame")
}

fn learners(entry: &Value, members: &[(&Value, &Value)]) -> String {
    array(&entry["learners"])
        .iter()
        .filter_map(|learner| {
            let (_, member) = members
                .iter()
                .find(|(_, m)| m["id"] == learner["species_id"])?;
            Some(format!(
                "<a href=\"{}\">{} <span>Lv. {}</span></a>",
                href(member),
                esc(string(&member["name"])),
                learner["level"]
            ))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn animation_index(catalog: &Value) -> String {
    array(&catalog["moves"])
        .iter()
        .map(|entry| {
            format!(
                "<a href=\"#{}\">{}</a>",
                string(&entry["id"]).to_ascii_lowercase(),
                esc(string(&entry["display_name"]))
            )
        })
        .collect()
}

pub(super) fn animation_cards(
    catalog: &Value,
    members: &[(&Value, &Value)],
    base: &str,
    kind: Option<&str>,
    learned: Option<&[(String, String)]>,
) -> String {
    array(&catalog["moves"])
        .iter()
        .filter(|entry| kind.is_none_or(|kind| string(&entry["kind"]) == kind))
        .filter(|entry| learned.is_none_or(|moves| moves.iter().any(|(_, id)| id == string(&entry["id"]))))
        .map(|entry| {
            let move_id = string(&entry["id"]);
            let id = move_id.to_ascii_lowercase();
            let name = esc(string(&entry["display_name"]));
            let animation = format!("{base}/animations/{id}");
            let battle = &entry["battle"];
            let signature = string(&entry["kind"]) == "signature";
            let card_class = if signature { "signature-card" } else { "imported-animation-card" };
            let label = if signature { "SIGNATURE MOVE" } else { "IMPORTED MOVE" };
            let learning = learned.map(|moves| {
                let methods = moves.iter().filter(|(_, id)| id == move_id).map(|(method, _)| esc(method)).collect::<Vec<_>>().join(" · ");
                format!("<p class=\"move-learning\"><span class=\"eyebrow\">THIS POKÉMON</span> {methods}</p>")
            }).unwrap_or_default();
            format!(
                "<article class=\"move-animation-card {card_class}\" id=\"{id}\" data-move-id=\"{move_id}\"><div class=\"move-animation-heading\"><span class=\"eyebrow\">{label}</span><h3>{name}</h3><div>{}<span class=\"move-category\">{}</span></div></div><div class=\"battle-preview\" data-animation-base=\"{animation}\" data-move-name=\"{name}\"><img src=\"{animation}-player-normal-still.png\" alt=\"{name}, player-side normal battle preview\" width=\"320\" height=\"288\" loading=\"lazy\" decoding=\"async\"><div class=\"animation-controls\"><button type=\"button\" data-animation-play aria-pressed=\"false\" aria-label=\"Play {name} animation\">Play</button><button type=\"button\" data-animation-side aria-pressed=\"false\" aria-label=\"Show {name} from enemy side\">Enemy side</button><button type=\"button\" data-animation-shiny aria-pressed=\"false\" aria-label=\"Show {name} with shiny Pokémon\">✦ Shiny</button></div><a class=\"animation-download\" href=\"{animation}-player-normal.png\">Open animation ↗</a></div><dl class=\"move-facts\"><div><dt>POWER</dt><dd>{}</dd></div><div><dt>ACCURACY</dt><dd>{}%</dd></div><div><dt>PP</dt><dd>{}</dd></div></dl><p>{}</p>{learning}<div class=\"move-learners\"><span class=\"eyebrow\">LEARNED BY</span>{}</div></article>",
                badge(string(&battle["type"])), title(string(&entry["category"])),
                battle["power"], battle["accuracy"], battle["pp"],
                esc(string(&entry["description"])), learners(entry, members)
            )
        })
        .collect()
}

pub(super) fn imported_table(catalog: &Value, members: &[(&Value, &Value)]) -> String {
    let mut html = String::from(
        "<div class=\"move-table-wrap\"><table class=\"move-table imported-moves\"><thead><tr><th>MOVE</th><th>TYPE / CLASS</th><th>POWER</th><th>ACC.</th><th>PP</th><th>EFFECT &amp; LEARNERS</th></tr></thead><tbody>",
    );
    for entry in array(&catalog["moves"]) {
        if string(&entry["kind"]) != "imported" {
            continue;
        }
        let battle = &entry["battle"];
        html += &format!(
            "<tr><td><a href=\"#{}\"><strong>{}</strong></a><small>Gen {}</small></td><td>{}<small>{}</small></td><td>{}</td><td>{}%</td><td>{}</td><td><p>{}</p><div class=\"move-learners\">{}</div></td></tr>",
            string(&entry["id"]).to_ascii_lowercase(),
            esc(string(&entry["display_name"])),
            entry["introduced_generation"],
            badge(string(&battle["type"])),
            title(string(&entry["category"])),
            battle["power"],
            battle["accuracy"],
            battle["pp"],
            esc(string(&entry["description"])),
            learners(entry, members)
        );
    }
    html + "</tbody></table></div>"
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{BTreeMap, Move, animation_cards, imported_table, merge_supplement, still_frame};

    #[test]
    fn idle_preview_uses_validated_native_contact_frame_with_legacy_fallback() {
        assert_eq!(
            still_frame(None, "AQUA_JET", "player-normal").expect("legacy still"),
            0
        );
        let metadata = json!({"move_id":"AQUA_JET","variant":"player-normal","frames":20,"contact_indices":[1,4,7,10,12,18]});
        assert_eq!(
            still_frame(Some(&metadata), "AQUA_JET", "player-normal").expect("target contact"),
            10
        );
        assert!(still_frame(Some(&metadata), "IRON_HEAD", "player-normal").is_err());
        assert!(still_frame(Some(&metadata), "AQUA_JET", "enemy-normal").is_err());
        let mut invalid = metadata;
        invalid["contact_indices"][3] = json!(20);
        assert!(still_frame(Some(&invalid), "AQUA_JET", "player-normal").is_err());
    }

    #[test]
    fn species_animation_cards_use_actual_learning_methods() {
        let catalog = json!({"moves":[
            {"id":"FAIRY_WIND","kind":"imported","display_name":"Fairy Wind","category":"special","description":"A soft wind.","learners":[],"battle":{"type":"FAIRY","power":40,"accuracy":100,"pp":30}},
            {"id":"BOG_SIPHON","kind":"signature","display_name":"Bog Siphon","category":"special","description":"A draining strike.","learners":[{"species_id":"NOT_THIS_SPECIES","level":20}],"battle":{"type":"BUG","power":75,"accuracy":100,"pp":10}}
        ]});
        let learned = vec![
            ("Lv. 16".into(), "FAIRY_WIND".into()),
            ("Inherited".into(), "FAIRY_WIND".into()),
        ];
        let html = animation_cards(&catalog, &[], "/pokedex", None, Some(&learned));
        assert!(html.contains("imported-animation-card"));
        assert!(html.contains("Lv. 16 · Inherited"));
        assert!(html.contains("data-move-id=\"FAIRY_WIND\""));
        assert!(!html.contains("bog_siphon"));
        assert!(animation_cards(&catalog, &[], "/pokedex", None, Some(&[])).is_empty());
        let signatures = animation_cards(&catalog, &[], "/pokedex", Some("signature"), None);
        assert!(signatures.contains("bog_siphon"));
        assert!(!signatures.contains("fairy_wind"));
    }

    #[test]
    fn fairy_supplement_only_advertises_moves_present_in_the_pack() {
        let mut catalog =
            json!({"moves":[{"id":"BOG_SIPHON","kind":"signature"}],"rules":"Native rules."});
        let supplement = json!({"moves":[{"id":"FAIRY_WIND","kind":"imported","display_name":"Fairy Wind","introduced_generation":6,"category":"special","description":"A soft wind.","learners":[],"battle":{"type":"FAIRY","power":40,"accuracy":100,"pp":30}}],"rules":"Fairy overlay."});
        let original = catalog.clone();
        merge_supplement(&mut catalog, &supplement, &BTreeMap::new())
            .expect("base pack omits optional Fairy moves");
        assert_eq!(catalog, original);
        let fairy_wind = Move {
            source_index: 265,
            name: "FAIRY_WIND".into(),
            move_type: "FAIRY".into(),
            power: 40,
            accuracy: 100,
            pp: 30,
            effect: "NORMAL_HIT".into(),
            effect_chance: 0,
            stat: None,
            amount: None,
        };
        merge_supplement(
            &mut catalog,
            &supplement,
            &BTreeMap::from([("FAIRY_WIND".into(), fairy_wind)]),
        )
        .expect("extended pack includes the optional move");
        assert_eq!(catalog["moves"][0], original["moves"][0]);
        assert_eq!(catalog["moves"].as_array().expect("merged moves").len(), 2);
        assert_eq!(catalog["rules"], "Native rules. Fairy overlay.");
        let html = imported_table(&catalog, &[]);
        assert!(html.contains("Fairy Wind"));
        assert!(html.contains("aria-label=\"Fairy type\""));
    }
}
