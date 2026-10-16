use std::{env, path::PathBuf, process::Command};

use anyhow::{bail, Context, Result};
use crystal_assets::{read_loaded_verified_compiled_game_pack, AssetRoot};
use crystal_bevy::{BevyShellConfig, BevyShellStart};
use crystal_runtime::CrystalRuntime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewMode {
    TwoD,
    TwoPointFiveD,
    Both,
}

impl ViewMode {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "2d" | "classic" => Ok(Self::TwoD),
            "2.5d" | "voxel" => Ok(Self::TwoPointFiveD),
            "both" | "compare" => Ok(Self::Both),
            _ => bail!("--view must be '2d', '2.5d', or 'both'"),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::TwoD => "2D",
            Self::TwoPointFiveD => "2.5D",
            Self::Both => "2D + 2.5D",
        }
    }
}

#[derive(Debug)]
struct Args {
    pack: PathBuf,
    map: Option<String>,
    maps: Vec<String>,
    all_maps: bool,
    kanto_interiors: bool,
    outdoors: bool,
    perimeter: bool,
    tile_x: Option<i16>,
    tile_y: Option<i16>,
    view: ViewMode,
    list_maps: bool,
    screenshot: Option<PathBuf>,
    walk: Option<String>,
    output_dir: Option<PathBuf>,
    hour: u8,
    live: bool,
    preview_flag: Option<String>,
}

fn main() -> Result<()> {
    #[cfg(feature = "operation-trace")]
    let _trace = crystal_bevy::operation_trace::start();
    let args = parse_args(env::args().skip(1))?;
    // The existing same-view capture mode can compare two actual 3D poses
    // without a visible-window readback. Normal single-view captures stay single.
    let temporal_3d = args.view == ViewMode::TwoPointFiveD
        && env::var("CRYSTAL_RENDER_TEST_SECOND_VIEW").as_deref() == Ok("same");
    if temporal_3d
        && (args.map.is_none()
            || args.screenshot.is_none()
            || args.all_maps
            || !args.maps.is_empty()
            || args.live
            || args.walk.is_some())
    {
        bail!("same-view 3D temporal capture requires one --map and --screenshot, without --live or --walk");
    }

    if args.preview_flag.is_some()
        && (args.map.is_none()
            || args.all_maps
            || !args.maps.is_empty()
            || args.screenshot.is_none())
    {
        bail!("--preview-flag requires one --map and --screenshot; it seeds a disposable visual fixture");
    }
    if args.walk.is_some()
        && (args.view == ViewMode::TwoD
            || args.all_maps
            || !args.maps.is_empty()
            || args.screenshot.is_none())
    {
        bail!("--walk requires a single map, --view 2.5d or both and --screenshot");
    }
    if args.live
        && (args.view != ViewMode::Both
            || args.screenshot.is_none()
            || args.all_maps
            || !args.maps.is_empty()
            || args.walk.is_some())
    {
        bail!("--live requires one map, --view both and --screenshot, without --walk");
    }
    let started = std::time::Instant::now();
    if args.view == ViewMode::Both
        && args.screenshot.is_none()
        && !args.all_maps
        && args.maps.is_empty()
    {
        bail!("--screenshot <prefix.png> is required for --view both");
    }
    let pack_path = args
        .pack
        .canonicalize()
        .with_context(|| format!("resolve compiled pack {}", args.pack.display()))?;
    let pack_directory = pack_path
        .parent()
        .context("compiled pack has no parent directory")?;
    let asset_root = AssetRoot::new(pack_directory.to_path_buf());
    let loaded = {
        #[cfg(feature = "operation-trace")]
        let _span = bevy::log::info_span!("crystal_pack_read_verify").entered();
        read_loaded_verified_compiled_game_pack(&pack_path)
            .with_context(|| format!("load compiled pack {}", pack_path.display()))?
    };
    let runtime = {
        #[cfg(feature = "operation-trace")]
        let _span = bevy::log::info_span!("crystal_runtime_load").entered();
        CrystalRuntime::from_loaded_compiled_pack(&asset_root, loaded)?
    };

    if args.list_maps {
        print_map_catalog(&runtime);
        return Ok(());
    }

    if args.all_maps || !args.maps.is_empty() {
        return render_map_batch(&args, &runtime);
    }

    let map = args
        .map
        .clone()
        .context("--map is required unless --list-maps is used")?;
    let (width, height) = runtime
        .data()
        .saved_map_tile_bounds(&map)
        .with_context(|| format!("unknown or dimensionless map {map}"))?;
    let tile_x = args
        .tile_x
        .unwrap_or_else(|| i16::try_from(width / 2).unwrap_or(i16::MAX));
    let tile_y = args
        .tile_y
        .unwrap_or_else(|| i16::try_from(height / 2).unwrap_or(i16::MAX));
    require_in_bounds(&map, tile_x, tile_y, width, height)?;

    let tileset = runtime.data().map_tileset_name(&map)?;
    let environment = runtime.data().map_environment(&map)?;
    println!(
        "rendering {map} at ({tile_x}, {tile_y}) in {} [{width}x{height}, {tileset}, {environment}]",
        args.view.label()
    );
    println!("press F3 to toggle faithful 2D / optional 2.5D at the same location");

    let pair = args.view == ViewMode::Both || temporal_3d;
    let first = args.screenshot.as_ref().map(|path| {
        if pair {
            suffixed_output(path, if temporal_3d { "3d-start" } else { "2d" })
        } else {
            path.clone()
        }
    });
    let second = args
        .screenshot
        .as_ref()
        .filter(|_| pair)
        .map(|path| suffixed_output(path, if temporal_3d { "3d-end" } else { "2.5d" }));
    let identity = format!(
        "{}|{:?}|{map}|{tile_x}|{tile_y}|{}|{:?}",
        pack_path.display(),
        std::fs::metadata(&pack_path)?.modified()?,
        args.hour,
        args.preview_flag
    );
    let previous = second
        .as_ref()
        .filter(|path| {
            std::fs::read_to_string(path.with_extension("identity"))
                .ok()
                .as_deref()
                == Some(&identity)
        })
        .and_then(|path| image::open(path).ok());
    println!("runtime ready in {:.2}s", started.elapsed().as_secs_f64());
    crystal_bevy::run_bevy_shell(
        asset_root,
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier()?,
            map_name: map.clone(),
            tile_x,
            tile_y,
        },
        BevyShellConfig {
            smoke_player_name: Some("RENDER".to_string()),
            // Warm the native scene before freezing a paired baseline. A
            // source-first startup can read back a uniform initial frame,
            // despite valid runtime state. Logical output labels and
            // comparison order remain 2D reference / native.
            voxel_view_enabled: Some(args.view != ViewMode::TwoD),
            window_title: Some(format!(
                "Crystal Render Tester — {map} ({tile_x}, {tile_y}) — {} — F3 toggles",
                args.view.label()
            )),
            render_test_screenshot: if args.view==ViewMode::Both {second.clone()} else {first.clone()},
            render_test_second_screenshot: if args.view==ViewMode::Both {first.clone()} else {second.clone()},
            render_test_live: args.live,
            render_test_walk: args.walk,
            render_test_hour: Some(args.hour),
            render_test_window_size: env::var("CRYSTAL_RENDER_TEST_WINDOW_SIZE")
                .ok()
                .map(|size| {
                    let (width, height) = size.split_once('x')
                        .context("CRYSTAL_RENDER_TEST_WINDOW_SIZE must be WIDTHxHEIGHT")?;
                    Ok::<_, anyhow::Error>((width.parse::<u32>()?, height.parse::<u32>()?))
                })
                .transpose()?,
            render_test_preview_flag: args.preview_flag,
            render_test_party: true,
            ..Default::default()
        },
    )?;
    if let (Some(first), Some(second)) = (first, second) {
        write_comparison(
            &first,
            &second,
            previous,
            args.screenshot.as_ref().unwrap(),
            temporal_3d,
        )?;
        std::fs::write(second.with_extension("identity"), identity)?;
    }
    println!(
        "render loop finished in {:.2}s",
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn render_map_batch(args: &Args, runtime: &CrystalRuntime) -> Result<()> {
    let output_dir = args
        .output_dir
        .as_ref()
        .context("--output-dir <directory> is required with --maps or --all-maps")?;
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("create batch output directory {}", output_dir.display()))?;
    let executable = env::current_exe().context("resolve render tester executable")?;
    let mut maps = if args.all_maps {
        runtime.map_ids().into_iter().collect::<Vec<_>>()
    } else {
        args.maps.clone()
    };
    if args.kanto_interiors {
        let mut selected = Vec::new();
        for map in maps {
            let region = &runtime.data().pokegear_landmark_for_map(&map)?.region;
            let environment = runtime.data().map_environment(&map)?;
            if region == "KANTO" && !matches!(environment, "TOWN" | "ROUTE") {
                selected.push(map);
            }
        }
        maps = selected;
    }
    if args.outdoors {
        let mut selected = Vec::new();
        for map in maps {
            if matches!(runtime.data().map_environment(&map)?, "TOWN" | "ROUTE") {
                selected.push(map);
            }
        }
        maps = selected;
    }
    if maps.is_empty() {
        bail!("batch map selection is empty");
    }
    println!("capturing {} selected maps at {}:00", maps.len(), args.hour);

    for map in maps {
        let (width, height) = runtime
            .data()
            .saved_map_tile_bounds(&map)
            .with_context(|| format!("unknown or dimensionless map {map}"))?;
        let locations = if args.perimeter {
            let east = width.saturating_sub(3);
            let south = height.saturating_sub(3);
            vec![
                ("northwest", 2, 2),
                ("north", width / 2, 2),
                ("northeast", east, 2),
                ("east", east, height / 2),
                ("southeast", east, south),
                ("south", width / 2, south),
                ("southwest", 2, south),
                ("west", 2, height / 2),
            ]
        } else {
            vec![("center", width / 2, height / 2)]
        };
        let mut captured = std::collections::BTreeSet::new();
        for (label, target_x, target_y) in locations {
            let (tile_x, tile_y) =
                nearest_walkable_tile(runtime, &map, width, height, target_x, target_y)
                    .with_context(|| format!("find a walkable {label} location on {map}"))?;
            if !captured.insert((tile_x, tile_y)) {
                continue;
            }
            let stem = if args.perimeter {
                format!("{}-{label}-{tile_x}-{tile_y}", safe_file_stem(&map))
            } else {
                safe_file_stem(&map)
            };
            let output = output_dir.join(format!("{stem}.png"));
            let view = match args.view {
                ViewMode::TwoD => "2d",
                ViewMode::TwoPointFiveD => "2.5d",
                ViewMode::Both => "both",
            };
            let status = Command::new(&executable)
                .arg("--pack")
                .arg(&args.pack)
                .arg("--map")
                .arg(&map)
                .arg("--x")
                .arg(tile_x.to_string())
                .arg("--y")
                .arg(tile_y.to_string())
                .arg("--view")
                .arg(view)
                .arg("--hour")
                .arg(args.hour.to_string())
                .arg("--screenshot")
                .arg(&output)
                .status()
                .with_context(|| format!("launch batch render for {map}"))?;
            if !status.success() {
                bail!("batch render for {map} exited with {status}");
            }
        }
    }
    Ok(())
}

fn nearest_walkable_tile(
    runtime: &CrystalRuntime,
    map: &str,
    width: u16,
    height: u16,
    target_x: u16,
    target_y: u16,
) -> Result<(i16, i16)> {
    let center_x = i32::from(target_x.min(width.saturating_sub(1)));
    let center_y = i32::from(target_y.min(height.saturating_sub(1)));
    let mut candidates = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|&(x, y)| {
        let dx = i32::from(x) - center_x;
        let dy = i32::from(y) - center_y;
        (dx * dx + dy * dy, y, x)
    });
    for (x, y) in candidates {
        let x = i16::try_from(x)?;
        let y = i16::try_from(y)?;
        let tile = crystal_runtime::core::world::map::TilePosition::new(x, y);
        if let Ok(session)=runtime.data().overworld_session(map,tile,0) {
            // Session initialization also permits connection-source exceptions.
            // Perimeter review needs a standable source tile, not a wall that
            // happens to share the connection's coordinate range.
            use crystal_runtime::core::world::collision::{can_enter_tile,is_warp_permission,sample_collision,allows_ledge_direction,PlayerTraversalState};
            use crystal_runtime::core::world::map::Direction;
            let directions=[Direction::Down,Direction::Up,Direction::Left,Direction::Right];
            if sample_collision(&session.map,&session.tileset,tile).is_some_and(|s|
                !is_warp_permission(s.permission)
                && !directions.iter().any(|&d|allows_ledge_direction(s.permission,d)))
                && directions.into_iter().any(|d|can_enter_tile(&session.map,&session.tileset,tile,d,PlayerTraversalState::Walk)) {
                return Ok((x,y));
            }
        }
    }
    bail!("map has no walkable runtime tile")
}

fn safe_file_stem(map: &str) -> String {
    map.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

// One inspection surface: source reference, previous iteration (if available),
// current render. Nearest sampling preserves the pixel-art edges when shrinking.
fn write_comparison(
    first: &std::path::Path,
    second: &std::path::Path,
    previous: Option<image::DynamicImage>,
    prefix: &std::path::Path,
    temporal_3d: bool,
) -> Result<()> {
    let mut panels = vec![image::open(first)?];
    let has_previous = previous.is_some();
    panels.extend(previous);
    panels.push(image::open(second)?);
    let width = 640;
    let height = 576;
    let mut sheet = image::RgbImage::new(width * panels.len() as u32, height);
    for (index, panel) in panels.iter().enumerate() {
        let panel = panel
            .resize_exact(width, height, image::imageops::FilterType::Nearest)
            .to_rgb8();
        image::imageops::replace(&mut sheet, &panel, index as i64 * i64::from(width), 0);
    }
    let output = suffixed_output(prefix, "compare").with_extension("png");
    sheet.save(&output)?;
    let labels = if temporal_3d && has_previous {
        "Earlier 3D pose · Previous 3D pose · Later 3D pose"
    } else if temporal_3d {
        "Earlier 3D pose · Later 3D pose"
    } else if has_previous {
        "2D reference · Previous 2.5D · Current 2.5D"
    } else {
        "2D reference · Current 2.5D"
    };
    let filename = output
        .file_name()
        .unwrap()
        .to_string_lossy()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;");
    std::fs::write(
        output.with_extension("html"),
        format!(
            "<!doctype html><meta charset=\"utf-8\"><title>Render comparison</title><style>body{{margin:24px;background:#141b19;color:#e7eee8;font:16px system-ui}}img{{max-width:100%;image-rendering:pixelated}}p{{color:#aebeb4}}</style><h2>{labels}</h2><p>Left to right. Open the PNG at full size to inspect pixel edges. Rerun the same command after each Rust edit, then reload this page.</p><img src=\"{filename}\" alt=\"{labels}\">"
        ),
    )?;
    println!("review {} — left to right: {labels}", output.display(),);
    Ok(())
}

fn suffixed_output(output: &std::path::Path, suffix: &str) -> PathBuf {
    let parent = output.parent().unwrap_or_else(|| std::path::Path::new(""));
    let stem = output
        .file_stem()
        .unwrap_or(output.as_os_str())
        .to_string_lossy();
    let extension = output
        .extension()
        .map(|value| format!(".{}", value.to_string_lossy()))
        .unwrap_or_default();
    parent.join(format!("{stem}-{suffix}{extension}"))
}

fn print_map_catalog(runtime: &CrystalRuntime) {
    println!("map\twidth\theight\ttileset\tenvironment");
    for map in runtime.map_ids() {
        let Some((width, height)) = runtime.data().saved_map_tile_bounds(&map) else {
            continue;
        };
        let tileset = runtime.data().map_tileset_name(&map).unwrap_or("?");
        let environment = runtime.data().map_environment(&map).unwrap_or("?");
        println!("{map}\t{width}\t{height}\t{tileset}\t{environment}");
    }
}

fn require_in_bounds(map: &str, x: i16, y: i16, width: u16, height: u16) -> Result<()> {
    if x < 0 || y < 0 || u16::try_from(x)? >= width || u16::try_from(y)? >= height {
        bail!(
            "location {map} ({x}, {y}) is outside gameplay tile bounds 0..{} x 0..{}",
            width.saturating_sub(1),
            height.saturating_sub(1)
        );
    }
    Ok(())
}

fn parse_args(values: impl IntoIterator<Item = String>) -> Result<Args> {
    let mut pack = None;
    let mut map = None;
    let mut maps = Vec::new();
    let mut all_maps = false;
    let mut kanto_interiors = false;
    let mut outdoors = false;
    let mut perimeter = false;
    let mut tile_x = None;
    let mut tile_y = None;
    let mut view = ViewMode::TwoD;
    let mut list_maps = false;
    let mut screenshot = None;
    let mut walk = None;
    let mut output_dir = None;
    let mut hour = 12;
    let mut live = false;
    let mut preview_flag = None;
    let mut values = values.into_iter();
    while let Some(flag) = values.next() {
        match flag.as_str() {
            "--pack" => pack = Some(PathBuf::from(next_value(&mut values, "--pack")?)),
            "--map" => map = Some(next_value(&mut values, "--map")?),
            "--maps" => maps.extend(
                next_value(&mut values, "--maps")?
                    .split(',')
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
            ),
            "--all-maps" => all_maps = true,
            "--perimeter" => perimeter = true,
            "--outdoors" => {
                all_maps = true;
                outdoors = true;
            }
            "--kanto-interiors" => {
                all_maps = true;
                kanto_interiors = true;
            }
            "--live" => live = true,
            "--preview-flag" => preview_flag = Some(next_value(&mut values, "--preview-flag")?),
            "--x" => tile_x = Some(next_value(&mut values, "--x")?.parse::<i16>()?),
            "--y" => tile_y = Some(next_value(&mut values, "--y")?.parse::<i16>()?),
            "--view" => view = ViewMode::parse(&next_value(&mut values, "--view")?)?,
            "--hour" => {
                hour = next_value(&mut values, "--hour")?.parse::<u8>()?;
                ensure_render_hour(hour)?;
            }
            "--walk" => walk = Some(next_value(&mut values, "--walk")?),
            "--list-maps" => list_maps = true,
            "--screenshot" => {
                screenshot = Some(PathBuf::from(next_value(&mut values, "--screenshot")?))
            }
            "--output-dir" => {
                output_dir = Some(PathBuf::from(next_value(&mut values, "--output-dir")?))
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            _ => bail!("unknown argument '{flag}'"),
        }
    }
    Ok(Args {
        pack: pack.context("--pack <game.crystalpack> is required")?,
        map,
        maps,
        all_maps,
        kanto_interiors,
        outdoors,
        perimeter,
        tile_x,
        tile_y,
        view,
        list_maps,
        screenshot,
        walk,
        output_dir,
        hour,
        live,
        preview_flag,
    })
}

fn ensure_render_hour(hour: u8) -> Result<()> {
    if hour >= 24 {
        bail!("--hour must be in 0..24");
    }
    Ok(())
}

fn next_value(values: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    values
        .next()
        .with_context(|| format!("{flag} requires a value"))
}

fn print_usage() {
    println!(
        r#"cargo run -p crystal-bevy --example render_at_location --features location-tester -- \
         --pack <game.crystalpack> [--list-maps | --map <id> [--x <tile>] [--y <tile>] \
         [--view 2d|2.5d|both] [--hour <0..23>] [--screenshot <output-or-prefix.png>] [--live] [--walk RDLU] \
         [--preview-flag <verified-flag>] | \
         (--maps <id,id,...> | --all-maps | --outdoors | --kanto-interiors) --output-dir <directory> [--perimeter] \
         [--view 2d|2.5d|both] [--hour <0..23>]]"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_render_modes() {
        assert_eq!(ViewMode::parse("2d").unwrap(), ViewMode::TwoD);
        assert_eq!(ViewMode::parse("2.5d").unwrap(), ViewMode::TwoPointFiveD);
        assert_eq!(ViewMode::parse("both").unwrap(), ViewMode::Both);
        assert!(ViewMode::parse("3d").is_err());
    }

    #[test]
    fn validates_deterministic_render_hour() {
        assert!(ensure_render_hour(0).is_ok());
        assert!(ensure_render_hour(12).is_ok());
        assert!(ensure_render_hour(23).is_ok());
        assert!(ensure_render_hour(24).is_err());
    }

    #[test]
    fn comparison_outputs_are_labeled_without_losing_extension() {
        assert_eq!(
            suffixed_output(std::path::Path::new("/tmp/NewBark.png"), "2.5d"),
            PathBuf::from("/tmp/NewBark-2.5d.png")
        );
        assert_eq!(
            suffixed_output(std::path::Path::new("comparison"), "2d"),
            PathBuf::from("comparison-2d")
        );
    }

    #[test]
    fn rejects_locations_outside_map_bounds() {
        assert!(require_in_bounds("Map", 0, 0, 10, 8).is_ok());
        assert!(require_in_bounds("Map", 10, 0, 10, 8).is_err());
        assert!(require_in_bounds("Map", -1, 0, 10, 8).is_err());
    }

    #[test]
    fn batch_output_stems_are_filesystem_safe() {
        assert_eq!(safe_file_stem("NewBarkTown"), "NewBarkTown");
        assert_eq!(safe_file_stem("Map / Test"), "Map___Test");
    }
}
