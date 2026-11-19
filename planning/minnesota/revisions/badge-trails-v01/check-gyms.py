#!/usr/bin/env python3
"""Trace all eight gym sight lanes and the two prior-badge conversations."""
import argparse
from collections import deque
from itertools import product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

lanes = runpy.run_path(str(HERE.parent / "trail-battles-v01/check-trainers.py"))
coords = lanes["coords"]
ORDER = ("Rochester", "Mankato", "Minneapolis", "StPaul", "StCloud", "Brainerd", "Bemidji", "Duluth")
AFFECTED = {"MnStPaulGym", "MnDuluthGym"}
CANONICAL = json.loads((HERE.parents[1] / "story.json").read_text())["dialogue_overhaul"]["texts"]


def public_door(data, room):
    city = data["maps"][room["id"].removesuffix("Gym")]
    targets = {w["target_warp_id"] for w in city["events"]["warps"]
               if w["target_map_constant"] == room["attributes"]["map_constant"]}
    (arrival,) = {coords(w) for w in room["events"]["warps"] if w["index"] in targets}
    return arrival


def street_approach(data, name, starts):
    """Use only level walking squares; honor the installed checkpoint removal callbacks."""
    city = data["maps"][name]
    gym = data["maps"][name + "Gym"]
    door_set = {coords(w) for w in city["events"]["warps"]
                if w["target_map_constant"] == gym["attributes"]["map_constant"]}
    require(door_set == {(48,31)}, f"unexpected {name} gym entrance")
    front = (48,32)
    require(not city["events"]["coord_events"], f"{name}: unchecked street event")
    hidden = set()
    if name == "MnDuluth":
        label = "MnPublicRoadsDuluthOpen"
        guards = {"MN_DULUTH_RANGE_GUARD_NORTH", "MN_DULUTH_RANGE_GUARD_SOUTH",
                  "MN_DULUTH_SHORE_GUARD_EAST", "MN_DULUTH_SHORE_GUARD_WEST"}
        callbacks = {(r["args"][0], r["args"][1]) for r in city["map_script_section_commands"]
                     if r["command"] == "callback"}
        require({("MAPCALLBACK_TILES", label), ("MAPCALLBACK_OBJECTS", label)} <= callbacks,
                "Duluth reentry no longer clears the road")
        commands = instructions(city, label)
        require(commands[-1] == ("endcallback",)
                and len(commands) == 5
                and set(commands[:-1]) == {("disappear", g) for g in guards},
                "Duluth public roads require progress or have stopped clearing a guard")
        hidden = guards
        for road, pair in [("Range", {"MN_DULUTH_RANGE_GUARD_NORTH", "MN_DULUTH_RANGE_GUARD_SOUTH"}),
                           ("Shore", {"MN_DULUTH_SHORE_GUARD_EAST", "MN_DULUTH_SHORE_GUARD_WEST"})]:
            fallback = instructions(city, f"MnDuluth{road}Check")
            require(set(fallback[:-1]) == {("disappear", g) for g in pair}
                    and fallback[-1] == ("end",), "unreconciled Duluth guard pushes player back")
    obstacles = {coords(o) for o in city["objects"] if o["object_identifier"] not in hidden}
    attrs = city["attributes"]
    pavement = {(x, y) for y in range(2*attrs["height"]) for x in range(2*attrs["width"])
                if lanes["quadrant"](data, name, x, y) in {"FLOOR", "TALL_GRASS"}}
    pavement -= obstacles
    require(lanes["quadrant"](data, name, 48, 31) == "DOOR", f"{name} gym-door collision")
    # Every edge in this restricted graph is plain walking in either direction.
    queue, distance = deque([front]), {front:0}
    while queue:
        at = queue.popleft()
        for nxt in lanes["adjacent"](at) & pavement:
            if nxt not in distance:
                distance[nxt] = distance[at]+1
                queue.append(nxt)
    connected = lanes["anchors"](data)[name]
    require(front in pavement and set(starts) <= connected & distance.keys(), f"{name} street path")
    print(f"{name}: road connections to gym doorstep {front}: "
          f"{[(p, distance[p]) for p in starts]} ordinary steps in either direction.")


def court_walk(data, room, start, flags, leader):
    """Follow installed terrain, ice and paired lift pads from the public arrival after reentry."""
    attrs = room["attributes"]
    blocks = room["blocks"].copy()
    (callback,) = [r["args"][1] for r in room["map_script_section_commands"]
                   if r["command"] == "callback" and r["args"][0] == "MAPCALLBACK_TILES"]
    lanes["run"](room, callback, flags.copy(), blocks)
    collision = data["tilesets"][attrs["tileset_name"]]["collision"]
    terrain = {(x,y): collision[f"{blocks[y//2*attrs['width']+x//2]:02x}"][2*(y%2)+x%2]
               for y in range(2*attrs["height"]) for x in range(2*attrs["width"])}
    walk = {"FLOOR", "ICE", "WARP_PANEL", "WARP_CARPET_DOWN"}
    solid = {"WALL", "BOOKSHELF"}
    require(set(terrain.values()) <= walk | solid, f"{room['id']}: unsupported terrain")
    objects = {coords(o) for o in room["objects"]}
    warps = room["events"]["warps"]
    home = attrs["map_constant"]
    points = {w["index"]: coords(w) for w in warps}
    lifts = {coords(w): points[w["target_warp_id"]] for w in warps if w["target_map_constant"] == home}
    exits = {coords(w) for w in warps if w["target_map_constant"] != home}
    queue, seen = deque([(start, 0)]), {start}
    while queue:
        at, distance = queue.popleft()
        if at in lanes["adjacent"](leader):
            return distance, at
        for dx, dy in lanes["D"].values():
            x, y = at
            while (p := (x + dx, y + dy)) not in objects and terrain.get(p) in walk:
                x, y = p
                if terrain[p] != "ICE":
                    break
            dest = x, y
            if dest == at or dest in exits:
                continue
            dest = lifts.get(dest, dest)
            if dest not in seen:
                seen.add(dest)
                queue.append((dest, distance + 1))
    return None


def verify(pack, baseline):
    data = pack["data"]
    street_approach(data, "MnStPaul", [(0,48), (0,49)])
    street_approach(data, "MnDuluth", [(0,44), (0,45), (64,0), (65,0), (66,0), (67,0)])
    previous_badge = None
    students = 0
    for number, city in enumerate(ORDER, 1):
        name = "Mn" + city + "Gym"
        room = data["maps"][name]
        (leader,) = [o for o in room["objects"] if o["script"] == name + "Leader"]
        teachers = [o for o in room["objects"] if o["object_type"] == "OBJECTTYPE_TRAINER"]
        require(leader["radius"] == 0 and leader["object_type"] == "OBJECTTYPE_SCRIPT", f"{city} leader talk")
        require(set(room["trainer_scripts"]) == {o["script"] for o in teachers}, f"{city} trainers")
        start = public_door(data, room)
        students += lanes["gym_sight"](data, room, teachers, {start})
        body = instructions(room, leader["script"])
        battle = body.index(("startbattle",))
        required = (("checkflag", previous_badge) if previous_badge else
                    ("checkevent", "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"))
        require(required in body[:battle], f"{city} missing prerequisite")
        (earned,) = [args[0] for op, *args in body[battle:] if op == "setflag"]
        if name in AFFECTED:
            label = name + "RequirementsText"
            pages = native_pages(room["scripts"][label])
            require(pages == native_pages(room["script_text_bodies"][label]["commands"])
                    == native_pages(data["map_scripts"][label]), f"{city} text mirrors")
            require(all(0 < len(p) <= 2 and all(0 < len(line) <= 18 for line in p) for p in pages),
                    f"{city} native LCD bounds")
            wording = " ".join(line for page in pages for line in page)
            before = "Beat both" if city == "StPaul" else "and ANVER"
            require((before in wording) == baseline, f"{city} stale directions")
            if not baseline:
                require(pages == CANONICAL[name][label], f"{city} uninstalled dialogue")
                cues = (("BELVA or ORVEN", "AVRIN", "CURRENT BADGE", "MINNEAPOLIS")
                        if city == "StPaul" else ("ORLIS", "PINE BADGE", "BEMIDJI"))
                require(all(cue in wording for cue in cues), f"{city} missing badge or court directions")
            prior = ORDER[number-2]
            origin = data["maps"]["Mn" + prior + "Gym"]
            gift = " ".join(line for p in native_pages(origin["scripts"][origin["id"]+"LeaderGiftText"]) for line in p)
            named_badge = "CURRENT BADGE" if city == "StPaul" else "PINE BADGE"
            require(named_badge in gift and ("AVRIN" if city == "StPaul" else "ORLIS") in gift,
                    f"{prior} previous badge giver")
            court = f"EVENT_MN_GYM_{number}_COURT_OPEN"
            outcomes = []
            flags = [room["trainer_scripts"][o["script"]]["event_flag"] for o in teachers]
            for defeated in product((False, True), repeat=len(flags)):
                state = {f for f, won in zip(flags, defeated) if won}
                lanes["run"](room, name+"V2Qualify", state, room["blocks"].copy())
                qualifies = any(defeated) if city == "StPaul" else all(defeated[:2])
                require((court in state) == qualifies, f"{city} court uses wrong teachers")
                actions, texts, _ = read_branch(room, leader["script"], state)
                require(texts == [label] and ("startbattle",) not in actions, f"{city} missing-badge response")
                with_badge, with_badge_texts, _ = read_branch(room, leader["script"], state | {previous_badge})
                require((("startbattle",) in with_badge) == qualifies, f"{city} badge-present decision")
                if not qualifies:
                    require(court_walk(data, room, start, state, coords(leader)) is None,
                            f"{city}: unearned physical access to leader")
                    if city == "StPaul":
                        expected = label if baseline else name + "GuideText"
                        require(with_badge_texts == [expected], "St Paul closed-court dialogue")
                if qualifies:
                    path = court_walk(data, room, start, state, coords(leader))
                    require(path is not None, f"{city}: lost earned route")
                    distance, stance = path
                    require(body.index(("checkevent", court)) < battle, "court checked before leader fight")
                    outcomes.append((defeated, distance, stance))
            print(f"{city}: {2 ** len(flags)} trainer outcomes. Public arrival {start}, leader {coords(leader)}, "
                  f"reachable without prior badge after {outcomes}.")
        previous_badge = earned
    require(students == 18, "gym student total")
    print("Eight public entrances, eight talk-led leader battles, 18 reachable trainer sight lanes, "
          "one Rochester parcel prerequisite and seven preceding-badge requirements.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--baseline", action="store_true")
    args = parser.parse_args()
    verify(load_pack(args.pack), args.baseline)
