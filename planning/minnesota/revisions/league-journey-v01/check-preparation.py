#!/usr/bin/env python3
"""Trace sign-in, healing, storage and restocking against the actual regional landings."""

import argparse
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, require
from render_reference import load_pack

rail = runpy.run_path(str(HERE.parent / "headwaters-journey-v01/check-rail.py"))
paths, quadrant, coords = (rail[k] for k in ("paths", "quadrant", "coords"))
CLINIC, MART, TOWN = "MnElyPokecenter1F", "MnElyMart", "MnEly"


def streets(data, name, start):
    room = data["maps"][name]
    blocked = {coords(o) for o in room["objects"]}
    return paths(data, name, start, blocked)


def fly(data):
    result = {}
    for flag, destination in data["fly_destinations"].items():
        if not flag.startswith("ENGINE_FLYPOINT_MN_"):
            continue
        spawn = data["runtime_spawn_points"][
            str(destination["destination_spawn_identifier"])
        ]
        name = spawn["mapName"]
        room = data["maps"][name]
        callbacks = [
            r["args"][1]
            for r in room["map_script_section_commands"]
            if r["command"] == "callback" and r["args"][0] == "MAPCALLBACK_NEWMAP"
        ]
        require(
            any(("setflag", flag) in instructions(room, cb) for cb in callbacks),
            f"{name} does not register its own destination",
        )
        tile = spawn["tileX"], spawn["tileY"]
        walk = streets(data, name, tile)
        require(
            quadrant(data, name, *tile) == "FLOOR" and len(walk) > 1,
            f"{name} Fly landing",
        )
        require(
            any(abs(tile[0] - x) + abs(tile[1] - y) >= 6 for x, y in walk),
            f"isolated arrival in {name}",
        )
        result[name] = tile
    require(len(result) == 19, "regional visit-unlocked Fly locations")
    return result


def verify(path):
    data = load_pack(path)["data"]
    landings = fly(data)
    lobby = data["maps"]["MnLeagueLobby"]
    # Follow each real street exit. Both gates must lead back to all three services.
    exits = {w["target_warp_id"] for w in lobby["events"]["warps"]}
    room = data["maps"][TOWN]
    town_doors = {w["index"]: w for w in room["events"]["warps"]}
    stops = {"clinic": (11, 22), "Mart": (27, 28), "League": (25, 14)}
    for index, service in [(5, "clinic"), (6, "Mart"), (7, "League")]:
        require(coords(town_doors[index]) == stops[service], f"{service} entrance")
        entrance = town_doors[index - 4]
        require(
            quadrant(data, TOWN, *coords(entrance)) == "DOOR"
            and entrance["target_map_constant"]
            == town_doors[index]["target_map_constant"],
            f"active entrance behind {service} landing",
        )
    require(exits == {7, 8}, "League town exits")
    for origin in [coords(town_doors[i]) for i in sorted(exits)] + [landings[TOWN]]:
        walk = streets(data, TOWN, origin)
        require(
            all(tile in walk for tile in stops.values()),
            "walking path to League preparations",
        )
    clinic = data["maps"][CLINIC]
    clinic_walk = streets(data, CLINIC, coords(clinic["events"]["warps"][2]))
    require(
        (3, 3) in clinic_walk
        and quadrant(data, CLINIC, 3, 2) == "COUNTER"
        and (9, 2) in clinic_walk,
        "physical nurse and PC approaches",
    )
    require(
        instructions(clinic, "MnElyNurse") == [("jumpstd", "PokecenterNurseScript")],
        "clinic nurse",
    )
    count = 0
    for name, at in data["maps"].items():
        if (
            not name.startswith("Mn")
            or at["attributes"]["tileset_name"] != "pokecenter"
        ):
            continue
        require(quadrant(data, name, 9, 1) == "PC", f"{name} stock terminal")
        for event in at["events"]["bg_events"]:
            if not any(
                row["command"] == "special" and row["args"] == ["PokemonCenterPC"]
                for row in at["scripts"].get(event["script"], [])
            ):
                continue
            require(coords(event) == (9, 1), f"{name} wall opens storage")
            require(
                any(
                    r["command"] == "bg_event"
                    and r["args"] == ["9", "1", event["event_type"], event["script"]]
                    for r in at["map_event_section_commands"]
                ),
                f"{name} stale PC position",
            )
        count += 1
    mart = data["maps"][MART]
    from_door = streets(data, MART, coords(mart["events"]["warps"][2]))
    require(
        (3, 3) in from_door
        and quadrant(data, MART, 2, 3) == "COUNTER"
        and any(
            coords(o) == (1, 3) and o["script"] == "MnElyClerk" for o in mart["objects"]
        ),
        "clerk access",
    )
    stock = data["marts"]["MART_MN_ELY"]
    require(
        stock
        == [
            "ULTRA_BALL",
            "HYPER_POTION",
            "MAX_POTION",
            "FULL_RESTORE",
            "FULL_HEAL",
            "REVIVE",
            "MAX_REPEL",
        ]
        and data["items"]["FULL_RESTORE"]["price"] == 3000,
        "Ely stock",
    )
    require(
        instructions(room, "MN_ELY_PINE_SUPPLY") == [("itemball", "FULL_RESTORE", "1")],
        "free pine cache",
    )
    print(
        f"{len(landings)} Fly landings and callbacks, {count} regional Center layouts, both League exits, "
        "clerk, nurse, storage and free pine supply verified."
    )


if __name__ == "__main__":
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("pack", type=Path)
    verify(args.parse_args().pack)
