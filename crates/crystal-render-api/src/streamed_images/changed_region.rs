/// Smallest whole-pixel rectangle containing every changed RGBA byte.
pub(super) fn changed_rect(before: &[u8], after: &[u8], width: usize) -> Option<[usize; 4]> {
    assert!(width > 0);
    assert_eq!(before.len(), after.len());
    assert_eq!(after.len() % (width * 4), 0);
    let mut left = width;
    let mut right = 0;
    let mut top = usize::MAX;
    let mut bottom = 0;
    for (y, (old, new)) in before
        .chunks_exact(width * 4)
        .zip(after.chunks_exact(width * 4))
        .enumerate()
    {
        if old == new {
            continue;
        }
        let first = old.iter().zip(new).position(|(a, b)| a != b).unwrap() / 4;
        let last = old.iter().zip(new).rposition(|(a, b)| a != b).unwrap() / 4;
        left = left.min(first);
        right = right.max(last + 1);
        top = top.min(y);
        bottom = y + 1;
    }
    (top != usize::MAX).then(|| [left, top, right - left, bottom - top])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_surface_needs_no_gpu_upload() {
        let pixels = vec![127; 928 * 912 * 4];
        assert_eq!(changed_rect(&pixels, &pixels, 928), None);
    }

    #[test]
    fn subrectangle_upload_reconstructs_every_pixel_including_alpha() {
        let before = vec![100; 17 * 13 * 4];
        for offsets in [
            vec![0],
            vec![before.len() - 1],
            vec![71, 290, 417],
            (0..before.len()).collect(),
        ] {
            let mut after = before.clone();
            for offset in offsets {
                after[offset] ^= 127;
            }
            let [x, y, w, h] = changed_rect(&before, &after, 17).unwrap();
            let mut reconstructed = before.clone();
            for row in y..y + h {
                let start = (row * 17 + x) * 4;
                reconstructed[start..start + w * 4].copy_from_slice(&after[start..start + w * 4]);
            }
            assert_eq!(reconstructed, after);
        }
    }

    #[test]
    fn animation_patch_does_not_upload_the_entire_fullscreen_halo() {
        let before = vec![0; 928 * 912 * 4];
        let mut after = before.clone();
        for y in 400..408 {
            for x in 320..328 {
                after[(y * 928 + x) * 4 + 3] = 255;
            }
        }
        assert_eq!(changed_rect(&before, &after, 928), Some([320, 400, 8, 8]));
    }
}
