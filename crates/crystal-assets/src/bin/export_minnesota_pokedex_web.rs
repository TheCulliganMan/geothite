//! Export the authored Northwoods Pokédex as a self-contained static website.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

#[allow(dead_code)]
#[path = "../minnesota_pokemon_pixel_art.rs"]
mod minnesota_pokemon_pixel_art;

#[allow(dead_code)]
#[path = "../minnesota_sprite_sheet.rs"]
mod sprite;

#[path = "export_minnesota_pokedex_web/moves.rs"]
mod web_moves;

#[path = "export_minnesota_pokedex_web/starters.rs"]
mod starters;

#[path = "export_minnesota_pokedex_web/media.rs"]
mod web_media;

#[path = "export_minnesota_pokedex_web/showcases.rs"]
mod showcases;

#[path = "export_minnesota_pokedex_web/machine_catalog.rs"]
mod machine_catalog;

#[path = "export_minnesota_pokedex_web/machine_revision.rs"]
mod machine_revision;

#[path = "export_minnesota_pokedex_web/locations.rs"]
mod locations;

const MOVES: &str = include_str!("../../../../web-client/pokedex/moves.html");
const INDEX: &str = include_str!("../../../../web-client/pokedex/index.html");
const ENTRY: &str = include_str!("../../../../web-client/pokedex/entry.html");
const CSS: &str = include_str!("../../../../web-client/pokedex/style.css");
const MEDIA_CSS: &str = include_str!("../../../../web-client/pokedex/media.css");
const JS: &str = include_str!("../../../../web-client/pokedex/app.js");
const TYPES: [&str; 18] = [
    "NORMAL", "FIRE", "WATER", "GRASS", "ELECTRIC", "ICE", "FIGHTING", "POISON", "GROUND",
    "FLYING", "PSYCHIC", "BUG", "ROCK", "GHOST", "DRAGON", "DARK", "STEEL", "FAIRY",
];

fn read(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| path.display().to_string())?)
        .context("reading authoring JSON")
}
fn string(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}
fn array(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn title(value: &str) -> String {
    value
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().to_string() + &chars.as_str().to_lowercase())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn slug(member: &Value) -> String {
    string(&member["name"]).to_ascii_lowercase()
}
fn number(member: &Value) -> u64 {
    member["regional_number"]
        .as_u64()
        .expect("validated regional Pokédex number")
}

fn total(profile: &Value) -> u64 {
    profile["base_stats"]
        .as_object()
        .map(|s| s.values().map(|v| v.as_u64().unwrap_or(0)).sum())
        .unwrap_or(0)
}
fn region(family: &Value) -> (&str, &str) {
    let raw = string(&family["region"]);
    if raw.contains("lake country") || raw.contains("northern_lakes") {
        return ("northern_lakes", "Northern lakes");
    }
    if raw.contains("cold_stream") {
        return ("cold_streams", "Coldwater streams");
    }
    match raw {
        "peatlands" => ("peatlands", "Peatlands"),
        "iron_range" => ("iron_range", "Iron Range"),
        "northwoods" => ("northwoods", "Northwoods"),
        "north_shore" => ("north_shore", "North Shore"),
        "central_lakes" => ("central_lakes", "Central lakes"),
        "prairie" => ("prairie", "Prairie"),
        "river_bluffs" => ("river_bluffs", "River bluffs"),
        "towns" | "twin_cities" | "urban" | "industrial" => ("towns", "Towns & traditions"),
        "agriculture" | "prairie_rivers" => ("prairie", "Prairie"),
        "driftless" | "mississippi_river" | "southeast_blufflands" => {
            ("river_bluffs", "River bluffs")
        }
        "voyageurs_greenstone" | "northeast_biwabik" => ("iron_range", "Iron Range"),
        "lake_superior_coast"
        | "lake_superior_observatory"
        | "lake_superior_cold_air"
        | "north_shore_cutover"
        | "duluth_smoked_fish" => ("north_shore", "North Shore"),
        "boundary_waters"
        | "boundary_waters_winter"
        | "boundary_waters_campcraft"
        | "boundary_waters_lichen"
        | "gunflint_divide" => ("boundary_waters", "Boundary Waters"),
        "northwoods_logging_tales" | "hardwood_canopy" => ("northwoods", "Northwoods"),
        "rainy_lake_superior_fish" => ("northern_lakes", "Northern lakes"),
        _ => ("across_minnesota", "Across Minnesota"),
    }
}
fn rarity(family: &Value) -> &str {
    let classification = family
        .get("rarity")
        .and_then(Value::as_str)
        .unwrap_or_else(|| string(&family["classification"]));
    if classification.contains("pseudo") {
        "pseudo"
    } else if classification.contains("myth") {
        "mythical"
    } else if classification.contains("legend") {
        "legendary"
    } else {
        "ordinary"
    }
}
fn color(kind: &str) -> &str {
    match kind {
        "FIRE" => "#b75c3e",
        "WATER" => "#467b9b",
        "GRASS" => "#597c45",
        "ELECTRIC" => "#a47a1c",
        "ICE" => "#4d8c8e",
        "FIGHTING" => "#9b5347",
        "POISON" => "#916599",
        "GROUND" => "#947044",
        "FLYING" => "#747caa",
        "PSYCHIC" => "#b26083",
        "BUG" => "#778342",
        "ROCK" => "#8d7d52",
        "GHOST" => "#77648c",
        "DRAGON" => "#716799",
        "DARK" => "#66615e",
        "STEEL" => "#657e80",
        "FAIRY" => "#a54875",
        _ => "#7c7b6b",
    }
}
fn badge(kind: &str) -> String {
    let kind = if kind == "PSYCHIC_TYPE" {
        "PSYCHIC"
    } else {
        kind
    };
    let ink = color(kind);
    format!(
        "<span class=\"type-badge\" aria-label=\"{} type\" style=\"--type-ink:{ink};--type-bg:{ink}10;--type-border:{ink}40\">{}</span>",
        title(kind),
        title(kind)
    )
}
fn badges(member: &Value) -> String {
    array(&member["types"])
        .iter()
        .map(|v| badge(string(v)))
        .collect()
}
fn href(member: &Value) -> String {
    format!("/pokedex/species/{}.html", slug(member))
}
fn image(member: &Value, base: &str, size: usize, lazy: bool) -> String {
    format!(
        "<img src=\"{base}/sprites/{}-front.png\" alt=\"{}\" width=\"{size}\" height=\"{size}\" {} decoding=\"async\">",
        string(&member["id"]).to_ascii_lowercase().replace('_', "-"),
        esc(string(&member["name"])),
        if lazy { "loading=\"lazy\"" } else { "" }
    )
}
fn header() -> String {
    format!(
        "<header class=\"site-header wrap\"><a class=\"brand\" href=\"/pokedex/\"><span class=\"brand-mark\" aria-hidden=\"true\">{}</span><span><strong>GEOTHITE</strong><small>NORTHWOODS POKÉDEX</small></span></a><nav class=\"main-nav\" aria-label=\"Main navigation\"><a href=\"/pokedex/#guide\">Pokédex</a><a href=\"/pokedex/?view=families#guide\">Evolution lines</a><a href=\"/pokedex/?rarity=rare#guide\">Legends</a><a href=\"/pokedex/moves.html\">Moves</a><a href=\"/pokedex/state-fair.html\">State Fair</a></nav><a class=\"play-link\" href=\"/\">Play Geothite <span>↗</span></a></header>",
        "<i></i>".repeat(12)
    )
}
fn footer() -> &'static str {
    "<footer class=\"site-footer wrap\"><a class=\"footer-brand\" href=\"/pokedex/\">GEOTHITE <span>✦</span></a><p>REGIONAL POKÉDEX · #001–{{SPECIES_COUNT}}</p><small>Original fan-made creatures. Pokémon is a trademark of Nintendo / Creatures / GAME FREAK.</small><a href=\"#\">BACK TO TOP ↑</a></footer>"
}
fn render(template: &str, fields: &[(&str, String)], base: &str) -> Result<String> {
    let mut html = template
        .replace("{{HEADER}}", &header())
        .replace("{{FOOTER}}", footer())
        .replace("{{ASSET_BASE}}", base);
    for (key, value) in fields {
        html = html.replace(&format!("{{{{{key}}}}}"), value);
    }
    ensure!(!html.contains("{{"), "unfilled template field");
    Ok(html)
}
fn pack_rgb(rgb: &Value) -> Result<u16> {
    let channels = array(rgb);
    ensure!(channels.len() == 3, "RGB5 requires three channels");
    let mut word = 0;
    for (i, v) in channels.iter().enumerate() {
        let c = v.as_u64().context("RGB5 integer")?;
        ensure!(c <= 31, "RGB5 range");
        word |= (c as u16) << (i * 5);
    }
    Ok(word)
}
fn swatches(words: &[u16]) -> String {
    words
        .iter()
        .map(|w| {
            let channel = |shift: u32| {
                let c = (*w >> shift) & 31u16;
                (c << 3) | (c >> 2)
            };
            let hex = format!("#{:02x}{:02x}{:02x}", channel(0), channel(5), channel(10));
            format!("<i style=\"--swatch:{hex}\" title=\"{hex}\"></i>")
        })
        .collect()
}
fn move_name(id: &str) -> String {
    match id {
        "PSYCHIC_M" => "Psychic".into(),
        "SOLARBEAM" => "Solar Beam".into(),
        "THUNDERSHOCK" => "Thunder Shock".into(),
        "ANCIENTPOWER" => "Ancient Power".into(),
        "EXTREMESPEED" => "Extreme Speed".into(),
        "DRAGONBREATH" => "Dragon Breath".into(),
        "DYNAMICPUNCH" => "Dynamic Punch".into(),
        "SANDATTACK" => "Sand Attack".into(),
        "POISONPOWDER" => "Poison Powder".into(),
        "THUNDERPUNCH" => "Thunder Punch".into(),
        "SONICBOOM" => "Sonic Boom".into(),
        "BUBBLEBEAM" => "Bubble Beam".into(),
        "DOUBLESLAP" => "Double Slap".into(),
        "SELFDESTRUCT" => "Self-Destruct".into(),
        "VICEGRIP" => "Vice Grip".into(),
        "SMOKESCREEN" => "Smokescreen".into(),
        _ => title(id),
    }
}
fn moves_table(
    rows: &[(String, String)],
    moves: &BTreeMap<String, crystal_core::models::Move>,
    level: bool,
) -> Result<String> {
    if rows.is_empty() {
        return Ok(
            "<div class=\"move-empty\">No moves listed for this learning method.</div>".into(),
        );
    }
    let mut html = format!(
        "<table class=\"move-table\"><thead><tr><th>{}</th><th>MOVE</th><th>TYPE</th><th>POWER</th><th>ACCURACY</th><th>PP</th></tr></thead><tbody>",
        if level { "LEVEL" } else { "METHOD" }
    );
    for (method, id) in rows {
        let m = moves
            .get(id)
            .with_context(|| format!("unknown move {id}"))?;
        let kind = m.move_type.to_uppercase();
        let power = if m.power == 0 {
            "—".into()
        } else {
            m.power.to_string()
        };
        let accuracy = if m.accuracy == 0 {
            "—".into()
        } else {
            format!("{}%", m.accuracy)
        };
        html += &format!(
            "<tr><td class=\"move-level\">{}</td><td>{}</td><td>{}</td><td>{power}</td><td>{accuracy}</td><td>{}</td></tr>",
            esc(method),
            move_name(id),
            badge(&kind),
            m.pp
        );
    }
    html += "</tbody></table>";
    Ok(html)
}
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "--locations") {
        ensure!(
            args.len() == 4,
            "usage: export_minnesota_pokedex_web --locations PACK BASELINE_RELEASE NEW_RELEASE"
        );
        let pack = crystal_assets::read_verified_compiled_game_pack(Path::new(&args[1]))?;
        return locations::publish(&pack, Path::new(&args[2]), Path::new(&args[3]));
    }
    if args.first().is_some_and(|arg| arg == "--machines") {
        ensure!(
            args.len() == 5,
            "usage: export_minnesota_pokedex_web --machines PACK BASELINE_RELEASE OUTPUT_DIR ASSET_BASE"
        );
        let pack = crystal_assets::read_verified_compiled_game_pack(Path::new(&args[1]))?;
        return machine_revision::publish(
            &pack,
            Path::new(&args[2]),
            Path::new(&args[3]),
            &args[4],
        );
    }
    ensure!(
        matches!(args.len(), 6 | 7 | 9),
        "usage: export_minnesota_pokedex_web BASE_PACK POKEMON_ROOT NATIVE_ROOT PROFILES_JSON OUTPUT_DIR ASSET_BASE [ANIMATION_ROOT [OVERWORLD_ROOT CRY_URL_BASE]]"
    );
    let root = Path::new(&args[1]);
    let native = Path::new(&args[2]);
    let output = Path::new(&args[4]);
    let base = args[5].trim_end_matches('/');
    ensure!(
        base.starts_with("/pokedex/") && !base.contains(['"', '<', '>', '\\']),
        "asset base must be an absolute /pokedex/ path"
    );
    let mut roster = read(&root.join("roster.json"))?;
    let source_count = array(&roster["families"])
        .iter()
        .map(|f| array(&f["members"]).len())
        .sum::<usize>();
    let curation_path = root.join("roster-curation.json");
    let curation = if curation_path.exists() {
        read(&curation_path)?
    } else {
        json!({"retired_families": []})
    };
    let retired = array(&curation["retired_families"])
        .iter()
        .map(|f| string(&f["id"]).to_owned())
        .collect::<BTreeSet<_>>();
    let known_families = array(&roster["families"])
        .iter()
        .map(|f| string(&f["id"]).to_owned())
        .collect::<BTreeSet<_>>();
    ensure!(
        retired.is_subset(&known_families),
        "curation names an unknown family"
    );
    let order = read(&root.join("pokedex-order.json"))?;
    let mut numbers = BTreeMap::new();
    let mut ordered_numbers = BTreeSet::new();
    for entry in array(&order["species"]) {
        let id = string(&entry["id"]).to_owned();
        let n = entry["number"]
            .as_u64()
            .context("regional number must be an integer")?;
        ensure!(
            (1..=source_count as u64).contains(&n) && ordered_numbers.insert(n),
            "duplicate or invalid regional number"
        );
        ensure!(
            numbers.insert(id, n).is_none(),
            "duplicate species in regional order"
        );
    }
    ensure!(
        numbers.len() == source_count,
        "regional order must cover every source species"
    );
    let families = roster["families"]
        .as_array_mut()
        .context("missing families")?;
    for family in families.iter_mut() {
        for member in family["members"]
            .as_array_mut()
            .context("missing family members")?
        {
            let n = *numbers
                .get(string(&member["id"]))
                .context("species missing from regional order")?;
            member
                .as_object_mut()
                .context("invalid species identity")?
                .insert("regional_number".into(), json!(n));
        }
    }
    families.sort_by_key(|family| {
        (
            retired.contains(string(&family["id"])),
            number(&family["members"][0]),
        )
    });
    let active_family_count = families
        .iter()
        .filter(|family| !retired.contains(string(&family["id"])))
        .count();
    let mut active_species_count = 0;
    for family in families.iter_mut() {
        if retired.contains(string(&family["id"])) {
            continue;
        }
        for member in family["members"]
            .as_array_mut()
            .context("missing family members")?
        {
            active_species_count += 1;
            member["regional_number"] = json!(active_species_count);
        }
    }
    ensure!(
        active_species_count > 0,
        "curation removed the complete roster"
    );
    let pack = crystal_assets::read_verified_compiled_game_pack(Path::new(&args[0]))?;
    let moves = &pack.data().moves;
    let machines = machine_catalog::MachineCatalog::load(&pack.data().items, moves)?;
    let move_catalog = web_moves::load(root, moves)?;
    let animation_root = Path::new(
        args.get(6)
            .map(String::as_str)
            .unwrap_or("planning/minnesota/renders/signatures-20261026-v1"),
    );
    web_moves::copy_previews(&move_catalog, animation_root, output)?;
    let mut profiles = read(Path::new(&args[3]))?;
    for profile in profiles["species"]
        .as_array_mut()
        .context("missing profiles")?
    {
        let family_member = array(&roster["families"]).iter().find_map(|family| {
            array(&family["members"])
                .iter()
                .find(|member| member["id"] == profile["id"])
                .map(|member| (family, member))
        });
        if let Some((family, member)) = family_member {
            let runtime_name = string(&member["name"]).to_ascii_uppercase();
            if !retired.contains(string(&family["id"])) {
                let species = pack
                    .data()
                    .pokemon
                    .get(&runtime_name)
                    .with_context(|| format!("missing active species {runtime_name}"))?;
                let authored: Vec<String> = serde_json::from_value(profile["tmhm_moves"].clone())?;
                machine_catalog::validate_compatibility(
                    &authored,
                    &species.tmhm_learnset,
                    &machines,
                    moves,
                )
                .with_context(|| format!("TM/HM learnset for {runtime_name}"))?;
            }
            if let Some(learnset) = pack.data().learnsets.get(&runtime_name) {
                profile["level_moves"] = serde_json::to_value(learnset)?;
            }
        }
    }
    let profile_map = array(&profiles["species"])
        .iter()
        .map(|p| (string(&p["id"]), p))
        .collect::<BTreeMap<_, _>>();
    let palette_doc = read(&root.join("palettes.json"))?;
    let registry = read(&root.join("sprite-generation.json"))?;
    fs::create_dir_all(output.join("sprites"))?;
    fs::create_dir_all(output.join("species"))?;
    let media = if args.len() == 9 {
        let active = array(&roster["families"])
            .iter()
            .filter(|family| !retired.contains(string(&family["id"])))
            .flat_map(|family| array(&family["members"]))
            .map(|member| string(&member["id"]).to_owned())
            .collect();
        web_media::MediaCatalog::export(Path::new(&args[7]), output, base, &args[8], &active)?
    } else {
        web_media::MediaCatalog::default()
    };
    let mut shiny_palettes = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for family in array(&registry["families"]) {
        let manifest = read(&root.join(string(&family["manifest"])))?;
        for frame in array(&manifest["sprites"]) {
            let id = string(&frame["species_id"]);
            let view = string(&frame["view"]);
            ensure!(
                seen.insert((id.to_owned(), view.to_owned())),
                "duplicate frame"
            );
            let stem = format!("{}.{}", string(&frame["id"]), view);
            let folder = native.join(string(&family["id"]));
            let src = folder.join(format!("{stem}.png"));
            let dest = format!("{}-{view}", id.to_ascii_lowercase().replace('_', "-"));
            fs::copy(&src, output.join("sprites").join(format!("{dest}.png")))
                .with_context(|| src.display().to_string())?;
            let p = profile_map.get(id).context("missing species profile")?;
            let shiny = [
                32767,
                pack_rgb(&p["shiny_rgb5"][0])?,
                pack_rgb(&p["shiny_rgb5"][1])?,
                0,
            ];
            let indices = read(&folder.join(format!("{stem}.json")))?;
            let rows = array(&indices["rows"])
                .iter()
                .map(|r| string(r).replace('.', "0"))
                .collect::<Vec<_>>();
            fs::write(
                output.join("sprites").join(format!("{dest}-shiny.png")),
                sprite::indexed_rows_png(&rows, shiny)?,
            )?;
            shiny_palettes.insert(id.to_owned(), shiny);
        }
    }
    let mut members = array(&roster["families"])
        .iter()
        .flat_map(|f| array(&f["members"]).iter().map(move |m| (f, m)))
        .collect::<Vec<_>>();
    members
        .sort_by_key(|(family, member)| (retired.contains(string(&family["id"])), number(member)));
    ensure!(
        members.len() == source_count && seen.len() == source_count * 2,
        "expected complete active and cut-proposal native art"
    );
    let active_members = &members[..active_species_count];
    let mut cards = String::new();
    let mut downloads = Vec::new();
    let mut cut_downloads = Vec::new();
    let mut regions = BTreeSet::new();
    for (position, (family, member)) in members.iter().enumerate() {
        let id = string(&member["id"]);
        let name = esc(string(&member["name"]));
        let family_id = string(&family["id"]);
        let is_cut = retired.contains(family_id);
        let profile = profile_map[id];
        let compatibility: Vec<String> = serde_json::from_value(profile["tmhm_moves"].clone())?;
        let machine_learnset = machines.learnset(&compatibility);
        let (region_key, region_label) = region(family);
        regions.insert((region_label, region_key));
        let types = array(&member["types"])
            .iter()
            .map(string)
            .collect::<Vec<_>>()
            .join(" ");
        let move_search = ["level_moves", "tmhm_moves", "egg_moves"]
            .iter()
            .flat_map(|key| array(&profile[*key]))
            .map(|v| move_name(string(if v.is_array() { &v[1] } else { v })))
            .collect::<Vec<_>>()
            .join(" ");
        let search = format!(
            "{} {} {} {} {} {} {} {} {}",
            string(&member["name"]),
            types,
            string(&family["theme"]),
            string(&family["region"]),
            string(&profile["classification"]),
            move_search,
            machine_learnset
                .iter()
                .map(|machine| machine.code.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            starters::role(family).map(|(_, label)| label).unwrap_or(""),
            if is_cut {
                id.to_owned()
            } else {
                format!("#{:03}", number(member))
            }
        );
        let stat_total = total(profile);
        let num = number(member);
        let cut_reason = array(&curation["retired_families"])
            .iter()
            .find(|entry| string(&entry["id"]) == family_id)
            .map(|entry| string(&entry["reason"]))
            .unwrap_or("");
        let hidden = if is_cut { " hidden" } else { "" };
        let card_label = if is_cut {
            "<span class=\"cut-label\">Cut proposal</span>".to_owned()
        } else {
            format!("<span>#{num:03}</span>")
        };
        let kind = rarity(family);
        let stage = member["stage"].as_u64().unwrap_or(1);
        let starter_badge = starters::badge(family);
        let species_media = media.controls(member, !is_cut, false);
        cards += &format!(
            "<article class=\"pokemon-card\" data-id=\"{id}\" data-family=\"{family_id}\" data-cut=\"{is_cut}\" data-name=\"{name}\" data-number=\"{num}\" data-total=\"{stat_total}\" data-types=\"{types}\" data-region=\"{region_key}\" data-stage=\"{stage}\" data-rarity=\"{kind}\" data-search=\"{}\"{hidden}><div class=\"card-top\">{card_label}</div><button type=\"button\" class=\"save-button\" data-save=\"{id}\" data-name=\"{name}\" aria-label=\"Save {name} to notebook\" aria-pressed=\"false\">☆</button><a class=\"card-link\" href=\"{}\"><div class=\"card-art\">{}</div><h3>{name}</h3><p class=\"card-classification\">{}</p>{starter_badge}<div class=\"type-badges\">{}</div></a>{species_media}<div class=\"card-region\"><span>{region_label}</span><span>↗</span></div></article>",
            esc(&search),
            href(member),
            image(member, base, 112, true),
            esc(string(&profile["classification"])),
            badges(member)
        );
        let stem = id.to_ascii_lowercase().replace('_', "-");
        let portrait = format!(
            "<img id=\"specimen-portrait\" src=\"{base}/sprites/{stem}-front.png\" data-name=\"{name}\" data-normal-front=\"{base}/sprites/{stem}-front.png\" data-normal-back=\"{base}/sprites/{stem}-back.png\" data-shiny-front=\"{base}/sprites/{stem}-front-shiny.png\" data-shiny-back=\"{base}/sprites/{stem}-back-shiny.png\" alt=\"{name}, normal front sprite\" width=\"280\" height=\"280\">"
        );
        let normal = array(&palette_doc["pokemon"][id]["gbc_rgb555_words"])
            .iter()
            .map(|v| v.as_u64().unwrap_or(0) as u16)
            .collect::<Vec<_>>();
        let palettes = format!(
            "<div id=\"palette-normal\" class=\"swatches\">{}</div><div id=\"palette-shiny\" class=\"swatches\" hidden>{}</div>",
            swatches(&normal),
            swatches(&shiny_palettes[id])
        );
        let dex_text = array(&profile["dex_pages"])
            .iter()
            .map(|value| string(value).replace(" @ ", " "))
            .collect::<Vec<_>>()
            .join(" ");
        let dex = format!("<p>{}</p>", esc(&dex_text));
        let mut stats = String::new();
        for (key, label) in [
            ("hp", "HP"),
            ("attack", "Attack"),
            ("defense", "Defense"),
            ("special_attack", "Sp. Atk"),
            ("special_defense", "Sp. Def"),
            ("speed", "Speed"),
        ] {
            let value = profile["base_stats"][key].as_u64().unwrap_or(0);
            stats += &format!(
                "<div><dt>{label}</dt><dd>{value}</dd><div class=\"stat-track\" aria-hidden=\"true\"><span style=\"--value:{}%\"></span></div></div>",
                (value as f64 / 255. * 100.).round()
            );
        }
        let mut training = String::from("<dl class=\"training-grid\">");
        for (label, value) in [
            ("Catch rate", profile["catch_rate"].to_string()),
            ("Base experience", profile["base_exp"].to_string()),
            ("Hatch cycles", profile["hatch_cycles"].to_string()),
            (
                "Growth rate",
                title(string(&profile["growth_rate"]).trim_start_matches("GROWTH_")),
            ),
            (
                "Egg groups",
                array(&profile["egg_groups"])
                    .iter()
                    .map(|g| title(string(g).trim_start_matches("EGG_")))
                    .collect::<Vec<_>>()
                    .join(" / "),
            ),
        ] {
            training += &format!("<div><dt>{label}</dt><dd>{}</dd></div>", esc(&value));
        }
        training += "</dl>";
        let mut evolution = String::new();
        for m in array(&family["members"]) {
            let mp = profile_map[string(&m["id"])];
            let method = if mp["evolution"].is_null() {
                if array(&family["members"]).len() == 1 {
                    "Single stage".into()
                } else {
                    "Final evolution".into()
                }
            } else if let Some(level) = mp["evolution"]["level"].as_u64() {
                format!("Evolves at Lv. {level}")
            } else {
                title(string(&mp["evolution"]["method"]))
            };
            evolution += &format!(
                "<a class=\"evolution-step {}\" href=\"{}\">{}<strong>{}</strong><span>{}</span></a>",
                if string(&m["id"]) == id {
                    "current"
                } else {
                    ""
                },
                href(m),
                image(m, base, 112, true),
                esc(string(&m["name"])),
                esc(&method)
            );
        }
        let habitat = family
            .get("habitat")
            .and_then(Value::as_str)
            .unwrap_or(region_label);
        let locations = array(&family["proposed_routes"])
            .iter()
            .map(|v| esc(string(v)))
            .collect::<Vec<_>>()
            .join(" · ");
        let location = format!(
            "{}<small>Encounter locations are design concepts for the Minnesota region; this guide does not imply they are currently catchable in the live game.</small>",
            if locations.is_empty() {
                String::new()
            } else {
                format!("<p class=\"planned-location\">Planned encounters: {locations}</p>")
            }
        );
        let level = array(&profile["level_moves"])
            .iter()
            .map(|v| (format!("Lv. {}", v[0]), string(&v[1]).to_owned()))
            .collect::<Vec<_>>();
        let tutor_moves = machines.tutors(&compatibility);
        let machine = machine_learnset
            .iter()
            .map(|m| (m.code.clone(), m.move_id.clone()))
            .collect::<Vec<_>>();
        let tutors = tutor_moves
            .iter()
            .map(|id| ("Tutor".into(), (*id).to_owned()))
            .collect::<Vec<_>>();
        let egg = array(&profile["egg_moves"])
            .iter()
            .map(|v| ("Inherited".into(), string(v).to_owned()))
            .collect::<Vec<_>>();
        let inches = profile["height_inches"].as_u64().unwrap_or(0);
        let pounds = profile["weight_tenths_lb"].as_u64().unwrap_or(0);
        let sources=array(&family["sources"]).iter().filter_map(|s|{
            let url=string(&s["url"]); if !url.starts_with("https://") {return None;}
            let label=s.get("title").and_then(Value::as_str).unwrap_or(url);
            Some(format!("<li><a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">{} ↗</a><p>{}</p></li>",esc(url),esc(label),esc(s.get("grounding").and_then(Value::as_str).unwrap_or("Regional inspiration; creature designs and powers are fictional."))))
        }).collect::<String>();
        let sources = if sources.is_empty() {
            String::new()
        } else {
            format!(
                "<details class=\"source-notes\"><summary>Regional inspiration &amp; sources <span>＋</span></summary><ul>{sources}</ul></details>"
            )
        };
        let pager = |offset: isize| {
            let n = position as isize + offset;
            if n < 0 || n as usize >= members.len() {
                return "<span></span>".into();
            }
            let m = members[n as usize].1;
            let neighbor_cut = retired.contains(string(&members[n as usize].0["id"]));
            if neighbor_cut != is_cut {
                return "<span></span>".into();
            }
            let label = if is_cut {
                "CUT PROPOSAL".to_owned()
            } else {
                format!("NO. {:03}", number(m))
            };
            format!(
                "<a href=\"{}\">{}<span>{label}</span><strong>{}</strong></a>",
                href(m),
                if offset < 0 { "←" } else { "→" },
                esc(string(&m["name"]))
            )
        };
        let fields = vec![
            ("SPECIES_COUNT", active_species_count.to_string()),
            ("FAMILY_COUNT", active_family_count.to_string()),
            ("ID", id.into()),
            ("NAME", name.clone()),
            (
                "DESCRIPTION",
                esc(&format!(
                    "Meet {}: {} Pokémon. Explore original sprites, evolution, stats and moves in the Northwoods Pokédex.",
                    string(&member["name"]),
                    string(&profile["classification"])
                )),
            ),
            ("NUMBER", format!("{num:03}")),
            (
                "INDEX_LINK",
                if is_cut {
                    "/pokedex/?cut=1#guide"
                } else {
                    "/pokedex/#guide"
                }
                .into(),
            ),
            (
                "ENTRY_LABEL",
                if is_cut {
                    "CUT PROPOSAL".into()
                } else {
                    format!("REGIONAL NO. {num:03}")
                },
            ),
            (
                "PORTRAIT_LABEL",
                if is_cut {
                    format!("PROPOSED / {}", esc(id))
                } else {
                    format!("POKÉDEX / {num:03}")
                },
            ),
            (
                "CUT_NOTICE",
                if is_cut {
                    format!(
                        "<aside class=\"cut-notice\"><strong>Cut proposal</strong><p>This Pokémon was removed from the active regional Pokédex.</p><p>{}</p></aside>",
                        esc(cut_reason)
                    )
                } else {
                    String::new()
                },
            ),
            ("PORTRAIT", portrait),
            ("MEDIA_CONTROLS", media.controls(member, !is_cut, true)),
            ("PALETTES", palettes),
            ("CLASSIFICATION", esc(string(&profile["classification"]))),
            ("STARTER_ROLE", starter_badge),
            (
                "RARITY",
                if kind == "ordinary" {
                    String::new()
                } else {
                    format!("<span class=\"rarity-label\">{}</span>", title(kind))
                },
            ),
            ("TYPES", badges(member)),
            ("DEX", dex),
            ("HEIGHT", format!("{}′ {}″", inches / 12, inches % 12)),
            ("WEIGHT", format!("{}.{:01} lb", pounds / 10, pounds % 10)),
            ("REGION", region_label.into()),
            ("THEME", esc(string(&family["theme"]))),
            ("TOTAL", stat_total.to_string()),
            ("STATS", stats),
            ("TRAINING", training),
            (
                "EVOLUTION_TITLE",
                if array(&family["members"]).len() == 1 {
                    "No evolutions".into()
                } else {
                    "Evolution line".into()
                },
            ),
            ("EVOLUTION", evolution),
            ("HABITAT", esc(habitat)),
            ("LOCATION", location),
            ("LEVEL_COUNT", level.len().to_string()),
            ("MACHINE_COUNT", machine.len().to_string()),
            ("TUTOR_COUNT", tutors.len().to_string()),
            ("EGG_COUNT", egg.len().to_string()),
            ("LEVEL_MOVES", moves_table(&level, moves, true)?),
            ("MACHINE_MOVES", moves_table(&machine, moves, false)?),
            ("TUTOR_MOVES", moves_table(&tutors, moves, false)?),
            ("EGG_MOVES", moves_table(&egg, moves, false)?),
            ("MOVE_ANIMATIONS", {
                let learned = level
                    .iter()
                    .chain(&machine)
                    .chain(&tutors)
                    .chain(&egg)
                    .cloned()
                    .collect::<Vec<_>>();
                let cards =
                    web_moves::animation_cards(&move_catalog, &members, base, None, Some(&learned));
                if cards.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<section class=\"move-animation-entry\" aria-label=\"Move animations\"><h2>Move animations</h2>{cards}</section>"
                    )
                }
            }),
            ("SOURCES", sources),
            ("PREVIOUS", pager(-1)),
            ("NEXT", pager(1)),
        ];
        fs::write(
            output
                .join("species")
                .join(format!("{}.html", slug(member))),
            render(ENTRY, &fields, base)?,
        )?;
        let download = json!({"identity":member,"family":family_id,"status":if is_cut { "cut" } else { "active" },"cut_reason":if is_cut { Some(cut_reason) } else { None },"starter_role":starters::role(family).map(|(key, _)| key),"theme":family["theme"],"region":region_label,"profile":profile,"machine_learnset":machine_learnset,"tutor_moves":tutor_moves,"normal_rgb555":normal,"shiny_rgb555":shiny_palettes[id]});
        if is_cut {
            cut_downloads.push(download);
        } else {
            downloads.push(download);
        }
    }
    fs::write(
        output.join("moves.html"),
        render(
            MOVES,
            &[
                ("SPECIES_COUNT", active_species_count.to_string()),
                ("MOVE_INDEX", web_moves::animation_index(&move_catalog)),
                (
                    "SIGNATURE_MOVES",
                    web_moves::animation_cards(
                        &move_catalog,
                        active_members,
                        base,
                        Some("signature"),
                        None,
                    ),
                ),
                (
                    "IMPORTED_MOVES",
                    web_moves::imported_table(&move_catalog, active_members),
                ),
                (
                    "IMPORTED_ANIMATIONS",
                    web_moves::animation_cards(
                        &move_catalog,
                        active_members,
                        base,
                        Some("imported"),
                        None,
                    ),
                ),
                ("MOVE_RULES", esc(string(&move_catalog["rules"]))),
                ("MOVE_SOURCE", esc(string(&move_catalog["source"]["url"]))),
            ],
            base,
        )?,
    )?;
    let mut families = String::new();
    for family in array(&roster["families"]) {
        let ms = array(&family["members"]);
        let first = &ms[0];
        let fid = string(&family["id"]);
        let is_cut = retired.contains(fid);
        let hidden = if is_cut { " hidden" } else { "" };
        let cut_label = if is_cut {
            "<span class=\"cut-label\">Cut proposal</span>"
        } else {
            ""
        };
        let name = esc(string(&first["name"]));
        let starter_badge = starters::badge(family);
        let art = ms
            .iter()
            .map(|m| {
                format!(
                    "<a href=\"{}\">{}<strong>{}</strong></a>",
                    href(m),
                    image(m, base, 112, true),
                    esc(string(&m["name"]))
                )
            })
            .collect::<String>();
        families += &format!(
            "<article class=\"family-card\" data-family=\"{fid}\" data-cut=\"{is_cut}\" data-name=\"{name}\" data-number=\"{}\" data-total=\"{}\"{hidden}><div class=\"family-card-top\"><span>LINE {}</span><span>{} {}</span></div>{cut_label}{starter_badge}<div class=\"family-art\">{art}</div><p>{}</p></article>",
            number(first),
            total(profile_map[string(&ms[ms.len() - 1]["id"])]),
            fid.trim_start_matches("MN_F"),
            ms.len(),
            if ms.len() == 1 { "STAGE" } else { "STAGES" },
            esc(string(&family["theme"]))
        );
    }
    let filters=TYPES.iter().map(|t|format!("<button type=\"button\" class=\"type-filter\" data-type=\"{t}\" aria-label=\"Filter by {} type\" aria-pressed=\"false\"><i class=\"type-dot\" aria-hidden=\"true\" style=\"--type:{}\"></i>{}</button>",title(t),color(t),title(t))).collect();
    let options = regions
        .iter()
        .map(|(label, key)| format!("<option value=\"{key}\">{label}</option>"))
        .collect();
    fs::write(
        output.join("index.html"),
        render(
            INDEX,
            &[
                ("SPECIES_COUNT", active_species_count.to_string()),
                ("FAMILY_COUNT", active_family_count.to_string()),
                ("CARDS", cards),
                ("FAMILIES", families),
                ("TYPE_FILTERS", filters),
                ("REGIONS", options),
                ("MOTION_CONTROL", media.motion_control()),
            ],
            base,
        )?,
    )?;
    fs::write(
        output.join("style.css"),
        format!("{CSS}\n{MEDIA_CSS}\n{}", showcases::NAV_CSS),
    )?;
    fs::write(output.join("app.js"), JS)?;
    showcases::write(output, base, &header(), footer(), active_species_count)?;
    fs::write(
        output.join("pokedex.json"),
        serde_json::to_vec(
            &json!({"schema_version":1,"species_count":active_species_count,"family_count":active_family_count,"species":downloads,"cut_species_count":cut_downloads.len(),"cut_family_count":retired.len(),"cut_species":cut_downloads,"moves":move_catalog["moves"],"machine_catalog":machines.entries,"pack_content_hash":pack.identity()?.content_hash,"walking_thumbnail_count":media.thumbnail_count(),"cry_species_count":if args.len()==9 {active_species_count} else {0}}),
        )?,
    )?;
    fs::write(
        output.join("mark.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 32 32\"><rect width=\"32\" height=\"32\" rx=\"6\" fill=\"#234b3b\"/><path d=\"M12 4h8v8h8v8h-8v8h-8v-8H4v-8h8z\" fill=\"#e8d59a\"/><path d=\"M12 12h8v8h-8z\" fill=\"#bd5739\"/></svg>",
    )?;
    println!(
        "Exported {active_species_count} active entries, {} cut proposals and {} native/shiny sprites to {}",
        source_count - active_species_count,
        source_count * 4,
        output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_psychic_alias_only_normalizes_the_display_badge() {
        assert_eq!(badge("PSYCHIC_TYPE"), badge("PSYCHIC"));
        let mov = crystal_core::models::Move {
            source_index: 93,
            name: "CONFUSION".into(),
            move_type: "PSYCHIC_TYPE".into(),
            power: 50,
            accuracy: 100,
            pp: 25,
            effect: "CONFUSE_HIT".into(),
            effect_chance: 10,
            stat: None,
            amount: None,
        };
        let moves = BTreeMap::from([("CONFUSION".into(), mov)]);
        let html = moves_table(&[("Lv. 9".into(), "CONFUSION".into())], &moves, true)
            .expect("native move uses canonical display badge");
        assert!(html.contains("aria-label=\"Psychic type\""));
        assert!(html.contains(color("PSYCHIC")));
        assert!(!html.contains("Psychic Type"));
        assert_eq!(moves["CONFUSION"].move_type, "PSYCHIC_TYPE");
    }

    #[test]
    fn fairy_species_and_moves_have_labeled_distinct_badges() {
        assert!(TYPES.contains(&"FAIRY"));
        assert_ne!(color("FAIRY"), color("NORMAL"));
        let mov = crystal_core::models::Move {
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
        let moves = BTreeMap::from([("FAIRY_WIND".into(), mov)]);
        let html = moves_table(&[("9".into(), "FAIRY_WIND".into())], &moves, true)
            .expect("Fairy move table uses packed move facts");
        assert!(html.contains("aria-label=\"Fairy type\""));
        assert!(html.contains(color("FAIRY")));
        assert!(html.contains("Fairy Wind"));
        assert_eq!(
            badges(&json!({"types":["GRASS","FAIRY"]}))
                .matches("type-badge")
                .count(),
            2
        );
    }
    #[test]
    fn palettes_and_html_preserve_authoring_boundaries() {
        assert_eq!(pack_rgb(&json!([31, 31, 31])).expect("valid RGB5"), 32767);
        assert!(pack_rgb(&json!([32, 0, 0])).is_err());
        assert_eq!(esc("<a \"&\">"), "&lt;a &quot;&amp;&quot;&gt;");
        assert!(render("{{MISSING}}", &[], "/pokedex/releases/test").is_err());
        let rows = vec!["0".repeat(56); 56];
        assert!(sprite::indexed_rows_png(&rows, [32767, 31, 1024, 0]).is_ok());
        let mut invalid = rows;
        invalid[1] = "4".repeat(56);
        assert!(sprite::indexed_rows_png(&invalid, [32767, 31, 1024, 0]).is_err());
    }
}
