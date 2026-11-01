//! Runtime map landmarks must fit the original caught-location field.
use crate::{GameDataSet, VerificationError};

pub(crate) fn issues(data: &GameDataSet) -> Vec<VerificationError> {
    let mut issues = Vec::new();
    for (map, constant) in &data.pokegear_landmarks.map_to_landmark {
        let Some(landmark) = data
            .pokegear_landmarks
            .landmarks
            .iter()
            .find(|l| &l.constant == constant)
        else {
            continue;
        };
        if landmark.id > 0x7f {
            issues.push(VerificationError::error(
                "runtime_caught_landmark_out_of_range",
                map,
                format!(
                    "landmark {} uses caught location {}, exceeding Crystal's seven-bit field",
                    constant, landmark.id
                ),
            ));
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::issues;

    #[test]
    #[ignore = "requires the external current regional gym candidate"]
    fn runtime_landmark_verification_rejects_non_native_caught_locations() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let pack = crate::read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-eight-gyms.crystalpack"),
        )
        .expect("current regional candidate");
        assert!(issues(pack.data()).is_empty());
        let mut data = pack.data().clone();
        data.pokegear_landmarks
            .landmarks
            .iter_mut()
            .find(|l| l.constant == "LANDMARK_MN_TWIN_CITIES")
            .expect("St Paul regional area")
            .id = 128;
        let errors = issues(&data);
        assert!(
            errors
                .iter()
                .any(|e| e.code == "runtime_caught_landmark_out_of_range")
        );
        data.pokegear_landmarks
            .landmarks
            .iter_mut()
            .find(|l| l.constant == "LANDMARK_MN_TWIN_CITIES")
            .expect("St Paul regional area")
            .id = 127;
        assert!(
            issues(&data).is_empty(),
            "127 remains native caught-location data"
        );
    }
}
