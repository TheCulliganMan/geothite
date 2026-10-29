# Minnesota habitats and Minneapolis cycling

The habitat/cyclist stage was verified as
`content-packs/minnesota-current-habitats-cyclists-r15-v01.crystalpack`,
built on the accepted R15 grass revision. The current composed selector is
recorded in `planning/minnesota/world.json`; later stages inherit this work.
This stage preserves the parent terrain,
warps, existing actors, authored events and unrelated catalogs.

Fourteen grass zones and seven Surf/fishing habitats add 40 ordinary family
roots. Seven private Good/Super Rod groups preserve Old Rod tables, thresholds,
timed tables and original swarm behavior. All additions use the active Minnesota
roster. Pokédex AREA includes zone-only grass residents.

Minneapolis has 18 roaming cyclists with optional battles, native bicycle
sprites and patrol loops of 32–48 tiles. Their teams use Minnesota species.
The inherited bicycle workshop supplies the Bicycle after the first badge;
registration, riding, duplicate prevention and Save/Continue have controller
coverage on the shared R12 lineage.

Build the publishers from the checkout:

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0 \
RUST_MIN_STACK=67108864 cargo build --locked --profile wasm-dev \
  -p geothite -p crystal-assets --bin geothite \
  --bin pack_minnesota_habitats --bin pack_minneapolis_cyclists
```

The Rust habitat publisher accepts parent pack, `planning/minnesota` and a
new ignored output path. The cyclist publisher then layers the authored city
riders onto that output. Publishers refuse occupied output paths. Check the
current world selector before building or selecting another child.

On the selected R15 child, four controller cases pass actual grass, cedar,
Surf and registered-rod captures with ball consumption and Save/Continue.
Three cyclist cases pass patrol bounds, refusal and battle continuation.
The stdio client completes a real fishing bite through the production
controller with byte-empty stderr. Strict habitat and cyclist preservation
audits pass, and all 377 audio PCM hashes, frame counts and loop ranges match.
These cases declare initial fixture parties, items and locations; they do not
prove an earned statewide journey. No new 3D visual acceptance is claimed.

The acquisition audit finds 133 potential direct species and a generous upper
bound of 210 after all evolution edges. Fourteen species in eleven families
still require fossil, landmark or rare discovery sources. Evolution conditions
and complete campaign access remain to be verified. Full-game completion
remains open.

The separate [fossil research stage](fossils.md) adds the paired specimen
choice, Rochester restoration and post-League exchange. The selected composed pack increases
the acquisition upper bound to 214; consult its verification and selection status.
