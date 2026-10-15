/// Presentation-only trail in map pixels. Camera scrolling cannot move it,
/// and it never registers a gameplay object or participates in collision.
#[derive(Default)]
struct PartyFollowerTrail {
    map: String,
    points: std::collections::VecDeque<Vec2>,
    facing: Vec2,
}

impl PartyFollowerTrail {
    fn follow(&mut self, map: &str, leader: Vec2, facing: Vec2, gap: f32) -> Vec2 {
        let reset = self.map != map
            || self
                .points
                .back()
                .is_none_or(|last| last.distance(leader) > gap * 2.5);
        if reset {
            self.map = map.to_owned();
            self.points.clear();
            self.facing = facing;
            self.points.push_back(leader - facing * gap);
            self.points.push_back(leader);
        } else if self
            .points
            .back()
            .is_some_and(|last| last.distance(leader) > 0.001)
        {
            self.points.push_back(leader);
        }
        // Walk backward along the actual path, retaining one tile of history.
        let mut remaining = gap;
        let mut target = self.points[0];
        let mut segment = 0;
        for index in (1..self.points.len()).rev() {
            let from = self.points[index - 1];
            let to = self.points[index];
            let length = from.distance(to);
            if length > 0.001 && remaining <= length {
                target = to.lerp(from, remaining / length);
                self.facing = (to - from) / length;
                segment = index - 1;
                break;
            }
            remaining -= length;
        }
        for _ in 0..segment {
            self.points.pop_front();
        }
        self.points[0] = target;
        target
    }
}

#[derive(Default)]
struct PartyFollowerPresentation {
    trail: PartyFollowerTrail,
    art: Option<(
        String,
        bool,
        SpriteFrame,
        Option<crystal_render_api::VisualPokemonModelPalette>,
    )>,
}

fn readonly_companion_snapshot(shell: &BevyRuntimeShell) -> Result<Arc<RuntimeShellSnapshot>> {
    if let Some((revision, snapshot)) = shell.cached_snapshot.as_ref()
        && *revision == shell.snapshot_revision
    {
        return Ok(Arc::clone(snapshot));
    }
    Ok(Arc::new(shell.shell.presentation_snapshot()?))
}

fn publish_party_follower(
    settings: Res<crystal_voxel_view::VoxelViewSettings>,
    rendered: Res<RenderedViewport>,
    shell: Res<BevyRuntimeShell>,
    mut frame: ResMut<crystal_render_api::VisualWorldFrame>,
    mut images: ResMut<Assets<Image>>,
    mut follower: Local<PartyFollowerPresentation>,
) {
    use crystal_render_api::{VisualActor, VisualActorId};
    if !settings.enabled {
        follower.trail = PartyFollowerTrail::default();
        return;
    }
    if !frame.active {
        return;
    }
    let Ok(snapshot) = readonly_companion_snapshot(&shell) else {
        return;
    };
    let Some(lead) = snapshot.party.slots.first() else {
        follower.trail = PartyFollowerTrail::default();
        if frame
            .actors
            .iter()
            .any(|actor| actor.id == VisualActorId::PartyFollower)
        {
            frame
                .actors
                .retain(|actor| actor.id != VisualActorId::PartyFollower);
        }
        return;
    };
    let Some(player) = frame.actors.iter().find(|a| a.id == VisualActorId::Player) else {
        return;
    };
    let facing = player.facing.unwrap_or(Vec2::NEG_Y);
    let scroll = frame.center
        - Vec2::new(
            frame.grid_origin.x as f32 * frame.tile_size.x,
            -frame.grid_origin.y as f32 * frame.tile_size.y,
        );
    let leader = crystal_voxel_view::actor_foot(player) - scroll;
    let gap = frame.tile_size.y * 2.0;
    // Keep recording while the lead is the mount. A newly selected lead
    // inherits the actual water trail instead of cutting from the old shore.
    let foot = follower.trail.follow(&frame.map_id, leader, facing, gap) + scroll;
    if matches!(
        rendered.player_sprite_mode,
        Some(MovementMode::Surf | MovementMode::SurfPika)
    ) && visible_lead_is_surf_actor(&snapshot)
    {
        if frame
            .actors
            .iter()
            .any(|a| a.id == VisualActorId::PartyFollower)
        {
            frame
                .actors
                .retain(|a| a.id != VisualActorId::PartyFollower);
        }
        return;
    }
    let species = if lead.pokemon.is_egg {
        "EGG"
    } else {
        &lead.pokemon.species.id
    };
    let asset_id = pokemon_asset_id_for_dvs(species, lead.pokemon.dvs);
    let shiny = visible_pokemon_is_shiny(&lead.pokemon);
    if follower
        .art
        .as_ref()
        .is_none_or(|(id, appearance, _, _)| id != &asset_id || *appearance != shiny)
    {
        let Ok(texture) = load_pokemon_frame(
            &shell.asset_root,
            &asset_id,
            PokemonSpriteSide::Front,
            shiny,
            &mut images,
        ) else {
            return;
        };
        let palette = if shiny {
            let (Ok(normal), Ok(appearance)) = (
                load_pokemon_palette(
                    &shell.asset_root,
                    &asset_id,
                    PokemonSpriteSide::Front,
                    false,
                ),
                load_pokemon_palette(&shell.asset_root, &asset_id, PokemonSpriteSide::Front, true),
            ) else {
                return;
            };
            Some(crystal_render_api::VisualPokemonModelPalette { normal, appearance })
        } else {
            None
        };
        follower.art = Some((asset_id, shiny, texture, palette));
    }
    let (_, _, texture, palette) = follower.art.as_ref().unwrap();
    // Exact models use the existing species resolver; every other species
    // keeps its own pack front sprite as the ordinary card fallback.
    let actor = VisualActor {
        id: VisualActorId::PartyFollower,
        source_id: Arc::from(format!("species:{species}:icon_monster")),
        model_palette: *palette,
        texture: texture.handle.clone(),
        center: foot + Vec2::Y * gap * 0.5,
        size: Vec2::splat(gap),
        flip_x: false,
        above_priority: false,
        facing: Some(follower.trail.facing),
    };
    if let Some(index) = frame.actors.iter().position(|a| a.id == actor.id) {
        if frame.actors[index] != actor {
            frame.actors[index] = actor;
        }
    } else {
        frame.actors.push(actor);
    }
}

#[cfg(test)]
mod party_follower_tests {
    use super::*;

    #[test]
    fn party_follower_tracks_corners_stops_and_resets_on_warps() {
        let mut trail = PartyFollowerTrail::default();
        assert_eq!(
            trail.follow("Route", Vec2::ZERO, Vec2::X, 16.),
            Vec2::new(-16., 0.)
        );
        assert_eq!(
            trail.follow("Route", Vec2::new(16., 0.), Vec2::X, 16.),
            Vec2::ZERO
        );
        assert_eq!(
            trail.follow("Route", Vec2::new(16., 8.), Vec2::Y, 16.),
            Vec2::new(8., 0.)
        );
        assert_eq!(trail.facing, Vec2::X);
        assert_eq!(
            trail.follow("Route", Vec2::new(16., 16.), Vec2::Y, 16.),
            Vec2::new(16., 0.)
        );
        assert_eq!(
            trail.follow("Route", Vec2::new(16., 16.), Vec2::NEG_X, 16.),
            Vec2::new(16., 0.)
        );
        assert_eq!(
            trail.follow("House", Vec2::new(32., 32.), Vec2::NEG_Y, 16.),
            Vec2::new(32., 48.)
        );
        assert_eq!(
            trail.follow("House", Vec2::new(320., 32.), Vec2::X, 16.),
            Vec2::new(304., 32.)
        );
    }

    #[test]
    fn party_follower_history_stays_bounded_over_long_walks() {
        let mut trail = PartyFollowerTrail::default();
        for x in 0..10000 {
            let leader = Vec2::new(x as f32, 0.);
            assert_eq!(
                trail.follow("Route", leader, Vec2::X, 16.),
                leader - Vec2::X * 16.
            );
            assert!(trail.points.len() <= 18);
        }
    }
}
