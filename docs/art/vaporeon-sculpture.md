# Vaporeon sculpture and rig

The generic blue fox is replaced by an original cyan aquatic cat with a smooth
rounded face, violet gaze, a small muzzle and smile, a navy crown and a tall
central crest. Broad blue-rimmed head fins carry fitted cream membranes and
two seated rays on each face. A continuous scalloped white neck fan frames
the head. Rounded shoulder and hind-leg roots support four planted paws.
A long curling tail ends in two broad mermaid flukes; navy scalloped ridges
follow the back and actual upper tail surface. The reference is the
[official Vaporeon artwork](https://sg.portal-pokemon.com/pokedex/0134/).
No reference pixels are included in the model.

`tools/vaporeon_sculpt.py` authors 57 closed parts, 13,565 vertices and
23,164 triangles. `tools/vaporeon_glb.py` exports a 569,232-byte canonical GLB
with 232 accessors and 16 joints at authoring height 1.00. The neutral digest is
`0d92fd28cf7c31c03b49e55538d760de684e22797e9a32a7ccfba31790e84a81`.
The local content pack supplies height digits 303 (0.9906 metres), BUBBLE JET.
The original pack size is retained.

The rig has body/neck/head carriers, two head-fan carriers, a separate neck-frill
carrier, four softly blended limb carriers, three tail carriers and independent
paired fluke carriers. Paws and toe creases remain on the neutral authoring root.
The face, crown and crest remain coherent, and each head-fan carrier owns its
blue frame, both cream faces and all four ribs. The navy tail ridge uses the
same continuous path weights as the tail. Idle, attack and hit clips retain
neutral endpoints and existing source-cue timing.

Nine Python asset checks cover canonical bytes and limits, closed outward
solids, actual face normals, fitted face and both head-fan surfaces, occupied
fan-ray seating with a detached negative control, ownership, occupied body/
frill/fan/limb/fluke roots and dorsal-ridge seating, floor/endpoints/winding/
strain/independent motion, and cue metadata. Front, rear and side poses of all
three clips were inspected. Rebuild with `python3 tools/vaporeon_glb.py`;
run asset checks with
`python3 -m unittest discover -s tools -p test_vaporeon_glb.py`.
Shiny verification is skipped at the user's request.

Both focused Rust checks, the shared volumetric/model and source-cue timing
regressions, and byte-identical Blender generation pass. The shared build
exposed a room test calling `pop` on `Arc<[IVec2]>`; its mutation now copies to
a temporary vector and assigns the shortened slice back. The external
room-donor regression passes against the local TUI pack.

The native example builds using `target/pokemon-models` and `opt-level=0`
overrides for crystal-core/assets/voxel-view/bevy. The Route 29 walking encounter
against Pidgey passes terrain admission. Its inspected 3D/2D pair preserves
state at core frame 3871, checksum 1754180436, LCD animation frame 4606 and
virtual time 85.294497. The gaze, head fans, crest, white frill, dorsal ridge
and grounded paws remain readable at original pack size, with correct species
labels and no reported shader errors. Evidence uses prefix
`target/pokemon-native-review/vaporeon-normal`.

A second Route 29 pair uses ordinary alternating down/up steps over checked
grass cells. It preserves state at frame 2906, checksum 761625219, LCD frame
3580 and virtual time 65.686622126, under prefix `vaporeon-tail-vertical`.
Those two native cameras self-occlude the long tail behind the body.

An opposite-side native review now exposes the full tail curl and both flukes,
with grounded paws, readable head fins/frill and correct species labels.
Its paired 3D/2D capture preserves state at core frame 3219, checksum
409803199, LCD frame 3890 and virtual time 69.192291249. Terrain admission
passes at original pack size, with no reported shader errors. Evidence uses
`target/pokemon-native-review/vaporeon-opposite`. This closes the native
tail-angle review; browser visual acceptance remains open.

For repeatable native art review, `CRYSTAL_BATTLE_REVIEW_YAW` selects one of
the existing establishing angles: 70, 90, 110, 140, -70, -90, -110 or -140.
This capture used -70. Unset or invalid values retain the normal search;
WASM retains the normal search. The selector preserves terrain support,
scenery omission, physical scale, full-body fit and frustum admission.
The selector check and seven camera/facing/Vaporeon rig regressions pass;
the native example rebuild also passes.

The optional Route 35 angle attempt aborted with a compute-worker stack
overflow. Retrying with `RUST_MIN_STACK=16777216` reached a Venonat encounter
but timed out waiting for an active world frame. Those attempts provide no
visual acceptance evidence; their logs use prefixes `vaporeon-tail-small-stack`
and `vaporeon-tail`. A leftward Route 29 attempt exhausted its inputs at a wall
before the encounter cooldown expired, recorded in `vaporeon-tail-route29.log`.
