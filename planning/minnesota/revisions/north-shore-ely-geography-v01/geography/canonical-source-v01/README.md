# Detached canonical source candidates

These candidates encode the frozen `native-west-v02` geometry and preserve the
warm North Shore epoch. They are detached until native publication, controller
checks and visual review pass. `source-preimages.json` binds the current
canonical files and all three candidate byte hashes; promotion must reject any
changed canonical preimage and must not replace another chat's edits.

- `MN_R11.layout.candidate.json`: original warm source composition, native
  `mn_north_shore` alias, then the 306 exact terminal headland cells. Its two
  affected chunk outputs were independently compared against the actual native
  parent plus the frozen cell edits (768 and 576 cells). Other chunks inherit
  the preserved warm candidate unchanged.
- `MnMaraisCedarApproach.layout.candidate.json`: original Cedar source composition
  followed by explicit reviewed native rows. All 608 resulting cells equal the
  frozen Cedar rows, using the native winter alias and cold-pine art recipe.
- `MnMaraisInlandNeck.layout.candidate.json`: all 1,235 hand-authored native cells
  as supported point layers. Its 65×19 dimensions, winding trail, grass, ledges
  and campsite match the frozen contract exactly.

Terrain alone is insufficient for a source rebuild. Promote the matching native
connection attributes from `native-west-v02/contract.json`: R11 C00 north closed,
R11 C01 east to Grand Marais offset0, Cedar south to town +19 and west to Neck0,
R12 C01R01 east to Neck0, Neck west to R12/east to Cedar both0. Preserve other
regional footprints. The parent town supplies reciprocal west0/north−19 ports.
Retain Cedar actors/programs except its two exact sign corrections. The new map
requires group40 map8, copied Cedar encounters, Berry×3 pickup at63,27 with
fresh event2387, camp sign69,25, Grand Marais landmark and no retained recovery.
The Rust builder owns those native mirrors and validates inverse residuals.
Do not replace the older source epochs or their renders.

Verification here is source row equivalence, not controller or visual acceptance.

Promotion applies schema-only normalization to the Neck layout: `shore_fitting`
is `explicit_source_edges` and `reject_cliff_cuts` is `true`, as required by the
Rust authored-layout compiler. The frozen candidate stays unchanged; the script
rasterizes both versions and requires identical cells before writing.
