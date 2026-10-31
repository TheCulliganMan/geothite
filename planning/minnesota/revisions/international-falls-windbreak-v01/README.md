# International Falls winter windbreaks

Status: native acceptance passed. The active local selection is recorded in
`planning/minnesota/world.json`.

The old snow court gives way to two short circuits. The upper brick service
lane connects the original clinic and supply shop, with snowy pines between
it and the main street. The lower loop bends around a pine windbreak past the
thermometer keeper and librarian. Snow shoulders retain the winter palette;
ordinary walks remain two gameplay tiles wide. The complete town stays
16×14 metatiles, using its existing native art and four original residents.

`before.json` binds the installed terrain; `after.json` contains every new
metatile row, a purpose statement and twelve player-camera locations.
`contract.json` binds the complete original native module, four service warps,
three boards and original actor anchors. Existing stories, room residents,
clinic care/PC/recovery, priced shop stock and the border-circuit reward are
retained from [the original town dossier](../../towns/MnInternationalFalls.json).

The actual regional connection is **south to MnR17C00R00**, offset−8.
Town `(20,27)` and `(21,27)` join route `(36,0)` and `(37,0)`.
The eastern local street ends at snowy pines: `(27,14)` is walkable and
`(28,14)` blocks movement. The old planned eastern port is not a playable
shortcut. The original ranger circuit still travels through Bemidji, Ely,
Rainy River and Voyageurs; the clinic's check-in and regional guards remain.

The optional Skier MARIT stands in the little snow yard at `(17,12)`, facing
south; approach from `(17,13)`. `trainers.json` defines the exact party,
dialogue and defeated flag. LUTEBRISK39 is pure Ice; FENCICLE40 is Ice/Fighting.
Native regional trainer scaling and legal level evolutions remain inherited.
The player must accept the offer; she never blocks the service lane or forces
a sight battle. Declining changes no money, party or story flags. Winning uses
the actual native battle/prize path and persists only her own defeated flag.

Review the whole town before accepting native publication. The preview watcher
uses the installed native palette and art without rebuilding packs:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-r10-survey-spurs-v02.crystalpack \
  --source planning/minnesota/revisions/international-falls-windbreak-v01/after.json \
  --output planning/minnesota/renders/international-falls-next-source --palette day --watch
```

Use a fresh output folder. All earlier whole-town images and rejected previews
remain retained. The first warm source pass took0.064s; changing the secondary
paths to existing brick rather than greenish gravel rendered in0.033s.

Acceptance requires every ordinary floor and real service approach reachable,
physical pine barriers at `(12,12)` and `(13,19)`, and the original closed east
edge. Native controller cases must walk both loops, refuse and accept MARIT,
observe spent move PP/prize and Continue, and verify that her battle does not
grant the lighthouse signal, ranger report, Voyageurs arrival or circuit reward.
Rerun all four original town cases, including the real long border circuit,
free inbound care, both room carpets, southern lanes and full reward-pocket
retry. Also require fresh stdio input, native images and exact preservation of
every other map, all native source art and all audio programs.

Current acceptance is in
`planning/minnesota/renders/international-falls-windbreak-v701/acceptance-v744.json`.
All six default-profile controller cases pass: both new windbreak/Skier cases
and all four original town cases. All twelve day cameras match every native
source pixel. Two unmodified reached saves were rendered and inspected at their
actual night-time state. Production stdio replays pass344calls with empty stderr.
The other732maps and all original native art/runtime files are exact; all377audio
programs pass PCM hash, frame-count and loop-range checks.

The complete original border circuit passes, including actual far-camp arrival,
the real Cedar gatehouse on return, full-pocket PP_UP refusal/retry, once-only
reward and Continue. The read-only planner now follows native bridge `LADDER`
collision, directional ledges, temporary inward-checkpoint movement and the
real directional gate carpets. It uses native Run for wild encounters and
never writes live position or flags. The gate carpets activate when leaving
the standing carpet; treating entry onto one as an immediate warp was incorrect.
The actual reached save and fresh production client confirmed that behavior.
All rejected diagnostic runs and images remain retained.

The border case explicitly supplies eight badges, the lighthouse signal,
Headwaters report and completed upstream mine custody/handover as local setup.
It carries an active Minnesota partner with Smoke Ball and Repels. It does not
supply the border report, far-camp arrival or PP_UP reward: those are earned by
native dialogue, travel and reward handling. This proves the local quest and
connections; it does not claim an earned main campaign.

The shared Pack target-label lookup used a filtered battle-usable list while
its cursor indexed the complete visible Items pocket. A Repel preceding a
Hyper Potion therefore broke the target prompt. The lookup now uses that same
complete pocket order. The new battle case starts with an explicitly injured
regional lead and asserts real healing-item consumption as well as native PP
and prize changes. All-target assets/Bevy Clippy passes with existing warnings;
workspace formatting still reports unrelated differences beginning at
`crates/offset-allocator/src/lib.rs:408`.

Executed guarded native publication (the output remains ignored):

```sh
target/verdigrim-scene/debug/rebuild_international_falls_windbreak \
  content-packs/minnesota-r10-survey-spurs-v02.crystalpack \
  8ae24f320b31423a8824d943c98bdd5548aac2fecc9394e7679bcd48a0db5b33 \
  planning/minnesota/revisions/international-falls-windbreak-v01/before.json \
  planning/minnesota/revisions/international-falls-windbreak-v01/after.json \
  planning/minnesota/revisions/international-falls-windbreak-v01/contract.json \
  planning/minnesota/revisions/international-falls-windbreak-v01/trainers.json \
  content-packs/minnesota-international-falls-windbreak-v01.crystalpack
```

The native candidate hash is
`7066f05448101c79b4dc761963b0f39a570bad07c4f80e3e777901ac2e207507`.
Its reviewed revision key is `international_falls_windbreak_revision`. The publisher validates the parent
hash before and after, original native module, all source rows, original service
facades/boundary, complete reachability and strict runtime readback.
