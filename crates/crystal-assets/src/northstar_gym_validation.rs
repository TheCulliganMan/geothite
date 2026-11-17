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

type Node = (u32, [u16; 2]);

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
        state: u32,
        pack: &CompiledGamePack,
        base: &MapModule,
        gym: &Gym,
        contract: &RoomContract,
        symbols: &BTreeMap<char, u16>,
    ) -> Result<Surface> {
        let mut room = base.clone();
        for gate in &gym.cut_gates {
            if self.has(state, &gate.event) {
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
                && self.has(state, &gym.defeated()))
                || self.requires(state, &gate.requires, gym, contract)?
            {
                symbols[&gate.open_symbol]
            } else {
                base.blocks[index]
            };
        }
        Surface::new(pack, &room)
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
    let initial: BTreeSet<_> = (0..1_u32 << receipts.len())
        .map(|selection| {
            let state = receipts
                .iter()
                .enumerate()
                .filter(|(index, _)| selection & (1 << index) != 0)
                .fold(0, |state, (_, bit)| state | bit);
            (state, gym.arrival_core)
        })
        .collect();
    let mut queue: VecDeque<_> = initial.iter().copied().collect();
    let mut nodes = initial;
    let mut reverse: BTreeMap<Node, Vec<Node>> = BTreeMap::new();
    let mut surfaces = BTreeMap::new();
    let mut victories = false;
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
        for trainer in &gym.trainers {
            let win = format!("EVENT_{}_DEFEATED", trainer.id);
            if !flags.has(state, &win)
                && check_sightline(surface, at, trainer.at, &contract.sightlines[&trainer.id])?
            {
                sightlines.insert(&trainer.id);
                ensure!(
                    flags.requires(state, &trainer.requires, gym, contract)?,
                    "sight encounter precedes a physical prerequisite: {}",
                    trainer.id
                );
                if forced_encounter.is_none() {
                    forced_encounter = Some(flags.set(state, &win)?);
                }
            }
            if adjacent(at, trainer.at) {
                trainers.insert(&trainer.id);
                // Native trainer encounters do not run an extra qualification
                // script. Physical geometry must enforce any prerequisite.
                if !flags.has(state, &win) {
                    ensure!(
                        flags.requires(state, &trainer.requires, gym, contract)?,
                        "trainer accessible before physical prerequisite: {} at {at:?}",
                        trainer.id
                    );
                    next.push((flags.set(state, &win)?, at));
                }
            }
        }
        for router in &gym.routers {
            let accessible = if let Some(operator) = &router.operator {
                let trainer = gym
                    .trainers
                    .iter()
                    .find(|trainer| &trainer.id == operator)
                    .context("control operator")?;
                adjacent(at, trainer.at) && flags.has(state, &format!("EVENT_{operator}_DEFEATED"))
            } else {
                adjacent(at, router.at)
            };
            if accessible {
                controls.insert(&router.id);
                if contract.victory_opens_gates && flags.has(state, &gym.defeated()) {
                    continue;
                }
                for mode in &router.modes {
                    if flags.requires(state, &mode.requires, gym, contract)? {
                        let mut changed = state;
                        for other in &router.modes {
                            changed &= !flags.bits[&other.event];
                        }
                        next.push((flags.set(changed, &mode.event)?, at));
                    }
                }
            }
        }
        for gate in &gym.cut_gates {
            if at == gate.approach_core {
                next.push((flags.set(state, &gate.event)?, at));
            }
        }
        if adjacent(at, gym.leader_at) && flags.qualify(state, gym, contract) {
            victories = true;
            next.push((flags.set(state, &gym.defeated())?, at));
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
                    changed.permission(at)? == surface.permission(at)?,
                    "control changes player footing at {at:?}"
                );
            }
            reverse.entry(target).or_default().push(node);
            if nodes.insert(target) {
                queue.push_back(target);
            }
        }
    }
    ensure!(victories, "Gym leader has no qualified route");
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
        "{}: {} static room states, {} control configurations; leader, trainers and returns reachable",
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

fn check_sightline(
    surface: &Surface,
    player: [u16; 2],
    trainer: [u16; 2],
    sight: &crate::northstar_gym_design::Sightline,
) -> Result<bool> {
    let dx = i32::from(player[0]) - i32::from(trainer[0]);
    let dy = i32::from(player[1]) - i32::from(trainer[1]);
    let (length, step) = match sight.direction.as_str() {
        "up" if dx == 0 && dy < 0 => (-dy, [0, -1]),
        "down" if dx == 0 && dy > 0 => (dy, [0, 1]),
        "left" if dy == 0 && dx < 0 => (-dx, [-1, 0]),
        "right" if dy == 0 && dx > 0 => (dx, [1, 0]),
        _ => return Ok(false),
    };
    if length > i32::from(sight.radius) {
        return Ok(false);
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
    Ok(true)
}
