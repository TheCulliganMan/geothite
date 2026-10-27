//! Battle humans reuse the field's weighted rigs, paintings and preload queue.
use crate::battle_layout::{BattleBody, BattleSceneLayout, WORLD_UNITS_PER_METER};
use crate::new_bark_actors::character_meshes::CharacterKind;
use crate::new_bark_actors::{ModeledActor, ModeledActors, character_meshes};
use crate::open_people_skin::Person;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use crystal_render_api::VisualBattleFrame;

/// Explicit battle-art aliases for classes whose field costume has another name.
pub(crate) fn kind(source: &str) -> Option<CharacterKind> {
    use CharacterKind::*;
    character_meshes::SOURCE_KINDS
        .iter()
        .find_map(|&(id, kind)| (id == source).then_some(kind))
        .or_else(|| {
            Some(match source {
                "outdoorsman" | "hiker" => Outdoorsman,
                "rival1" | "rival2" => Rival,
                "lt_surge" => Surge,
                "champion" => Lance,
                "skier" => CooltrainerF,
                "pokefanm" | "pokemaniac" => PokefanM,
                "pokefanf" => PokefanF,
                "fisherman" => Fisher,
                "bird_keeper" | "boarder" | "juggler" | "psychic_t" => CooltrainerM,
                "firebreather" => PokefanM,
                "medium" => Granny,
                "schoolboy" => Youngster,
                "picnicker" => Lass,
                "camper" => BugCatcher,
                "guitarist" => Rocker,
                "burglar" => Biker,
                "grunts_m" | "grunt_m" | "executive_m" => Rocket,
                "grunts_f" | "grunt_f" | "executive_f" => RocketGirl,
                "swimmer_m" => SwimmerGuy,
                "swimmer_f" => SwimmerGirl,
                "pokefan_m" => PokefanM,
                "pokefan_f" => PokefanF,
                "twins" => Twin,
                "blackbelt_t" => BlackBelt,
                "pokemon_prof" => Oak,
                "mysticalman" => CooltrainerM,
                _ => return None,
            })
        })
}

/// A real human supplies the layout contact before a Pokémon is selected.
pub(super) fn body(source: &str) -> Option<BattleBody> {
    let look = crate::npc_appearance::look(kind(source)?);
    Some(BattleBody::modeled(
        "TRAINER",
        Vec3::new(-0.45, 0., -0.45),
        Vec3::new(0.45, look.height, 0.45),
        WORLD_UNITS_PER_METER,
    ))
}

/// Reserve trainer height throughout the encounter so narration cannot refit
/// the camera when a trainer replaces a Pokémon at the same checked contact.
pub(super) fn reserve(mut body: BattleBody, source: &str) -> BattleBody {
    let Some(kind) = kind(source) else {
        return body;
    };
    let look = crate::npc_appearance::look(kind);
    let radius = 0.45 * WORLD_UNITS_PER_METER;
    let min = Vec3::new(
        -radius / body.scale.x,
        body.support_y,
        -radius / body.scale.z,
    );
    let max = Vec3::new(
        radius / body.scale.x,
        body.support_y + look.height * WORLD_UNITS_PER_METER / body.scale.y,
        radius / body.scale.z,
    );
    let (old_min, old_max) = body.visual_bounds.unwrap_or((body.min, body.max));
    body.visual_bounds = Some((old_min.min(min), old_max.max(max)));
    // The trainer must be visible to the same static-sight proof as the Pokémon.
    let (old_min, old_max) = body.source_bank_bounds.unwrap_or((body.min, body.max));
    body.source_bank_bounds = Some((old_min.min(min), old_max.max(max)));
    body
}

struct Instance {
    root: Entity,
    kind: CharacterKind,
    person: Person,
}
#[derive(Resource, Default)]
pub(super) struct BattleTrainers {
    instances: [Option<Instance>; 2],
    pub ready: bool,
}

pub(super) fn prepare(
    frame: Res<VisualBattleFrame>,
    mut people: Option<ResMut<ModeledActors>>,
    mut state: ResMut<BattleTrainers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut binds: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let kinds = frame
        .trainers
        .iter()
        .flatten()
        .filter_map(|trainer| kind(&trainer.source_id))
        .collect::<Vec<_>>();
    state.ready = kinds.is_empty();
    if !frame.active || kinds.is_empty() {
        return;
    }
    let Some(people) = people.as_deref_mut() else {
        return;
    };
    #[cfg(target_arch = "wasm32")]
    if people.synchronous_battle_people() {
        for &kind in &kinds {
            let base = crate::npc_appearance::look(kind).base;
            if !people.people_gpu.0.contains_key(&(base, kind)) {
                let rig = crate::open_people::get(base).expect("required battle character rig");
                let appearance = crate::open_people_skin::prepare_appearance(rig, kind);
                people
                    .people_gpu
                    .2
                    .lock()
                    .expect("prepared actor lock")
                    .insert((base, kind), appearance);
            }
        }
    }
    // CPU work shares the field worker queue; publish at most one appearance
    // per update while the encounter remains hidden by its entry transition.
    state.ready = people.people_gpu.upload_next_prepared(
        &kinds,
        &mut meshes,
        &mut materials,
        &mut images,
        &mut binds,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync(
    mut commands: Commands,
    frame: Res<VisualBattleFrame>,
    status: Res<super::BattleViewStatus>,
    layout: Res<BattleSceneLayout>,
    mut state: ResMut<BattleTrainers>,
    mut people: Option<ResMut<ModeledActors>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut binds: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    time: Res<Time>,
    mut transforms: Query<&mut Transform, With<ModeledActor>>,
) {
    for index in 0..2 {
        let trainer = frame.trainers[index].as_ref().filter(|_| frame.active);
        let kind = trainer.and_then(|trainer| kind(&trainer.source_id));
        if state.instances[index]
            .as_ref()
            .is_some_and(|instance| Some(instance.kind) != kind)
        {
            let old = state.instances[index]
                .take()
                .expect("existing battle trainer");
            commands.entity(old.root).despawn();
        }
        let Some(kind) = kind else {
            continue;
        };
        // Preparation renders to the hidden battle image, warming the real
        // skin/material pipelines before the field wipe exposes this camera.
        let visible =
            (status.active || status.prepared) && trainer.is_some_and(|trainer| trainer.visible);
        let look = crate::npc_appearance::look(kind);
        let origin = layout.origins[index];
        let direction = layout.origins[1 - index] - origin;
        let rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
        let rig = if let Some(instance) = &state.instances[index] {
            instance.person.rig.clone()
        } else {
            if !state.ready {
                continue;
            }
            crate::open_people::get(look.base)
                .expect("preloaded battle character rig")
                .clone()
        };
        let scale = WORLD_UNITS_PER_METER * look.height / rig.height;
        let pose = Transform::from_translation(origin)
            .with_rotation(rotation)
            .with_scale(Vec3::new(scale * look.width, scale, scale));
        let visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if let Some(instance) = &mut state.instances[index] {
            commands.entity(instance.root).insert((pose, visibility));
            if visible {
                instance
                    .person
                    .update(0., 0., time.elapsed_secs(), &mut transforms);
            }
        } else if let Some(people) = people.as_deref_mut() {
            let root = commands.spawn((pose, visibility, ModeledActor)).id();
            let person = people.people_gpu.spawn_on_layer(
                look.base,
                kind,
                rig,
                root,
                false,
                false,
                None,
                super::BATTLE_LAYER,
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                &mut binds,
            );
            state.instances[index] = Some(Instance { root, kind, person });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battle_classes_resolve_field_costumes_without_sprite_cards() {
        for source in [
            "chris",
            "kris",
            "outdoorsman",
            "falkner",
            "hiker",
            "bird_keeper",
            "rival1",
            "rival2",
            "grunt_m",
            "grunt_f",
            "swimmer_m",
            "swimmer_f",
            "blackbelt_t",
            "medium",
            "schoolboy",
            "twins",
            "pokemon_prof",
        ] {
            assert!(
                kind(source).is_some(),
                "missing battle costume for {source}"
            );
        }
        assert_eq!(kind("hiker"), Some(CharacterKind::Outdoorsman));
        assert_eq!(kind("kris"), Some(CharacterKind::TrainerFemale));
        assert_eq!(kind("UNKNOWN"), None);
    }

    #[test]
    fn trainers_expand_only_the_visibility_envelope() {
        let body = BattleBody::modeled("CYNDAQUIL", Vec3::ZERO, Vec3::ONE, 0.5);
        let next = reserve(body, "falkner");
        assert_eq!(
            (body.min, body.max, body.scale, body.support_y),
            (next.min, next.max, next.scale, next.support_y)
        );
        assert!(
            next.visual_bounds.expect("trainer envelope").1.y * body.scale.y
                >= 1.38 * WORLD_UNITS_PER_METER
        );
        assert_eq!(reserve(next, "falkner"), next);
    }
    #[test]
    fn battle_humans_preload_hidden_and_keep_weighted_meshes_on_the_battle_layer() {
        use bevy::camera::visibility::RenderLayers;
        use crystal_render_api::VisualBattleTrainer;
        let mut app = super::super::tests::headless_battle_app();
        app.init_resource::<ModeledActors>();
        let look = crate::npc_appearance::look(CharacterKind::TrainerFemale);
        let rig = crate::open_people::get(look.base).expect("production rig");
        let appearance =
            crate::open_people_skin::prepare_appearance(rig, CharacterKind::TrainerFemale);
        app.world_mut()
            .resource_mut::<ModeledActors>()
            .people_gpu
            .2
            .lock()
            .expect("prepared actor lock")
            .insert((look.base, CharacterKind::TrainerFemale), appearance);
        {
            let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
            frame.preparing = true;
            frame.battlers[0] = None;
            frame.trainers[0] = Some(VisualBattleTrainer {
                source_id: "kris".into(),
                visible: true,
            });
        }
        app.update();
        let root = app.world().resource::<BattleTrainers>().instances[0]
            .as_ref()
            .expect("preloaded trainer")
            .root;
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("visibility"),
            Visibility::Inherited
        );
        let mut composites = app
            .world_mut()
            .query_filtered::<&Visibility, With<super::super::BattleCompositeSprite>>();
        assert!(
            composites
                .iter(app.world())
                .all(|visibility| *visibility == Visibility::Hidden),
            "preloaded trainer pipelines must not expose the battle over the retained field"
        );
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .preparing = false;
        app.update();
        assert!(
            app.world()
                .resource::<super::super::BattleViewStatus>()
                .active
        );
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("visibility"),
            Visibility::Inherited
        );
        let mut meshes = app
            .world_mut()
            .query_filtered::<&RenderLayers, (With<Mesh3d>, With<ModeledActor>)>();
        let layers = meshes.iter(app.world()).collect::<Vec<_>>();
        assert!(
            layers.len() >= 2,
            "weighted body and costume/accessories must be present"
        );
        assert!(
            layers
                .into_iter()
                .all(|layers| *layers == RenderLayers::layer(super::super::BATTLE_LAYER))
        );
        app.world_mut().resource_mut::<VisualBattleFrame>().trainers[0]
            .as_mut()
            .expect("trainer")
            .visible = false;
        app.update();
        assert_eq!(
            app.world().resource::<BattleTrainers>().instances[0]
                .as_ref()
                .expect("retained trainer")
                .root,
            root
        );
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("visibility"),
            Visibility::Hidden
        );
    }
    #[test]
    fn replacement_selection_retains_the_uploaded_pokemon_and_3d_scene() {
        use crystal_render_api::VisualBattleTrainer;
        let mut app = super::super::tests::headless_battle_app();
        app.init_resource::<ModeledActors>();
        let look = crate::npc_appearance::look(CharacterKind::Trainer);
        let rig = crate::open_people::get(look.base).expect("production rig");
        let appearance = crate::open_people_skin::prepare_appearance(rig, CharacterKind::Trainer);
        app.world_mut()
            .resource_mut::<ModeledActors>()
            .people_gpu
            .2
            .lock()
            .expect("prepared actor lock")
            .insert((look.base, CharacterKind::Trainer), appearance);
        app.world_mut().resource_mut::<VisualBattleFrame>().trainers[0] =
            Some(VisualBattleTrainer {
                source_id: "chris".into(),
                visible: false,
            });
        app.update();
        let entity = app.world().resource::<super::super::BattleScene>().actors[0]
            .as_ref()
            .expect("uploaded Pokemon")
            .entity;
        let pokemon = app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0].take();
        for _ in 0..3 {
            app.update();
            assert!(
                app.world()
                    .resource::<super::super::BattleViewStatus>()
                    .active
            );
            assert_eq!(
                *app.world().get::<Visibility>(entity).expect("retained rig"),
                Visibility::Hidden
            );
            assert_eq!(
                app.world().resource::<super::super::BattleScene>().actors[0]
                    .as_ref()
                    .expect("retained Pokemon")
                    .entity,
                entity
            );
        }
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0] = pokemon;
        app.update();
        assert!(
            app.world()
                .resource::<super::super::BattleViewStatus>()
                .active
        );
        assert_eq!(
            app.world().resource::<super::super::BattleScene>().actors[0]
                .as_ref()
                .expect("same Pokemon")
                .entity,
            entity
        );
        assert_eq!(
            *app.world().get::<Visibility>(entity).expect("visible rig"),
            Visibility::Visible
        );
    }
}
