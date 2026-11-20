# Reading a ledge from the trail

At Shiner Narrows, walk along the west bank to the meadow at its south end. The strip of sand above the river is a high bank. None of its nine eastward approaches has dry ground on the far side. The three approaches along the meadow rim each have a landing two squares south.

In the terminal, the eight straight bank squares now read as ordinary ground. The corner and the two squares beside it point south. A trainer walking across a landing can temporarily block a jump. The direction on the ground stays stable while that trainer moves.

Crystal uses the standing square and the direction the player is facing to choose a ledge jump. A corner can accept either of two cardinal inputs. Polished Crystal also looks two squares ahead to check the landing. Geothite's native movement already checks the landing and recognizes declared map connections. The terminal now reads those walking permissions from the active map, including changes made by puzzle switches. [Crystal movement](https://github.com/pret/pokecrystal/blob/master/engine/overworld/player_movement.asm#L342-L379) · [Polished Crystal movement](https://github.com/Rangi42/polishedcrystal/blob/master/engine/overworld/player_movement.asm#L325-L369) · [Geothite movement](../../../../crates/crystal-core/src/world/movement.rs)

The painted terminal draws a small chevron toward each usable landing. The two arms round to the available pixels on narrow terminals. The character takes precedence over a terrain mark when standing on the lip. The same marked dots reach the browser canvas and the native terminal. [Terrain cues](../../../../crates/crystal-tui-view/src/terrain_glyph.rs) · [Painted dots](../../../../crates/crystal-tui-view/src/ink.rs)

The installed-pack regression checks the nine blocked jumps, the three actual meadow drops, a temporarily occupied landing and the dry eastward jump in Crystal's Fuchsia City. An optional capture directory receives a disposable save on the west bank for inspecting the terminal. [Pack regression](../../../../crates/crystal-tui-view/src/terrain_glyph_tests.rs)
