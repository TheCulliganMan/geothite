fn visible_surf_actor(snapshot: &RuntimeShellSnapshot) -> Option<(String, Dv)> {
    use crate::core::systems::field_moves::{SURF_ACTOR_DVS_MEMORY, SURF_ACTOR_SPECIES_MEMORY};
    let saved_dvs = snapshot
        .script_events
        .memory
        .get(SURF_ACTOR_DVS_MEMORY)
        .and_then(|value| serde_json::from_str::<Dv>(value).ok());
    if let Some(slot) = snapshot
        .script_events
        .memory
        .get(crate::core::systems::field_moves::SURF_ACTOR_PARTY_MEMORY)
        .and_then(|value| value.parse::<usize>().ok())
        .and_then(|index| snapshot.party.slots.iter().find(|slot| slot.index == index))
        .filter(|slot| Some(slot.pokemon.dvs) == saved_dvs)
    {
        return Some((slot.pokemon.species.id.clone(), slot.pokemon.dvs));
    }
    if let (Some(species), Some(dvs)) = (
        snapshot.script_events.memory.get(SURF_ACTOR_SPECIES_MEMORY),
        snapshot.script_events.memory.get(SURF_ACTOR_DVS_MEMORY),
    ) {
        if let Ok(dvs) = serde_json::from_str(dvs) {
            return Some((species.clone(), dvs));
        }
    }
    // Older saves predate the retained identity. Use the source's party
    // selection if it still knows Surf, otherwise its first Surf user.
    let selected = snapshot
        .script_events
        .memory
        .get("wCurPartyMon")
        .and_then(|value| value.parse::<usize>().ok());
    let can_surf = |slot: &&crate::RuntimePartySlotSnapshot| {
        !slot.pokemon.is_egg && slot.pokemon.moves.iter().any(|m| m.name == "SURF")
    };
    snapshot
        .party
        .slots
        .iter()
        .filter(can_surf)
        .find(|slot| Some(slot.index) == selected)
        .or_else(|| snapshot.party.slots.iter().find(can_surf))
        .map(|slot| (slot.pokemon.species.id.clone(), slot.pokemon.dvs))
}

fn visible_lead_is_surf_actor(snapshot: &RuntimeShellSnapshot) -> bool {
    let Some(lead) = snapshot.party.slots.first() else {
        return false;
    };
    if let Some(index) = snapshot
        .script_events
        .memory
        .get(crate::core::systems::field_moves::SURF_ACTOR_PARTY_MEMORY)
        .and_then(|value| value.parse::<usize>().ok())
    {
        return lead.index == index;
    }
    visible_surf_actor(snapshot)
        .is_some_and(|(species, dvs)| lead.pokemon.species.id == species && lead.pokemon.dvs == dvs)
}

fn publish_surf_mount(
    settings: Res<crystal_voxel_view::VoxelViewSettings>,
    rendered: Res<RenderedViewport>,
    shell: Res<BevyRuntimeShell>,
    mut frame: ResMut<crystal_render_api::VisualWorldFrame>,
    mut images: ResMut<Assets<Image>>,
    mut rider: ResMut<crystal_render_api::VisualSurfRider>,
    mut art: Local<PartyFollowerPresentation>,
) {
    use crystal_render_api::{VisualActor, VisualActorId};
    if !settings.enabled || !frame.active {
        return;
    }
    if !matches!(
        rendered.player_sprite_mode,
        Some(MovementMode::Surf | MovementMode::SurfPika)
    ) {
        if frame
            .actors
            .iter()
            .any(|a| a.id == VisualActorId::SurfMount)
        {
            frame.actors.retain(|a| a.id != VisualActorId::SurfMount);
        }
        return;
    }
    let Ok(snapshot) = readonly_companion_snapshot(&shell) else {
        return;
    };
    let Some((species, dvs)) = visible_surf_actor(&snapshot) else {
        return;
    };
    let Some(index) = frame
        .actors
        .iter()
        .position(|a| a.id == VisualActorId::Player)
    else {
        return;
    };
    let player = &frame.actors[index];
    let foot = crystal_voxel_view::actor_foot(player);
    let facing = player.facing;
    let asset_id = pokemon_asset_id_for_dvs(&species, dvs);
    let shiny = visible_dvs_are_shiny(dvs);
    if art
        .art
        .as_ref()
        .is_none_or(|(id, appearance, _, _)| *id != asset_id || *appearance != shiny)
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
        art.art = Some((asset_id, shiny, texture, palette));
    }
    let (_, _, texture, palette) = art.art.as_ref().unwrap();
    let size = Vec2::splat(frame.tile_size.y * 2.0);
    let mount = VisualActor {
        id: VisualActorId::SurfMount,
        source_id: Arc::from(format!("species:{species}:icon_monster")),
        model_palette: *palette,
        texture: texture.handle.clone(),
        center: foot + Vec2::Y * size.y * 0.5,
        size,
        flip_x: false,
        above_priority: false,
        facing,
    };
    let female = snapshot.trainer.player_gender == PLAYER_GENDER_FEMALE;
    if rider.female != female {
        rider.female = female;
    }
    if let Some(index) = frame
        .actors
        .iter()
        .position(|a| a.id == VisualActorId::SurfMount)
    {
        if frame.actors[index] != mount {
            frame.actors[index] = mount;
        }
    } else {
        frame.actors.push(mount);
    }
    // A Pokémon carrying the player cannot also be a second copy behind them.
    if visible_lead_is_surf_actor(&snapshot)
        && frame
            .actors
            .iter()
            .any(|a| a.id == VisualActorId::PartyFollower)
    {
        frame
            .actors
            .retain(|a| a.id != VisualActorId::PartyFollower);
    }
}
