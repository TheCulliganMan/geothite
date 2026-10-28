//! Short cross-line listening sequences, without changing any cry's gain or PCM.

use std::path::Path;

use anyhow::Result;
use crystal_audio::original_cry::CryDesignBook;
use crystal_audio::synth::{self, SynthContext};
use crystal_audio::wav;

use super::escape;

pub(super) fn render(book: &CryDesignBook, directory: &Path) -> Result<String> {
    // These are listening selections from the original book, not another source
    // catalog. Other design books can omit them without affecting normal export.
    let selections: &[(&str, &str, &[&str])] = &[
        (
            "pseudo-legendary",
            "Wrigglet → Pupump → Mosquitan · revision 8",
            &["MN_151", "MN_152", "MN_153"],
        ),
        (
            "legendary-land",
            "Legendary land voices · revision 8",
            &["MN_095", "MN_098", "MN_099", "MN_248"],
        ),
        (
            "legendary-water-and-memory",
            "Water, ice and memory · revision 8",
            &["MN_096", "MN_097", "MN_100", "MN_250"],
        ),
        (
            "gills-and-deer",
            "Gill voices and deer voices",
            &["MN_310", "MN_311", "MN_312", "MN_313"],
        ),
        (
            "velcro",
            "Velcro peels beside voiced calls",
            &["MN_298", "MN_299", "MN_310", "MN_312"],
        ),
        (
            "large",
            "Four large voices",
            &["MN_050", "MN_106", "MN_204", "MN_276"],
        ),
        (
            "small",
            "Four small voices",
            &["MN_001", "MN_063", "MN_067", "MN_198"],
        ),
        (
            "insects",
            "Four insect voices",
            &["MN_021", "MN_024", "MN_144", "MN_153"],
        ),
        (
            "clear",
            "Four clear calls",
            &["MN_003", "MN_042", "MN_076", "MN_127"],
        ),
    ];
    let programs = book.programs()?;
    let context = SynthContext::cartridge()?;
    let mut html = String::new();
    for &(slug, title, ids) in selections {
        if !ids.iter().all(|id| programs.contains_key(*id)) {
            continue;
        }
        let mut pcm = Vec::new();
        let mut names = Vec::new();
        for &id in ids {
            if !pcm.is_empty() {
                pcm.extend(std::iter::repeat_n(0, 22050));
            }
            pcm.extend(synth::render(&programs[id], context.clone())?.downsample());
            let variant = book
                .families
                .values()
                .find_map(|family| family.species.get(id));
            if let Some(variant) = variant {
                names.push(variant.character.split(':').next().unwrap_or(id));
            }
        }
        let filename = format!("contrast-{slug}.wav");
        wav::write_canonical_wav(&directory.join(&filename), &pcm)?;
        html.push_str(&format!(
            "<article><h3>{}</h3><p>{}</p><audio controls preload=\"none\" src=\"{}\"></audio></article>",
            escape(title), escape(&names.join(" → ")), filename
        ));
    }
    if !html.is_empty() {
        html = format!(
            "<aside><h2>Compare different lines</h2><p>Calls play in the order shown, separated by half a second. Each retains its exported sound and level.</p><main>{html}</main></aside>"
        );
    }
    Ok(html)
}
