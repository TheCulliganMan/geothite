# Ely pine-knoll trail revision

For the current whole-town pine exterior, native tile contract and fresh
verification, see [Ely in the pines](PINES.md).

The original new approach was a straight 32-tile corridor with detached grass
bands. This revision makes a complete pine-knoll walk from the snowy Ely pass
to the new Center south of Split Rock. Ely remains in snowy pines, the coast
stays green, and every existing stream, waterfall and bridge cell is retained.
No new Cut gate, water access or dangling regional seam is introduced.

## Playable geometry

All coordinates below are 16-pixel core tiles in `MnR11C00R02`; source cells are
32-pixel metatiles. The western landing remains `(0,30)/(0,31)`, paired with
`MnElyShorePass(13,10)/(13,11)`. The clinic door is `(33,33)` with passive return
`(33,34)`. Both reciprocal Ely/pass seams and the original Ely north exit retain
the previous published coordinates and offsets.

The eastbound path bends north through pines, turns around a basalt knoll,
then bends south to the clinic forecourt. Required grass lies across the actual
path. Native source inspection finds a minimum of **14 grass landings** on the
walking approach. There is no continuous dry bypass.

The lower shelf provides a **36-input dry return** west. Its native `4c` block
at metatile `(11,17)` has `HOP_LEFT` on the eastern half and blocking collision
on the western half. The authored course jumps from `(23,35)` to `(21,35)`;
the reverse is blocked. Pines above and below the lip close the side bypass.
The ordinary uphill climb remains available after returning. Removing the
ledge exposes a forbidden dry eastbound route; blocking its landing removes
the dry westbound route. Both negative cases are checked by the native audit.
The ordinary eastbound course takes 43 inputs, including the grass climb.

This is a short connecting walk with the existing guide and clinic service.
It does not add a trainer quota or turn the coast into another town. The
original R11 trainers/items remain at their original positions.

## Current candidate and regional ecology

The ignored candidate is `content-packs/minnesota-ely-pine-knolls-v02.crystalpack`:
691 maps, SHA256 `1f5604a6c743904e5b0c8fbfe36b32ef056f50dab309a42795aa639457ac0bf3`,
identity `1d3d63099937c03ad2c390ef2910ddfe23de232321cfdcb226aa26e250df6347`.
The strict Rust pipeline imported 224 currently active Minnesota species and
bound Minnesota appearances to that curated roster, retaining the original
251 species for the original regions. There are 475 installed species and
270 moves in this candidate. Regional fish tables are separate from the
original Johto/Kanto groups. Existing species profiles, art and rule overlays
were consumed through their existing Rust builders.

The regional binding applies to Minnesota teams, encounters, visible creature
actors and species dialogue across this child pack, rather than leaving the
old copied Johto encounter table in the new grass. It leaves the geometry,
events, object positions and original runtime art/audio exact to the geometry
stage. This migration does not establish earned campaign balance or final
visual approval. The old 691-map v01 candidate and its saves remain intact;
saves from that identity are not migrated into this candidate.

## Fast source review

With Pillow and cbor2 available:

```sh
export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/revisions/ely-trail-quality-v02/author.py --epoch NEW_EPOCH
```

Choose a fresh epoch. `author.py` reads the immutable compact-coast parent and
reviewed snow alias, writes explicit patch preimages and input courses, and
renders the complete Ely, pass, shore chunk and connected context. It never
writes a pack. Source-v01 took 4.81 seconds; source-v02 took 11.70 seconds under
concurrent compilation. The second pass improves path tie-breaking; source-v03 corrects the northward
input label. Neither changes the terrain or reviewed contract hash. All originals, snapshots,
previews and rejected passes remain saved.

The authoritative contract is `source-v03/native.contract.json`, SHA256
`3dde583414e13f81ab84d0b46baff667ac6581e0322bd3db0c63625f6ed57130`.
The parent is `content-packs/minnesota-superior-compact-v03.crystalpack` with
SHA256 `032304d0fd1d51863a2c3bf2562b20728e22c7535d1c194555621bc8e85b4899`.
See [PUBLICATION.md](PUBLICATION.md) for the executed native pipeline.

## Evidence

- [Whole connected exterior](../../renders/ely-trail-quality-v02/native-v03/connected-whole.png)
- [Whole shore chunk](../../renders/ely-trail-quality-v02/native-v03/MnR11C00R02-whole.png)
- [Pine-knoll trail detail](../../renders/ely-trail-quality-v02/native-v03/pine-knoll-trail.png)
- [Native publication and roster audit](../../renders/ely-trail-quality-v02/native-v03/receipt.json)
- [Pack-bound controller input plan](../../renders/ely-trail-quality-v02/native-v03/input-plan.json)
- [Full connected exterior atlas](../../renders/ely-trail-quality-v02/state-continuity-v01/full-state-native-current.png)
- [Remaining quality scope](REVIEW.md)

The static/native audit passes for all authored terrain, retained water, map
positions/events, original runtime/audio bytes and the complete regional
species closure. All 377 audio PCM/hash/frame/loop checks pass. The source, whole-map native art and reached headless captures were inspected.
Production stdio MCP passes the clinic, Start menu, all 36 dry-return inputs,
actual ledge, blocked reverse and both pass seams back into Ely. Both focused controller tests pass: complete climb/ledge/reverse, native
HP/PP/status care, Save/Continue and an actual regional encounter/Run. Both
seam lanes and Ely's original north exit also pass. The native stdio grass
replay observes Bandrake. See PUBLICATION.md for the explicit fixture and
executed commands; these checks do not establish full campaign completion.

The actual current exterior atlas has 97 maps in one direct connection group,
zero inconsistent placement cycles and zero footprint overlaps. Its explicit
warp/seam graph reaches all 97 from Winona; this is not a Surf or prerequisite
playthrough. The boundary inventory lists 98 water-to-nonwater edge samples:
most meet a connected wall or bank, and are not automatically defects. Eight
are unbound water cells at the north/south edges of `MnR02C00R00`. The later
[actual compiled-channel audit](../../renders/minneapolis-waterfront-v01/native-v02/prairie-channel-qualification.json)
confirms that fences and bridge rails seal every land-to-water entry: zero
mounting pairs and zero reachable outer Surf cells. Three barrier-opening
mutations are detected; no extra rocks are needed in this inaccessible channel.
Detroit Lakes, Moorhead,
International Falls and Mille Lacs are absent as named hubs in this candidate.
These concrete gaps remain open rather than being concealed in the atlas.

Final reached views: [ledge lip](../../renders/ely-trail-quality-v02/native-reached-v02/ledge-lip.png), [landing](../../renders/ely-trail-quality-v02/native-reached-v02/ledge-landing.png), [snowy pine gate](../../renders/ely-trail-quality-v02/native-reached-v02/ely-pine-gate.png). Native receipt also confirms LUTEBRISK remains pure Ice.
