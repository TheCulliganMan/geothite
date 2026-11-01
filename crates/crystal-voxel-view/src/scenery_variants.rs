//! Three original mesh recipes per scenery family, chosen by source tile.
//! Never seed from camera-relative positions, time, or a process RNG.
pub(crate) fn index([x, z]: [i32; 2], family: u32) -> usize {
    let mut h = (x as u32).wrapping_mul(0x9e3779b9)
        ^ (z as u32).wrapping_mul(0x85ebca6b)
        ^ family.wrapping_mul(0x27d4eb2d);
    h = (h ^ (h >> 16)).wrapping_mul(0x7feb352d);
    h = (h ^ (h >> 15)).wrapping_mul(0x846ca68b);
    ((h ^ (h >> 16)) % 3) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_tiles_choose_all_three_without_stripes_or_camera_state() {
        for family in 0..6 {
            let mut counts = [0; 3];
            let mut adjacent_changes = 0;
            for z in -32..32 {
                for x in -32..32 {
                    let coordinates = [x * 2, z * 2];
                    let selected = index(coordinates, family);
                    assert_eq!(selected, index(coordinates, family));
                    counts[selected] += 1;
                    adjacent_changes += usize::from(selected != index([x * 2 + 2, z * 2], family));
                    // Two cropped views must recover the same source seed.
                    let origin = [-17, 23];
                    let local = [coordinates[0] - origin[0], coordinates[1] - origin[1]];
                    assert_eq!(
                        selected,
                        index([local[0] + origin[0], local[1] + origin[1]], family)
                    );
                }
            }
            assert!(counts.iter().all(|&n| (1150..1600).contains(&n)));
            assert!(adjacent_changes > 2400);
        }
    }
}
