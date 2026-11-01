use anyhow::{Context, Result, ensure};

pub(super) fn restore_blocks(
    bytes: &mut [u8],
    width: usize,
    edits: &[(usize, usize, u8)],
) -> Result<()> {
    for &(x, y, expected) in edits {
        let block = bytes.get(y * width + x).context("override outside map")?;
        ensure!(
            *block == expected || *block == 0x05,
            "unexpected block at ({x},{y}): {block:#04x}; expected {expected:#04x} or tree 0x05"
        );
    }
    for &(x, y, _) in edits {
        bytes[y * width + x] = 0x05;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overrides_only_documented_cells_and_is_idempotent() {
        for (width, height, edits) in [
            (20, 9, &[(17, 0, 0x58), (18, 0, 0x59)][..]),
            (10, 27, &[(9, 25, 0x54), (9, 26, 0x58)][..]),
        ] {
            let mut bytes = vec![0x42; width * height];
            for &(x, y, block) in edits {
                bytes[y * width + x] = block;
            }
            let before = bytes.clone();
            restore_blocks(&mut bytes, width, edits).unwrap();
            for (i, (&old, &new)) in before.iter().zip(&bytes).enumerate() {
                assert_eq!(
                    new,
                    if edits.iter().any(|&(x, y, _)| y * width + x == i) {
                        0x05
                    } else {
                        old
                    }
                );
            }
            let after = bytes.clone();
            restore_blocks(&mut bytes, width, edits).unwrap();
            assert_eq!(bytes, after);
        }
    }

    #[test]
    fn unexpected_source_block_rejects_without_partial_edits() {
        let mut bytes = vec![0; 180];
        bytes[17] = 0x58;
        let before = bytes.clone();
        assert!(restore_blocks(&mut bytes, 20, &[(17, 0, 0x58), (18, 0, 0x59)]).is_err());
        assert_eq!(bytes, before);
    }
}
