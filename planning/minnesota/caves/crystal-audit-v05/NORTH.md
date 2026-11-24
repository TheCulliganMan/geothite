# Northern caves compared with Pokémon Crystal

## What the selected v04 maps miss

I reviewed the complete original Game Boy Color floor pictures and then the
selected Minnesota floors and their 160×144 approaches. The floor pictures
make the scale difference clear. Crystal makes the exposed part of a ledge,
the width of a passage and the position of a bank do different jobs. A change
in the wall's surface usually accompanies a visible change in where the player
can stand. Several Minnesota scenes exhaust the camera with the same fine
stipple on solid rock, passable ground and water. A background color change or
an isolated boulder rarely gives a lost player a usable heading. The problem
is most pronounced in Inner Basin at noon. [Original cave pictures](../crystal-reference.html)

### Winding Cleft

Dark Cave uses its water and long protruding rock to make a Z-shaped route
with separate passages and rewards. Successive turns around the rock give
different views. Winding Cleft reads as a long horizontal hall around an
equally horizontal pool. Both mouths look onto long, flat foregrounds. From
the scout, the stair announces a descent but the named rock is out of frame.
The lookout adds another stair and a dead end even though the landmark is the
reason to choose that path. The long walk around the protected westward jump
proves it is a shortcut, but the westbound route itself hardly changes visual
character across the lower bank. The isolated midlevel and upper caches have
little terrain separating them from the through journey. [Crystal's Dark Cave](https://vgmaps.com/Atlas/GB-GBC/Pokemon-CrystalVersion-Johto-DarkCave.png)

### Gunflint Grotto

Union Cave alternates pools, purposeful banks and small shelves. Tohjo Falls
gives one compact room a visible water direction. Gunflint's almost rectangular
waterline creates a dry perimeter and one ample blue rectangle. The dry journey
recovers the same broad view on successive landings and the Surf journey
crosses open water to one unremarkable square rock. The optional high supply
starts within sight of its own stair and lacks a distinct pocket. Otta and the
native timber bench make a good destination, but the camp is closed off from the
spring it overlooks in dialogue. The apparent width of the bank around the
angler obscures the division between rock face, route and waterfront. [Crystal's Union Cave](https://vgmaps.com/Atlas/GB-GBC/Pokemon-CrystalVersion-Johto-UnionCave.png) · [Crystal's Tohjo Falls](https://vgmaps.com/Atlas/GB-GBC/Pokemon-CrystalVersion-Kanto-TohjoFalls.png)

### Ore Line

Mt. Mortar builds separate trips around large shelves. Each stair announces a
new space and hidden rocks sit against a recognizably different bank or ledge.
Ore Line has a promising fork by the entrance, but the descent and ascent
are stretched out around several indistinct open patches. On the survey path,
the real fracture is only another dark mark in a narrow gap. Broad nearby
floors similarly frame the ordinary item and the locked seam. The bottom
spring has a big blocky pool and almost equally broad banks, despite the fishing
spot and the remote reward being different motivations. The safe exit when
the fracture respawns is worth retaining and deserves stronger topographic
foreshadowing. [Crystal's Mt. Mortar](https://vgmaps.com/Atlas/GB-GBC/Pokemon-CrystalVersion-Johto-MtMortar.png)

### Inner Basin

Dragon's Den uses a dominant lake and distinct shore to set up a destination
across water before the player can reach it. Inner Basin shows the lower pool
and separate upper reservoir, but the long brown stretches around the dials
and wall take up much of ordinary play. The levee reads like a long decorative
stripe before the lever changes it. The empty upper-right slope and Kensigard's
corner read as more of the same brown ground. The sanctuary is hard to
anticipate while flooded and easy to mistake for ordinary rock when drained.
The gold day palette washes several pale wall/floor transitions together, then
the saturated blue takes attention away from the puzzle's landmarks. The
drain already changes collision, so the most valuable fix is to give the
newly revealed geography a clear shape before adding detail. [Crystal's Dragon's Den](https://vgmaps.com/Atlas/GB-GBC/Pokemon-CrystalVersion-Johto-Dragon%27sDen.png)

## What changed

- **Winding Cleft** turns around a stone projection and shorter bank chambers.
  The real seep cache moved to the sheltered side of the central shelf. The
  scout now stands inside the turn with the rock in the western conversation
  camera. A lower overlook gives a second view and the cache still requires
  Surf. The westward drop keeps the return faster.
- **Gunflint Grotto** brings a narrow inlet up beside the camp and gives
  the spring a stone finger and offset landings. The camp-side launch is
  connected to the whole spring. A walk leads down the contoured bank and
  through two stair landings. The high supply sits behind the far shelf,
  beyond the first stair's view. The Revive remains out on the spring.
- **Ore Line** contracts the upper survey route around a rock shelf and
  folds the descent toward the spring. Water cuts off the straight dry reach
  to the supply shore. The player can walk around the south rim or swim the
  shorter cut. The upper Star Piece remains a separate Rock Smash trip with
  a hop escape if the fracture returns behind the player. Dark floor beside
  two broken boulders makes the narrow approach visible in both fracture states.
- **Inner Basin** adds a narrower dial bank around the lower water and a
  stone perch visible from the lever on the flood sill. Pulling it reveals stairs,
  exposes shadowed rock in the former upper reservoir, and replaces the
  perch's paired stones with Kensigard. The visible ledge stays in the same
  place. A forced night lighting group uses the cave's intended night palette
  at noon as well as at night. The original nine terrain changes still
  reopen from the same event flag.

| Room | Metatiles changed | Measured route in v04 and v05 |
| --- | ---: | --- |
| Winding Cleft | 41 | Eastbound 64 → 62. Westbound by the drop stays 43, saving 19 steps against the new ordinary path. |
| Gunflint Grotto | 49 | Far supply by walking or Surf changes from 62 / 30 to 66 / 34. |
| Ore Line | 59 | Spring supply by walking or Surf changes from 40 / 38 to 56 / 42. Restored-fracture egress stays 39 and the opened Star Piece stays 44. |
| Inner Basin | 54 | Lever access changes from 42 to 32. After draining, the sanctuary changes from 42 to 39. |

Steps count moves from arrival to a usable tile. They exclude interacting with
objects, operating field moves and triggering warps. The source review found
an inaccessible four-tile pocket under the Cleft return and seven false floor
tiles beside Ore Line's survey ledge, then removed them and checked the full
routes again. The largest empty squares in Gunflint and Ore Line shrink from
five walking tiles across to four. Cleft and Basin remain at four. The new
wet and dry routes have complete returns, including while ordinary items
block their own positions and after they are picked up. The Basin perch
remains blocked from both approaches until the outlet opens.

Run the comparison on the compiled parent and candidate to save complete
floors, paired 160×144 views and the measurements to a new ignored directory.

```sh
uv run --with pillow --with cbor2 python \
  planning/minnesota/caves/crystal-audit-v05/review_north_audit.py \
  content-packs/minnesota-cave-detail-v04.crystalpack \
  content-packs/minnesota-cave-audit-v05.crystalpack \
  target/north-v05/compiled-comparison
```
