//! Small authored landscape corrections applied to verified external packs.

use crystal_assets::GameDataSet;

pub(super) fn apply(data: &mut GameDataSet) {
    for name in ["CherrygroveCity", "Route30"] {
        if let Some(map) = data.maps.get_mut(name) {
            replace_center_lake(
                name,
                &map.attributes.tileset_name,
                map.attributes.width,
                map.attributes.height,
                &mut map.blocks,
            );
        }
    }
}

fn replace_center_lake(map: &str, tileset: &str, width: u16, height: u16, blocks: &mut [u16]) {
    if tileset != "johto" || blocks.len() != usize::from(width) * usize::from(height) {
        return;
    }
    // The lake is split across the city's northeast edge and the last two
    // blocks of Route30's east edge, visible through its south connection.
    // Match the original pair as a whole so independently modded packs survive.
    let original: &[(usize, u16)] = match (map, width, height) {
        ("CherrygroveCity", 20, 9) => &[(17, 0x58), (18, 0x59)],
        ("Route30", 10, 27) => &[(259, 0x54), (269, 0x58)],
        _ => return,
    };
    if original
        .iter()
        .all(|&(index, block)| blocks[index] == block)
    {
        for &(index, _) in original {
            blocks[index] = 0x05;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_lake_mod_replaces_both_connected_parts_and_preserves_other_blocks() {
        for (name, width, height, lake) in [
            ("CherrygroveCity", 20, 9, [(17, 0x58), (18, 0x59)]),
            ("Route30", 10, 27, [(259, 0x54), (269, 0x58)]),
        ] {
            let mut blocks = vec![0x35; usize::from(width) * usize::from(height)];
            for (index, block) in lake {
                blocks[index] = block;
            }
            let before = blocks.clone();
            replace_center_lake(name, "johto", width, height, &mut blocks);
            for (index, &block) in blocks.iter().enumerate() {
                assert_eq!(
                    block,
                    if lake.iter().any(|&(i, _)| i == index) {
                        0x05
                    } else {
                        before[index]
                    }
                );
            }
            let replaced = blocks.clone();
            replace_center_lake(name, "johto", width, height, &mut blocks);
            assert_eq!(blocks, replaced, "loading again must be idempotent");

            let mut modified = before.clone();
            modified[lake[0].0] = 0x01;
            let retained = modified.clone();
            replace_center_lake(name, "johto", width, height, &mut modified);
            assert_eq!(
                modified, retained,
                "preserve independently authored landscape changes"
            );
            let mut unrelated = before.clone();
            replace_center_lake("Route29", "johto", width, height, &mut unrelated);
            assert_eq!(unrelated, before);
            replace_center_lake(name, "kanto", width, height, &mut unrelated);
            assert_eq!(unrelated, before);
        }
    }
}
