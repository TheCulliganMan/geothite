#!/usr/bin/env python3
"""Trace the three Pepin observations, the northern inlet, every return reply and the wild encounter."""

import argparse
import json
from itertools import product
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

rare = runpy.run_path(str(HERE.parent / "trail-discoveries-v01/check-rare.py"))
winter = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))
indexes = runpy.run_path(str(HERE.parent / "return-visits-v01/check-returns.py"))[
    "check_indexes"
]
ZONE = "MnMississippiLowerWoods"
WEST, EAST, INLET = [ZONE + suffix for suffix in ("C00R01", "C01R01", "C01R00")]
SKETCH, EDDY, PINES, SAFE, CAUGHT = [
    "EVENT_MN_PEPIN_" + k
    for k in [
        "DOCK_SKETCH_READ",
        "EDDY_OBSERVED",
        "REEDS_OBSERVED",
        "SAFE_INLET_IDENTIFIED",
        "CAUGHT",
    ]
]
ACTOR = "MN_PEPIN_PEPIVERN"
TEXT = "MnPepinText_"
LABELS = {
    WEST: "dock_intro eddy artist_foam artist_skipper",
    EAST: "operator_partial operator_eddy operator_pines operator_compare operator_complete needs_surf reeds",
    INLET: "inlet_early reveal retreat caught",
}


def reach(data):
    # These forest banks use regular ground, grass and water. Prove the directions on
    # this subgraph; reverse directions also have to exist with every original actor present.
    places = {n for n in data["maps"] if n.startswith(ZONE)} | {"MnStPaul"}
    trail = winter["Trail"](data, places)
    room = data["maps"][WEST]
    home = data["maps"]["MnStPaul"]
    (connector,) = [
        c for c in room["attributes"]["connections"] if c["target_map"] == "MnStPaul"
    ]
    require(
        connector["direction"] == "west"
        and connector["offset"] == -14
        and any(
            c["target_map"] == WEST and c["direction"] == "east"
            for c in home["attributes"]["connections"]
        ),
        "St. Paul shore connection",
    )
    globals_ = trail.step.__func__.__globals__
    globals_["WALK"] = {"FLOOR", "TALL_GRASS"}
    s = WEST, 22, 11
    globals_["WALK"].add("WATER")
    public = [
        (WEST, 0, y)
        for y in range(len(trail.grid[WEST]))
        if trail.kind(WEST, 0, y) in globals_["WALK"]
        and trail.step((WEST, 0, y), "LEFT")
    ]
    require(
        public and trail.both(public[len(public) // 2], s),
        "public St. Paul river approach",
    )
    globals_["WALK"].discard("WATER")
    require(
        trail.both(s, (WEST, 23, 10)) == (2, 2)
        and trail.both((EAST, 51, 16), (EAST, 50, 19)) == (4, 4),
        "artist and skipper bank approaches",
    )
    globals_["WALK"] |= {"WATER"}
    # The western water read also has an ordinary launch just south of it.
    # A player can read the foam first, then board beside the same landmark.
    for shore, water, forwards, back in [
        ((WEST, 23, 10), (WEST, 24, 10), "RIGHT", "LEFT"),
        ((WEST, 23, 11), (WEST, 24, 11), "RIGHT", "LEFT"),
        ((EAST, 50, 19), (EAST, 49, 19), "LEFT", "RIGHT"),
    ]:
        require(
            trail.kind(*shore) == "FLOOR"
            and trail.kind(*water) == "WATER"
            and trail.step(shore, forwards) == water
            and trail.step(water, back) == shore,
            f"boarding and portage at {shore}",
        )
    require(trail.both(s, (EAST, 51, 16)) == (166, 166), "route round the big bend")
    require(
        trail.step((EAST, 50, 19), "LEFT") == (EAST, 49, 19)
        and trail.kind(EAST, 49, 19) == "WATER"
        and trail.kind(EAST, 50, 19) == "FLOOR"
        and trail.kind(EAST, 50, 20) == "WALL",
        "bank, launch and pine",
    )
    globals_["WALK"] = {"WATER"}
    path = trail.path((EAST, 49, 20), (INLET, 24, 22))
    require(
        len(path) == 71
        and path[0] == "UP"
        and "LEFT" in path
        and trail.both((EAST, 49, 20), (INLET, 24, 22)) == (71, 71),
        "uninterrupted water route to the north-bank notch",
    )


def scene(pack, baseline):
    d = pack["data"]
    rooms = d["maps"]
    artist, skipper, inlet = [rooms[n] for n in (WEST, EAST, INLET)]
    for r, label, at, kind in [
        (artist, "MnPepinDock", (22, 10), "SPRITE_FISHER"),
        (artist, "MnPepinEddy", (24, 10), "BGEVENT_READ"),
        (skipper, "MnPepinOperator", (50, 16), "SPRITE_SAILOR"),
        (skipper, "MnPepinReeds", (50, 20), "BGEVENT_READ"),
        (inlet, ACTOR, (24, 21), "SPRITE_BARBELISK"),
    ]:
        (e,) = [
            r for r in r["objects"] + r["events"]["bg_events"] if r["script"] == label
        ]
        require(
            (e["x"], e["y"], e.get("sprite", e.get("event_type"))) == (*at, kind), label
        )
    story = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"][
        "texts"
    ]
    quest = next(
        q
        for q in json.loads(
            (REGION / "pokemon/acquisition-v01/rare-discoveries.json").read_text()
        )["quests"]
        if q["id"] == "pepin-three-wakes"
    )
    for name, suffixes in LABELS.items():
        for label, program in rooms[name]["scripts"].items():
            if label.startswith(("MnPepin", "MN_PEPIN")):
                indexes(d, rooms[name], label)
        if not baseline:
            for key in suffixes.split():
                label = TEXT + key
                native = native_pages(
                    rooms[name]["script_text_bodies"][label]["commands"]
                )
                require(
                    native == story[name][label]
                    and native
                    == [
                        [s.replace("PEPIVERN", "BARBELISK") for s in pg]
                        for pg in quest["dialogue_pages"][key]
                    ],
                    label,
                )
    for combination in product((False, True), repeat=3):
        flags = {
            flag for flag, active in zip([SKETCH, EDDY, PINES], combination) if active
        }
        actions, lines, after = read_branch(skipper, "MnPepinOperator", flags)
        if all(combination):
            require(
                lines == [TEXT + "operator_compare"]
                and after == flags | {SAFE}
                and actions.index(("promptbutton",))
                < actions.index(("setevent", SAFE)),
                "signoff too early",
            )
        else:
            first = (
                [TEXT + "operator_intro", TEXT + "operator_partial"]
                if baseline or SKETCH not in flags
                else [
                    TEXT + "operator_eddy"
                    if EDDY not in flags
                    else TEXT + "operator_pines"
                ]
            )
            require(lines == first and after == flags, f"wrong missing clue {flags}")
        _, lines, after = read_branch(artist, "MnPepinDock", flags)
        expected = (
            [TEXT + "dock_intro", TEXT + "dock_sketch"]
            if baseline or SKETCH not in flags
            else []
        )
        if not baseline:
            expected.append(
                TEXT + ("artist_skipper" if EDDY in flags else "artist_foam")
            )
        require(
            lines == expected and after == flags | {SKETCH},
            "artist reminder or initial sketch",
        )
    for room, root, page in [
        (artist, "MnPepinDock", "dock_complete"),
        (skipper, "MnPepinOperator", "operator_complete"),
    ]:
        require(
            read_branch(room, root, {SAFE})[1:] == ([TEXT + page], {SAFE}),
            "completed observation",
        )
    for room, script, flag, page in [
        (artist, "MnPepinEddy", EDDY, "eddy"),
        (skipper, "MnPepinReedsSurf", PINES, "reeds"),
    ]:
        a, l, f = read_branch(room, script)
        require(
            l == [TEXT + page]
            and f == {flag}
            and a.index(("promptbutton",)) < a.index(("setevent", flag)),
            "observation confirmation",
        )
    require(
        instructions(skipper, "MnPepinReeds")
        == [
            ("readvar", "VAR_MOVEMENT"),
            ("ifequal", "4", "MnPepinReedsSurf"),
            ("ifequal", "8", "MnPepinReedsSurf"),
            ("jumptext", TEXT + "needs_surf"),
        ],
        "water-only vantage",
    )
    [battle] = inlet["scripted_wild_battles"]
    require(
        battle["request"]["species"] == "BARBELISK"
        and battle["request"]["level"] == 55,
        "living species and level",
    )
    species = "BARBELISK"
    require(
        species in d["pokemon"]
        and species in d["pokemon_cries"]
        and "gfx/pokemon/barbelisk/overworld.2bpp" in pack["runtime_files"],
        "active sprite/cry",
    )
    for caught in (False, True):
        v = rare["Visit"](set())
        rare["entry"](inlet, v, new_map=True)
        require(ACTOR not in v.visible, "premature encounter")
        v.flags.add(SAFE)
        rare["entry"](inlet, v, new_map=True)
        require(ACTOR in v.visible, "missing encounter")
        decline = rare["follow"](inlet, ACTOR, v)
        require(
            ("startbattle",) not in decline
            and v.flags == {SAFE}
            and ACTOR in v.visible,
            "decline",
        )
        t = rare["follow"](inlet, ACTOR, v, yes=True, captured=caught)
        require(
            ("writetext", TEXT + "ask") in t
            and ("cry", species) in t
            and ("loadwildmon", species, "55") in t
            and ACTOR not in v.visible
            and (CAUGHT in v.flags) == caught,
            "encounter result",
        )
        rare["entry"](inlet, v, new_map=False)
        require(ACTOR not in v.visible, "Continue or battle reload")
        rare["entry"](inlet, v, new_map=True)
        require((ACTOR in v.visible) != caught, "physical return")


def water(pack, baseline):
    f = pack["runtime_files"]
    d = pack["data"]
    room = d["maps"][WEST]
    i = room["blocks"][5 * room["attributes"]["width"] + 12]
    require(i == (0x35 if baseline else 0x8D), "western eddy position")
    if baseline:
        return
    layout = bytes(f["data/tilesets/johto_metatiles.bin"])
    square = layout[16 * i : 16 * i + 16]
    require(
        square == bytes([0x14] * 4 + [0x71, 0x72] + [0x14] * 10)
        and d["tilesets"]["johto"]["collision"]["8d"] == ["WATER"] * 4,
        "water around the curl",
    )
    clips = json.loads(bytes(f["data/tilesets/johto_animations.json"]))
    frames = json.loads((HERE / "pepin-eddy.json").read_text())
    for k, g in enumerate((0x71, 0x72)):
        clip = clips[str(g)]
        require(
            clip["frame_ticks"] == 22
            and len(clip["frames"]) == 4
            and len(set(map(tuple, clip["frames"]))) == 4,
            "four distinct water phases",
        )
        for frame, expected in zip(clip["frames"], frames):
            rows = [
                "".join(
                    str(
                        (frame[2 * y] >> (7 - x) & 1)
                        + 2 * (frame[2 * y + 1] >> (7 - x) & 1)
                    )
                    for x in range(8)
                )
                for y in range(8)
            ]
            require(
                rows == [r[k * 8 : k * 8 + 8] for r in expected],
                "actual native water pixels",
            )
        require(
            clip["frames"][0]
            == list(f["gfx/tilesets/johto.2bpp"][16 * g : 16 * g + 16])
            and d["tilesets"]["johto"]["palette_map"][g] == 3,
            "resting water image and palette",
        )
        require(
            layout.count(g) == 1
            and all(
                g not in f[f"data/tilesets/{n}_metatiles.bin"]
                for n in (
                    "mn_outdoor_johto_stairs",
                    "mn_r10_rockstairs",
                    "johto_glacier_rail",
                    "mn_winter_iron_descent",
                    "mn_iron_descent_thaw",
                    "mn_winter_bemidji_northwoods",
                )
            ),
            "foam animation escaped to inherited tileset",
        )


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    cli.add_argument("--baseline", action="store_true")
    args = cli.parse_args()
    pack = load_pack(args.pack)
    reach(pack["data"])
    scene(pack, args.baseline)
    water(pack, args.baseline)
    print(
        "Pepin passed: five bound interactions, three acknowledged observations, eight combinations, repeat visits, 166-step bank crossing, 71-step water route, both encounter outcomes and physical reentry."
    )
