//! Preserve small enclosed source marks during native field-art resampling.
use crate::minnesota_sprite_sheet::MinnesotaSheet;

use super::{OverworldFrameSource, OverworldSpeciesSource, PIXELS, SIZE, Transform};

pub(super) fn enclosed_black_coverage(
    sheet: &MinnesotaSheet,
    frame: &OverworldFrameSource,
    species: &OverworldSpeciesSource,
    transform: &Transform,
    point_sampled: &[u8],
) -> Vec<bool> {
    let [left, top, width, height] = transform.bounds;
    let mut dark = vec![false; width * height];
    let opaque = |x: usize, y: usize| {
        sheet
            .alpha(frame.crop[0] + left + x, frame.crop[1] + top + y)
            .is_some_and(|alpha| alpha >= species.alpha_threshold)
    };
    for y in 0..height {
        for x in 0..width {
            if opaque(x, y) {
                let rgb = sheet
                    .pixel(frame.crop[0] + left + x, frame.crop[1] + top + y)
                    .expect("validated source bounds");
                dark[y * width + x] = rgb.iter().all(|channel| *channel <= 48)
                    && rgb
                        .iter()
                        .max()
                        .expect("RGB")
                        .saturating_sub(*rgb.iter().min().expect("RGB"))
                        <= 20;
            }
        }
    }
    let mut protected = vec![false; PIXELS];
    let mut seen = vec![false; dark.len()];
    for start in 0..dark.len() {
        if !dark[start] || seen[start] {
            continue;
        }
        seen[start] = true;
        let mut component = vec![start];
        let mut cursor = 0;
        let mut enclosed = true;
        while cursor < component.len() {
            let index = component[cursor];
            cursor += 1;
            let x = index % width;
            let y = index / width;
            if x == 0 || y == 0 || x + 1 == width || y + 1 == height {
                enclosed = false;
            }
            for (nx, ny) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if nx >= width || ny >= height {
                    continue;
                }
                if !opaque(nx, ny) {
                    enclosed = false;
                }
                let neighbor = ny * width + nx;
                if dark[neighbor] && !seen[neighbor] {
                    seen[neighbor] = true;
                    component.push(neighbor);
                }
            }
        }
        let native_area = component.len() * transform.size[0] * transform.size[1];
        let source_area = width * height;
        // Ignore negligible speckles and large contour/body-shadow networks.
        if !enclosed || native_area * 10 < source_area || native_area > source_area * 8 {
            continue;
        }
        let mut coverage = [0_usize; PIXELS];
        for index in component {
            let x = transform.origin[0] + index % width * transform.size[0] / width;
            let y = transform.origin[1] + index / width * transform.size[1] / height;
            coverage[y * SIZE + x] += 1;
        }
        let mut candidates: Vec<usize> = (0..PIXELS).filter(|&index| coverage[index] > 0).collect();
        // An enclosed source pupil can straddle a native eye and its exterior
        // rim. Prefer its real footprint inside the colored eye over merging it
        // into an already-black rim, where the expression would disappear.
        candidates.sort_by_key(|&index| {
            (
                point_sampled[index] == 0 || point_sampled[index] == 3,
                std::cmp::Reverse(
                    [index - 1, index + 1, index - SIZE, index + SIZE]
                        .iter()
                        .filter(|&&neighbor| matches!(point_sampled[neighbor], 1 | 2))
                        .count(),
                ),
                std::cmp::Reverse(coverage[index]),
                index,
            )
        });
        for index in candidates
            .into_iter()
            .take(native_area.div_ceil(source_area))
        {
            protected[index] = true;
        }
    }
    protected
}
