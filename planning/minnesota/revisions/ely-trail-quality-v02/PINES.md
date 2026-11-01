# Ely in the pines

Ely is a snowy clearing enclosed by native snow-covered pines, with smaller
groves between its lodge, clinic and supply shop. The published 20×18-metatile
town has 171 fully blocking pine blocks and a matching pine border. Every
original rounded canopy block (0f/32/33) is replaced; there are no incidental
Cut-tree quadrants. Keep the lodge's native building block37 intact.

The current exterior is the native overlay in
[source-v03/native.contract.json](source-v03/native.contract.json), published in
the ignored `content-packs/minnesota-ely-pine-knolls-v02.crystalpack`.
Its SHA256 is
`1f5604a6c743904e5b0c8fbfe36b32ef056f50dab309a42795aa639457ac0bf3`.
The older `towns/MnEly.layout.json` remains the Kanto building preimage;
rendering or publishing that file alone omits the accepted snowy pine art.

Use block83 in private tileset `mn_ely_snow` for the forest and border. Its
four core quadrants are WALL. Six imported native winter-pine glyphs occupy
slots96–101; original Kanto art and lodge rows remain intact. Do not substitute
the broadleaf canopy or a Cut gate for ordinary forest. Snowfall is confined
to Ely's outdoor exterior, with the warmer coastal end of the pass retained.

Preserve the original north port at core(24/25,0) to R13 and the east port at
core(39,14/15) to `MnElyShorePass`. Keep the paths to the lodge's two doors,
clinic and mart open. The new pass reaches the North Shore without the long
southern walk; the new Center remains south of Split Rock. League registration,
badges and the survey story retain their existing requirements.

## Review and verification

- [Whole town](../../renders/ely-trail-quality-v02/pine-review-v01/MnEly-whole.png)
- [Town, pass and North Shore at the same scale](../../renders/ely-trail-quality-v02/pine-review-v01/connected-whole.png)
- [Reached native pine gate with snowfall](../../renders/ely-trail-quality-v02/native-reached-v02/ely-pine-gate.png)
- [Fresh native audit](../../renders/ely-trail-quality-v02/pine-review-v01/receipt.json)

The whole-town and reached native images were inspected. Rechecking the current
pack confirmed all171 pine cells, the solid pine border, no old canopy remnants
and no Cut quadrants. Four focused production tests passed: the actual grass,
ledge, clinic and Save/Continue journey; a native Minnesota encounter; both
back-route seam lanes and Ely's original north exit; and bounded Ely-only
snowfall. The new controller checkpoints are retained in
`renders/ely-trail-quality-v02/reached-v07/`. These are explicit local fixtures,
not an earned campaign playthrough or final 3D visual acceptance.

The complete source loop, Rust publication and production test commands remain
in [PUBLICATION.md](PUBLICATION.md). The fresh audit used the same command with
`--epoch pine-review-v01`; the controller replay used the same pack/course and
fresh save directory `reached-v07`. Logs are retained as
`.local/ely-pines-review-{native,controller,seams,snow}-v01.log`.

The active `r10_rock_gallery_revision` descendant was independently rechecked
against its actual pack on October28. It retains all171 pine blocks and their
four WALL quadrants, the pine border and both reciprocal regional connections.
Ordinary walking from the north arrival reaches the east pass and every lodge,
clinic and mart entrance with the resident treated as an obstacle. This is a
static traversal check; the production-controller journey above is separate.
The fresh [whole-town render](../../renders/ely-pines-active-v01/MnEly-whole.png),
[headless native pine-gate view](../../renders/ely-pines-active-v01/pine-gate-native.png)
and [pack-bound receipt](../../renders/ely-pines-active-v01/receipt.json) are
retained together. The active pack pointer was not changed for this review.

The selected `r12_shelter_revision` pack was freshly checked as well: SHA256
`49b8121a4f1b0d00da34d7d93c61fffe784f097fbdfd682e15d9ecc9bb5993c2`.
Its whole-town native-art render retains all171 pine blocks and the blocking
pine border. Actor-aware ordinary walking reaches all eight active/passive
door records and both lanes of both regional ports. The north/east connections
are reciprocal. This check does not imply a new controller playthrough.
The [whole town](../../renders/ely-pines-current-v02/MnEly-whole.png) and
[pack-bound receipt](../../renders/ely-pines-current-v02/receipt.json) are
preserved in a fresh render directory. Recheck a later selected pack with:

```sh
PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/revisions/ely-trail-quality-v02/review_active_pines.py \
  --epoch NEW_UNIQUE_RENDER_DIRECTORY
```

This read-only loop renders the installed overlay directly, rather than the
older broadleaf Kanto preimage. Each output directory must be new.

The selected `current_story_revision` was rechecked against SHA256
`560e49986d14e48c07526e9878eddc69be46d546ad079774f04b10f58e7e5c88`.
It retains all171 blocking pine blocks, the pine border and clear approaches
to every service door and both regional ports. The production-controller
seam test passed on this exact pack, covering both back-route connections and
the original north exit in both directions (3.06seconds). Fresh
[whole-town](../../renders/ely-pines-story-current-v01/MnEly-whole.png) and
[headless native pine-gate](../../renders/ely-pines-story-current-v01/pine-gate-native.png)
renders are retained with their
[pack-bound audit](../../renders/ely-pines-story-current-v01/receipt.json).
The camera capture uses an explicit local position fixture. Logs are
`.local/ely-pines-story-current-{seams,native}-v01.log`; no new terrain or
pack selection was needed.

The selected `route_grass_r10_revision` was freshly reviewed against SHA256
`f9e3153964a966d610e1ec8c9ecfb45e980e80eab8ea48e336e061863747acd0`.
Its [whole-town render](../../renders/ely-pines-review-20261028-v03/MnEly-whole.png)
and [audit receipt](../../renders/ely-pines-review-20261028-v03/receipt.json)
confirm the same171 blocking pine cells, solid pine border, clear service
entrances and reciprocal north/east connections. The existing production test
binary passed the back-route seam and original north-exit round trips on this
exact pack (21.26seconds); its bounded Ely-only snowfall test also passed.
Headless native views use explicit local camera fixtures. All previous renders
remain retained. This review required no terrain or pack selection changes.

The selected `route_grass_r12_revision` was freshly checked against SHA256
`8c9a01edf629270392e43ae8761fd0ea78d57035b0cb4b097bf9eebc3fc38717`.
The [whole-town render](../../renders/ely-pines-user-review-20261028-v04/MnEly-whole.png)
was visually inspected. Its [pack-bound audit](../../renders/ely-pines-user-review-20261028-v04/receipt.json)
confirms all171 blocking pine cells, the pine border, no incidental Cut gates,
clear approaches to all eight door records and reciprocal north/east ports.
The existing production-controller test binary
`target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652` passed the
back-route and original north-exit round trips on this exact pack (31.58seconds).
Its Ely-only bounded snowfall test also passed (0.01seconds). These runs reuse
the previously built controller binary; they do not certify unrelated current
source changes. Logs are `.local/ely-pines-user-review-20261028-v04-{seams,snow}.log`.
No new terrain or pack selection was needed; previous renders remain retained.
