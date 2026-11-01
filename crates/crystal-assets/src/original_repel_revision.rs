//! Repair the original Repel's exported duration from exact native source labels.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

/// Reads the first literal duration instruction under one exact Repel label.
///
/// Only `RepelEffect`, `SuperRepelEffect` and `MaxRepelEffect` are supported.
/// Assembly comments and surrounding whitespace are ignored. The label must
/// occur exactly once as a whole declaration line, so a suffix of another
/// effect cannot select its routine. Decimal and `$` hexadecimal literals are
/// read from the first nonempty `ld b, ...` instruction; no duration is inferred
/// from an item name or a subsequent routine.
///
/// # Errors
/// Returns an error for an unsupported, missing or duplicate label, an empty
/// routine, a nonliteral first instruction or a value outside `u16`.
pub fn parse_source_repel_duration(source_text: &str, source_effect: &str) -> Result<u16> {
    ensure!(
        matches!(
            source_effect,
            "RepelEffect" | "SuperRepelEffect" | "MaxRepelEffect"
        ),
        "unsupported source repel effect {source_effect}"
    );
    let label = format!("{source_effect}:");
    let mut lines = source_text
        .lines()
        .map(|line| line.split(';').next().unwrap_or("").trim());
    ensure!(
        lines.by_ref().any(|line| line == label),
        "missing exact source repel routine {source_effect}"
    );
    let instruction = lines
        .by_ref()
        .find(|line| !line.is_empty())
        .context("empty repel routine")?;
    ensure!(
        !lines.any(|line| line == label),
        "duplicate source repel routine {source_effect}"
    );
    let literal = instruction
        .strip_prefix("ld b, ")
        .context("repel duration is not a source literal")?
        .trim();
    if let Some(hex) = literal.strip_prefix('$') {
        u16::from_str_radix(hex, 16).context("invalid hexadecimal repel duration")
    } else {
        literal.parse().context("invalid decimal repel duration")
    }
}

/// Repairs only `REPEL.repel_steps` from the known exported 200 to source 100.
///
/// Requires exact source durations 100/200/250, the three corresponding native
/// effect identities, and the known native duration preimage 200/200/250. Already
/// repaired packs are rejected. Every other item field, map/program/catalog,
/// compiled audio, audio/storage manifest, runtime file and compile report is
/// retained. Only that one typed field and the derived content identity change.
/// The complete base and result undergo strict runtime pack verification.
///
/// # Errors
/// Returns an error for source-label/literal drift, missing or mismatched native
/// items/effects/durations, an already repaired preimage, or failed base/result
/// pack verification. Borrowed inputs remain unchanged on failure.
pub fn repair_original_repel_duration(
    base: &CompiledGamePack,
    source_text: &str,
) -> Result<CompiledGamePack> {
    for (id, effect, original, exported) in [
        ("REPEL", "RepelEffect", 100, 200),
        ("SUPER_REPEL", "SuperRepelEffect", 200, 200),
        ("MAX_REPEL", "MaxRepelEffect", 250, 250),
    ] {
        ensure!(
            parse_source_repel_duration(source_text, effect)? == original,
            "original source {id} duration differs from {original}"
        );
        let item = base
            .data
            .items
            .get(id)
            .with_context(|| format!("native {id} is missing"))?;
        ensure!(
            item.effect == effect && item.repel_steps == Some(exported),
            "native {id} effect/duration preimage differs"
        );
    }
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let mut repaired = base.clone();
    repaired
        .data
        .items
        .get_mut("REPEL")
        .context("certified native REPEL")?
        .repel_steps = Some(100);
    repaired.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        repaired.format_version,
        &repaired.data,
        &repaired.audio_manifest,
        &repaired.runtime_files,
        &repaired.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&repaired)?;
    Ok(repaired)
}
