#!/usr/bin/env python3
"""Check each ski departure, return, the existing ice crossing and the walk to the stamp."""

import argparse
import base64
from collections import deque
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import native_pages, require
from render_reference import load_pack

TOWN = "MnInternationalFalls"
ICE = "MnRainyRiverIcefields"
WEST = "MnR20C00R00"
EAST = "MnR20C01R00"
CAMP = "MnBoundaryWaters"
SITES = json.loads((HERE / "ski-crossing.json").read_text())
DIRECTIONS = {"UP": (0, -1), "RIGHT": (1, 0), "DOWN": (0, 1), "LEFT": (-1, 0)}
SIDES = dict(UP="north", RIGHT="east", DOWN="south", LEFT="west")
WALK = {"FLOOR", "TALL_GRASS", "ICE", "BRIDGE_EW_DECK", "BRIDGE_NS_DECK", "UP_WALL"}


class Trail:
    def __init__(self, data, maps):
        self.maps = maps
        self.grid = {}
        self.bodies = {}
        self.conns = {}
        for name in maps:
            room = data["maps"][name]
            a = room["attributes"]
            w, h = a["width"], a["height"]
            collision = data["tilesets"][a["tileset_name"]]["collision"]
            blocks = [collision[f"{block:02x}"] for block in room["blocks"]]
            self.grid[name] = [
                [
                    blocks[(y // 2) * w + x // 2][2 * (y % 2) + x % 2]
                    for x in range(2 * w)
                ]
                for y in range(2 * h)
            ]
            self.bodies[name] = {(o["x"], o["y"]) for o in room["objects"]}
            self.conns[name] = {e["direction"]: e for e in a["connections"]}

    def kind(self, name, x, y):
        grid = self.grid[name]
        if 0 <= y < len(grid) and 0 <= x < len(grid[0]):
            return grid[y][x]
        return "OUTSIDE"

    def step(self, node, direction, dry=False):
        name, x, y = node
        dx, dy = DIRECTIONS[direction]
        old = self.kind(name, x, y)
        x, y = x + dx, y + dy
        if self.kind(name, x, y) == "OUTSIDE":
            link = self.conns[name].get(SIDES[direction])
            if not link or link["target_map"] not in self.maps:
                return None
            name = link["target_map"]
            w, h = len(self.grid[name][0]), len(self.grid[name])
            off = 2 * link["offset"]
            if dx:
                x, y = (0 if dx > 0 else w - 1), y - off
            else:
                x, y = x - off, (0 if dy > 0 else h - 1)
        kind = self.kind(name, x, y)
        if (x, y) in self.bodies[name] or kind not in WALK or (dry and kind == "ICE"):
            return None
        if (old == "UP_WALL" and dy < 0) or (kind == "UP_WALL" and dy > 0):
            return None
        return name, x, y

    def move(self, node, direction, dry=False):
        target = self.step(node, direction, dry)
        if target is None:
            return None
        while self.kind(*target) == "ICE":
            onward = self.step(target, direction, dry)
            if onward is None:
                break
            target = onward
        return target

    def path(self, start, end, dry=False):
        queue = deque([start])
        parents = {start: None}
        while queue:
            at = queue.popleft()
            if at == end:
                path = []
                while parents[at] is not None:
                    at, d = parents[at]
                    path.append(d)
                return list(reversed(path))
            for direction in DIRECTIONS:
                target = self.move(at, direction, dry)
                if target is not None and target not in parents:
                    parents[target] = (at, direction)
                    queue.append(target)
        raise ValueError(f"no {'dry ' if dry else ''}path from {start} to {end}")

    def both(self, a, b, dry=False):
        return (len(self.path(a, b, dry)), len(self.path(b, a, dry)))


def check_topology(old, new):
    for n, r in new["maps"].items():
        require(
            old["maps"][n]["attributes"]["connections"]
            == r["attributes"]["connections"],
            f"invented world seam at {n}",
        )
    for s in SITES:
        n = s["map"]
        r = new["maps"][n]
        p = old["maps"][n]
        a = r["attributes"]
        x, y = s["snow_turnout"]["at"]
        index = y * a["width"] + x
        expected = p["blocks"].copy()
        require(expected[index] == s["snow_turnout"]["from"], f"{n} forest preimage")
        expected[index] = s["snow_turnout"]["to"]
        require(
            expected == r["blocks"] and a == p["attributes"],
            f"{n} changed beyond its turnout",
        )
        require(
            bytes(expected) == base64.b64decode(new["map_blocks"][a["blocks_label"]]),
            f"{n} transport blocks",
        )
        require(
            p["events"] == r["events"] and p["objects"] == r["objects"][:-1],
            f"{n} changed an existing event",
        )
        require(new["npcs"][n] == r["objects"], f"{n} split objects")
        require(
            old["tilesets"][s["tileset"]] == new["tilesets"][s["tileset"]],
            f"{n} changed native art",
        )
        require(
            old["wild_encounters"].get(n) == new["wild_encounters"].get(n),
            f"{n} changed wildlife",
        )
        event = a["map_events_label"]
        source = new["map_scripts"][event]
        if event in r["scripts"]:
            require(source == r["scripts"][event], f"{n} native object source mismatch")
        require(
            source[-1]["args"][11] == s["guide"],
            f"{n} guide missing from native events",
        )
    for n in (WEST, EAST, "MnR20C00R01", "MnR20C01R01", CAMP):
        for k in ("blocks", "events", "objects", "attributes"):
            require(
                old["maps"][n][k] == new["maps"][n][k],
                f"changed existing circuit {n}/{k}",
            )


def exercise_guide(data, trail, here, there):
    name = here["map"]
    room = data["maps"][name]
    guide = here["guide"]
    x, y = here["stand"]
    actor = room["objects"][-1]
    require(
        (
            actor["x"],
            actor["y"],
            actor["script"],
            actor["object_identifier"],
            actor["object_type"],
            actor["event_flag"],
            actor["radius"],
        )
        == (x, y, guide, guide, "OBJECTTYPE_SCRIPT", "-1", 0),
        f"public guide {guide}",
    )
    require(
        native_pages(room["scripts"][guide + "OfferText"]) == here["offer"],
        "actual crossing offer",
    )
    require(
        room["script_text_bodies"][guide + "OfferText"]["commands"]
        == [
            dict(row, command_index=i)
            for i, row in enumerate(room["scripts"][guide + "OfferText"])
        ],
        "compiled ski offer",
    )
    for label, body in room["scripts"].items():
        if label.startswith(guide):
            require(data["map_scripts"][label] == body, f"{label} global program")

    def execute(approach, facing, accept):
        player = approach
        leader = (x, y)
        heading = None
        following = False
        warp = None
        text = False
        condition = None
        label = guide
        for _ in range(9):
            for idx, row in enumerate(room["scripts"][label]):
                op, args = row["command"], row["args"]
                if op in ("opentext", "closetext", "faceplayer"):
                    continue
                if op == "writetext":
                    require(args == [guide + "OfferText"], "unexpected guide text")
                    text = True
                elif op == "yesorno":
                    require(text, "missing travel offer")
                    condition = accept
                elif op == "readvar":
                    require(args == ["VAR_FACING"], "incorrect facing test")
                    heading = facing
                elif op in ("sjump", "iffalse", "ifequal"):
                    if (
                        op == "sjump"
                        or (op == "iffalse" and not condition)
                        or (op == "ifequal" and heading == args[0])
                    ):
                        matches = [
                            c
                            for c in room["script_control_commands"]
                            if c["source_script"] == label and c["command_index"] == idx
                        ]
                        require(
                            len(matches) == 1
                            and matches[0]["resolved_target_script"] == args[-1],
                            f"unlinked {label}/{idx}",
                        )
                        label = args[-1]
                        break
                elif op == "follow":
                    require(
                        args == [guide, "PLAYER"] and not following,
                        "invalid group departure",
                    )
                    following = True
                elif op == "applymovement":
                    require(following and args[0] == guide, "guide must lead")
                    movement = room["scripts"][args[1]]
                    for z in movement:
                        if z["command"] == "step_end":
                            break
                        require(z["command"] == "step", "unexpected departure movement")
                        dx, dy = DIRECTIONS[z["args"][0]]
                        nextpos = leader[0] + dx, leader[1] + dy
                        require(
                            nextpos != player
                            and nextpos not in trail.bodies[name]
                            and trail.kind(name, *nextpos) == "FLOOR",
                            "guide would walk into the player, a tree or another actor",
                        )
                        require(
                            abs(leader[0] - player[0]) + abs(leader[1] - player[1])
                            == 1,
                            "broken follower stride",
                        )
                        player, leader = leader, nextpos
                elif op == "stopfollow":
                    require(following, "missing follower")
                    following = False
                elif op == "warpfacing":
                    records = [
                        c
                        for c in room["script_map_commands"]
                        if c["source_script"] == label and c["command_index"] == idx
                    ]
                    require(len(records) == 1 and not following, "unresolved ski trip")
                    record = records[0]
                    dest = there["map"]
                    require(
                        record["target_map"] == dest
                        and [record["x"], record["y"]] == there["arrival"]
                        and record["facing"] == there["facing"],
                        "incorrect native destination",
                    )
                    require(
                        args
                        == [
                            data["maps"][dest]["attributes"]["map_constant"],
                            *map(str, there["arrival"]),
                            there["facing"],
                        ],
                        "mismatched raw warp",
                    )
                    warp = (dest, *there["arrival"])
                elif op == "end":
                    require(
                        warp == (there["map"], *there["arrival"])
                        if accept
                        else warp is None and player == approach and leader == (x, y),
                        "incorrect accept or decline",
                    )
                    return
                else:
                    raise ValueError(f"unauthorized ski travel effect {op}")
            else:
                raise ValueError(f"unterminated program {label}")
        raise ValueError("looping ski guide")

    approaches = 0
    for facing, (dx, dy) in DIRECTIONS.items():
        at = (x - dx, y - dy)
        if trail.kind(name, *at) != "FLOOR" or at in trail.bodies[name]:
            continue
        for answer in (False, True):
            execute(at, facing, answer)
        approaches += 1
    require(
        approaches == 3,
        f"{guide}: expected 3 open conversation positions, found {approaches}",
    )
    print(
        f"{guide} verifies accept and decline from {approaches} sides; arrives {there['map']} {there['arrival']}"
    )


def walk_checks(data):
    route_names = {TOWN, ICE, WEST, EAST, "MnR20C00R01", "MnR20C01R01", CAMP}
    trail = Trail(data, route_names)
    walks = [
        ("Falls clinic to ski guide", (TOWN, 9, 8), (TOWN, 27, 14)),
        ("Falls arrival to clinic", (TOWN, 28, 15), (TOWN, 9, 8)),
        ("hut to return guide", (ICE, 9, 28), (ICE, 6, 28)),
        ("ski arrival to hut", (ICE, 5, 29), (ICE, 9, 28)),
        ("hut to west ice landing", (ICE, 9, 28), (ICE, 37, 36)),
        ("east landing to R20 warden", (ICE, 54, 36), (ICE, 61, 45)),
        ("R20 west to eastbound stamp", (WEST, 0, 45), (CAMP, 0, 25)),
    ]
    for desc, a, b in walks:
        print(desc, trail.both(a, b))
    require(
        trail.move((ICE, 37, 36), "RIGHT") == (ICE, 54, 36)
        and trail.move((ICE, 54, 36), "LEFT") == (ICE, 37, 36),
        "lost original reciprocal river slide",
    )
    print(
        "Lower Rainy River crossing: 17 squares by ice, dry walking alternative",
        trail.both((ICE, 37, 36), (ICE, 54, 36), True),
    )
    for y in (44, 45):
        require(
            trail.step((ICE, 63, y), "RIGHT") == (WEST, 0, y)
            and trail.step((WEST, 0, y), "LEFT") == (ICE, 63, y),
            "bad icefield/Voyageurs seam",
        )
    for y in (24, 25):
        require(
            trail.step((EAST, 63, y + 12), "RIGHT") == (CAMP, 0, y)
            and trail.step((CAMP, 0, y), "LEFT") == (EAST, 63, y + 12),
            "bad camp stamp arrival",
        )
    for h in SITES:
        name = h["map"]
        a = data["maps"][name]["attributes"]
        for sx, sy in [h["arrival"], h["stand"]]:
            require(trail.kind(name, sx, sy) == "FLOOR", "unsafe departure snow")
        at = h["snow_turnout"]["at"]
        require(
            at[0] not in (0, a["width"] - 1) and at[1] not in (0, a["height"] - 1),
            "turnout must stay inside the established forest rim",
        )
    return trail


def gallery(before, after, out):
    from PIL import Image, ImageDraw
    from render_progress_atlas import native_room

    sheet = Image.new("RGB", (640, 2 * (288 + 30)), "#1a2230")
    draw = ImageDraw.Draw(sheet)
    for row, s in enumerate(SITES):
        for col, pack in enumerate((before, after)):
            room = pack["data"]["maps"][s["map"]]
            im = native_room(room, pack, native_origin=True)
            x, y = s["stand"]
            ox = max(0, min(im.width - 160, 16 * x - 72))
            oy = max(0, min(im.height - 144, 16 * y - 64))
            crop = im.crop((ox, oy, ox + 160, oy + 144)).resize(
                (320, 288), Image.Resampling.NEAREST
            )
            left, top = 320 * col, row * 318
            draw.text(
                (left + 6, top + 6),
                f"{s['map']} | {('Before', 'After')[col]}",
                fill="white",
            )
            sheet.paste(crop, (left, top + 30))
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    parser.add_argument("--gallery", type=Path)
    args = parser.parse_args()
    parent = load_pack(args.parent)
    child = load_pack(args.child)
    old = parent["data"]
    data = child["data"]
    check_topology(old, data)
    trail = walk_checks(data)
    for i, site in enumerate(SITES):
        exercise_guide(data, trail, site, SITES[1 - i])
    if args.gallery:
        gallery(parent, child, args.gallery)


if __name__ == "__main__":
    main()
