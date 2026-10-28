# Substantial winter sections — detached source review

This replaces the user-rejected tiny-patch scope. Root published the bounded native overlay, but production-controller
acceptance and native GPU review remain pending. Canonical route promotion has
not happened. The 681-map parent is
`content-packs/minnesota-cohesion-harbor-water-v01.crystalpack`, file SHA256
`3d1cbdd18b05ce10ed66db7ab56d41f0032f6e045a0e72ad753064c5dc6ec8df`.
Canonical routes/world/interiors remain untouched by this author. Root owns
pack publication and shared registration; the focused Rust installer and tests
were authored here and executed only by Root.

## Geography

R17 now reads as a continuous cold cedar forest from the northern entrance to
the lower marsh's southern bank. Snow-covered native pines and ordinary snow
Floor surround the original roost and marsh encounter habitats, which retain
their exact TALL_GRASS quadrants. The actual upper marsh freezes into a broad
sliding surface with snow-crusted stops; the lower lake remains open. The
narrow north-eastern woodland lane has linked icy runs and dry turning bays at
the original fern branch and sign. No new actor, reward, room or quest is added.

The timber bridge and all deck/rail quadrants remain exact. Even an intentionally
conservative graph allowing free turns anywhere on ICE cannot bypass that
crossing before Cut. The original Cut shortcut remains 66 steps closed versus
6 opened, for both Foot and Foot+Surf over-approximations. Every old actor,
reward, sign and room approach has the same prior Foot accessibility; freezing
the upper marsh does not unlock an old Surf-only reward/islet. The lower sedge
neck remains required. These are static source facts, not executed controller
or earned-access evidence.

R11 becomes a continuous upper frosted gorge landscape, tapering at the actual
second stream. Snowy forest and icy versions of every old northern grass patch
frame the original cliffs. The long dry upper-bank shelf and shoreward hollow
use native sliding ICE with deliberate stops and turns. Every tributary Water,
animated Waterfall, plunge pool, bridge rail and Lake Superior cell stays exact.
The installed northern cohesion opening at meta20,0 is retained as native01,
although the current canonical layout still has05 there. Do not copy the frozen
old dossier over live native programs or reinstall its interiors.

The separate Rainy River Icefields branch west of R20 belongs to another author.
These sections do not change R20, its reserved west port, Boundary Waters Safari,
the Mississippi, or the explicitly planned International Falls town.

## Review history and current limitations

`source-v01` retains the first whole-section composition. `source-v02` refines
snowlines and gives the old fern branch a readable snow stopping bay. Frozen
`source-v03` adds exact native mixed sign/Headbutt compositions and explicit
geographic snowlines; its integration contract SHA256 is
`c901add54e8ec698b72f563bcd2fa93ef0383e95bd399db8f6bb70f048032735`. A native
wall stop on ICE is legal; a dry bay here is a design choice, not an engine
requirement. The source model also records intentional wall stops and legal
ordinary turns to dry ground.

The frozen mixed-block sign/Headbutt recipes remove the isolated green ground
squares. Original Cut0f deliberately remains unrecolored; no Cut replacement
semantics were changed for appearance.
The large original yards/terrace shapes are retained; this seasonal pass is not
an all-route geometry or Johto-quality completion claim.

All complete originals, crops, previews, rejected stages and input snapshots are
retained under `../../../renders/ice/sections-v01/`. Whole/camera review:

- [R17 whole](../../../renders/ice/sections-v01/review-v03/r17/whole-review-fit.png)
- [R11 whole](../../../renders/ice/sections-v01/review-v03/r11/whole-review-fit.png)
- [17 before/after camera pairs](../../../renders/ice/sections-v01/review-v03/decision-before-after-sheet.png)
- [Conservative source proof](../../../renders/ice/sections-v01/proof-v07/proof.json)

The second complete review took 16.469 seconds after art load, including archive
writes and all 34 camera images. It reuses the existing native player/priority
renderer on exact small metatile windows; both entire route scenes remain
archived first. This is an observed run, not a guaranteed timing.

Native integration must carry original Cut/Headbutt replacement behavior, water
animation, encounter tables, trainer parties/programs, aliases' grass rustle,
NPC OAM priority and all room/warp/recovery/story metadata. Production input
must prove sliding, wall-stop turns, route/grass/resource journeys and
Save/Continue. None is claimed from these reconstructed source images.


## Actual native publication and independent readback

Root's completed file is
`content-packs/minnesota-northern-winter-v01.crystalpack`, SHA256
`aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a`,
content identity
`ab1c60993ffc1bb0d4c3ef4c02f2455208cadc939faee3ffa5ce310ebbb61805`.
The chosen preceding mixed-art/new-region parent is
`content-packs/minnesota-winter-mixed-art-v01.crystalpack`, SHA256
`2b2887fd056d507d85ac5d2b8861e92559d8679cf11902f63c42f270d617842d`.
Both contain683 maps; the two new Rainy River maps were installed by their
separate builder before these sections. This installer adds no map.

The [independent written proof](../../../renders/ice/sections-native/written-aa1aa251-v02/proof-8fe3ec8efbac7c371c8f50607fcf103d31b99a79746e570f2e0a1cd93d46351d.json)
passes821 checks and10 intentional corruptions. All679 unrelated whole native
modules, all raw/typed gameplay/catalogs, encounters, original actors/rooms,
recovery, aliases, runtime assets and audio metadata remain exact. Only4 old
chunks' declared2078 blocks and tileset/attribute/block mirrors change. The
wire parser's inherited old representation is compared independently before
and after; it is not silently repaired to match a cache. No raw program dump
or generated pack is included in the source deliverables.

Root's assets run observed16/17 cases passing; the remaining failure was a
negative fixture incorrectly expecting embedded `compiled_audio` in a modular
pack. The fixture now corrupts an actual music descriptor's PCM frame count;
its rerun is pending. The strict residual continues to compare both embedded
audio (even empty), all audio descriptors and all runtime files. The first
failed independent consumer epoch is retained for the same corrected assumption.

The native controller leaf `bevy_shell/tests/winter_sections.rs` has5 pending
cases: all82 ICE entry/return courses and16 wall-stop turns; completed original
ULTRA BALL Pocket pages/once/Continue; native Cut denial and66→6 walking shortcut
with vegetation reload; and retained Surf/Waterfall qualification/current plus
the real upper tributary bridge. Explicit initial fixtures are not earned
regional progression. No passing controller or GPU result is claimed here.

## Reproduce source review and Root publication

From the repository root, use a new output folder for every review:

```sh
PYTHONDONTWRITEBYTECODE=1 .local/minnesota-review-env/bin/python \
  planning/minnesota/regions/ice/sections/review_sections.py \
  planning/minnesota/regions/ice/sections/source-v03 \
  planning/minnesota/renders/ice/sections-v01/NEW-REVIEW
PYTHONDONTWRITEBYTECODE=1 .local/minnesota-review-env/bin/python \
  planning/minnesota/regions/ice/sections/check_sections.py \
  planning/minnesota/regions/ice/sections/source-v03 \
  planning/minnesota/renders/ice/sections-v01/NEW-PROOF
```

These tools reuse the existing native SourceArt/player/priority/archive helpers.
The first whole review and every rejected pass remain retained. The original
winter spec/helper stay frozen; the corrected source224 helper and separate
mixed-edge append are explicitly hashed in each final review input epoch.

Root owns the existing `build_winter_sections` native CLI. Its exact interface
is `PARENT CONTRACT EXPECTED_PARENT_SHA256 NEW_OUTPUT`. A future rerun must use
a fresh output path, unchanged frozen contract/source hashes and the completed
art→region→mixed parent; the borrowed input pack is never edited. No native
export or competing Cargo job was run by this author.
