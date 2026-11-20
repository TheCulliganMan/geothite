# Northstar Fair release

Published November 20, 2026 as `northstar-fair-20261120`. [Public checks and rollback](../../../../docs/deployment.md#minnesota-fair-patrols-release-2026-11-20)

The cumulative Intro Fair Patrols edition includes the State Fair trainers and concessions, the Minneapolis Game Corner, the Mothlight Battle Café challenge, and the restored native Pokémon sprite outlines. The release uses the Intro edition’s original trainer portraits. The hosted edition uses the existing clock and player-customization overlays.

The [build recipe](build.json) pins the external inputs and both outputs by checksum. Build the local and hosted archives together in a fresh ignored directory. Use the external lakeside archive from the previous public release for the hosted input. The sprite tool requires a fresh output path.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo build --locked -j1 -p crystal-assets \
    --bin pack_minnesota_sprite_edges --bin verify_crystalpack --example pack_northstar_fair_release
work=$(mktemp -d target/northstar-fair-rebuild.XXXXXX)
target/debug/pack_minnesota_sprite_edges \
    content-packs/minnesota-intro-fair-patrols-v01.crystalpack "$work/sprites.crystalpack"
target/debug/examples/pack_northstar_fair_release \
    "$work/sprites.crystalpack" content-packs/minnesota-lakeside-v01.crystalpack \
    target/lakeside-20261118/hosted.crystalpack "$work/raw.crystalpack" "$work/hosted.crystalpack"
for edition in raw hosted; do
    target/debug/verify_crystalpack "$work/$edition.crystalpack" --desktop --audio || exit 1
done
shasum -a 256 "$work"/{raw,hosted}.crystalpack
```

The builder preserves all existing map IDs, dimensions, saved actor indices and catalog references. Its compatibility metadata records tile and actor changes between the exact previous and new layouts. The established save loader validates the prior identity and writes a separate copy after checking the entire migrated state. Replacements for the Mankato and Duluth gym gates, the Layered Fen and Tall Tale Trail are tied to their original event scripts. Walking and boating checkpoints displaced by terrain are placed on reachable ground or water on the same map. Existing coordinates take priority wherever still valid. The previous archive and prior saves remain available for unfinished battles, cutscenes and other checkpoints that the idle-save migration cannot accept.

Verify both direct upgrade paths without gameplay input. These fixtures preserve a trained party member, money, Coins, inventory, defeated-trainer progress and gate state. They also check the original save bytes and an incorrect source identity.

```sh
CRYSTAL_FAIR_PARENT="$PWD/content-packs/minnesota-lakeside-v01.crystalpack" \
CRYSTAL_FAIR_TARGET="$PWD/$work/raw.crystalpack" \
cargo test --locked -j1 -p crystal-runtime --lib fair_release_copies_checkpoints_across_exact_archives -- --ignored
CRYSTAL_FAIR_PARENT="$PWD/target/lakeside-20261118/hosted.crystalpack" \
CRYSTAL_FAIR_TARGET="$PWD/$work/hosted.crystalpack" \
cargo test --locked -j1 -p crystal-runtime --lib fair_release_copies_checkpoints_across_exact_archives -- --ignored
```
