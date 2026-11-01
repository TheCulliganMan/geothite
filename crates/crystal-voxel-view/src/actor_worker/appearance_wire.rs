//! Private representation bridge: no GPU handles or pose state enter the wire.
use super::PreparedAppearance;
use crate::terrain_worker::{blobs::BlobStore, mesh_buffers::MeshWire};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct AppearanceWire {
    mesh: MeshWire,
    #[serde(default)]
    arm_deformation: Option<crate::arm_deformation::Recipe>,
    bounds: [[u32; 3]; 2],
    head: Option<MeshWire>,
    costume: Option<MeshWire>,
    backpack: Option<MeshWire>,
}
impl AppearanceWire {
    pub(crate) fn capture(appearance: &PreparedAppearance) -> Result<Self, String> {
        Ok(Self {
            mesh: MeshWire::capture(&appearance.mesh)?,
            arm_deformation: appearance.arm_deformation.as_deref().cloned(),
            bounds: [
                appearance.bounds.0.to_array().map(f32::to_bits),
                appearance.bounds.1.to_array().map(f32::to_bits),
            ],
            head: appearance
                .head
                .as_ref()
                .map(MeshWire::capture)
                .transpose()?,
            costume: appearance
                .costume
                .as_ref()
                .map(MeshWire::capture)
                .transpose()?,
            backpack: appearance
                .backpack
                .as_ref()
                .map(MeshWire::capture)
                .transpose()?,
        })
    }
    pub(crate) fn resolve(&mut self, blobs: &mut BlobStore) -> Result<bool, String> {
        let mut ready = self.mesh.resolve(blobs)?;
        for mesh in [&mut self.head, &mut self.costume, &mut self.backpack]
            .into_iter()
            .flatten()
        {
            ready &= mesh.resolve(blobs)?;
        }
        Ok(ready)
    }
    pub(crate) fn restore(self) -> Result<PreparedAppearance, String> {
        let min = Vec3::from_array(self.bounds[0].map(f32::from_bits));
        let max = Vec3::from_array(self.bounds[1].map(f32::from_bits));
        if !min.is_finite() || !max.is_finite() || !min.cmple(max).all() {
            return Err("invalid actor appearance bounds".into());
        }
        let mesh = self.mesh.into_mesh()?;
        if let Some(recipe) = &self.arm_deformation {
            recipe.validate_mesh(&mesh)?;
        }
        Ok(PreparedAppearance {
            mesh,
            arm_deformation: self.arm_deformation.map(std::sync::Arc::new),
            bounds: (min, max),
            head: self.head.map(MeshWire::into_mesh).transpose()?,
            costume: self.costume.map(MeshWire::into_mesh).transpose()?,
            backpack: self.backpack.map(MeshWire::into_mesh).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain_worker::blobs;
    fn bytes(wire: &AppearanceWire) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::into_writer(wire, &mut bytes).unwrap();
        bytes
    }
    #[test]
    #[ignore = "requires supplied character rigs"]
    fn five_real_appearances_preserve_all_mesh_and_skin_bits_through_pages() {
        use crate::new_bark_actors::character_meshes::CharacterKind;
        for kind in [
            CharacterKind::Trainer,
            CharacterKind::TrainerFemale,
            CharacterKind::Nurse,
            CharacterKind::Kurt,
            CharacterKind::KimonoGirl,
        ] {
            assert_page_round_trip(kind);
        }
    }

    #[test]
    #[ignore = "requires supplied character rigs and paintings"]
    fn long_hair_carriers_cuts_and_bounds_survive_paged_transport() {
        use crate::new_bark_actors::character_meshes::CharacterKind;
        for kind in [
            CharacterKind::Karen,
            CharacterKind::Jasmine,
            CharacterKind::RocketGirl,
        ] {
            assert_page_round_trip(kind);
        }
    }

    fn assert_page_round_trip(kind: crate::new_bark_actors::character_meshes::CharacterKind) {
        let rig = crate::open_people::get(crate::npc_appearance::look(kind).base).unwrap();
        let appearance = super::super::prepare_appearance(rig, kind);
        assert!(appearance.mesh.count_vertices() > 1000);
        assert!(
            appearance
                .mesh
                .attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
                .is_some()
        );
        assert!(
            appearance
                .mesh
                .attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)
                .is_some()
        );
        let wire = AppearanceWire::capture(&appearance).unwrap();
        let expected = bytes(&wire);
        drop(appearance);
        let (header, mut producer) = blobs::encode_header(17, &wire).unwrap();
        drop(wire);
        let (mut decoded, mut store) = blobs::decode_header::<AppearanceWire>(17, &header).unwrap();
        while let Some(page) = producer.next_page().unwrap() {
            let ack = blobs::append_packet(&mut store, 17, &page).unwrap();
            producer.acknowledge(17, ack.0, ack.1).unwrap();
            let _ = decoded.resolve(&mut store).unwrap();
        }
        assert!(producer.finished());
        assert!(decoded.resolve(&mut store).unwrap());
        assert!(store.all_consumed());
        let restored = decoded.restore().unwrap();
        assert_eq!(
            bytes(&AppearanceWire::capture(&restored).unwrap()),
            expected,
            "{kind:?}: positions, normals, UVs, index width, joints, weights, arm anchors, bounds and accessories"
        );
    }
}
