# Minnesota Pokémon

Current animation release `20261027-northwoods-12` is published: all nineteen added moves have native Gen II animations, 76 side/shiny variants and 141 genuinely learned previews across species pages. The 220-species Fairy v2 roster and all 1,252 sprite PNGs from sprite publication11 are preserved. [Animation build and verification](../../../docs/northwoods-move-animations.md) describe the completed native/controller/web checks.

The retained [sprite publication `20261027-sprites-11`](expansion/20261027-sprite-publication.md) updates Dozebear, Denumbra and Mudmender in front/back and shiny views. Its 1,926 public byte comparisons and desktop/mobile controls passed.

[Echolet → Sonatlas](sprites/proposals/bat-20261027-v1/DESIGN.md) is a separately saved, unregistered Psychic/Flying bat proposal with native front/back conversion candidates. It addresses fast Psychic and special Flying coverage; it has not changed the active species count or live move catalog.

Fairy baseline release `20261027-northwoods-10` is published and verified: thirteen regional Fairies, all 220 active species, five Fairy attacks plus Poison Jab, and all 1,926 public exact-byte checks passed. [The verified rules and build](../../../docs/northwoods-fairy.md) describe the local v2 pack and completed battle/animation checks.

The current source catalog contains **313 original species**, with **220 active species across 100 families** after curation: **39 three-stage families, 42 two-stage families and 19 single-stage species**. Kindlit → Rimepelt → Borealynx is the quadrupedal Fire → Fire → Fire/Ice starter. Cindore → Pelletor → Mesablast remains active as an ordinary Iron Range family. Stable source IDs MN_271–273 remain reserved for the withdrawn water proposal; the lynx uses MN_274–276 / MN_F125. Earlier source designs and every render remain retained.

**Kensigard (MN_277 / LINE126)** is the new ordinary pure Ground solo: a thick Kensington/Viking-inspired runestone with an integral carved brow, forked beard, shield-like forearms and buried-root feet. Its 475 base-stat total is 90 HP / 105 Attack / 110 Defense / 35 Speed / 40 Special Attack / 95 Special Defense; it is genderless, belongs to the Mineral egg group and has no evolution. Its own normal RGB555 palette is `32767 / 17080 / 13641 / 0`. It opens lake country immediately before Riceling, preserving every earlier species' relative order and the rare families at the end.

The [Runestone Museum](https://runestonemuseum.org/runestone/) documents the artifact's discovery near Kensington in 1898 and its disputed origin; [MNopedia](https://www.mnhs.org/mnopedia/search/index/thing/kensington-runestone) describes the inscribed graywacke slab and the authenticity debate. Kensigard's moving glyphs and soil-stamping powers are original fiction, not a claim of medieval Norse settlement in Minnesota.

The current [Fairy ruleset](../../../docs/northwoods-fairy.md) preserves all 220 species while adding thirteen regional Fairies, thirteen returning retypes, five Fairy attacks and physical Poison Jab. Compile the Fairy type base before the species overlay; the runbook provides the complete pipeline. Earlier releases below retain their frozen pre-Fairy verification history.

Historical web release `20261027-northwoods-09` uses the built merged v6 species/move packs; full final local web integration and all four production capture/controller tests have passed. Atomic deployment and all 1,926 public exact-byte checks have passed. All eleven user-approved themed families are integrated, replacing 23 species from complete overlapping lines and keeping 220 active. The approved Taconarch redraw and Kensigard remain; the first two taconite stages, types, palettes and balanced profiles are preserved. Earlier v4/v5 packs and captures remain historical proof for their frozen rosters. Release `20261026-northwoods-08` remains retained as historical proof. The native-art polish audit and all candidates are retained under `planning/minnesota/renders/roster-polish-20261027-v1/`; the F001–F040 slice covers every one of its 32 active families / 73 species / 146 views, with approved bee, Flourgeist, snowplow and Lutebrisk refinements.

The [eleven-family generation plan](expansion/20261027-eleven-generation.json) records the approved 23-for-23 replacement. New families cover underwater-gardener moose, locking-anchor porcupines, dream-den bears, sundial turtles, cavity-architect woodpeckers, false-branch martens, nursery mudpuppies, returning-antler deer, finger-trap sundews, four-eyed whirligigs and leaf-puzzle rafts. F062's Dark/Flying windvane pair stays active; the repeated Water/Psychic F096 line is retired instead. River otters F130 and bur oaks F133 also remain. Every approved family retains its sources, exact prompts, individual palettes and reviewed native front/back conversion in its own sprite directory; [the web runbook](../../../docs/pokedex-web.md#eleven-family-replacement) links those assets. Final merged v6 production verification and web integration passed; publication and all 1,926 public-byte checks passed.

The [native refinement of the eleven new families](expansion/20261027-native-refinement.md) reviews all 46 views and applies 200 guarded face/eye pixels plus five battle-facing corrections. Original source art, palettes and previous recipes are preserved. These accepted native assets are included in the published sprite release.

[Flarkindle’s flower mythical redesign](sprites/mn-f116/redesign-v4/DESIGN.md) adds an exposed pale face, a three-petal lavender hood and an asymmetric leaf gesture, with matching close rear art. Its previous sprites and both new source candidates remain saved. This art is included in the published sprite release.

[Mudmender’s adult sprite redesign](sprites/mn-f140/mudmender-redesign-v2/DESIGN.md) exposes its broad head and feet, shortens the three-pair gill fans and opens the curled tail into a side cradle. Gillpup’s native art is preserved exactly. Accepted exports and both new source alternatives remain saved; these assets are included in the published sprite release.

[Dozebear and Denumbra differentiation](sprites/mn-f136/differentiation-v2/DESIGN.md) replaces their shared seated outline with a low sleepy quadruped and a tall upright den guardian. Densy and all original versions are preserved. The accepted art is included in the published sprite release.

The sections below retain the earlier expansion and tooling history. Current gameplay compiler inputs are `roster.json`, `roster-curation.json`, `pokedex-order.json`, `species-profiles.json`, `palettes.json` and the registered sprite manifests. Historical assembly/checkpoint scripts still assume the older dense 266-species milestone; do not use them to overwrite the current hand-authored profiles.

The website's **Show cut proposals** switch includes the retired species named in `roster-curation.json`, after the active regional index. Cut cards and detail pages are labeled and retain their sprites, evolutions, stats, moves and removal reasons. The switch combines with the other filters and evolution-line view; `?cut=1` restores it from a shared link. Active numbering stays unchanged. The JSON download keeps active records in `species` and adds the archived proposals in `cut_species`.

The previous 150 names and families are preserved. Lutebrisk is pure Ice following the user correction; its butter heart is a visual food detail. Their expansion contains **40 prairie species and 10 Iron Range species**, including the magnetic taconite wyrm Taconib → Pelletank → Taconarch. The three new slots add **Wrigglet → Pupump → Mosquitan**, a giant-mosquito pseudo-legendary family from the peatlands.

[roster.json](roster.json) is the canonical authoring source. [ROSTER.md](ROSTER.md) presents all names, types, evolution lines and themes. [COVERAGE.md](COVERAGE.md) reports type presence and independent ordinary families.

All seventeen Crystal types appear in the **active 220-species roster**, counting both types of dual-type species. Coverage ranges from **11 Dragon species to 42 Water species**; Ground and Flying each have **17 species**, and Psychic has **19 species**. Each type has at least four independent ordinary families. [COVERAGE.md](COVERAGE.md) distinguishes these curated counts from the older uncurated source checkpoint and its 17–44 guard. Complete authored moves are compiled separately; encounter availability and competitive balance need progression testing.

The [219-species balance pass](BALANCE.md) checks stats, natural move access,
default moves and evolution timing against the level-48–53 League. Bugs retain
their stronger stats and early evolutions. The modern physical/special split is
required by Northwoods builds. Run the Rust `audit_pokemon_balance` command after
changing profiles or imported/signature learners; `--check-pack` also detects a
stale compiled pack. Regional encounter availability and earned campaign pacing
still need integrated playtesting.

Lady-slipper dancer, haunted axe, pancake stack, voyageur beaver, Steel/Water fishing lure, butter-filled pure Ice Lutebrisk, blue-ox legendary and Devil's Kettle legendary remain in the roster. Legendary locations are proposals, not registered map changes.

[sprite-generation.json](sprite-generation.json) indexes persistent per-family records. Each records the exact ImageGen prompt, explicit per-Pokémon RGB5/RGB555 palettes, source art and reviewed crops. Work proceeds one evolution family at a time, with independently drawn front and rear views. The [progress list](SPRITE-PROGRESS.md) covers every species; the generated local gallery lives at `target/minnesota-pokemon/gallery/index.html`. Earlier image-generation probes are not accepted sprites for this catalog.

[POKEDEX.md](POKEDEX.md) presents full entries, stats, level moves, TM/HM/tutor compatibility and egg moves. [species-profiles.json](species-profiles.json) is the complete Rust compiler input. Original prose and family roles remain editable in [dex-authoring.txt](dex-authoring.txt) and [profile-authoring.txt](profile-authoring.txt); [movesets.json](movesets.json) contains the authored per-species learnsets.

Dex prose follows Crystal's concise field-guide voice: one distinctive habit,
ability or warning, stated plainly and developed across a single short entry.
Each entry still occupies two native pages of three lines, with at most 18
ASCII characters per line. Those pages continue the same prose; the readable
guide and website join them into one paragraph. Names, stats, moves and artwork
are authored separately from the entry text.

The [Rust pack builder and runbook](../../../modpacks/minnesota-pokemon/README.md) compile the aggregate species extension and complete family packs from the supplied base. Generated packs remain ignored. New Minnesota overworld encounters and the fossil-revival service still need authored map integration. Existing audio, maps and original species are preserved.

The following commands record the historical 266-species assembly workflow. Do not run them against the current hand-authored catalog; they can overwrite newer identities, profiles and milestones:

```sh
python3 -S planning/minnesota/tools/build_minnesota_pokedex.py
python3 -S planning/minnesota/tools/review_minnesota_pokemon.py --write-review
```

The Pokédex assembler balances each of the 532 pages into exactly three 18-cell
lines, merges the 266 explicit move lists, and writes the complete compiler
profiles and readable guide. Its intermediate metadata and optional vocabulary
index stay ignored under `target/minnesota-pokemon/`. The Rust compiler always
resolves every move, item, icon, cry and footprint against the selected base.

The new fish families are Winkfin → Glaciris → Tipulord (Water/Ice, living
ice-fishing tip-up), Rillbow → Torrout (Water/Fighting, current recoil),
Keypout → Latchusk (Water/Ghost, key barbel and shutter fins), and
Pikeling → Pikeman → Musketunge (Water → Water/Fighting → Water/Steel,
spear jaw to compressed-water muzzle). Murkib → Barbastion, Whirlnut →
Samurrel and Hushiss → Castanake complete the sixteen additions.

The earlier complete-sprite checkpoint used this workflow; its Python coverage step is historical and must not regenerate the current catalog:

```sh
CARGO_TARGET_DIR=target/minnesota-pokemon cargo run --locked -p crystal-assets --example minnesota_sprite_sheet
CARGO_TARGET_DIR=target/minnesota-pokemon cargo test --locked -p crystal-assets --test minnesota_sprite_sheet
python3 -S planning/minnesota/tools/review_minnesota_pokemon.py --sprites
```

For a new family, use the converter with `-- --manifest sprites/mn-fNNN/manifest.json` before publication. After inspecting the actual native preview, register the manifest with `python3 -S planning/minnesota/tools/review_minnesota_pokemon.py --register-manifest sprites/mn-fNNN/manifest.json`, then record the inspection with `--review-family MN_FNNN --review-note "Concrete observations"`. Registration adds its individual palette plans; it does not replace the visual inspection.

Use `-- --family MN_F001` with the converter to rebuild a single family. Native fronts are 56×56 / 784 bytes; native backs are 48×48 / 576 bytes. Generated PNG, palette-index JSON and 2bpp outputs remain ignored under `target/minnesota-pokemon/native/<family>/`. They are not game content packs and must not be committed.

The independent sprite verifier compares every native PNG pixel with its indexed JSON and tile bitplanes, checks all four RGB555 colors, and refreshes the gallery. `--require-all-sprites` additionally fails until all 266 species have both views and a recorded visual review. Successful conversion alone never marks a visual review complete.

Every native indexed export records the source PNG SHA-256, crop and fit. The verifier rejects stale exports after source/settings changes and compares the contact sheet with every individual frame. Visual review is tied to the exact review-sheet SHA-256, so changed pixels require a fresh inspection. Review observations are recorded with `--review-family` and `--review-note`; this is an explicit inspection step, never inferred from successful conversion. Negative checks confirmed that changed fit settings and a stale review image cannot keep completion credit.

Palette correction: each individual Pokémon has its own four-color palette. Only that species' front and back share it. The original153 corrected palette plans are individual; new species receive individual plans before generation. Earlier shared-palette sheets remain saved as superseded sources and receive no completion credit.

Gen 2 style references are rendered from the existing ignored local Crystal pack with `cargo run --locked -p crystal-assets --example minnesota_gen2_reference` into ignored `target/minnesota-pokemon/gen2-reference/`. The pack stays untouched. Reference comparison also established the native tile layouts: front tiles use row order; back tiles use column order. Back portraits can meet the lower canvas edge, as in Crystal, while retaining top and side margins. At the earlier 123-family checkpoint, all families had close rear portraits and per-Pokémon palettes; the earlier shared-palette/full-body-back candidates remain saved as superseded versions.

Every returned ImageGen render is retained at full resolution in its family folder, including alternates, rejected designs and images that fail conversion. Exact prompts and reference mappings remain alongside the source. Conversion now preserves immutable SHA-256 versions under each native family's `versions/` directory before updating a latest filename, including previous exports, source sheets, source-resolution crops, manifests, prompt/reference copies, palette/index JSON, 2bpp, native PNGs and previews. Indexed exports link directly to their immutable source and crop copies. A failed later conversion never removes the saved input or earlier output.

Run `python3 -S planning/minnesota/tools/archive_minnesota_renders.py` to snapshot all current authoring files, references and exports into the ignored content-addressed archive at `planning/minnesota/renders/creature-archive/`. It also copies every historical object and inventory from the older `target/minnesota-pokemon/archive/` so build-cache cleanup cannot remove the only saved copy. Each inventory snapshot is immutable; `latest.json` is a convenience pointer. These reusable archives are deliberate saved assets and must not be treated as disposable verification scratch. Generated archives and exports remain ignored and are never staged. The verified 09 snapshot retains 61,656 files in [immutable inventory `f2cb53850e6282285ee279a07e9638f4b0871fe2ac63466feb6eb4c039155696`](../renders/creature-archive/inventory-f2cb53850e6282285ee279a07e9638f4b0871fe2ac63466feb6eb4c039155696.json), including all new family sources, prompts, rejects, native alternatives, v6 production captures and live browser proofs.

The converter can map a preserved source palette to that Pokémon's current normal palette without resampling its ink indices into different hues. `source_palette_rgb555` records this explicitly; exported provenance records both palettes. This is used for the approved earlier fronts. Their later-stage back crops were too small on visual inspection; separately generated ImageGen close back portraits replace those crops while preserving the original candidates.

Small source eye glints can use a per-view `white_detail_min_samples` override (default 16 of 64 area samples). Overrides and all tested native alternatives are preserved in the per-view provenance. Native fit adjustments are also retained in provenance. The review gallery links directly to each original sheet, authoring manifest, native PNG, 2bpp tile file and indexed provenance record. All gallery links are checked against saved files.

The `source-contour-v1` conversion uses 16×16 area sampling per destination pixel. Interior black requires a majority vote; thin exterior rims use separate evidence from actual near-black source pixels and a source foreground mask. This reduces black expansion into colored limbs while preserving white wings and the open lower edge of rear portraits. Outline revisions invalidate earlier visual reviews: completion requires the current profile plus a fresh review of the exact sheet hash. Flourgeist also uses a targeted ImageGen source alternate to strengthen its cream ribbon contours; its original sheet remains saved.

The completed250-species artwork catalog follows the [expansion plan](expansion/PLAN.md): preserve the existing153 and add97 research-backed designs. The [fossil choice](fossil-choice.json) offers LENS FOSSIL → Trilume → Panoplite (Rock/Bug) and SPIRE FOSSIL → Siphlet → Obelurk (Rock/Dark) within those97 additions. Both now evolve at level34 following the campaign balance pass. Distinct two-stage body growth accompanies a proposed Driftless Trail choice, Rochester revival after badge1 and post-League access to the other line. These new gameplay services remain authoring proposals. New artwork receives completion credit only after native conversion and explicit visual review.

An optional per-view `interior_black_min_samples` setting (default32 of64) preserves thin structural seams when required, independently of the exterior contour pass. Changing it invalidates stale exported provenance. The original153 use the unchanged default. The two fossil lines use8 to retain their shell divisions; their current native candidates have explicit root visual reviews.
