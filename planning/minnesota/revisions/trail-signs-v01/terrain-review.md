# Route 15 west bank

Keep the west bank of Shiner Narrows walkable. The selected Research Choices Intro and Field archives contain identical terrain here. In the native daytime art, the pale path ends at a continuous brown face and the blue channel begins below it. Walking along that rim makes sense. A dry landing east of it would intrude into the channel. [Route 15 crossing design](../../routes/r15-shiner-narrows-v01/README.md)

| In-game squares in `MnR15C00R00` | Installed behavior |
| --- | --- |
| (24,10) through (24,17) | The player can stand here from either route entrance. The eight squares advertise an eastward hop. Each has a wall at x=25 and water at the prospective landing at x=26. The movement code rejects that landing. |
| (22,18), (23,18), (24,18) | Jumping south clears a wall at y=19 and lands on dry path at y=20. This is the meadow's intended return. The corner at (24,18) also advertises an unusable eastward hop. |
| (26,10) through (26,18) | These water squares connect in both directions to the outlet beneath the bridge. A rider approaching from the east meets the same vertical face. |

In Crystal's Fuchsia City, the comparable brown side face at (22,28) has an eastward hop with visible dry ground at (24,28). On Route 15, the immediate water and the deep brown shoreline distinguish the drop. I found no visible break or promise of a boat launch on this bank. [Fuchsia City layout](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/maps/FuchsiaCity.blk) · [Crystal collision definitions](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/data/tilesets/kanto_collision.asm#L72-L90)

The discrepancy belongs to collision data. Four placements of metatile `47` at (12,5–8) have `HOP_RIGHT` on the two walkable quadrants. An art-identical local variant could use `FLOOR, WALL, FLOOR, WALL`. The corner at metatile (12,9) could use the pixels of `5b` with `HOP_DOWN` on its upper-left quadrant. Comparing the installed and proposed terrain yielded identical directed walking and paddling connections, including all three southward hops. No gameplay or map edit was made. [Meadow return contract](../../routes/MN_R15.json#L680) · [Native ledge and landing check](../../../../crates/crystal-core/src/world/movement.rs#L260)
