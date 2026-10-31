# Boundary cabin cast revision

The Boundary Waters zone contract places Dorothy at the outdoor root-beer
stand. The preserved, hash-pinned 187-room interior source also names its indoor
healer Dorothy. Apply `boundary-cabin-volunteer-v01.json` after that interior
layer to replace the duplicate with an unnamed cabin volunteer using the native
Teacher sprite. Keep the outdoor Dorothy and indoor paddler.

The Rust publisher preserves the existing acknowledgment followed by HealParty,
both exits, passive arrivals, inventory, fees, expedition counters and all other
content. It rejects incompatible actors or care programs and occupied labels.
The original story source remains unchanged.

Executed publication from the private worktree:

```sh
/Users/ryanculligan/GitHub/geothite/target/verdigrim-scene/debug/revise_boundary_cabin_cast \
  content-packs/devils-kettle-rainy-review-v01.crystalpack \
  planning/minnesota/interior-life/revisions/boundary-cabin-volunteer-v01.json \
  content-packs/devils-kettle-rainy-cabin-volunteer-v01.crystalpack
```

The ignored, unselected result has 731 maps and SHA256
`59ffe2900fc64a91353282d5c76a528e1e3cf8475f20ac174b3e6ced286220cc`.
It includes the separate Devil's Kettle branch and clinic revision; it is not a
replacement for the selected mapping parent.

Both `boundary_cabin_cast_controller_` regressions passed through the production
controller: held dialogue and blocked movement, same-pack Save/Continue, HP/PP
restoration and repeat care, unchanged money/inventory/recovery destination, both
exit lanes, outdoor Continue and physical reentry. These begin with a declared
unpaid cabin fixture. They do not prove paid Safari admission or an active
expedition's capture/session lifecycle.

The independent `audit_boundary_cabin_cast.py BEFORE AFTER SOURCE` passes with
730 other maps exact. Its four negative mutations reject outdoor Dorothy, door,
care-operation and compiled-text corruption. The reached healed save was rendered
headlessly using the production native 2D renderer and inspected. Generated packs,
saves, images and audit logs remain ignored.

The mapping owner's older empty-room assertion still needs reconciliation to the
explicit two-person cast after integration. This revision alone does not close
the comprehensive campaign or paid Safari acceptance.
