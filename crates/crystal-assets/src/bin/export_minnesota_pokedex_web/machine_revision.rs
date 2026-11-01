//! Publish machine learnsets while preserving an existing reviewed web release.
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_assets::CompiledGamePack;
use serde_json::json;

use super::{ENTRY, array, esc, machine_catalog, moves_table, read, render, string};

fn replace_panel(html: &str, panel: &str) -> Result<String> {
    let start = html
        .find("<section class=\"move-panel\"")
        .context("missing move panel")?;
    let end = start
        + html[start..]
            .find("</section>")
            .context("unclosed move panel")?
        + "</section>".len();
    Ok(format!("{}{panel}{}", &html[..start], &html[end..]))
}

fn copy_release(source: &Path, output: &Path, old_base: &str, base: &str) -> Result<()> {
    fs::create_dir(output)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = output.join(entry.file_name());
        if kind.is_dir() {
            copy_release(&entry.path(), &target, old_base, base)?;
        } else {
            ensure!(kind.is_file(), "baseline must contain regular files");
            if matches!(
                entry.path().extension().and_then(|ext| ext.to_str()),
                Some("html" | "json" | "js" | "css")
            ) {
                fs::write(
                    target,
                    fs::read_to_string(entry.path())?.replace(old_base, base),
                )?;
            } else {
                fs::copy(entry.path(), target)?;
            }
        }
    }
    Ok(())
}

pub(super) fn publish(
    pack: &CompiledGamePack,
    baseline: &Path,
    output: &Path,
    base: &str,
) -> Result<()> {
    ensure!(
        base.starts_with("/pokedex/releases/")
            && !base.contains(['"', '<', '>', '\\'])
            && !base.ends_with('/'),
        "invalid immutable asset base"
    );
    ensure!(!output.exists(), "machine revision output must be fresh");
    let baseline = baseline.canonicalize()?;
    let destination = output
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(output.file_name().context("output name")?);
    ensure!(
        !destination.starts_with(&baseline),
        "output must be outside the baseline release"
    );
    let old_base = format!(
        "/pokedex/releases/{}",
        baseline
            .file_name()
            .context("baseline release name")?
            .to_str()
            .context("UTF-8 release name")?
    );
    let mut index = fs::read_to_string(baseline.join("index.html"))?;
    ensure!(
        index.contains(&old_base),
        "baseline asset URLs do not match its release directory"
    );
    let catalog = machine_catalog::MachineCatalog::load(&pack.data().items, &pack.data().moves)?;
    let mut data = read(&baseline.join("pokedex.json"))?;
    let panel_start = ENTRY
        .find("<section class=\"move-panel\"")
        .context("move template")?;
    let panel_end = panel_start
        + ENTRY[panel_start..]
            .find("</section>")
            .context("move template closing tag")?
        + "</section>".len();
    let template = &ENTRY[panel_start..panel_end];
    copy_release(&baseline, output, &old_base, base)?;
    index = index.replace(&old_base, base);
    for group in ["species", "cut_species"] {
        for entry in data[group].as_array_mut().context("species array")? {
            let name = string(&entry["identity"]["name"]).to_owned();
            let id = string(&entry["identity"]["id"]);
            let profile = &entry["profile"];
            let compatibility: Vec<String> = serde_json::from_value(profile["tmhm_moves"].clone())?;
            if group == "species" {
                let species = pack
                    .data()
                    .pokemon
                    .get(&name.to_ascii_uppercase())
                    .with_context(|| format!("missing active species {name}"))?;
                machine_catalog::validate_compatibility(
                    &compatibility,
                    &species.tmhm_learnset,
                    &catalog,
                    &pack.data().moves,
                )
                .with_context(|| format!("TM/HM learnset for {name}"))?;
            }
            let machines = catalog.learnset(&compatibility);
            let tutors = catalog.tutors(&compatibility);
            let machine_rows: Vec<_> = machines
                .iter()
                .map(|m| (m.code.clone(), m.move_id.clone()))
                .collect();
            let tutor_rows: Vec<_> = tutors
                .iter()
                .map(|id| ("Tutor".into(), (*id).to_owned()))
                .collect();
            let level: Vec<_> = array(&profile["level_moves"])
                .iter()
                .map(|mov| (format!("Lv. {}", mov[0]), string(&mov[1]).to_owned()))
                .collect();
            let egg: Vec<_> = array(&profile["egg_moves"])
                .iter()
                .map(|id| ("Inherited".into(), string(id).to_owned()))
                .collect();
            let panel = render(
                template,
                &[
                    ("LEVEL_COUNT", level.len().to_string()),
                    ("MACHINE_COUNT", machines.len().to_string()),
                    ("TUTOR_COUNT", tutors.len().to_string()),
                    ("EGG_COUNT", egg.len().to_string()),
                    (
                        "LEVEL_MOVES",
                        moves_table(&level, &pack.data().moves, true)?,
                    ),
                    (
                        "MACHINE_MOVES",
                        moves_table(&machine_rows, &pack.data().moves, false)?,
                    ),
                    (
                        "TUTOR_MOVES",
                        moves_table(&tutor_rows, &pack.data().moves, false)?,
                    ),
                    ("EGG_MOVES", moves_table(&egg, &pack.data().moves, false)?),
                ],
                base,
            )?;
            let page = output
                .join("species")
                .join(format!("{}.html", name.to_ascii_lowercase()));
            fs::write(&page, replace_panel(&fs::read_to_string(&page)?, &panel)?)?;
            let card = index
                .find(&format!("data-id=\"{}\"", esc(id)))
                .context("missing index card")?;
            let search = card
                + index[card..]
                    .find("data-search=\"")
                    .context("missing card search")?
                + "data-search=\"".len();
            let codes = machines
                .iter()
                .map(|m| m.code.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            index.insert_str(search, &format!("{codes} "));
            entry["machine_learnset"] = serde_json::to_value(machines)?;
            entry["tutor_moves"] = json!(tutors);
        }
    }
    data["machine_catalog"] = serde_json::to_value(&catalog.entries)?;
    data["pack_content_hash"] = json!(pack.identity()?.content_hash);
    fs::write(output.join("pokedex.json"), serde_json::to_vec(&data)?)?;
    fs::write(output.join("index.html"), index)?;
    let css = output.join("style.css");
    fs::write(
        &css,
        format!(
            "{}\n.move-tabs{{flex-wrap:wrap;row-gap:0}}.move-tabs button{{white-space:nowrap}}\n",
            fs::read_to_string(&css)?
        ),
    )?;
    println!(
        "Published numbered TM/HM learnsets for {} active entries and {} archived proposals",
        data["species_count"], data["cut_species_count"]
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::replace_panel;

    #[test]
    fn revision_preserves_content_outside_the_move_panel() {
        assert_eq!(
            replace_panel(
                "before<section class=\"move-panel\">old</section>after",
                "new"
            )
            .expect("bounded replacement"),
            "beforenewafter"
        );
        assert!(replace_panel("no panel", "new").is_err());
        assert!(replace_panel("<section class=\"move-panel\">", "new").is_err());
    }
}
