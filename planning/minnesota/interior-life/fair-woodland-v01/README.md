# Three inhabited fair halls on a real woodland approach

The local compact composition now contains all 216 reserved non-gym interiors
plus the Rainy River warming hut: 217 rooms on 716 maps. This closes the three
missing fair rooms; it does not establish whole-region quality or completion of
the all-interiors objective. Native 2D only; Red Wing's station remains excluded.

The fair fits west of `MnR05C00R02`, northwest of the metro. Its former north
St Paul attachment overlapped the current river rectangles. The Rust publisher
retains the full geography guard instead of accepting that false placement.
Nine exact native pine cells, metatile `(0..8,4)`, become a two-walking-tile-wide
trail onto the existing road. Every old road actor, program, trigger, eastern
river link and north/south connection remains exact. Both new lanes play in
both directions: fair `(95,64/65)` to road `(0,8/9)`, offsets `+28/-28`.
The old fair southern entrance is visibly closed by pines. No portal or story
prerequisite bypass is added, and the city/river exteriors remain exact.

The three rooms have seven native interactions and six actual carpet returns:

- The grandstand host introduces a Bemidji folk performer and an Iron Range
  choir member, whose family stories give the waiting performers local lives.
- The barn's OATLET handler explains care and awards the remembered livestock
  stamp; the creature has its own grain-coat interaction.
- The dairy is an exhibit counter with a sculptor and a visible BUTTERMUSE
  carving butter. The inherited healing-machine/PC room is replaced by native
  exhibit furniture. Its dairy stamp remains independent of the barn and quiz.

All actual exits, arrival indices and exterior return coordinates have been
copied into the three canonical `interiors.json` reservations. Canonical fair
authoring and its `world.json` destination record now describe this physical
approach. The shared selected regional revision is preserved.

Verified Rust publication, from the repository root:

```sh
target/interior-life/wasm-dev/pack_state_fair \
  content-packs/minnesota-interiors-current-no-red-wing-v01.crystalpack \
  planning/minnesota/interior-life/fair-woodland-v01/layout.json \
  planning/minnesota/interior-life/fair-woodland-v01/dossier.json \
  content-packs/minnesota-interiors-fair-woodland-v02.crystalpack
```

That completed child has SHA-256
`f16fe1baea23a797c545f606af9f0e36fff176f86d22b2e31fb53ac2bec314e6`,
identity `10a23e13649a9aedb43edc33c2231a9bbd5c3822876bc9031ff9ee21f5c7939e`.
It retains all 255 exterior characters, 35 trainers, 32 shops, 16 food items and
14 authored habitats. Visible creatures, opponents and habitats use the active
Minnesota roster. The tests now use a legal Minnesota POUCHID fixture instead
of Mew. Initial party/money/prior optional victories are explicit setup; all
subsequent movement, text, rewards, purchases, food effects and battles use the
production controller.

Four controller cases pass on that exact child (163.04s): all room stories,
all six carpets and Continue; both walking lanes, stamps/quiz/prize, real battle
and first aid; every shop and all 16 actual food effects; and full-pocket prize
retry with persistence. Two additional cases verify the spread crowd's walking,
running, rotation and text (10.83s), and two native laps/talking/Continue for
four patrols (16.95s). The earlier 85-NPC assertion was stale; the actual
255-person contract is retained and verified.

The Rust deterministic replay/preservation/rejection case passes (17.60s),
including a fresh replay from the promoted canonical sources (15.19s).
All previous map modules stay exact except the nine cells and reciprocal
connection on lower R05. Original art files, species/moves/items, trainers,
encounters, HOME and audio are preserved. It rejects stale trail preimages,
wrong offsets/cells/materials, counter-blocked residents, classic creatures or
opponents, reopening the unconnected old gate, and duplicate installation.
The new fair rooms intentionally differ from the historical fair templates;
their original Johto template maps and every pre-existing game interior remain
intact.

Three whole-room native 2D captures match every pixel and were personally
inspected. The new road approach matches every pixel too. At the fair-side
camera, 358,358 static pixels match; 10,282 pixels cover authored moving-actor
rectangles and are reported separately (954 full-frame pose differences).
The untouched captures and full differences remain retained. Those actors'
motion is independently controller-tested above; do not call this a zero-error
full-frame comparison. The whole fair and both road views were visually reviewed.
The v02 metadata/sign update leaves all three room modules and source art equal
to the captured v01. All 377 audio PCM hashes, frame counts and loop ranges pass.

The final small direction overlay corrects the Capitol clerk's stale instruction
to use a northern city fair road, while retaining the records/Union Depot story:

```sh
target/interior-life/wasm-dev/pack_minnesota_interiors \
  content-packs/minnesota-interiors-fair-woodland-v02.crystalpack \
  planning/minnesota/interior-life/fair-woodland-v01/directions.json \
  content-packs/minnesota-interiors-fair-directions-v01.crystalpack
```

The corrected local child has SHA-256
`857f209c8bc55affea5c4344c8f9c175efda79826d05577e069f25516e8692e1`,
identity `40ab3d8aaec04cbf6c8d1f30c98bee10427e0f0ed9cd23e86b4718025473cb68`.
Only that text's typed body and raw payload change; all callers, actors, other
maps, catalogs, art and audio remain exact. No visible Red Wing reference remains.
The frozen 187-room story source is unchanged.

Evidence is under `renders/interior-life/fair-woodland-{accepted,coverage}-v01`,
`fair-woodland-rooms-native-v01` and `fair-woodland-road-native-v03`; logs use
`target/interior-life/fair-woodland-*`. Every rejected render and the previous
canonical sources are preserved. Packs and verification outputs remain ignored.

Changed Rust leaves pass formatting. Workspace formatting still stops first at
the unrelated `offset-allocator/src/lib.rs:408`; normal all-target Clippy stops
at the existing core `bad_bit_mask` denial. Both affected crates' supplemental
all-target check passes with `--no-deps` and the two existing denied lints
`bad_bit_mask`/`never_loop` allowed. Baseline warnings remain, with no diagnostics
in the new woodland/room leaves.

The inventory audit is intentionally narrower than completion. All reserved
rooms are present, but older return reservations still need reconciliation,
full visual/gameplay acceptance remains open, and the International Falls
regional circuit and five-stop passenger service are unfinished. The independently
selected branch has also advanced through new story maps and R03 on a 704-map
parent. Those additions must be composed without discarding these interiors or
rewinding the shared selector; this is no longer just the earlier R01 terrain
delta. Preserve the complete newer story branch and its reviewed programs.
