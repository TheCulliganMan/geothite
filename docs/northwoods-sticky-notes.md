# Northwoods sticky-note restoration

LINE087 restores **Jotlet → Memofold → Recallisk** as a complete ordinary
family. All three are pure Psychic. The source remains 313 species; the active
roster is **223 species across 101 families**, with 90 cut proposals and an
active mix of 40 three-stage, 42 two-stage and 19 single-stage families. No other
family is retired to make room. The six legendaries, two mythicals and mosquito
pseudo-legendary remain at the end of the regional order.

Release `20261027-sticky-13` is published. Its pack, full web integration and
three native production tests pass. All 2,038 public exact-byte checks and six
status checks pass.
Retained `20261027-northwoods-12` is the frozen 220-species checkpoint.

## Design and stable identity

| Stage | Source ID / runtime number | Body | Total stats | Evolution |
| --- | --- | --- | ---: | --- |
| Jotlet | MN_186 / 437 | One living square sticky note | 285 | Level 24 |
| Memofold | MN_187 / 438 | Three connected notes, with two peeling side sheets | 390 | Level 44 |
| Recallisk | MN_188 / 439 | Seven-note keeper with a central face, four side-stack sheets and two base sheets | 500 | Final |

The sheets themselves form the creatures: no animal body, dragon neck, tail,
wings, writing or logos. The independent close rear portraits show blank reverse
pages, adhesive edges and broad overlap seams. The larger front fits are 40/48/54
pixels inside the 56×56 native canvas; rear portraits use 46×47 inside 48×48.
Area-vote sampling and the opt-in Solid contour preserve closed paper rims,
separate white eye windows, black pupils and broad structural seams. No hand
painting or pixel edits were used.

Each species retains four RGB555 inks, including white and black:

| Species | Normal RGB555 palette |
| --- | --- |
| Jotlet | 32767 / 17310 / 6445 / 0 |
| Memofold | 32767 / 27324 / 14572 / 0 |
| Recallisk | 32767 / 23421 / 15944 / 0 |

The [canonical manifest](../planning/minnesota/pokemon/sprites/mn-f087/manifest.json)
selects the [ImageGen source](../planning/minnesota/pokemon/sprites/mn-f087/redesign-20261027-v2/front-back.imagegen-v2.png)
and [exact prompt](../planning/minnesota/pokemon/sprites/mn-f087/redesign-20261027-v2/prompt-v2.txt).
All earlier artwork, rejected sampling/fit candidates and source baselines remain
under `redesign-20261027-v2/`; `integration-before/` preserves the original global
records before restoration. Root and independent native review approved the six
views, tied to review SHA-256
`323f976da8223b6c64ce6e716e6b1a4069a8881fa09e173c25bf1caf4fd1cda3`.
The strict production graphics loader also passes.

Approved stats are 285/390/500. Recallisk has 90 HP / 50 Attack / 80 Defense /
45 Speed / 110 Special Attack / 125 Special Defense. Levels 24/44, genderless
Mineral/Indeterminate breeding and Medium Slow growth remain. Reflect and Light
Screen are earned level-up moves; the machine list contains actual Crystal
TM/HM tokens. Dragon Breath and Outrage are removed from this pure-Psychic
redesign. The proposal's strict profile and native-token validation passed
before canonical integration.

The existing town-section order places the restored family at regional
**#118–120**, without editing the master order. Stable app LINE087, source IDs
and all previous species' relative order are preserved. The rare tail becomes
#213–223. Psychic coverage rises 13→16; Dragon remains 11. All 310 other authored
profiles, other retirements and unrelated roster/moveset/palette records are
preserved.

## Completed Rust build

The final ignored pack is `content-packs/northwoods-sticky-v1.crystalpack`,
containing **474 species and 270 moves**. Its SHA-256 is
`77a7a72b41b289d8cb14161fc57b81aaab394feae6cb855cf55efad7a51b8ddc`.
The native 251 species plus 223 regional species account for 474; the nineteen
extension moves are unchanged.

Start from the existing verified Fairy type base, with its eighteen-type chart
and returning Fairy retypes. The Minnesota publisher installs or verifies the
modern per-move split. A completed species/animation pack cannot be used as the
base for adding this family again: duplicate-overlay guards deliberately reject
that operation. The root publisher recorded this pipeline; use fresh ignored
output names for a later rebuild rather than overwriting retained packs.

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/agent-northwoods-publish \
cargo build --locked -p crystal-assets \
  --bin pack_minnesota_pokemon --bin pack_northwoods_moves \
  --bin pack_fairy_type --bin pack_northwoods_animations \
  --bin export_minnesota_pokedex_web --bin audit_pokemon_balance \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0'

target/agent-northwoods-publish/debug/pack_minnesota_pokemon \
  content-packs/northwoods-fairy-base-v1.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/northwoods-sticky-species-v1.crystalpack

target/agent-northwoods-publish/debug/pack_northwoods_moves \
  content-packs/northwoods-sticky-species-v1.crystalpack \
  content-packs/northwoods-sticky-northwoods-v1.crystalpack

target/agent-northwoods-publish/debug/pack_fairy_type moves \
  content-packs/northwoods-sticky-northwoods-v1.crystalpack \
  content-packs/northwoods-sticky-fairy-v1.crystalpack \
  planning/minnesota/pokemon/fairy-moves.json

target/agent-northwoods-publish/debug/pack_northwoods_animations \
  content-packs/northwoods-sticky-fairy-v1.crystalpack \
  content-packs/northwoods-sticky-v1.crystalpack
```

Read-back verification passes for every completed stage. The final `--check-pack`
audit reports **zero findings**, **22,300 default move sets**, **631 battles
spending real PP and dealing damage**, and **122 evolution boundaries**:

```sh
target/agent-northwoods-publish/debug/audit_pokemon_balance \
  content-packs/northwoods-sticky-v1.crystalpack \
  planning/minnesota/pokemon --check-pack
```

Pack preservation verifies unchanged existing move facts, learnsets, type chart,
all nineteen authored animations and original species records. Compiled audio,
PCM hashes, frame counts and loop ranges are exact. The only changed existing
runtime files relative to the earlier animated-v 2 pack are the twelve separately
approved Dozebear/Denumbra/Mudmender sprite files from sprite publication 11;
all other existing runtime files are exact. This restoration adds no encounters
or campaign placement.

## Local web and production checks

The completed immutable candidate was exported with the unchanged 76 native
animation variants and their retained contact-frame metadata:

```sh
target/agent-northwoods-publish/debug/export_minnesota_pokedex_web \
  content-packs/northwoods-sticky-v1.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  target/minnesota-pokedex-web/pokedex/releases/20261027-sticky-13 \
  /pokedex/releases/20261027-sticky-13 \
  planning/minnesota/renders/moves-animation-20261027-v1/captures

POKEDEX_WEB_RELEASE=target/minnesota-pokedex-web/pokedex/releases/20261027-sticky-13 \
POKEDEX_ANIMATION_SOURCE=planning/minnesota/renders/moves-animation-20261027-v1/captures \
node web-client/pokedex/app.test.mjs
```

Full JSDOM integration passes for 223 active entries, 101 families, 90 labeled cut
proposals and all 313 species pages. It checks restored ordinary-family order,
approved identities/types/stats/evolutions, every active family's reviewed
native art and all nineteen playable move cards. The 76 APNGs and 76 idle stills
match their retained native source files exactly. Each species page shows only
extension moves it genuinely learns.

Exact release 12→13 comparison confirms only the twelve F087 normal/shiny
front/back PNGs change. The other 1,240 sprite PNGs, all 152 animation/still files,
browser JavaScript/CSS and complete move catalog are byte-identical. Every old
active identity and relative order is retained, with the same eleven-species
rare tail. The unregistered bat proposal remains separate.

All three native production regressions pass on the exact final pack in 9.81s:

- `northwoods_sticky_family_evolves_at_24_and_44_through_production_pack` uses the
  actual Start→PACK→Rare Candy path. Levels 23/43 stop short; 24/44 evolve to the
  correct stage and grant its new move.
- `northwoods_sticky_family_uses_stage_legal_moves_through_production_controller`
  selects Psybeam through FIGHT at Jotlet 23, Memofold 24 and Recallisk 44, proving
  authoritative PP expenditure and target damage.

- `northwoods_sticky_family_renders_normal_and_shiny_in_production_battles`
  retains six actual normal/shiny battle screens and twelve exact front/back
  extracts. Root visually accepted all six captures and their close rear framing.
  Screens, nearest 4× views and contact boards remain under
  `planning/minnesota/renders/sticky-note-20261027-v2/battles/`.

The production log is
`planning/minnesota/renders/sticky-note-20261027-v2/production-tests.log`.
Deployment published release 13 atomically, preserving the service image,
environment, ports, networks and mounts. Exact root HTML and game health checks
pass. The completed public sweep passes all 2,038 exact-byte checks and six
status checks, with zero failures. Live browser review accepts all three
56×56 family fronts and the 223-species index; its screenshot is retained as
`web-published-family.png` in the verification directory. No complete-playthrough
claim is made for the regional campaign.

```sh
bash tools/deploy-pokedex.sh 100.101.29.133 \
  target/minnesota-pokedex-web/pokedex/releases/20261027-sticky-13 \
  20261027-sticky-13
```

The [native verification receipt](../planning/minnesota/renders/sticky-note-20261027-v2/verification-summary.json)
and [public-byte summary](../planning/minnesota/renders/sticky-note-20261027-v2/verification/public-byte-summary.json)
record the completed proofs. Build, pack-preservation, final balance,
web-integration and release-parity receipts are retained under
`planning/minnesota/renders/sticky-note-20261027-v2/verification/`. Existing
workspace-formatting and core Clippy baseline failures remain documented in the
[animation runbook](northwoods-move-animations.md); they were not repaired by
this content-only restoration. Packs, generated native files and verification
output remain ignored and are never staged.
