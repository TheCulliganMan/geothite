// The runtime Pokédex stores the original feet/inches display digits. Keep the
// conversion at the presentation boundary rather than duplicating a species
// catalog or allowing authored mesh normalization to become a physical size.
fn pokedex_dimension_meters(height_digits: u16) -> Option<f32> {
    let feet = height_digits / 100;
    let inches = height_digits % 100;
    if height_digits == 0 || inches >= 12 {
        return None;
    }
    Some(f32::from(feet * 12 + inches) * 0.0254)
}

#[cfg(test)]
mod battle_measurement_tests {
    use super::*;

    #[test]
    fn original_feet_inches_convert_without_decimal_feet_or_height_normalization() {
        for (digits, expected) in [
            (8, 0.2032),
            (108, 0.508),
            (200, 0.6096),
            (311, 1.1938),
            (411, 1.4986),
            (2810, 8.7884),
        ] {
            assert!((pokedex_dimension_meters(digits).unwrap() - expected).abs() < 0.00001);
        }
        let ratio = pokedex_dimension_meters(311).unwrap() / pokedex_dimension_meters(108).unwrap();
        assert!((ratio - 2.35).abs() < 0.00001);
    }

    #[test]
    fn absent_or_invalid_inches_have_no_invented_physical_size() {
        for digits in [0, 12, 99, 112, 399, u16::MAX] {
            assert_eq!(pokedex_dimension_meters(digits), None);
        }
        for digits in [1, 11, 100, 111, 65511] {
            assert!(pokedex_dimension_meters(digits).unwrap() > 0.0);
        }
    }
}
