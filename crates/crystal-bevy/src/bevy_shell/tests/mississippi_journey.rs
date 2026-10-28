//! Full native Surf itinerary. Party, badges and the initial bank are fixtures;
//! every subsequent transition and HM action goes through production input.

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::*;

#[derive(Deserialize)]
struct Position {
    map: String,
    point: [i16; 2],
}

#[derive(Deserialize)]
struct Input {
    map: String,
    point: [i16; 2],
    direction: String,
    #[serde(default)]
    action: Option<String>,
}

#[derive(Deserialize)]
struct BridgeAperture {
    id: String,
    map: String,
    metatile_bounds_exclusive: [i16; 4],
}

#[derive(Deserialize)]
struct Itinerary {
    pack_sha256: String,
    start: Position,
    launch: Input,
    steps: Vec<Input>,
    target: Position,
    bridge_apertures: Vec<BridgeAperture>,
    #[serde(default)]
    source_only_wooded_reach_inputs: std::collections::BTreeMap<String, usize>,
}

fn opposite(value: &str) -> GameButton {
    match value {
        "up" => GameButton::Down,
        "down" => GameButton::Up,
        "left" => GameButton::Right,
        "right" => GameButton::Left,
        _ => panic!("unsupported itinerary direction: {value}"),
    }
}

fn continue_under_bridge(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    controller: &mut VisibleShellController,
    label: &str,
) {
    let directory = std::env::var_os("CRYSTAL_MISSISSIPPI_CAPTURE_SAVES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&directory).expect("retained native bridge saves");
    let path = directory.join(format!(
        "{label}-{}-{}.crystalsave",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let before = controller.snapshot().expect("bridge Continue preimage");
    controller.save(&path).expect("production bridge save");
    *controller = VisibleShellController::load_save(root.clone(), runtime.clone(), path, None)
        .expect("production Continue beneath bridge");
    let after = controller.snapshot().expect("continued beneath bridge");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.overworld.facing, after.overworld.facing);
}

fn button(value: &str) -> GameButton {
    match value {
        "up" => GameButton::Up,
        "down" => GameButton::Down,
        "left" => GameButton::Left,
        "right" => GameButton::Right,
        _ => panic!("unsupported itinerary direction: {value}"),
    }
}

fn climb(c: &mut VisibleShellController, point: Point) {
    let (map, before) = at(c);
    assert_eq!(map, CITY);
    face_up(c, before);
    c.press(GameButton::A).expect("native Waterfall prompt");
    for _ in 0..256 {
        if at(c) == (CITY.into(), point)
            && c.shell.visible_waterfall_animation.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            assert!(c.shell.flygon_field_actions.contains("waterfall"));
            return;
        }
        if c.shell.visible_waterfall_animation.is_some() && c.shell.field_notice.is_none() {
            c.wait_frames(16).expect("native Waterfall ascent clock");
        } else {
            c.press(GameButton::A).expect("confirm native Waterfall");
            c.wait_frames(16).expect("native field action");
        }
    }
    panic!("native Waterfall itinerary did not reach {point:?}");
}

#[test]
#[ignore = "requires the published connected Mississippi pack and exact native itinerary"]
fn mississippi_continuous_surf_winona_to_st_cloud_and_bridge_continue() {
    let plan_path = std::env::var_os("CRYSTAL_MISSISSIPPI_ITINERARY")
        .expect("exact published-pack itinerary path");
    let plan: Itinerary =
        serde_json::from_slice(&std::fs::read(plan_path).expect("read exact native itinerary"))
            .expect("native itinerary schema");
    let pack_path = std::env::var_os("CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK")
        .expect("published connected Mississippi pack");
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(pack_path).expect("published pack"))
        ),
        plan.pack_sha256,
        "itinerary must match the exact installed native pack"
    );
    assert_eq!(plan.start.map, "MnWinona");
    assert_eq!(plan.target.map, "MnStCloud");
    let (root, r, mut c) = fixture(&plan.start.map, plan.start.point.into(), true, true, true);
    mount(
        &mut c,
        button(&plan.launch.direction),
        &plan.launch.map,
        plan.launch.point.into(),
    );
    let mut crossed_bridges = std::collections::BTreeSet::new();
    let mut crossed_maps = std::collections::BTreeSet::from([plan.launch.map.clone()]);
    let mut continued = false;
    let mut apertures = std::collections::BTreeSet::new();
    let mut reach_inputs = std::collections::BTreeMap::<String, usize>::new();
    for (index, input) in plan.steps.iter().enumerate() {
        let previous_map = at(&mut c).0;
        match input.action.as_deref() {
            None => {
                step(
                    &mut c,
                    button(&input.direction),
                    &input.map,
                    input.point.into(),
                    &format!("continuous Surf itinerary step {index}"),
                );
            }
            Some("waterfall_up") => climb(&mut c, input.point.into()),
            Some(other) => panic!("unsupported itinerary action: {other}"),
        }
        let snapshot = c.snapshot().expect("continuous native Surf");
        assert_eq!(
            snapshot.overworld.mode,
            MovementMode::Surf,
            "step {index}: no land/teleport shortcut"
        );
        crossed_maps.insert(input.map.clone());
        for reach in ["LowerWoods", "UpstreamWoods"] {
            let prefix = format!("MnMississippi{reach}");
            if previous_map.starts_with(&prefix) && input.map.starts_with(&prefix) {
                *reach_inputs.entry(reach.into()).or_default() += 1;
            }
        }
        let bridge_capture = input.map == CITY
            && ((11..=18).contains(&input.point[1]) || (53..=60).contains(&input.point[1]));
        let wooded_reach_capture = matches!(
            (input.map.as_str(), input.point),
            ("MnMississippiLowerWoodsC00R02", [36, 39])
                | ("MnMississippiLowerWoodsC01R01", [43, 3])
                | ("MnMississippiLowerWoodsC00R01", [0, 24])
                | ("MnMississippiUpstreamWoodsC00R02", [0, 36])
                | ("MnMississippiUpstreamWoodsC01R01", [36, 23])
                | ("MnMississippiUpstreamWoodsC00R00", [0, 40])
        );
        if (bridge_capture || wooded_reach_capture || previous_map != input.map)
            && let Some(directory) = std::env::var_os("CRYSTAL_MISSISSIPPI_CAPTURE_SAVES_DIR")
        {
            let directory = PathBuf::from(directory);
            std::fs::create_dir_all(&directory).expect("ignored native capture saves");
            let path = directory.join(format!(
                "{}-{}-{}-{:?}.crystalsave",
                input.map, input.point[0], input.point[1], snapshot.overworld.facing
            ));
            c.save(&path)
                .expect("production save for actual native GPU river capture");
        }
        let permission = collision(&r, &mut c, input.point.into());
        if permission.starts_with("BRIDGE_") {
            crossed_bridges.insert(input.map.clone());
            for aperture in &plan.bridge_apertures {
                let [x0, y0, x1, y1] = aperture.metatile_bounds_exclusive;
                if input.map == aperture.map
                    && (x0 * 2..x1 * 2).contains(&input.point[0])
                    && (y0 * 2..y1 * 2).contains(&input.point[1])
                {
                    apertures.insert(aperture.id.clone());
                }
            }
            if !continued {
                continue_under_bridge(&root, &r, &mut c, "upstream-bridge-continue");
                continued = true;
            }
        }
    }
    assert_at(
        &mut c,
        &plan.target.map,
        plan.target.point.into(),
        "Winona to St Cloud completed",
    );
    assert!(
        continued,
        "the route must use and Continue under a real bridge"
    );
    assert_eq!(
        plan.bridge_apertures.len(),
        7,
        "all existing river apertures are required"
    );
    for aperture in &plan.bridge_apertures {
        assert!(
            apertures.contains(&aperture.id),
            "native Surf missed bridge {}",
            aperture.id
        );
    }
    assert!(
        crossed_bridges.contains(CITY),
        "Minneapolis road bridges must be navigable below"
    );
    assert!(
        crossed_maps.len() >= 8,
        "a real multi-map river journey is required"
    );
    assert!(c.shell.flygon_field_actions.contains("waterfall"));
    if !plan.source_only_wooded_reach_inputs.is_empty() {
        assert_eq!(reach_inputs, plan.source_only_wooded_reach_inputs);
    }

    let mut reverse_apertures = std::collections::BTreeSet::new();
    let mut reverse_continued = false;
    let mut reverse_reach_inputs = std::collections::BTreeMap::<String, usize>::new();
    for (index, input) in plan.steps.iter().enumerate().rev() {
        let (map, point) = if index == 0 {
            (&plan.launch.map, plan.launch.point)
        } else {
            (&plan.steps[index - 1].map, plan.steps[index - 1].point)
        };
        if input.action.as_deref() == Some("waterfall_up") {
            descend(&mut c, input.point[0]);
            assert_at(&mut c, map, point.into(), "native downstream current");
        } else {
            step(
                &mut c,
                opposite(&input.direction),
                map,
                point.into(),
                &format!("downstream Surf itinerary step {index}"),
            );
        }
        assert_eq!(
            c.snapshot().expect("downstream native Surf").overworld.mode,
            MovementMode::Surf,
            "reverse step {index}: no land shortcut"
        );
        for reach in ["LowerWoods", "UpstreamWoods"] {
            let prefix = format!("MnMississippi{reach}");
            if input.map.starts_with(&prefix) && map.starts_with(&prefix) {
                *reverse_reach_inputs.entry(reach.into()).or_default() += 1;
            }
        }
        if collision(&r, &mut c, point.into()).starts_with("BRIDGE_") {
            for aperture in &plan.bridge_apertures {
                let [x0, y0, x1, y1] = aperture.metatile_bounds_exclusive;
                if *map == aperture.map
                    && (x0 * 2..x1 * 2).contains(&point[0])
                    && (y0 * 2..y1 * 2).contains(&point[1])
                {
                    reverse_apertures.insert(aperture.id.clone());
                }
            }
            if !reverse_continued {
                continue_under_bridge(&root, &r, &mut c, "downstream-bridge-continue");
                reverse_continued = true;
            }
        }
    }
    assert_eq!(reverse_apertures, apertures, "all seven return apertures");
    assert!(reverse_continued, "downstream Continue beneath a bridge");
    assert_eq!(reverse_reach_inputs, reach_inputs);
    step(
        &mut c,
        opposite(&plan.launch.direction),
        &plan.start.map,
        plan.start.point.into(),
        "return to the real Winona bank",
    );
    assert_eq!(
        c.snapshot().expect("Winona return dismount").overworld.mode,
        MovementMode::Normal
    );
}

#[test]
#[ignore = "requires the published connected Mississippi pack"]
fn mississippi_bridge_keeps_native_foot_road_and_upper_save() {
    let (_, _, mut c) = fixture(CITY, (31, 14), true, true, true);
    step(
        &mut c,
        GameButton::Right,
        CITY,
        (32, 14),
        "walk onto the real upper deck",
    );
    assert_eq!(
        c.snapshot().expect("upper bridge").overworld.mode,
        MovementMode::Normal
    );
    if let Some(directory) = std::env::var_os("CRYSTAL_MISSISSIPPI_CAPTURE_SAVES_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("ignored native capture saves");
        c.save(directory.join("MnMinneapolis-32-14-Foot.crystalsave"))
            .expect("production upper-deck capture save");
    }
    for x in 33..=38 {
        step(
            &mut c,
            GameButton::Right,
            CITY,
            (x, 14),
            "native road remains continuous",
        );
        assert_eq!(
            c.snapshot().expect("upper bridge mode").overworld.mode,
            MovementMode::Normal
        );
    }
}
