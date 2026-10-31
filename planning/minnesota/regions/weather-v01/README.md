# Minnesota outdoor weather and the inland winter belt

Boundary Waters remains green, including all six Safari chunks and the preserve
entrance. Keep open connected lakes, grassy portages, pines, campsites and the
Root Beer Lady. Do not recolor this area as winter. The native precipitation
policy explicitly excludes these maps from snow even if a future shared art
alias contains winter graphics. Ordinary occasional rain remains enabled.
North Shore and Grand Marais also stay warm. The user confirmed Bemidji and
Grand Rapids for the connected inland winter belt; that larger terrain revision
is pending, rather than delivered by the weather effect.

`plan.json` records scope and honest implementation status. Native policy lives
in `crates/crystal-bevy/src/bevy_shell/outdoor_weather_policy.rs` and the bounded
2D presentation in `outdoor_precipitation.rs`. One saved/synchronized clock
schedule controls twenty-minute shower windows across map boundaries, with a
twelve-second fade. Rain adds narrow drops and subtle blue-gray outdoor shade;
snow uses its own small flake pool. The effects hide in interiors, battles and
the title surface, and sit below dialogue/menu layers. They change no collision,
encounters, battle weather, progression RNG, save schema or pack assets.

Four actual Bevy policy tests passed in `.local/weather-policy-v995.log`,
including Boundary Waters snow exclusion with warm-area rain retained. The
bounded particle motion test also passed. The selected-pack read-only audit
found all eight Boundary Waters exteriors use warm native art and contain zero
ICE quadrants; its receipt is `.local/boundary-waters-no-winter-v995.json`.

Both untouched headless GPU screenshots in
`../../renders/boundary-waters-weather-v1001/` were visually inspected. The
fixed clock fixtures use Boundary Waters(6,24), Jan1 2000, 10:00 clear and 12:00
rain. Pines and lake remain green/open in both. Same-view light ground changes
from RGB222,255,222 to209,240,209 during rain. These are explicit visual fixtures,
not earned gameplay travel. The renderer is frozen at
`.local/weather-native-renderer-v995`; the capture receipt records its hash.
A subsequent behavior-preserving SystemParam/query-alias cleanup passes
all-target Clippy with no warnings in the weather modules. Scoped formatting
passes. Workspace formatting still has unrelated existing differences, recorded
in `.local/weather-workspace-format-v1002.log`. The earlier frozen executable
does not contain that parameter-only cleanup.

The larger inland winter terrain, ice/icy-grass gameplay, readable wet menus and
battles, Continue checks and full regional travel verification remain active
work. No completion of those features is implied by clear/rain screenshots.
