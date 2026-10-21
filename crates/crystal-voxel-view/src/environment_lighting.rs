//! Small procedural reflection/irradiance maps derived from the visible sky.
//! Persistent handles and a five-minute lighting key bound allocations/uploads.
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageSampler, ImageSamplerDescriptor},
    light::EnvironmentMapLight,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};
use crystal_render_api::{VisualWorldClock, VisualWorldFrame, VisualWorldLighting};

// The procedural pixels describe normalized sky colors. StandardMaterial
// consumes environment radiance in cd/m² before camera exposure; intensity 1
// leaves its unlit sides almost black beside the 3000-lux world sun.
const SKY_RADIANCE_CD_M2: f32 = 600.0;

#[derive(Resource, Default)]
struct EnvironmentCache {
    key: Option<u32>,
    diffuse: Handle<Image>,
    specular: Handle<Image>,
}

pub(super) fn register(app: &mut App) {
    app.init_resource::<EnvironmentCache>()
        .add_systems(Update, sync.after(crate::sync_voxel_atmosphere));
}

fn sync(
    mut commands: Commands,
    frame: Res<VisualWorldFrame>,
    clock: Option<Res<VisualWorldClock>>,
    lighting: Option<Res<VisualWorldLighting>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: ResMut<EnvironmentCache>,
    mut cameras: Query<(Entity, Option<&mut EnvironmentMapLight>), With<crate::VoxelWorldCamera>>,
) {
    if !crate::render_quality::requested().materials() {
        return;
    }
    let second = clock
        .as_ref()
        .and_then(|c| c.seconds_of_day())
        .unwrap_or_else(|| match lighting.as_deref() {
            Some(VisualWorldLighting::Morning) => 7 * 3600,
            Some(VisualWorldLighting::Night | VisualWorldLighting::Dark) => 22 * 3600,
            _ => 12 * 3600,
        });
    let key = second / 300;
    if cache.key != Some(key) {
        let colors = crate::sky::environment_colors(key * 300);
        let diffuse = cubemap(colors, 8, 1, true);
        let specular = cubemap(colors, 32, 6, false);
        if cache.key.is_none() {
            cache.diffuse = images.add(diffuse);
            cache.specular = images.add(specular);
        } else {
            images
                .insert(cache.diffuse.id(), diffuse)
                .expect("retained environment image handle");
            images
                .insert(cache.specular.id(), specular)
                .expect("retained environment image handle");
        }
        cache.key = Some(key);
    }
    let interior = crate::interior_cutaway::is_cave(&frame)
        || frame
            .tiles
            .first()
            .is_some_and(|t| crate::interior::has_back_wall(&t.source.tileset_id)
                && !crate::interior::open_air_roof(&frame.map_id,&t.source.tileset_id));
    let intensity = if interior || matches!(lighting.as_deref(), Some(VisualWorldLighting::Dark)) {
        0.0
    } else {
        SKY_RADIANCE_CD_M2
    };
    for (entity, light) in &mut cameras {
        if let Some(mut light) = light {
            if light.intensity != intensity {
                light.intensity = intensity;
            }
        } else {
            commands.entity(entity).insert(EnvironmentMapLight {
                diffuse_map: cache.diffuse.clone(),
                specular_map: cache.specular.clone(),
                intensity,
                ..default()
            });
        }
    }
}

/// Cube-face directions use the WebGPU +/-X, +/-Y, +/-Z convention. A continuous
/// height field avoids seams; roughness levels analytically broaden the sky /
/// horizon / ground lobes rather than simply shrinking sharp sky texels.
fn direction(face: u32, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, -v, -u),
        1 => Vec3::new(-1.0, -v, u),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, -v),
        4 => Vec3::new(u, -v, 1.0),
        _ => Vec3::new(-u, -v, -1.0),
    }
    .normalize()
}
fn cubemap(colors: [Vec3; 3], size: u32, levels: u32, diffuse: bool) -> Image {
    let mut pixels = Vec::new();
    let average = (colors[0] + colors[1] * 2.0 + colors[2]) * 0.25;
    // Image's default LayerMajor order stores each face's complete mip chain.
    for face in 0..6 {
        for level in 0..levels {
            let width = (size >> level).max(1);
            let roughness = if diffuse {
                0.8
            } else {
                level as f32 / (levels - 1).max(1) as f32
            };
            for y in 0..width {
                for x in 0..width {
                    let n = direction(
                        face,
                        (x as f32 + 0.5) / width as f32 * 2.0 - 1.0,
                        (y as f32 + 0.5) / width as f32 * 2.0 - 1.0,
                    );
                    let height = n.y;
                    let color = if height >= 0.0 {
                        colors[1].lerp(colors[0], height.sqrt())
                    } else {
                        colors[1].lerp(colors[2], (-height).sqrt())
                    };
                    let color = color.lerp(average, roughness * roughness * 0.65);
                    let srgb = Color::linear_rgb(color.x, color.y, color.z).to_srgba();
                    pixels.extend(srgb.to_u8_array());
                }
            }
        }
    }
    // The payload includes every mip, while Image::new validates only the
    // base level. Install the complete payload after its mip count is known.
    let mut image = Image::new_uninit(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = levels;
    image.data = Some(pixels);
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clock_changes_reuse_two_images_and_fixed_darkness_disables_sky_light() {
        let mut app = App::new();
        app.init_resource::<VisualWorldFrame>()
            .init_resource::<VisualWorldClock>()
            .init_resource::<VisualWorldLighting>()
            .init_resource::<Assets<Image>>()
            .init_resource::<EnvironmentCache>()
            .add_systems(Update, sync);
        let camera = app.world_mut().spawn(crate::VoxelWorldCamera).id();
        let frame = app.world().resource::<VisualWorldFrame>().clone();
        app.world_mut()
            .resource_mut::<VisualWorldClock>()
            .minute_of_day = Some(720);
        app.update();
        let original = app
            .world()
            .get::<EnvironmentMapLight>(camera)
            .unwrap()
            .clone();
        for minute in [721, 722, 730, 1320] {
            app.world_mut()
                .resource_mut::<VisualWorldClock>()
                .minute_of_day = Some(minute);
            app.update();
            let light = app.world().get::<EnvironmentMapLight>(camera).unwrap();
            assert_eq!(light.diffuse_map, original.diffuse_map);
            assert_eq!(light.specular_map, original.specular_map);
            assert_eq!(app.world().resource::<Assets<Image>>().len(), 2);
            assert_eq!(*app.world().resource::<VisualWorldFrame>(), frame);
        }
        *app.world_mut().resource_mut::<VisualWorldLighting>() = VisualWorldLighting::Dark;
        app.update();
        assert_eq!(
            app.world()
                .get::<EnvironmentMapLight>(camera)
                .unwrap()
                .intensity,
            0.0
        );
        *app.world_mut().resource_mut::<VisualWorldLighting>()=VisualWorldLighting::Day;
        {
            let mut frame=app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.map_id="CeladonMansionRoof".into();
            frame.tiles=vec![crystal_render_api::VisualTile {
                column:0,row:0,texture:Handle::default(),priority:false,animation_frames:None,
                source:crystal_render_api::VisualTileSource {tileset_id:"mansion".into(),
                    metatile_id:0,subtile_column:0,subtile_row:0,tile_index:0},
            }];
        }
        app.update();
        assert_eq!(app.world().get::<EnvironmentMapLight>(camera).unwrap().intensity,SKY_RADIANCE_CD_M2,
            "an open rooftop receives the same clock-driven sky reflections");
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id="CeladonMansionRoofHouse".into();
        app.update();
        assert_eq!(app.world().get::<EnvironmentMapLight>(camera).unwrap().intensity,0.0,
            "the enclosed rooftop house must not inherit the outdoor environment");
        assert_eq!(app.world().resource::<Assets<Image>>().len(),2);
    }
    #[test]
    fn reflection_maps_have_all_faces_mips_and_follow_visible_daylight() {
        let colors = crate::sky::environment_colors(12 * 3600);
        let day = cubemap(colors, 32, 6, false);
        let bytes_per_face = (0..6)
            .map(|l| ((32 >> l) * (32 >> l) * 4) as usize)
            .sum::<usize>();
        assert_eq!(day.data.as_ref().unwrap().len(), bytes_per_face * 6);
        assert_eq!(day.texture_descriptor.mip_level_count, 6);
        assert_eq!(
            day.texture_view_descriptor.unwrap().dimension,
            Some(TextureViewDimension::Cube)
        );
        let night = crate::sky::environment_colors(22 * 3600);
        assert!(colors[0].length() > night[0].length() * 2.0);
        // No face contains uninitialised black/transparent padding.
        assert!(
            day.data
                .as_ref()
                .unwrap()
                .chunks_exact(4)
                .all(|p| p[3] == 255)
        );
    }
    #[test]
    fn stored_sky_fill_has_daylight_energy_without_replacing_the_sun_or_night() {
        // Decode the actual stored sRGB diffuse pixels, rather than accepting
        // a large intensity number as proof of useful indirect illumination.
        let rear_fill = |second| {
            let image = cubemap(crate::sky::environment_colors(second), 8, 1, true);
            let pixels = image.data.as_ref().unwrap();
            let offset = (5 * 8 * 8 + 4 * 8 + 4) * 4;
            let rgb = Color::srgba_u8(pixels[offset],pixels[offset+1],pixels[offset+2],255)
                .to_linear();
            Vec3::new(rgb.red,rgb.green,rgb.blue).dot(Vec3::new(0.2126,0.7152,0.0722))
                * SKY_RADIANCE_CD_M2
        };
        let noon = rear_fill(12 * 3600);
        let direct = 3000.0 / std::f32::consts::PI;
        assert!((0.18..0.45).contains(&(noon / direct)),
            "sky fill must reveal a sun-facing-away surface while retaining directional volume: {noon}");
        assert!(rear_fill(22 * 3600) < noon * 0.08,
            "the real night sky must remain much darker than daylight");
        assert!(rear_fill(6 * 3600) > rear_fill(4 * 3600));
    }
    #[test]
    fn cubemap_face_axes_and_edges_agree() {
        assert_eq!(direction(2, 0.0, 0.0), Vec3::Y);
        assert_eq!(direction(3, 0.0, 0.0), Vec3::NEG_Y);
        assert_eq!(direction(0, -1.0, 0.0), direction(4, 1.0, 0.0));
        assert_eq!(direction(4, 0.0, -1.0), direction(2, 0.0, 1.0));
    }
}
