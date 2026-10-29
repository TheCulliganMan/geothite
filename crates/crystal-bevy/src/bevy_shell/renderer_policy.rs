//! Renderer availability is a content rule, independent of saved preferences.

use bevy::prelude::Resource;
use crystal_runtime::CrystalRuntime;

/// Whether this pack permits the voxel renderer, including its 2.5D cameras.
///
/// Minnesota packs use native 2D throughout the session, including original
/// maps retained in a regional pack. Detect pack identity and content rather
/// than a pack filename.
pub fn runtime_supports_voxel_view(runtime: &CrystalRuntime) -> bool {
    supports_content(
        runtime.modpack().id(),
        runtime.map_ids().iter().map(String::as_str),
    )
}

fn supports_content<'a>(modpack_id: &str, maps: impl IntoIterator<Item = &'a str>) -> bool {
    !modpack_id.split('+').any(|id| id.starts_with("minnesota"))
        && !maps.into_iter().any(|name| name.starts_with("Mn"))
}

#[derive(Resource, Clone, Copy)]
pub(super) struct RendererPolicy {
    pub(super) voxel_allowed: bool,
}

#[cfg(feature = "voxel-view")]
pub(super) fn enforce(
    policy: bevy::prelude::Res<RendererPolicy>,
    mut settings: bevy::prelude::ResMut<crystal_voxel_view::VoxelViewSettings>,
) {
    if !policy.voxel_allowed {
        settings.enabled = false;
        settings.allow_f3_toggle = false;
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static BROWSER_VOXEL_ALLOWED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(target_arch = "wasm32")]
pub(super) fn publish_browser_policy(policy: RendererPolicy) {
    BROWSER_VOXEL_ALLOWED.with(|value| value.set(policy.voxel_allowed));
}

/// Whether the loaded game allows a browser 3D or 2.5D presentation.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn crystal_voxel_view_allowed() -> bool {
    cfg!(feature = "voxel-view") && BROWSER_VOXEL_ALLOWED.with(std::cell::Cell::get)
}

#[cfg(test)]
mod tests {
    use super::supports_content;

    #[test]
    fn minnesota_content_is_2d_only_even_when_original_maps_are_retained() {
        for regional_map in ["MnWinonaHome2F", "MnR01C0R0", "MnGrandMarais"] {
            assert!(!supports_content(
                "renamed-review-pack",
                ["PlayersHouse2F", "NewBarkTown", regional_map]
            ));
        }
        assert!(supports_content(
            "pokecrystal",
            ["PlayersHouse2F", "NewBarkTown", "VermilionCity"]
        ));
        assert!(!supports_content(
            "pokecrystal+minnesota-pokemon-v1",
            ["NewBarkTown"]
        ));
    }

    #[test]
    #[ignore = "requires CRYSTAL_RENDER_TEST_PACK pointing to a completed Minnesota pack"]
    fn minnesota_external_pack_is_2d_only() {
        use crystal_assets::AssetRoot;

        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .expect("set CRYSTAL_RENDER_TEST_PACK to a completed Minnesota pack");
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path)
            .expect("load verified external Minnesota pack");
        let root = AssetRoot::new(path.parent().expect("pack directory").to_path_buf());
        let runtime = crystal_runtime::CrystalRuntime::from_loaded_compiled_pack(&root, loaded)
            .expect("load Minnesota runtime");
        assert!(!super::runtime_supports_voxel_view(&runtime));
    }

    #[cfg(feature = "voxel-view")]
    #[test]
    fn minnesota_rejects_repeated_3d_requests_and_f3() {
        use bevy::prelude::{App, ButtonInput, KeyCode, Update};
        use crystal_voxel_view::VoxelViewSettings;

        let mut app = App::new();
        app.insert_resource(super::RendererPolicy {
            voxel_allowed: false,
        })
        .init_resource::<VoxelViewSettings>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, super::enforce);
        for _ in 0..3 {
            let mut settings = app.world_mut().resource_mut::<VoxelViewSettings>();
            settings.enabled = true;
            settings.allow_f3_toggle = true;
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::F3);
            app.update();
            let settings = app.world().resource::<VoxelViewSettings>();
            assert!(!settings.enabled);
            assert!(!settings.allow_f3_toggle);
        }
        app.world_mut()
            .resource_mut::<super::RendererPolicy>()
            .voxel_allowed = true;
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        app.update();
        assert!(app.world().resource::<VoxelViewSettings>().enabled);
    }
}
