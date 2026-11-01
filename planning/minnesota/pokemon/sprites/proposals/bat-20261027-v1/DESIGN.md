# Echolet → Sonatlas

**Psychic/Flying**, two stages, evolving at level 28. Echolet has 290 total base
stats; Sonatlas has 480: 70 HP / 45 Attack / 60 Defense / 115 Speed /
110 Special Attack / 80 Special Defense.

The proposal was drafted against the frozen 220-species baseline: thirteen
Psychic species, none faster than 95, and seventeen Flying species, none with
100 Special Attack. Restoring LINE087 raises current authoring to sixteen
Psychic species; the three slow sticky-note stages do not fill this proposed
fast role. This family fills a
fast special attacker role and an unused regional type pair. It stays fragile,
with five weaknesses and a meaningful Dark-type matchup. Its theme is a bat
that remembers cave routes through winter: small folded listening pockets
become broad three-rib echo-catching wing fans with sparse path-like folds.

The [Minnesota DNR bat guide](https://www.dnr.state.mn.us/mammals/bats.html)
grounds the long-eared bat, roosting and cupped tail membrane; the
[NPS echolocation guide](https://www.nps.gov/subjects/bats/echolocation.htm)
grounds acoustic navigation. Psychic memory and sensory wing folds are
original fantasy. A nighttime woodland-to-cave-mouth habitat is proposed;
no encounter is registered.

The built-in ImageGen source and [exact prompt](prompt-v1.txt) are retained.
The existing Rust native sprite converter produced independent 56×56 fronts
and 48×48 close battle backs, plus native 2bpp and indexed rows. Each Pokémon
has its own exact four-ink RGB555 palette; the native files have no gradients.
The [conversion recipe](conversion-v1.json) records crops, fits and palettes.
The [native review board](native-review-board.png) puts Echolet on the top row,
Sonatlas below, with fronts left and close battle backs right.

Existing Gust provides special Flying damage; Confusion, Psybeam and Psychic
support progression. Air Slash is a separate future move recommendation for
stronger special Flying damage, with a native flinch effect already supported.
It has not been added to the current 270-move pack. The
[full proposal](../../../expansion/20261027-bat-echolet-sonatlas-proposal.json)
records the type audit, growth, moves, weaknesses, ecology and source links.

These are saved design and native conversion candidates. Current authoring is
223 species after the separate sticky-note restoration; this family remains
unregistered, has no permanent numbers and does not contribute to that count.
