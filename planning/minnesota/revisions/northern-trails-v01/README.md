# Northern trails

At the east end of International Falls, a skier taking mail to the Rainy River warming hut offers the player a ride. At the hut, a second skier carrying clinic notes offers the return trip. Each guide waits for a yes or no and leads the player off. The trip is free, and each landing leaves room to walk back to the guide. The town and icefield retain their original positions on the regional map.

From the hut, follow the bank through the icefield and the old warden checkpoint. The ice slide and the longer walking route both still work. Route 20 then leads east to the western entrance of Boundary Waters. Crossing there with all eight badges, the Duluth harbor signal and a name in the clinic book registers the visit. The border ranger acknowledges the radio call. The Falls clinic holds the PP Up until the player returns with room for it. A player who registers early now hears which prerequisite is still missing.

Crystal's Olivine dock gives the sailor a visible place to stand and stages the departure at the boarding point. Polished Crystal keeps a physical boarding approach at its dock. Those scenes informed the guide's placement and the step off the road. The Minnesota guides, snow turnouts, dialogue and mail run were authored for this trip. [Crystal's Olivine Port](https://github.com/pret/pokecrystal/blob/master/maps/OlivinePort.asm) · [Polished Crystal's Olivine Port](https://github.com/Rangi42/polishedcrystal/blob/master/maps/OlivinePort.asm)

## Caves and prairie

- In the Inner Basin the three survey dials now share a recognizable handwheel. The final lever has a separate handle. Each occupies just its original 16-pixel interaction square, including the controls that can be operated from only one side.
- On Voyageurs' lower tamarack loop, the ranger directs the player to the split rock in the snow that actually holds NeverMeltIce. The original item, both water-free approaches and the response after collecting it are retained.
- Gunflint's hidden Revive is back in the solid spring pillar. The player can search it from either of its two water-facing sides. The shore approach, 112 connected water squares and 470 reachable dry squares stay intact.
- In the Driftless Lamp Gallery the fossil description now agrees with the western shell bed. It points out the kit on the eastern shelf only while the player can still take it. The Route 9 mine panel already has distinct responses for the running and stopped motor. The regression follows the second branch inside the isolated-feed response.
- On the farm road west of Rochester, Tess looks toward players emerging from town. The inside grass lane starts her existing battle with Podlet and Pinit. Walking one square farther out keeps the battle optional. Both walks between the city boundaries take 76 steps.

## Build and verify

Use the corresponding external Field or Mothlight parent and a fresh output path. The two build receipts give the exact product hashes. From the repository root:

```sh
stem=content-packs/minnesota-mothlight
# Use content-packs/minnesota for Field.
parent="$stem-campaign-quality-v01.crystalpack"
child="$stem-northern-trails-v01.crystalpack"
src=planning/minnesota/revisions/northern-trails-v01
CARGO_INCREMENTAL=0 cargo run --locked -j1 -p crystal-assets --example pack_northern_trails -- "$parent" "$child"
for audit in check-voyageurs check-cave-controls check-cave-journey check-tamarack-hint check-journey check-preservation; do
    uv run --with cbor2 --with pillow python "$src/$audit.py" "$parent" "$child" || exit 1
done
MN_NORTHERN_TRAILS_PACK="$PWD/$child" RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 \
    cargo test --locked -j1 -p crystal-assets --lib northern_trails_native_sight_and_guide_records -- --ignored
```

The checks compare both editions with their own parent. They use native geometry, parsed scripts, item and encounter tables, still renders and the production sight query. They do not establish the difficulty of a played battle or the timing of a played trip. Existing saves retain their original pack identity.

## Wider quality review

The hidden-item survey located 57 unique rewards on 47 Minnesota maps. It compared every event with its item, artwork, collision and nearby objects. More detailed approach checks confirmed the boat requirement for St. Paul's BlackGlasses, the Whirlpool gate at Leech Lake and the two legal approaches to the Voyageurs erratic. The review found no additional blocked pickup. It did not trace a complete trip from the starter town to each item.

The boss-team review covered all eight first-challenge gym leaders, four Elite Four members and the Champion. It compared first and last partners, base stats, installed moves, move categories and available route encounters. Two issues remain open. Fern's Dakotaskip and the lead on Saven's rematch team know Baton Pass, but the trainer AI does not supply a party target. Flint's Bandrake and Fortigard share both types and all four moves. The next battle pass should verify Baton Pass decisions with and without a living reserve, then give Flint's ace a distinct role using moves it learns.

The town survey checked all eight gym towns plus Winona, Grand Rapids, Grand Marais, International Falls, Ely and Boundary Waters. From the 68 public road entry squares it found an outdoor approach to each of the 163 street doors in its town once local road officers have cleared. Each door has an indoor arrival and a reciprocal exit. This establishes access through the town exterior. Interior puzzles and story locks have their own conditions.

One town direction remains misleading. In St. Paul the fair guard at (29,2) and the steward at (30,7) direct players south toward a public gate. The southern seam is water. The standalone fairgrounds entrance is on Route 5 north of Minneapolis. A revision should distinguish the nearby fair annex buildings from the road to the fairgrounds and give the actual first turn from each speaker.
