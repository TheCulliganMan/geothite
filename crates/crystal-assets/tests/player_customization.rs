#[cfg(test)]
mod tests {
    use crystal_assets::GameDataSet;
    #[test]
    fn verified_browser_pack_gets_a_distinct_repeatable_customization_identity() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../content-packs/core-modular.browser.crystalpack")
            .canonicalize()
            .unwrap();
        let base = crystal_assets::read_verified_compiled_game_pack(path).unwrap();
        let pack = crystal_assets::build_player_customization_modpack(&base).unwrap();
        assert!(pack.data().player_customization);
        let (legacy, content_hash) = crystal_assets::player_customization_base_save_identity(&pack).unwrap().unwrap();
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../content-packs/core-modular.browser.crystalpack")
        ).unwrap();
        assert_eq!(legacy, loaded.save_modpack_identity().unwrap());
        assert_eq!(content_hash, base.identity().unwrap().content_hash);
        assert!(!base.data().player_customization);
        assert!(
            pack.runtime_modpack_id()
                .unwrap()
                .ends_with(crystal_assets::PLAYER_CUSTOMIZATION_MANIFEST_ID)
        );
        assert_ne!(base.identity().unwrap(), pack.identity().unwrap());
        assert_eq!(
            pack.identity().unwrap(),
            crystal_assets::build_player_customization_modpack(&base)
                .unwrap()
                .identity()
                .unwrap()
        );
        assert!(crystal_assets::build_player_customization_modpack(&pack).is_err());
        let mut unchanged = pack.data().clone();
        unchanged.player_customization = false;
        assert_eq!(&unchanged, base.data());
        assert_eq!(
            pack.audio_manifest().unwrap(),
            base.audio_manifest().unwrap()
        );
        assert_eq!(pack.runtime_files(), base.runtime_files());
    }

    #[test]
    fn disabled_customization_preserves_base_pack_serialization() {
        let data = GameDataSet::default();
        assert!(
            serde_json::to_value(&data)
                .unwrap()
                .get("player_customization")
                .is_none()
        );
        let data = GameDataSet {
            player_customization: true,
            ..data
        };
        assert_eq!(serde_json::to_value(&data).unwrap()["player_customization"], true);
    }
}
