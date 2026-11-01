# Ely pine camp on the combined winter parent

The reviewed complete20×18 town is now carried forward onto the Itasca snowline
candidate. Three residents, fitted pine windbreaks and the northern Full Restore
loop are installed together with the later opening/story/fauna, coast passing
width, Bemidji roof/plateau, continuous inland winter, thaw banks and cliff/Cut
repairs. Original Ely doors, road lanes, scripts, League registration and indoor
services remain exact. This candidate is **unselected**.

[`contract-winter-v03.json`](contract-winter-v03.json) binds the actual parent
`content-packs/minnesota-itasca-snowline-v01.crystalpack`, SHA256
`d7139c4a1e4ada8aa9b389675b07aaf119bb670c2003fa21974ddf9ac6856c88`.
Its Ely module is byte-equivalent in canonical JSON to the original guarded
preimage, SHA256
`5bfe6d254ccbbb3447f7885191d85c59a64f076ef72ff4ddade255bd0b6df104`.
The original terrain, activities and older contracts remain untouched.

Child: `content-packs/minnesota-ely-pine-camp-winter-v03.crystalpack`, SHA256
`2d4f4e01216ee770f07bebd665a9d96a0badc2f346f802aceaa04f0143c409d6`,
content identity
`0c82ca57bbf206f2005922aa40d0fe3a59a62223d978f6f44e4f41a12c8252db`.
This stage is followed by the verified [Inner Basin bedrock revision](../../caves/inner-basin-bedrock-v01/README.md). Use that latest child for further integration. `world.json` continues to select
`north_shore_passing_width_revision`; no app/browser release changed.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 \
  cargo build --locked -p crystal-assets --bin rebuild_ely_pine_camp
target/verdigrim-scene/debug/rebuild_ely_pine_camp \
  content-packs/minnesota-itasca-snowline-v01.crystalpack \
  d7139c4a1e4ada8aa9b389675b07aaf119bb670c2003fa21974ddf9ac6856c88 \
  planning/minnesota/towns/ely-pine-camp-v01/before.json \
  planning/minnesota/towns/ely-pine-camp-v01/after.json \
  planning/minnesota/towns/ely-pine-camp-v01/activities.json \
  planning/minnesota/towns/ely-pine-camp-v01/contract-winter-v03.json \
  content-packs/minnesota-ely-pine-camp-winter-FRESH.crystalpack
```

Serialize Cargo with `.local/minnesota-cargo.slot.lock` and preserve every
existing output. The complete source preview takes0.126 seconds after loading;
its whole town and six source views are retained in
`renders/ely-winter-source-v1229/`. Strict Rust readback passes. Independent
preservation in `renders/ely-winter-preservation-v1237/` proves all742 other
complete maps, all original Ely events/programs/commands, native assets,
wildlife, trainer teams and audio exact. The town matches the authored source
in all368,640 pixels. The earlier audit attempt incorrectly required unchanged
global script-catalog keys; the final audit permits only the seven exact new
resident/text/pickup labels, while retaining every original key and program.

Three native controller cases pass in `renders/ely-winter-controller-v1231/`:
resident talks and supply loop, actual Full Restore HP/status effects with PP
unchanged, free clinic PP recovery, four road seam lanes and three building
returns, once-only supply/Continue, full-pocket refusal and actual Hyper Potion
use to free space for retry, plus both original readable signs. Initial local
position, an injured Voyagear and the capacity fixture are explicit setup;
there are no state rewrites after input and no earned campaign claim.

The production stdio course passes two sessions and391 calls with empty stderr:
the three resident conversations, original sign, actual supply collection,
Continue with the claimed ball absent, all four regional lanes, clinic/Mart/lodge
returns and Start. It consumes the actual controller's retained initial save.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/check_ely_pine_camp_mcp.py \
  --pack content-packs/minnesota-ely-pine-camp-winter-v03.crystalpack \
  --save planning/minnesota/renders/ely-winter-controller-v1231/native-saves/initial.crystalsave \
  --client .local/inland-grass-cut-client-v1163-bin \
  --output planning/minnesota/renders/ely-winter-mcp-FRESH
```

All five reached native night/snow views in `renders/ely-winter-native-v1233/`
were inspected: lodge, supply approach and claimed return, clinic garden and
Mart windbreak. Original full facades, roof/wall/door assemblies, snow/pine
foreground and night palette remain coherent. Live particles are reviewed
separately from static source pixels. Every original/rejected pass is retained.

The native town-source guard regression rejects closed road ports, open outside
edges, stranded services/supply and an actor blocking the original sign approach
on this actual winter parent (`.local/itasca-ely-final-v1239/`). Owned Rust
formatting and Python syntax pass; workspace rustfmt retains existing unrelated
differences beginning in offset-allocator. The roster audit finds only225 active
Minnesota species across355 regional maps/323 trainers. Rust audio/desktop
verification passes all602 PCM hashes, frame counts and loop ranges.

## Connected regional evidence and limits

The [combined exterior atlas](../../renders/connected-winter-ely-v1238/full-state-native-current.png)
shows all108 actual exterior maps at one scale. There are no footprint overlaps,
inconsistent connection cycles or missing planned hubs. All108 lie in Winona's
explicit warp/seam graph; three panels distinguish direct seams from room/special
connections. This is actual native source art, not invented terrain or proof
that every story gate is presently open.

The bridge-aware Surf-boundary audit in `renders/connected-winter-surf-boundaries-v1240/`
checks closed/open native Minneapolis gate geometry with and without actor bodies.
Its only eight unmatched outer lanes belong to R02's inaccessible fenced drainage:
both water components have zero mount banks. Connected shores are not mislabeled
as escapes. Early reachable fishing banks must remain open; do not fence every
shore while closing real off-map exits.

The exact current river plan in `renders/connected-winter-river-plan-v1241/`
requires1,247 route actions each way, seven bridge underpasses, both wooded
reaches and St Anthony Falls. Actual production controller journey, bridge-plane
Continue/foot crossing and all five Falls/badge/seam cases pass in
`renders/connected-winter-river-controller-v1242/`, retaining the ordinary Winona
dismount and reached saves. Initial Slidewake, legal Surf/Waterfall and prior
badges are declared fixtures; this is not earned HM progression.

The [early-fishing check](../../connections/early-fishing-v01/README.md) now
verifies the free first-route Old Rod and actual dry-bank fishing with zero
badges and no field moves. Broader cave/terrain quality, remaining statewide
progression and final selection remain open. Older saved terrain
overrides are not migrated by this town rebase.
