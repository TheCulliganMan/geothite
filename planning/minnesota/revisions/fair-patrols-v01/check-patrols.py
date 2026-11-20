#!/usr/bin/env python3
"""Compare a written Fair Patrols edition with its Fair Table parent."""
import argparse
import base64
from collections import deque
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
VENDORS = runpy.run_path(str(HERE.parent / "fair-table-v01" / "check-vendors.py"))
read_pack = VENDORS["LAKESIDE"]["read_pack"]
terrain, xy = VENDORS["terrain"], VENDORS["xy"]
MOVES = {"trainer_walk": "SPRITEMOVEDATA_PATROL_CLOCKWISE_SLOW", "trainer_run": "SPRITEMOVEDATA_PATROL_CLOCKWISE"}
GATE = (95, 64)
CHANGED_MAPS = {"MnR15C00R00", "MnStateFairgrounds", "MnWinonaFieldStation"}


def changed_keys(old, new):
    return (old.keys() ^ new.keys()) | {k for k in old.keys() & new.keys() if old[k] != new[k]}


def check_archive(parent, pack):
    # Includes identical compiled audio, audio manifest, compression and runtime artwork.
    assert changed_keys(parent, pack) == {"identity", "data", "report"}
    expected_manifests = parent["report"]["manifests"] + ["northstar-fair-patrols-v1"]
    assert pack["report"] == parent["report"] | {"manifests": expected_manifests}
    old, new = parent["data"], pack["data"]
    # Includes items, vendors, wild/field encounters, learnsets, species, tiles and movement rules.
    assert changed_keys(old, new) == {"maps", "map_blocks", "map_scripts", "npcs", "trainers",
                                      "story_event_script_constants"}
    assert old["maps"].keys() == new["maps"].keys() and changed_keys(old["maps"], new["maps"]) == CHANGED_MAPS
    assert changed_keys(old["map_blocks"], new["map_blocks"]) == {"MnR15C00R00_Blocks"}
    assert changed_keys(old["npcs"], new["npcs"]) == {"MnStateFairgrounds", "MnWinonaFieldStation"}
    starters = ("POUCHID", "KINDLIT", "BIRCHIT")
    script_labels = {"MnStateFairgrounds_MapEvents", "MnWinonaFieldStation_MapScripts",
                     "MnWinonaFieldStation_MapEvents", "MnWinonaResearchBenches"}
    for partner in starters:
        script_labels.update((f"MnEarlyPartner{partner}", f"MnWinonaBench{partner}", f"MnWinonaBench{partner}Hide"))
    assert changed_keys(old["map_scripts"], new["map_scripts"]) == script_labels
    assert changed_keys(old["maps"]["MnStateFairgrounds"], new["maps"]["MnStateFairgrounds"]) == {
        "objects", "map_event_section_commands"
    }
    check_bakers_party(old, new)
    check_dex_numbers(old, new)


def check_bakers_party(old, new):
    baker, pie = "MN_FAIR_BAKER_TRAINER", "MN_BLUEBERRY_SLICE"
    previous = old["trainers"]["trainers"]
    ada = previous[baker]
    lead, *rest = ada["party"]
    assert ada["name"] == "ADA" and lead["species"] == "DOUGHLIN" and lead["item"] is None
    expected = previous | {baker: ada | {"party": [lead | {"item": pie}, *rest]}}
    assert new["trainers"] == old["trainers"] | {"trainers": expected}, "exactly one held slot changes"
    item = new["items"][pie]
    assert item["held_effect"] == "HELD_BERRY" and item["parameter"] == 50 and item["consumable"]


def check_dex_numbers(old, new):
    original = old["story_event_script_constants"]
    prefix = "MN_DEX_MEMBER_"
    markers = {key for key in original["global"] if key.startswith(prefix)}
    base = HERE.parents[1] / "pokemon"
    roster = json.loads((base / "roster.json").read_text())["families"]
    order = json.loads((base / "pokedex-order.json").read_text())["species"]
    numbers = {row["id"]: row["number"] for row in order}
    families = sorted(roster, key=lambda family: numbers[family["members"][0]["id"]])
    keys = [prefix + species["name"].upper() for family in families for species in family["members"]]
    ordered = {key: number for number, key in enumerate((key for key in keys if key in markers), 1)}
    assert len(ordered) == 225 and ordered.keys() == markers and set(ordered.values()) == set(range(1, 226))
    assert new["story_event_script_constants"] == original | {"global": original["global"] | ordered}


def lap(actor):
    """Yield the square and facing when the walker finishes each step."""
    x, y = xy(actor)
    for count, dx, dy in ((actor["move_range_x"], 1, 0), (actor["move_range_y"], 0, 1),
                          (actor["move_range_x"], -1, 0), (actor["move_range_y"], 0, -1)):
        for _ in range(count):
            x, y = x + dx, y + dy
            yield (x, y), (dx, dy)


def check(parent, pack):
    check_archive(parent, pack)
    data = pack["data"]
    check_cabin(parent["data"], data)
    room = data["maps"]["MnStateFairgrounds"]
    prior = parent["data"]["maps"]["MnStateFairgrounds"]
    routes = {r["id"]: r for r in json.loads((HERE / "patrols.json").read_text())}
    assert len(routes) == 6 and room["blocks"] == prior["blocks"]
    bodies = {xy(actor) for actor in room["objects"]}
    actors = {a["script"]: a for a in room["objects"]}
    assert len(bodies) == len(actors) == len(room["objects"]) == 255
    assert data["npcs"][room["id"]] == room["objects"]
    assert data["map_scripts"][room["attributes"]["map_events_label"]] == [
        {"command": row["command"], "args": row["args"]} for row in room["map_event_section_commands"]
    ]
    assert len(room["trainer_scripts"]) == 35 and room["trainer_scripts"] == prior["trainer_scripts"]
    protected = {xy(event) for events in room["events"].values() for event in events}
    stands = json.loads((HERE.parents[1] / "zones" / "state-fairgrounds.json").read_text())
    public = {tuple(v["approach"]) for v in stands["native_map"]["residents"] if v.get("service") == "pokemart"}
    reserved = set()
    floor = {(x, y) for y in range(room["attributes"]["height"] * 2)
             for x in range(room["attributes"]["width"] * 2) if terrain(data, room, (x, y)) == "FLOOR"}
    long_views = {}
    for old in prior["objects"]:
        name = old["script"]
        actor = actors[name]
        movement = actor["spritemovedata"]
        if name in routes:
            spec = routes[name]
            w, h = spec["size_core"]
            expected = old | dict(spritemovedata=MOVES[spec["behavior"]], move_range_x=w, move_range_y=h, radius=8)
            assert actor == expected and actor["sightline_direction_override"] is None, name
            row, = [r for r in room["map_event_section_commands"]
                    if r["command"] == "object_event" and r["args"][11] == name]
            assert row["args"][3:6] == [movement, str(w), str(h)] and row["args"][10] == "8", name
        else:
            assert actor == old, name
        if movement not in MOVES.values():
            continue
        steps = list(lap(actor))
        path = {point for point, _ in steps}
        assert len(path) == len(steps) and steps[-1][0] == xy(actor), name
        assert path <= floor and path.isdisjoint(reserved | protected | public | (bodies - {xy(actor)})), name
        reserved.update(path)
        if name in routes:
            # A complete eight-square view with the Fair crowd at its anchors.
            clear = floor - (bodies - {xy(actor)}) - protected
            views = [(point, direction) for point, direction in steps
                     if all((point[0] + distance * direction[0], point[1] + distance * direction[1]) in clear
                            for distance in range(1, actor["radius"] + 1))]
            assert len(views) >= 2, name
            long_views[name] = len(views)
    assert long_views.keys() == routes.keys()

    # Reserve every square of every moving circuit at once. Any route surviving
    # that stronger obstruction is available at *every* combination of positions.
    clear = floor - reserved - bodies
    assert GATE in clear
    seen = {GATE}
    queue = deque([GATE])
    while queue:
        x, y = queue.popleft()
        for point in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
            if point in clear and point not in seen:
                seen.add(point)
                queue.append(point)
    assert len(public) == 32 and public <= seen, sorted(public - seen)
    VENDORS["check_fair"](data)  # actual staff, counters, inventories and prices
    return long_views, len(reserved)


def check_cabin(before, after):
    name = "MnR15C00R00"
    old, room = (data["maps"][name] for data in (before, after))
    width = room["attributes"]["width"]
    patches = [(24, 4, 0x0f, 0x6d), (24, 8, 0x77, 0x01), (24, 9, 0x34, 0x0f)]
    blocks = list(old["blocks"])
    for x, y, source, dest in patches:
        assert blocks[y * width + x] == source
        blocks[y * width + x] = dest
    assert room == old | {"blocks": blocks}, "the underpass, creel, pier and cabin events must stay intact"
    assert list(base64.b64decode(after["map_blocks"][room["attributes"]["blocks_label"]])) == blocks
    traits = room["attributes"]["tileset_name"]
    assert before["tilesets"][traits] == after["tilesets"][traits]
    assert before["field_moves"]["cut"] == after["field_moves"]["cut"]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("pack", type=Path)
    args = parser.parse_args()
    views, count = check(read_pack(args.parent), read_pack(args.pack))
    print(f"{args.pack.name}: 3 changed maps, 1 held pie, 225 regional numbers, "
          f"6 trainer circuits, {sum(views.values())} eight-square sightings, "
          f"32 public stalls accessible with all {count} moving squares reserved")
