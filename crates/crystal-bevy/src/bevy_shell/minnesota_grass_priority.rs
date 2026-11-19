//! Crystal's upper two walking OAM pieces stay in front of the grass.
use bevy::prelude::{
    Commands, Component, Entity, Query, Rect, Res, Sprite, Transform, Vec2, Visibility, With,
    Without,
};
use crystal_core::world::collision::{permissions, sample_collision};
use crystal_core::world::map::TilePosition;
use crystal_core::world::session::OverworldSession;

use super::{
    BevyRuntimeShell, PLAYFIELD_LEFT, PLAYFIELD_TOP, PlayerMarker, PlayerSpriteFrames,
    PlayfieldBridgeTile, PlayfieldPriorityTile, PlayfieldTile, RenderedTilesetArt,
    RenderedViewport, ResolvedRenderSource, SOURCE_TILE_SIZE, TILE_SIZE, TilesetArt,
};

const GRASS: u8 = 0x04;

fn uses_johto_blades(map: &str, bank: &str) -> bool {
    (bank == "johto" && map.starts_with("Mn"))
        || (bank == "mn_prairie_headwaters" && map == "MnFormerShores")
}

/// At a mixed grass square only the blades belong in front of the boots.
/// The same brown glyph used by a cliff or facade keeps its own priority.
pub(super) fn grass_edge_ground(
    source: &ResolvedRenderSource<'_>,
    art: &TilesetArt,
    block: u16,
    sub_x: usize,
    sub_y: usize,
    glyph: u8,
) -> bool {
    let bank = source.tileset.tileset_id.as_str();
    if !uses_johto_blades(&source.map.map_name, bank) {
        return false;
    }
    let ground = matches!(
        glyph,
        0x05 | 0x2c | 0x2d | 0x3b | 0x3d | 0x4b | 0x4c | 0x4d | 0x5c | 0x5d
    ) || bank == "johto" && matches!(glyph, 0x74..=0x77);
    let quadrant = sub_y / 2 * 2 + sub_x / 2;
    if !ground
        || !super::tileset_collision_tokens(source.tileset, block)
            .and_then(|roles| roles.get(quadrant))
            .is_some_and(|role| role == "TALL_GRASS")
    {
        return false;
    }
    let start = usize::from(block) * 16 + (sub_y & 2) * 4 + (sub_x & 2);
    [0, 1, 4, 5]
        .into_iter()
        .any(|offset| art.metatile_layout.get(start + offset) == Some(&GRASS))
}

#[derive(Component)]
pub(super) struct ForegroundPiece;

type Ground = (
    With<PlayfieldTile>,
    Without<PlayfieldPriorityTile>,
    Without<PlayfieldBridgeTile>,
);

fn grass_glyph(world: &OverworldSession, layout: &[u8], gx: i16, gy: i16) -> bool {
    let position = TilePosition::new(gx.div_euclid(2), gy.div_euclid(2));
    let Some(tile) = sample_collision(&world.map, &world.tileset, position) else {
        return false;
    };
    let index = usize::from(tile.metatile_id) * 16
        + (gy.rem_euclid(4) as usize) * 4
        + gx.rem_euclid(4) as usize;
    tile.permission == permissions::TALL_GRASS && layout.get(index) == Some(&GRASS)
}

/// Draw only the overlap between a grass glyph and the protected OAM rows.
/// Each piece is a rectangle of the currently displayed player texture. This
/// keeps the real stride, flip and palette, and needs no second sprite atlas.
pub(super) fn draw(
    mut commands: Commands,
    shell: Res<BevyRuntimeShell>,
    viewport: Res<RenderedViewport>,
    art: Res<RenderedTilesetArt>,
    ground: Query<&Transform, Ground>,
    players: Query<(&Transform, &Sprite, &PlayerSpriteFrames, &Visibility), With<PlayerMarker>>,
    pieces: Query<Entity, With<ForegroundPiece>>,
) {
    let world = shell.shell.session().overworld();
    let mut retained = pieces.iter();
    if world.map.name.starts_with("Mn")
        && viewport.map_name.as_deref() == Some(world.map.name.as_str())
        && [Some(world.player.tile), shell.player_walk_from]
            .into_iter()
            .flatten()
            .any(|tile| {
                sample_collision(&world.map, &world.tileset, tile)
                    .is_some_and(|sample| sample.permission == permissions::TALL_GRASS)
            })
        && let Some(room) = shell.shell.runtime().data().maps.get(&world.map.name)
        && uses_johto_blades(&world.map.name, &room.attributes.tileset_name)
        && let Some((left, top)) = viewport.viewport_origin
        && let Ok(ground) = ground.single()
        && let Ok((player, sprite, frames, visibility)) = players.single()
        && *visibility != Visibility::Hidden
        && let Some((_, tileset)) = art
            .cache
            .iter()
            .find(|(key, _)| key.tileset_id == room.attributes.tileset_name)
    {
        let protected = match frames.source_id.as_ref() {
            "chris" | "kris" => 8,
            "mn_canoe_chris" | "mn_canoe_kris" => 16,
            _ => 0,
        };
        let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
        let center = player.translation - ground.translation;
        let x0 = (f32::from(left) * 8.0 + (center.x - PLAYFIELD_LEFT) / scale - 8.0).round() as i16;
        let y0 = (f32::from(top) * 8.0 + (PLAYFIELD_TOP - center.y) / scale - 8.0).round() as i16;
        let mut y = 0;
        while y < protected {
            let gy = (y0 + y).div_euclid(8);
            let bottom = protected.min((gy + 1) * 8 - y0);
            let mut x = 0;
            while x < 16 {
                let gx = (x0 + x).div_euclid(8);
                let right = 16.min((gx + 1) * 8 - x0);
                if grass_glyph(world, &tileset.metatile_layout, gx, gy) {
                    let (src_left, src_right) = if sprite.flip_x {
                        (16 - right, 16 - x)
                    } else {
                        (x, right)
                    };
                    let mut piece = sprite.clone();
                    piece.rect = Some(Rect::new(
                        src_left.into(),
                        y.into(),
                        src_right.into(),
                        bottom.into(),
                    ));
                    piece.custom_size =
                        Some(Vec2::new((right - x).into(), (bottom - y).into()) * scale);
                    let transform = Transform::from_xyz(
                        player.translation.x + (f32::from(x + right) * 0.5 - 8.0) * scale,
                        player.translation.y - (f32::from(y + bottom) * 0.5 - 8.0) * scale,
                        2.41, // Above grass at 2.4, below its rustling blades at 2.5.
                    );
                    if let Some(entity) = retained.next() {
                        commands
                            .entity(entity)
                            .insert((piece, transform, *visibility));
                    } else {
                        commands.spawn((ForegroundPiece, piece, transform, *visibility));
                    }
                }
                x = right;
            }
            y = bottom;
        }
    }
    for unused in retained {
        commands.entity(unused).despawn();
    }
}
