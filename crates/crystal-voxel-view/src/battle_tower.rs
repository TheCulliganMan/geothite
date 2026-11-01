//! Presentation profiles for the Battle Tower Outside atlas.

use crystal_render_api::VisualTileSource;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TreeGroup {
    pub(crate) local_column: u8,
    pub(crate) local_row: u8,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) ground_tile_index: u16,
}

/// Resolve every complete tree drawing embedded in Battle Tower Outside's
/// metatiles, including compact and mixed-wall variants.
pub(crate) fn tree_group(source: &VisualTileSource) -> Option<TreeGroup> {
    if source.tileset_id.as_ref() != "battle_tower_outside" {
        return None;
    }
    let (local_column, local_row, height) = match source.metatile_id {
        0x05 => (source.subtile_column % 2, source.subtile_row, 4),
        0x0d if source.subtile_column >= 2 => (source.subtile_column - 2, source.subtile_row, 4),
        0x1c if source.subtile_row < 2 => (source.subtile_column % 2, source.subtile_row, 2),
        0x1d if source.subtile_row >= 2 => (source.subtile_column % 2, source.subtile_row - 2, 2),
        0x1f => (source.subtile_column % 2, source.subtile_row % 2, 2),
        0x2d | 0x2e | 0x30 if source.subtile_column < 2 => {
            (source.subtile_column, source.subtile_row, 4)
        }
        0x32 if source.subtile_column >= 2 => {
            (source.subtile_column - 2, source.subtile_row % 2, 2)
        }
        0x33 if source.subtile_column >= 2 => (source.subtile_column - 2, source.subtile_row, 4),
        _ => return None,
    };
    Some(TreeGroup {
        local_column,
        local_row,
        width: 2,
        height,
        ground_tile_index: 0x06,
    })
}

/// Source-proven rock cells share the existing outdoor mountain producer.
/// Mixed tree columns remain entirely outside the rock footprint.
pub(crate) fn cliff_source(source:&VisualTileSource)->bool {
    if source.tileset_id.as_ref()!="battle_tower_outside" || source.subtile_column>=4 || source.subtile_row>=4 {return false;}
    let x=source.subtile_column as usize;let y=source.subtile_row as usize;
    let art=match source.metatile_id {
        0x0a=>[0x2b,0x2c,0x2c,0x2d,0x3b,0x3c,0x3c,0x3d,0x3b,0x3c,0x3c,0x3d,0x4b,0x4c,0x4c,0x4d],
        0x19=>[0x2d,0x3c,0x3c,0x3c,0x3d,0x2d,0x3c,0x3c,0x3d,0x3d,0x3c,0x3c,0x3d,0x3d,0x3c,0x3c],
        0x1e=>[0x3d,0x3d,0x3c,0x3c,0x3d,0x3d,0x3c,0x3c,0x3d,0x3d,0x3c,0x3c,0x3d,0x3d,0x3c,0x3c],
        0x2a=>[0x2c,0x2c,0x2c,0x2c,0x3c,0x3c,0x3c,0x3c,0x3c,0x3c,0x3c,0x3c,0x3c,0x3c,0x3c,0x3c],
        0x2b=>[0x3c;16],
        0x32 if x<2=>[0x2d,0x3c,0,0,0x3d,0x2d,0,0,0x3d,0x3d,0,0,0x3d,0x3d,0,0],
        0x33 if x<2=>[0x3d,0x3d,0,0,0x3d,0x3d,0,0,0x3d,0x3d,0,0,0x3d,0x3d,0,0],
        _=>return false,
    };
    source.tile_index==art[y*4+x]
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn source(metatile_id: u16, column: u8, row: u8) -> VisualTileSource {
        VisualTileSource {
            tileset_id: Arc::from("battle_tower_outside"),
            metatile_id,
            subtile_column: column,
            subtile_row: row,
            tile_index: 0,
        }
    }

    #[test]
    fn battle_tower_mountain_requires_native_rock_and_excludes_mixed_tree_columns() {
        for block in [0x0a,0x19,0x1e,0x2a,0x2b,0x32,0x33] {for y in 0..4 {for x in 0..4 {
            let mut s=source(block,x,y);let allowed=x<2 || !matches!(block,0x32|0x33);
            let accepted=(0..256).filter(|tile|{s.tile_index=*tile;cliff_source(&s)}).collect::<Vec<_>>();
            assert_eq!(accepted.len(),usize::from(allowed));
            if let Some(&tile)=accepted.first() {
                s.tile_index=tile;
                assert_eq!(crate::profile::shape_for_source(&s),crate::profile::CellShape::RaisedTop {
                    height:crate::profile::MOUNTAIN_CLIFF_HEIGHT,solid:crate::profile::SolidKind::Bank});
                s.tileset_id="johto".into();assert!(!cliff_source(&s));
            }
        }}}
        let mut s=source(0x0c,0,0);s.tile_index=0x3c;assert!(!cliff_source(&s));
    }
    #[test]
    fn every_full_and_compact_tree_variant_has_an_exact_group() {
        for (metatile, column) in [
            (0x05, 0),
            (0x0d, 2),
            (0x2d, 0),
            (0x2e, 0),
            (0x30, 0),
            (0x33, 2),
        ] {
            assert_eq!(tree_group(&source(metatile, column, 0)).unwrap().height, 4);
        }
        for (metatile, column, row) in [(0x1c, 0, 0), (0x1d, 0, 2), (0x1f, 0, 0), (0x32, 2, 0)] {
            assert_eq!(
                tree_group(&source(metatile, column, row)).unwrap().height,
                2
            );
        }
    }

    #[test]
    fn non_tree_quadrants_stay_unclaimed() {
        assert!(tree_group(&source(0x0d, 0, 0)).is_none());
        assert!(tree_group(&source(0x1c, 0, 2)).is_none());
        assert!(tree_group(&source(0x2d, 2, 0)).is_none());
        assert!(tree_group(&source(0x33, 0, 0)).is_none());
    }
}
