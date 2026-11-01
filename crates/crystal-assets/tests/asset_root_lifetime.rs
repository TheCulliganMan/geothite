use crystal_assets::AssetRoot;

#[test]
fn temporary_asset_root_survives_clones_and_removes_only_its_owned_path() {
    let parent = AssetRoot::new_temporary_in(std::env::temp_dir()).unwrap();
    let source_path = parent.repository_root.join("user-source");
    std::fs::create_dir(&source_path).unwrap();
    let pack_path = source_path.join("original.crystalpack");
    std::fs::write(&pack_path, b"user pack").unwrap();
    let source = AssetRoot::new(&source_path);
    drop(source.clone());
    drop(source);
    assert_eq!(std::fs::read(&pack_path).unwrap(), b"user pack");

    let mut owned = AssetRoot::new_temporary_in(&parent.repository_root).unwrap();
    let owned_path = owned.repository_root.clone();
    let reader = owned.clone();
    // Public lookup-path mutation cannot change which directory we own.
    owned.repository_root = source_path;
    drop(owned);
    assert!(owned_path.is_dir());
    drop(reader);
    assert!(!owned_path.exists());
    assert_eq!(std::fs::read(&pack_path).unwrap(), b"user pack");
}

#[test]
fn serialized_asset_root_does_not_grant_directory_ownership() {
    let owned = AssetRoot::new_temporary_in(std::env::temp_dir()).unwrap();
    let path = owned.repository_root.clone();
    let serialized = serde_json::to_value(&owned).unwrap();
    assert_eq!(serialized, serde_json::json!({"repository_root": path}));
    let borrowed: AssetRoot = serde_json::from_value(serialized).unwrap();
    assert_eq!(owned, borrowed);
    drop(borrowed);
    assert!(path.is_dir());
    drop(owned);
    assert!(!path.exists());
}
