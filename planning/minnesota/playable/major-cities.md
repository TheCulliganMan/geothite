# Minneapolis, St Paul and Duluth: smaller, tighter cities

The published baseline replaces the96×72 proposals with64×48 whole cities,
retaining original native identities, regional roads, clinics, gyms and story
rooms. **“Smaller but tighter”** remains the active direction: player-camera
review found excessive lawn. [Compact Minneapolis with St. Anthony Falls](minneapolis-falls.md)
is now published at40×32 with all40 entrances and44 rooms retained. Its river
continues through both map edges. St Paul remains a separate source assessment
under the [recomposition contract](../cities/RECOMPOSITION.md). The user has not granted final visual
approval or accepted all20 routes.

[Three whole towns at one scale](../renders/cities/tight-final-v02/whole-town-review/whole-towns-review.png)
shows actual native-source exterior geometry. Every original full-resolution
image, rejected candidate, conversion, player view and raw GPU capture remains
saved. The older connected regional atlas is explicitly historical; it has not
been silently redrawn around these larger town footprints.

## Places and activities

| City | Street entrances / total rooms | Distinct local play |
| --- | --- | --- |
| Minneapolis |40 /44 | Five-floor department store; separate paid markets; print-order delivery; three-plaque mill museum trail; library, native dance-theater music room, workshops and optional club battles. |
| St Paul |40 /42 | Three-floor Lowertown Exchange; fair food, agriculture, exhibition and quiz buildings; archive and river observation trails; public PC/care; original Union Depot and optional battles. |
| Duluth |38 /48 | Rocky hillside stairs and four short ledge lanes; waterfront/piers; three-floor maritime co-op; cargo requisition; two-win training award; reading/weather/union rooms and discoveries. |

The 117 new rooms include upper floors; they are not 117 street doors. The 20
optional trainers do not award regional badges. All 29 new marts use separate
native inventory definitions and ordinary priced purchases.

Minneapolis' Grain Exchange gives FLOWER MAIL for its Print Shop. Delivery
consumes one mail and awards three SUPER POTIONS once. Three separately read
museum plaques unlock a MAGNET. St Paul's fair quiz awards SILVERPOWDER; its
three archive records award a MAGNET and three river markers award MYSTIC WATER.
Duluth's cargo clerk trades one X DEFEND for HP UP; winning both named co-op
battles permits one METAL COAT. Refusals, full-pocket retries, repeat visits and
Continue are separate controller cases. These activities preserve the existing
courier, checkpoint, Harbor and gym programs. Rail service is a separate transit
implementation; the retained depot is not proof of a completed glacier line.

## Authoritative source and fast review

The executable inputs are in `planning/minnesota/towns/`:
`MnMinneapolis`, `MnStPaul` and `MnDuluth` each have `.layout.json`, `.json` rich
dossier and `.expansion.contract.json` native contract. All terrain is explicit
hand-authored strokes and room rows. The [contract guide](../cities/EXPANSION.md)
defines coordinates, indexed old-anchor overrides, connections and event fields.

Use the reusable source reviewer after an edit. Choose a fresh output directory:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_major_city.py \
  --base content-packs/minnesota-northstar-itasca-quality.crystalpack \
  --layout planning/minnesota/towns/MnMinneapolis.layout.json \
  --contract planning/minnesota/towns/MnMinneapolis.expansion.contract.json \
  --views planning/minnesota/renders/minneapolis-expansion/tight64-v01/native-views.json \
  --output planning/minnesota/renders/cities/my-next-minneapolis-review
```

This renders source art without rebuilding a pack. It records source hashes and
elapsed times, renders the complete town plus room/camera views, and retains the
originals before conversion. Source review is not controller or GPU acceptance.
The first full Minneapolis loop, including 39 new rooms/eight cameras/immutable
image writes, measured 13.45 seconds; a warm rerender measured 9.83 seconds under
concurrent verification load. Pack loading measured 11.92 seconds separately.
For focused street edits, use `--exterior-only` and a single-camera view file.
The [measured focused loop](../reviews/major-city-fast-source-review.md) retains
the complete exterior plus one player view in 2.11 seconds warm, with 7.23-second
pack loading separately. Cached repeats do not claim a watch/reload feature.
The later implemented `--watch` mode does reload edited layout/contract/view
files while retaining the pack/art cache. Add `--watch` to the command above
(and `--exterior-only` for street edits). It saves a new immutable revision for
each input hash, including attempted revisions changed mid-render. A bounded
two-revision test measured a real edit-triggered render at 0.53 seconds and
1.08 seconds from edit to verified result; only the expected sprite pixels
changed. `--max-renders 2` bounds a demonstration; omit that limit for ongoing
local authoring. Exact commands and receipts are in the source-review runbook.

## Native publication

The verified final pack is ignored local content:
`content-packs/minnesota-three-cities-tight-final-v02.crystalpack`, SHA256
`052de08898bef175959d7beeab75409b949612d71266e59c86561221a8c137e6`.
It has 660 maps/251 species/251 moves/377 audio programs. It does not yet merge
the separately authored Northwoods species pack, new art experiments or the
distinct 544-map North Shore story successor.

The Rust publisher `expand_authored_city` accepts `PACK LAYOUT CONTRACT OUTPUT`.
Build it with the pinned toolchain, then publish Minneapolis, Duluth, St Paul in
that order from the completed 543-map quality/story parent. Use new output names;
never overwrite an input or historical review pack. The executed commands,
input/binary/pack hashes and strict readback receipts are retained in
[`tight-final-v02/publication-receipts.json`](../renders/cities/tight-final-v02/publication-receipts.json).
The original 17 room modules, original programs, unrelated maps, audio, HOME and
existing recovery records remain preserved. Duluth adds its declared clinic
recovery; original outdoor spawn coordinates and six reciprocal seams move by
exact contracts. This publisher does not migrate old saves across pack identities.

## Evidence and limits

The [independent660-map audit](../renders/cities/expansion-native-proof/final-v02/README.md)
passes 8,768 checks and rejects seven corruptions. The additional
[furniture-only proof](../renders/cities/expansion-native-proof/furniture-v01-to-v02/proof-67a781cd483fd472c04bcf7cc394e420361c031d84196304179839086574548e.json)
certifies that the last pass changes only 410 metatiles in 29 new rooms; every
original map and all three exterior modules/assets remain exact.

Twenty-four actual640×576 exterior frames match native source pixels, with
consistent genuine water animation phases and native foreground priority.
[Seven furnished-room GPU frames](../renders/cities/tight-final-v02/interiors-native-01/production-decisions.png)
match 2,580,480 pixels exactly, including actual indoor palettes, finite camera
clamping and native borders. These are explicit empty-party local placements;
they do not prove earned regional travel. Whole towns, weak street cameras,
all room contact sheets and furnishing comparisons were reviewed by sight.
Some St Paul forecourts still read broad; do not describe perfect density or
final Johto parity as established.

All 377 PCM hashes, frame counts and loop ranges pass on the exact final pack.
Six focused native asset cases pass. [Thirty local controller cases pass](../reviews/major-city-controller-final.json):
all 118 facade entries/returns, every new interior stair, 29 priced marts, optional
battles/PP, care/blackout, both road lanes/checkpoints, local activities, full-bag
retries and Continue. Duluth also walks all five outdoor stairs and all four
ledge lanes, blocked reverse and ordinary returns. Passes come from retained
focused runs; earlier fixture failures are preserved. No post-setup state edits
stand in for quest or battle actions. Earned regional progression remains open.

Scoped Rust formatting passes. Global `cargo fmt --all -- --check` retains the
pre-existing offset-allocator difference. Assets/Bevy all-target Clippy completes
with established allowances (`never_loop`, `bad_bit_mask`, `approx_constant`,
`useless_conversion`) and pre-existing warnings. The combined Core all-target
attempt is blocked by ten existing missing radio-program fixture includes in
`crystal-core/tests/radio_text.rs`; no exported command dumps were added to bypass
that failure. Core library Clippy also completes under those allowances. The focused
map-object identity tests separately verify nonzero
byte-sized authored event indices while preserving twelve live NPC slots,
compiled-map bounds and duplicate checks.

Canonical promotion is recorded in
`revisions/major-cities-tight-final-v02/receipt.json`; its original sources are
retained. The later rich-door label repair preserves author-only Duluth IDs
without changing native fields. Current original HarborWorkshop story annotations
were preserved instead of replacing them with stale candidate metadata.
