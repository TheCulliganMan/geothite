# R18 public shore-watch circuit — source proposal

This candidate connects the existing public-launch bay and eastern beach into
a walking circuit. A two-core protected shorewalk contrasts with the wider
brown-deck working marina piers; the northern dune road remains an ordinary
return and trainer/field alternative. The west marina, three SQ04 reports,
necessary habitat neck, open lake and optional Surf islet retain their roles.
No new HM, story actor, service, reward, art recipe, stair or ledge is proposed.
The active source and native pack have not changed. Parent visual review and
native testing are pending.

## Review and retained iterations

[Whole before/after](../renders/mille-lacs-shore/refinement-20261027/v04/before-after-whole-native32.png)
and [all fourteen player-size views](../renders/mille-lacs-shore/refinement-20261027/v04/candidate-decision-cameras.png)
show the entire40×24 route, both working piers, existing activities, new shore
connection, dune road and automatically selected weakest view. Native full32px
metatiles and original160×144 crops remain separate from the2× contact sheet.
These are exact source-art composites with actual day NPC palettes, standing
facing, tile top-left and bounded native wall-foreground masking; they are not
GPU captures or proof of runtime input/actor visibility.

The first prototype `v01` supplied a real connection but looked like a third
long working pier. It is retained as a rejected visual direction, along with
its failed preliminary bounded-delta assumptions. `v02` narrows the path using
existing paired rail halves and removes the broad deck rectangle. `v03` adds
the foreground mask, a weak camera and eleven adversarial cases. `v04` fixes
east/west labels in the diagnostic resource-price table without changing any
source byte or rendered terrain. R18 lists its west endpoint first, opposite
R15; the earlier equality comparisons were valid but their item-side labels were
reversed. All original inputs, rejected renders, full-resolution RGBA buffers,
conversions and crops remain immutable.

The complete layout, all decision views, new lake frame and weakest frame were
personally inspected. The new path gives a visible open-lake frame at core
`(65,18)`, containing30 WATER core cells in its native camera area. Its lower
rail correctly masks six foreground pixels over CHRIS. The widest remaining
all-Floor square is still the unchanged central lawn, core bbox
`[42,6,52,16)` with camera `(47,11)`. That ten-core region is shown explicitly;
this proposal does not claim that one new circuit completes all R18 quality work.

## Exact source and topology

Baseline pack: `content-packs/minnesota-northstar-lake-country.crystalpack`,
538 maps, file SHA256
`da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50`.
Previous dossier SHA256
`c8b56362ddd07924284e7b2004d1a566cfef02cf141e66d7afa0fbc1aed69e91`;
previous layout
`bb122560bb6f12cba044a9caa710c3146e63d58810ddcce674e05c13f4b535f5`.
Both were verified against actual native blocks and preserved under
`renders/mille-lacs-shore/refinement-20261027/before-da27/`.

Candidate inputs are `routes/MN_R18.shorewalk.candidate.json` SHA256
`21a5bd33acb1760380b1a94494728965f8760d267ee4f969f9bee7e795842c4c`
and `routes/MN_R18.shorewalk.layout.json` SHA256
`2b5b4a542513691e07dc35845c16af4c6f6a20bad879f3156ed76c9a29290618`.

Every existing dossier field is exact except adding one ordinary walking-loop
record, four camera records and editorial review metadata. All original records
in those lists remain exact prefixes. Existing SQ04 command/flag/party/reward
source is preserved, not relowered or rewritten. The layout keeps all original
metadata, native art/derived recipes and its complete layer prefix. There are
27 final metatile deltas: eight added dune-margin grass blocks, the narrow
rail/corner connection and the native water-fitting halo. The full exact delta
is in the source report. No existing grass quadrant is removed.

| Connection / condition | Exact core coordinates and result |
| --- | --- |
| New west foot entrance | `(57,18)` ↔ `(58,18)`; connects the old launch-side protected bank. |
| New east foot entrance | `(71,18)` ↔ `(72,18)`; connects the old beach bay. |
| New shore course | Central x60–69: rows17/18 FLOOR, rows16/19 WALL. Exactly two walking rows and exactly two ordinary entrances. |
| Shorter loop leg | `(59,18)` → `(72,18)` decreases from33 to13 walking steps with actor bodies present. Northern ordinary return remains available. |
| New ordinary cross-chunk seam | `(63,17)` ↔ `(64,17)` and `(63,18)` ↔ `(64,18)` are Floor across the existing reciprocal zero-offset neighbor pair. Actual native walking input remains unproved. |
| Existing required habitat | `SHORE_MEADOW_NECK` x36–41/y10–17 stays exact; west→east and east→west both still cost six grass steps. The new coast loop is east of that neck and cannot bypass it. |
| Existing public Surf launch | `(49,19)` FLOOR → `(49,20)` WATER remains exact. New shorewalk Floor has zero adjacent WATER gaps. |
| Existing optional islet | Revive `(61,35)` still requires Surf; no dry approach to the islet is added. |
| Existing water seam / perimeter | Both native neighbors and sea seam remain; all252 perimeter cells and actual south/east/west rock closures pass the original checker. |

The correctly labeled original optional-resource grass prices are unchanged:
Hyper Potion west0/east6, Ultra Ball west2/east8, Max Repel west6/east0.
Both original piers retain exact decks, rails, endcaps and approaches; all four
SQ04 actors remain reachable without Surf. The original badges, report events,
ranger reward logic, encounter tables, actors, parties and item flags are exact.

## Proof and native handoff boundary

[The current report](../renders/mille-lacs-shore/refinement-20261027/v04/review.json)
passes31 checks and rejects eleven saved negative sources: new third entrance,
Surf rail gap, closed gate, blocked course, unsafe corner, dried required grass,
free islet bridge, incidental Cut, moved report actor, changed reward flag and
changed native trainer party. It invokes the unchanged complete shore checker,
including the existing flat-shore contract. No shared validator was weakened.

Use the existing Rust same-extent `revise_authored_route_terrain` API after
source visual acceptance. The revised dossier adds a regular loop and views,
while its R18 constraints retain no Cut/ledges and one Surf-only pickup.
Certify the immediate base's R18 modules/art against this preimage first when
composing on later map overlays. Expected native changes are only blocks in
`MnR18C00R00` and `MnR18C01R00`, their uniquely owned encoded block payloads,
and derived identity. Every existing event/script/typed program, attribute,
connection, party, catalog, audio, art file, report, HOME/recovery and all other
modules must remain exact. Source planning does not prove that output exists.

After publication, exercise the new circuit in both directions through
`VisibleShellController::press`, including the actual walking seam, both
entrances, native upper/lower rail and corner denials, main-road return and the
old grass neck. Retain the existing SQ04 report/badge/GOOD_ROD/retry/Continue
cases and real Surf-islet/outer-boundary proof. Capture both ends, `(65,18)`,
`(65,11)`, the unchanged weak `(47,11)` and both original piers using the exact
native day palette/facing/tile top-left. Compare original buffers and foreground
pixels. Local initial fixtures do not prove earned regional story progression.
