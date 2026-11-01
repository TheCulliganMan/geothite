# Full v27 story consolidation on the current game

New source leaves: `crates/crystal-assets/src/northstar_story_consolidation.rs`
and `crates/crystal-assets/src/bin/pack_northstar_story_consolidation.rs`.
Task 6 owns registration, Cargo validation and pack publication. This authoring
task did not run Cargo or change shared registration, selection, existing packs
or saves. Source formatting passes; compilation and gameplay remain unverified.

Add these lines to `crystal-assets/src/lib.rs`:

```rust
mod northstar_story_consolidation;
pub use northstar_story_consolidation::consolidate_northstar_story;
```

The publisher pins the exact current passing-width/fauna identity
`544539b3c51802c03cc284ae7ec3d683ea251596af80c71ef6f886ef4888aa16`,
the exact Minneapolis cycling common parent and exact v27 donor. It verifies
the reviewed 655-program / 57-map delta. Unknown conflicting programs fail;
the new Verdigrim `MnArcIntakeStop` remains current. Current train/research/
containment programs and callbacks are explicitly checked for preservation.
Private cast classes change without replacing current teams, AI or rewards.
The new introduction flag is allocated beyond existing event IDs. Existing
IDs, maps, geography and audio remain, with the five reviewed indoor gray-fence
cells as the sole terrain change. The already aligned current League floor
must be present; the publisher never rolls back current coast geography.

For unchanged actual attributes/blocks, publication restores each exact current
split entry, including a missing entry or retained legacy alias. The only split
attribute edit is the holding-room tileset name; only that room's split blocks
may change. Separate whole-table checks enforce these bounds before the broader
catalog comparison. A self-contained regression covers the stale R08 tileset
alias and accidental insertion of a previously absent split entry.

After registration, the integration owner can run:

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets \
  --bin pack_northstar_story_consolidation
target/debug/pack_northstar_story_consolidation \
  content-packs/minnesota-north-shore-passing-width-fauna-v02.crystalpack \
  content-packs/minnesota-minneapolis-citywide-cycling-v02.crystalpack \
  content-packs/minnesota-dialogue-overhaul-v27.crystalpack \
  planning/minnesota/revisions/story-consolidation-v01/writing.json \
  content-packs/minnesota-story-consolidation-v01.crystalpack
cargo test --locked -p crystal-assets northstar_story_consolidation
cargo test --locked -p crystal-assets \
  consolidation_keeps_current_services_and_all_three_authored_opening_scenes \
  -- --ignored --nocapture
cargo clippy --locked -p crystal-assets --all-targets
```

Use task 6's coordinated `CARGO_TARGET_DIR`; the executable path above assumes
the default target. Choose another fresh ignored output if v01 already exists.
The ignored external-pack test reads its three prerequisites at runtime and
proves source/data integration only. Controller and audible acceptance remain
separate requirements, including the whole earned campaign.

## Bounded prose resolutions

`writing.json` owns an exact union of **24 conversations** for this consolidation.
The first twelve resolve the current integration context. Eight
resolve actual current context: farmer stairs/grammar, Tamarack's early starter
research referral, the functioning rail timetable/clerk, Ivy/Finn's three
containment controls, and the two Leech Lake hints that combine the stronger
v27 investigation with current island/path directions. Four explicitly retain
the accepted v27 Tamarack introduction, adult Reed arrival, envelope receipt and
harbor directions verbatim. All pages remain one/two lines of at most18 glyphs.

The additional twelve are semantically recovered from the exact patch at
`/Users/ryanculligan/Documents/Codex/2026-10-31/task/story-audit-evidence/northstar-writing-improvements.patch`.
They preserve Jo's supper-table aftermath, Ellis's next lead, Reed's need for
work and promised promotion/stronger team, Sera helping the other shift through
clinic staff, and clear officer/depot evidence. The officer says the copied
reports cannot establish each lake's level, avoiding the unsupported claim that
those places were never measured. The farmer explicitly says
`The east steps / reach the bluff.`

Those twelve recovered labels and the farmer correction are synchronized to
`story.json`'s canonical dialogue. Eight existing rival-arc source labels are
synchronized too. The older patch's intermediate milestone arrays, flattened
harbor monologue and static test-count edit were not blindly applied: v27's
staged evidence/reaction/directions and existing chapter contracts remain.
The Rust guard names all24 exact map/label pairs; it does not merely accept a
larger arbitrary replacement count. The fixed655-program donor delta is unchanged.

The earlier handoff remains
`.local/dialogue-overhaul/integration-handoff-v01/HANDOFF.md`. The new read-only
anchor/conflict preflight is `consolidation-preflight.json` in that directory.
The public-road redesign in `playable/travel-gate-overhaul.md` remains planned;
this consolidation preserves current access rules and working railway service.
