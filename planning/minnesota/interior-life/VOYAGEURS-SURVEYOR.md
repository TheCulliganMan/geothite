# A canoe guide on the lower ice landing

The last character-free local non-gym interior now has a winter surveyor.
He guides canoes at Voyageurs in summer and draws winter crossings for his
grandson, who thinks the arrows look like a dancing loon. His native GRAMPS
sprite stands on the existing dry landing at `(3,20)`, with approach `(4,20)`.
His family story gives the return landing a resident with a reason to stay.

The completed ignored child is
`content-packs/minnesota-interiors-voyageurs-surveyor-v01.crystalpack`:

- 729 maps; 230 registered non-gym interiors.
- File SHA-256 `21c5918b95f86aa43782065c0358fab1b80a8b0fa1a3106231095a366cdab7a9`.
- Content identity `0eebabd411a87d544c4ef8bfabd74bd63c6d7ebeee7a7a7400ed4f40f1a3a11b`.

Executed with the existing Rust publisher:

```sh
target/interior-life/wasm-dev/pack_minnesota_interiors \
  content-packs/minnesota-interiors-r03-current-story-v01.crystalpack \
  planning/minnesota/interior-life/voyageurs-surveyor-v01.json \
  content-packs/minnesota-interiors-voyageurs-surveyor-v01.crystalpack
```

Preserve the completed output and choose a fresh path for a replay. The first
attempt rejected incorrect contract field names before writing a pack; the
corrected source uses the publisher's `manifest` and
`all_installed_interiors` fields. Both logs remain retained.

## Verification

Only `MnVoyageursIceB1F` changes. Its terrain, attributes, events, original
objects and original programs remain exact. All other map modules, native art,
audio manifest, species, moves, encounters, items and trainer teams remain
unchanged. The changed data fields are `maps`, `map_scripts` and `npcs`.
The publisher's dry-floor traversal guard accepts the resident and preserves
the original landing's remaining walkable area.

Both production-controller cases pass on this exact child:

- The new conversation owns movement, advances page by page with A, repeats
  after Continue, includes the Voyageurs/grandson/loon story, and permits the
  original ladder return (10.80 seconds).
- The unchanged original ice-dungeon course still plays the forced slides,
  rejects perpendicular redirection, collects both lower caches and the upper
  supply, preserves rewards through Continue, returns outside and reenters the
  same cave mouth (20.56 seconds).

These are declared local empty-party fixtures; they do not earn regional
access or demonstrate an earned campaign. Both complete native 2D frames match
installed source pixels with zero exclusions and were personally inspected.
The guide is visible beside the landing, clear of the ice and ladder. All 377
audio programs pass PCM/hash/frame/loop verification again. Images, comparisons
and preservation evidence are retained under
`planning/minnesota/renders/interior-life/voyageurs-surveyor-v01/`.

All 230 local non-gym interiors now have character objects and explicit
authored purposes in the room registry/story sources. This is a local
composition milestone. The newer selected R09–R12 terrain, Mille Lacs park and
progression rewards are now integrated in the verified 730-map
[selected union](selected-union-v01/README.md). Remaining regional journeys,
the five-stop passenger service and wider acceptance remain open.
Red Wing has no station.
