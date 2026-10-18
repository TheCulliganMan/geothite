//! Closed native ponds inside a mineral plateau share its base, not sea level.
//! Resolve from complete source evidence before any camera crop or worker split.
use super::*;
use bevy::prelude::IVec2;
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::mesh) fn plateau(source: &VisualTileSource) -> bool {
    source.tileset_id.as_ref() == "kanto"
        && crate::kanto_cliff::kanto_cliff_shape(source).is_some_and(|s| {
            s.surface_height(SOURCE_TILE_HEIGHT) >= crate::kanto_cliff::KANTO_CLIFF_HEIGHT
        })
}

pub(super) fn raise(
    native: &BTreeMap<(i32, i32), (&VisualTileSource, bool)>,
    origin: IVec2,
    width: usize,
    levels: &mut [[f32; 4]],
) -> bool {
    let water = |s: &VisualTileSource| {
        s.tileset_id.as_ref() == "kanto"
            && s.tile_index == 0x14
            && shape_for_source(s) == CellShape::Water
    };
    let bank = |s: &VisualTileSource| super::kanto_shore::source_bank(s);
    let mut remaining = native
        .iter()
        .filter_map(|(&p, &(s, _))| (water(s) || bank(s)).then_some(p))
        .collect::<BTreeSet<_>>();
    let index = |(y, x): (i32, i32)| (y - origin.y) as usize * width + (x - origin.x) as usize;
    let rise = crate::profile::JUMP_LEDGE_HEIGHT;
    let mut changed = false;
    while let Some(start) = remaining.pop_first() {
        let mut body = vec![start];
        let mut cursor = 0;
        let mut closed = true;
        let mut has_water = false;
        let mut has_bank = false;
        let mut rim = f32::INFINITY;
        while cursor < body.len() {
            let (y, x) = body[cursor];
            cursor += 1;
            let s = native[&(y, x)].0;
            has_water |= water(s);
            has_bank |= bank(s);
            for p in [(y, x - 1), (y, x + 1), (y - 1, x), (y + 1, x)] {
                let Some(&(other, _)) = native.get(&p) else {
                    closed = false;
                    continue;
                };
                if water(other) || bank(other) {
                    if remaining.remove(&p) {
                        body.push(p);
                    }
                } else if other.tileset_id.as_ref() == "kanto"
                    && plateau(other)
                {
                    // Relief fades at the pocket boundary. Use the native
                    // mineral base plus its calibrated terrace, never a peak.
                    let terrace = levels[index(p)].into_iter().fold(f32::INFINITY, f32::min);
                    rim = rim.min(
                        shape_for_source(other).surface_height(SOURCE_TILE_HEIGHT) / rise + terrace,
                    );
                } else {
                    closed = false;
                }
            }
        }
        if closed && has_water && has_bank && rim.is_finite() && rim > 0. {
            for p in body {
                levels[index(p)] = [rim; 4];
            }
            changed = true;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<(IVec2, VisualTileSource, bool)> {
        (0..144)
            .map(|i| {
                let (x, y) = (i % 12, i / 12);
                let wet = (4..8).contains(&x) && (4..8).contains(&y);
                let bank = (3..9).contains(&x) && (3..9).contains(&y) && !wet;
                let (block, col, row, tile) = if wet {
                    (0x43, x % 4, y % 4, 0x14)
                } else if bank {
                    if y == 3 {
                        (0x1f, x % 4, 0, 0x33)
                    } else if x == 8 {
                        (0x64, 3, y % 4, 0x54)
                    } else {
                        (0x65, 0, y % 4, 0x33)
                    }
                } else {
                    (0x2c, x % 4, y % 4, 0x11)
                };
                (
                    IVec2::new(x - 6, y - 6),
                    VisualTileSource {
                        tileset_id: "kanto".into(),
                        metatile_id: block,
                        subtile_column: col as u8,
                        subtile_row: row as u8,
                        tile_index: tile,
                    },
                    false,
                )
            })
            .collect()
    }
    #[test]
    fn complete_mineral_pocket_has_one_level_and_open_or_altered_evidence_cannot_lift_it() {
        let get = |sources: &[(IVec2, VisualTileSource, bool)]| {
            let native = sources
                .iter()
                .map(|(p, s, b)| ((p.y, p.x), (s, *b)))
                .collect();
            let mut levels = vec![[2.; 4]; 144];
            let changed = raise(&native, IVec2::splat(-6), 12, &mut levels);
            (changed, levels)
        };
        let sources = fixture();
        let (changed, levels) = get(&sources);
        assert!(changed);
        let expected =
            2. + crate::kanto_cliff::KANTO_CLIFF_HEIGHT / crate::profile::JUMP_LEDGE_HEIGHT;
        for (i, level) in levels.iter().enumerate() {
            let inside = (3..9).contains(&(i % 12)) && (3..9).contains(&(i / 12));
            assert_eq!(
                *level,
                [if inside { expected } else { 2. }; 4],
                "cell {},{}",
                i % 12,
                i / 12
            );
        }
        for fault in 0..4 {
            let mut bad = sources.clone();
            match fault {
                0 => {
                    bad.remove(2 * 12 + 5);
                } // unknown rim
                1 => {
                    bad[2 * 12 + 5].1.metatile_id = 1;
                    bad[2 * 12 + 5].1.tile_index = 0x2c;
                } // open ground
                2 => {
                    bad[3 * 12 + 5].1.tile_index = 0x39;
                } // altered bank artwork
                _ => {
                    for (_, s, _) in &mut bad {
                        s.tileset_id = "johto".into();
                    }
                }
            }
            let (changed, levels) = get(&bad);
            assert!(!changed, "fault {fault}");
            assert!(levels.iter().all(|l| *l == [2.; 4]));
        }
    }
    #[test]
    fn native_east_cliff_courses_do_not_receive_a_second_plateau_lift() {
        let mut sources=fixture();
        for (_,source,_) in &mut sources {
            if source.metatile_id==0x2c {
                source.metatile_id=0x29;
                source.tile_index=if source.subtile_column<2 {0x11}else{0x24};
            }
        }
        let native=sources.iter().map(|(p,s,land)|((p.y,p.x),(s,*land))).collect();
        let mut levels=vec![[2.;4];144];
        assert!(raise(&native,IVec2::splat(-6),12,&mut levels));
        let rim=2.+crate::kanto_cliff::KANTO_CLIFF_HEIGHT/crate::profile::JUMP_LEDGE_HEIGHT;
        for (i,level) in levels.iter().enumerate() {
            let pocket=(3..9).contains(&(i%12)) && (3..9).contains(&(i/12));
            assert_eq!(*level,[if pocket {rim}else{2.};4],
                "only the pond meets the native cliff; its rock courses already own their height at {i}");
        }
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_volcanic_pond_is_level_with_its_native_plateau_and_ocean_stays_zero() {
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::env::var("CRYSTAL_RENDER_TEST_PACK").unwrap(),
        )
        .unwrap();
        let pack = loaded.pack();
        let module = &pack.data().maps["CinnabarIsland"];
        let atlas = &module.attributes.tileset_name;
        let layout = &pack.runtime_files()[&format!("data/tilesets/{atlas}_metatiles.bin")];
        let (w, h) = (
            module.attributes.width as usize * 4,
            module.attributes.height as usize * 4,
        );
        let sources = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let block = module.blocks[y / 4 * (w / 4) + x / 4];
                (
                    IVec2::new(x as i32, y as i32),
                    VisualTileSource {
                        tileset_id: atlas.as_str().into(),
                        metatile_id: block,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: layout[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                    false,
                )
            })
            .collect::<Vec<_>>();
        let datum = super::super::source_terrace_datum("CinnabarIsland", &[], &sources).unwrap();
        let expected = crate::kanto_cliff::KANTO_CLIFF_HEIGHT / crate::profile::JUMP_LEDGE_HEIGHT;
        let mut pond = 0;
        let mut ocean = 0;
        for (i, (p, s, _)) in sources
            .iter()
            .enumerate()
            .filter(|(_, (_, s, _))| s.tile_index == 0x14)
        {
            if (29..35).contains(&p.x) && (5..12).contains(&p.y) {
                assert_eq!(datum.levels[i], [expected; 4]);
                pond += 1;
            } else {
                assert_eq!(datum.levels[i], [0.; 4]);
                ocean += 1;
            }
            assert_eq!(s.tileset_id.as_ref(), "kanto");
        }
        assert_eq!(pond, 42);
        assert!(ocean > 100);
    }
}
