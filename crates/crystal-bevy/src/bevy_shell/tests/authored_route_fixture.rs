//! Fixture-only path planning; every gameplay step uses the production controller.
use super::super::VisibleShellController;
use crystal_assets::MapModule;
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
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
    doors: BTreeMap<Point, &'static str>,
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
            for (q, _) in neighbors(p) {
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

    pub fn path(&self, from: Point, to: Point) -> Vec<GameButton> {
        let mut parents = BTreeMap::from([(from, (from, GameButton::A))]);
        let mut queue = VecDeque::from([from]);
        while let Some(p) = queue.pop_front() {
            if p == to {
                break;
            }
            for (q, button) in neighbors(p) {
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
            .expect("talk approach");
        self.walk(controller, approach);
        controller.press(opposite(button)).expect("face actor");
        controller.press(GameButton::A).expect("talk or collect");
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
        let text = s.ui.text.and_then(|t| t.asm_text).unwrap_or_default();
        finish_text(controller);
        text
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
