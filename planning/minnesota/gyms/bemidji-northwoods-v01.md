# Bemidji northwoods garden — review candidate

The selected room repeated the same hedge in two full-width bands. This
candidate keeps its greenhouse edges and two offset native Cut trees while
using native conifers, flowers and an earthen path to give the enclosed garden
more of a northwoods identity. The gold gate remains closed until Orlis is
defeated, then opens a direct return. The trainer at the west side remains
optional; both required trainer battles and both original Cut crossings
remain meaningful. The existing Lake Itasca and Whirlpool directions remain
in the gym dialogue.

Review the full room and native entry in
`../renders/gyms/bemidji-northwoods-published-v02/`, the open-state source
render in `../renders/gyms/bemidji-northwoods-open-v01/`, and the explicit
victory-flag native capture in the published directory. The earlier study
renders are retained. Source terrain pixels and collision quadrants were
compared against the compiled initial room. The victory fixture is a visual
preview; it is not earned progression.

The Rust publisher extends the canoe-equipped six-gym review pack, checks
the installed actors, city warps, current Minnesota parties and native Cut
replacements, and retains the existing gym scripts while remapping Cut block
IDs and adding the return gate callback and two explanatory text pages. It
checks that the pruned
return is at least four walking tiles shorter after victory. The resulting
ignored review pack is `content-packs/minnesota-seven-gyms-review-v03.crystalpack`,
content identity `c5100712c38c7cca51495a9d86b5ff1b8aae6c5a060503bb88e222afb79febbc`.
Compared with its parent, only `MnBemidjiGym` changes among maps. Brindel's
retired Glisnare is replaced by active Grass/Dark Vergloom at the same level,
with legal level-up moves; all other trainer data and the species, item and
audio catalogs remain unchanged. Existing event numbers are unchanged; one
return-open event is added.

Four existing production-controller cases passed: Cut denial and cancel,
required trainer ordering, both trees and persistence, and battle loss and
sixth-badge denial. A literal Mew100/six-prior-badge fixture then earned both
trainer wins and the leader win, pruned both trees, saved and resumed, walked
the 18-step victory return, and used both city exits. A separate controller
run entered from the actual city facade, obtained the guide's HM, used Cut,
completed the battles and returned through both exits. These do not establish
ordinary-party balance or the earned regional journey. All 602 audio programs
passed PCM hash, frame-count and loop checks.

Build from the exact review parent with a fresh ignored output filename:

```sh
cargo run --locked -p crystal-assets --bin pack_bemidji_northwoods_garden -- \
  content-packs/minnesota-six-gyms-buoys-canoe-review-v01.crystalpack \
  planning/minnesota/gyms/bemidji-northwoods-v01.json \
  content-packs/minnesota-seven-gyms-review-v03.crystalpack
```

The world selection remains unchanged. In-room saves and block overrides need
migration review before selection. Duluth, the full regional journey, and
integration of the gym candidates remain unfinished.
