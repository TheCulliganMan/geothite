# R20 Voyageurs Border Loop — unselected grass candidate

Five floor-to-grass metatiles close two useful gaps. The sheltered camp's ordinary
uphill return now crosses six grass steps while its native ledge drop stays dry.
The western meadow crossing changes from zero to three required grass steps in
both directions. Actor-aware collision graphs confirm those costs. Grass increases
880 to 900 walking quadrants. Original ledge/wall/water/Cut/warp blocks remain exact.

Typed Rust publication writes minnesota-current-grass-r20-v01.crystalpack from
selected R19. Parent identity is pinned in parent.json. World selection and canonical
sources remain unchanged. The independent audit passes 762 checks and four negative
mutations: only five cells across two native chunks and their encoded mirrors change.
All other complete maps, actors/events/scripts, encounters, catalogs and art/audio
remain exact. Eleven installed trainer/wild species bound into the review dossier
are checked against the active original Minnesota roster.

The actual Officer battle/PP/victory/Continue case and current cave-quality
cameras/stairs/hops/mouths/pickup/Continue case pass. The older outdoor geometry
fixture accidentally entered the current inland ice-cave mouth while planning
another walk. Its local planner now lists that existing mouth at (70,33); no game
warp or geometry was changed. Rebuild/replay of that fixture remains required.
A new populated test covers the six-grass camp climb, dry ledge descent and reverse
denial, an actual new-grass encounter/native RUN, western meadow crossing and
party/bag/flags/location through Continue. It has been authored and formatted but
has not yet been compiled or run. Existing local party/badge/position fixtures are
setup, not evidence of earned regional access or battle balance.

The full source route, both source feature crops and fresh native 2D captures were
inspected. All 2,048 metatiles and 2,097,152 native/source pixels match. All source,
raw/native and alternate captures are retained. The camp camera and western east
approach are the primary native views; the original grass-centered western view is
also retained. Audio verification passes all 377 programs. Coast maps remain zero
ICE with no scripted block changes, preserving the established no-ice fix.

Other live Cargo/rustc jobs on the shared host deferred our test rebuild; no other
job was interrupted and no competing rebuild was started. Revalidate live host
processes before starting the serialized two-job target/route-grass rebuild.
Pending: updated outdoor and populated test replays, affected Clippy, production
stdio from the newly reached save, and guarded canonical/selection acceptance.
Other outdoor regional links/parks and final current-story coverage remain in scope
after R20. This candidate does not complete the statewide objective.
