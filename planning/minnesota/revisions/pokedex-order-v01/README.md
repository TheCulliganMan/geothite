# Regional Pokédex ordering

The authored sections already placed ordinary families before the mosquito
pseudo-legendary line, then legendaries and mythicals. The source-number table
had drifted: five active legendaries preceded 31 ordinary replacement species,
and the mosquito line came last. Regenerating the table from its sections keeps
the first 186 active numbers unchanged and corrects the final 39 entries.

The active roster remains **225 species in 99 families**. Ordinary species are
#001–217. The final entries are:

| Regional number | Species | Group |
| --- | --- | --- |
| #218 | Wrigglet | Pseudo-legendary family |
| #219 | Pupump | Pseudo-legendary family |
| #220 | Mosquitan | Pseudo-legendary family |
| #221 | Babehemoth | Legendary |
| #222 | Kettlurk | Legendary |
| #223 | Verdigrim | Legendary |
| #224 | Bunjant | Legendary |
| #225 | Vergloom | Legendary |

Grass, Fire and Water starter families remain #001–009, and evolution stages
remain adjacent. Stable species IDs, family LINE identifiers and active
membership are unchanged. All currently authored mythicals are retired and
remain archived; ordering does not restore retired species.

The source audit rejects divergent section/table order, missing or repeated
families, separated evolution stages, misplaced starters and ordinary families
after special families. Its publication check compares every active identity
and number. Seven focused regressions reject the previous numbering drift and
incorrect source or published sequences.

```sh
python3 planning/minnesota/tools/audit_pokedex_roster.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s planning/minnesota/tools -p test_pokedex_order.py
```

The ordering-only Rust builder accepts the exact published Fair presentation
raw and hosted archives. It verifies complete pack preservation after restoring
only the regional-number constants, save policy, ordering manifest and derived
identity. The comparison covers every `CompiledGamePack` field, including
compiled audio, audio metadata, runtime files and the compile report. All maps,
encounters, species records, artwork and audio remain identical to the
corresponding parent edition. Both output archives also pass verified readback
and `verify_crystalpack --desktop --audio`.

Schema 11 adds the two exact Fair presentation save identities and retains the
22 inherited identities. Direct parent saves preserve the entire validated idle
checkpoint without normalization, terrain reconciliation or actor resets.
Older releases retain their existing migration policies. Isolated fixtures at
`MnR10C00R00 (24, 20)` and `MnStateFairgrounds (36, 28)` verify research, party,
bag, money, flags and saved actor state through complete checkpoint equality.
Both raw and hosted editions pass, including a corrupt-primary recovery case,
unchanged source primary/backup bytes and subsequent resume.

The external input archives used for this verification are:

- `/mnt/homelab/config/geothite/releases/northstar-fair-presentation-20261122-r2/input.crystalpack`
- `/mnt/homelab/config/geothite/releases/northstar-fair-presentation-20261122-r2/hosted.crystalpack`

The isolated Linux verification container mounts these as
`/inputs/raw-parent.crystalpack` and `/inputs/hosted-parent.crystalpack`, outside
its frozen source at `/source/rust`. The following build and focused checks
were executed there. `pack-authoring` enables the builder and preservation
test; the 64 MiB test-thread stack accommodates complete pack equality. Output
paths must be fresh and ignored. An exported source tree needs `git init
--quiet .` so the builder can check `.gitignore`.

```sh
cd /source/rust
export CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864
git init --quiet .
dex_check_dir=$(mktemp -d "$PWD/target/pokedex-order-verification.XXXXXX")
dex_raw_parent=/inputs/raw-parent.crystalpack
dex_hosted_parent=/inputs/hosted-parent.crystalpack
dex_raw="$dex_check_dir/input.crystalpack"
dex_hosted="$dex_check_dir/hosted.crystalpack"

cargo build --locked -p crystal-assets --features pack-authoring \
  --example pack_northstar_dex_order --bin verify_crystalpack
target/debug/examples/pack_northstar_dex_order \
  "$dex_raw_parent" "$dex_hosted_parent" "$dex_raw" "$dex_hosted"
target/debug/verify_crystalpack "$dex_raw" --desktop --audio
target/debug/verify_crystalpack "$dex_hosted" --desktop --audio

CRYSTAL_DEX_ORDER_RAW_PARENT="$dex_raw_parent" \
CRYSTAL_DEX_ORDER_HOSTED_PARENT="$dex_hosted_parent" \
  cargo test --locked -p crystal-assets --features pack-authoring --lib \
  dex_order_preserves_all_other_pack_content -- --ignored --nocapture

CRYSTAL_DEX_ORDER_PARENT="$dex_raw_parent" CRYSTAL_DEX_ORDER_TARGET="$dex_raw" \
  cargo test --locked -p crystal-runtime --lib \
  dex_order_copies_exact_idle_checkpoints_and_preserves_research -- --ignored --nocapture
CRYSTAL_DEX_ORDER_PARENT="$dex_hosted_parent" CRYSTAL_DEX_ORDER_TARGET="$dex_hosted" \
  cargo test --locked -p crystal-runtime --lib \
  dex_order_copies_exact_idle_checkpoints_and_preserves_research -- --ignored --nocapture
cargo test --locked -p crystal-runtime --lib dex_order
```

The website revision preserves the existing guide's profiles, acquisition
information, archives and media while updating catalog order, species/family
cards, search numbers, detail labels and adjacent-entry links. This candidate
was generated and verified with:

```sh
python3 planning/minnesota/tools/revise_pokedex_order.py \
  target/minnesota-pokedex-web/pokedex/releases/20261120-fair-patrols-sprite-edges-47 \
  target/minnesota-pokedex-web/pokedex/releases/20261124-regional-order-48
python3 planning/minnesota/tools/audit_pokedex_roster.py \
  --release target/minnesota-pokedex-web/pokedex/releases/20261124-regional-order-48
POKEDEX_WEB_RELEASE=target/minnesota-pokedex-web/pokedex/releases/20261124-regional-order-48 \
POKEDEX_WEB_BASELINE=target/minnesota-pokedex-web/pokedex/releases/20261120-fair-patrols-sprite-edges-47 \
  node web-client/pokedex/order.test.mjs
```

The focused web regression passes all 225 numbered searches and detail-page
neighbors, 99 family cards, the five-legendary and three-pseudo filters, and
baseline preservation. Immutable staging verifies all 2,341 remote files and
227 public responses against local bytes. Desktop Chromium at 1280px and
mobile Chromium at 390px also pass ordering, filters, search and loaded-sprite
checks. Pre-promotion adjacent-page clicks explicitly route to the immutable
candidate; those checks do not claim the current public alias has switched.

Generated packs, verification fixtures, release scripts and browser captures
remain ignored. Builds and publication use an export of the committed source,
excluding unrelated cave work in the shared checkout. Verification is limited
to the affected content and isolated saves; no campaign playthrough is used.
Client/WASM release checks and final deployment receipts are separate from
these source, pack, save and immutable-website checks.

## Verified release build

The server, both native clients and all six browser WASM modules were built
from committed source `0518db8642bc8683cf61f0cb7e145f1bcbf74eb1`. The
macOS arm64 and Linux x86-64 executables each passed an isolated production
Start/Pokédex check covering all 225 entries, the final eight detail pages,
and return to the unchanged checkpoint. The fixtures are generated test saves.

| Edition | Archive SHA-256 | Content identity |
| --- | --- | --- |
| Raw Minnesota | `9415b9ae8def05303e8cd07ade5525c9c6f92c8a775498cb69e67975a19e7f13` | `8123e801db425666fefa8c5d2b532f0c9f130ecc4b59b1636fe20a5ee251add6` |
| Browser and installer | `3f1df5134910d4c8b32e31428a668736ca8fbbc8b05e7c240865abcf6c7276d8` | `d77d63d93dc30b953cc5a8f8a8681a6c2af2fca4a4b480b86360863a1f13a8f6` |

Formatting and both affected crates' all-target Clippy checks passed. The
installer regressions passed silent install/reinstall, saved-file preservation
and corrupt executable/content rejection. Existing unrelated compiler warnings
remain. The older broad web test still expects at least three Water/Ice species;
the current roster contains none. The dedicated ordering regression verifies
all active entries against the current roster and passes.

## Publication

Release `northstar-pokedex-order-20261124-v1` is live for the browser game,
`/tui`, and both native installer targets. Website release
`20261124-regional-order-48` is live at `/pokedex/`. Public checks matched all
74 active web assets and six WASM modules to the private candidate. Both
native downloads passed real PTY and installed-launcher MCP checks, including
F5 save, silent reinstall and resume with all 225 seen/caught entries intact.
Desktop Chromium and mobile WebKit passed the same entry/detail checks and
Save/Continue. All checks used disposable fixtures without campaign movement.

The site's 227 canonical responses match the generated release. Desktop
Chromium and mobile WebKit checked the five-legendary and three-pseudo filters,
search 222, previous/next navigation and the final entry, with decoded sprites
and no URL overrides. The server image, environment and original mounts stayed
unchanged during this separate static publication.

The ignored release folder `target/pokedex-order-release/` retains the source,
pack, migration, browser, native and website proofs. The deployment host retains
its release and rollback record under
`/mnt/homelab/config/geothite/releases/northstar-pokedex-order-20261124-v1`.
Mobile text controls also passed on-screen bounds and real-tap checks; the
WebKit text screenshot omitted their button pixels, so that capture alone is
not evidence of their appearance. The default painted mobile capture shows
all eight controls.
