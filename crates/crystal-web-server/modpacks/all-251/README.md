# All 251 Catchable

A server-owned Pokémon Crystal encounter expansion inspired by [Wowabox's All Pokémon Catchable 151](https://github.com/wowabox/All_Pokemon_Catchable_151_Mod). This is an original Gen 2 content overlay, not a port of the reference project's Lua code.

All 251 Gen 1–2 species, including Mew and Celebi, have renewable wild encounters. No trading, breeding, gifts, or event-only captures are required. Original evolution and capture rules are unchanged. Existing encounters retain their normal time-of-day behavior; every added slot applies morning, day, and night. Grass includes the game's ordinary cave/tower land encounters; water means Surf, not fishing.

The checked-in `encounters.json` is the authoritative mod content. It replaces selected slots in existing tables while preserving encounter rates and the original 7-slot land / 3-slot water distributions. Land slots 5–7 have 5%, 4%, and 1% chances per encounter; Surf slots 2–3 have 30% and 10%. Levels and habitats are explicit. Legendaries are repeatable rare encounters in late areas; Celebi is level 30 in Ilex Forest and Mew is level 40 outside Silver Cave. Progression gates and map geometry remain unchanged.

## Server integration

`crystal-web-server` builds the overlay at startup from `<web-root>/core-modular.browser.crystalpack`, verifies it, and writes `<data-dir>/modpacks/all-251.browser.crystalpack`. Its exact identity is registered in the multiplayer allowlist. The browser's `/realtime-clock.browser.crystalpack` request serves this new file, composed with the server's realtime-clock overlay; the source pack is never overwritten. Startup fails on invalid or incomplete content. The Docker server uses this path automatically; the data directory must be writable.

The added manifest `all-251-catchable-v1` separates this world from vanilla saves and matchmaking. Existing vanilla browser saves remain in their original namespace; there is no automatic save migration. Players must reload the game to load the new pack. This modifies content availability, not the relay server's existing trust model.

Build a standalone pack from the Rust workspace:

```sh
cargo run --locked -p crystal-web-server --bin catchable -- ../content-packs/core-modular.browser.crystalpack /tmp/all-251.browser.crystalpack
```

The same builder accepts the full desktop pack and preserves its audio storage mode. Keep the source pack's audio sidecars available when distributing sidecar-based packs.

## Added or relocated encounters

Slots below are one-based and have the same species and level at all three times of day.

| Map | Method | Slot | Pokémon | Level |
| --- | --- | ---: | --- | ---: |
| Route31 | Land | 7 | CHIKORITA | 5 |
| Route31 | Land | 6 | TOGEPI | 5 |
| Route32 | Land | 7 | MAREEP | 8 |
| Route32 | Land | 6 | FLAAFFY | 15 |
| Route36 | Land | 7 | VULPIX | 14 |
| Route36 | Land | 6 | CYNDAQUIL | 10 |
| IlexForest | Land | 7 | BULBASAUR | 10 |
| IlexForest | Land | 6 | CELEBI | 30 |
| Route34 | Land | 7 | IGGLYBUFF | 10 |
| Route34 | Land | 6 | PICHU | 10 |
| Route34 | Land | 5 | DITTO | 12 |
| Route35 | Land | 7 | EEVEE | 15 |
| Route35 | Land | 6 | AIPOM | 12 |
| Route35 | Land | 5 | YANMA | 12 |
| NationalPark | Land | 7 | SCYTHER | 15 |
| NationalPark | Land | 6 | PINSIR | 15 |
| NationalPark | Land | 5 | BEEDRILL | 15 |
| Route37 | Land | 7 | PINECO | 15 |
| Route37 | Land | 6 | EXEGGCUTE | 15 |
| Route37 | Land | 5 | EXEGGUTOR | 25 |
| Route38 | Land | 7 | SNUBBULL | 16 |
| Route38 | Land | 6 | HERACROSS | 18 |
| Route39 | Land | 7 | GIRAFARIG | 18 |
| Route39 | Land | 6 | SUNFLORA | 20 |
| Route42 | Land | 7 | BAYLEEF | 22 |
| Route42 | Land | 6 | IVYSAUR | 22 |
| Route43 | Land | 7 | JUMPLUFF | 27 |
| Route43 | Land | 6 | AMPHAROS | 30 |
| Route44 | Land | 7 | BELLOSSOM | 25 |
| Route44 | Land | 6 | VICTREEBEL | 30 |
| Route45 | Land | 7 | GOLEM | 35 |
| Route45 | Land | 6 | CHARMELEON | 26 |
| Route45 | Land | 5 | SKARMORY | 27 |
| Route46 | Land | 7 | MANKEY | 8 |
| BurnedTower1F | Land | 7 | MAGBY | 12 |
| BurnedTower1F | Land | 6 | QUILAVA | 22 |
| BurnedTower1F | Land | 5 | WEEZING | 25 |
| BurnedTowerB1F | Land | 7 | ENTEI | 40 |
| BurnedTowerB1F | Land | 6 | RAIKOU | 40 |
| SproutTower2F | Land | 7 | CLEFFA | 5 |
| SproutTower3F | Land | 7 | TOGETIC | 20 |
| TinTower2F | Land | 7 | GENGAR | 35 |
| TinTower3F | Land | 7 | ESPEON | 30 |
| TinTower4F | Land | 7 | UMBREON | 30 |
| TinTower5F | Land | 7 | NINETALES | 35 |
| TinTower6F | Land | 7 | ARCANINE | 35 |
| TinTower7F | Land | 7 | CROBAT | 35 |
| TinTower8F | Land | 7 | HOUNDOOM | 35 |
| TinTower9F | Land | 7 | HO_OH | 60 |
| MountMortar1FOutside | Land | 7 | TYROGUE | 15 |
| MountMortar1FInside | Land | 7 | HITMONLEE | 25 |
| MountMortar2FInside | Land | 7 | HITMONCHAN | 30 |
| MountMortar2FInside | Land | 6 | MACHAMP | 35 |
| MountMortarB1F | Land | 7 | HITMONTOP | 25 |
| UnionCave1F | Land | 7 | CHARMANDER | 10 |
| UnionCaveB1F | Land | 7 | SHUCKLE | 20 |
| UnionCaveB2F | Land | 7 | LAPRAS | 25 |
| DarkCaveBlackthornEntrance | Land | 7 | STEELIX | 35 |
| IcePath1F | Land | 7 | SMOOCHUM | 20 |
| IcePathB1F | Land | 7 | PILOSWINE | 33 |
| IcePathB2FMahoganySide | Land | 7 | DEWGONG | 30 |
| IcePathB3F | Land | 7 | ARTICUNO | 50 |
| Route2 | Land | 7 | FORRETRESS | 30 |
| Route3 | Land | 7 | NIDOQUEEN | 35 |
| Route3 | Land | 6 | NIDOKING | 35 |
| MountMoon | Land | 7 | CLEFABLE | 30 |
| MountMoon | Land | 6 | WIGGLYTUFF | 30 |
| Route5 | Land | 7 | ALAKAZAM | 35 |
| Route6 | Land | 7 | VILEPLUME | 35 |
| Route7 | Land | 7 | PORYGON | 25 |
| Route7 | Land | 6 | PORYGON2 | 40 |
| Route8 | Land | 7 | FLAREON | 30 |
| Route8 | Land | 6 | JOLTEON | 30 |
| Route8 | Land | 5 | KADABRA | 20 |
| Route9 | Land | 7 | PRIMEAPE | 30 |
| Route10North | Land | 7 | ELECTRODE | 30 |
| Route10North | Land | 6 | ZAPDOS | 50 |
| Route10North | Land | 5 | ELECTABUZZ | 25 |
| Route11 | Land | 7 | SNORLAX | 40 |
| Route13 | Land | 7 | SCIZOR | 35 |
| Route14 | Land | 7 | BLISSEY | 40 |
| Route14 | Land | 6 | CHANSEY | 25 |
| Route15 | Land | 7 | PIDGEOT | 36 |
| Route16 | Land | 7 | MEGANIUM | 36 |
| Route17 | Land | 7 | TYPHLOSION | 36 |
| Route18 | Land | 7 | VENUSAUR | 36 |
| Route21 | Land | 7 | MAGCARGO | 35 |
| RockTunnel1F | Land | 7 | ELEKID | 15 |
| RockTunnelB1F | Land | 7 | RAICHU | 30 |
| RockTunnelB1F | Land | 6 | MAGNETON | 30 |
| RockTunnelB1F | Land | 5 | KANGASKHAN | 25 |
| VictoryRoad | Land | 7 | AERODACTYL | 35 |
| VictoryRoad | Land | 6 | CHARIZARD | 40 |
| VictoryRoad | Land | 5 | RHYDON | 40 |
| RuinsOfAlphOutside | Land | 7 | SUDOWOODO | 20 |
| RuinsOfAlphInnerChamber | Land | 7 | XATU | 25 |
| SilverCaveRoom1 | Land | 7 | TYRANITAR | 55 |
| SilverCaveRoom1 | Land | 6 | PUPITAR | 40 |
| SilverCaveRoom2 | Land | 7 | MOLTRES | 50 |
| SilverCaveRoom3 | Land | 7 | MEWTWO | 70 |
| SilverCaveOutside | Land | 7 | MEW | 40 |
| Route30 | Surf | 3 | TOTODILE | 10 |
| Route31 | Surf | 3 | SQUIRTLE | 10 |
| Route34 | Surf | 3 | CROCONAW | 22 |
| Route35 | Surf | 3 | WARTORTLE | 22 |
| Route40 | Surf | 3 | CHINCHOU | 20 |
| Route41 | Surf | 3 | LANTURN | 27 |
| Route41 | Surf | 2 | MANTINE | 20 |
| CianwoodCity | Surf | 3 | CORSOLA | 20 |
| OlivineCity | Surf | 3 | STARYU | 20 |
| OlivinePort | Surf | 3 | SHELLDER | 20 |
| WhirlIslandSW | Surf | 3 | CLOYSTER | 35 |
| WhirlIslandB2F | Surf | 3 | KINGDRA | 40 |
| WhirlIslandLugiaChamber | Surf | 3 | LUGIA | 60 |
| WhirlIslandLugiaChamber | Surf | 2 | SEADRA | 30 |
| UnionCaveB1F | Surf | 3 | OMANYTE | 20 |
| UnionCaveB2F | Surf | 3 | OMASTAR | 40 |
| Route32 | Surf | 3 | QWILFISH | 20 |
| Route44 | Surf | 3 | REMORAID | 20 |
| LakeOfRage | Surf | 3 | OCTILLERY | 30 |
| LakeOfRage | Surf | 2 | GYARADOS | 25 |
| SlowpokeWellB1F | Surf | 3 | AZUMARILL | 20 |
| SlowpokeWellB2F | Surf | 3 | SLOWKING | 30 |
| SlowpokeWellB2F | Surf | 2 | SLOWBRO | 25 |
| EcruteakCity | Surf | 3 | POLITOED | 25 |
| Route42 | Surf | 3 | POLIWRATH | 30 |
| Route43 | Surf | 3 | SUICUNE | 40 |
| DragonsDenB1F | Surf | 3 | DRAGONAIR | 30 |
| DragonsDenB1F | Surf | 2 | DRATINI | 15 |
| Route45 | Surf | 3 | DRAGONITE | 55 |
| Route19 | Surf | 3 | KABUTO | 25 |
| Route20 | Surf | 3 | KABUTOPS | 40 |
| CinnabarIsland | Surf | 3 | STARMIE | 35 |
| VermilionCity | Surf | 3 | KINGLER | 30 |
| Route27 | Surf | 3 | FERALIGATR | 36 |
| Route26 | Surf | 3 | BLASTOISE | 36 |
| Route25 | Surf | 3 | VAPOREON | 30 |

## Verification

```sh
cargo test --locked -p crystal-web-server
```

The integration tests use the repository's real browser pack. They verify 251-species coverage, reject a deliberately removed Celebi encounter, reject unknown species, preserve all non-encounter content, round-trip the compiled pack, and check deterministic identity and multiplayer acceptance. A separate hosted-pack test verifies composition with the realtime-clock overlay.
