# Minnesota cave routes and discoveries

The selected v04 source combines eight cave rooms in `rooms.json`. Its build
starts from the published refinement v02 packs. The room notes explain the individual changes.
The regional `story.json` supplies the installed dialogue.

| Room | Player-facing improvement | Rewards |
| --- | --- | --- |
| Cedar Shelter, `MnR13Cave1F` | A northern bank completes a dry loop around the pool, with a narrow connection to the healer's chamber. | New hidden Max Repel; existing Max Ether retained. |
| Granite Den, `MnR17Cave1F` | West climbing steps and a true one-way ledge make the return from the cache shorter. | Existing Revive and hidden Spell Tag retained. |
| Route20 cave, `MnR20Cave1F` | An eastern side passage reconnects two levels; a dry foothold in the seep rewards Surf exploration. | New hidden Max Revive and Surf-only Full Restore; existing Hyper Potion and hidden Revive retained. |

Each room in `rooms.json` follows the existing cave source schema and records its exact
map preimage, rows, arrivals, actor-placement policy, rewards and navigation
contracts. `requires_surf:true` on the seep reward is an authoring validator
contract; the installed item still uses ordinary native hiddenitem behavior
and a persistent event receipt.

## Why these rooms

The native source-art atlas was reviewed across all 19 rendered Minnesota cave
and mine rooms, then compared with the supplied original Union Cave, Dark
Cave and Mt. Mortar maps. Their useful lessons are choices around terrain,
worthwhile side discoveries, different outward/return routes and a clear mix
of visible supplies and hidden items. A new ladder, switch or trainer is not
required in every room. Actual scripts were also read at
[Union Cave](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/UnionCave1F.asm),
[Dark Cave](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/DarkCaveVioletEntrance.asm)
and [Mt. Mortar](https://raw.githubusercontent.com/pret/pokecrystal/master/maps/MountMortar1FInside.asm).

The weakest safe remaining candidates were the pool's dead western bank at
Cedar Shelter, the Granite Den's identical outward/return walk, and Route20's
empty eastern wall beside a decorative seep. These revisions make those
features useful while preserving each room's size and existing identity.

The remaining rooms need different treatment.

- `MnR05RockCut` already has two useful mouths, native climbing stairs, a
  return ledge, a healer, visible Repel and hidden Hard Stone. It was retained
  after reviewing the actual selected map rather than changed for a quota.
- `MnR03Cave1F` has a water chamber, existing cave activities and movable-rock
  concerns. Its native events need a dedicated progression-aware revision
  before changing those paths.
- `MnR09Cave1F` has block-changing programs; `MnInnerBasin` has nine terrain
  mutations. Their saved geometry cannot be discarded as though it were an
  ordinary static cave.
- `MnDriftlessLampGallery` is a dark side-discovery with its own lighting
  behavior. Keep the light/reward contract intact when revisiting its layout.
- The six Iron mine rooms have the weakest remaining large empty galleries,
  especially the Drill Core and Sump. They are machinery/story spaces with
  stateful progression. Redesign them together with their valves, elevators,
  battles and return states; a static wall rearrangement would not be enough.
- The five first-pass rooms are the Driftless passage, Ore Line, Gunflint
  cave and two Voyageurs ice floors. Gunflint gains a single landmark rock.
  The Ore Line and Gunflint rewards now sit on inspectable rocks. Driftless gains a north entrance and a one-way western ledge in the east reward alcove. The existing Escape Rope sits on its search rock, and the return to the east exit takes seven movements through the drop or fourteen via the entrance.

## Evidence and limits

R13 and R17 preserve their original actors, event receipts and non-dialogue programs. Their static
graphs check retained entrances, both itemball states, hidden-item approaches,
return paths, native directional collision and closed water boundaries. The
Granite Den's HOP_DOWN departure `(7,8)` reaches `(7,10)` and rejects the reverse;
the hidden corner returns in six edges versus eleven without the ledge.

The compiled build checks every combination of native trainer approach
endpoints, before and after collection. All six cave trainers use fixed
three-tile sightlines. Each triggering player position must remain reachable
after the trainer moves. The Route 20 Full Restore requires Surf. The upper
Max Revive, Cedar Max Repel, Ore Line Nugget and Gunflint Elixir sit on their
visible rock tiles. Facing the stone from an accessible neighbor uses the
native hidden-item interaction and its existing event receipt.

All three rooms have no Strength/Cut objects, coordinate scenes, terrain-write
programs or hide-only story collectibles. The room notes identify exact old
receipts and safe arrivals used by the bounded migration.

The release check covers all 355 Minnesota maps. The required first passage stays
lit. Only the optional Lamp Gallery requires Flash, and the opening route has
compatible local Pokémon in the morning, during the day and at night.
