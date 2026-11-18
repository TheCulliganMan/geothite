# Route 7–11 hidden discoveries

Five existing outdoor caches gain local native-pixel clues. Rewards, positions,
collection flags, scripts, NPCs, route connections and collision stay intact.
This leaf is for the next combined release; it does not select or deploy a pack.

| Route / map | Search square | Stand / face | Reward | Clue |
| --- | --- | --- | --- | --- |
| 7 / MnR07C00R01 | 24,25 | 24,26 / north | Mystic Water | A pale glint in the water-worn root beside the lake |
| 8 / MnR08C00R01 | 35,42 | 36,42 / west | Miracle Seed | Fallen cones in a thawed hollow beneath the pines |
| 9 / MnR09C01R00 | 31,18 | 32,18 / west | Rare Candy | A pale seam through the trail-side stone |
| 10 / MnR10C01R00 | 22,15 | 23,15 / west | Super Potion | Discarded field dressing and boot scuffs in the forest turnout |
| 11 / MnR11C00R02 | 51,19 | 50,19 / east | Ultra Ball | Slack rope leading across the step to the existing mooring ring |

Coordinates use native 16-pixel walking squares. The five `MN_HIDDEN_Rnn`
scripts retain the five corresponding `EVENT_MN_HIDDEN_Rnn` flags. No extra
item or fresh flag grants a second reward to players who already searched here.
Route 11's previously authored mooring ring is preserved and extended, not replaced.

## Survey and comparison

The selected Crossings parent is
`content-packs/minnesota-crossings-v01.crystalpack`, content identity
`5ad096a303e84c34353f4ac61cca7e52184001f8df71c9a9de9f0cabfa115aff`.
The survey found one native `BGEVENT_ITEM` outdoor cache on each of these five
routes. Route 7 used an ordinary tree root, Routes 8 and 10 ordinary flowers,
Route 9 an ordinary boulder and Route 11 a custom shore mooring ring. These
are useful existing side pockets; their rewards do not need inflation.

[Crystal's Itemfinder lesson](https://github.com/pret/pokecrystal/blob/master/maps/EcruteakItemfinderHouse.asm)
teaches that detection is nearby rather than an exact marker, then suggests
Burned Tower as a place to practice.
[Crystal Route 32](https://github.com/pret/pokecrystal/blob/master/maps/Route32.asm)
combines ordinary visible pickups with hidden items. The design inference here
is to make searching a second way to read the landscape, not require a label on
every collectible.

The existing local Polished Crystal clone, revision
`f8e67fbd53ad7a99870d8ab87fd73612e6c6a679`, retains that lesson and adds more
[Route 32 hidden opportunities](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/maps/Route32.asm):
five hidden rewards, a paired tree-grotto interaction and two visible item balls.
Its repeated opportunities invite players to reuse the search habit. Minnesota
uses five original environmental marks rather than copying that layout or art.
Visual review also used the retained native Crystal Route 32 shore camera and
Polished Crystal’s [ice-coated boulder](https://github.com/Rangi42/polishedcrystal/blob/f8e67fbd53ad7a99870d8ab87fd73612e6c6a679/screenshots/ice-rock.png):
a distinctive surface can make a small object worth examining without a floating
marker. The Minnesota seam and root glint use this principle at a smaller scale.
All new stamps are 32×32 indexed native art, with dots preserving source pixels.
Each stamp appends a metatile and uses checked unoccupied glyphs. Four-shade
native palettes and the exact original four collision quadrants are retained.

## Integration and checks

The integration owner registers `minnesota_hidden_route_clues` in
`pack_authoring.rs` and calls
`install_minnesota_hidden_route_clues(&mut candidate)` before finalizing the
combined identity. The installer rejects changed preimages or occupied art
slots. Do not apply twice. The five JSON art files and `caches.json` are the
owned authoring sources; central world registration belongs to the release owner.

The targeted ignored test uses the actual supplied pack and native core
functions. It checks the interaction target from the specified standing square,
Itemfinder identity before collection, exactly one item and the original flag,
no Itemfinder result after collection, and no duplicate on a second collection.
It also compares every untouched map, all non-art fields of edited maps and the
audio manifest. Static walking checks honor native collision, ledges and NPC
occupancy, checking both directions between the approach and a route seam.
These are data/core checks, not a playthrough or proof of earned progression.

```sh
MN_CLUES_PARENT="$PWD/content-packs/minnesota-crossings-v01.crystalpack" \
MN_CLUES_OUTPUT="$PWD/target/route-07-11-clues-v01/review-v02.crystalpack" \
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p crystal-assets \
  --lib route_clues_preserve_rewards_flags_and_native_itemfinder -- --ignored
.local/minnesota-review-env/bin/python \
  planning/minnesota/discoveries/route-07-11-clues-v01/render_review.py \
  content-packs/minnesota-crossings-v01.crystalpack \
  target/route-07-11-clues-v01/review-v02.crystalpack \
  target/route-07-11-clues-v01/review-v02
```

Choose a fresh ignored output for each candidate. The renderer reads the actual
packs and retains whole-route native terrain, equal-position camera crops and
an immutable render history. It never exports a pack or copies a source catalog.
No game inputs, public deployment or user saves are involved.

Verification status (2026-11-18): **v02 accepted for integration**. The corrected
candidate passed the actual named test (1 passed, 0 failed, 1303 filtered),
including all five native interaction queries, reward/flag checks, one-time
collection, Itemfinder before/after, static two-way access and pack write/readback.
The integration owner used the preserved current-source executable at
`target/northstar-next-combined/field-tests`; results are retained in
`target/field-review-20261118/clues.log`. Earlier zero-test shared-cache runs
were rejected as evidence and produced no candidate.

`cargo fmt --all -- --check` passed. The all-target `crystal-assets` Clippy
command exited successfully with repository warnings outside this leaf; the
integration owner also owns the final current-source combined checks.

The native verifier also passed:

```sh
target/debug/verify_crystalpack \
  target/route-07-11-clues-v01/review-v02.crystalpack --audio --desktop
```

It verified 743 maps and all 602 audio programs, including PCM hashes, frame
counts and loop ranges. No game was played and no public deployment occurred.

Visual review covered all five equal-position native camera pairs and the whole
routes. The first version's brown palette overrides made rectangular patches
around R08/R10; that version was rejected and retained. The accepted v02 uses
its surrounding terrain palettes. R09's pale stone seam is the best comparison:
`target/route-07-11-clues-v01/review-v02/best-clue-before-after.png`.
All five pairs are in `before-after.png` in the same directory, and complete
route terrain in `whole-routes.png` plus individual full-resolution originals.
These are native source-art terrain renders of actual verified packs, not
screenshots of a played session.

Scratch helpers were removed. The shared build cache remains in use by the
combined release; it was not cleaned or duplicated. The final disk check left
373 GiB available. Packs, images and verification logs remain ignored.
