//! Original outdoor frames; the inscription is supplied by the live scene atlas.
use crate::interior_models::Model;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Modern,
    Kanto,
    Park,
    Forest,
}
impl Kind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Modern => "outdoor-sign/modern-frame",
            Self::Kanto => "outdoor-sign/kanto-board",
            Self::Park => "outdoor-sign/park-frame",
            Self::Forest => "outdoor-sign/forest-timber",
        }
    }
    pub(crate) fn height(self) -> f32 {
        match self {
            Self::Kanto => 1.80,
            Self::Modern | Self::Park | Self::Forest => 1.95,
        }
    }
}
pub(crate) struct SignModel {
    pub(crate) mesh: Model,
    /// Normalized left, right, bottom, top and front depth in the actual model bounds.
    pub(crate) face: [f32; 5],
}
impl SignModel {
    fn parse<'a>(data: impl Into<crate::model_storage::Source<'a>>) -> Result<Self, String> {
        let data = data.into();
        #[derive(Deserialize)]
        struct Face {
            live_face: [f32; 5],
        }
        let face: Face = crate::model_storage::parse(data)?;
        if face
            .live_face
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || face.live_face[0] >= face.live_face[1]
            || face.live_face[2] >= face.live_face[3]
        {
            return Err("invalid outdoor sign lettering inset".into());
        }
        Ok(Self {
            mesh: Model::parse(data)?,
            face: face.live_face,
        })
    }
}
pub(crate) fn model(kind: Kind) -> &'static SignModel {
    macro_rules! load {
        ($slot:ident, $path:literal) => {{
            static $slot: OnceLock<SignModel> = OnceLock::new();
            $slot.get_or_init(|| {
                SignModel::parse(crate::model_storage::include_model!($path))
                    .expect("validated outdoor sign")
            })
        }};
    }
    match kind {
        Kind::Modern => load!(MODERN, "models/outdoor_signs/modern_frame.mesh.json"),
        Kind::Kanto => load!(KANTO, "models/outdoor_signs/kanto_board.mesh.json"),
        Kind::Park => load!(PARK, "models/outdoor_signs/park_frame.mesh.json"),
        Kind::Forest => load!(FOREST, "models/outdoor_signs/forest_timber.mesh.json"),
    }
}
