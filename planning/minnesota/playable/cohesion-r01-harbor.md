# Connected state, R01 ridge and Grand Marais harbor

The completed local pack is
`content-packs/minnesota-cohesion-harbor-water-v01.crystalpack`, SHA256
`3d1cbdd18b05ce10ed66db7ab56d41f0032f6e045a0e72ad753064c5dc6ec8df`,
identity `174f1942c42bd590e292e5a95340ffb9d93758bf0ab1ec31d83dadfd9fc9c722`.
It retains681 maps and98 Minnesota exteriors in one direct-seam component,
with no footprint overlaps. This is scoped native progress, not acceptance of
every route's visual quality or proof of earned regional progression.

[Full connected state](../renders/outdoor-connections/native-cohesion-harbor-water-v01/full-state-native-current.png),
[Winona–St. Cloud river](../renders/outdoor-connections/native-cohesion-harbor-water-v01/winona-st-cloud-native-connected.png),
and the [Root receipt](../renders/outdoor-connections/native-cohesion-harbor-water-v01/root-review-v01/acceptance.json)
bind the actual written content and retained evidence.

The long wooded Mississippi reaches remain deliberately simple: five optional
bank trainers, small grassy landings and hand-authored river bends. The native
1,269-input journey passes all seven bridge apertures, Waterfall and Continue
under a bridge. Walking above the bridge also passes. Surf/Waterfall partners
and required badges are explicit test fixtures, not newly granted progression.

Grand Marais's20 southern metatiles now use original native buoy blocks. All40
outer lanes and both corner caps block Surf while the inner water remains open.
Testing uncovered an inherited grass-only encounter record: the separate repair
adds only its Water rate/table from the existing adjacent North Shore source.
Encounters remain enabled. The actual harbor test consumes one Max Repel through
the Pack menu, traverses the harbor, probes every reachable buoy, Continues on
water and dismounts on the original shore. [Actual gameplay-Save GPU views](../renders/outdoor-connections/native-cohesion-harbor-water-v01/harbor-gpu-v01/production-decisions.png)
show the boundary in native play.

R01's49-cell exterior revision shapes the Potion ridge and its cedar boundary.
The [13 actual exterior views](../renders/outdoor-connections/native-cohesion-r01-v01/r01-exterior-gpu-v01/production-decisions.png)
and [seven NITE cave views](../renders/outdoor-connections/native-cohesion-r01-v01/r01-cave-gpu-v01/production-decisions.png)
match7,372,800 source pixels exactly. The final Water repair leaves all681 map
modules byte/value exact, so it preserves those terrain comparisons. The native
cave's three-cell upper connection creates a26-step loop with6/18-step returns;
the guide, Antidote, stairs and true passive exit remain. Three cave cases pass,
including medicine consumption, once behavior and Continue. The exterior case
passes all four ledge lanes and their45/43/43/43-step ordinary returns, signs,
rewards, dock barriers and both rooms. Retained Found-text provenance is distinct
from directly observed, fully printed Pocket pages.

The accepted exterior source is promoted to `routes/MN_R01.layout.json` with
guarded before/after bytes in [revisions/r01-cedar-ridge-v01](../revisions/r01-cedar-ridge-v01/).
Newer concurrent cave authoring in `routes/MN_R01.json` is preserved. Its newer
rows are **not** claimed installed in this pack; the frozen tested cave candidate
is retained in the publication inputs. `interiors.json` and `world.json` were
not rewritten by this promotion.

## Reproduction

Use the existing Rust publishers, in order, with separate fresh ignored output
paths. The exact initial parent is `minnesota-state-cohesion-v01.crystalpack`,
SHA256 `c4816242295a236fdfe2a6a96fb248cd3eab11de1b526be0decb7df36e5e95eb`.

1. `revise_grand_marais_harbor BASE CONTRACT LAYOUT OUTPUT`, using the contract
   and candidate layout in `connections/grand-marais-harbor-closure-v01/`.
2. `revise_authored_route BASE BEFORE AFTER DOSSIER OUTPUT`, using
   `revisions/r01-cedar-ridge-v01/publication-inputs/{before.layout.json,candidate.layout.json,native-dossier.json}`.
3. `revise_authored_cave BASE PREVIOUS_ROOM REVISED_ROOM OUTPUT`, using the two
   `MnR01Cave1F` JSON files in that same publication-inputs folder.
4. `repair_grand_marais_water BASE routes/MN_R11.json OUTPUT`.

Arguments above are resolved from the repository and planning directories as
appropriate. Executed commands and stage failures/successes remain in
`.local/{grand-marais-harbor,r01-cedar-ridge,r01-upper-loop,grand-marais-water}-publication-v01.log`.
Build binaries with the pinned toolchain, `cargo build --locked --profile wasm-dev
-p crystal-assets --bin NAME`, using a distinct `CARGO_TARGET_DIR` when another
owner is building. Generated packs and render outputs remain ignored.

Seven final controller cases and nine focused external asset cases pass.
Independent written audits pass696+886 checks and reject12+15 corruption cases.
All377 audio PCM hashes, frame counts and loop ranges pass on the final pack.
Only eight decorative R02 drainage edge cells remain unmatched by water seams;
they have no ordinary bank launch or incoming Surf access. Source plan checking
passes after the exterior-only promotion. Standard all-target Clippy is blocked
by pre-existing `multiplayer/mod.rs:1332` bad-bit-mask; lib/tests Clippy passes
with four documented baseline lint allowances. Focused Rust formatting passes;
workspace formatting still reports unrelated existing changes.

Ely/Fairgrounds integration, the new ice work and broader route/cave quality
remain separate unfinished work. Every original render, failed pass, raw GPU
readback, save and proof epoch remains retained.
