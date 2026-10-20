# Pokémon world models and bedroom dolls

The 31 explicitly named Pokémon templates in `johto_actor_props.rs` now read
their neutral geometry from the canonical species rigs. Living actors keep
their species animation and pack dimensions. Bedroom dolls reuse that geometry
without a skin or idle motion. Their uniform scale preserves the previous
authored display heights: 1.48 for big Snorlax, 1.52 for big Lapras, 1.72 for big
Onix, and 0.82 for the small named icons. These are presentation dimensions,
not replacements for the pack's Pokémon measurements.

The publisher sends `species:SPECIES:bitmap` for living Pokémon and
`display:SPECIES:bitmap` for the three actual bedroom doll slots. The existing
bitmap remains the 2D reference. A shared icon family never establishes a
species identity. Exact species markers also allow a doll such as Blastoise
to use its actual model even when the original bitmap is shared with Squirtle.
Display labels say `NAME DOLL`; living actor labels say `NAME`.

Vermilion's Snorlax and the bedroom Snorlax doll share `SPRITE_BIG_SNORLAX`.
The verified map, object identifier, and script distinguish the living actor.
The bedroom-only Staryu/Starmie sprite exception remains restricted to the
small doll slots. Generic bird/fish/monster templates cannot identify a species.

The Surfing Pikachu ornament uses `display:PIKACHU:surfing_pikachu` only in
the two bedroom ornament slots. Its canonical Pikachu body is translated onto
a separate closed, bevelled surfboard without changing the body's positions
relative to each other, normals, colors, UVs, indices, or canonical rig. The
complete display retains the small doll height of 0.82. Its label is
`SURFING PIKACHU DOLL`. Raw Surf avatar sources and animated mounted Pokémon
retain their existing routes.

Both roles use the floor-rooted actor anchor. Living actors retain the pack
scale, including longitudinal reference spans. Dolls use the canonical body's
vertical span to preserve their authored display height. No global terrain,
grass, material, lighting, or shader changes are required.

Bedroom displays on a complete modeled table use the same tabletop support
query as stationary books and lab balls. A missing or customized drawing
provides no substitute support. Floor dolls and living Pokémon keep their
ordinary floor roots, and player collision/footing is unchanged.

## Disposable room review

`render_at_location --preview-doll DECO_BIG_LAPRAS_DOLL` requires one
`PlayersHouse2F` screenshot without `--x` or `--y`. It grants the catalog
ownership flag in a fresh session, calls the normal `set_up_decoration`
mutation, then reloads at the title's bedroom spawn through ordinary map entry.
This allocates the newly visible doll's object struct before capture. It cannot
load or save a player's game. Substitute
`DECO_BIG_SNORLAX_DOLL` or `DECO_BIG_ONIX_DOLL` for the other big dolls.
Small ornaments use the normal left-side selection; examples are
`DECO_SURF_PIKACHU_DOLL` and `DECO_PIKACHU_DOLL`. The previous
`--preview-big-doll` spelling remains an alias.

The callback creates four variable-sprite slots after initialization. Save
validation recognizes those exact slots only when the pack declares
`ToggleDecorationsVisibility`; unknown slots and missing replacement sprites
remain invalid. This also fixes ordinary equipped bedroom decorations being
rejected by runtime snapshot validation.
Species-named replacements such as `SPRITE_PIKACHU` validate against the
pack's exact menu-icon catalog, matching the normal visual resolver. They do
not require an unrelated sprite-palette row. Unknown species aliases still
fail validation.

Always render with `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, and `CRYSTAL_PREVIEW_MUTE=1`.
Use `--view both --hour 12` for a pair at the same authoritative state. Keep
the pack and capture output ignored. A scene capture establishes visual state,
not gameplay progression or a completed playthrough.

## Reviewed scope

Thirteen focused Rust checks passed: canonical template sharing, uniform doll
dimensions, floor anchors, inert display entities versus animated living skins,
source identity and replacement guards, the compiled decoration catalog, the
named Snorlax encounter, labels, and callback-slot validation. No shiny checks
were run for this change.

Offscreen native pairs were inspected for Vermilion Snorlax and the three big
bedroom dolls at noon. Each pair preserved its authoritative state. The living
actor receipt identified `species:SNORLAX:big_snorlax`; the room receipts
identified `display:LAPRAS:big_lapras`, `display:SNORLAX:big_snorlax`, and
`display:ONIX:big_onix`. The room silhouettes fit their corner without clipping,
and their floor contacts and display sizes remained readable. These four
scenes establish this migration's visual scope; they do not establish visual
acceptance of every Pokémon or decoration.

## Surfing Pikachu and tabletop follow-up

The first offscreen Surfing Pikachu pair exposed the previous floor anchor:
the body intersected the bedroom table and only its ears were visible. Final
noon pairs for Surfing Pikachu and the plain Pikachu doll were inspected after
the support fix. Both bodies now stand visibly on the actual tabletop; the
Surfing ornament's board is fully visible and the plain ornament has no board.
Each source/native pair preserved its authoritative state. The final native
binary remained unchanged through both captures.

Thirteen focused checks passed for this follow-up, including unchanged body
attributes, closed board topology/winding, full display height, static versus
animated entities, real Surf turns/dismount, source and label guards, table and
floor support, incomplete table rejection, and both save-reference rules. No
shiny checks were run. An additional existing Elm starter-ball control failed:
`hidden_room_prop` currently suppresses all Elm ball props while that test
expects three. That separate room behavior was left intact; this is not an
all-green repository test claim.
