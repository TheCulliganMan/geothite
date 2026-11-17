//! Publication graph over geometry, control choices and monotonic earned flags.
//!
//! This never runs a controller, battle, script interpreter or player save. It
//! checks static resting positions and every reachable authored control choice.
//! Battle wins are abstract transitions; no win rate or runtime input behavior
//! is inferred from this graph.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::{Terrain, describe_collision, permissions};

use crate::gym_design::Gym;
use crate::northstar_gym_design::RoomContract;
use crate::northstar_gym_surface::{Surface, adjacent};
use crate::{CompiledGamePack, MapModule};

type Node = (RoomState, [u16; 2]);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct RoomState {
    flags: u32,
    trainer_positions: [[u16; 2]; 3],
}

impl RoomState {
    fn new(flags: u32, gym: &Gym) -> Result<Self> {
        ensure!(gym.trainers.len() <= 3, "bounded supporting Gym cast");
        let mut positions = [[0; 2]; 3];
        for (position, trainer) in positions.iter_mut().zip(&gym.trainers) {
            *position = trainer.at;
        }
        Ok(Self {
            flags,
            trainer_positions: positions,
        })
    }

    fn with_flags(self, flags: u32) -> Self {
        Self { flags, ..self }
    }
}

struct Flags {
    bits: BTreeMap<String, u32>,
}

impl Flags {
    fn new(gym: &Gym) -> Result<Self> {
        let names: BTreeSet<_> = gym
            .trainers
            .iter()
            .map(|trainer| format!("EVENT_{}_DEFEATED", trainer.id))
            .chain(
                gym.routers
                    .iter()
                    .flat_map(|router| router.modes.iter().map(|mode| mode.event.clone())),
            )
            .chain(gym.cut_gates.iter().map(|gate| gate.event.clone()))
            .chain([gym.defeated()])
            .collect();
        ensure!(names.len() <= 20, "bounded Gym state graph");
        Ok(Self {
            bits: names
                .into_iter()
                .enumerate()
                .map(|(index, name)| (name, 1 << index))
                .collect(),
        })
    }
    fn has(&self, state: u32, name: &str) -> bool {
        self.bits.get(name).is_some_and(|bit| state & bit != 0)
    }
    fn set(&self, state: u32, name: &str) -> Result<u32> {
        Ok(state | self.bits.get(name).context("declared Gym state flag")?)
    }
    fn qualify(&self, state: u32, gym: &Gym, contract: &RoomContract) -> bool {
        self.has(state, &gym.defeated())
            || (contract.court.all.iter().all(|name| self.has(state, name))
                && (contract.court.any.is_empty()
                    || contract.court.any.iter().any(|name| self.has(state, name))))
    }
    fn requires(
        &self,
        state: u32,
        requirements: &[String],
        gym: &Gym,
        contract: &RoomContract,
    ) -> Result<bool> {
        for name in requirements {
            if name == &gym.court_event() {
                if !self.qualify(state, gym, contract) {
                    return Ok(false);
                }
            } else {
                ensure!(
                    self.bits.contains_key(name),
                    "unmodelled Gym requirement {name}"
                );
                if !self.has(state, name) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    fn surface(
        &self,
        state: RoomState,
        pack: &CompiledGamePack,
        base: &MapModule,
        gym: &Gym,
        contract: &RoomContract,
        symbols: &BTreeMap<char, u16>,
    ) -> Result<Surface> {
        let mut room = base.clone();
        for gate in &gym.cut_gates {
            if self.has(state.flags, &gate.event) {
                let index = usize::from(gate.tree_core[1] / 2) * usize::from(room.attributes.width)
                    + usize::from(gate.tree_core[0] / 2);
                room.blocks[index] = symbols[&gate.replacement_symbol];
            }
        }
        for gate in &gym.gates {
            let index = usize::from(gate.at[1]) * usize::from(room.attributes.width)
                + usize::from(gate.at[0]);
            // Match the idempotent callback's open and closed writes, including
            // explicit ordering for controls that share a physical metatile.
            room.blocks[index] = if (contract.victory_opens_gates
                && self.has(state.flags, &gym.defeated()))
                || self.requires(state.flags, &gate.requires, gym, contract)?
            {
                symbols[&gate.open_symbol]
            } else {
                base.blocks[index]
            };
        }
        for (trainer, at) in gym.trainers.iter().zip(state.trainer_positions) {
            let actor = room
                .objects
                .iter_mut()
                .find(|actor| actor.object_identifier.as_deref() == Some(trainer.id.as_str()))
                .context("retained trainer actor in Gym graph")?;
            [actor.x, actor.y] = at;
        }
        let surface = Surface::new(pack, &room)?;
        for at in state.trainer_positions.iter().take(gym.trainers.len()) {
            ensure!(
                surface.permission(*at)? == permissions::FLOOR,
                "control changes a trainer's resting floor at {at:?}"
            );
        }
        Ok(surface)
    }
}

pub(crate) fn validate(
    pack: &CompiledGamePack,
    room: &MapModule,
    gym: &Gym,
    contract: &RoomContract,
    symbols: &BTreeMap<char, u16>,
) -> Result<()> {
    let occupied: BTreeSet<_> = room
        .objects
        .iter()
        .map(|actor| [actor.x, actor.y])
        .collect();
    ensure!(
        occupied.len() == room.objects.len() && !occupied.contains(&gym.arrival_core),
        "safe unique Gym actors and arrival"
    );
    let base = Surface::new(pack, room)?;
    ensure!(
        base.permission(gym.arrival_core)? == permissions::FLOOR,
        "base arrival requires ordinary dry floor"
    );
    for at in gym.exit_tiles_core {
        ensure!(
            base.permission(at)? == permissions::WARP_CARPET_DOWN,
            "native exit carpet"
        );
    }
    for at in &occupied {
        ensure!(
            base.permission(*at)? == permissions::FLOOR,
            "actor needs ordinary floor at {at:?}"
        );
    }
    crate::gym_lifts::validate(pack, room, gym, &occupied)?;
    crate::gym_cut::validate_geometry(pack, room, gym, &occupied)?;
    let flags = Flags::new(gym)?;
    for name in contract.court.all.iter().chain(&contract.court.any) {
        ensure!(
            flags.bits.contains_key(name),
            "court consumes a real earned/control flag: {name}"
        );
    }
    // Migration retains old battle/Cut receipts but starts new controls unset.
    // Include every such receipt combination at the safe base arrival.
    let receipts: Vec<_> = flags
        .bits
        .iter()
        .filter(|(name, _)| !name.contains("_V2_"))
        .map(|(_, bit)| *bit)
        .collect();
    let fresh = (RoomState::new(0, gym)?, gym.arrival_core);
    let initial: BTreeSet<_> = (0..1_u32 << receipts.len())
        .map(|selection| {
            let state = receipts
                .iter()
                .enumerate()
                .filter(|(index, _)| selection & (1 << index) != 0)
                .fold(0, |state, (_, bit)| state | bit);
            (fresh.0.with_flags(state), gym.arrival_core)
        })
        .collect();
    let mut queue: VecDeque<_> = initial.iter().copied().collect();
    let mut nodes = initial;
    let mut reverse: BTreeMap<Node, Vec<Node>> = BTreeMap::new();
    let mut surfaces = BTreeMap::new();
    let mut victories = BTreeSet::new();
    // A required encounter may move its actor before the battle. Optional
    // routes must remain available from every such completion, not merely
    // from a favorable initial approach that leaves all actors at home.
    let mut required_victories = BTreeSet::new();
    let mut trainers = BTreeSet::new();
    let mut sightlines = BTreeSet::new();
    let mut controls = BTreeSet::new();
    let mut exits = VecDeque::new();
    while let Some(node @ (state, at)) = queue.pop_front() {
        ensure!(nodes.len() <= 100_000, "Gym graph exceeded authored bound");
        if let std::collections::btree_map::Entry::Vacant(entry) = surfaces.entry(state) {
            entry.insert(flags.surface(state, pack, room, gym, contract, symbols)?);
        }
        let surface = &surfaces[&state];
        reject_water_bypass(surface, gym, at)?;
        let mut next: Vec<_> = surface
            .edges(at, gym)?
            .into_iter()
            .map(|to| (state, to))
            .collect();
        let mut forced_encounter = None;
        if at == gym.arrival_core || gym.exit_tiles_core.contains(&at) {
            exits.push_back(node);
        }
        for (index, trainer) in gym.trainers.iter().enumerate() {
            let win = format!("EVENT_{}_DEFEATED", trainer.id);
            let trainer_at = state.trainer_positions[index];
            if !flags.has(state.flags, &win)
                && let Some(approach) =
                    sightline_approach(surface, at, trainer_at, &contract.sightlines[&trainer.id])?
            {
                ensure!(
                    flags.requires(state.flags, &trainer.requires, gym, contract)?,
                    "sight encounter precedes a physical prerequisite: {}",
                    trainer.id
                );
                if forced_encounter.is_none() {
                    sightlines.insert(&trainer.id);
                    let mut won = state.with_flags(flags.set(state.flags, &win)?);
                    won.trainer_positions[index] = approach;
                    forced_encounter = Some(won);
                }
            }
            if adjacent(at, trainer_at) {
                trainers.insert(&trainer.id);
                // Native trainer encounters do not run an extra qualification
                // script. Physical geometry must enforce any prerequisite.
                if !flags.has(state.flags, &win) {
                    ensure!(
                        flags.requires(state.flags, &trainer.requires, gym, contract)?,
                        "trainer accessible before physical prerequisite: {} at {at:?}",
                        trainer.id
                    );
                    next.push((state.with_flags(flags.set(state.flags, &win)?), at));
                }
            }
        }
        for router in &gym.routers {
            let accessible = if let Some(operator) = &router.operator {
                let index = gym
                    .trainers
                    .iter()
                    .position(|trainer| &trainer.id == operator)
                    .context("control operator")?;
                adjacent(at, state.trainer_positions[index])
                    && flags.has(state.flags, &format!("EVENT_{operator}_DEFEATED"))
            } else {
                adjacent(at, router.at)
            };
            if accessible && forced_encounter.is_none() {
                controls.insert(&router.id);
                if contract.victory_opens_gates && flags.has(state.flags, &gym.defeated()) {
                    continue;
                }
                for mode in &router.modes {
                    if flags.requires(state.flags, &mode.requires, gym, contract)? {
                        let mut changed = state.flags;
                        for other in &router.modes {
                            changed &= !flags.bits[&other.event];
                        }
                        next.push((state.with_flags(flags.set(changed, &mode.event)?), at));
                    }
                }
            }
        }
        for gate in &gym.cut_gates {
            if at == gate.approach_core {
                next.push((state.with_flags(flags.set(state.flags, &gate.event)?), at));
            }
        }
        if forced_encounter.is_none()
            && adjacent(at, gym.leader_at)
            && flags.qualify(state.flags, gym, contract)
        {
            victories.insert(node);
            next.push((
                state.with_flags(flags.set(state.flags, &gym.defeated())?),
                at,
            ));
        }
        // An undefeated trainer owns input as soon as this sightline is
        // entered. The graph may assume a win, but cannot walk through it or
        // operate a control first. Leaders remain explicit adjacent actions.
        if let Some(won) = forced_encounter {
            next = vec![(won, at)];
        }
        for target in next {
            // A control must never close the tile occupied by its operator.
            if target.0 != state {
                let changed = flags.surface(target.0, pack, room, gym, contract, symbols)?;
                ensure!(
                    changed.permission(at)? == surface.permission(at)? && !changed.occupied(at)?,
                    "control or approach changes player footing at {at:?}"
                );
            }
            for trainer in gym.trainers.iter().filter(|trainer| !trainer.optional) {
                let win = flags.bits[&format!("EVENT_{}_DEFEATED", trainer.id)];
                if state.flags & win == 0 && target.0.flags & win != 0 {
                    required_victories.insert(target);
                }
            }
            reverse.entry(target).or_default().push(node);
            if nodes.insert(target) {
                queue.push_back(target);
            }
        }
    }
    ensure!(
        qualification_routes_without(&reverse, &victories, 0).contains(&fresh),
        "fresh Gym arrival has no qualified route to the leader"
    );
    if contract.court.open {
        let trainer_wins = gym.trainers.iter().fold(0, |wins, trainer| {
            wins | flags.bits[&format!("EVENT_{}_DEFEATED", trainer.id)]
        });
        ensure!(
            qualification_routes_without(&reverse, &victories, trainer_wins).contains(&fresh),
            "the open court must be reachable without fighting either supporting trainer"
        );
    }
    for trainer in gym.trainers.iter().filter(|trainer| trainer.optional) {
        let win = flags.bits[&format!("EVENT_{}_DEFEATED", trainer.id)];
        let routes = qualification_routes_without(&reverse, &victories, win);
        let blocked = required_victories
            .iter()
            .find(|node| node.0.flags & win == 0 && !routes.contains(node));
        // Do not require this after committing to the optional encounter's own
        // sightline. This checks the entrance and required battle completions.
        ensure!(
            routes.contains(&fresh) && blocked.is_none(),
            "optional trainer {} cannot be bypassed after native approach movement; fresh route: {}; required battle endpoint: {blocked:?}",
            trainer.id,
            routes.contains(&fresh)
        );
    }
    ensure!(
        trainers.len() == gym.trainers.len(),
        "unreachable optional/required Gym trainer"
    );
    ensure!(
        sightlines.len() == gym.trainers.len(),
        "every Gym trainer needs a reachable sight encounter"
    );
    ensure!(
        controls.len() == gym.routers.len(),
        "unreachable Gym control"
    );
    let mut escapable = BTreeSet::new();
    while let Some(node) = exits.pop_front() {
        if escapable.insert(node) {
            exits.extend(reverse.get(&node).into_iter().flatten().copied());
        }
    }
    let trapped: Vec<_> = nodes.difference(&escapable).take(8).collect();
    ensure!(
        trapped.is_empty(),
        "Gym has reachable states without an exit: {trapped:?}"
    );
    eprintln!(
        "{}: {} static room states, {} terrain/actor configurations; leader, trainers and returns reachable",
        gym.id,
        nodes.len(),
        surfaces.len()
    );
    Ok(())
}

fn reject_water_bypass(surface: &Surface, gym: &Gym, [x, y]: [u16; 2]) -> Result<()> {
    for at in [
        x.checked_sub(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        (x + 1 < gym.dimensions_metatiles[0] * 2).then_some([x + 1, y]),
        (y + 1 < gym.dimensions_metatiles[1] * 2).then_some([x, y + 1]),
    ]
    .into_iter()
    .flatten()
    {
        ensure!(
            describe_collision(surface.permission(at)?).terrain != Terrain::Water,
            "Gym walking course exposes a Surf/canoe bypass at {at:?}"
        );
    }
    Ok(())
}

fn sightline_approach(
    surface: &Surface,
    player: [u16; 2],
    trainer: [u16; 2],
    sight: &crate::northstar_gym_design::Sightline,
) -> Result<Option<[u16; 2]>> {
    let dx = i32::from(player[0]) - i32::from(trainer[0]);
    let dy = i32::from(player[1]) - i32::from(trainer[1]);
    let (length, step) = match sight.direction.as_str() {
        "up" if dx == 0 && dy < 0 => (-dy, [0, -1]),
        "down" if dx == 0 && dy > 0 => (dy, [0, 1]),
        "left" if dy == 0 && dx < 0 => (-dx, [-1, 0]),
        "right" if dy == 0 && dx > 0 => (dx, [1, 0]),
        _ => return Ok(None),
    };
    if length > i32::from(sight.radius) {
        return Ok(None);
    }
    for distance in 1..length {
        let at = [
            u16::try_from(i32::from(trainer[0]) + step[0] * distance)?,
            u16::try_from(i32::from(trainer[1]) + step[1] * distance)?,
        ];
        ensure!(
            surface.permission(at)? == permissions::FLOOR && !surface.occupied(at)?,
            "native trainer sightline crosses a wall or forced surface at {at:?}"
        );
    }
    Ok(Some([
        u16::try_from(i32::from(player[0]) - step[0])?,
        u16::try_from(i32::from(player[1]) - step[1])?,
    ]))
}

fn qualification_routes_without(
    reverse: &BTreeMap<Node, Vec<Node>>,
    victories: &BTreeSet<Node>,
    omitted_victory: u32,
) -> BTreeSet<Node> {
    let mut todo: VecDeque<_> = victories.iter().copied().collect();
    let mut visited = BTreeSet::new();
    while let Some(node) = todo.pop_front() {
        if node.0.flags & omitted_victory != 0 || !visited.insert(node) {
            continue;
        }
        todo.extend(reverse.get(&node).into_iter().flatten().copied());
    }
    visited
}

#[cfg(test)]
#[path = "northstar_gym_validation_tests.rs"]
mod tests;
