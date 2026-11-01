# Minimal Fair species dependencies after connected-state Ely

This is a source implementation proposal, not an executed catalog build. No
catalog, pack, source art, shared Rust registration or canonical map was changed.
The current repaired terrain candidate is
`content-packs/minnesota-cohesion-harbor-water-v01.crystalpack`, file SHA256
`3d1cbdd18b05ce10ed66db7ab56d41f0032f6e045a0e72ad753064c5dc6ec8df`,
identity `174f1942c42bd590e292e5a95340ffb9d93758bf0ab1ec31d83dadfd9fc9c722`.
Root still owns its native acceptance. Ely's subsequent additive publication
must become the actual catalog parent, keeping all 691 maps and the R13 seam.
Do not use the older 474-species Fair branch as the base or copy its catalogs.

## Exact closure and source authority

The current `zones/state-fairgrounds.json` asks for 29 regional grass species;
its trainers additionally require POUCHID and SODCALF. Every native party
partner already exists. Complete evolution families add only four more species:
SLIPPERINA, CYPRIELLE, RUMBLEHIDE and THUNDRAZE. The resulting overlay is exactly
35 new species, producing 286 total, without compacting their stable numeric IDs.

| Family | Complete selected members |
| --- | --- |
| MN_F001 | POUCHID, SLIPPERINA, CYPRIELLE |
| MN_F005 | RICELING, REEDRAFT, ZIZALITH |
| MN_F010 | OATLET, MILLIMP, FLOURGEIST |
| MN_F016 | FLAPIP, STACKAJACK |
| MN_F020 | CRISPIP, CIDERMAW |
| MN_F032 | TOTLIT, CASSEROGRE |
| MN_F033 | CURDLET, SQUEAKURD |
| MN_F038 | LUTEBRISK |
| MN_F042 | TILTAWISP |
| MN_F051 | SODCALF, RUMBLEHIDE, THUNDRAZE |
| MN_F083 | CINDERFIN, BRINEBRAZE |
| MN_F090 | DOUGHLIN, CRUMCONE, PAILBARON |
| MN_F098 | CHISLET, BUTTERMUSE |
| MN_F101 | ZAPSAP, TAPACITOR, BEETYNAMO |
| MN_F105 | PIPCORN, PUFFLARE, POPUMMEL |

Source authority remains `pokemon/roster.json`, `roster-curation.json`,
`species-profiles.json`, `sprite-generation.json`, `palettes.json`,
`northwoods-moves.json` and `fairy-moves.json`. Their exact current hashes and the
15-family closure are in the ignored, bounded
[static inventory](../renders/integration/fair-catalog-after-cohesion-v01/source-inventory.json).
It contains identity/dependency metadata, not a duplicated species/profile or
command catalog. The seven source hashes still match the prior integration
plan; move-source hashes are now recorded too.

The authoritative loader first validates all 143 source families/313 profiles,
then applies curation to 101 active families/223 species. It loads **all 101
active families' art before any family selection**. Thus a minimal 35-species
pack still needs the complete existing `target/minnesota-pokemon/native` tree;
do not author a second reduced roster/profile directory. Fresh read-only
inventory found all 101 accepted review-sheet hashes current, and every active
member's front/back JSON, PNG and 2bpp path present. Selected source moves and
native footprint donors exist in the current native parent. These are file and
reference checks, not execution of Rust's conversion/palette/2bpp validators.
No missing input was found at this inventory stage.

## Existing Rust path; no new catalog engine

`MinnesotaPokemonCatalog::load` in `minnesota_pokemon_modpack.rs` is the authority.
It validates profile identity/evolution coverage before curation;
`minnesota_pokemon_graphics::load_family_graphics` then validates each accepted
review, source SHA, crop/fit/sampling, normal palettes, source-derived pixel
conversion, PNG indices and 2bpp. The runtime species number is exactly
`251 + MN_number`; the selected family is installed in full. `Catalog::build`
rejects duplicate IDs/art/manifests, checks native reference donors, verifies
whole evolution closure, retains old species and updates the last Oak rating
limit to cover the new species count. Each family adds eight files per species
(front/back PNG/2bpp, front dimensions, normal/shiny palettes, native footprint).
It does not synthesize new cries; source profiles bind existing native cry
metadata and compiled audio remains exact.

Use the public builders in this order:

1. `build_fairy_type_modpack` installs the source Fairy/Steel rules.
2. Load `MinnesotaPokemonCatalog` once from the original source/native tree;
   call `catalog.build(&stage, Some(family))` for the 15 IDs above. Preserve
   each intermediate in-memory result as the next base, so all family closures
   accumulate. `build(None)` would install all 223 and is outside this scope.
3. `build_northwoods_moves_modpack` appends the 13 supported source moves and
   their five signature animations. Its native starting requirement is exactly
   251 moves/names and 252 animation slots; no Fair attacks can precede it.
4. `build_fairy_moves_modpack_from_json` consumes the exact current authored
   `fairy-moves.json`, appends six moves after 264 and applies declared learners.
5. `build_northwoods_animations_modpack` installs distinct native-object programs
   for the 14 imported moves; its prerequisite is the completed 270-move stage.

All methods already return `Result<CompiledGamePack>` without publication.
A focused wrapper can load art once and call these existing APIs; no core
species loader, replacement catalog, art conversion or exporter change is
needed. The existing CLI equivalent and exact argument order remain in
[fair-ely-after-cohesion-v01.md](fair-ely-after-cohesion-v01.md). Root alone builds
and runs that command sequence after Ely acceptance. Give every output a new
ignored path and stop on the first failing stage.

## Explicit native exceptions; all other facts stay exact

“Preserve all 251 originals” means retain their identities, stats, capture,
evolution, egg moves, art, audio and unrelated facts. The requested source Fairy
rules intentionally change the following named facts, so byte-exact original
species/type/learnset equality cannot honestly be claimed for this combined
rules stage:

- Only `type1`/`type2` change for national IDs 35,36,173,39,40,174,175,176,
  183,184,209,210,122, according to `build_fairy_type_modpack`'s exact tuples.
  Remove those two changed fields and every complete original species must
  equal the parent. Every other original species must be wholly equal.
- Native CHARM, SWEET_KISS and MOONLIGHT change only `move_type` to FAIRY;
  all other original move fields and native indices 1–251 remain exact.
- The Fairy chart adds 12 exact source rows; only DARK→STEEL and GHOST→STEEL
  old resistances are removed. Foresight exceptions remain exact.
- The type stage appends FAIRY to the special-type list. The parent already has
  the complete modern per-move categories; the species builder verifies and
  retains them. Never silently replace an incompatible existing split.
- Six Fairy move definitions append **31 declared learnset grants to 13 native
  species**, as recorded in the inventory's exact `(national number, move,
  level)` list. Filter precisely those added moves from each affected native
  learnset and require the original sequence exact. No native TM/HM, egg move,
  evolution or existing trainer-party record is rebuilt. Existing trained
  parties remain exact; new Fair parties use the resulting legal learnsets.
- Northwoods grants target only the installed selected regionals (six grants);
  Fairy grants add 14 selected-regional entries. Absent other active regional
  learners are explicitly skipped by both existing source builders. They do
  not force an aggregate 223-species overlay.
- New species add normal species/learnset/evolution/Dex/menu/cry/front-animation/
  capture-weight keys, and only the final Oak caught-count limit may rise.
  New moves append indices 252–270, names/categories/priority entries, animation
  slots and explicitly owned native-object/program additions. No native
  animation program or existing animation slot is replaced by the final
  imported-move animation stage. Signature animation runtime assets are new
  owned files; every original runtime file stays byte-exact.

No map, map script/object/encounter, recovery, HOME, item/mart, story flag,
landmark, native trainer, source audio program or PCM payload is part of this
catalog overlay. The subsequent Fair map/foods publication is a separate delta.
Its integration still needs the approved east approach and exact additive
flag/spawn allocation; this catalog plan neither places nor connects the Fair.

## Required focused implementation and regression contract

The narrow wrapper should pin the actual accepted Ely parent identity plus the
seven original source hashes, verify the exact 15-family set is active, load
once, apply the existing stages and assert exactly 35 new canonical species.
It should return the result in memory. Root can expose one focused CLI using the
usual strict read/write/readback and fresh ignored output protections.

Before publication, require:

- Whole old-map/modules, raw/split programs, current R13/Ely seam, recovery,
  flags/trainers/items/marts/art/audio tables exact. Normalize **only** the
  explicit species/rules/learnset/Oak/new-key delta above and compare the full
  remaining `GameDataSet`; compare all old runtime files and audio exactly.
- Each selected token/number equals the authoritative source; 251 originals
  still exist, all 35 additions and 15 full evolution closures exist, no other
  active/retired species is reintroduced. Build legal partners at levels1,50,100,
  validate evolution methods/items, actual battle effects and every learner's
  source level. Verify LUTEBRISK retains the source Ice typing.
- Full catalog load uses the existing authentic validator for all101 families.
  Existing useful tests are `catalog_rejects_stale_review_import_and_bitplanes`,
  `species_overlay_preserves_real_base_and_rejects_unknown_moves`, and
  `catalog_accepts_pure_and_dual_fairy_species`. The full authored-catalog test
  installs all223; it is useful loader coverage but not the minimal-closure
  output-count oracle. Add a focused minimal35 output/preservation test.
- Use existing `fairy_rules_preserve_unaffected_native_content`,
  `northwoods_extension_preserves_native_moves_and_installs_real_learnsets`,
  `fairy_moves_preserve_existing_content_and_install_earned_counterplay`, and
  animation tests at their exact stage-specific external pack inputs. Negative
  cases should reject duplicate family installation, missing/stale art, missing
  selected native references, changed closure/IDs, missing Fairy prerequisites,
  incompatible modern categories and any unauthorized old catalog mutation.
- Strict written/read identity, full new runtime species verification and all
  377 original PCM hashes/frame counts/loop ranges remain exact. No successful
  result is asserted here because none of those Rust stages ran in this task.

The remaining blockers are execution/acceptance dependencies, not absent source
inputs: Root's repaired terrain acceptance, additive Ely acceptance, then the
actual full loader/build/roundtrip and source-rule regression results. The Fair
map itself still needs its separately scoped additive rebase.
