# Iron Range pump maintenance gallery

The whole 16×12 floor changes from a facility crate maze to a native rock gallery.
The western climb reaches the pump-guard shelf; the eastern corner stair descends
into the intake passage and feed console. Both guards control the first approach.
Accepting the original feed-isolation choice opens the drill-core shutter and a
useful service return. The ordinary return remains available when refusing.

Native cave30 and corner36 use the existing `mn_iron_haulage_rock` art and night
palette. The original numeric13 callback remains ordinary cave floor. No pixels,
graphics, palettes, actors, parties, warps, scripts or audio are replaced.
The dry floor's unused fishing reference becomes the active Minnesota lake group.
The original two reciprocal shaft pairs and safe passive arrivals remain intact.

`plan.json` is the gameplay contract; `before.json` and `after.json` are whole-room
terrain sources. `contract.json` freezes the complete original map, actors,
ports and control. `current.json` is the native source for shared reservations.
Resolve the active `world.json` source index before any new publication.

The standalone Rust publisher is `rebuild_iron_pump_gallery`. Its arguments are
`PACK SHA256 BEFORE AFTER CONTRACT FRESH_IGNORED_OUTPUT`. It validates the exact
current module and terrain, two sealed openings, both unavoidable guard sights,
connected floor, dry arrivals, solid boundaries and useful return shortening.
The current R15-parent native revision was built with the pinned toolchain:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_iron_pump_gallery
planning/minnesota/renders/iron-pump-gallery-v561/publisher-v579 \
  content-packs/minnesota-current-r15-open-shore-and-spur-v02.crystalpack \
  e02fb85d4db327d146cee269809b1af3e4b387dc2e29240b71cae0ae5837ec4c \
  planning/minnesota/caves/iron-pump-gallery-v01/before.json \
  planning/minnesota/caves/iron-pump-gallery-v01/after.json \
  planning/minnesota/caves/iron-pump-gallery-v01/contract.json \
  content-packs/minnesota-iron-pump-gallery-v02.crystalpack
```

Use `review_cave_source.py --pack PACK --source .../after.json --output FRESH --watch`
for fast whole-floor iteration. The measured warm source render is 0.058 seconds;
each edit keeps immutable original and 2× images. Source preview, native camera
comparison and actual control verification are distinct checks.

`audit.py --pack PACK --output FRESH` independently compares the complete parent,
all other 731 maps, every existing runtime file and audio program, retained native
stair pixels, guard parties and active regional evolution closure. It proves a
44-step closed course and 24-step open return, a sealed core before isolation,
solid boundaries and both required guard sight crossings. Invalid open shutters
and unconnected edge variants are rejected before any pack is written.

The opt-in controller case `iron_pump_controller_native_...` requires
`CRYSTAL_WINONA_TEST_PACK`, fresh `CRYSTAL_IRON_PUMP_REVIEW_DIR` and separate
`CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR`. Initial setup explicitly qualifies the prior
foreman chapter, badges, supplies and four level 55 original Minnesota partners
at the existing mine mouth. No local pressure, pump guard, feed or core guard
result is supplied. Subsequent travel, battles, choices and saves use production
input only. The original core-west guard sees the return ladder; its real battle
must resolve before retrying that shaft. No witnessed drill, rival victory,
shutdown or recovered instrument is granted by this course.

`check_mcp.py --pack PACK --save REACHED_DECLINED_FEED --client CLIENT --verifier
READ_ONLY_NATIVE_SAVE_VERIFIER --output FRESH` separately plays refusal, acceptance,
both physical openings, both climbs, both shaft pairs and the original core-return
guard, then returns to the mine mouth. It uses a real reached same-pack checkpoint,
three client processes for Continue, zero Surf moves and byte-empty stderr.
The read-only verifier and retained all-call trace verify the native core guard
result and actual move PP spending. This is qualified local access and geometry
coverage; it does not establish earned regional arrival or intended-level balance.

Nine native default-state cameras cover arrivals, both stairs, the upper shelf,
guards, feed and both shutters. All 368,640 pixels per view match their source.
The four-floor overview renders actual current rooms at one native scale. Its
nine lettered bindings are real reciprocal shaft pairs, including the core link;
no walking terrain is invented between floors. Recreate it with
`review_iron_workings_network.py --include-pump --pack PACK --output FRESH`.

[Whole before/after](../../renders/iron-pump-gallery-v561/audit-v594/whole-before-after.png),
[whole native cave](../../renders/iron-pump-gallery-v561/audit-v594/after-2x.png),
[native camera views](../../renders/iron-pump-gallery-v561/native-v594/MnIronPumpGallery-production-cameras.png),
[four connected floors](../../renders/iron-pump-gallery-v561/four-floors-v594/connected-native-floors-original.png).

The R15-parent child passed the complete controller case and a separate 846-call
stdio replay with byte-empty stderr. Nine native cameras have zero pixel
differences. All 377 audio programs pass PCM hashes, frame counts and loop ranges.
All-target Clippy passes for crystal-assets and crystal-bevy with existing
warnings; new Rust leaves pass formatting. Whole-workspace formatting retains
existing differences beginning at offset-allocator/src/lib.rs:408.

The actual core guard wins through native input, and the save verifier observes
Crunch PP falling from 11 to 7 during the stdio course. The
[played shutter frames](../../renders/iron-pump-gallery-v561/shutter-states-v596/actual-before-after.png)
show both reached states. The current source index and active selection are in
world.json; [acceptance](../../renders/iron-pump-gallery-v561/acceptance-v597.json)
records this local stage. The drill-core room remains a separate quality task.
