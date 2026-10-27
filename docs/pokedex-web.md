# Northwoods Pokédex web app

The Pokédex lives at **https://geothite.ryanculligan.com/pokedex/** alongside
Geothite. Published release `20261027-northwoods-12` uses the current authored
catalog of **220 active species across 100 families**: 39 three-stage families,
42 two-stage families and 19 single-stage species. The rare tail contains the
three-stage mosquito pseudo-legendary, six legendaries and two mythicals.
The source catalog retains 313 species, including 93 cut proposals.

The retained [sprite-only publication](../planning/minnesota/pokemon/expansion/20261027-sprite-publication.md)
updates Dozebear, Denumbra and Mudmender in both views and palettes. It preserves
release10’s verified interface and data: exactly twelve PNGs differ among
1,611 files after immutable asset-URL normalization. All1,926 public byte checks
and six health/private-route checks pass. Desktop and390px mobile front/back/shiny
checks pass for all three species. Flarkindle and the eleven-family native
polish were already included in release10. The separate imported-move animation
feature remains outside that historical sprite release. Release12 preserves
all 1,252 of its sprite PNGs byte-for-byte and adds the completed move previews.

The [Fairy balance pass](northwoods-fairy.md) adds **thirteen regional Fairies
across eight families**, five Fairy attacks, physical Poison Jab and practical
Poison/Steel progression. The final v2 pack contains 471 species, including
thirteen returning Fairy retypes, and 270 moves. All 220 profiles pass the
progression audit with zero findings. Nine production tests cover all eleven
new/signature controller moves, actual damage/PP, native donor compilation,
type matchups/AI, Fairy UI and twenty signature animation variants.
The fresh 1,416 frames, twenty APNGs and twenty filmstrips are retained under
`planning/minnesota/renders/fairy-balance-20261027-v2/`.

Final web integration verifies all eighteen type filters, 220 active entries,
93 cut proposals, 1,252 sprites and nineteen packed extension learnsets.
The [animation overlay](northwoods-move-animations.md) adds distinct native
presentations for all nineteen moves: 76 side/shiny variants and 5,484 retained
frames. Release12 exposes nineteen catalog previews and 141 genuinely learned
move previews across species pages. Native controller/render tests, thirteen
exporter tests and final web integration pass. Actual desktop/mobile browser
controls pass at 1280px and 390px, including mobile Moves navigation, real APNG
motion and corrected native Psychic labels.
Atomic publication12 and all **2,038 public exact-byte checks** pass, plus the
six game-root/health/private-route checks.
The retained sprite publication11 passed 1,926 public exact-byte checks and the
game root/health/private-route checks.
Release09 and its merged v6 captures remain the retained pre-Fairy checkpoint,
with the eleven approved families integrated through the
[23-for-23 replacement plan](../planning/minnesota/pokemon/expansion/20261027-eleven-generation.json).

Release `20261026-northwoods-08` remains retained as historical proof: 219 active
entries / 100 families, with all 1,362 public byte checks passing. The frozen
v4/v5 local candidates and captures are historical proof for their earlier
rosters; they do not establish verification of the merged v6 roster.

The numbered index opens at #001 and always sorts numerically. Active families
remain contiguous: ordinary species #001–209, mosquito pseudo-legendary
#210–212, legendaries #213–218 and mythicals #219–220. Stable source IDs and
LINE labels remain unchanged while active display numbers are densely
repacked. Kensigard opens lake country at #118 immediately before Riceling;
the walleye line is #136–138. These numbers are derived from current master
order and whole-family curation inputs and match the current Fairy v2 export.
The **Show cut proposals** switch adds labeled retired cards after the active
index without changing active numbering. Its `?cut=1` URL state, filters,
family view, notebook and labeled detail pages are covered separately. The JSON
keeps active records in `species` and retained proposals in `cut_species`.

The authored master order in `planning/minnesota/pokemon/pokedex-order.json`
follows regional-game pacing rather than the order families were created.
Its `sections` preserve the grouping rationale, including retired families;
the exporter applies the curation cuts before assigning consecutive display
numbers. These groups describe the dex sequence, not encounter availability.

| Active numbers | Dex progression |
| --- | --- |
| #001–009 | Starter partners |
| #010–039 | First routes and woodland edges |
| #040–065 | Southern rivers, bluffs and fossils |
| #066–095 | Prairie and farmland |
| #096–117 | Twin Cities and town specialties |
| #118–141 | Lake country and fishing |
| #142–165 | Northern forests and peatlands |
| #166–184 | North Shore and deep-water coast |
| #185–190 | Iron Range and abandoned mines |
| #191–203 | Winter habitats and retained source designs |
| #204–209 | Rare ordinary families |
| #210–212 | Pseudo-legendary family |
| #213–218 | Legendaries |
| #219–220 | Mythicals |

Kensigard (MN_277 / LINE126) is a Kensington/Viking-inspired runestone with an
integral carved brow and forked beard, shield-like forearms and broad earthen
feet. It is pure Ground, genderless and Mineral, with no evolution and a 475
base-stat total: 90 HP / 105 Attack / 110 Defense / 35 Speed / 40 Special Attack /
95 Special Defense. Its individual normal RGB555 palette is
`32767 / 17080 / 13641 / 0`. The [Runestone Museum](https://runestonemuseum.org/runestone/)
and [MNopedia](https://www.mnhs.org/mnopedia/search/index/thing/kensington-runestone)
ground the inscribed slab and disputed local story. Its living glyphs and
soil-stamping powers are original fiction. Ground and Flying each have 17 species;
Psychic and Fairy each have 13. All eighteen types span 11–42 active species.

Kindlit → Rimepelt → Borealynx (LINE125, stable source IDs MN_274–276)
is the Fire → Fire → Fire/Ice starter. All three remain quadrupedal.
Cindore → Pelletor → Mesablast (LINE002) stays active in the Iron Range
section with preserved profiles and evolution chain plus refined native art; it has no
starter badge. The withdrawn MN_F124 and MN_271–273 water proposal remains
archived and its IDs are not reused.

The first nine entries are the Grass, Fire and Water starter families. Birchit
evolves into Portabeau and Voyagear. All three starter families use
318→405→530 base-stat totals, evolve at levels 16/36, and have catch rate 45,
gender ratio 31, 20 hatch cycles and Medium Slow growth. Static labels on cards,
families and entries identify the authored starter role. Mega Drain supports
the special Grass starter, with Pouchid learning it at 9. Aqua Jet arrives at 9
for Birchit and 13 for its later stages; Water Pulse at 18 supports every
Water stage. Voyagear gains Vital Throw at 36 and Drain Punch at 38, while
Mesablast gains Iron Head at 36. These use the existing native catalog and
original thirteen-move extension, complemented by the six-move Fairy extension.

Recent designs give Greytag two colorful pre-evolutions, Tapduck and
Dartduck; add Knotitan to the twine family; remake
Cinderfin/Brinebraze as fin-propelled smokehouse fish; and turn
the cookie line into Doughlin→Crumcone→Pailbaron (cookie, white paper cone,
overflowing white bucket). LINE060 remains the two-stage Tugget→Lakernaut
family, with 292/520 base-stat totals and evolution at 30. The unselected
Oreviathan addition and its renders remain archived. The twine family uses
286→430→540 totals and evolves at 26/44. The unregistered MN_F124 water proposal
also remains archived; it does not replace the beaver starter. Each selected
source, exact ImageGen prompt, earlier render, native
candidate and version is retained. The native converter supports opt-in solid
contours and opt-in `source_center` sampling for enlarged native-grid sources.
The selected beaver fronts/rears use center sampling so eye pupils and tooth
separators survive conversion. Voyagear's fierce adult front uses the v4 brow,
cheek and chest refinement. All three backs are proper close rear portraits:
head, shoulders and upper back fill the frame, with lower bodies cropped.
Portabeau's canoe-pack straps match its bareheaded front. Voyagear's raised paw
and paddle T-grip appear at screen right in the rear view, with the blade left,
preserving the same actual holding arm as its front. The furnace family's
Mesablast rear crop begins at source y545 to exclude a detached front-foot bar;
its other five views remain unchanged. Legacy imports retain area-vote behavior; the
graphics loader reconverts center-sampled sources and rejects stale native ink.

The 08 revision replaces the former Swaylet/Seichamp/Surgewrit designs
at stable IDs MN_163–165 with Mitwist → Mispair → Shawlter (LINE076).
One cream mitten with a smaller single side eye becomes two completed mismatched
mittens, then those two mittens plus a white shawl. A smiling fabric fold gives
the shawl expression without eyes on the shawl itself. Types progress
Ghost/Fairy → Ghost/Fairy → Fairy/Flying, with 300→405→515 base-stat totals and evolution
at levels 22/40. Only the final shawl stage gains Fly. The first two identities
and their four reviewed native views remain unchanged in the completed final
v7 refinement. The shawl's front fabric fold forms a continuous smile while
both mittens remain complete; the rear preserves their opposite-side thumbs.
Former Swaylet, Seichamp, Surgewrit, Hemhowl and Gustitch names and
entry routes must disappear from the exported release.
Bobcast/Buoyostic (LINE079) retain their names and Electric → Electric/Psychic
types while their weather-buoy artwork is refined. Their selected source-center
conversion preserves separate eye windows and pupils in all four native views.

Chislet → Buttermuse (LINE098) both use Ice/Fairy. Accepted Chislet artwork is
preserved; Buttermuse becomes an original carved-butter princess with the
Dairy Royal classification, tiara, curled shaving collar and star-cut sash.
The princess uses area-vote sampling with solid contours; accepted Chislet
source and native pixels remain unchanged.
Its 495 base-stat total uses Attack 65 and Special Attack 100. Charm, Encore,
Swift, Heal Bell and Moonlight are actual learnset entries. In the Fairy mod,
Charm and Moonlight are Fairy; Encore, Swift and Heal Bell are Normal.
The original princess artwork was published in 08; release 10 adds its Ice/Fairy
typing and earned Fairy attacks. Production starter-back captures remain retained.

`roster-curation.json` records 43 retired families. LINE050 Taconite, both fossil
choices, the six active legendaries, two mythicals and mosquito line remain.
Active Rock and Ground coverage is 17 species each. The pack builder and website apply
the same whole-family cuts after validating the full source catalog.

The Rust exporter generates static HTML, native/shiny sprites, and the move
catalog at build time. Minimal browser JavaScript handles filters, the local
notebook, sprite controls, tabs and animation file selection. No TypeScript,
third-party font, analytics, network API, content pack or audio payload is
shipped with the website. Habitat proposals are explicitly marked as concepts.

The Moves page includes fourteen imported attacks and five Minnesota signatures.
Entry learnsets come from the completed 270-move Fairy pack with the presentation
overlay, including actual signature levels. All nineteen previews use the production Gen2 script compiler,
OAM playback and battle rasterizer. Rust exports native 160×144 APNGs for player
and enemy sides with normal and shiny actors. They start as native contact stills; Play/Stop,
side and palette buttons select verified files. APNG timing 400/23891 seconds
matches the production 59.7275Hz clock within 0.16 ns per frame.

## Whole-roster native polish

The 20261027 audit visually covers every one of the previous 219 active Pokémon:
100 families and 438 front/back views, plus the new Kensigard pair. Individual
range audits retain concrete observations, input/output hashes, before/after
native boards and every candidate/rejected conversion under
`planning/minnesota/renders/roster-polish-20261027-v1/`.

The three ranges cover 32 families / 73 species, 36 families / 77 species and 32
families / 69 species. Selected changes improve bee wing/rivet clarity,
Flourgeist's millstone/ribbon contours, white snowplow and cod rims, carnival
and mirror shapes, paddlefish anatomy, and close freighter/ice-crack backs.
The accepted concurrent Fortigard eye refinement has its own supplemental
receipt. Good native silhouettes, identities and individual palettes are
preserved; rejected heavier-black alternatives remain saved for reuse.

The 09 web test compares every active normal front/back PNG against its
currently registered canonical native asset and checks conversion metadata.
It does not freeze old source filenames or impose one contour/sampling mode
on all artwork. Pack byte parity and full production-battle capture are
separate checks, so a source sheet alone never establishes in-game fidelity.

## Eleven-family replacement

All 23 new species have root-approved native front/back art and schema-complete profiles. Every stage has its own four-ink RGB555 palette. The source sheets and exact built-in ImageGen prompts below are retained; native files and rejected conversion variants remain in each family’s authoring/audit directories. Source review, full merged-v6 production verification and final fresh-capture web integration have passed. Publication 09 and all 1,926 public byte checks have passed.

| LINE / new family | Selected source | Exact prompt |
| --- | --- | --- |
| 134 / Wadebud → Rakehorn → Marshrack | [front-back.imagegen-v1.png](../planning/minnesota/pokemon/sprites/mn-f134/front-back.imagegen-v1.png) | [prompt-v1.txt](../planning/minnesota/pokemon/sprites/mn-f134/prompt-v1.txt) |
| 135 / Bristub → Latchquill | [front-back.imagegen-v1.png](../planning/minnesota/pokemon/sprites/mn-f135/proposal-v1/front-back.imagegen-v1.png) | [prompt-v1-retry.txt](../planning/minnesota/pokemon/sprites/mn-f135/proposal-v1/prompt-v1-retry.txt) |
| 136 / Densy → Dozebear → Denumbra | [Densy source](../planning/minnesota/pokemon/sprites/mn-f136/front-back.imagegen-v1.png)<br>[Dozebear / Denumbra differentiation](../planning/minnesota/pokemon/sprites/mn-f136/differentiation-v2/front-back.imagegen-v1.png) | [Densy prompt](../planning/minnesota/pokemon/sprites/mn-f136/prompt-v1.txt)<br>[Differentiation prompt](../planning/minnesota/pokemon/sprites/mn-f136/differentiation-v2/prompt-v1.txt) |
| 137 / Ticklet → Solstort | [front-back.imagegen-v1.png](../planning/minnesota/pokemon/sprites/mn-f137/front-back.imagegen-v1.png) | [prompt-v1.txt](../planning/minnesota/pokemon/sprites/mn-f137/prompt-v1.txt) |
| 138 / Peckit → Notchbill → Gougecrest | [front-back.imagegen-v1.png](../planning/minnesota/pokemon/sprites/mn-f138/front-back.imagegen-v1.png) | [prompt-v1.txt](../planning/minnesota/pokemon/sprites/mn-f138/prompt-v1.txt) |
| 139 / Crookit → Crookveil | [front-back.imagegen-v1.png](../planning/minnesota/pokemon/sprites/mn-f139/proposal-v1/front-back.imagegen-v1.png)<br>[front-back.anatomy-v2.png](../planning/minnesota/pokemon/sprites/mn-f139/proposal-v1/front-back.anatomy-v2.png) | [prompt-v1.txt](../planning/minnesota/pokemon/sprites/mn-f139/proposal-v1/prompt-v1.txt)<br>[prompt-anatomy-v2.txt](../planning/minnesota/pokemon/sprites/mn-f139/proposal-v1/prompt-anatomy-v2.txt) |
| 140 / Gillpup → Mudmender | [Gillpup source](../planning/minnesota/pokemon/sprites/mn-f140/front-back.imagegen-v1.png)<br>[Mudmender redraw](../planning/minnesota/pokemon/sprites/mn-f140/mudmender-redesign-v2/front-back.imagegen-v2.png) | [Gillpup prompt](../planning/minnesota/pokemon/sprites/mn-f140/prompt-v1.txt)<br>[Mudmender prompts and review](../planning/minnesota/pokemon/sprites/mn-f140/mudmender-redesign-v2/DESIGN.md) |
| 141 / Velvawn → Shedstag | [front-back.imagegen-v1.png](../planning/minnesota/pokemon/sprites/mn-f141/proposal-v1/front-back.imagegen-v1.png) | [prompt-v1.txt](../planning/minnesota/pokemon/sprites/mn-f141/proposal-v1/prompt-v1.txt) |
| 142 / Dewbit → Glisnare | [front-back.imagegen-v2.png](../planning/minnesota/pokemon/sprites/mn-f142/front-back.imagegen-v2.png) | [prompt-v2-face.txt](../planning/minnesota/pokemon/sprites/mn-f142/prompt-v2-face.txt) |
| 143 / Rondive | [front-back.imagegen-v3.png](../planning/minnesota/pokemon/sprites/mn-f143/front-back.imagegen-v3.png) | [prompt-v3-tall-head.txt](../planning/minnesota/pokemon/sprites/mn-f143/prompt-v3-tall-head.txt) |
| 144 / Driftile | [front-back.imagegen-v1.png](../planning/minnesota/pokemon/sprites/mn-f144/front-back.imagegen-v1.png) | [prompt-v1.txt](../planning/minnesota/pokemon/sprites/mn-f144/prompt-v1.txt) |

The replacement retires F127, F128, F129, F131, F132, F022, F034, F035, F096 and F118 as complete families. Keeping F062’s Dark/Flying windvane pair instead of the repeated Water/Psychic F096 line preserves Flying coverage. River otters F130 and bur oaks F133 also remain. Kensigard’s approved runestone source and Taconarch’s compact final redraw stay selected in the table below. Cut proposals remain inspectable through the website toggle; none of their renders are deleted.

## Selected authoring assets

All creature revisions use built-in ImageGen. Paths below are relative to
`planning/minnesota/pokemon/sprites/`; earlier versions remain alongside them.

| Revision | Selected source | Exact prompt |
| --- | --- | --- |
| Orchid starter | `mn-f001/render-improvement-20261026-v1/front-back.imagegen-v2.png` | `mn-f001/render-improvement-20261026-v1/prompt-v2.txt` |
| Furnace ore family | `mn-f002/render-improvement-20261026-v1/front-back.imagegen-v2.png` | `mn-f002/render-improvement-20261026-v1/prompt-v2.txt` |
| Repair bees | `mn-f008/outline-polish-20261027-v1/bees.imagegen-v1.png` | `mn-f008/outline-polish-20261027-v1/prompt-bees-v1.txt` |
| Flourgeist | `mn-f010/outline-polish-20261027-v1/flourgeist.imagegen-v1.png` | `mn-f010/outline-polish-20261027-v1/prompt-flourgeist-v1.txt` |
| Taconarch final front | `mn-f050/redesign-20261027-v1/taconarch.compact-front-v4.imagegen.png` | `mn-f050/redesign-20261027-v1/prompt-taconarch-compact-front-v4.txt` |
| Taconarch close rear | `mn-f050/redesign-20261027-v1/taconarch.front-back.fresh-v3.imagegen.png` | `mn-f050/redesign-20261027-v1/prompt-taconarch-fresh-v3.txt` |
| Snowplows; refined native contours | `mn-f037/front-back.imagegen-v1.png` | `mn-f037/prompt-v1.txt` |
| Lutebrisk; refined native contours | `mn-f038/front-back.imagegen-v2.png` | `mn-f038/prompt-v2.txt` |
| Kensigard | `mn-f126/runestone-v1/front-back.imagegen-v1.png` | `mn-f126/runestone-v1/prompt-v1.txt` |
| Birchit front | `mn-f004/native-refinement/birchit-front-back-v2.png` | `mn-f004/native-refinement/prompt-birchit-v2.txt` |
| Portabeau front | `mn-f004/sash-v4/portabeau-native-v11.png` | `mn-f004/sash-v4/prompt-portabeau-native-v11.txt` |
| Voyagear fierce front | `mn-f004/native-refinement/voyagear-front-back-ferocity-v4.png` | `mn-f004/native-refinement/prompt-voyagear-ferocity-v4.txt` |
| Birchit / Portabeau close backs | `mn-f004/native-refinement/close-backs-v1/backs.imagegen-v1.png` | `mn-f004/native-refinement/close-backs-v1/prompt-v1.txt` |
| Voyagear close back | `mn-f004/native-refinement/close-backs-v1/voyagear-back-v2.png` | `mn-f004/native-refinement/close-backs-v1/prompt-voyagear-v2.txt` |
| Lynx starter | `mn-f125/front-back.imagegen-v7.png` | `mn-f125/prompt-v7.txt` |
| Icemyth | `mn-f049/redesign-v2/front-back.imagegen-v2.png` | `mn-f049/redesign-v2/prompt-v2.txt` |
| Loomlet / Inverveil | `mn-f080/redesign-v4/front-back.imagegen-v4.png` | `mn-f080/redesign-v4/prompt-v4.txt` |
| Mitwist | `mn-f076/wardrobe-v6/mitwist-front-back.imagegen-v6.png` | `mn-f076/wardrobe-v6/prompt-mitwist-identity-v6.txt` |
| Mispair front | `mn-f076/wardrobe-v3/front-back.imagegen-v3.png` | `mn-f076/wardrobe-v3/prompt-v3.txt` |
| Mispair rear | `mn-f076/wardrobe-v3/front-back.imagegen-rear-v4.png` | `mn-f076/wardrobe-v3/prompt-rear-thumb-v4.txt` |
| Shawlter front | `mn-f076/wardrobe-v7/shawlter-front-back.native-smile-v10.png` | `mn-f076/wardrobe-v7/prompt-native-smile-v10.txt` |
| Shawlter rear | `mn-f076/wardrobe-v7/shawlter-front-back.rear-thumb-v9.png` | `mn-f076/wardrobe-v7/prompt-rear-blue-thumb-v9.txt` |
| Bobcast / Buoyostic | `mn-f079/redesign-v2/front-back.imagegen-v2.png` | `mn-f079/redesign-v2/prompt-v2.txt` |
| Buttermuse princess | `mn-f098/princess-v2/front-back.imagegen-v1.png` | `mn-f098/princess-v2/prompt-v1.txt` |
| Frontagal | `mn-f115/redesign-v3/front-back.imagegen-v1.png` | `mn-f115/redesign-v3/prompt-v1.txt` |
| Knotitan | `mn-f063/twine-v2/front-back.imagegen-v1.png` | `mn-f063/twine-v2/prompt-v1.txt` |
| Cinderfin | `mn-f083/redesign-v2/front-back.imagegen-v2.png` | `mn-f083/redesign-v2/prompt-v2.txt` |
| Brinebraze | `mn-f083/redesign-v2/front-back.imagegen-collar-v3.png` | `mn-f083/redesign-v2/prompt-collar-v3.txt` |
| Crumcone / Pailbaron | `mn-f090/redesign-v3/front-back.imagegen-v3.png` | `mn-f090/redesign-v3/prompt-v3.txt` |
| Flarkindle | `mn-f116/redesign-v4/front-back.imagegen-v2.png` | `mn-f116/redesign-v4/prompt-v2.txt` |

Each family also retains its reviewed native sheet, canonical conversion
manifest, old profiles and earlier source art. Immutable archive inventories
and objects live under `planning/minnesota/renders/creature-archive/`, including
rejected conversions and the verified v1/v2 signature-animation captures.
New starter source selections are recorded in each family's canonical manifest;
all earlier images and the unselected freighter/water proposals remain retained.
The final mitten-family manifest uses source-center sampling and solid contours
for all six views. Its full-family review SHA256 is
`554bbaf6ab5122e617d40b5f326b663f24dc0170086757db6cb70fc6bdd45ebc`.
The production graphics catalog has validated the final family, and the v3 pack
and published 08 web export include it. All earlier
final-stage artwork remains archived, together with the complete-mittens and smiling-fold prompts leading
to the selected native-smile revision.

## Build and verify

Run from the repository root with the pinned Rust toolchain. Inputs are the
canonical authoring JSON, completed native sprite outputs and a verified local
base content pack for move facts. Packs and output directories remain ignored.
The root-owned merged v6 species and move packs have built. They contain 471
species (251 native plus 220 regional) and 264 moves; the source authoring
catalog has 313 species. The final local 09 export has 1,611 files: 220 active
entries and93 labeled cut proposals, 100 active families and 1,252 normal/shiny
front/back sprites. Full `app.test.mjs` integration, including the cut-notebook
suite, passed on this merged roster and passed again after the fresh-v6
signature export. All 1,611 final files are byte-identical to the tested preview;
there are no changed or removed files.

The frozen v5 pack passed all four production tests before the eleven-family
replacement. Its historical captures remain under `roster-battles-20261027-v2`,
`signatures-20261027-v5` and `starters-battle-20261027-v5`. Those proofs do not
replace the fresh merged-roster production checks. The following commands
record the verified v6 build, final verification and published 09 path:

```sh
CARGO_TARGET_DIR=target/agent-northwoods-publish cargo build --locked \
  -p crystal-assets --bin pack_minnesota_pokemon --bin pack_northwoods_moves \
  --bin export_minnesota_pokedex_web \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0'

target/agent-northwoods-publish/debug/pack_minnesota_pokemon \
  content-packs/minnesota-geography-review.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/northwoods-species-v6.crystalpack

target/agent-northwoods-publish/debug/pack_northwoods_moves \
  content-packs/northwoods-species-v6.crystalpack \
  content-packs/northwoods-moves-v6.crystalpack

target/agent-northwoods-publish/debug/export_minnesota_pokedex_web \
  content-packs/northwoods-moves-v6.crystalpack \
  planning/minnesota/pokemon \
  target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  target/minnesota-pokedex-web/pokedex/releases/20261027-northwoods-09 \
  /pokedex/releases/20261027-northwoods-09 \
  planning/minnesota/renders/signatures-20261027-v6

POKEDEX_WEB_RELEASE=target/minnesota-pokedex-web/pokedex/releases/20261027-northwoods-09 \
  node web-client/pokedex/app.test.mjs

CARGO_TARGET_DIR=target/agent-northwoods-publish cargo test --locked \
  -p crystal-assets --bin export_minnesota_pokedex_web \
  --config 'profile.dev.package.crystal-assets.opt-level=0'
```

Use a fresh release ID when changing assets. The exporter checks complete
active-species/front-and-back coverage, validates RGB555 palettes, rejects missing move
facts and rejects unresolved HTML template fields. Browser integration tests
check every active and cut entry/image, search, dual-type intersection, eight rare species,
three pseudo-legendary stages, all active families, notebook persistence, sprite
views and keyboard-accessible move tabs. The 09 assertions additionally cover
the nine starter identities/roles/stats/evolutions and move levels, the two-stage
freighter family, the twine progression, and absence of unregistered species. Retired species remain labeled
cut proposals, hidden by default; Kensigard stays active. The retained v4 local 09 candidate was inspected at
desktop and 390px mobile widths: revised paddlefish images load, body width
stays 390px, Kensigard appears at #112 as pure Ground, and no browser errors were
recorded. The previously verified 08 mitten browser proofs remain under
`planning/minnesota/renders/proposals-20261026-v1/release08-web-proof/`.

The updated 09 web assertions also reject every replaced seiche and intermediate
wool-ghost name/route, verify the 300/405/515 mitten progression at 22/40,
and exercise the Ghost/Flying intersection with only Shawlter visible from its
line. They check both butter species'
Normal/Ice badges, Dairy Royal classification, special-stat redistribution and
rendered native move rows. Every active family's normal sprite bytes must match
its selected native files and recorded conversion metadata, using the canonical
sprite registry. The evolution mix must remain 39 three-stage, 42 two-stage and
19 single. Kensigard must remain an ordinary pure Ground solo with exact normal
palette, 475 stats, genderless Mineral profile, valid learned moves, no evolution,
and working distinct normal/shiny front/back sprites. Every starter's front/back
and shiny controls are exercised, catching stale
preview files as well as incorrect URLs. The complete integration command above
passed on the merged v6 preview and final fresh-signature export. The new checks cover all eleven approved families, 23 complete
authored profiles and distinct shiny views, exact 23-for-23 cuts, canonical
master order and preservation of Taconite, Kensigard, windvane, otter and oak families. It verifies the authored two-stage freighter
and butter move levels. Broad thematic search can also match ordinary species,
while starter roles and badges remain restricted to the three designated families.

## Verification

The current 313-source authored-catalog test passed (22.93s), including every
active species' party/battle/evolution construction and preservation of native
content. The exact check is
`authored_catalog_constructs_parties_battles_evolutions_and_preserves_base`.
The v6
`northwoods_extension_preserves_native_moves_and_installs_real_learnsets`
check also passed (2.79s), preserving original moves, assets, maps and audio.

All four current-v6 production tests passed, 0 failed, exit 0 (88.02s):

- `northwoods_full_roster_renders_front_and_back_in_production_battles`
  covers all 220 active species, exact 440 canonical PNG/2bpp pairs, 220 palettes,
  56/48 textures and actual battle-anchor rasters. All 100 visual-review hashes
  match. The 909 files under `roster-battles-20261027-v3/` retain 220 native battles,
  220 nearest 4x views, 440 front/back banks, 14 native and 14 review boards, and order.
- `northwoods_signatures_execute_through_production_controller` proves actual
  PP/damage/HP events, dialogue and the correct authored program for all five
  signatures, including valid enemy healing after damage.
- `northwoods_signatures_render_native_frames_both_sides_and_cleanup` verifies
  all 20 side/shiny variants, native OAM/scanline/tile/palette/mirror behavior,
  APNG pixel equality/timing and cleanup. `signatures-20261027-v6/` has 1,456
  files: 1,416 frame PNGs, 20 APNGs and 20 filmstrips.
- `northwoods_starter_backs_render_in_production_classic_battles` verifies the
  nine regional starters and native Totodile/Croconaw/Feraligatr. Its 38 files
  under `starters-battle-20261027-v6/` retain 12 battles, 12 nearest 4x scenes,
  12 extracted backs and two boards.

Root inspected production boards 4, 5, 11, 12, 13 and 14, including all 23 new species
and the changed art. All seven shared source globals remained unchanged during
capture. [The exact v6 proof receipt](../planning/minnesota/renders/roster-polish-20261027-v1/audit-f081-f125/verification-v6/summary.json)
records the test names, canonical inventory and pack SHA256
`fccf7e056d35091f6eefaa5ada02f764d278db5ab24e1eb023ef8923a047c47d`.
Packs/captures remain ignored; the static website does not ship the game pack.
Final local web integration passes all 1,611 files and the active/cut notebook
flows. Publication and all 1,926 public-byte checks pass.

Reproduce the current four production checks with:

```sh
RUST_MIN_STACK=67108864 \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/northwoods-moves-v6.crystalpack" \
NORTHWOODS_ROSTER_RENDER_DIR="$PWD/planning/minnesota/renders/roster-battles-20261027-v3" \
NORTHWOODS_ANIMATION_RENDER_DIR="$PWD/planning/minnesota/renders/signatures-20261027-v6" \
NORTHWOODS_STARTER_RENDER_DIR="$PWD/planning/minnesota/renders/starters-battle-20261027-v6" \
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/agent-northwoods-animation \
cargo test --locked -p crystal-bevy --lib northwoods_ \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0' \
  --config 'profile.dev.package.crystal-bevy.opt-level=0' -- --nocapture
```

The v4/v5 evidence later in this section remains historical and does not replace
these fresh merged-roster results.

The core extension has six authoritative move regressions and eleven schema
regressions, covering all 13 definitions, damage and PP, priority, half-drain,
stat changes, confusion, flinch, wide indices and native Metronome isolation.
The external-pack preservation test verifies that the original 251 moves,
animations, runtime assets, species, maps and all compiled audio remain exact.
Focused sprite converter and graphics-loader regressions cover legacy byte
identity, closed contours, white details, loop holes, open rear cropping,
center-sampled pupils/tooth separators and strict source-to-native validation.

For history, two focused external-input checks passed against the frozen v4 authored catalog:
`authored_catalog_constructs_parties_battles_evolutions_and_preserves_base`
constructs every added species at levels 1/50/100, validates Party/Day Care and
capture metadata, exercises real damage/PP and authored level evolutions, and
preserves native species, runtime files and compiled audio (20.71s).
`northwoods_extension_preserves_native_moves_and_installs_real_learnsets`
preserves the original 251 move definitions, animations, runtime assets, native
learnsets, species, maps and audio while installing the 13 extension moves and
their actual learners (5.38s). Use these exact filters with the current v6 inputs
below so other ignored fixtures requiring unrelated inputs are not included:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/minnesota-geography-review.crystalpack" \
CRYSTAL_MINNESOTA_POKEMON_ROOT="$PWD/planning/minnesota/pokemon" \
CRYSTAL_MINNESOTA_NATIVE_ROOT="$PWD/target/minnesota-pokemon/native" \
CRYSTAL_MINNESOTA_PROFILES="$PWD/planning/minnesota/pokemon/species-profiles.json" \
CARGO_TARGET_DIR=target/agent-northwoods-publish \
cargo test --locked -p crystal-assets --lib \
  minnesota_pokemon_modpack::authored_tests::authored_catalog_constructs_parties_battles_evolutions_and_preserves_base \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0' -- --ignored --exact

CRYSTAL_NORTHWOODS_TEST_PACK="$PWD/content-packs/northwoods-species-v6.crystalpack" \
CARGO_TARGET_DIR=target/agent-northwoods-publish \
cargo test --locked -p crystal-assets --lib \
  northwoods_moves_modpack::tests::northwoods_extension_preserves_native_moves_and_installs_real_learnsets \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0' -- --ignored --exact
```

The fresh v4 move pack contains 471 total species (251 native plus 220 regional)
and 264 moves. All five signatures pass through the production
VisibleShellController with actual PP use, damage events and HP scene changes,
dialogue, and the correct authored animation program. Legitimate enemy Rest
healing after damage remains supported by the regression. All 20 side/palette
animation variants pass native OAM/scanline bounds, tile/palette checks, mirror
checks, APNG frame equality/timing and terminal cleanup against this exact pack.

Fresh v4 captures are retained at
`planning/minnesota/renders/signatures-20261027-v4/`: 1,456 files comprising
1,416 frame PNGs, 20 APNGs and 20 filmstrips. The retained v4 local 09 candidate
uses these captures. Its 1,146 files were byte-identical to the earlier local preview,
including all 40 APNG/still files; full web integration passed again after the
fresh-capture export. The Taconarch change subsequently passed frozen v5 parity and captures.
Those results also predate the eleven-family replacement; the fresh merged-v6 results above supersede them. The v4 commands below
record the historical inputs:

```sh
RUST_MIN_STACK=67108864 \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/northwoods-moves-v4.crystalpack" \
NORTHWOODS_ANIMATION_RENDER_DIR="$PWD/planning/minnesota/renders/signatures-20261027-v4" \
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/agent-northwoods-animation \
cargo test --locked -p crystal-bevy --lib northwoods_signatures_ \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0' \
  --config 'profile.dev.package.crystal-bevy.opt-level=0' -- --nocapture
```

The full-roster production test
`northwoods_full_roster_renders_front_and_back_in_production_battles` passed
for all 220 species in 18.04s. It verifies exact selected canonical PNG/2bpp and
palette parity with the pack, 56/48 source textures, and the actual front/back
battle-anchor rasters. The 909 retained files under
`planning/minnesota/renders/roster-battles-20261027-v1/` include 220 native
battle scenes, 220 nearest 4x scenes, 440 front/back sprite banks, 14 native
and 14 enlarged grouped boards, and `species-order.json`. Root subsequently accepted the corrected Taconarch board and all four frozen
v5 production tests passed (100.33s). The merged v6 full roster was subsequently captured and reviewed as recorded
above. Historical v4 command:

```sh
RUST_MIN_STACK=67108864 \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/northwoods-moves-v4.crystalpack" \
NORTHWOODS_ROSTER_RENDER_DIR="$PWD/planning/minnesota/renders/roster-battles-20261027-v1" \
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/agent-northwoods-animation \
cargo test --locked -p crystal-bevy --lib \
  northwoods_full_roster_renders_front_and_back_in_production_battles \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0' \
  --config 'profile.dev.package.crystal-bevy.opt-level=0' -- --nocapture
```

The separate starter/reference regression also passed against the v4 pack.
It captures the nine regional starters alongside native Totodile, Croconaw and
Feraligatr through the same classic battle renderer. Its 38 retained files are
under `planning/minnesota/renders/starters-battle-20261027-v4/`: 12 full
battles, 12 nearest 4x views, 12 extracted 48x48 backs and two review boards.
The test is `northwoods_starter_backs_render_in_production_classic_battles`;
set `NORTHWOODS_STARTER_RENDER_DIR` to a fresh ignored capture directory.
Capture directories and earlier revisions are never discarded.

For history, the verified 08 v3 pack had 470 species and 264 moves. Its two
signature/controller tests passed in 9.60s and its starter test in 4.71s.
Those earlier captures remain under `signatures-20261026-v3/` and
`starters-battle-20261026-v1/` beside the new retained revisions. They do not
substitute for the merged v6 results above.

Workspace formatting reports existing differences beginning in
`crates/offset-allocator`. Changed leaves are formatted independently. Required
Clippy across assets/core/runtime/bevy remains blocked by the existing
`clippy::bad_bit_mask` error at
`crates/crystal-core/src/multiplayer/mod.rs:1332`; this is not a passing check.

Release `20261027-northwoods-09` is published and publicly verified. The deployer
switched the release atomically while preserving the existing service image,
environment, ports, networks and mounts. All1,926 public exact-byte checks pass:
all 1,611 release files, 313 canonical species pages, the index and Moves page.
The root and health endpoint return 200. All 93 cut pages return 200 with the
CUT PROPOSAL banner; unregistered Oreviathan, Vanesail and Sodger routes and
private backup paths return 404.

Live desktop and 390px mobile review of the dream-den family passed: body width
is 390px without horizontal overflow, both images load, and Denumbra's back/shiny
controls use fresh 09 assets without browser errors. Kensigard was also reviewed
on the live site. [Public verification details](../planning/minnesota/renders/roster-polish-20261027-v1/web-proof/public-verification.json)
and the public screenshot proofs remain under
`planning/minnesota/renders/roster-polish-20261027-v1/web-proof/`.
The completed 09 archive retains 61,656 files in the immutable
[inventory `f2cb53850e6282285ee279a07e9638f4b0871fe2ac63466feb6eb4c039155696`](../planning/minnesota/renders/creature-archive/inventory-f2cb53850e6282285ee279a07e9638f4b0871fe2ac63466feb6eb4c039155696.json).
It includes every new source/prompt/reject/native candidate, all fresh v6
battle/signature/starter captures, live public proofs and exported normal/shiny
PNG/APNG variants. All 100 current review references and 1,320 selected view
files match the frozen capture inventory; all seven shared source globals
remain unchanged.

For history, release `20261026-northwoods-08` was published with the deployment script.
The exact public index, game root and health checks pass, and the existing
service image, environment, ports, networks and original mounts remain
unchanged. The public byte comparison passes for all 1,362 checked URLs:
all 1,141 release files, 219 canonical entries, the index and Moves page. The
game root and health endpoint return 200, and seven retired/private routes
return 404. Live Shawlter back/shiny controls use the current 08 assets and work
without browser warnings or errors. Local and public browser proofs are
retained in the release08 web-proof directory noted above.

The verified 08 durable creature archive contains 33,600 retained files in immutable
inventory `30b0ab93263fa3a0695a0aea8415a11cfc3568afeea3f58dfc0b39fd48a61daa`.
It includes the live mitten desktop and Shawlter shiny-back proofs, the actual
starter battle board, every retained source revision and the signature captures.

The extended move pack requires a matching rebuilt engine because move source
indices are now u16. This release publishes only static Pokédex assets to the
existing web service; it does not replace the running game's content pack or
executable.

## Sprite-only publication

For an art revision that preserves a verified interface while other UI work is
in progress, build the Rust `publish_pokedex_sprite_revision` example. It takes
an existing release, a fresh ignored output path and stable species IDs. It
copies the baseline with new immutable asset URLs and regenerates only the
selected normal/shiny sprite pairs from the canonical native indexed art.
Release11 used baseline10 and `MN_301 MN_302 MN_311`. Exhaustive baseline parity
and desktop/mobile controls were checked before atomic deployment.

The accepted release is at
`target/minnesota-pokedex-web/pokedex/releases/20261027-sprites-11`.
The [release record](../planning/minnesota/pokemon/expansion/20261027-sprite-publication.md)
links its native battle captures, browser proof, public verification and retained
sources. The existing deployment command accepts this new release path and ID.

## Deploy to the homelab

`sshpop` was not a resolvable SSH alias on the authoring machine. Its verified
Tailscale address is `100.101.29.133` (`pop-server`), with the homelab at
`/mnt/homelab`. Release 09 was published successfully with the command below. Its local
production/integration checks and all 1,926 public-byte checks pass:

```sh
bash tools/deploy-pokedex.sh 100.101.29.133 \
  target/minnesota-pokedex-web/pokedex/releases/20261027-northwoods-09 \
  20261027-northwoods-09
```

The deployer transfers only the static release, obtains the homelab Docker
update lock, and adds one read-only bind to the existing `geothite` Compose
service:

```text
/mnt/homelab/config/geothite/pokedex:/srv/crystal/web/pokedex:ro
```

Only the first deployment needs to recreate that service. Subsequent releases
switch a relative `current` symlink atomically. Canonical entry routes use
`/pokedex/species/<name>.html`; assets use immutable release paths under
`/pokedex/releases/<release-id>/`. Old release assets remain available for open
browser tabs. The existing Rust static-file service handles these paths.

Before changing Compose, the deployer privately backs up its original bytes and
container configuration in `/mnt/homelab/config/geothite/pokedex-backups/`, outside the served tree. These files may
contain secrets: their directory is mode0700, files mode0600, and they must
never be published or copied into this repository. The deployment compares the
image, environment, ports, networks and all previous mounts in memory without
printing secrets. It checks health, the game root and the exact served guide
HTML, and automatically restores Compose and the old release on failure.

For a content-only rollback, point `current` at a previously retained release
using a temporary symlink and atomic rename. To remove the integration, restore
the private Compose backup and recreate only `geothite` using
`docker compose up -d --no-deps --no-build --pull never geothite` from
`/mnt/homelab`. Do not run a whole-stack restart or remove the game data volume.
