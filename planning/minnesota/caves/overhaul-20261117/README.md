# Minnesota cave revision

This overlay addresses five exploration rooms in the installed refinement game.
It accepts that cave geometry/program preimage even when the eight-Gym overlay
has already been installed. It does not replay historical cave builders.

## Mainline references

The design comparison uses the actual event/object sources, alongside static
native-art renders from the locally supplied Crystal pack:

- [Mt. Moon 1F](https://github.com/pret/pokered/blob/master/data/maps/objects/MtMoon1F.asm):
  branching ladder approaches, trainers and visible rewards punctuate the first
  dungeon. Exploration has a nearby payoff before the next branch.
- [Union Cave 1F](https://github.com/pret/pokecrystal/blob/master/maps/UnionCave1F.asm)
  and [B1F](https://github.com/pret/pokecrystal/blob/master/maps/UnionCaveB1F.asm):
  a through-route coexists with optional deeper branches and later visits. The
  cave remains useful after its first crossing.
- [Dark Cave](https://github.com/pret/pokecrystal/blob/master/maps/DarkCaveVioletEntrance.asm)
  and [Whirl Islands B1F](https://github.com/pret/pokecrystal/blob/master/maps/WhirlIslandB1F.asm):
  distinct mouths, recognizable terrain and hidden rewards make detours matter.
- [Ice Path B1F](https://github.com/pret/pokecrystal/blob/master/maps/IcePathB1F.asm)
  and [B2F](https://github.com/pret/pokecrystal/blob/master/maps/IcePathB2FMahoganySide.asm):
  stopping positions, landing geometry and optional pickups define a readable
  navigation problem. This revision uses fixed native rock stops and dry shelves;
  it does not claim to implement Ice Path's falling-boulder mechanism.

No disassembly, map exports, ROMs, extracted graphics catalog or generated pack
is added to tracked source.

## Authored changes

| Cave | Player-facing change |
| --- | --- |
| Driftless hollow | The previously empty eastern extension wraps a fossil ridge, with an optional novice sightline battle and a hidden Repel on the lower branch. The western HM donor and Lamp Gallery entrance are preserved. |
| Route 10 gallery | An upper ore fork connects the west galleries to the eastern spring, closing a circuit around the central rock mass. A third sightline trainer, a hidden Nugget and an Ether reward distinct detours. The existing Revive, Escape Rope and two trainers retain their identities. |
| Gunflint grotto | The quiet healing alcove joins an upper rim circuit. The flooded basin retains a Surf-only island with its original reward. A voluntary angler battle occupies the pool rim; an Elixir rewards the upper mineral pocket. Healing remains reachable without entering the angler's sightline. |
| Voyageurs 1F | Native ice-rock quadrants create deliberate stopping alignments between the entrance, existing Ice Heal pocket, middle dry landing and lower-floor ladder. The middle landing contains a hidden Full Heal. |
| Voyageurs B1F | Two slide basins meet through a narrow central connection. Their rock stops create distinct approaches to the retained NeverMeltIce and Max Revive alcoves and a new hidden PP Up in the northeast pocket. |

The actual pack inventory covered 19 natural/ice/mine rooms. The Iron story
complex, Inner Basin puzzle and smaller existing caves were inspected but are
outside this overlay. Their story machinery is not silently replaced.

## Preservation and validation

`rooms.json` is the explicit native-metatile design and contract. `story.json`
is the canonical prose authority; `story-fragment.json` is the review handoff
for merging its `cave_overhaul` section into that authority.

Per-map SHA256 preimages cover blocks, objects, events and scripts. Existing
room dimensions, attributes, warp coordinates/order/destinations, object
identities/order, story programs and encounter tables are preserved. Three
trainer definitions and nine event constants append; saved numeric identifiers
are never reassigned. Existing collectible actors and hidden items use native
event receipts. None of these five rooms has a Strength/Cut object, changeblock
program, coordinate-event scene or collectible whose ownership exists only in
an object-visibility snapshot.

The installer checks a graph of native collision steps and ledge jumps, settling
ice slides before adding graph edges. It requires all entrances, NPCs, item
approaches, hidden-item landmarks and authored waypoints to remain reachable.
Every reachable resting position must have a return route. The checks run with
item balls present and removed, so collecting a reward cannot remove a required
ice stop. The Gunflint island must be reachable with Surf and unreachable on
foot. Every trainer retains native `OBJECTTYPE_TRAINER` sightline dispatch.

This is static data/geometry validation and native-art review, not gameplay or
controller verification. No game/client is launched for these checks, following
the user's explicit instruction. Root release work owns compilation, tests,
combined preservation checks, bounded save migration and deployment.
