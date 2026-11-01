//! Explicit, guarded native-pixel authoring after source conversion.

use std::collections::BTreeSet;

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

/// One authored change in the four-ink native grid.
/// `from` guards the source conversion so a later crop cannot silently move edits.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PixelEdit {
    pub x: usize,
    pub y: usize,
    pub from: u8,
    pub to: u8,
}

/// Replay native authoring edits, preserving the existing four-ink alphabet.
///
/// Returns an error for duplicate/out-of-bounds coordinates, invalid inks,
/// malformed touched rows, or a changed source pixel. Validation precedes all mutations.
pub fn apply(rows: &mut [String], edits: &[PixelEdit]) -> Result<()> {
    const INKS: &[u8; 4] = b".123";
    let mut coordinates = BTreeSet::new();
    for edit in edits {
        ensure!(edit.from < 4 && edit.to < 4, "native edit ink must be 0..3");
        ensure!(
            coordinates.insert((edit.x, edit.y)),
            "duplicate native pixel edit"
        );
        let pixel = rows.get(edit.y).and_then(|row| row.as_bytes().get(edit.x));
        ensure!(pixel.is_some(), "native edit outside sprite");
        ensure!(
            rows[edit.y].is_ascii() && rows[edit.y].bytes().all(|ink| INKS.contains(&ink)),
            "invalid native row ink"
        );
        ensure!(
            pixel == Some(&INKS[usize::from(edit.from)]),
            "stale native edit at ({}, {})",
            edit.x,
            edit.y
        );
    }
    for edit in edits {
        let ink = [".", "1", "2", "3"][usize::from(edit.to)];
        rows[edit.y].replace_range(edit.x..edit.x + 1, ink);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PixelEdit, apply};

    #[test]
    fn native_edits_are_guarded_atomic_and_limited_to_four_inks() {
        let original = vec![".123".to_string()];
        let edit = PixelEdit {
            x: 2,
            y: 0,
            from: 2,
            to: 0,
        };
        let mut rows = original.clone();
        apply(&mut rows, std::slice::from_ref(&edit)).expect("authored edit");
        assert_eq!(rows, [".1.3"]);
        for invalid in [
            PixelEdit { from: 1, ..edit },
            PixelEdit { x: 4, ..edit },
            PixelEdit { y: 1, ..edit },
            PixelEdit { to: 4, ..edit },
        ] {
            let mut rows = original.clone();
            let first = PixelEdit {
                x: 1,
                y: 0,
                from: 1,
                to: 2,
            };
            assert!(apply(&mut rows, &[first, invalid]).is_err());
            assert_eq!(rows, original, "failed edits leave all rows unchanged");
        }
        assert!(apply(&mut original.clone(), &[edit, edit]).is_err());
    }
}
