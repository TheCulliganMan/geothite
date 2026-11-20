# The two field stations

Tamarack offers the player the lighthouse part on the first visit to Grand Marais. On a separate conversation she offers Birchit, who has been standing on the windowsill to watch the boats. Both the dark-harbor and repaired-lighthouse versions ask for a decision. Declining leaves Birchit at the station. The player can walk out through either side of the doorway, visit the lighthouse, save and come back. Both exit lanes still let Tamarack catch a player who needs the urgent part.

Tamarack wears the same native sprite here as in Winona. A successful adoption gives one level-8 Birchit holding a Berry. If the party is full and the selected PC box has room, Tamarack tells the player where to find Birchit. If both are full, she waits. She packs five Poké Balls and three Potions separately. Each delivery gets a message only after that item is in the Pack. Tamarack tells the player exactly what she is holding when a pocket needs space. The keeper continues to handle his own Super Potion at the lighthouse.

A new adoption records which supplies the player actually receives. Older completed adoptions carry only a single receipt. It is impossible to distinguish a lost item from one the player received and used. Those completions retain their original state. Existing saves continue to use their original pack.

In Winona, the research aide now tells a player without a partner that they can meet all three before picking. Once the player has chosen, the aide still gives the original travel supplies. Before Tamarack hands over the reading, the aide points back to her. With the reading in hand, the aide gives the Rochester clinic direction. After delivery, the original research partner offers resume. Previously, before the player had chosen anyone, the aide answered in Tamarack’s voice and told them to take an envelope they did not possess.

Bill’s Eevee offer in Crystal asks a real question and ends the conversation if the player declines. In Polished Crystal, each of Elm’s starter balls can be put back, and Elm’s reward scripts confirm delivery before recording collection. Grand Marais keeps the option to send a partner to the PC and lets the two travel supplies arrive on different visits. [Crystal’s Eevee offer](https://github.com/pret/pokecrystal/blob/master/maps/BillsFamilysHouse.asm#L9-L41) · [Polished Crystal’s starter and research gifts](https://github.com/Rangi42/polishedcrystal/blob/master/maps/ElmsLab.asm)

## Verification

The [whole-pack comparison](check-pack.py) uses the exact [Field](field-build.json) and [Intro](intro-build.json) parents. It compares both station programs and their compiled records, the doorway positions, Tamarack’s sprite binding, and every other map, trainer and embedded asset. The [regional gift audit](gift-audit.md) covers the other scripted Pokémon and item gifts and records the remaining Lamp Gallery feedback issue.

Both editions passed the three Grand Marais checks and the four-stage Winona aide check. The native checks use declared party, bag and story setups. The Grand Marais controller walks to the keeper and back, operates the Pack and clinic PC, saves, continues and tries the same gifts again. The Winona check speaks to the aide at four stages of Ivy’s errand and reads the pages from the game. These are focused station checks. The regional plan and the five-room League comparison are documented in the [previous pass](../ely-return-v01/README.md).

Use the appropriate receipt, choose a fresh external output path and run from the repository root.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_ely_challenge -- --marais-supplies "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/marais-supplies-v01/check-pack.py "$parent" "$child"
export CRYSTAL_WINONA_TEST_PACK="$child"
cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 marais_partner_controller_ winona_aide_uses_the_players_actual_stage
```

Set `MN_MARAIS_PARTNER_FRAMES` to a fresh capture directory to retain the native question, held-supply and aide screens.
