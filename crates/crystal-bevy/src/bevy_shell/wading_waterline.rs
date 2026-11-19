//! Water at the soles of the standing or walking trainer in Brainerd's swimming beach.
use bevy::prelude::{
    Assets, Handle, Image, Local, Query, Res, ResMut, Sprite, Transform, With, Without,
};
use crystal_core::world::collision::{permissions, sample_collision};
use crystal_core::world::map::TilePosition;
use crystal_core::world::movement::MovementMode;
use crystal_core::world::session::OverworldSession;

use super::{
    BevyRuntimeShell, PLAYFIELD_LEFT, PLAYFIELD_TOP, PlayerMarker, PlayerSpriteFrames,
    PlayfieldBridgeTile, PlayfieldPriorityTile, PlayfieldTile, RenderedTilesetArt,
    RenderedViewport, SOURCE_TILE_SIZE, TILE_SIZE, TilesetArt,
};

const BEACH: &str = "MnBrainerdGym";
const ATLAS: &str = "mn_gym_6";
const RIPPLE_FILE: &str = "data/tilesets/mn_gym_6_animations.json";
const SHALLOW_RIPPLE: usize = 0x52;
const FOOT_ROW: usize = 14;

type Ground = (
    With<PlayfieldTile>,
    Without<PlayfieldPriorityTile>,
    Without<PlayfieldBridgeTile>,
);

#[derive(Default)]
pub(super) struct WaterlineFrame {
    source: Handle<Image>,
    wet: u32,
    output: Option<Handle<Image>>,
}

/// The native beach gives water its own material palette. At a mixed edge,
/// the upper bitplane is water and the lower two indices are dry grains.
fn wet_pixel(world: &OverworldSession, art: &TilesetArt, palettes: &[u8], x: i16, y: i16) -> bool {
    let tile = TilePosition::new(x.div_euclid(16), y.div_euclid(16));
    let Some(footing) = sample_collision(&world.map, &world.tileset, tile) else {
        return false;
    };
    if footing.permission != permissions::FLOOR {
        return false;
    }
    let subtile =
        usize::from(footing.metatile_id) * 16 + (y as usize % 32 / 8) * 4 + x as usize % 32 / 8;
    let Some(&glyph) = art.metatile_layout.get(subtile) else {
        return false;
    };
    let glyph = usize::from(glyph);
    match palettes.get(glyph) {
        Some(3) => true,
        Some(6) => art
            .transition_tiles
            .get(glyph)
            .is_some_and(|native| native.indices[(y as usize % 8) * 8 + x as usize % 8] & 2 != 0),
        _ => false,
    }
}

fn sink_boots(data: &mut [u8], wet: u32, ripple: [u8; 3]) {
    let ankle = &data[FOOT_ROW * 16 * 4..(FOOT_ROW + 1) * 16 * 4];
    let opaque = |x: usize| ankle[x * 4 + 3] != 0;
    let first = (0..16).find(|&x| opaque(x));
    let last = (0..16).rfind(|&x| opaque(x));
    for bit in 0..32 {
        if wet & (1 << bit) == 0 {
            continue;
        }
        let pixel = &mut data[(FOOT_ROW * 16 + bit) * 4..][..4];
        // Two half-crests touch the boot, with live water visible between them.
        let edge = [first, last].into_iter().flatten().any(|end| {
            wet & (1 << end) != 0
                && (bit == end
                    || first == Some(end) && bit.checked_add(1) == Some(end)
                    || last == Some(end) && bit == end + 1)
        });
        if bit < 16 && edge {
            pixel.copy_from_slice(&[ripple[0], ripple[1], ripple[2], 255]);
        } else {
            pixel[3] = 0;
        }
    }
}

/// Samples the two actual rendered soles against the retained terrain transform.
/// No game tick, geometry, controls, shared sprite or water frame is mutated.
pub(super) fn draw(
    shell: Res<BevyRuntimeShell>,
    viewport: Res<RenderedViewport>,
    tilesets: Res<RenderedTilesetArt>,
    ground: Query<&Transform, Ground>,
    mut players: Query<(&Transform, &mut Sprite, &PlayerSpriteFrames), With<PlayerMarker>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<WaterlineFrame>,
) {
    let world = shell.shell.session().overworld();
    if world.map.name != BEACH && cache.output.is_none() {
        return;
    }
    let Ok((player, mut sprite, frames)) = players.single_mut() else {
        return;
    };
    // A retained entity may survive a warp or a change of mount. Revert only
    // our private handle, and never leave the previous trainer on a new actor.
    if cache.output.as_ref() == Some(&sprite.image) {
        sprite.image =
            if cache.source == frames.standing || frames.walking.as_ref() == Some(&cache.source) {
                cache.source.clone()
            } else {
                frames.standing.clone()
            };
    }
    if world.map.name != BEACH || viewport.map_name.as_deref() != Some(BEACH) {
        *cache = WaterlineFrame::default();
        return;
    }
    // The step commits Surf before the walking silhouette physically leaves
    // the shoal. Let the selected native frame and its footing own that edge.
    if !matches!(world.player.mode, MovementMode::Normal | MovementMode::Surf)
        || !matches!(frames.source_id.as_ref(), "chris" | "kris")
        || shell.visible_ledge_jump.is_some()
        || shell.visible_fly_animation.is_some()
        || shell.shell.runtime().runtime_file(RIPPLE_FILE).is_none()
    {
        return;
    }
    let (Ok(ground), Some((left, top))) = (ground.single(), viewport.viewport_origin) else {
        return;
    };
    // This is the same decoded, indexed 8x8 sheet as the current native room.
    // The animation is the release's opt-in. The original beach has the same
    // room and name but does not declare it.
    let Some((key, art)) = tilesets.cache.iter().find(|(key, art)| {
        key.tileset_id == ATLAS
            && key.time_of_day == "indoor"
            && art.animated_tiles.contains_key(&SHALLOW_RIPPLE)
    }) else {
        return;
    };
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let center = player.translation - ground.translation;
    let x0 = (f32::from(left) * 8.0 + (center.x - PLAYFIELD_LEFT) / scale - 8.0).round() as i16;
    let y0 = (f32::from(top) * 8.0 + (PLAYFIELD_TOP - center.y) / scale - 8.0).round() as i16;
    let mut mask = 0_u32;
    for bit in 0_u32..32 {
        let col = (bit % 16) as i16;
        let x = x0 + if sprite.flip_x { 15 - col } else { col };
        let y = y0 + FOOT_ROW as i16 + (bit / 16) as i16;
        if wet_pixel(world, art, &key.palette_map, x, y) {
            mask |= 1 << bit;
        }
    }
    if mask == 0 {
        return;
    }
    let source = sprite.image.clone();
    if cache.source != source || cache.wet != mask || cache.output.is_none() {
        let Some(mut frame) = images
            .get(&source)
            .filter(|image| image.width() == 16 && image.height() == 16)
            .cloned()
        else {
            return;
        };
        let Some(data) = frame.data.as_mut().filter(|data| data.len() == 16 * 16 * 4) else {
            return;
        };
        let Some(water) = art.transition_tiles.get(SHALLOW_RIPPLE) else {
            return;
        };
        sink_boots(data, mask, water.palette[1]);
        if let Some(mut output) = cache
            .output
            .as_ref()
            .and_then(|handle| images.get_mut(handle))
        {
            *output = frame;
        } else {
            cache.output = Some(images.add(frame));
        }
        cache.source = source;
        cache.wet = mask;
    }
    if let Some(output) = &cache.output {
        sprite.image = output.clone();
    }
}
