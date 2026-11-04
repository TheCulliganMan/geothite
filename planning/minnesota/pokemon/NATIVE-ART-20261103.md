# November 3 native artwork checkpoint

The requested sprite regeneration pass is complete locally: 52 requested species
in 26 families, including all nine starters, the fully redesigned Rillbow/Torrout,
four-legged Crookveil, and two-eyed Siphlet/Obelurk. Original sources and earlier
revisions remain preserved. Casserogre stays on its original selected artwork.
This is an artwork checkpoint, not completion of separate typing, roster proposal,
story, map, or deployment tasks.

The registry in `sprite-generation.json` selects the exact versioned manifests.
Normal and shiny fronts/backs were reviewed at native size and integer zoom;
the independent verifier passed 118 views including seven preserved family stages.
The local candidate is ignored output:
`target/native-integration-20261103/reviewed-sprites.crystalpack`.
The existing `world.json` selection and public deployment were not changed.

## Verified local integration

The current map pack lacked the already requested Mudnip/Bankmaw restoration.
Its explicit family installation preceded the graphics refresh. Their profile
listed ROCK_SLIDE as TM/HM-compatible although the supplied Crystal pack does not
have a ROCK_SLIDE TM/HM; that invalid compatibility entry was removed for these
two species. Their learnsets and other compatibility entries were retained.

These commands were executed from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo build --locked -p crystal-assets --bin pack_minnesota_pokemon
mkdir -p target/native-integration-20261103
target/debug/pack_minnesota_pokemon content-packs/minnesota-reviewed-maps-v11.crystalpack planning/minnesota/pokemon target/minnesota-pokemon/native planning/minnesota/pokemon/species-profiles.json target/native-integration-20261103/restored-turtles.crystalpack --family MN_F132
target/debug/pack_minnesota_pokemon target/native-integration-20261103/restored-turtles.crystalpack planning/minnesota/pokemon target/minnesota-pokemon/native planning/minnesota/pokemon/species-profiles.json target/native-integration-20261103/reviewed-sprites.crystalpack --refresh-graphics MN_F001,MN_F004,MN_F005,MN_F016,MN_F031,MN_F043,MN_F054,MN_F060,MN_F064,MN_F083,MN_F084,MN_F090,MN_F092,MN_F093,MN_F097,MN_F101,MN_F102,MN_F121,MN_F125,MN_F126,MN_F132,MN_F133,MN_F134,MN_F139,MN_F141,MN_F142
```

The candidate passes runtime pack verification with 478 total installed species.
Its content hash is
`c4d477b7e6d42aef37dba84914316f13c9d8599a9da0bc37d98d33eefe76ce45`.
The graphics refresh preserves all game data, audio, installation reports and
runtime file keys exactly relative to the explicit restoration intermediate.
Only 200 graphics files change: the turtles already had their new art installed.
All 208 requested front/back PNG and 2bpp files match the selected manifest's
native files byte for byte. Compared with the original map pack, Casserogre,
all audio and all seven map data sections remain exact.

`graphics_refresh_preserves_installed_game_and_rejects_reinstallation` passes
against the external original pack. It also checks idempotence, unchanged data
and audio, and rejection of empty, duplicate, unknown or uninstalled selections.

## Battle-side verification

The verification-only `native_sprite_battle_fixtures` example makes legitimate
link-battle starts using the production shell, saves each result, then reloads it
through production validation. It never enters production startup or player saves.
The executed command covers every requested species on both sides in normal and
shiny appearances, producing 104 successfully reopened battle saves:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 cargo build --locked -p crystal-runtime --example native_sprite_battle_fixtures
target/debug/examples/native_sprite_battle_fixtures target/native-integration-20261103/reviewed-sprites.crystalpack target/native-integration-20261103/battles POUCHID,SLIPPERINA,CYPRIELLE,BIRCHIT,PORTABEAU,VOYAGEAR,REEDRAFT,ZIZALITH,STACKAJACK,SPLITBEAM,WRECKQUIEM,PIPBOOM,BOOMRUFF,PRAIRONDO,LAKERNAUT,CHORIP,CHOIRBOG,BRINEBRAZE,BARBELISK,DOUGHLIN,CRUMCONE,PAILBARON,TRILUME,PANOPLITE,SIPHLET,OBELURK,PINGLET,PADDARRAY,ZAPSAP,TAPACITOR,BEETYNAMO,EMBERM,SMOULDEN,CINDERFEN,RILLBOW,TORROUT,KINDLIT,RIMEPELT,BOREALYNX,KENSIGARD,MUDNIP,BANKMAW,BURROOT,WADEBUD,RAKEHORN,MARSHRACK,CROOKIT,CROOKVEIL,VELVAWN,SHEDSTAG,DEWBIT,GLISNARE
target/debug/examples/render_saved_session target/native-integration-20261103/reviewed-sprites.crystalpack target/native-integration-20261103/battles/SIPHLET-OBELURK-normal.crystalsave target/native-integration-20261103/wizards-normal.png --2d
```

Eleven actual native 2D battle captures were visually inspected: both wizard
fronts in normal/shiny palettes, the three starter families, trout, deer,
wild-rice and restored turtle families. Eye whites remain distinct from shell
highlights on both wizards. Torrout retains its curled biological fish silhouette;
Shedstag has four body legs, with floating antlers separate from its anatomy.
Native pixel contours and dither remain visible without smoothing or gold tint.

These captures reuse the existing November 1 `render_saved_session` executable
with the new verified pack and saves. They establish artwork placement, not a
new engine release: that executable shows pre-existing battle text/menu clipping.
No claim of resolving unrelated battle UI or gameplay regressions is made here.
Generated packs, saves, captures and audit files remain ignored review material.
