# Dynamic bedroom decoration models

The room has 45 selectable decoration states, backed by existing exact original
sculptures plus one original cloth model. This is a source-selection checklist,
not a count of 45 new meshes or a claim that every state was played manually.
No game artwork is embedded in the model or editable source files.

| Category | States | Source-correct model treatment |
| --- | --- | --- |
| Beds (4) | Feathery, Pink, Polkadot, Pikachu | Four existing quilt/headboard variants; exact complete 16×32 tile drawings |
| Carpets (4) | Red, Blue, Yellow, Green | Closed cloth backing with unchanged live atlas pattern and palette |
| Plants (3) | Magna, Tropic, Jumbo | Three existing distinct pot and foliage sculptures |
| Posters (4) | Town Map, Pikachu, Clefairy, Jigglypuff | Existing real mitered frame and backing; unchanged live picture pixels |
| Consoles (4) | NES, Super NES, Nintendo 64, Virtual Boy | Existing distinct console/controller sculptures |
| Big dolls (3) | Snorlax, Onix, Lapras | Existing full-volume large sculptures |
| Ornaments (23) | Pikachu, Surf Pikachu, Clefairy, Jigglypuff, Bulbasaur, Charmander, Squirtle, Poliwag, Diglett, Staryu, Magikarp, Oddish, Gengar, Shellder, Grimer, Voltorb, Weedle, Unown, Geodude, Machop, Tentacool, Gold Trophy, Silver Trophy | Exact existing species sculptures or dedicated surfboard/trophy models; source identity survives shared icon selection |

## Current appearance remains authoritative

The production bitmap resolver first resolves variable sprites and active Day
Care residents. The model publisher preserves that currently visible species
along with the bitmap source that actually exists. A generic monster, fish,
ghost, blob, or humanoid icon is never evidence of an exact species by itself.
Copycat and story-variable appearances use their current replacement; no hidden
original identity is read. Unknown species or unknown artwork keeps card fallback.
The 251 normal species sculptures are reused through bounded per-kind mesh caches.

The original catalog names the Staryu ornament with `SPRITE_STARMIE`. The Staryu
binding applies only to the two live bedroom doll slots. An ordinary Starmie,
including a Day Care resident, retains its own species model.

The renderer follows the existing visible actor list, so put-away and ownership
flags remain simulation-owned. Changing a source on the same object ID replaces
the rendered instance, reuses already loaded meshes when switching back, and
removes stale entities. Positions, facing, source palette inputs, foot anchors,
collision, warps, and current cosmetic selection remain production-owned.

## Cloth and picture ownership

Each carpet state contains four cloth fields totaling 52 source cells. The
central desk-foot row remains separately owned. Matching checks tileset, block,
subtile phase, complete drawing dimensions, and every source tile. Altered,
clipped, or custom-profile-owned drawings stay on their existing renderer.
Carpet tops use the live atlas cells verbatim; frames use the same source-UV
picture mounting as the existing interior implementation. No patterns,
illustrations, replacement text, or source colors are baked into new assets.

`tools/build-bedroom-cloth.py` reproduces the new canonical JSON, editable
Blender source and optional oblique studio render. The 1,672-triangle backing
contains named closed textile layers, rolled bindings and overcast stitches.
The compressed JSON decodes through the shared model-storage contract.

## Focused regressions

- `visible_species_source_tests`: current variable replacement, Day Care,
  generic-icon non-inference, scoped Staryu exception, all 30 actor catalog states
- `exact_visible_species_tests`: shared-icon separation, unknown-source fallback,
  and the complete 251 normal-species model registry
- `dynamic_species_decorations_replace_reuse_and_disappear_without_stale_entities`:
  same-ID changes, unchanged roots, mesh reuse, unknown fallback and disappearance
- `bedroom_carpet_tests`: all four states, exact source ownership, no duplicate
  claims, unchanged footing, changed-art/phase refusal and clipped fallback
- `every_live_bed_and_poster_selects_its_current_complete_source` and the existing
  three-plant regression: exact model selection and whole-source refusal

Asset previews can inspect silhouette and source mounting. They are not a
substitute for running the production controller decoration-menu regression
and the renderer regressions against the local ignored content pack.
