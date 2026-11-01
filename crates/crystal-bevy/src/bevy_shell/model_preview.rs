//! Legal DV choices for disposable model previews, never loaded game sessions.
use anyhow::{Result, ensure};
use crystal_core::models::Dv;

pub(super) fn unown_form(letter: &str) -> Result<u8> {
    let bytes = letter.as_bytes();
    ensure!(bytes.len() == 1 && bytes[0].is_ascii_alphabetic(), "preview Unown form must be A..Z");
    Ok(bytes[0].to_ascii_uppercase() - b'A' + 1)
}

pub(super) fn dvs(species: &str, shiny: bool, form: Option<u8>) -> Result<Dv> {
    if let Some(form) = form {
        ensure!(species == "UNOWN", "an Unown form requires preview species UNOWN");
        ensure!((1..=26).contains(&form), "Unown form must be A..Z");
        if shiny {
            ensure!(matches!(form, 9 | 22), "Crystal shiny Unown DVs select I or V");
            Ok(Dv::from_non_hp(if form == 9 { 2 } else { 14 }, 10, 10, 10))
        } else {
            let packed = (form - 1) * 10;
            Ok(Dv::from_non_hp(((packed >> 6) & 3) << 1, ((packed >> 4) & 3) << 1,
                ((packed >> 2) & 3) << 1, (packed & 3) << 1))
        }
    } else {
        Ok(if shiny { Dv::from_non_hp(2, 10, 10, 10) } else { Dv::from_non_hp(9, 9, 9, 9) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_normal_unown_form_uses_legal_constructor_dvs() {
        for form in 1..=26 {
            let value = dvs("UNOWN", false, Some(form)).unwrap();
            assert_eq!(value.unown_letter(), form);
            assert_eq!(value, Dv::from_non_hp(value.attack, value.defense, value.speed, value.special));
            assert!([value.attack, value.defense, value.speed, value.special].into_iter().all(|v| v <= 15));
        }
        assert_eq!(dvs("SEEL", false, None).unwrap(), Dv::from_non_hp(9, 9, 9, 9));
    }

    #[test]
    fn invalid_forms_and_other_species_are_rejected() {
        for form in [0, 27, u8::MAX] {
            assert!(dvs("UNOWN", false, Some(form)).is_err());
        }
        assert!(dvs("SEEL", false, Some(25)).is_err());
    }

    #[test]
    fn letter_parser_accepts_one_ascii_letter() {
        for (letter, expected) in [("A", 1), ("y", 25), ("Z", 26)] {
            assert_eq!(unown_form(letter).unwrap(), expected);
        }
        for letter in ["", "AA", "!", "é", "["] {
            assert!(unown_form(letter).is_err());
        }
    }
}
