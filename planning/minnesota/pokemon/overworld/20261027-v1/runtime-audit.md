# Northwoods 2D overworld sprite audit

Current scope: **224 active Northwoods species / 101 families**, including the
separately authored Crappeller addition. The historical baseline below covers
223 active species. The 251 returning species are outside this work.
Source IDs, regional order, battle sprites, all nineteen move animations
and original NPC/trainer art remain independent. This audit inspected the actual
`content-packs/northwoods-sticky-v1.crystalpack` (474 species / 270 moves), SHA-256
`77a7a72b41b289d8cb14161fc57b81aaab394feae6cb855cf55efad7a51b8ddc`.
No new overworld art or pack coverage is claimed by this baseline.

## Completed 224-species overlay

All 224 species are accepted in [accepted.json](accepted.json): 1,344 native
frames and 896 runtime files, with independent normal/shiny palettes. Every
selection has source-eye/true-rear evidence and actual production terrain review.
The [full publication](reviews/full224-pack-publication.json) recompiled all
selected source manifests, matched reviewed files and current palettes, preserved
all game data/audio/unrelated art, and reopened the actual resulting pack:
`content-packs/northwoods-overworld-v1.crystalpack`, SHA-256
`97e290de2d52c2d51500420051cbe702cdf64fbefa88f0eca4c39d5e600c93f1`.

The [independent native audit](verification/finish-independent-native-audit.json)
checks all 224 sets, including 344,064 source-to-native pixels. The
[provenance addendum](verification/finish-independent-default-provenance-addendum.json)
resolves its four initially unavailable default references: all 292 selected
sources match their default originals byte for byte. All 148 raw renders returned
in the root session are retained, including rejected attempts.

The actual full-pack `party_menus_use_native_species_sprites_and_preserve_shiny_eggs_and_overlays`
regression passes with the required opt-in fixture environment; it navigates the
production field and battle PKMN menus. See
[menu proof](reviews/party-menu-integration.json) and
[actual test log](verification/full224-party-menu-actual.log). A prior zero-second
run omitted the fixture environment and is explicitly excluded from this claim.

The public Pokédex release `20261027-overworld-224-18` exposes all 224 animated
walkers, 448 normal/shiny atlases and 224 cry buttons. Desktop/mobile media and
TM/HM controls pass against the deployed site. All 458,752 atlas pixels match
independently decoded native banks and palettes. Battle sprites and move previews
are preserved. The local native pack is not installed into the live game service.

The baseline and earlier numerical checkpoints below are historical. The
[source-eye repair resolution](reviews/source-eye-audit-resolution.json) retains
all 17 repaired selections and their original/rejected predecessors.

## Historical baseline coverage

A small Rust reader loaded the verified external pack and inspected its actual
runtime files; it did not infer coverage from battle pictures or source plans.

| Inventory | Verified count |
| --- | ---: |
| Active Northwoods species | 223 |
| Northwoods `gfx/sprites/<species>.png` files | 0 |
| Northwoods dedicated overworld banks | 0 |
| Northwoods battle-front PNGs | 223 |
| All native `gfx/sprites/*.png` files | 102 |
| All `gfx/icons/*.png` files | 38 |
| Shared menu-icon groups used by the 223 Northwoods species | 23 |

All 223 regional menu-icon entries point to shared groups, such as Bug, Bird,
Oddish, Monster or Unown. They are not 223 individually authored walking sprites.
Among returning species, exact-name sprite PNGs exist for Entei, Raikou,
Slowpoke and Suicune at 16×16, and Sudowoodo at 16×32. These are incomplete
walking contracts, not a complete directional returning-species catalog.

## Current code paths

- [`bitmap_font.rs`](../../../../../crates/crystal-bevy/src/bevy_shell/bitmap_font.rs):
  `resolve_visible_object_sprite_asset_id` resolves variable-sprite replacements,
  named files and menu-icon fallbacks. `sprite_asset_path` selects
  `gfx/sprites/<id>.png` or `gfx/icons/<id>.png`. `load_sprite_art` reads a vertical
  PNG sheet and chooses one of eight time-of-day NPC palettes.
- The same file's `map_sprite_sheet_frames` accepts 1/2/3/6 frames. Six means
  down/up/left standing followed by down/up/left walking. Right is a horizontal
  mirror of left. Fewer frames are static approximations. Padded static sheets
  can collapse to one frame; a custom six-frame bank must never use that shortcut.
- [`menu_rendering.rs`](../../../../../crates/crystal-bevy/src/bevy_shell/menu_rendering.rs)
  resolves actual Day Care residents, then loads directional frames, places their
  feet and retains `VisibleObjectSprite` entities for normal map rendering.
- [`script_callbacks.rs`](../../../../../crates/crystal-bevy/src/bevy_shell/script_callbacks.rs)
  selects the standing/action handles from authoritative visible object movement.
  [`deterministic_session.rs`](../../../../../crates/crystal-bevy/src/bevy_shell/deterministic_session.rs)
  uses an action frame on odd phases and mirrors alternate north/south actions
  on phase 3. This provides the ordinary standing/step/standing/opposite-step
  cadence without inventing a nine-frame art format.
- [`party_follower.rs`](../../../../../crates/crystal-bevy/src/bevy_shell/party_follower.rs)
  is compiled behind `voxel-view` and publishes only when voxel settings are
  enabled. Its presentation-only trail uses a battle-front texture as the card
  fallback. It is not a classic 2D directional Pokémon follower. The new work
  must not relabel that existing fallback as custom overworld coverage or change
  the 3D modeled-follower fallback.
- [`source_sprite_renderer.rs`](../../../../../crates/crystal-bevy/src/bevy_shell/source_sprite_renderer.rs)
  supplies the existing direct Bevy sprite draw path. New art should retain that
  path, nearest sampling, map anchors, movement ownership and scene priority.

Current general NPC PNG decoding flood-fills border-connected background but
keeps enclosed white opaque. This is unsuitable for the new strict OBJ contract:
**index 0 must be transparent everywhere**, including enclosed holes. The new
species loader must not reuse that flood-fill inference or the NPC palette bank.

## Agreed native contract

Root approved six 16×16 authored frames per species. Current source generation
uses one species per call in a three-column/two-row pose sheet. Runtime banks
are vertical 16×96 sheets.

| Index | Direction | Pose |
| ---: | --- | --- |
| 0 | Down / south | Standing |
| 1 | Up / north | Standing |
| 2 | Left / west | Standing |
| 3 | Down / south | Walking step |
| 4 | Up / north | Walking step |
| 5 | Left / west | Walking step |

Right/east standing and walking are exact horizontal mirrors of indices 2/5.
North/south alternate step mirroring follows the existing movement cadence.
Frames share one baseline/foot anchor and must depict a newly authored miniature
of the actual species, with appropriate limb/wing/slither/hover movement.
Shrinking a battle front/back picture or substituting a shared icon does not
satisfy this contract.

Each 16×16 frame has four row-major 8×8 tiles: top-left, top-right, bottom-left,
bottom-right. Each tile has 16 bytes (low/high bitplane bytes per row), so one
frame occupies 64 bytes and the six-frame bank is **384 bytes**. The native
indexed bank remains independent of its PNG review representation.

Palette index 0 is transparent, indices 1/2/3 are the only visible inks. Root chose
the canonical species palette's slots 1/2/3 for identity continuity, with an
independent shiny palette. White body/eye details must simplify to light index 1;
there is no fourth opaque white ink. Each palette is exactly four little-endian
RGB555 words (eight bytes), with bit 15 clear. Each pixel has binary alpha:
zero for index 0, 255 otherwise. Palette index 0's stored RGB word is unused for
object drawing.

The hardware color/alpha model follows the primary
[Pan Docs palette specification](https://github.com/gbdev/pandocs/blob/master/src/Palettes.md)
and [OAM specification](https://github.com/gbdev/pandocs/blob/master/src/OAM.md).
A 16×16 frame can be composed from four 8×8 OBJ tiles; this audit does not claim
that the Bevy map renderer enforces a complete ROM/VRAM/OAM scheduling budget.
The six-frame shape and cadence come from the actual local Crystal renderer
and pack rather than from later-generation follower sheets.

## Converter/runtime integration contract

The converter and runtime owners agreed these literal pack paths, where
`<lower_token>` is the validated lowercase runtime species token:

```text
gfx/pokemon/<lower_token>/overworld.2bpp
gfx/pokemon/<lower_token>/overworld.png
gfx/pokemon/<lower_token>/overworld.gbcpal
gfx/pokemon/<lower_token>/overworld-shiny.gbcpal
```

The PNG is 16×96 indexed grayscale RGBA (255/170/85/0 for indices 0/1/2/3), with
alpha 0 exactly at index 0. The 2bpp file and both palette files are authoritative;
converter validation must prove PNG/index/tile equivalence. Review sheets,
source-resolution crops, exact prompts, failed candidates, source hashes,
per-frame notes and immutable source copies remain separate provenance files.

Runtime integration uses a dedicated focused leaf and namespaced sprite IDs:
`pokemon_overworld:<lower_token>:normal` or `:shiny`. It decodes the exact 2bpp
bank and own palette, bypassing generic NPC palette/tint/flood behavior. Named
species objects may choose the bank only when the requested visible species
has that exact asset; normal trainers, generic NPC tokens, variable-sprite
replacements and returning species retain their existing resolution. A present
but malformed custom bank must fail visibly rather than silently reuse an icon.
Missing banks cannot be marked completed in coverage.

A new classic 2D party follower is a separate integration decision: the current
3D presentation trail is not a second gameplay dispatcher and must not become
one. The initial approved implementation scope is a genuine species loader and
existing named-object routes, with the current 3D fallback preserved.

The implementation is in
[`pokemon_overworld_art.rs`](../../../../../crates/crystal-bevy/src/bevy_shell/pokemon_overworld_art.rs).
Its tests cover exact six-pose textures, east mirroring, literal enclosed alpha,
own normal/shiny colors, visible variable-sprite precedence and malformed banks.
The separate
[`pokemon_overworld_context.rs`](../../../../../crates/crystal-bevy/src/bevy_shell/tests/pokemon_overworld_context.rs)
test requests a real converted bank through `NORTHWOODS_OVERWORLD_RUNTIME_DIR`
and `NORTHWOODS_OVERWORLD_SPECIES`. It places a named species object beside the
trainer in the actual Route 36 renderer, then exercises a real shiny Day Care
resident through the same object slot. It checks native 16×16 textures and
retains all four directions and both poses for both appearances under the optional
`NORTHWOODS_OVERWORLD_RENDER_DIR`. The test fixture owns its temporary mounted
assets; it never rewrites the external pack. Both runtime loader tests passed
(2 passed, 0 failed, 0.01 seconds), and the context test compiled in the same
production Bevy test binary. See the retained
[loader test log](verification/runtime-loader-tests.log). The first diagnostic
context attempt used explicitly unaccepted F001 art; it exposed a missing
standing/action synchronization system in the test fixture, now corrected.
The failed diagnostic is retained separately and is not an art acceptance.
The corrected fixture compiled and completed the first provisional real-art
context suite: Whirlnut's `true16-whirlnut-center-v4/MN_F119` candidate passed
all 16 normal/shiny facing/pose cases (1 test, 0 failures, 80.84 seconds).
Its [test log](verification/context-provisional-whirlnut-v4-v1.log),
[all-case native context board](diagnostics/provisional-whirlnut-v4-context-v1/all16-context.native.png)
and [nearest actor board](diagnostics/provisional-whirlnut-v4-context-v1/all16-actors.8x.png)
are retained with all 48 original captures. Both south poses keep separate eyes;
north views are genuine rears; east mirrors west; the feet and silhouette remain
readable beside the native trainer. This is candidate verification, not a claim
that an updated pack has been installed or published.

Pouchid's `pouchid-v3-phase-x0-y4` candidate also passed all 16 context cases
(1 test, 0 failures, 63.49 seconds). Its
[test log](verification/context-provisional-pouchid-v3-x0-y4-v1.log),
[native context board](diagnostics/provisional-pouchid-v3-x0-y4-context-v1/all16-context.native.png)
and [actor board](diagnostics/provisional-pouchid-v3-x0-y4-context-v1/all16-actors.8x.png)
retain the normal and shiny views at their actual native scale. Both south eyes
survive the walking phase. The two small north-cloak vein marks can resemble
eyes when enlarged; that detail remains subject to final visual review.
Scoped formatting passes; workspace formatting remains blocked by pre-existing
differences beginning at `crates/offset-allocator/src/lib.rs:408`, retained in
[the workspace formatting log](verification/workspace-fmt.log).
Required Cargo Clippy attempts for both affected crates are retained as
[assets](verification/clippy-assets.log) and [Bevy](verification/clippy-bevy.log).
Both reach the pre-existing `crystal-core` `bad_bit_mask` failure at
`multiplayer/mod.rs:1332` (200 baseline warnings).

Small authentic source-PNG references are retained in
[references/native-pack-16px](references/native-pack-16px/README.md). Their native
and exact nearest-eight-times boards show actual 16px feature density and all
six poses, without substituting those original identities for new creatures.

## Required acceptance evidence

- All 223 species have six intentionally authored, visually inspected native
  frames; identities, stages and secondary themes match current approved designs.
- Converter checks exact dimensions, indices, binary alpha, palette words,
  distinct directional/action frames and source/index/PNG/2bpp hashes. Root
  judges actual 16px output, not only the enlarged source sheet.
- Runtime checks select all four normal directions and standing/walking in
  production map scenes; east is an exact mirror and index 0 remains transparent.
  Every species' six shiny textures must also equal its bank and own palette.
  Targeted 16-case normal/shiny Day Care context regressions prove the actual
  resident appearance route; the batch path provides broad map coverage without
  repeating those slower route checks for all 223 species.
- Offscreen production draw checks use the dedicated source loader and normal
  sprite pipeline at native 16px. Raw PNG existence alone is insufficient.
- A final pack-overlay test preserves battle art, all nineteen moves and their
  animations, gameplay fields, original NPC art, compiled audio hashes/frame
  counts/loop ranges and every unrelated runtime byte.

Initial measured coverage remains **0/223** until authored outputs are converted,
reviewed, installed and exercised through the actual renderer. The generation
queue records partial progress explicitly; it must never fill missing species
with generic battle downscales.

## Optional batch context review

The focused [batch test](../../../../../crates/crystal-bevy/src/bevy_shell/tests/pokemon_overworld_batch.rs)
accepts one to six distinct Northwoods species through
`NORTHWOODS_OVERWORLD_BATCH_MANIFEST`. Its schema is:

```json
{
  "species": [
    {
      "token": "pouchid",
      "runtime_dir": "planning/minnesota/pokemon/overworld/20261027-v1/candidates/pouchid-v4-back-alpha250"
    }
  ]
}
```

Set `NORTHWOODS_OVERWORLD_BATCH_RENDER_DIR` to a fresh output directory and run
`species_overworld_batch_renders_normal_poses_and_verifies_shiny_banks` with the
explicit frozen pack. The fixture materializes that pack once, copies only each
candidate's bank and two palettes into its owned temporary mount, and clones an
authored map-object record for each review species. It selects visible Route 36
land tiles using the actual collision map and all four walking-entry predicates,
avoiding the player, existing occupied tiles and warps. Clones enter the normal
map-object setup/visible-slot boundary; source maps and gameplay dispatch stay
unchanged.

Eight shared production scenes cover all four directions and both normal poses.
Each selected actor texture is exactly 16×16 and must equal independently decoded
bank/palette bytes, including transparent zero and mirrored east pixels. Each
species' six shiny poses are also loaded and checked against its own palette.
Outputs include native/4× contexts, native/2×/4× scene boards, per-species exact
actor crops and native/4×/8× normal/shiny pose boards. The retained input and
`verification.json` record asset SHA-256, chosen land permission/tile, world
translation and actual selected-raster hashes. Existing individual Day Care
normal/shiny context tests remain available; batching does not replace their
route proof.

The first six-species capacity attempt exposed a fixture-only setup omission:
a same-tile roster refresh correctly does not load newly appended actors. The
fixture now calls the existing map-object setup boundary and checks every new
actor receives a normal visible slot. That failed attempt is retained.

The corrected initial batch passed in **31.75 seconds**: six species, eight
production scenes, 48 exact selected normal actor rasters and 36 exact shiny
textures. Pouchid v4, Whirlnut v4-equivalent/shared-v6, Samurrel v6 and the three
approved fire-starter v2 candidates were used. All six chosen tiles have native
collision permission zero (plain walkable land). All six actor boards and the
full native context board were visually inspected; Pouchid's corrected rear has
no false face. This remains provisional art evidence, with no source promotion
or pack publication performed here.

- [Passing batch log](verification/batch-provisional-six-v2.log)
- [Exact six-species input](verification/batch-provisional-six-v2.input.json)
- [Selected-raster/source hashes and placements](diagnostics/batch-provisional-six-v2/verification.json)
- [Native eight-scene board](diagnostics/batch-provisional-six-v2/all8-context.native.png)
- [Nearest four-times scene board](diagnostics/batch-provisional-six-v2/all8-context.4x.png)

The output folder retains 99 files: eight native and eight 4× context scenes,
three complete scene boards, each species' eight exact 16px actor crops,
normal/shiny native banks and three pose boards, plus source/evidence JSON.
The fixture compiled without errors or new warnings. Scoped formatting passes;
the fresh [workspace formatting check](verification/workspace-fmt-batch.log)
and [required Bevy Clippy attempt](verification/clippy-bevy-batch.log) retain the
same unrelated baseline failures described above.

## Historical 17-species acceptance checkpoint

Seventeen species now pass root native visual review and actual production-map
context validation. All nine starter stages, the three burrow rodents, all three
tag ducks, and both flying squirrels retain six16×16 frames with a384-byte2bpp
bank and independent normal/shiny palettes. The three retained batch suites
cover136 actual normal direction/pose rasters and102 shiny native frames. Each
selected raster is checked against an independently decoded native bank. Root
inspected the actual terrain/player contexts before copying only the four native
asset files into the accepted workspace registry.

See [accepted registry](accepted.json), [size audit](reviews/native-size-audit.json),
[starters/Greytag context](diagnostics/batch-starters-greytag-v1/all8-context.2x.png),
[ordinary four context](diagnostics/batch-ordinary-four-v1/all8-context.2x.png), and
[Dartduck rear correction context](diagnostics/batch-dartduck-v4/all8-context.2x.png).
There are206 species still unfinished. No overworld pack has been published;
acceptance of these native files is separate from game-pack publication.

## Historical 37-species acceptance checkpoint

The accepted registry now contains37 of223 species and222 authored native frames.
Eight retained passing production batches cover296 actual normal direction/pose
rasters and222 shiny native frames. Root inspected all native normal/shiny boards
and actual terrain/player contexts before registration. The Rust converter
continues to own sampling, palette quantization and native2bpp export. The small
metadata registration helper refuses native banks or palette bytes that differ
from the context batch hashes, checks all six native frame rows against indexed
proofs and requires explicit visual-review selections. It does not create art.

All37 sets use16×16 frames, transparent borders, distinct walking poses and
individual RGB555 palettes. The flour-mill line additionally retains exact
source-to-native pupil-coordinate proofs after several rejected sampling trials.
The root-selected dots are the original source pupils rather than incidental
black contour fragments. All trial sources, failed renders and prompts remain
saved. There are186 species still unfinished; no overworld pack is published.

See [accepted registry](accepted.json), [current native size audit](reviews/native-size-audit.json),
[flour-mill/pine/den production context](diagnostics/batch-mills-pine-den-v1/all8-context.2x.png),
and [latest root selections](reviews/selection-native-32-37.json).

## Historical 51-species acceptance checkpoint

The registry now contains **51 of 223** accepted species: 306 authored native
frames, 408 selected normal direction/pose cases in the actual production map
renderer, and 306 selected shiny native textures. Root inspected all native
normal/shiny boards and terrain/player contexts. All selected source, prompt,
manifest, compiled-index and runtime hashes were rechecked after registration.
The full scope remains223;172 species are unfinished. No overworld pack has
been published.

This checkpoint adds34 accepted species beyond the earlier17-species registry.
Hatchrush now uses a wider source head so both real pupils survive the shared
16px transform. Denumbra loses its misleading rear-eye pair through a north-only
ImageGen repair, preserving all four front/profile native frames. Notchbill uses
source-edited raised pupils above its bill; the blank-visor candidates remain
rejected and retained. All raw generations, exact prompts and failed trials are
saved. Six artists/tooling specialists have contributed across the work; current
art work continues across the remaining families within the actual worker cap.

See [51-species checkpoint](reviews/size-checkpoint-51.json),
[current size audit](reviews/native-size-audit.json),
[Hatchrush/woodpecker map review](diagnostics/batch-hatch-woodpecker-v1/all8-context.2x.png),
and [Notchbill/Hushiss map review](diagnostics/batch-notch-rattle-v1/all8-context.2x.png).

## Historical source-anatomy and native-size checkpoint: 66 species

The [66-species checkpoint](reviews/size-checkpoint-66.json) records 396 authored
native frames, 528 selected normal direction/pose cases in the production map
renderer, and 396 shiny bank frames. All use 16×16 cells, a transparent border,
384-byte banks and individual RGB555 palettes. Actual opaque bounds range from
6–14 pixels wide and 7–14 high: small larvae retain smaller silhouettes, while
evolutions gain body mass without exceeding the native cell. Root inspected the
normal/shiny sheets and eight terrain views beside the trainer.

The independent [56-selection source-eye audit](reviews/source-eye-audit-report-56-v1.md)
passed 39 and held 17. At this checkpoint 14 held selections have source-proven
repairs accepted; Borealynx, Furrowbank and Tapduck remain held. Every accepted
selection now has a separate anatomical receipt bound to its manifest, source
PNGs and native bank. Complete native eye clusters must sample their actual
source pupils, rather than black hands, ear marks or jaw contours. Canonical
exceptions are explicit: Sinknook has one notch eye, Rondive has four frontal
and two profile pupils, and the fossil pair retains bright compound-eye windows.
All original and rejected art remains saved.

The metadata registrar requires those receipts before accepting new candidates.
Its preflight was rerun across the accepted assets; changed-bank and missing
source-review rejection checks also passed. New work in this checkpoint changes
art/provenance and this metadata gate; Rust converter/runtime tests and their
previously documented baseline lint failures remain recorded above.

The external pack still hashes to the baseline SHA-256 above. These are staged
native assets, not a published overworld pack. The full223-species goal remains
active with157 unfinished at this checkpoint.

## Latest verified checkpoint: 76 species

The [76-species checkpoint](reviews/size-checkpoint-76.json) records456 native
frames,608 selected normal production direction/pose cases and456 shiny frames.
All76 selections pass the source-feature and native-bank preflight. The17
source-eye audit failures are now repaired and accepted after independent
normal/shiny/terrain review; [resolution receipts](reviews/source-eye-audit-resolution.json)
retain each replacement and original revoked acceptance.

Sizing also checks evolution proportions. Castoroar's latest candidate passes
native constraints and actual-pupil checks but is held because its9×9 frontal
body is shorter than Castorock's9×10. The artist is enlarging the authored source
torso and armor while retaining one shared transform and the compact profile
tail. No independent per-pose native stretching is allowed. This candidate is
excluded from the76 accepted species.

All223 species remain required;147 are unfinished. All raw renders, prompts,
unused edit frames and rejected candidates are preserved. No overworld pack has
been published, and the verified external pack remains unchanged.


## Latest verified checkpoint: 88 species

The [88-species checkpoint](reviews/size-checkpoint-88.json) records528 native
frames,704 normal production direction/pose cases and528 shiny frames. All88
selections pass the full metadata preflight, including actual source anatomy,
current manifest/crops, literal 2bpp bank and both canonical palettes.

Prairondo, Bluestip, Stembolt, Prairion, Mudmender and Podlet passed the
`prairie-orchid-mudmender-podlet-6` production batch. Castoroar, Flossail,
Parafloss, Dewdant, Barricant and Zapsap passed `castoroar-floss-ant-zapsap-6`.
Root reviewed normal/shiny native strips and each batch's eight actual terrain
poses beside the trainer. Castoroar's enlarged12×12 frontal body now grows
clearly beyond Castorock's9×10; its shared transform and compact tail remain.
The earlier Castoroar hold is resolved by v19; all earlier candidates remain saved.

The acceptance gate caught an artist receipt that hashed the PNG rather than
its literal 2bpp bank. The artist independently revalidated all1,536 decoded
indices and the four actual source pupil components before correcting affected
receipt metadata. No accepted art was altered by that correction.

All223 species remain required;135 are unfinished. No overworld pack has been
published. The external base pack still has SHA-256
`77a7a72b41b289d8cb14161fc57b81aaab394feae6cb855cf55efad7a51b8ddc`.


## Latest verified checkpoint: 94 species

The [94-species checkpoint](reviews/size-checkpoint-94.json) adds Vanechick,
Silohawk, Crowscare, Chorip, Choirbog and Pothollow from the passing
`weather-birds-frogs-pothollow-6` production batch. Root reviewed both palettes,
six authored frames, all eight map directions/poses and true rear anatomy.
Crowscare retains its canonical light eye organs; Choirbog has three distinct
vocal sacs. The selected94 banks total564 native frames,752 normal map cases
and564 independently verified shiny frames.129 species remain unfinished.

All128,386 archived files at this checkpoint are covered by immutable inventory
`inventory-f2e99f932c20841204870ee0f0d508d76ead0eaa24426fa8370e22ee0e196a11.json`.
Sources generated after that snapshot remain saved in their original folders.
No overworld pack has been published.


## Pokémon menu reuse

Both field Pokémon and battle PKMN menus use the installed species bank's
front idle/step frames0/3, retaining16×16 native cells, transparent index0 and
the species' own normal/shiny RGB555 palette. Namespaced cache keys keep shiny
and ordinary members separate. Eggs and unconverted legacy species retain
their original icons; held-item/mail overlays retain their existing layering.

Menu animation now reads the presentation LCD clock rather than the semantic
state-checksum frame, which can freeze while menus own input. The native
deterministic clock invalidates party presentation at crossed eight-frame
boundaries, and the retained render key includes the same phase. Existing
HP-dependent8/72/136-frame cadence and selected-row bob remain.

Three native-loader unit tests passed in the Cargo-built test binary and in the
current-source review binary. The two-menu production regression navigates
through VisibleShellController, then checks six native textures, normal/shiny
identity, eggs, legacy icons and item/mail overlays in both idle and step phases.
Its render-only app explicitly advances the LCD presentation clock between
frames; it does not claim gameplay ticks advance the graphical clock.
`party-production-menu-verified-tests.log` passes. Root inspected all four
normal/shiny mixed-party captures in `diagnostics/party-menu-verified`.

The capture helper now renders Bevy's default white quad with its sprite tint;
previously it skipped the battle party's opaque backdrop. A corner-pixel check
requires that white backdrop in saved menu renders. Sprite pixels and production
background rendering were not changed by that review-tool correction.

The required Cargo test compilation passed. The final review binary compiles
current source against the Cargo build's exact dependencies; its native/menu
tests and scoped rustfmt checks pass. Full-workspace rustfmt remains blocked by
pre-existing formatting, including offset-allocator. Required Clippy stops at
the pre-existing `crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error and
existing warnings. Logs and source hashes are bound in
[the menu receipt](reviews/party-menu-integration.json). No pack was published.

The Cargo-built two-menu integration test also passes
(`party-production-menu-cargo-tests.log`). Re-running the six-species map batch
with the corrected default-quad capture helper passes; all16 native/4x map
images remain byte-identical to the previously reviewed terrain scenes.


## Latest verified checkpoint: 110 species

[110 selected species](reviews/size-checkpoint-110.json) now cover660 native
frames,880 normal production direction/pose cases and660 shiny frames. All110
current selection entries were revalidated against current source/manifest,
2bpp rows, normal/shiny palettes and hash-bound context evidence. The current
roster, curation, ordering, profiles and palette files still match the original
223-species snapshot.113 species remain unfinished.

Three new batches passed native/context review: `beet-calf-dust-ventifact-6`,
`roots-loess-hotdish-curds-6` and `voyagear-pancake-bison-butter-5`. The latter
contains four new species and a replacement Voyagear. Root generated and saved
individual Flapip, Stackajack, Chislet, Buttermuse and bulk Voyagear sources with
the built-in ImageGen tool; every original/prompt and failed edit remains.
Stackajack is still a candidate undergoing source-outline refinement.

An evolution-proportion review compared39 families with two or more accepted
stages. Area metrics flagged candidates for direct visual inspection, not
automatic size changes. Squat frogs, a floating reed raft, broad moth wings and
hunched bear posture have valid shape differences. Voyagear's upright adult
was visually underweight. Its new source-backed native front13×12/119opaque
pixels replaces13×10/80; the broad head/chest, voyageur sash/headband, shoulder
canoe paddle and low beaver paddle-tail now read as a mature counterpart to
Portabeau14×12/125. The [review](reviews/evolution-size-review-100.json) and
[superseded receipt](reviews/superseded/MN_012-pre-bulk.json) preserve the comparison.
All prior native bytes remain in the original candidate directory.

The new Voyagear also passes both production Pokémon menus with actual native
idle/step textures, individual palettes, eggs and item/mail overlays; see
`party-production-menu-voyagear-bulk-tests.log` and
`diagnostics/party-menu-voyagear-bulk`. Full pack publication remains pending.

## Anatomical native checkpoint: 130/223

All 130 accepted species were revalidated against selected source manifests, anatomical receipts, exact canonical normal/shiny RGB555 palettes, eight production terrain poses, and all six shiny native frames. Copied native files remain byte-identical to the tested candidates. This covers 780 authored frames and 1040 normal direction/pose scenes; 93 species remain unfinished. Every frame is16×16 in a16×96 sheet with384-byte indexed bank, transparent border and distinct idle/step artwork.

The field Pokémon menu and battle PKMN switch menu consume these same banks at native16×16 with frames0/3 and independent shiny palettes. Their production controller tests, exact pixel comparisons and inspected captures are recorded in [party-menu-integration.json](reviews/party-menu-integration.json). The immutable baseline pack has not been replaced or published.

## Anatomical native checkpoint: 136/223

All 136 accepted species passed the full registration preflight again, including exact source-feature, canonical palette, native border/gait, production context and copied-file checks. This covers 816native frames and 1088direction/pose scenes; 87species remain. The complete walleye and lure families are accepted. Eight root-authored peat/axe/twine source sheets are retained for conversion; they are not counted as accepted. Pack publication remains pending.

## Native checkpoint155 and Pokédex media

All155accepted banks passed source-feature, exact palette, border/gait, production context and copied-file checks. This covers930native frames and1240normal direction/pose scenes;68species remain. Newly accepted families include both leeches, pike middle/final stages, biting midges, crayfish, all three twine stages, both weather buoys, Keypout, the mosquito pseudo-legendary line, Emberm and Smoulden.

The web exporter now creates normal/shiny64×16 walking atlases directly from accepted native banks; no source art or native pixels are altered. Each uses idle, step, idle and mirrored step. Cry controls use the existing external Rust gallery, with223distinct authored cries verified against actual packed runtime PCM. Web publication is separate from full overworld-pack publication, which remains pending.

## Native checkpoint174 and Pokédex media15

All174 accepted banks passed source/anatomy, palette, transparent-border/gait, hash-bound production context and copied-file preflight. They cover1044 native frames and1392 normal direction/pose cases;49 species remain. The 19 additions since155 include axes, freighters, burbot, oak, auger, campfire, lift-bridge, marten and deer families, plus Kettlurk. Its actual eye slots and source-transparent torso openings survive in both palettes. Every source original and rejected trial remains retained; the latest archive contains132290 files.

Media15 publishes223 original cries and174 normal/shiny walking thumbnails; public desktop/mobile playback, movement, sizing and reduced-motion checks pass. Source generation remains separate from native acceptance. The full223 overworld-pack goal remains active and pack publication is still pending.
