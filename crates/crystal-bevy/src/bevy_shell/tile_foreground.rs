//! Authored foreground colors retain the current native ground color.
use std::collections::BTreeMap;

type Palette = [[u8; 3]; 4];

pub(super) struct Overrides<'a> {
    pub foregrounds: &'a BTreeMap<usize, [[u8; 3]; 3]>,
    pub backgrounds: &'a BTreeMap<usize, [u8; 3]>,
}

pub(super) fn resolve(
    tile: usize,
    palette: usize,
    bank: Option<&[Palette]>,
    foregrounds: &BTreeMap<usize, [[u8; 3]; 3]>,
) -> Option<Palette> {
    if let Some(colors) = foregrounds.get(&tile) {
        let ground = bank
            .and_then(|values| values.first())
            .map(|colors| colors[0])
            .unwrap_or([255; 3]);
        Some([ground, colors[0], colors[1], colors[2]])
    } else {
        bank.and_then(|values| values.get(palette).or_else(|| values.first()))
            .copied()
    }
}

/// A mixed seasonal sheet can retain the native ground behind its original
/// glyphs while newly authored snow glyphs use the private snow background.
pub(super) fn resolve_with_background(
    tile: usize,
    palette: usize,
    bank: Option<&[Palette]>,
    foregrounds: &BTreeMap<usize, [[u8; 3]; 3]>,
    backgrounds: &BTreeMap<usize, [u8; 3]>,
) -> Option<Palette> {
    let mut colors = resolve(tile, palette, bank, foregrounds)?;
    if let Some(background) = backgrounds.get(&tile) {
        colors[0] = *background;
    }
    Some(colors)
}

#[cfg(test)]
mod tests {
    use super::{resolve, resolve_with_background};
    use std::collections::BTreeMap;

    #[test]
    fn authored_fire_keeps_warm_ink_and_uses_the_night_floor() {
        let ground = [123, 115, 198];
        let mut bank = [[[0; 3]; 4]; 8];
        bank[0][0] = ground;
        bank[4] = [[247, 247, 90], [132, 115, 148], [132, 115, 82], [0; 3]];
        let colors = [[255, 255, 57], [255, 132, 8], [57; 3]];
        let foregrounds = BTreeMap::from([(32, colors)]);
        assert_eq!(
            resolve(32, 4, Some(&bank), &foregrounds),
            Some([ground, colors[0], colors[1], colors[2]])
        );
        // Original yellow artwork retains its full native night palette.
        assert_eq!(resolve(3, 4, Some(&bank), &foregrounds), Some(bank[4]));
    }

    #[test]
    fn moving_between_day_and_night_changes_only_the_authored_background() {
        let mut day = [[[0; 3]; 4]; 8];
        day[0][0] = [222, 255, 222];
        let mut night = day;
        night[0][0] = [123, 115, 198];
        let colors = [[255, 255, 57], [255, 132, 8], [57; 3]];
        let foregrounds = BTreeMap::from([(32, colors)]);
        let day = resolve(32, 4, Some(&day), &foregrounds).expect("authored day palette");
        let night = resolve(32, 4, Some(&night), &foregrounds).expect("authored night palette");
        assert_ne!(day[0], night[0]);
        assert_eq!(day[1..], night[1..]);
    }

    #[test]
    fn mixed_snow_and_green_overrides_only_zero_without_changing_foreground_inks() {
        let snow = [239, 255, 255];
        let mut bank = [[[0; 3]; 4]; 8];
        bank[0][0] = snow;
        bank[2] = [snow, [181, 255, 82], [99, 206, 8], [41, 115, 0]];
        let inks = [[198, 231, 231], [132, 181, 189], [82, 132, 140]];
        let foregrounds = BTreeMap::from([(224, inks)]);
        for background in [[222, 255, 222], [123, 115, 198]] {
            let addresses = BTreeMap::from([(224, background)]);
            assert_eq!(
                resolve_with_background(224, 3, Some(&bank), &foregrounds, &addresses),
                Some([background, inks[0], inks[1], inks[2]])
            );
            assert_eq!(
                resolve_with_background(2, 2, Some(&bank), &foregrounds, &addresses),
                Some(bank[2])
            );
        }
        assert_eq!(
            resolve_with_background(224, 3, Some(&bank), &foregrounds, &BTreeMap::new()),
            resolve(224, 3, Some(&bank), &foregrounds)
        );
        assert_eq!(
            resolve_with_background(2, 2, None, &BTreeMap::new(), &BTreeMap::from([(2, snow)])),
            None
        );
    }
}
