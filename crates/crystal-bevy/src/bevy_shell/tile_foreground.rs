//! Authored foreground colors retain the current native ground color.
use std::collections::BTreeMap;

type Palette = [[u8; 3]; 4];

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

#[cfg(test)]
mod tests {
    use super::resolve;
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
}
