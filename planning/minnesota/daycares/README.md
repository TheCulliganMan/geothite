# Independent Minnesota daycares

The additive `pack_minnesota_daycares` publisher adds two ordinary native
small houses without replacing existing residents or story buildings.
The authoritative house contract is `../daycares.json`.

| Settlement | Exterior facade (metatiles) | Door / passive return (walking tiles) | Interior | Stable save identity |
| --- | --- | --- | --- | --- |
| Mankato | `(10,12)` | `(21,27)` / `(21,28)` | `MnMankatoDayCare` | `minnesota-mankato` |
| Bemidji | `(6,15)` | `(13,33)` / `(13,34)` | `MnBemidjiDayCare` | `minnesota-bemidji` |

Mankato supplies an early southern stop; Bemidji supplies a later northern stop.
Both houses use original complete small-house facade blocks and the native
four-by-four-metatile house interior. No fence is needed to close an approach.
The publisher rejects plots containing actors, events or non-floor collision,
then checks the new door and all previously reachable exterior entrances.

Each interior has a man at `(2,3)`, a lady at `(5,3)` and an egg attendant at
`(6,5)`. Their approaches are `(2,4)`, `(5,4)` and `(5,5)`; the arrival is
`(2,6)`. Exit carpets `(2,7)` and `(3,7)` return to the matching exterior's
passive return. The top-left bookcase explains the local service and the
exterior sign identifies the daycare and fees.

## Custody and normal mechanics

Each site owns two slots, its breeding pair, prebuilt/ready egg and countdown.
A partner or egg at Mankato is never visible or collectible as Bemidji custody,
and vice versa. There is no cross-location transfer operation. Runtime snapshots
project the current interior's daycare so NPC dialogs and price reports agree
with the authoritative mutation target.

The composed script runner pauses before either caretaker or the egg attendant
service. The production controller presents the offer and obtains fresh input
before applying it, including when ordinary script commands precede the service.

Every accepted overworld step advances each site's residents by the original
Crystal experience mechanic, even while the player is elsewhere. Retrieval
costs 100 plus 100 for each level gained. Insufficient money and a full party
leave the resident and wallet intact. Deposit rejects eggs, mail carriers,
the last party member and the last healthy non-egg partner. Nicknames, species,
DVs, owner and held items remain with the resident. Original daycare move
learning, stat recalculation and healing occur on retrieval; daycare growth
never invokes evolution or encounter scaling.

Compatible regional parents use the existing egg compatibility, inheritance,
base-species normalization and walking countdown. An attendant delivers a ready
egg once, only with party room. A ready egg stays at its site if a parent is
retrieved; an unready egg is discarded when the pair becomes incompatible.
Collecting an egg initializes the next breeding cycle only for that site.

## Saves and publication

Save version 26 stores regional states under stable identity keys. The original
`day_care` field remains the original Crystal daycare's custody. Versions 24
and 25 are checksum-validated with their original binary layouts before
migration; their legacy residents are never assigned to a new location.
Unknown regional daycare identities and invalid stored Pokemon are rejected.

Build with the repository's shared Cargo slot and a dedicated target directory:

```sh
CARGO_TARGET_DIR=target/daycares CARGO_INCREMENTAL=0 \
  cargo build --locked -p crystal-assets --bin pack_minnesota_daycares
target/daycares/debug/pack_minnesota_daycares BASE.crystalpack OUTPUT.crystalpack
```

Rebuild the game client from the feature source before loading the child pack.
The maps/scripts are in the pack; independent custody and save version 26 are
implemented in the matching core/runtime. Compose the additive daycare overlay
over the selected story/note pack and validate it with that rebuilt client.

Both paths must be real local packs; the output must be ignored and differ from
the source. The publisher produces a new content identity, does not select or
publish the candidate, and refuses a second installation. Apply this additive
overlay after other selected town/room overlays so a later town regeneration
does not replace its exterior facade. No remote push or deployment is involved.

## Verification

Self-contained engine regressions:

```sh
cargo test --locked -p crystal-core day_care --lib
cargo test --locked -p crystal-core binary_save_ --lib
RUST_MIN_STACK=67108864 \
  cargo test --locked -p crystal-runtime --lib runtime_composed_script_leaves_day_care
```

Opt-in pack/controller regressions require a completed parent and its published
child. They read external packs at runtime, never through compile-time fixtures.

```sh
CRYSTAL_DAY_CARE_BASE_PACK=BASE.crystalpack \
  cargo test --locked -p crystal-assets minnesota_daycares --lib -- --ignored
RUST_MIN_STACK=67108864 CRYSTAL_DAY_CARE_TEST_PACK=OUTPUT.crystalpack \
  cargo test --locked -p crystal-bevy --lib day_care_controller -- --ignored
```

Controller tests declare initial location, regional partners and RNG as fixtures;
subsequent walking, door entry/return, choices, Party selection, deposits,
withdrawals, breeding, egg collection and Save/Continue use production controller
inputs. They do not establish earned regional progression. Set
`CRYSTAL_DAY_CARE_RENDER_DIR` to a new ignored directory to retain native 2D
captures; earlier filenames are never overwritten.

The local candidate built over `minnesota-dialogue-overhaul-v20.crystalpack`
passed all three controller regressions at both houses, with eight native 2D
captures visually reviewed. This covers door entry/return, declined offers,
cancelled Party selection, deposits, walking growth, Save/Continue, cancelled
and paid withdrawals, breeding, single egg handoff and interrupted offers.
Three existing Crystal daycare UI regressions also passed against the original
core pack, including exact fee operands and egg handoff timing. The daycare
core regressions (28), state suite (62), binary migrations, pack preservation
and composed-service boundary regression passed. These are source/controller
fixtures, not a manual story playthrough.
