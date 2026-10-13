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
    app.add_systems(Last, remove_mesh);
    app.sub_app_mut(RenderApp).add_systems(
        ExtractSchedule,
        extract.after(bevy::sprite_render::extract_text2d_sprite),
    );
}
fn remove_mesh(
    mut commands: Commands,
    mut sprites: Query<(Entity, &mut bevy::camera::visibility::VisibilityClass), With<Sprite>>,
) {
    for (entity, mut class) in &mut sprites {
        let kind = std::any::TypeId::of::<Sprite>();
        if !class.0.contains(&kind) {
            class.0.push(kind);
        }
        commands
            .entity(entity)
            .remove::<Mesh2d>()
            .remove::<bevy::sprite_render::MeshMaterial2d<bevy::sprite_render::SpriteMeshMaterial>>(
            );
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
