# Brainerd current exterior and Mart roof repair

The selected parent had a native tree block (`05`) at Mart roof metatile
(10,9). The source and candidate replace it with the original Johto right roof
(`19`). All four completed Mart blocks match original Johto pixels. The initial
seven-view native review is retained as rejected: the tree-over-roof defect was
visible despite passing collision and doorway checks.

`roof-before.layout.json` materializes all 768 installed metatiles, including
the existing private stair alias and closed east gym cliff. `roof-after.layout.json`
changes only one roof cell. The canonical town layout receives the same single
point. No art pixels are edited. All collision quadrants, actors, events, warps,
731 other maps, audio and runtime files remain exact in the candidate.

The existing native cave12 stairs at metatile (10,20) are ordinary bidirectional
walking: upper (21,39), treads (21,40)/(21,41), lower (21,42). The tall gym east
cliff has wall tiles (25,38)/(25,39), with walkable upper lips (24,38)/(24,39).
The current dossier retires the historical high-cliff jump contract. Original
resident and sign dialogue remains unchanged.

Verification evidence:

- Parent native public roads/Surf, individual badge denials/free inbound, and all
  five buildings/both returns/Continue passed. Initial zero-test selector is
  retained and excluded from passing evidence.
- Parent production MCP stairs, reverse/side walls, Save/Continue and menus:
  351 calls, unchanged source/input saves and empty stderr. The repair preserves
  their complete collision/event state exactly.
- Candidate native all five buildings, ten return carpets and Continue passed:
  `.local/route-grass-review/brainerd-roof-child-doors-v01.log`.
- Candidate Mart and stairs GPU views were actually inspected; both have zero
  source/native pixel mismatches. Whole source before/after was inspected.
- Four invalid publisher input cases reject without output or parent mutation.
- Owned Rust formatting passes; repository-wide formatting has unrelated existing
  differences. Publisher build and all-target affected-crate lint statuses are in
  `mart-publisher-build-progress-v01.json`.

Reproduce the guarded Rust publisher with the retained executable:

```sh
.local/route-grass-review/portable-brainerd-tools-v01/revise_brainerd_mart_facade \
  content-packs/minnesota-iron-crosscut-rock-cache-v02.crystalpack \
  bc38b8920bdc6d5e3c0e18ddac283aa4087e0020d694861f277d10314a3875d1 \
  planning/minnesota/revisions/route-grass-v01/current-brainerd-successor-v01/roof-before.layout.json \
  planning/minnesota/revisions/route-grass-v01/current-brainerd-successor-v01/roof-after.layout.json \
  content-packs/minnesota-current-brainerd-mart-roof-reproduction.crystalpack
```

Output must be fresh. Packs, native captures, saves, build output and generated
verification material remain ignored. No commit or external publication is made.
Selection requires successful build/lint plus the inspected visual receipt;
`acceptance.json` and `post-selection-proof-v01.json`, when present, record the
completed guarded selection. This is local terrain/building verification with QA
party/badge fixtures, not earned campaign progression or whole-goal completion.
