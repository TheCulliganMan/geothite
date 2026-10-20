/// Presentation-only trail in map pixels. Camera scrolling cannot move it,
/// and it never registers a gameplay object or participates in collision.
#[derive(Default)]
struct PartyFollowerTrail {
    map: String,
    points: std::collections::VecDeque<Vec2>,
    facing: Vec2,
}

impl PartyFollowerTrail {
    fn needs_reset(&self, map: &str, leader: Vec2, step: f32) -> bool {
        self.map != map
            || self
                .points
                .back()
                .is_none_or(|last| last.distance(leader) > step * 2.5)
    }

    fn follow(
        &mut self,
        map: &str,
        leader: Vec2,
        facing: Vec2,
        gap: f32,
        step: f32,
        seed_clear: bool,
    ) -> Option<Vec2> {
        if self.needs_reset(map, leader, step) {
            self.map = map.to_owned();
            self.points.clear();
            self.facing = facing;
            if seed_clear {
                self.points.push_back(leader - facing * gap);
            }
            self.points.push_back(leader);
        } else if self
            .points
            .back()
            .is_some_and(|last| last.distance(leader) > 0.001)
        {
            self.points.push_back(leader);
        }
        // Walk backward along the actual path, retaining the full body gap.
        let mut remaining = gap;
        let mut target = self.points[0];
        let mut segment = 0;
        let mut has_gap = gap <= 0.001;
        for index in (1..self.points.len()).rev() {
            let from = self.points[index - 1];
            let to = self.points[index];
            let length = from.distance(to);
            // Trimming to a fractional body gap rounds map-pixel coordinates.
            // A stationary trail may then be a fraction of a pixel shorter
            // than the requested distance. Stay on its endpoint, never extend
            // beyond the recorded path or hide an otherwise settled follower.
            if length > 0.001 && remaining <= length + 0.001 {
                target = to.lerp(from, (remaining / length).clamp(0., 1.));
                self.facing = (to - from) / length;
                segment = index - 1;
                has_gap = true;
                break;
            }
            remaining -= length;
        }
        // A blocked spawn has no invented path through furniture. Keep the
        // real history until a complete trailing gap has been walked.
        if !has_gap {
            return None;
        }
        for _ in 0..segment {
            self.points.pop_front();
        }
        self.points[0] = target;
        Some(target)
    }
}

fn party_follower_seed_is_clear(
    overworld: &crystal_core::world::session::OverworldSession,
    steps: u32,
) -> bool {
    use crystal_core::world::movement::{
        StepOptions, StepOutcome, attempt_step_with_occupied_tiles,
    };
    let behind = match overworld.player.facing {
        Direction::Down => Direction::Up,
        Direction::Up => Direction::Down,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    };
    let Ok(occupied) = overworld.occupied_tiles_checked() else {
        return false;
    };
    // Probe a copy with the production movement predicate. No session state,
    // object position, script, or input is advanced by this placement check.
    let mut probe = overworld.player;
    (0..steps).all(|_| matches!(attempt_step_with_occupied_tiles(
        &mut probe, behind, &overworld.map, &overworld.tileset,
        StepOptions { force_step_after_turn: true, ..Default::default() }, &occupied,
    ), StepOutcome::Moved { to, .. } if crystal_core::world::collision::sample_collision(
        &overworld.map, &overworld.tileset, to).is_some()))
}

#[derive(Default)]
struct PartyFollowerPresentation {
    trail: PartyFollowerTrail,
    trace_reason: Option<String>,
    art: Option<(
        String,
        bool,
        SpriteFrame,
        Option<crystal_render_api::VisualPokemonModelPalette>,
    )>,
}

impl PartyFollowerPresentation {
    fn trace(&mut self, reason: impl FnOnce() -> String) {
        if std::env::var_os("CRYSTAL_POKEMON_FOLLOW_TRACE").is_none() { return; }
        let reason = reason();
        if self.trace_reason.as_ref() != Some(&reason) {
            eprintln!("pokemon follower: {reason}");
            self.trace_reason = Some(reason);
        }
    }
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
    dimensions: Option<Res<crystal_render_api::VisualPokemonDimensions>>,
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
    let sprite_size = frame.tile_size.y * 2.0;
    let species = if lead.pokemon.is_egg { "EGG" } else { &lead.pokemon.species.id };
    let gap = dimensions.as_deref().and_then(|dimensions|
        crystal_voxel_view::pokemon_following_distance(&frame, species, dimensions))
        .unwrap_or(sprite_size);
    // Keep recording while the lead is the mount. A newly selected lead
    // inherits the actual water trail instead of cutting from the old shore.
    let overworld = shell.shell.session().overworld();
    let seed_clear = !follower.trail.needs_reset(&frame.map_id, leader, sprite_size)
        || (overworld.map.name == frame.map_id.as_ref() && party_follower_seed_is_clear(overworld,
            (gap / sprite_size).ceil() as u32));
    let Some(foot) = follower
        .trail
        .follow(&frame.map_id, leader, facing, gap, sprite_size, seed_clear)
    else {
        follower.trace(|| format!("{species}: waiting for {gap:.3}px of clear trail (seed_clear={seed_clear})"));
        frame
            .actors
            .retain(|actor| actor.id != VisualActorId::PartyFollower);
        return;
    };
    let foot = foot + scroll;
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
    let asset_id = pokemon_asset_id_for_dvs(species, lead.pokemon.dvs);
    let shiny = visible_pokemon_is_shiny(&lead.pokemon);
    if follower
        .art
        .as_ref()
        .is_none_or(|(id, appearance, _, _)| id != &asset_id || *appearance != shiny)
    {
        let texture = match load_pokemon_frame(
            &shell.asset_root,
            &asset_id,
            PokemonSpriteSide::Front,
            shiny,
            &mut images,
        ) {
            Ok(texture) => texture,
            Err(error) => {
                follower.trace(|| format!("{species}: cannot load {asset_id}: {error:#}"));
                return;
            }
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
    follower.trace(|| format!("{species}: published with {gap:.3}px trail gap"));
    let (_, _, texture, palette) = follower.art.as_ref().unwrap();
    // Exact models use the existing species resolver; every other species
    // keeps its own pack front sprite as the ordinary card fallback.
    let actor = VisualActor {
        id: VisualActorId::PartyFollower,
        source_id: Arc::from(format!("species:{species}:icon_monster")),
        model_palette: *palette,
        texture: texture.handle.clone(),
        center: foot + Vec2::Y * sprite_size * 0.5,
        size: Vec2::splat(sprite_size),
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
            trail.follow("Route", Vec2::ZERO, Vec2::X, 16., 16., true).unwrap(),
            Vec2::new(-16., 0.)
        );
        assert_eq!(
            trail.follow("Route", Vec2::new(16., 0.), Vec2::X, 16., 16., true).unwrap(),
            Vec2::ZERO
        );
        assert_eq!(
            trail.follow("Route", Vec2::new(16., 8.), Vec2::Y, 16., 16., true).unwrap(),
            Vec2::new(8., 0.)
        );
        assert_eq!(trail.facing, Vec2::X);
        assert_eq!(
            trail.follow("Route", Vec2::new(16., 16.), Vec2::Y, 16., 16., true).unwrap(),
            Vec2::new(16., 0.)
        );
        assert_eq!(
            trail.follow("Route", Vec2::new(16., 16.), Vec2::NEG_X, 16., 16., true).unwrap(),
            Vec2::new(16., 0.)
        );
        assert_eq!(
            trail.follow("House", Vec2::new(32., 32.), Vec2::NEG_Y, 16., 16., true).unwrap(),
            Vec2::new(32., 48.)
        );
        assert_eq!(
            trail.follow("House", Vec2::new(320., 32.), Vec2::X, 16., 16., true).unwrap(),
            Vec2::new(304., 32.)
        );
    }

    #[test]
    fn party_follower_history_stays_bounded_over_long_walks() {
        let mut trail = PartyFollowerTrail::default();
        for x in 0..10000 {
            let leader = Vec2::new(x as f32, 0.);
            assert_eq!(
                trail.follow("Route", leader, Vec2::X, 16., 16., true).unwrap(),
                leader - Vec2::X * 16.
            );
            assert!(trail.points.len() <= 18);
        }
    }

    #[test]
    fn party_follower_larger_body_waits_for_history_and_does_not_mask_teleports() {
        let mut trail = PartyFollowerTrail::default();
        assert_eq!(trail.follow("Route", Vec2::ZERO, Vec2::X, 16., 16., true), Some(-Vec2::X * 16.));
        // Switching to a larger animal must not invent an older path.
        assert_eq!(trail.follow("Route", Vec2::X * 8., Vec2::X, 48., 16., true), None);
        assert_eq!(trail.follow("Route", Vec2::X * 24., Vec2::X, 48., 16., true), None);
        assert_eq!(trail.follow("Route", Vec2::X * 32., Vec2::X, 48., 16., true), Some(-Vec2::X * 16.));
        // Reset uses gameplay step distance, not this animal's longer gap.
        assert_eq!(trail.follow("Route", Vec2::X * 80., Vec2::X, 48., 16., false), None);
        assert_eq!(trail.follow("Route", Vec2::X * 96., Vec2::X, 48., 16., true), None);
    }

    #[test]
    fn party_follower_fractional_body_gap_stays_visible_when_the_trainer_stops() {
        for gap in [19.37,78.12345,156.27] {
            let mut trail=PartyFollowerTrail::default();
            let start=Vec2::new(1280.123,-1024.321);
            assert!(trail.follow("Route",start,Vec2::Y,gap,64.,true).is_some());
            for tick in 1..100 {
                let leader=start+Vec2::Y*(tick as f32*0.73);
                let foot=trail.follow("Route",leader,Vec2::Y,gap,64.,true).expect("recorded fractional trail remains complete");
                for _ in 0..4 {
                    let stopped=trail.follow("Route",leader,Vec2::Y,gap,64.,true).expect("stopping must not hide the companion");
                    assert!(stopped.distance(foot)<0.001);
                }
            }
        }
    }
}
