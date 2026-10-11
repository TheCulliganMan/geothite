//! Native, opt-in diagrams extracted from the real app and its read-only presentation cache.

use std::{collections::BTreeSet, fmt::Write as _, path::PathBuf};

use bevy::{
    dev_tools::schedule_data::serde::{AppData, ScheduleData, ScheduleIndex},
    ecs::schedule::{InternedScheduleLabel, ScheduleBuildMetadata, ScheduleBuilt, Schedules},
    platform::collections::HashMap,
    prelude::*,
    render::{ExtractSchedule, RenderApp},
};

#[derive(Resource)]
pub(super) struct DiagramOutput {
    pub(super) directory: PathBuf,
    prefix: &'static str,
}

#[derive(Resource, Default)]
struct BuildData {
    metadata: HashMap<InternedScheduleLabel, ScheduleBuildMetadata>,
    revision: u64,
    exported_revision: Option<u64>,
}

fn remember_build(event: On<ScheduleBuilt>, mut builds: ResMut<BuildData>) {
    builds.metadata.insert(
        event.label,
        ScheduleBuildMetadata {
            warnings: Vec::new(),
            edges_added_by_build_passes: event.build_metadata.edges_added_by_build_passes.clone(),
        },
    );
    builds.revision += 1;
}

pub(super) fn install(app: &mut App) {
    let Some(directory) = std::env::var_os("CRYSTAL_BEVY_DIAGRAMS") else {
        return;
    };
    install_at(app, PathBuf::from(directory));
}

fn install_at(app: &mut App, directory: PathBuf) {
    app.insert_resource(DiagramOutput {
        directory: directory.clone(),
        prefix: "main",
    })
    .init_resource::<BuildData>()
    .add_observer(remember_build)
    .init_resource::<ObservedFlow>()
    .add_systems(Main, export_schedules.before(Main::run_main))
    .add_systems(Last, super::observe_controller_flow);
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.insert_resource(DiagramOutput {
            directory,
            prefix: "render",
        });
        render
            .init_resource::<BuildData>()
            .add_observer(remember_build);
        // Extract runs outside Render, so the actual Render schedule and any
        // dynamically created per-view schedules remain available to inspect.
        render.add_systems(ExtractSchedule, export_schedules);
    }
}

// Like Bevy's SerializeSchedulesPlugin, collect before the schedules execute,
// retaining metadata for deferred sync points inserted by schedule build passes.
fn collect_schedules(world: &mut World) -> anyhow::Result<AppData> {
    let labels: Vec<_> = world
        .resource::<Schedules>()
        .iter()
        .map(|(_, schedule)| schedule.label())
        .collect();
    for label in labels {
        let built = world.schedule_scope(label, |world, schedule| schedule.initialize(world))?;
        if let Some(built) = built {
            world
                .resource_mut::<BuildData>()
                .metadata
                .insert(label, built);
        }
    }
    Ok(AppData::from_schedules(
        world.resource::<Schedules>(),
        world.components(),
        &world.resource::<BuildData>().metadata,
    )?)
}

fn export_schedules(world: &mut World) {
    let builds = world.resource::<BuildData>();
    let refresh = world
        .get_resource::<ButtonInput<KeyCode>>()
        .is_some_and(|keys| keys.just_pressed(KeyCode::F7));
    if builds.exported_revision == Some(builds.revision) && !refresh {
        return;
    }
    let result = (|| -> anyhow::Result<()> {
        let mut data = collect_schedules(world)?;
        data.schedules.sort_by(|a, b| a.name.cmp(&b.name));
        let output = world.resource::<DiagramOutput>();
        std::fs::create_dir_all(&output.directory)?;
        std::fs::write(
            output
                .directory
                .join(format!("{}-schedules.json", output.prefix)),
            serde_json::to_vec_pretty(&data)?,
        )?;
        for (index, schedule) in data.schedules.iter().enumerate() {
            // IDs make filenames safe even for parameterized schedule labels.
            let stem = format!("{}-{index:03}", output.prefix);
            std::fs::write(
                output.directory.join(format!("{stem}-order.dot")),
                ordering_dot(schedule),
            )?;
            std::fs::write(
                output.directory.join(format!("{stem}-access.dot")),
                access_dot(schedule),
            )?;
            std::fs::write(
                output.directory.join(format!("{stem}-order.md")),
                format!(
                    "# {}\n\n```mermaid\n{}```\n",
                    schedule.name,
                    ordering_mermaid(schedule)
                ),
            )?;
        }
        let mut index = String::from(
            "# Bevy schedules\n\nSolid arrows are ordering constraints; dashed arrows are set membership. Access DOT graphs show reads, writes and conflicts. The JSON retains query filters and inverted-access exclusions.\n\n",
        );
        for (i, schedule) in data.schedules.iter().enumerate() {
            writeln!(index, "- [{}]({}-{:03}-order.md) · [ordering DOT]({}-{:03}-order.dot) · [access DOT]({}-{:03}-access.dot)", schedule.name, output.prefix, i, output.prefix, i, output.prefix, i).unwrap();
        }
        std::fs::write(
            output.directory.join(format!("{}-index.md", output.prefix)),
            index,
        )?;
        Ok(())
    })();
    if let Err(error) = result {
        warn!("Bevy diagram export failed: {error:#}");
    }
    let mut builds = world.resource_mut::<BuildData>();
    builds.exported_revision = Some(builds.revision);
}

fn quoted(text: &str) -> String {
    serde_json::to_string(text).expect("strings serialize")
}

fn node(index: ScheduleIndex) -> String {
    match index {
        ScheduleIndex::System(i) => format!("s{i}"),
        ScheduleIndex::SystemSet(i) => format!("g{i}"),
    }
}

fn ordering_mermaid(data: &ScheduleData) -> String {
    let escape = |label: &str| {
        label
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\n', "<br/>")
    };
    let mut graph = String::from("flowchart LR\n");
    for (i, system) in data.systems.iter().enumerate() {
        writeln!(graph, "    s{i}[\"{}\"]", escape(&system.name)).unwrap();
    }
    for (i, set) in data.system_sets.iter().enumerate() {
        writeln!(graph, "    g{i}([\"{}\"])", escape(&set.name)).unwrap();
    }
    for (before, after) in &data.dependency {
        writeln!(graph, "    {} --> {}", node(*before), node(*after)).unwrap();
    }
    for (parent, child) in &data.hierarchy {
        writeln!(graph, "    g{} -. contains .-> {}", parent.0, node(*child)).unwrap();
    }
    graph
}

fn ordering_dot(data: &ScheduleData) -> String {
    let mut dot = format!(
        "digraph schedule {{\n  rankdir=LR;\n  label={};\n",
        quoted(&data.name)
    );
    for (i, system) in data.systems.iter().enumerate() {
        let shape = if system.apply_deferred {
            "diamond"
        } else {
            "box"
        };
        writeln!(
            dot,
            "  s{i} [label={}, shape={shape}];",
            quoted(&system.name)
        )
        .unwrap();
    }
    for (i, set) in data.system_sets.iter().enumerate() {
        let conditions = set
            .conditions
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let label = if conditions.is_empty() {
            set.name.clone()
        } else {
            format!("{}\nif {conditions}", set.name)
        };
        writeln!(dot, "  g{i} [label={}, shape=ellipse];", quoted(&label)).unwrap();
    }
    for (before, after) in &data.dependency {
        writeln!(dot, "  {} -> {};", node(*before), node(*after)).unwrap();
    }
    for (parent, child) in &data.hierarchy {
        writeln!(
            dot,
            "  g{} -> {} [style=dashed, label=\"contains\"] ;",
            parent.0,
            node(*child)
        )
        .unwrap();
    }
    dot.push_str("}\n");
    dot
}

fn access_dot(data: &ScheduleData) -> String {
    let mut dot = format!(
        "digraph access {{\n  rankdir=LR;\n  label={};\n",
        quoted(&format!("{}: component/resource access", data.name))
    );
    for (i, component) in data.components.iter().enumerate() {
        writeln!(
            dot,
            "  c{i} [label={}, shape=ellipse];",
            quoted(&component.name)
        )
        .unwrap();
    }
    for (i, system) in data.systems.iter().enumerate() {
        writeln!(dot, "  s{i} [label={}, shape=box];", quoted(&system.name)).unwrap();
        if system.exclusive {
            writeln!(
                dot,
                "  s{i} -> world [label=\"exclusive world access\", color=orange];"
            )
            .unwrap();
        }
        let mut edges = BTreeSet::new();
        for access in &system.filtered_accesses {
            for component in &access.access.reads {
                edges.insert(format!("  c{component} -> s{i} [label=\"read\"];\n"));
            }
            for component in &access.access.writes {
                edges.insert(format!(
                    "  s{i} -> c{component} [label=\"write\", color=blue];\n"
                ));
            }
            // Inverted access means 'all except'; never mislabel it as no access.
            if access.access.reads_inverted || access.access.writes_inverted {
                writeln!(
                    dot,
                    "  s{i} -> world [label={}, color=orange];",
                    quoted(&format!(
                        "inverted access: read={}, write={}; exclusions in JSON",
                        access.access.reads_inverted, access.access.writes_inverted
                    ))
                )
                .unwrap();
            }
        }
        for edge in edges {
            dot.push_str(&edge);
        }
    }
    for conflict in &data.conflicts {
        writeln!(
            dot,
            "  s{} -> s{} [dir=both, style=dotted, color=red, label={}];",
            conflict.system_1,
            conflict.system_2,
            quoted(&format!("conflict: {:?}", conflict.conflicting_access))
        )
        .unwrap();
    }
    dot.push_str("}\n");
    dot
}

#[derive(Resource, Default)]
pub(super) struct ObservedFlow {
    previous: Option<&'static str>,
    nodes: BTreeSet<&'static str>,
    edges: BTreeSet<(&'static str, &'static str)>,
}

impl ObservedFlow {
    pub(super) fn observe(&mut self, state: &'static str) -> bool {
        let mut changed = self.nodes.insert(state);
        if let Some(previous) = self.previous
            && previous != state
        {
            changed |= self.edges.insert((previous, state));
        }
        self.previous = Some(state);
        changed
    }

    pub(super) fn mermaid(&self) -> String {
        let mut graph = String::from("stateDiagram-v2\n");
        for state in &self.nodes {
            writeln!(graph, "    {state}").unwrap();
        }
        for (before, after) in &self.edges {
            writeln!(graph, "    {before} --> {after}").unwrap();
        }
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct Counter(u32);
    fn write_counter(mut value: ResMut<Counter>) {
        value.0 += 1;
    }
    fn read_counter(value: Res<Counter>) {
        assert_eq!(value.0, 1);
    }

    #[test]
    fn installed_exporter_writes_main_and_render_graphs_and_refreshes_after_schedule_changes() {
        use bevy::{app::SubApp, render::Render};
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let scratch = Scratch(std::env::temp_dir().join(format!(
                "geothite-diagrams-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
        let mut app = App::new();
        app.insert_sub_app(RenderApp, SubApp::new());
        fn first() {}
        fn later() {}
        app.add_systems(Update, first);
        app.sub_app_mut(RenderApp).add_systems(Render, first);
        install_at(&mut app, scratch.0.clone());
        app.update();
        let read = |prefix: &str| -> AppData {
            serde_json::from_slice(
                &std::fs::read(scratch.0.join(format!("{prefix}-schedules.json"))).unwrap(),
            )
            .unwrap()
        };
        assert!(read("main").schedules.iter().any(|schedule| {
            schedule.name == "Update"
                && schedule
                    .systems
                    .iter()
                    .any(|system| system.name.ends_with("first"))
        }));
        app.sub_app_mut(RenderApp)
            .world_mut()
            .run_schedule(ExtractSchedule);
        assert!(
            read("render")
                .schedules
                .iter()
                .any(|schedule| schedule.name == "Render")
        );
        app.add_systems(Update, later.after(first));
        app.update();
        // The changed Update schedule builds after this frame's Main export;
        // its ScheduleBuilt notification refreshes the following frame.
        app.update();
        assert!(read("main").schedules.iter().any(|schedule| {
            schedule.name == "Update"
                && schedule
                    .systems
                    .iter()
                    .any(|system| system.name.ends_with("later"))
        }));
        let index = std::fs::read_to_string(scratch.0.join("main-index.md")).unwrap();
        assert!(index.contains("ordering DOT") && index.contains("access DOT"));
        let preview = std::fs::read_dir(&scratch.0)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| path.to_string_lossy().ends_with("order.md"))
            .unwrap();
        assert!(
            std::fs::read_to_string(preview)
                .unwrap()
                .contains("```mermaid\nflowchart LR")
        );
    }

    #[test]
    fn diagrams_extract_real_ordering_access_and_deferred_metadata_without_running_gameplay() {
        let mut app = App::new();
        app.init_resource::<BuildData>()
            .add_observer(remember_build)
            .init_resource::<Counter>()
            .add_systems(Update, (write_counter, read_counter).chain());
        let data = collect_schedules(app.world_mut()).unwrap();
        // Exporting a schedule twice must retain build-pass metadata.
        let again = collect_schedules(app.world_mut()).unwrap();
        assert_eq!(data.schedules.len(), again.schedules.len());
        let update = data.schedules.iter().find(|s| s.name == "Update").unwrap();
        let order = ordering_dot(update);
        let access = access_dot(update);
        assert!(order.contains("write_counter") && order.contains("read_counter"));
        assert!(!update.dependency.is_empty());
        assert!(access.contains("Counter") && access.contains("read") && access.contains("write"));
        assert_eq!(app.world().resource::<Counter>().0, 0);
        app.update();
        assert_eq!(app.world().resource::<Counter>().0, 1);
    }

    #[test]
    fn flow_records_only_observed_transitions_and_deduplicates_idle_frames() {
        let mut flow = ObservedFlow::default();
        assert!(flow.observe("Overworld"));
        assert!(!flow.observe("Overworld"));
        assert!(flow.observe("StartMenu"));
        assert!(flow.observe("Overworld"));
        for _ in 0..1000 {
            flow.observe("StartMenu");
            flow.observe("Overworld");
        }
        assert_eq!(flow.nodes.len(), 2);
        assert_eq!(flow.edges.len(), 2);
        assert!(flow.mermaid().contains("StartMenu --> Overworld"));
        assert!(!flow.mermaid().contains("TrainerBattle"));
    }
}
