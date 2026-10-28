//! Fixture-only path planning; every gameplay step uses the production controller.
use super::super::VisibleShellController;
use crystal_assets::MapModule;
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
pub(super) type Point = (i16, i16);

pub(super) struct RouteSpec {
    pub prefix: &'static str,
    pub columns: i16,
    pub rows: i16,
    pub tileset: &'static str,
    pub start: Point,
    pub doors: BTreeMap<Point, &'static str>,
}

pub(super) struct RouteGrid {
    pub floor: BTreeSet<Point>,
    pub blocked: BTreeSet<Point>,
    pub hazards: BTreeSet<Point>,
    ice: BTreeSet<Point>,
    doors: BTreeMap<Point, &'static str>,
}

/// Retains reached saves only when a review explicitly names its evidence directory.
pub(super) fn reached_save_path(filename: &str) -> PathBuf {
    let directory = std::env::var_os("CRYSTAL_MAP_RETAIN_SAVES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&directory).expect("reached fixture save directory");
    let path = directory.join(filename);
    assert!(!path.exists(), "reached fixture save must be fresh");
    path
}

pub(super) fn cleanup_reached_save(path: &Path) {
    if std::env::var_os("CRYSTAL_MAP_RETAIN_SAVES_DIR").is_none() {
        std::fs::remove_file(path).expect("remove disposable fixture save");
    }
}

pub(super) fn origin(name: &str) -> Point {
    let (col, row) = name
        .split_once('C')
        .unwrap_or_else(|| panic!("expected route chunk, got {name}"))
        .1
        .split_once('R')
        .expect("chunk coordinates");
    (
        col.parse::<i16>().expect("column") * 64,
        row.parse::<i16>().expect("row") * 48,
    )
}

pub(super) fn position(controller: &mut VisibleShellController) -> Point {
    let s = controller.snapshot().expect("route position");
    let (x, y) = origin(&s.overworld.map_name);
    (x + s.overworld.tile.x, y + s.overworld.tile.y)
}

impl RouteGrid {
    pub fn new(runtime: &CrystalRuntime, spec: RouteSpec) -> Self {
        Self::build(runtime, spec, false)
    }

    /// Includes grass; populated journeys must settle encounters through input.
    pub fn new_with_grass(runtime: &CrystalRuntime, spec: RouteSpec) -> Self {
        Self::build(runtime, spec, true)
    }

    fn build(runtime: &CrystalRuntime, spec: RouteSpec, include_grass: bool) -> Self {
        let mut grid = Self {
            floor: BTreeSet::new(),
            blocked: BTreeSet::new(),
            hazards: BTreeSet::new(),
            ice: BTreeSet::new(),
            doors: spec.doors,
        };
        let mut trainers = Vec::new();
        for row in 0..spec.rows {
            for col in 0..spec.columns {
                let name = format!("{}C{col:02}R{row:02}", spec.prefix);
                let module = runtime
                    .data()
                    .assemble_map_module_from_compiled_payloads(&name)
                    .expect("published chunk");
                for y in 0..usize::from(module.attributes.height) * 2 {
                    for x in 0..usize::from(module.attributes.width) * 2 {
                        let block =
                            module.blocks[(y / 2) * usize::from(module.attributes.width) + x / 2];
                        let kind = &runtime.data().tilesets[spec.tileset].collision
                            [&format!("{block:02x}")][(y % 2) * 2 + x % 2];
                        if kind == "ICE" {
                            grid.ice.insert((col * 64 + x as i16, row * 48 + y as i16));
                        }
                        if matches!(kind.as_str(), "FLOOR" | "LADDER" | "DOOR" | "CAVE")
                            || (include_grass && kind == "TALL_GRASS")
                        {
                            grid.floor
                                .insert((col * 64 + x as i16, row * 48 + y as i16));
                        }
                    }
                }
                for o in module.objects {
                    let p = (col * 64 + o.x as i16, row * 48 + o.y as i16);
                    grid.blocked.insert(p);
                    if o.object_type == "OBJECTTYPE_TRAINER" {
                        trainers.push((p, o));
                    }
                }
            }
        }
        for ((x, y), o) in trainers {
            let (dx, dy) = match o.spritemovedata.as_str() {
                "SPRITEMOVEDATA_STANDING_UP" => (0, -1),
                "SPRITEMOVEDATA_STANDING_LEFT" => (-1, 0),
                "SPRITEMOVEDATA_STANDING_RIGHT" => (1, 0),
                _ => (0, 1),
            };
            for n in 1..=o.radius as i16 {
                let p = (x + dx * n, y + dy * n);
                if !grid.floor.contains(&p) {
                    break;
                }
                grid.hazards.insert(p);
            }
        }
        // Native rock-top tiles can be FLOOR while remaining inaccessible
        // scenery. Tour seams in the actual clear-route component only.
        let mut connected = BTreeSet::from([spec.start]);
        let mut queue = VecDeque::from([spec.start]);
        while let Some(p) = queue.pop_front() {
            for (q, _) in grid.destinations(p) {
                if grid.floor.contains(&q)
                    && !grid.blocked.contains(&q)
                    && !grid.hazards.contains(&q)
                    && connected.insert(q)
                    && !grid.doors.contains_key(&q)
                {
                    queue.push_back(q);
                }
            }
        }
        grid.floor.retain(|p| connected.contains(p));
        grid
    }

    // A single production press entering ICE owns the remainder of that
    // straight slide. Plan its actual dry stop rather than pretending every
    // sliding quadrant is a separately controllable walking position.
    fn destinations(&self, from: Point) -> Vec<(Point, GameButton)> {
        neighbors(from)
            .into_iter()
            .filter_map(|(mut next, button)| {
                let delta = (next.0 - from.0, next.1 - from.1);
                while self.ice.contains(&next) {
                    if self.blocked.contains(&next) || self.hazards.contains(&next) {
                        return None;
                    }
                    next = (next.0 + delta.0, next.1 + delta.1);
                }
                Some((next, button))
            })
            .collect()
    }

    pub fn path(&self, from: Point, to: Point) -> Vec<GameButton> {
        let mut parents = BTreeMap::from([(from, (from, GameButton::A))]);
        let mut queue = VecDeque::from([from]);
        while let Some(p) = queue.pop_front() {
            if p == to {
                break;
            }
            for (q, button) in self.destinations(p) {
                if !self.floor.contains(&q)
                    || self.blocked.contains(&q)
                    || self.hazards.contains(&q)
                    || parents.contains_key(&q)
                {
                    continue;
                }
                // The exterior door is a terminal, never a through path.
                if self.doors.contains_key(&q) && q != to {
                    continue;
                }
                parents.insert(q, (p, button));
                queue.push_back(q);
            }
        }
        assert!(
            parents.contains_key(&to),
            "no safe route {from:?} -> {to:?}"
        );
        let mut buttons = Vec::new();
        let mut cursor = to;
        while cursor != from {
            let (prior, button) = parents[&cursor];
            buttons.push(button);
            cursor = prior;
        }
        buttons.reverse();
        buttons
    }

    pub fn walk(&self, controller: &mut VisibleShellController, to: Point) {
        let from = position(controller);
        for button in self.path(from, to) {
            let before = position(controller);
            let expected = self
                .destinations(before)
                .into_iter()
                .find_map(|(point, input)| (input == button).then_some(point))
                .expect("planned production route destination");
            for _ in 0..8 {
                controller.press(button).expect("native route input");
                let s = controller.snapshot().expect("native route step");
                assert!(
                    s.battle.is_none(),
                    "unexpected trainer encounter at {before:?}"
                );
                if self
                    .doors
                    .values()
                    .any(|&name| name == s.overworld.map_name)
                {
                    assert_eq!(
                        self.doors.get(&to).copied(),
                        Some(s.overworld.map_name.as_str()),
                        "unexpected door"
                    );
                    return;
                }
                if position(controller) != before {
                    break;
                }
            }
            if (expected.0 - before.0).abs() + (expected.1 - before.1).abs() > 1 {
                controller
                    .wait_frames(320)
                    .expect("native ICE settles at planned dry stop");
                assert_eq!(
                    position(controller),
                    expected,
                    "actual forced slide endpoint"
                );
            }
            assert_ne!(
                position(controller),
                before,
                "blocked route step {before:?}, {button:?}"
            );
        }
        assert_eq!(position(controller), to);
    }

    pub fn talk(&self, controller: &mut VisibleShellController, at: Point) -> String {
        let (approach, button) = neighbors(at)
            .into_iter()
            .find(|(p, _)| {
                self.floor.contains(p)
                    && !self.blocked.contains(p)
                    && !self.hazards.contains(p)
                    && (p.0 / 64, p.1 / 48) == (at.0 / 64, at.1 / 48)
            })
            .unwrap_or_else(|| panic!("no reachable talk approach for {at:?}"));
        self.walk(controller, approach);
        controller.press(opposite(button)).expect("face actor");
        controller.press(GameButton::A).expect("talk or collect");
        let mut pages = Vec::new();
        collect_visible_page(controller, &mut pages);
        controller
            .wait_frames(128)
            .expect("fully print source interaction page");
        let s = controller.snapshot().expect("visible conversation");
        assert!(
            s.phase == RuntimeShellPhase::Text && s.ui.text.is_some(),
            "interaction at {at:?} did not show dialogue: ui={:?}, bag={:?}",
            s.ui,
            s.bag.items
        );
        let before = s.overworld.tile;
        controller
            .press(GameButton::Down)
            .expect("movement while text owns input");
        assert_eq!(
            controller
                .snapshot()
                .expect("dialogue input owner")
                .overworld
                .tile,
            before
        );
        collect_visible_page(controller, &mut pages);
        for _ in 0..64 {
            if dialogue_finished(controller) {
                return pages.join("\n");
            }
            controller
                .press(GameButton::A)
                .expect("native interaction continuation");
            collect_visible_page(controller, &mut pages);
            controller
                .wait_frames(16)
                .expect("native interaction printer");
            collect_visible_page(controller, &mut pages);
        }
        panic!("interaction at {at:?} did not release input: {pages:?}")
    }
}

pub(super) fn neighbors((x, y): Point) -> [(Point, GameButton); 4] {
    [
        ((x + 1, y), GameButton::Right),
        ((x - 1, y), GameButton::Left),
        ((x, y + 1), GameButton::Down),
        ((x, y - 1), GameButton::Up),
    ]
}

pub(super) fn opposite(button: GameButton) -> GameButton {
    match button {
        GameButton::Left => GameButton::Right,
        GameButton::Right => GameButton::Left,
        GameButton::Up => GameButton::Down,
        GameButton::Down => GameButton::Up,
        _ => panic!("direction required"),
    }
}

fn collect_visible_page(controller: &mut VisibleShellController, pages: &mut Vec<String>) {
    let snapshot = controller
        .snapshot()
        .expect("actual visible interaction page");
    if let Some(text) = snapshot.ui.text.and_then(|t| t.asm_text)
        && !text.is_empty()
        && pages.last() != Some(&text)
    {
        pages.push(text);
    }
    if let Some(notice) = &controller.shell.visible_field_item_notice
        && !pages.contains(&notice.sound_trigger_text)
    {
        pages.push(notice.sound_trigger_text.clone());
    }
}

fn dialogue_finished(controller: &mut VisibleShellController) -> bool {
    let snapshot = controller.snapshot().expect("interaction input owner");
    snapshot.phase == RuntimeShellPhase::Overworld
        && snapshot.ui.pending_text_wait.is_none()
        && !snapshot.ui.text_window_open
        && !controller.has_pending_script_work()
        && controller.shell.field_notice.is_none()
        && controller.shell.visible_field_item_notice.is_none()
        && snapshot
            .ui
            .text
            .as_ref()
            .is_none_or(|text| text.label != "visible-shell:battle-message")
}

pub(super) fn finish_text(controller: &mut VisibleShellController) {
    for _ in 0..64 {
        let s = controller.snapshot().expect("text state");
        if s.phase == RuntimeShellPhase::Overworld
            && s.ui.pending_text_wait.is_none()
            && !controller.has_pending_script_work()
            && s.ui
                .text
                .as_ref()
                .is_none_or(|text| text.label != "visible-shell:battle-message")
        {
            return;
        }
        controller
            .press(GameButton::A)
            .expect("native dialogue page");
    }
    panic!("route dialogue did not release input");
}

pub(super) fn indoor_walk(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    to: Point,
) {
    let before = controller.snapshot().expect("indoor location");
    let m: MapModule = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(&before.overworld.map_name)
        .expect("room");
    let mut grid = RouteGrid {
        floor: BTreeSet::new(),
        blocked: m.objects.iter().map(|o| (o.x as i16, o.y as i16)).collect(),
        hazards: BTreeSet::new(),
        ice: BTreeSet::new(),
        doors: BTreeMap::new(),
    };
    for y in 0..m.attributes.height * 2 {
        for x in 0..m.attributes.width * 2 {
            let block =
                m.blocks[usize::from(y / 2) * usize::from(m.attributes.width) + usize::from(x / 2)];
            let k = &runtime.data().tilesets[&m.attributes.tileset_name].collision
                [&format!("{block:02x}")][usize::from(y % 2) * 2 + usize::from(x % 2)];
            if matches!(k.as_str(), "FLOOR" | "WARP_CARPET_DOWN" | "LADDER") {
                grid.floor.insert((x as i16, y as i16));
            }
        }
    }
    let path = grid.path((before.overworld.tile.x, before.overworld.tile.y), to);
    for button in path {
        let old = controller
            .snapshot()
            .expect("room before step")
            .overworld
            .tile;
        for _ in 0..8 {
            controller.press(button).expect("native room movement");
            let now = controller.snapshot().expect("room step");
            if now.overworld.map_name != before.overworld.map_name {
                return;
            }
            if now.overworld.tile != old {
                break;
            }
        }
    }
}

#[test]
fn authored_route_planner_preserves_dry_steps_and_skips_uncontrollable_ice_positions() {
    let grid = RouteGrid {
        floor: BTreeSet::from([(0, 0), (1, 0), (5, 0), (6, 0), (0, 1), (1, 1)]),
        ice: BTreeSet::from([(2, 0), (3, 0), (4, 0)]),
        blocked: BTreeSet::new(),
        hazards: BTreeSet::new(),
        doors: BTreeMap::new(),
    };
    assert_eq!(
        grid.path((0, 0), (1, 1)).len(),
        2,
        "ordinary dry route remains tile by tile"
    );
    assert_eq!(
        grid.path((0, 0), (6, 0)),
        vec![GameButton::Right; 3],
        "single input owns full ice course"
    );
    assert_eq!(
        grid.path((6, 0), (0, 0)),
        vec![GameButton::Left; 3],
        "native reverse reaches original dry approach"
    );
    assert!(
        !grid
            .destinations((1, 0))
            .iter()
            .any(|(p, _)| grid.ice.contains(p)),
        "no planned mid-slide stops"
    );
    let mut occupied = grid;
    occupied.blocked.insert((3, 0));
    assert!(
        !occupied
            .destinations((1, 0))
            .iter()
            .any(|(p, _)| *p == (5, 0)),
        "ice never bypasses actor collision"
    );
    occupied.blocked.clear();
    occupied.hazards.insert((3, 0));
    assert!(
        !occupied
            .destinations((1, 0))
            .iter()
            .any(|(p, _)| *p == (5, 0)),
        "ice never bypasses trainer sight"
    );
}
