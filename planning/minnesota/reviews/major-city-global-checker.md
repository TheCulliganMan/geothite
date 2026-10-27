# Indexed major-city planning checks

The shared checker now validates the117 new city rooms through each exact
expansion contract. It retains the legacy checks for the original town rooms.
The clear room fixture position is separate from an indexed native entrance:
passive floor arrivals and active department-store stairs are both represented.
Each incoming entrance must have a reciprocal return, the correct target index,
parent safe position and actual native collision. Six city/road connections use
their recorded offsets and lane pairs rather than assuming a zero offset.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_plan.py \
  --pack content-packs/minnesota-three-cities-tight-final-v02.crystalpack
.local/minnesota-review-env/bin/python planning/minnesota/tools/audit_city_plan_bindings.py \
  --pack content-packs/minnesota-three-cities-tight-final-v02.crystalpack \
  --output planning/minnesota/reviews/major-city-global-checker-NEW
```

Both commands passed against the frozen660-map city pack. The
[42-check negative-fixture audit](major-city-global-checker-v01/proof-1c17db0377d19ddad9d00925e30ed4af31bf73ed15d26e1846f928e3a6785c81.json)
includes39 in-memory corruptions rejected across all three cities: stale
upper-floor entries/returns, wrong target indices, missing reservations/pairs,
wrong source files, facade indices, road offsets/lanes and native room terrain,
warp roster, destination or extent drift. Neither the checker nor its audit
executes controller inputs. Gameplay evidence remains in the
[separate30-case controller receipt](major-city-controller-final.json).
