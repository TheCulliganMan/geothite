//! Native movement QA: ordinary keyboard input, per-frame trace and moving captures.
use super::*;

#[derive(Resource)]
pub(super) struct RenderWalk {
    route: Vec<KeyCode>,
    tick: usize,
    finished: bool,
    settle: usize,
    path: PathBuf,
    trace: String,
    times: Vec<f64>,
    first_builds: Option<u64>,
    origins: std::collections::HashSet<(i32, i32)>,
    maps: std::collections::HashSet<String>,
}
impl RenderWalk {
    pub(super) fn settled(&self) -> bool {
        self.finished && self.settle >= 30
    }
}

pub(super) fn install(app: &mut App, route: &str, path: &Path) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let route = route
        .chars()
        .map(|c| match c {
            'R' => Ok(KeyCode::ArrowRight),
            'L' => Ok(KeyCode::ArrowLeft),
            'U' => Ok(KeyCode::ArrowUp),
            'D' => Ok(KeyCode::ArrowDown),
            _ => Err(anyhow::anyhow!("--walk accepts only R,L,U,D")),
        })
        .collect::<Result<Vec<_>>>()?;
    anyhow::ensure!(!route.is_empty(), "--walk must contain directions");
    app.insert_resource(RenderWalk {
        route,
        tick: 0,
        finished: false,
        settle: 0,
        path: path.to_owned(),
        trace: "frame,map,origin_x,origin_y,center_x,center_y,builds,frame_ms,focused\n".into(),
        times: Vec::new(),
        first_builds: None,
        origins: default(),
        maps: default(),
    })
    .add_systems(PreUpdate, drive.after(bevy::input::InputSystem));
    Ok(())
}

fn drive(
    mut walk: ResMut<RenderWalk>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    status: Res<crystal_voxel_view::VoxelViewStatus>,
    frame: Res<crystal_render_api::VisualWorldFrame>,
    time: Res<Time<Real>>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut screenshots: ResMut<ScreenshotManager>,
) {
    if walk.finished {
        walk.settle += 1;
        return;
    }
    if walk.first_builds.is_none() && status.active_frames < 90 {
        return;
    }
    walk.first_builds.get_or_insert(status.terrain_builds);
    for key in [
        KeyCode::ArrowRight,
        KeyCode::ArrowLeft,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ] {
        if walk.tick % 32 == 0 {
            keyboard.release(key);
        }
    }
    let segment = walk.tick / 32;
    if segment == walk.route.len() {
        walk.finished = true;
        std::fs::write(walk.path.with_extension("movement.csv"), &walk.trace)
            .expect("write movement trace");
        let mut times = walk.times.clone();
        times.sort_by(f64::total_cmp);
        println!(
            "movement QA: {} frames, {} grid origins, {} additional terrain builds, median {:.2} ms, p95 {:.2} ms",
            times.len(),
            walk.origins.len(),
            status.terrain_builds - walk.first_builds.unwrap(),
            times[times.len() / 2],
            times[times.len() * 95 / 100]
        );
        assert!(
            walk.origins.len() > 1 || walk.maps.len() > 1,
            "movement QA must actually cross a tile origin or map boundary"
        );
        return;
    }
    keyboard.press(walk.route[segment]);
    let dt = time.delta_seconds_f64() * 1000.0;
    walk.times.push(dt);
    walk.maps.insert(frame.map_id.to_string());
    walk.origins
        .insert((frame.grid_origin.x, frame.grid_origin.y));
    let line = format!(
        "{},{},{},{},{:.3},{:.3},{},{:.3},{}\n",
        walk.tick,
        frame.map_id,
        frame.grid_origin.x,
        frame.grid_origin.y,
        frame.center.x,
        frame.center.y,
        status.terrain_builds,
        dt,
        windows.get_single().is_ok_and(|(_, window)| window.focused)
    );
    walk.trace.push_str(&line);
    if walk.tick % 32 == 12 {
        if let Ok((window, _)) = windows.get_single() {
            let stem = walk.path.file_stem().unwrap().to_string_lossy();
            let path = walk
                .path
                .with_file_name(format!("{stem}-moving-{segment:02}.png"));
            screenshots
                .save_screenshot_to_disk(window, path)
                .expect("capture moving frame");
        }
    }
    walk.tick += 1;
}
