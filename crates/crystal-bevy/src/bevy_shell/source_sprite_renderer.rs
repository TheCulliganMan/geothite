//! Restore source sprites to Bevy's direct sprite draw pipeline. Mesh-material
//! sprite conversion corrupts the LCD's independent textures and tile sizes.
//! This is the single drawing path for source sprites in every game view.

use bevy::prelude::*;
use bevy::render::sync_world::TemporaryRenderEntity;
use bevy::render::{Extract, ExtractSchedule, RenderApp};
use bevy::sprite::Anchor;
use bevy::sprite_render::{
    ExtractedSlice, ExtractedSlices, ExtractedSprite, ExtractedSpriteKind, ExtractedSprites,
};

pub(super) fn install(app: &mut App) {
    install_visibility_preparation(app);
    app.sub_app_mut(RenderApp).add_systems(
        ExtractSchedule,
        extract.after(bevy::sprite_render::extract_text2d_sprite),
    );
}
fn install_visibility_preparation(app: &mut App) {
    // SpriteMeshPlugin adds its mesh before this asset-change system. Restore
    // the direct sprite path before bounds and visible-entity collection, so
    // a fresh animation sprite can draw in the same frame it is spawned.
    app.add_systems(PostUpdate, remove_mesh
        .after(bevy::mesh::mark_2d_meshes_as_changed_if_their_assets_changed)
        .before(bevy::camera::visibility::VisibilitySystems::CalculateBounds));
}
type ConvertedSpriteMaterial = bevy::sprite_render::MeshMaterial2d<bevy::sprite_render::SpriteMeshMaterial>;

fn remove_mesh(
    mut commands: Commands,
    mut sprites: Query<&mut bevy::camera::visibility::VisibilityClass, With<Sprite>>,
    converted: Query<(Entity, Has<Mesh2d>, Has<ConvertedSpriteMaterial>),
        (With<Sprite>, Or<(With<Mesh2d>, With<ConvertedSpriteMaterial>)>)>,
) {
    #[cfg(feature = "voxel-view")]
    let _trace = crystal_voxel_view::loading_trace::span("frame.source_sprite_cleanup", ||
        serde_json::json!({"converted_sprites": converted.iter().count()}));
    for mut class in &mut sprites {
        let kind = std::any::TypeId::of::<Sprite>();
        if !class.0.contains(&kind) {
            class.0.push(kind);
        }
    }
    // Most source sprites already use the direct draw path. Queue changes
    // only for conversion components actually published by SpriteMeshPlugin,
    // including a newly converted material on a retained animation sprite.
    for (entity, mesh, material) in &converted {
        match (mesh, material) {
            (true, true) => { commands.entity(entity).remove::<(Mesh2d, ConvertedSpriteMaterial)>(); }
            (true, false) => { commands.entity(entity).remove::<Mesh2d>(); }
            (false, true) => { commands.entity(entity).remove::<ConvertedSpriteMaterial>(); }
            (false, false) => unreachable!("conversion query requires at least one component"),
        }
    }
}
fn extract(
    mut commands: Commands,
    mut sprites: ResMut<ExtractedSprites>,
    mut slices: ResMut<ExtractedSlices>,
    query: Extract<Query<(Entity, &Sprite, &GlobalTransform, &ViewVisibility, &Anchor)>>,
    images: Extract<Res<Assets<Image>>>,
    atlases: Extract<Res<Assets<TextureAtlasLayout>>>,
) {
    for (entity, sprite, global, visible, anchor) in &query {
        if !visible.get() {
            continue;
        }
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        let mut rect = Rect::from_corners(Vec2::ZERO, image.size_f32());
        if let Some(atlas) = &sprite.texture_atlas {
            let Some(layout) = atlases.get(&atlas.layout) else {
                continue;
            };
            let Some(tile) = layout.textures.get(atlas.index) else {
                continue;
            };
            rect = tile.as_rect();
        }
        if let Some(crop) = sprite.rect {
            rect = Rect::from_corners(rect.min + crop.min, rect.min + crop.max);
        }
        let size = sprite.custom_size.unwrap_or(rect.size());
        let start = slices.slices.len();
        slices.slices.push(ExtractedSlice {
            offset: Vec2::ZERO,
            rect,
            size,
        });
        let local = Transform::from_translation((-anchor.as_vec() * size).extend(0.0))
            .with_scale(Vec3::new(if sprite.flip_x { -1.0 } else { 1.0 }, 1.0, 1.0));
        sprites.sprites.push(ExtractedSprite {
            main_entity: entity,
            render_entity: commands.spawn(TemporaryRenderEntity::default()).id(),
            transform: *global * local,
            color: sprite.color.to_linear(),
            image_handle_id: sprite.image.id(),
            flip_y: sprite.flip_y,
            kind: ExtractedSpriteKind::Slices {
                indices: start..start + 1,
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Resource, Default)]
    struct Observed(Vec<bool>);
    fn collect_visibility(
        sprites: Query<(&bevy::camera::visibility::VisibilityClass, Has<Mesh2d>), With<Sprite>>,
        mut observed: ResMut<Observed>,
    ) {
        observed.0.push(!sprites.is_empty() && sprites.iter().all(|(class, mesh)|
            class.0.contains(&std::any::TypeId::of::<Sprite>()) && !mesh));
    }
    #[test]
    fn fresh_source_animation_sprites_are_ready_before_visibility_collection() {
        let mut app = App::new();
        app.init_resource::<Observed>();
        install_visibility_preparation(&mut app);
        app.add_systems(Update, |mut commands: Commands, sprites: Query<Entity, With<Sprite>>| {
            for entity in &sprites { commands.entity(entity).despawn(); }
            commands.spawn((Sprite::default(), Mesh2d(Handle::default()), bevy::sprite_render::MeshMaterial2d::<bevy::sprite_render::SpriteMeshMaterial>(Handle::default())));
        });
        app.add_systems(PostUpdate, collect_visibility
            .in_set(bevy::camera::visibility::VisibilitySystems::CheckVisibility)
            .after(bevy::camera::visibility::VisibilitySystems::CalculateBounds));
        for _ in 0..4 { app.update(); }
        assert_eq!(app.world().resource::<Observed>().0, vec![true; 4]);
    }
    #[test]
    fn retained_source_animation_keeps_the_direct_path_after_production_conversion() {
        use bevy::asset::AssetApp;
        use bevy::camera::visibility::VisibilityClass;
        use bevy::sprite_render::SpriteMeshPlugin;
        #[derive(Resource, Default)]
        struct ConversionObserved(Vec<(usize, usize, usize)>);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<bevy::shader::Shader>()
            .init_asset::<Image>()
            .init_asset::<TextureAtlasLayout>()
            .add_plugins((bevy::mesh::MeshPlugin, SpriteMeshPlugin))
            .init_resource::<ConversionObserved>();
        install_visibility_preparation(&mut app);
        app.add_systems(PostUpdate, (
            |mut removed_meshes: RemovedComponents<Mesh2d>,
             mut removed_materials: RemovedComponents<ConvertedSpriteMaterial>,
             changed_classes: Query<Entity, (With<Sprite>, Changed<VisibilityClass>)>,
             mut observed: ResMut<ConversionObserved>| {
                observed.0.push((removed_meshes.read().count(), removed_materials.read().count(), changed_classes.iter().count()));
            })
            .in_set(bevy::camera::visibility::VisibilitySystems::CheckVisibility)
            .after(bevy::camera::visibility::VisibilitySystems::CalculateBounds));
        let source = Sprite { custom_size: Some(Vec2::new(8., 16.)), flip_x: true, ..default() };
        let entity = app.world_mut().spawn(source.clone()).id();
        app.update();
        let assert_direct = |app: &App, expected: &Sprite| {
            let sprite = app.world().entity(entity);
            assert_eq!(sprite.get::<Sprite>(), Some(expected), "cleanup preserves source texture, crop, size and facing");
            assert!(!sprite.contains::<Mesh2d>() && !sprite.contains::<ConvertedSpriteMaterial>());
            assert!(sprite.get::<VisibilityClass>().unwrap().0.contains(&std::any::TypeId::of::<Sprite>()));
        };
        assert_direct(&app, &source);
        let first = app.world().resource::<ConversionObserved>().0[0];
        assert_eq!((first.0, first.1), (1, 1), "real SpriteMeshPlugin conversion is stripped before visibility");
        for _ in 0..3 { app.update(); assert_direct(&app, &source); }
        assert!(app.world().resource::<ConversionObserved>().0[1..].iter().all(|&counts| counts == (0, 0, 0)),
            "retained direct sprites have no later conversion removals or visibility mutations");
        let mut next_frame = source.clone();
        next_frame.flip_x = false;
        next_frame.rect = Some(Rect::from_corners(Vec2::ZERO, Vec2::splat(8.)));
        app.world_mut().entity_mut(entity).insert(next_frame.clone());
        app.update();
        assert_direct(&app, &next_frame);
        let changed = *app.world().resource::<ConversionObserved>().0.last().unwrap();
        assert_eq!((changed.0, changed.1), (0, 1), "changed retained sprites receive a new material, not a new mesh");
        app.update();
        assert_direct(&app, &next_frame);
        assert_eq!(*app.world().resource::<ConversionObserved>().0.last().unwrap(), (0, 0, 0));
    }

}
