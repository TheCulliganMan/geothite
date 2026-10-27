# Grand Marais: town-only slice

This is the town-specific review loop. Preserve its tested layout while the separate [North Shore](../routes/MN_R11.md) and [Gunflint](../routes/MN_R12.md) route work proceeds. Their native overlays and focused fixtures now pass; the full twenty-route objective and regional integration remain tracked in [the regional status](../STATUS.md). Town visual acceptance remains open.

[grand-marais.json](grand-marais.json) is the authoritative layout and dialogue. Its explicit metatile rows, six door anchors, seven residents, three signs and four authored interiors are consumed by both the Rust builder and the fast source-art preview. The `dialogue` entries contain all eleven authored indoor text bodies; exterior residents use `lines` and optional `signal_lines`. Keep authored lines within 18 characters and compose each pair as one page. The exterior is 20×19 metatiles (40×38 gameplay tiles). Coordinates on events are 16px gameplay tiles; rows contain 32px metatiles. No geographic generator runs.

The composition follows the harbor, Artists' Point and lighthouse breakwater described in [Visit Cook County's Grand Marais map](https://assets.simpleviewinc.com/simpleview/image/upload/v1/clients/cookcountymn/FINAL_Grand_Marais_Map_2sided_updated_2024_18553956-0a31-456e-acac-430155094598.pdf), translated into the supplied pack's Johto tile language. It is a game layout, not a street-for-street geographic reconstruction.

## Fast review loop

The existing local environment has Pillow and cbor2. Start from the repository root:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_grand_marais.py --watch --serve
```

Open `http://127.0.0.1:8765/`. Edit the JSON and save. After the source tiles are cached, measured edits rendered in 40–93ms. The browser polls for changes every 300ms and preserves the selected layer and zoom. No Cargo invocation, synthesis, pack export, or terrain generation occurs during this loop.

The review includes terrain-only and resident views, a collision/door overlay, 160×144 pixel viewport crops at integer zoom, and the four authored room layouts. Each edit checks all six door collision types and arrivals, town traversal, resident footing, reachable talk positions, physical sign anchors, and room exit carpets. Invalid edits must remain visibly marked; an old image is not evidence that a failed edit passed.

Every walkable exterior tile must belong to the same reachable area, excluding occupied resident tiles. This catches stranded decorative pockets as well as blocked required destinations. The west lookout has a clear floor behind its shoreline guardrail; walk to `(2, 22)` and return through `(6, 20)` using normal controls.

Generated previews live in the ignored `../renders/grand-marais/` directory. Their existence proves geometry and pixels only. Rebuild the pack and run the controller regression after accepting a layout revision.

## Build once, then republish quickly

The builder reads the design at runtime. Editing streets, furniture, door coordinates, signs, or any authored dialogue does not recompile Rust. Native script logic stays in Rust. Missing, unused and multiply authored dialogue labels fail the build before publication.

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo build --profile wasm-dev --locked -p crystal-assets --bin pack_grand_marais
./target/runtime-fps/wasm-dev/pack_grand_marais \
  content-packs/text-tui.crystalpack \
  planning/minnesota/playable/grand-marais.json \
  content-packs/minnesota-grand-marais.crystalpack
```

The Rust builder adds `MnGrandMarais`, its field station, Pokémon Center, Mart, Folk School, fisher's cottage, and lighthouse. The four authored interiors use distinct laboratory, traditional room, house and lighthouse tile languages with original furniture arrangements. Their rows and exit carpets can change without recompiling Rust. The Center and Mart retain native service layouts. The builder retains source art and audio in the supplied pack. It redirects HOME spawn 0 into the field station for this isolated demo and retains the remaining 27 original recovery destinations. The town has its own landmark and persistent scene/event constants. No pack or extracted source assets belong in Git.

There are no active regional connections. Grass in the town's pine groves uses native encounters. Source healing, PC and Mart behavior runs through the same production controller as normal play. The center's unused upstairs stair is removed.

For composition with other routes, append `--preserve-home` to the publishing
command. This installs the identical town and services while keeping every
existing HOME/recovery destination. The current twenty-route composition uses
that mode; see [region-review.md](region-review.md) for its executed commands and
remaining hub/earned-progression limits.

## Local story

1. Begin in Dr. Tamarack's field station. Both exit lanes intercept an unprepared player.
2. Receive Totodile at level 8, five Poké Balls, three Potions and a spare signal part. The gift is persistent and one-time.
3. Explore the civic street, Folk School, fisher's cottage, harbor landing and Artists' Point.
4. Bring the spare part to the lighthouse keeper. Delivery consumes the part, sets `EVENT_MN_GRAND_MARAIS_SIGNAL`, and grants one Super Potion once.
5. Return to Tamarack for her completion dialogue. Optional grove battles, healing, shopping and saving remain available.

If the item pocket is full, the keeper refuses the delivery and leaves the signal part with the player. Free an item slot and retry. Repair consumes the part and awards exactly one Super Potion. The harbormaster, student, harbor child and painter then use their `signal_lines` dialogue from the same JSON; the change survives Continue. Other residents keep their local navigation and encounter guidance.

## Resolved connections and story anchors

Coordinates below are gameplay tiles, not metatiles. Each exterior door targets interior warp 1; both indoor carpet lanes return to the numbered exterior warp. The production controller owns arrival direction, trigger suppression and stepping out.

| Exterior warp | Door | Clear exterior approach | Interior map | Exit carpet lanes |
| --- | --- | --- | --- | --- |
| 1 | `(6, 7)` | `(6, 8)` | `MnGrandMaraisFieldStation` | `(2, 7)`, `(3, 7)` |
| 2 | `(15, 9)` | `(15, 10)` | `MnGrandMaraisPokecenter` | `(3, 7)`, `(4, 7)` |
| 3 | `(21, 9)` | `(21, 10)` | `MnGrandMaraisMart` | `(2, 7)`, `(3, 7)` |
| 4 | `(9, 15)` | `(9, 16)` | `MnGrandMaraisArtistHouse` | `(3, 7)`, `(4, 7)` |
| 5 | `(27, 15)` | `(27, 16)` | `MnGrandMaraisFisherHouse` | `(2, 7)`, `(3, 7)` |
| 6 | `(15, 27)` | `(15, 28)` | `MnGrandMaraisLighthouse` | `(4, 9)`, `(5, 9)` |

Tamarack and the keeper stand at `(5, 3)` in their respective rooms; talk from `(5, 4)` facing up. Read the three physical signs from immediately south of `(10, 10)`, `(12, 14)` and `(26, 18)`.

In the Center, use the nurse from `(3, 3)` facing up and the PC from `(9, 2)` facing up. In the Mart, speak to the clerk from `(3, 3)` facing left across the counter. These use the production service menus and return to ordinary walking when closed.

| Resident | Town tile | Before repair | After repair |
| --- | --- | --- | --- |
| Harbormaster | `(7, 17)` | Fog hides the reef; keeper needs a part. | Beacon turns; boats can find the harbor gap. |
| Student | `(12, 11)` | Directs players to Tamarack and services. | Reports that Tamarack heard the good news. |
| Harbor child | `(19, 13)` | Boats are late in the fog. | Spots a returning boat. |
| Painter | `(30, 29)` | Describes Artists Point and the breakwater. | Adds the beacon to the harbor sketch. |

All four reactions check `EVENT_MN_GRAND_MARAIS_SIGNAL`. They grant no additional items or progression flags. The keeper's repeat conversation confirms the repair without duplicating the reward. No regional gate opens in this isolated slice.

## Controller and client checks

```sh
CRYSTAL_MINNESOTA_TEST_PACK="$PWD/content-packs/minnesota-grand-marais.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked -p crystal-bevy --test grand_marais
CRYSTAL_MINNESOTA_TEST_PACK="$PWD/content-packs/minnesota-grand-marais.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked -p geothite --test grand_marais
CRYSTAL_MINNESOTA_TEST_PACK="$PWD/content-packs/minnesota-grand-marais.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked -p crystal-bevy --lib \
  grand_marais_keeper_retries_a_full_bag_without_losing_the_part_or_reward -- --ignored
```

The controller regression starts a real new game, checks dialogue input ownership and page advancement, obtains the starter, reads all three physical signs, walks the west lookout, crosses each door and return, completes the persistent delivery once, checks all four changed resident conversations, enters a real wild battle, executes a move with PP mutation, heals at the nurse, opens and closes the PC menu, buys from the clerk with money deduction, and saves/continues the same town state and resident reaction. The stdio regression checks the MCP move/press/observe path and empty stderr. The separate full-bag regression deliberately initializes a capacity fixture, then uses production controller inputs to verify refusal, intact inventory, successful retry and a single reward; it is not a fresh-game playthrough.

See [STATUS.md](STATUS.md) for executed results and the remaining visual review.

For local play after building the production text client:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo build --release --locked -p geothite
./target/runtime-fps/release/geothite play content-packs/minnesota-grand-marais.crystalpack --save .local/grand-marais.crystalsave
```

Use the same save slot with `--load .local/grand-marais.crystalsave --save .local/grand-marais.crystalsave` to Continue. During layout iteration a new pack identity may require a fresh game; retain older saves rather than overwriting them.

The native graphical client can also load this pack; its one-shot captures must use the repository's headless image-capture settings. Visual review is of the 2D town. Keep this town review in 2D; separate route work is tracked in the regional status.

Render the complete published exterior and check it against the design with:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_grand_marais_pack.py
```

This writes a full native-scale image, a 2× image, an image with a content-based filename for stable review links, and a contact sheet of all six published interiors with residents. It checks the authored rooms and their exits as well as the exterior. It does not crop or stitch screenshots.
