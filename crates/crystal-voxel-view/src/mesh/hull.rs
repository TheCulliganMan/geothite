//! Pixel-grid solid construction from a complete artwork silhouette.
//! Rows become circular voxel chords; only faces bordering empty space survive.

#[derive(Debug)]
pub(super) struct HullFace {
    pub corners: [[f32; 3]; 4],
    pub normal: [f32; 3],
    pub source: [usize; 2],
    pub cap_interior: bool,
}

pub(super) fn round_hull_faces(mask: &[bool], width: usize, height: usize) -> Vec<HullFace> {
    assert_eq!(mask.len(), width * height);
    let mut spans = vec![(0_i32, 0_i32); mask.len()];
    for y in 0..height {
        let row = &mask[y * width..(y + 1) * width];
        let Some(left) = row.iter().position(|&on| on) else {
            continue;
        };
        let right = row.iter().rposition(|&on| on).unwrap();
        let radius = (right + 1 - left) as f32 * 0.5;
        let center = (left + right + 1) as f32 * 0.5;
        for x in left..=right {
            if !row[x] {
                continue;
            }
            let offset = x as f32 + 0.5 - center;
            let length = (2.0 * (radius * radius - offset * offset).max(0.0).sqrt())
                .round()
                .max(1.0) as i32;
            let start = (width as f32 * 0.5 - length as f32 * 0.5).round() as i32;
            spans[y * width + x] = (start, start + length);
        }
    }
    let mut source_rows: Vec<usize> = (0..height).collect();
    if let Some(bottom) = (0..height).rfind(|&y| {
        spans[y * width..(y + 1) * width]
            .iter()
            .any(|&(a, b)| a < b)
    }) {
        // Source shadows are not geometry. Extend the last body section to
        // the foot plane so removing the painted shadow cannot levitate it.
        for y in bottom + 1..height {
            for x in 0..width {
                spans[y * width + x] = spans[bottom * width + x];
            }
            source_rows[y] = bottom;
        }
    }
    let occupied = |x: i32, y: i32, z: i32| {
        if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
            return false;
        }
        let (start, end) = spans[y as usize * width + x as usize];
        z >= start && z < end
    };
    // Integer positions use Y upward, while source rows run downward.
    let directions = [
        ([1, 0, 0], [[1, 0, 1], [1, 0, 0], [1, 1, 0], [1, 1, 1]]),
        ([-1, 0, 0], [[0, 0, 0], [0, 0, 1], [0, 1, 1], [0, 1, 0]]),
        ([0, -1, 0], [[0, 1, 1], [1, 1, 1], [1, 1, 0], [0, 1, 0]]),
        ([0, 1, 0], [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]]),
        ([0, 0, 1], [[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]]),
        ([0, 0, -1], [[1, 0, 0], [0, 0, 0], [0, 1, 0], [1, 1, 0]]),
    ];
    let mut faces = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let (start, end) = spans[y * width + x];
            for (step, corners) in directions {
                let mut z = start;
                while z < end {
                    if occupied(x as i32 + step[0], y as i32 + step[1], z + step[2]) {
                        z += 1;
                        continue;
                    }
                    let first = z;
                    z += 1;
                    // Side/top faces sample one artwork texel along depth.
                    // Merge its exposed run without crossing a hidden voxel.
                    if step[2] == 0 {
                        while z < end && !occupied(x as i32 + step[0], y as i32 + step[1], z) {
                            z += 1;
                        }
                    }
                    let segments = if step[1] == -1
                        && z - first >= 3
                        && (y == 0 || !mask[(y - 1) * width + x])
                    {
                        [
                            (first, first + 1, false),
                            (first + 1, z - 1, true),
                            (z - 1, z, false),
                        ]
                    } else {
                        [(first, z, false), (0, 0, false), (0, 0, false)]
                    };
                    for (from, to, cap_interior) in segments {
                        if from == to {
                            continue;
                        }
                        faces.push(HullFace {
                            corners: corners.map(|p| {
                                [
                                    x as f32 + p[0] as f32,
                                    (height - y - 1) as f32 + p[1] as f32,
                                    from as f32 + p[2] as f32 * (to - from) as f32,
                                ]
                            }),
                            normal: [step[0] as f32, -step[1] as f32, step[2] as f32],
                            source: [x, source_rows[y]],
                            cap_interior,
                        });
                    }
                }
            }
        }
    }
    faces
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hull_is_closed_quantized_and_has_internal_chord_steps() {
        let faces = round_hull_faces(&vec![true; 16], 4, 4);
        assert!(faces.iter().any(|f| f.normal == [1.0, 0.0, 0.0]
            && f.corners[0][0] > 0.0
            && f.corners[0][0] < 4.0));
        assert!(
            faces
                .iter()
                .flat_map(|f| f.corners.iter().flatten())
                .all(|v| v.fract() == 0.0)
        );
        for axis in 0..3 {
            assert_eq!(
                faces
                    .iter()
                    .map(|f| {
                        let a = bevy::prelude::Vec3::from(f.corners[1])
                            - bevy::prelude::Vec3::from(f.corners[0]);
                        let b = bevy::prelude::Vec3::from(f.corners[3])
                            - bevy::prelude::Vec3::from(f.corners[0]);
                        a.cross(b)[axis]
                    })
                    .sum::<f32>(),
                0.0
            );
        }
        for face in faces {
            let a = bevy::prelude::Vec3::from(face.corners[1])
                - bevy::prelude::Vec3::from(face.corners[0]);
            let b = bevy::prelude::Vec3::from(face.corners[2])
                - bevy::prelude::Vec3::from(face.corners[0]);
            assert!(a.cross(b).dot(face.normal.into()) > 0.0);
        }
    }
    #[test]
    fn body_reaches_ground_after_background_shadow_is_removed() {
        let mut mask = vec![false; 16];
        mask[5] = true;
        mask[6] = true;
        let faces = round_hull_faces(&mask, 4, 4);
        assert!(
            faces
                .iter()
                .any(|f| f.normal == [0.0, -1.0, 0.0] && f.corners.iter().all(|p| p[1] == 0.0))
        );
        assert!(faces.iter().all(|f| f.source[1] == 1));
    }
    #[test]
    fn empty_silhouette_emits_nothing() {
        assert!(round_hull_faces(&[false; 16], 4, 4).is_empty());
    }
}

/// A solid rock's bright cracks belong to its painted surface. Close each
/// outline span before giving it depth, including wholly bright interior rows.
pub(super) fn solid_rock_mask(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    assert_eq!(mask.len(), width * height);
    let spans: Vec<_> = mask
        .chunks_exact(width)
        .map(|row| {
            row.iter()
                .position(|&on| on)
                .zip(row.iter().rposition(|&on| on))
        })
        .collect();
    let mut solid = vec![false; mask.len()];
    for y in 0..height {
        let span = spans[y].or_else(|| {
            let north = (0..y).rfind(|&row| spans[row].is_some())?;
            let south = (y + 1..height).find(|&row| spans[row].is_some())?;
            let (left0, right0) = spans[north]?;
            let (left1, right1) = spans[south]?;
            let t = (y - north) as f32 / (south - north) as f32;
            Some((
                (left0 as f32 + (left1 as f32 - left0 as f32) * t).round() as usize,
                (right0 as f32 + (right1 as f32 - right0 as f32) * t).round() as usize,
            ))
        });
        if let Some((left, right)) = span {
            solid[y * width + left..=y * width + right].fill(true);
        }
    }
    solid
}

#[cfg(test)]
mod rock_tests {
    use super::*;
    #[test]
    fn bright_ice_cracks_are_paint_not_holes_through_the_boulder() {
        let mask = [
            false, true, false, true, false, true, false, false, false, true, false, false, false,
            false, false, false, true, false, true, false,
        ];
        let solid = solid_rock_mask(&mask, 5, 4);
        assert!(solid[2] && solid[7] && solid[12] && solid[17]);
        assert!(!solid[0] && !solid[4]);
    }
}
