//! Authenticate the complete outer north row before closing its missing joins.
//! Grating stays horizontal; the inferred wall stands at the map boundary.
use super::*;
use crate::live_profiles::Document;

pub(super) fn gaps(
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> Option<Vec<(i32, i32)>> {
    let size = frame.source_map_size_core_tiles?;
    if size.x != 20 || size.y != 18 || g.width.checked_mul(g.height) != Some(cells.len()) {
        return None;
    }
    let expected = match frame.map_id.as_ref() {
        "TinTower1F" => 0xa3aedfc13dac7965,
        "TinTower2F" => 0x5a6216bcbfa96345,
        "TinTower3F" => 0x7a5e14e9613ed465,
        "TinTower4F" => 0x50723cf8fcb1f0bd,
        "TinTower5F" => 0xcd055a3acd8931fc,
        "TinTower6F" => 0xb70fbba2e3d17cf5,
        "TinTower7F" => 0x8e11cb8ad43e473d,
        "TinTower8F" => 0xfdbd83d371598fbd,
        "TinTower9F" => 0x13884126c6716365,
        _ => return None,
    };
    let index = |x: i32, y: i32| {
        let col = x - frame.grid_origin.x;
        let row = y - frame.grid_origin.y;
        (col >= 0 && row >= 0 && (col as usize) < g.width && (row as usize) < g.height)
            .then(|| row as usize * g.width + col as usize)
    };
    let mut hash = 0xcbf29ce484222325u64;
    for y in 0..4 {
        for x in 0..40 {
            let s = &cells[index(x, y)?].source;
            if s.tileset_id.as_ref() != "tower"
                || s.subtile_column != x as u8 % 4
                || s.subtile_row != y as u8
            {
                return None;
            }
            for byte in [
                (s.metatile_id & 255) as u8,
                (s.metatile_id >> 8) as u8,
                s.subtile_column,
                s.subtile_row,
                (s.tile_index & 255) as u8,
                (s.tile_index >> 8) as u8,
            ] {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
    }
    if hash != expected {
        return None;
    }
    if let Some(profiles) = profiles {
        // Check both a custom drawing and the old footprint of an edited
        // canonical object. Disabling a native source owner must not cause
        // an inferred replacement wall to appear at that same source band.
        let canonical = built_in_profiles();
        let mut objects = Vec::new();
        for custom in &profiles.objects {
            if canonical.objects.contains(custom) {
                continue;
            }
            objects.push(custom.clone());
            if let Some(old) = canonical.objects.iter().find(|o| o.name == custom.name) {
                objects.push(old.clone());
            }
        }
        let custom = Document {
            objects,
            atmosphere: None,
        };
        if live::resolve(cells, g.width, g.height, &frame.map_id, Some(&custom))
            .iter()
            .any(|p| {
                p.indices(g.width).any(|i| {
                    let x = frame.grid_origin.x + (i % g.width) as i32;
                    let y = frame.grid_origin.y + (i / g.width) as i32;
                    (0..40).contains(&x) && (0..4).contains(&y)
                })
            })
        {
            return None;
        }
    }
    let mut gaps = Vec::new();
    let mut start = None;
    for x in 4..=36 {
        // These complete drawings already supply their folded 16px rear
        // course, including the backing behind guardians. Other first-row
        // fields are grating or floor and get an outer boundary at row zero.
        let covered = x == 36
            || matches!(
                cells[index(x, 0)?].source.metatile_id,
                4 | 5 | 6 | 0x21 | 0x22 | 0x3b
            );
        if covered {
            if let Some(begin) = start.take() {
                gaps.push((begin, x));
            }
        } else {
            start.get_or_insert(x);
        }
    }
    Some(gaps)
}
