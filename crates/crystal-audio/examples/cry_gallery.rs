//! Render an original design book to an external, ordinary HTML/WAV gallery.
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use crystal_audio::original_cry::CryDesignBook;
use crystal_audio::synth::{self, SynthContext};
use crystal_audio::wav;

#[path = "cry_gallery/contrasts.rs"]
mod contrasts;
#[path = "cry_gallery/similarity.rs"]
mod similarity;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let usage = "usage: cry_gallery DESIGNS_JSON EXTERNAL_OUTPUT_DIRECTORY [ROSTER_JSON [BEFORE_DESIGNS_JSON [REVISION_LABEL]]]";
    let designs = args.next().context(usage)?;
    let directory = PathBuf::from(args.next().context(usage)?);
    let roster = args.next();
    let before = args.next();
    let revision = args.next().unwrap_or_else(|| "Current revision".into());
    ensure!(args.next().is_none(), "{usage}");
    fs::create_dir_all(&directory)?;
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    ensure!(
        !directory.canonicalize()?.starts_with(repository),
        "WAV exports must stay outside the repository"
    );
    let book: CryDesignBook = serde_json::from_slice(&fs::read(designs)?)?;
    let mut order = std::collections::BTreeMap::new();
    if let Some(path) = roster {
        let roster: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        for (family_index, family) in roster["families"]
            .as_array()
            .context("roster families")?
            .iter()
            .enumerate()
        {
            for (stage, member) in family["members"]
                .as_array()
                .context("family members")?
                .iter()
                .enumerate()
            {
                order.insert(
                    member["id"].as_str().context("species id")?.to_owned(),
                    (family_index, stage),
                );
            }
        }
    }
    let programs = book.programs()?;
    let before = before
        .map(|path| -> Result<_> {
            let book: CryDesignBook = serde_json::from_slice(&fs::read(path)?)?;
            book.programs()
        })
        .transpose()?;
    if before.is_some() {
        fs::create_dir_all(directory.join("before"))?;
    }
    let context = SynthContext::cartridge()?;
    let mut hashes = BTreeSet::new();
    let mut voices = Vec::new();
    let mut before_voices = Vec::new();
    let mut html = String::from(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>Northwoods Pokémon cries</title><style>body{font:17px system-ui;max-width:1050px;margin:40px auto;padding:0 22px;background:#10231e;color:#e9eee5}section{margin:36px 0}main{display:grid;grid-template-columns:repeat(auto-fit,minmax(285px,1fr));gap:14px}article{padding:18px;background:#203a31;border-radius:10px}h2{font-size:22px}h3{font-size:18px}audio{width:100%}a{color:#bce6a4}small{opacity:.75}</style><h1>Northwoods Pokémon cries</h1><p>Original pulse and noise performances, synthesized in Rust. Press play to compare related voices.</p>",
    );
    html.push_str(&format!(
        "<p><strong>{}</strong> · {} voices in {} lines. Current players appear first; expand Previous version for comparison.</p>",
        escape(&revision), programs.len(), book.families.len()
    ));
    html.push_str(&contrasts::render(&book, &directory)?);
    let references = [
        (
            "LUGIA",
            "Lugia",
            "A long fractured foreground against a sustained independent upper voice; 1.808 seconds in this render.",
        ),
        (
            "HO_OH",
            "Ho-Oh",
            "Unequal bright response phrases and cycling duty; a developed call need not be uniformly slow.",
        ),
        (
            "TYRANITAR",
            "Tyranitar",
            "Growing envelopes, contrasting registers and independent attacks give a short cry physical weight.",
        ),
        (
            "SUICUNE",
            "Suicune",
            "A nearly fixed upper register against a much more mobile companion; asymmetric pulse and noise timing.",
        ),
        (
            "PARASECT",
            "Parasect",
            "Unequal pulse attacks and moving noise develop across an extended call.",
        ),
        (
            "NOCTOWL",
            "Noctowl",
            "Interleaved pulse articulation; species length changes tonal tracks independently of noise.",
        ),
        (
            "RAICHU",
            "Raichu",
            "Widely separated pulse registers give the two voices different jobs.",
        ),
        (
            "SLUGMA",
            "Slugma",
            "A two-channel pulse-and-noise voice with abrupt register changes.",
        ),
        (
            "TOGEPI",
            "Togepi",
            "Sparse short phrases and a meaningful silence. The supplied import has a silent setup-only first pulse.",
        ),
    ];
    let mut reference_html = String::new();
    for (id, name, description) in references {
        if directory.join(format!("references/{id}.wav")).is_file() {
            reference_html.push_str(&format!(
                "<article><h3>{name}</h3><p>{description}</p><audio controls preload=\"none\" src=\"references/{id}.wav\"></audio></article>"
            ));
        }
    }
    if !reference_html.is_empty() {
        html.push_str(&format!(
            "<aside><h2>Original Crystal reference study</h2><p>Effective species renders from the supplied local pack, using Geothite's renderer. These illustrate techniques; they are not the new compositions.</p><main>{reference_html}</main></aside>"
        ));
    }
    let mut families = book.families.iter().collect::<Vec<_>>();
    families.sort_by_key(|(_, family)| {
        family
            .species
            .keys()
            .filter_map(|id| order.get(id))
            .min()
            .copied()
    });
    for (family_id, family) in families {
        let mut evolution_pcm = Vec::new();
        let mut species = family.species.iter().collect::<Vec<_>>();
        species.sort_by_key(|(id, _)| order.get(*id).copied());
        let names = species
            .iter()
            .map(|(_, variant)| variant.character.split(':').next().unwrap_or("Creature"))
            .collect::<Vec<_>>()
            .join(" / ");
        html.push_str(&format!(
            "<section id=\"{family_id}\"><h2>{}</h2><small>{family_id}</small><p>{}</p><main>",
            escape(&names),
            escape(&family.intent)
        ));
        for (id, variant) in species {
            ensure!(
                id.bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
                "unsafe species filename"
            );
            let rendered = synth::render(&programs[id], context.clone())?;
            ensure!(rendered.loop_samples.is_empty(), "cry {id} loops");
            let pcm = rendered.downsample();
            if !evolution_pcm.is_empty() {
                evolution_pcm.extend(std::iter::repeat_n(0, 8820));
            }
            evolution_pcm.extend_from_slice(&pcm);
            voices.push(similarity::Voice::measure(id, family_id, &pcm));
            let frames = pcm.len() / 2;
            ensure!(
                (3307..=99225).contains(&frames),
                "cry {id} duration outside 0.15..=4.5s"
            );
            let hash = pcm
                .iter()
                .flat_map(|sample| sample.to_le_bytes())
                .fold(2166136261u32, |hash, byte| {
                    (hash ^ u32::from(byte)).wrapping_mul(16777619)
                });
            ensure!(hashes.insert(hash), "duplicate PCM {id}");
            let peak = pcm
                .iter()
                .map(|sample| i32::from(*sample).abs())
                .max()
                .context("empty PCM")?;
            let tail = pcm[pcm.len() - 100..]
                .iter()
                .map(|sample| i32::from(*sample).abs())
                .max()
                .context("empty tail")?;
            let rms = (pcm
                .iter()
                .map(|sample| f64::from(*sample).powi(2))
                .sum::<f64>()
                / pcm.len() as f64)
                .sqrt();
            let onset = pcm
                .as_chunks::<2>()
                .0
                .iter()
                .position(|frame| frame.iter().any(|v| i32::from(*v).abs() >= 512))
                .context("cry never reaches audible attack threshold")?;
            let onset_ms = onset as f64 / 22.05;
            wav::write_canonical_wav(&directory.join(format!("{id}.wav")), &pcm)?;
            let name = variant.character.split(':').next().unwrap_or(id);
            println!(
                "{id}: {frames} frames, onset {onset_ms:.2}ms, peak {peak}, rms {rms:.0}, tail {tail}, hash {hash:08x}"
            );
            let comparison = if let Some(program) = before.as_ref().and_then(|book| book.get(id)) {
                let old = synth::render(program, context.clone())?.downsample();
                before_voices.push(similarity::Voice::measure(id, family_id, &old));
                wav::write_canonical_wav(
                    &directory.join("before").join(format!("{id}.wav")),
                    &old,
                )?;
                format!(
                    "<details><summary>Previous version · {:.3}s</summary><audio controls preload=\"none\" src=\"before/{id}.wav\"></audio></details>",
                    old.len() as f64 / 44100.0
                )
            } else {
                String::new()
            };
            html.push_str(&format!("<article><h3>{}</h3><p>{}</p><p><small>Current</small></p><audio controls preload=\"none\" src=\"{id}.wav\"></audio><p><small>{id} · {:.3}s · attack {onset_ms:.1}ms · {} voices · <a href=\"{id}.wav\" download>WAV</a></small></p>{comparison}</article>", escape(name), escape(&variant.character), frames as f64 / 22050.0, programs[id].music_data.channel_count));
        }
        wav::write_canonical_wav(
            &directory.join(format!("evolution-{family_id}.wav")),
            &evolution_pcm,
        )?;
        html.push_str(&format!(
            "</main><p>Revised evolution sequence, in roster order</p><audio controls preload=\"none\" src=\"evolution-{family_id}.wav\"></audio></section>"
        ));
    }
    html.push_str("<script>document.addEventListener('play',e=>{if(e.target.tagName==='AUDIO')document.querySelectorAll('audio').forEach(a=>{if(a!==e.target)a.pause()})},true)</script></html>");
    fs::write(directory.join("index.html"), html)?;
    if before.is_some() {
        similarity::report(&before_voices, "Before");
    }
    similarity::report(&voices, "Revised");
    println!(
        "rendered {} unique original cries; gallery {}",
        programs.len(),
        directory.join("index.html").display()
    );
    Ok(())
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
