# All Pokémon, solo

The normal `tools/build-crystalpack.sh` initializer (and its Windows wrapper)
enables this mod by default for native, browser and terminal/MCP packs. Hosted
browser servers apply the same Rust builder. All 251 Generation I–II Pokémon,
including every starter, fossil, trade evolution, Mew and Celebi, have renewable
wild encounters. No other player, trade, gift, breeding or timed distribution is
required. Capture probabilities, story gates and encounter rates are preserved.

Buy a **LINK CABLE for ¥2,100** at **Celadon Department Store, 4F**, alongside
Fire, Water, Thunder, Leaf, Moon and Sun Stones. The mod extends the existing
counter's inventory and keeps its original merchandise. Select **PACK → ITEMS
→ LINK CABLE → USE**, then choose a Pokémon. Each successful evolution consumes
one cable. Eggs, Everstone holders and incompatible Pokémon keep the cable.

| Pokémon | Evolution | Required held item |
| --- | --- | --- |
| Kadabra | Alakazam | — |
| Machoke | Machamp | — |
| Graveler | Golem | — |
| Haunter | Gengar | — |
| Poliwhirl | Politoed | King's Rock |
| Slowpoke | Slowking | King's Rock |
| Onix | Steelix | Metal Coat |
| Scyther | Scizor | Metal Coat |
| Seadra | Kingdra | Dragon Scale |
| Porygon | Porygon2 | Up-Grade |

Give the required held item before using the cable. Successful evolution also
consumes that held item. Ordinary link trades and other evolution methods
remain available. Every evolved form can also be caught directly.

The single encounter catalog is [encounters.json](encounters.json); the
[encounter guide](../../crates/crystal-web-server/modpacks/all-251/README.md)
lists locations, methods, levels and rarity. Added slots apply morning, day
and night. Water entries require Surf. Legendaries remain rare and some areas
require normal story progress.

To apply the mod to an existing local pack without rebuilding source exports:

```sh
cargo run --locked -p crystal-assets --bin pack_all_pokemon -- \
  content-packs/core-modular.crystalpack content-packs/all-pokemon.crystalpack
cargo run --locked -p crystal-assets --bin pack_all_pokemon -- \
  content-packs/core-modular.browser.crystalpack content-packs/all-pokemon.browser.crystalpack
cargo run --locked -p crystal-assets --bin pack_text_tui -- . \
  content-packs/all-pokemon.browser.crystalpack content-packs/all-pokemon.text-tui.crystalpack
cargo run --locked -p geothite -- play content-packs/all-pokemon.text-tui.crystalpack
```

Inputs must be verified local packs. The Rust builder preserves audio storage,
embedded programs, PCM hashes, frame counts, loop ranges and unrelated runtime
assets. Reapplying the defaults preserves pack identity. Keep every generated
pack ignored; distribute any existing audio sidecars with a sidecar-based pack.
Use a client built from the current repository for cable held-item behavior.

The manifests `all-251-catchable-v1` and `solo-evolutions-v1` distinguish the mod
from vanilla saves and multiplayer worlds. Older vanilla or encounter-only saves are not
migrated automatically; hosted users need to reload after their server updates.

Verification uses the real external pack:

```sh
cargo test --locked -p crystal-assets --test all_pokemon
export CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/all-pokemon.text-tui.crystalpack"
cargo test --locked -p crystal-bevy renderer_neutral_controller_uses_link_cable_through_production_pack
```
