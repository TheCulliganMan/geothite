# Minnesota habitats and Minneapolis cycling

The selected [complete encounter progression](encounter-progression-v01/README.md)
covers all 111 regional encounter maps and all 225 active species, follows the
actual badge gates, corrects 112 local trainer teams and retains free early
partners and both fossil choices. The current pack also preserves the reviewed
Northstar opening and all accepted geography. Web release
`20261031-progression-32` uses its actual levels, conditions and evolution paths.
The older counts below describe their named historical stages.

The [early research revision](early-research.md) removes the Champion gates for
the unchosen starters and second fossil. Its three native cases include an
earned first-delivery return before any badge and both complete fossil paths.
The larger full-roster candidate and biome integration remain separate work;
historical stage counts below retain their original pack bindings.

## Earlier habitat and campaign review stages

The selected world and the comprehensive campaign are separate lineages. Resolve
`world.json` before interpreting the selected-world missing-source counts below.
The unselected 743-map comprehensive V18 candidate includes the optional discovery
sites, Verdigrim story, indoor fossil service and current landmark habitats. Its
recorded acquisition closure reaches all 225 active species only under generous
access and evolution assumptions; that is not an earned collection or proof that
all dependencies work in one journey. Local discovery replays declare their
initial access, party and resources.

The selected-world pine island, water/cave habitats and newer mainland trail
are now reconciled in the unselected743-map V20 campaign. The first water
publication rejected a cache-container mismatch before output; corrected V19
and V20 publication/preservation checks pass. V20 reproduces byte for byte,
retains742 other complete maps across its mainland update, and passes current
225-species policy and602 audio checks. Native components, legal acquisition
paths and earned final-pack acceptance remain incomplete. The current
requirement-by-requirement record is [the comprehensive campaign tracker](../../revisions/comprehensive-game-v01/requirements.json).

The [water and cave habitat phase](water-caves-v01/README.md) adds 19 ordinary
Surf areas and ten cave floors, a signed bounded Moorhead bank and repairs26
inherited fishing headers. It preserves the Leech Lake pine-island grass and
all prior landmark habitats. The web guide refreshes times, levels, named town
locations and search from the same verified pack. The216-species structural
upper bound and nine pending routes remain explicit; broader grass/rod ecology
and rare-species integration are still active work.

The [current landmark habitats and web guide](landmark-habitats.md) add Oatlet
in the Route 2 harvest fields and Cinderfin beside the North Shore Smokehouse.
The web Pokédex now follows installed encounter sources, conditional gifts and
evolutions, with missing acquisition routes labeled explicitly. Historical
stage counts below describe their own packs; resolve `world.json` for the
current selected lineage and remaining work.

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

The current [indoor fossil lab](../../landmarks/fossil-lab-v01/README.md) adds a
signed native room beside Rochester's clinic. Both specimen gifts, Care Badge
and full-party gates, two real exits/reentry, caught Dex and Continue pass.
The live web guide now directs both fossil families to this lab. The older
stage counts below describe their respective historical parents.

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

## Exact V22 candidate witnesses

The unselected full campaign now has a source-labelled candidate path for each
of225 active species. On exact V22 content
`dcb19c6e85835fb0e93ba7e44aa31a443c29b6018c9e182abf7f76ba9f3d9275`,
141 ordinary habitat/fishing candidates cover201 species through level
branches. The remaining24 species depend on16 special roots: early research
partners, both fossil restorations, Burrbud Headbutt, Castorock, the two Tall
Tale discoveries, Kettlurk, Pepivern, Icemyth, Kensigard, Flarkindle, Strandrak,
Vergloom and Verdigrim. The two Tall Tale roots share one native investigation
course; research tests cover four partner/fossil roots.

Ignored ledgers `.local/regional-acquisition-ledger-v22-refresh-v01.json` and
`.local/v22-acquisition-witnesses-v01.json` retain each exact source label and
evolution chain. Every witness explicitly has actual arrival/transaction
verification false. Generous map/surface reachability, conditional programs and
positive tables do not prove actual access or a complete collection. Targeted
native captures and prerequisite interactions are queued for the special roots;
ordinary habitat/rod/field replays and the final earned campaign are separate.
No fixture-backed caught records are presented as earned collection progress.
