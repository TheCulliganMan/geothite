//! Fixture-only path planning; every gameplay step uses the production controller.
use super::super::VisibleShellController;
use crystal_assets::MapModule;
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};
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
    battle_handler: Option<fn(&mut VisibleShellController)>,
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
    /// Opts a populated tour into production battle inputs. Default tours reject encounters.
    pub fn with_battle_handler(mut self, handler: fn(&mut VisibleShellController)) -> Self {
        self.battle_handler = Some(handler);
        self
    }

    fn handle_battle(&self, controller: &mut VisibleShellController) {
        if let Some(handler) = self.battle_handler {
            handler(controller);
        }
    }

    pub fn new(runtime: &CrystalRuntime, spec: RouteSpec) -> Self {
        Self::build(runtime, spec, false, false)
    }

    /// Includes grass; populated journeys must settle encounters through input.
    pub fn new_with_grass(runtime: &CrystalRuntime, spec: RouteSpec) -> Self {
        Self::build(runtime, spec, true, false)
    }

    /// Includes trainer sight cells for callers that play every native encounter.
    /// Occupied actor tiles remain blocked; ordinary tours still avoid battles.
    pub fn new_for_battles(runtime: &CrystalRuntime, spec: RouteSpec) -> Self {
        Self::build(runtime, spec, true, true)
    }

    fn build(
        runtime: &CrystalRuntime,
        spec: RouteSpec,
        include_grass: bool,
        include_trainers: bool,
    ) -> Self {
        let mut grid = Self {
            floor: BTreeSet::new(),
            blocked: BTreeSet::new(),
            hazards: BTreeSet::new(),
            ice: BTreeSet::new(),
            doors: spec.doors,
            battle_handler: None,
        };
        let mut trainers = Vec::new();
        for row in 0..spec.rows {
            for col in 0..spec.columns {
                let name = format!("{}C{col:02}R{row:02}", spec.prefix);
                let module = runtime
                    .data()
                    .assemble_map_module_from_compiled_payloads(&name)
                    .expect("published chunk");
                // Regional revisions can give individual chunks their own art
                // bank. Plan against the installed collision, as gameplay does.
                let tileset = runtime
                    .data()
                    .tilesets
                    .get(&module.attributes.tileset_name)
                    .unwrap_or_else(|| {
                        panic!(
                            "missing installed tileset {} for {name}; fixture reference {}",
                            module.attributes.tileset_name, spec.tileset
                        )
                    });
                for y in 0..usize::from(module.attributes.height) * 2 {
                    for x in 0..usize::from(module.attributes.width) * 2 {
                        let block =
                            module.blocks[(y / 2) * usize::from(module.attributes.width) + x / 2];
                        let kind = &tileset.collision[&format!("{block:02x}")][(y % 2) * 2 + x % 2];
                        if kind == "ICE" {
                            grid.ice.insert((col * 64 + x as i16, row * 48 + y as i16));
                        }
                        if matches!(
                            kind.as_str(),
                            "FLOOR"
                                | "LADDER"
                                | "DOOR"
                                | "CAVE"
                                | "BRIDGE_NS_DECK"
                                | "BRIDGE_EW_DECK"
                        ) || (include_grass && kind == "TALL_GRASS")
                        {
                            grid.floor
                                .insert((col * 64 + x as i16, row * 48 + y as i16));
                        }
                    }
                }
                for o in module.objects {
                    let p = (col * 64 + o.x as i16, row * 48 + o.y as i16);
                    grid.blocked.insert(p);
                    if o.object_type == "OBJECTTYPE_TRAINER" && !include_trainers {
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
        self.walk_with_actions(controller, to, |_| {}, |c| self.handle_battle(c));
    }

    pub fn walk_with_step_action(
        &self,
        controller: &mut VisibleShellController,
        to: Point,
        before_step: impl FnMut(&mut VisibleShellController),
    ) {
        self.walk_with_actions(controller, to, before_step, |c| self.handle_battle(c));
    }

    pub fn walk_with_actions(
        &self,
        controller: &mut VisibleShellController,
        to: Point,
        mut before_step: impl FnMut(&mut VisibleShellController),
        mut on_battle: impl FnMut(&mut VisibleShellController),
    ) {
        let from = position(controller);
        for button in self.path(from, to) {
            before_step(controller);
            let before = position(controller);
            let expected = self
                .destinations(before)
                .into_iter()
                .find_map(|(point, input)| (input == button).then_some(point))
                .expect("planned production route destination");
            for _ in 0..8 {
                controller.press(button).expect("native route input");
                let mut s = controller.snapshot().expect("native route step");
                if s.battle.is_some() {
                    on_battle(controller);
                    s = controller
                        .snapshot()
                        .expect("native route after battle input");
                }
                assert!(
                    s.battle.is_none(),
                    "unexpected battle at {before:?}: {:?}",
                    s.battle.as_ref().map(|battle| &battle.kind)
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
    if before.overworld.map_name == "MnMinneapolis" {
        walk_live_city(controller, runtime, to);
        return;
    }
    if runtime.data().maps.contains_key("MnBoundaryIntake")
        && matches!(
            before.overworld.map_name.as_str(),
            "MnRochesterPokecenter1F" | "MnStCloud"
        )
    {
        for _ in 0..1024 {
            let tile = controller.snapshot().expect("live arc walk").overworld.tile;
            if (tile.x, tile.y) == to {
                return;
            }
            let button = *super::northstar_ice_lair::path(controller, runtime, to)
                .expect("native arc route")
                .first()
                .expect("remaining step");
            for _ in 0..8 {
                controller.press(button).expect("native arc step");
                controller
                    .wait_frames(24)
                    .expect("live street presentation");
                super::northstar_rival_arc::settle_walk_scene(controller, runtime);
                if controller
                    .snapshot()
                    .expect("actual arc step")
                    .overworld
                    .tile
                    != tile
                {
                    break;
                }
            }
            assert_ne!(
                controller
                    .snapshot()
                    .expect("walked arc tile")
                    .overworld
                    .tile,
                tile
            );
        }
        panic!("bounded authored arc walk");
    }
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
        battle_handler: None,
    };
    for y in 0..m.attributes.height * 2 {
        for x in 0..m.attributes.width * 2 {
            let block =
                m.blocks[usize::from(y / 2) * usize::from(m.attributes.width) + usize::from(x / 2)];
            let k = &runtime.data().tilesets[&m.attributes.tileset_name].collision
                [&format!("{block:02x}")][usize::from(y % 2) * 2 + usize::from(x % 2)];
            if matches!(
                k.as_str(),
                "FLOOR" | "WARP_CARPET_DOWN" | "LADDER" | "BRIDGE_NS_DECK"
            ) {
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

// Follow a complete route while it remains clear. Replanning every single
// step around moving cyclists can alternate forever between two detours.
fn walk_live_city(c: &mut VisibleShellController, r: &CrystalRuntime, to: Point) {
    for _ in 0..128 {
        let start = c.snapshot().expect("live city route").overworld;
        assert_eq!(
            start.map_name, "MnMinneapolis",
            "city path entered a building"
        );
        if (start.tile.x, start.tile.y) == to {
            return;
        }
        let Some(path) = super::northstar_ice_lair::path(c, r, to) else {
            c.wait_frames(24).expect("wait for passing cyclist");
            continue;
        };
        for button in path {
            let old = c.snapshot().expect("city step origin").overworld.tile;
            let delta = match button {
                GameButton::Up => (0, -1),
                GameButton::Down => (0, 1),
                GameButton::Left => (-1, 0),
                GameButton::Right => (1, 0),
                _ => panic!("city path contains a non-direction"),
            };
            for _ in 0..8 {
                c.press(button).expect("actual city walking input");
                c.wait_frames(24).expect("city walking presentation");
                super::northstar_rival_arc::settle_walk_scene(c, r);
                if c.snapshot().expect("actual city step").overworld.tile != old {
                    break;
                }
            }
            let now = c.snapshot().expect("city route continuation").overworld;
            assert_eq!(now.map_name, "MnMinneapolis", "unexpected city doorway");
            if (now.tile.x, now.tile.y) != (old.x + delta.0, old.y + delta.1) {
                // A passing actor or an authored scene changed this route.
                break;
            }
            if (now.tile.x, now.tile.y) == to {
                return;
            }
        }
    }
    let actual = c.snapshot().expect("blocked city diagnostic").overworld;
    panic!("bounded city route to {to:?}: actual {actual:?}");
}

#[test]
fn authored_route_planner_preserves_dry_steps_and_skips_uncontrollable_ice_positions() {
    let grid = RouteGrid {
        floor: BTreeSet::from([(0, 0), (1, 0), (5, 0), (6, 0), (0, 1), (1, 1)]),
        ice: BTreeSet::from([(2, 0), (3, 0), (4, 0)]),
        blocked: BTreeSet::new(),
        hazards: BTreeSet::new(),
        doors: BTreeMap::new(),
        battle_handler: None,
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

// Apply carried field medicine entirely through the production Pack UI.
pub(super) fn use_super_potion(controller: &mut VisibleShellController) {
    use_carried_healing_item(controller, "SUPER_POTION", false);
}

pub(super) fn use_revive(controller: &mut VisibleShellController) {
    use_carried_healing_item(controller, "REVIVE", true);
}

fn use_carried_healing_item(controller: &mut VisibleShellController, item_id: &str, revive: bool) {
    finish_text(controller);
    let before_use = controller.snapshot().expect("carried medicine and party");
    let target_partner = before_use
        .party
        .slots
        .iter()
        .position(|p| {
            if revive {
                p.pokemon.hp == 0
            } else {
                p.pokemon.hp > 0 && p.pokemon.hp < p.pokemon.max_hp
            }
        })
        .expect("living injured partner");
    let quantity_before = before_use
        .bag
        .items
        .iter()
        .find(|item| item.item_id == item_id)
        .expect("carried Super Potion")
        .quantity;
    controller
        .press(GameButton::Start)
        .expect("production Start");
    let snapshot = controller.snapshot().expect("Start options");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|o| *o == super::super::StartMenuOption::Pack)
        .expect("PACK option");
    for _ in 0..16 {
        let current = controller
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("open PACK");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select PACK");
    }
    let bag = controller.snapshot().expect("carried medicine").bag;
    let target = bag
        .items
        .iter()
        .filter(|s| s.quantity > 0)
        .position(|s| s.item_id == item_id)
        .expect("collected Super Potion");
    for _ in 0..16 {
        let current = controller
            .shell
            .bag_cursor
            .as_ref()
            .expect("production bag cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("medicine actions");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select medicine");
    }
    let snapshot = controller.snapshot().expect("medicine actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("production item actions");
    let target = actions
        .iter()
        .position(|o| *o == super::super::FieldPackAction::Use)
        .expect("USE action");
    for _ in 0..16 {
        let current = controller
            .shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item action cursor")
            .option_index;
        if current == target {
            controller.press(GameButton::A).expect("use medicine");
            break;
        }
        controller
            .press(if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select USE");
    }
    assert!(
        controller.shell.field_pack_target_mode
            == Some(super::super::FieldPackTargetMode::PartyPokemon),
        "medicine target did not open; selected={:?}, ui={:?}",
        super::super::selected_field_pack_item_id(&mut controller.shell),
        controller
            .snapshot()
            .expect("medicine target diagnostic")
            .ui
    );
    for _ in 0..6 {
        if controller.shell.party_cursor == target_partner {
            break;
        }
        controller
            .press(if controller.shell.party_cursor < target_partner {
                GameButton::Down
            } else {
                GameButton::Up
            })
            .expect("select injured partner");
    }
    assert_eq!(controller.shell.party_cursor, target_partner);
    let before = controller.snapshot().expect("injured partner").party.slots[target_partner]
        .pokemon
        .hp;
    controller
        .press(GameButton::A)
        .expect("apply Super Potion to injured partner");
    let healed = controller.snapshot().expect("native healing result");
    assert!(
        healed.party.slots[target_partner].pokemon.hp > before,
        "medicine did not restore HP: before={before}, after={}, target={:?}, notice={:?}, field_notice={:?}, ui={:?}",
        healed.party.slots[target_partner].pokemon.hp,
        controller.shell.field_pack_target_mode,
        controller.shell.visible_field_item_notice,
        controller.shell.field_notice,
        healed.ui
    );
    assert_eq!(
        healed
            .bag
            .items
            .iter()
            .find(|s| s.item_id == item_id)
            .map_or(0, |s| s.quantity),
        quantity_before - 1,
        "medicine not consumed"
    );
    for _ in 0..16 {
        let current = controller.snapshot().expect("close field Pack");
        if controller.shell.field_notice.is_some() || current.ui.pending_text_wait.is_some() {
            controller
                .wait_frames(60)
                .expect("healing presentation frames");
            controller.press(GameButton::A).expect("healing notice");
        } else if current.ui.menu.is_some() {
            controller
                .press(GameButton::B)
                .expect("close medicine menu");
        } else {
            return;
        }
    }
    let final_ui = controller.snapshot().expect("healing close diagnostic").ui;
    panic!(
        "medicine did not return to walking: field_notice={:?}, item_notice={:?}, ui={:?}",
        controller.shell.field_notice, controller.shell.visible_field_item_notice, final_ui
    );
}

pub(super) fn escape_wild(c: &mut VisibleShellController) {
    let before = c.snapshot().expect("actual encounter before RUN");
    for _ in 0..512 {
        let s = c.snapshot().expect("native RUN input ownership");
        let input = if let Some(battle) = &s.battle {
            assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
            if !c.shell.battle_messages.is_empty() || s.ui.menu.is_none() {
                GameButton::A
            } else if let Some(cursor) = &c.shell.battle_action_cursor {
                match cursor.option_index {
                    0 | 1 => GameButton::Down,
                    2 => GameButton::Right,
                    3 => GameButton::A,
                    _ => panic!("invalid native action cursor"),
                }
            } else {
                GameButton::A
            }
        } else if s.phase == RuntimeShellPhase::Text
            || c.has_pending_script_work()
            || !c.shell.battle_messages.is_empty()
        {
            GameButton::A
        } else {
            assert_eq!(s.phase, RuntimeShellPhase::Overworld);
            break;
        };
        c.press(input).expect("native RUN or narration input");
        c.wait_frames(16).expect("native encounter presentation");
        assert_eq!(c.shell.last_error, None);
    }
    let after = c.snapshot().expect("native RUN must finish");
    assert!(after.battle.is_none());
    assert_eq!(
        before.overworld.map_name, after.overworld.map_name,
        "RUN must not be a blackout"
    );
    assert_eq!(
        before.overworld.tile, after.overworld.tile,
        "RUN preserves the reached tile"
    );
    assert!(after.party.slots.iter().any(|p| p.pokemon.hp > 0));
}
