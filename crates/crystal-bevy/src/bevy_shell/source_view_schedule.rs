//! Avoid worker-dispatch overhead for the small native source-view systems.

use bevy::app::{App, Last, PostUpdate, Update};
use bevy::ecs::schedule::{
    MultiThreadedExecutor, ScheduleLabel, Schedules, SingleThreadedExecutor,
};
use bevy::ecs::system::Local;
use bevy::ecs::world::World;

/// Update execution policy between frames, after the affected schedules finish.
pub(super) fn install(app: &mut App) {
    app.init_schedule(Update)
        .init_schedule(PostUpdate)
        .add_systems(Last, synchronize);
}

fn synchronize(world: &mut World, mut previous: Local<Option<bool>>) {
    #[cfg(feature = "voxel-view")]
    let source_view = world
        .get_resource::<crystal_voxel_view::VoxelViewSettings>()
        .is_some_and(|settings| !settings.enabled);
    #[cfg(not(feature = "voxel-view"))]
    let source_view = true;
    if *previous == Some(source_view) {
        return;
    }
    configure(world, source_view);
    *previous = Some(source_view);
}

fn configure(world: &mut World, source_view: bool) {
    let mut schedules = world.resource_mut::<Schedules>();
    for label in [Update.intern(), PostUpdate.intern()] {
        let schedule = schedules
            .get_mut(label)
            .expect("source view schedules were initialized during install");
        if source_view {
            schedule.set_executor(SingleThreadedExecutor::new());
        } else {
            schedule.set_executor(MultiThreadedExecutor::new());
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{
        App, Commands, Component, Local, MinimalPlugins, PostUpdate, Query, ResMut, Resource,
        Update,
    };

    #[derive(Component)]
    struct Generated(usize);

    #[derive(Resource, Default)]
    struct Observed(Vec<usize>);

    fn spawn(mut commands: Commands, mut sequence: Local<usize>) {
        *sequence += 1;
        commands.spawn(Generated(*sequence));
    }

    fn observe(generated: Query<&Generated>, mut observed: ResMut<Observed>) {
        observed.0.push(generated.iter().map(|value| value.0).sum());
    }

    #[test]
    fn executor_changes_preserve_deferred_commands_and_system_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Observed>()
            .add_systems(Update, spawn)
            .add_systems(PostUpdate, observe);
        for source in [true, false, true, false] {
            super::configure(app.world_mut(), source);
            app.update();
        }
        assert_eq!(app.world().resource::<Observed>().0, vec![1, 3, 6, 10]);
    }
}
