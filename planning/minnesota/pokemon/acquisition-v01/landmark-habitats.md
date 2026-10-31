# Harvest oats and North Shore smokehouse

The authored source is `landmark-habitats.json`. This stage adds Oatlet at
level 8 along the eastern Route 2 harvest fields and Cinderfin at level 40 in
all four grass beds behind and beside the North Shore Smokehouse. Both occur
morning, day and night. The existing Wickit lighthouse heath remains intact.

Each habitat keeps the original route's common slots 0, 1 and 4, rates and
levels. The named resident uses slots 2, 3, 5 and 6, totaling 35 percent of
encounters within its grass beds. Other grass, fishing, Surf and all unrelated
content remain exact to the parent. No terrain change was necessary: the
harvest margins and the smokehouse already have usable native encounter grass.

Reference review used the supplied pack's Route 38/39 table structure and
native Route 39/Olivine maps: common local residents, a distinctive local
species, and separate day/night tables. The Minnesota additions use only the
regional roster. Saved reference and current-site renders remain ignored under
`planning/minnesota/renders/landmark-habitats-v01`.

## Local publication

Executed build and pack publication:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_INCREMENTAL=0 \
RUST_MIN_STACK=67108864 cargo build --locked -p crystal-assets \
  --bin pack_landmark_habitats --example publish_pokedex_locations
RUST_MIN_STACK=67108864 target/verdigrim-scene/debug/pack_landmark_habitats \
  content-packs/minnesota-glacier-current-v03.crystalpack \
  planning/minnesota/pokemon/acquisition-v01/landmark-habitats.json \
  content-packs/minnesota-landmark-habitats-v02.crystalpack
```

The immutable child has content identity
`6b28ef1aab2c5e843d6321d4f2790f4f1b578a442ce9406d1a9a85dd657462fc`.
The publisher verifies compact geography, installed native grass, nonoverlap, active regional Dex
membership, the seven-slot level range and written pack readback. Its external
regression rejects duplicate overlays, nonregional species and non-grass sites.
Independent full-parent comparison preserves all 734 maps, original encounter
tables and zones, fishing, scripts, actors, art and audio. All 377 audio programs
pass PCM hash, frame-count and loop-range checks.

Two current-pack controller cases pass legal path entry, natural encounters,
Master Ball consumption, caught Dex registration and Save/Continue. Fourteen
production stdio calls verify the caught saves, real Start menus and subsequent
movement with empty stderr. All 60 grass cells have a conservative dry path
from the existing approaches. Native 2D screenshots were inspected. A fresh
guarded Rust replay matches the selected pack byte for byte.

The selected v02 retains the five-stop train service, including all station
stories and terrain. See `world.json` for the current local selection.
These are declared fixture cases; they do not prove earned statewide access.

## Website workflow

The Rust `publish_pokedex_locations` example adds actual location panels and updates
the existing Pokédex JSON download in a preserved reviewed website release. It preserves all prior
species facts, cries, sprites, walking thumbnails and move previews. Location
search uses the installed habitat names. Entries include times, levels, rods,
Headbutt, conditional gifts/discoveries and incoming level evolutions. A species
without a source or an acquisition path through evolution is labeled pending.
Location-source identity is recorded separately from the art/audio source pack.

Use a fresh release directory for every export, then verify desktop/mobile
pages and deploy with `tools/deploy-pokedex.sh`. Existing releases are retained.
The website's habitat prose describes the natural setting; the separate
Where to find panel describes installed local-map sources.

The guide contains 225 active species pages, 216 with a table/program/evolution
source and nine explicitly pending. These are installed-source records, not a
claim that every conditional route or evolution has been earned in one run.
The base-level caption explains the inherited wild scaling. Smokehouse beds
are grouped into one readable location; their exact map/zone records remain
in the JSON. Region-specific search terms are replaced on each update rather
than accumulating stale locations. HTML page directories retain their existing
contents; the added data belongs to the established catalog download.

## Remaining statewide work

This stage does not complete the statewide goal. Its later selected child moves
restoration inside the signed [Rochester Fossil Lab](../../landmarks/fossil-lab-v01/README.md).
The later [water/cave phase](water-caves-v01/README.md) fills 19 ordinary Surf
areas and ten cave floors, preserves deliberate peaceful/Gym/Safari exceptions,
and fixes the 26 inherited fishing headers described in the original audit.
It retains both harvest/smokehouse beds, the indoor lab and Leech Lake island.

Existing grass and rod ecology still require regional review. Route 2's isolated
water edges remain inaccessible; nine rare species still lack acquisition
sources, and Verdigrim remains absent from the compiled regional roster and
completion threshold. Separate story review packs require deliberate
integration before their acquisition routes can be called active.

Scoped formatting passes. Assets/Bevy all-target Clippy passes with existing
warnings. The full workspace format check still reports unrelated differences
in offset-allocator, pack-core and web-server files.
