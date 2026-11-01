# Gray holding cell fence

The user's 2026-10-29 correction requires a gray fence for the holding cell.
The native 2D office uses the original Johto Modern metal rail pixels over its
existing Underground floor. Source blocks `46` and `41` supply the vertical and
horizontal fence glyphs. Rust copies them directly from the external pack into
the private `mn_northstar_holding` tileset; original art files stay exact.

The office keeps its 10×8 metatile footprint, actors, entrances and script/event
identities. Five solid barrier cells become fence sides/corner at `(6,1)`,
`(6,2)`, `(6,4)`, `(7,4)` and `(8,4)`. Coordinates here are metatiles. The
vertical barrier blocks its western walking quadrants; the bottom barrier blocks
its southern quadrants. The corner closes both. The surrounding office walls
complete the enclosure.

The gate at `(6,3)` retains block `38`. Its western quadrants remain impassable
until the existing rescue script replaces it with floor `05`. That occurs after
the power shutdown, enforcer battle and acknowledged release conversation.
Ellis's existing walk through `(12,7)` and out by `(11,15)` remains the exit.
The return callback uses the same rescue flag and block replacement.

The original Minneapolis builder and final story revision use the same Rust
fence authoring function. No 3D asset or renderer change is involved. Native
before/after captures, collision closure, the separate gate action, actor exit
and Continue must be verified before promotion.
