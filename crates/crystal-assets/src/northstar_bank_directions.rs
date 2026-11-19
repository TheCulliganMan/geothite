//! Point the Fen observer's clues at the native saplings, pool corner and low eastern fir.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_dialogue};

/// Replaces directions on the existing Fen interactions without changing their programs.
///
/// Returns an error if a parent clue or its split script no longer matches the Fen Trail.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let name = "MnLayeredFen";
    let fen = pack.data.maps.get_mut(name).context(name)?;
    let mut revised = BTreeMap::new();
    for (label, op, line) in [
        ("MnFenText_observer_after", "para", "I'll take that new"),
        ("MnFenText_observer_intro", "para", "I was on the root"),
        ("MnFenText_observer_partial", "text", "Feel the worn root"),
        ("MnFenText_early", "text", "Look at the pool"),
        ("MnFenText_wrong_order", "text", "Look for the moss"),
        ("MnFenText_observer_compare", "text", "Press the root."),
        ("MnFenText_root_selected", "para", "in the little pool"),
        ("MnFenText_pool_selected", "para", "the east ridge"),
        ("MnFenText_rib", "line", "the low ridge."),
        ("MnFenText_rib_selected", "line", "the ridge."),
        ("MnFenText_folded_path", "line", "Beyond the ridge,"),
        ("MnFenText_layers_settle", "line", "ridge stirs last."),
        ("MnFenText_caught", "line", "of the stone. The"),
    ] {
        let old = fen.scripts.get(label).context(label)?;
        ensure!(
            old.as_array()
                .is_some_and(|body| body.contains(&command(op, &[&format!("\"{line}\"")])))
                && pack.data.map_scripts.get(label) == Some(old),
            "unexpected former Fen landmark directions {label}"
        );
        authored_dialogue::install_canonical(fen, label)?;
        revised.insert(label.into(), fen.scripts[label].clone());
    }
    fen.script_text_bodies
        .extend(crate::parse_script_text_bodies(name, &revised)?);
    pack.data.map_scripts.extend(revised);
    Ok(())
}
