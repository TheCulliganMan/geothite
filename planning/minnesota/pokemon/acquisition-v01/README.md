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
the acquisition upper bound to 214. The selected
[Castorock discovery](castorock/README.md) on the warm Cedar parent raises that
structural upper bound to 216, installs all 224 reviewed walking banks and
passes native teaching/gate/clue/capture/evolution/retry/Continue cases. Eight
other rare families still lack sources. Consult `world.json` for the current
composed pack; structural availability does not establish earned collection.

The selected [Kindlit research partner](starter-return.md) closes the Fire
starter choice lock after the regional Champion victory. The structural upper
bound remains 216; eight rare families still require acquisition sources.

The source roster subsequently added VERDIGRIM, bringing the active count to
225. The selected starter-return pack still contains 224 regional Dex members
and a 224-species completion threshold. Nine families lack acquisition sources
when audited against the expanded roster. The new species must be compiled,
added to Dex membership and included in the final reward threshold before full
collection can be accepted.

Run the read-only acquisition check with `--require-roster-alignment` to make
catalog, membership or final-threshold drift fail explicitly. Its JSON report
separates these integration failures from absent acquisition sources. Default
reporting remains available for inspecting an intentionally incomplete pack.

The selected [Grand Marais delivery guard](marais-partner-retry.md) preserves
Birchit when both party and selected PC box are full, while retaining native PC
delivery. Current-parent controller and production stdio checks pass.
