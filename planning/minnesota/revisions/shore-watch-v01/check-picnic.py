#!/usr/bin/env python3
"""Walk the Mille Lacs park, enter and leave its cove, and verify the native picnic collectible."""

import argparse
from collections import deque
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import SourceArt, load_pack
from review_grand_marais import sprite

PARK, BANK = "MnMilleLacs", "mn_headwaters"
ATLAS = "data/tilesets/mn_headwaters_metatiles.bin"
FLAG, PICKUP = "EVENT_MN_HIDDEN_MILLE_LACS_PICNIC_LURE", "MnMlPicnicLure"
PICNIC, GREETING, HINT = (
    "MN_ML_PARK_PICNIC",
    "MnMlPicnicLunchText",
    "MnMlPicnicSpareBallText",
)
SITE, ACCESS = (18, 34), (18, 35)
COVE = json.loads((HERE / "picnic-cove.json").read_text())
DIRECTIONS = ((0, -1), (1, 0), (0, 1), (-1, 0))
DRY = {"FLOOR", "TALL_GRASS"}


def grid(data, name=PARK):
    room = data["maps"][name]
    a = room["attributes"]
    cols = data["tilesets"][a["tileset_name"]]["collision"]
    return {
        (x, y): cols[f"{room['blocks'][y // 2 * a['width'] + x // 2]:02x}"][
            (y % 2) * 2 + x % 2
        ]
        for y in range(2 * a["height"])
        for x in range(2 * a["width"])
    }


def path(g, old, kinds, blocked=()):
    dist = {old: 0}
    q = deque([old])
    while q:
        x, y = q.popleft()
        for dx, dy in DIRECTIONS:
            p = (x + dx, y + dy)
            if g.get(p) == "HOP_DOWN" and (dx, dy) == (0, 1):
                p = (x, y + 2)
            if p not in dist and p not in blocked and g.get(p) in kinds:
                dist[p] = 1 + dist[x, y]
                q.append(p)
    return dist


def neighbours(p):
    return [(p[0] + dx, p[1] + dy) for dx, dy in DIRECTIONS]


def silhouettes(pack, names):
    covered = set()
    for name in names:
        for pose in range(6):
            image = sprite(pack, name, frame_offset=16 * pose)
            for x in range(16):
                for y in range(16):
                    if image.getpixel((x, y))[3]:
                        covered.add((x, y))
                        covered.add((15 - x, y))
    return covered


def crossing_pixels(pack, g, raw):
    """Sweep all opaque rider/boat and pedestrian pixels between consecutive standing squares."""
    boots = silhouettes(pack, ("CHRIS", "KRIS"))
    boats = silhouettes(pack, ("MN_CANOE_CHRIS", "MN_CANOE_KRIS"))
    require(
        boots and boats and boats != boots, "original walkers and canoe silhouettes"
    )
    segments = 0
    for p in raw:
        for q in neighbours(p):
            if (
                q not in raw
                or g[p] not in {"FLOOR", "WATER"}
                or g[q] not in {"FLOOR", "WATER"}
            ):
                continue
            segments += 1
            for step in range(17):
                anchor_x = p[0] * 16 + (q[0] - p[0]) * step
                anchor_y = p[1] * 16 + (q[1] - p[1]) * step
                # The bottom center of the visible soles or keel determines the
                # displayed sprite on a step crossing the physical shoreline.
                contact = ((anchor_x + 8) // 16, (anchor_y + 15) // 16)
                shape = boats if g[contact] == "WATER" else boots
                for i, j in shape:
                    x, y = anchor_x + i, anchor_y + j
                    cell = (x // 16, y // 16)
                    glyph = raw[cell][0][(y % 16 // 8) * 2 + x % 16 // 8]
                    if (glyph == 0x14) != (g[cell] == "WATER"):
                        raise AssertionError(
                            f"{p} to {q}, pixel {step}, "
                            f"actor offset {(i, j)}: artwork disagrees with {g[cell]}"
                        )
    return segments


def cove_footprints(pack, room, g, before, foot, water):
    """Measure the boat and boot standing squares against the actual native 2D artwork."""
    art = SourceArt(pack).blocks(BANK)
    width = room["attributes"]["width"]
    atlas = bytes(pack["runtime_files"][ATLAS])
    # The stock open-water tile has blue fill and pale ripple pixels.
    patch = art[0x35].crop((0, 0, 16, 16))
    blue = max(patch.getcolors())[1]
    colors = {rgb for _, rgb in patch.getcolors()}
    require(
        len(colors) == 2 and blue[2] > blue[0],
        "the park's actual daylight water palette",
    )

    def measure(p):
        x, y = p
        block = room["blocks"][y // 2 * width + x // 2]
        sx, sy = x % 2 * 2, y % 2 * 2
        ids = [
            atlas[block * 16 + (sy + dy) * 4 + sx + dx]
            for dy in (0, 1)
            for dx in (0, 1)
        ]
        pix = art[block].crop((8 * sx, 8 * sy, 8 * sx + 16, 8 * sy + 16)).getcolors()
        return (
            ids,
            sum(n for n, rgb in pix if rgb == blue),
            sum(n for n, rgb in pix if rgb in colors),
        )

    window = {(x, y) for y in range(23, 36) for x in range(14, 22)}
    raw = {p: measure(p) for p in window}
    converted = {p for p in window if before[p] == "FLOOR" and g[p] == "WATER"}
    require(
        len(converted) == 15,
        "all newly occupied canoe squares are inside the checked footprint",
    )
    for p in window:
        glyphs, blue_pixels, water_pixels = raw[p]
        if g[p] == "FLOOR":
            require(
                0x14 not in glyphs and blue_pixels == 0 and p in foot,
                f"walking on lake art or disconnected ground at {p}",
            )
        elif g[p] == "WATER":
            require(
                p in water and glyphs == [0x14] * 4 and water_pixels == 256,
                f"the canoe footprint sits on the bank at {p}: {water_pixels}/256 lake",
            )
    crossings = {
        (p, q)
        for p in window
        if g[p] == "FLOOR"
        for q in neighbours(p)
        if q in window and g[q] == "WATER"
    }
    require(
        len(crossings) == 20 and all(p in foot and q in water for p, q in crossings),
        f"all cove launches and portages must be reachable without walking through water: {len(crossings)}",
    )
    for p, q in [((16, 30), (17, 30)), ((17, 28), (17, 29)), ((19, 34), (19, 33))]:
        require((p, q) in crossings, f"verified boat launch and return at {p} / {q}")
    require(
        raw[16, 30][0] == [0x06] * 4 and raw[17, 30][0] == raw[18, 30][0] == [0x14] * 4,
        "the picnic launch needs a complete dirt landing and two full water tiles",
    )
    segments = crossing_pixels(pack, g, raw)
    print(
        f"Cove footprints: {len(crossings)} two-way shore contacts, all 256/256 lake pixels. "
        f"Full native boots/boat silhouettes agree over {segments} directed segments in all six "
        "poses and mirrored, at every pixel of each stride. All dry squares are free of lake blue. "
        "(16,30) east to (17,30), paddle to (18,30); (19,33) south to the flower point."
    )


def park(parent, candidate=None):
    data = (candidate or parent)["data"]
    old = parent["data"]
    room = data["maps"][PARK]
    former = old["maps"][PARK]
    g, before = grid(data), grid(old)
    route = data["maps"]["MnR18C01R00"]
    require(
        room["objects"] == former["objects"]
        and room["events"]["warps"] == []
        and room["events"]["coord_events"] == []
        and not room["trainer_scripts"],
        "quiet park, places and real border crossing",
    )
    require(
        room["attributes"]["connections"]
        == [{"direction": "west", "target_map": "MnR18C01R00", "offset": 1}]
        and any(
            e["direction"] == "east" and e["target_map"] == PARK and e["offset"] == -1
            for e in route["attributes"]["connections"]
        ),
        "public Route 18 walking seam",
    )
    actors = {(o["x"], o["y"]) for o in room["objects"]}
    spawn = data["runtime_spawn_points"][
        str(
            data["fly_destinations"]["ENGINE_FLYPOINT_MN_MILLE_LACS"][
                "destination_spawn_identifier"
            ]
        )
    ]
    require(
        (spawn["mapName"], spawn["tileX"], spawn["tileY"]) == (PARK, 2, 12),
        "registered Fly return",
    )
    grass = lambda G: {p for p, v in G.items() if v == "TALL_GRASS"}
    wet = lambda G: {p for p, v in G.items() if v == "WATER"}
    require(
        grass(g) == grass(before) and len(grass(g)) == 48 and wet(before) <= wet(g),
        "changed battle grass or stranded a previously afloat player",
    )
    immutable = {p for p, v in before.items() if v in {"BUOY", "HOP_DOWN"}}
    require(
        all(g[p] == before[p] for p in immutable)
        and old["wild_encounters"][PARK] == data["wild_encounters"][PARK],
        "buoys, bird hide ledge and seasonal/local wild tables",
    )
    water = path(g, (30, 22), {"WATER"})
    require(
        set(water) == wet(g) and len(water) == (566 if candidate else 551),
        "connected Surf domain",
    )
    road = grid(data, "MnR18C01R00")
    for y in [12, 13]:
        require(
            g[0, y] in DRY and road[route["attributes"]["width"] * 2 - 1, y - 2] in DRY,
            "two corresponding dry approach lanes",
        )
    foot = path(g, (2, 12), DRY, actors)
    for p in (
        (0, 12),
        (0, 13),
        ACCESS,
        (26, 23),
        (39, 10),
        (16, 16),
        (12, 30),
        (24, 7),
    ):
        require(p in foot and (2, 12) in path(g, p, DRY, actors), f"dry round trip {p}")
    roost = next(o for o in room["objects"] if o["script"] == "MN_ML_PARK_ROOST")
    target = (roost["x"], roost["y"])
    afloat = path(g, (30, 22), DRY | {"WATER"}, actors)
    require(
        instructions(room, roost["script"]) == [("itemball", "PP_UP", "1")]
        and all(p not in foot for p in neighbours(target))
        and any(p in afloat for p in neighbours(target)),
        "original offshore reward requires water travel",
    )
    if not candidate:
        require(
            all(b["event_type"] != "BGEVENT_ITEM" for b in room["events"]["bg_events"]),
            "parent hidden sites",
        )
        print(
            f"Existing park: {len(water)} connected water tiles, {len(grass(g))} grass, and a PP Up reachable by canoe or Surf."
        )
        return
    fixed = bytes(candidate["runtime_files"][ATLAS])
    base = bytes(parent["runtime_files"][ATLAS])
    require(
        fixed.startswith(base) and len(fixed) % 16 == 0,
        "changed an existing shared block",
    )
    conditions = data["tilesets"][BANK]["collision"]
    require(
        all(conditions[k] == v for k, v in old["tilesets"][BANK]["collision"].items()),
        "old shared collision",
    )
    # Route 18 authors can append to this same bank. Locate our consecutive block recipes by their actual bytes and permissions.
    art = b"".join(bytes(q["tiles"]) for q in COVE["recipes"])
    offsets = [
        i for i in range(len(base), len(fixed), 16) if fixed[i : i + len(art)] == art
    ]
    require(
        len(offsets) == 1, "identify the picnic shoreline after the parent's metatiles"
    )
    begin = offsets[0] // 16
    for number, entry in enumerate(COVE["recipes"], begin):
        require(
            conditions[f"{number:02x}"] == entry["collision"],
            f"cove collision {number}",
        )
    require(
        json.loads(
            bytes(candidate["runtime_files"]["data/tilesets/mn_headwaters.json"])
        )
        == conditions,
        "shared native shore collision mirror",
    )
    layout = former["blocks"].copy()
    for (x, y), was, dest in COVE["placements"]:
        index = y * room["attributes"]["width"] + x
        require(layout[index] == was, "cove parent placement")
        layout[index] = begin + dest - 1000 if dest >= 1000 else dest
    require(
        room["blocks"] == layout and len(wet(g) - wet(before)) == 15,
        "fifteen new shallows, authored landform only",
    )
    cove_footprints(candidate, room, g, before, foot, water)
    for p, q in [
        ((16, 30), (17, 30)),
        ((18, 33), (19, 33)),
        ((19, 34), (19, 33)),
        ((26, 23), (26, 24)),
    ]:
        require(
            p in foot and q in water and q in neighbours(p),
            f"can board and come ashore {p} to {q}",
        )
    at = next(o for o in room["events"]["bg_events"] if (o["x"], o["y"]) == SITE)
    require(
        at == dict(x=18, y=34, event_type="BGEVENT_ITEM", script=PICKUP)
        and room["events"]["bg_events"] == former["events"]["bg_events"] + [at],
        "one flower-point search",
    )
    require(
        instructions(room, PICKUP) == [("hiddenitem", "LURE_BALL", FLAG)]
        and SITE in neighbours(ACCESS)
        and g[SITE] == "FLOOR",
        "native facing pickup on dry land",
    )
    bx, by = SITE[0] // 2, SITE[1] // 2
    block = layout[by * 24 + bx]
    flower = [
        fixed[16 * block + 4 * ((SITE[1] % 2) * 2 + dy) + (SITE[0] % 2) * 2 + dx]
        for dy in (0, 1)
        for dx in (0, 1)
    ]
    require(
        3 in flower,
        "the item square must really display the same native flower as the meadow",
    )
    pickups = [
        x
        for m in data["maps"].values()
        for x in m["script_field_pickups"]
        if x.get("event_flag") == FLAG
    ]
    require(
        len(pickups) == 1
        and (pickups[0]["source_script"], pickups[0]["quantity"], pickups[0]["item_id"])
        == (PICKUP, 1, "LURE_BALL")
        and FLAG in data["story_event_script_constants"]["global"],
        "Itemfinder and receipt",
    )
    for claimed, spoken in [(False, [GREETING, HINT]), (True, [GREETING])]:
        flags = {FLAG} if claimed else set()
        _, pages, after = read_branch(room, PICNIC, flags)
        require(pages == spoken and after == flags, f"picnic return {claimed}")
    for name, key in [(GREETING, "picnic_pages"), (HINT, "clue_pages")]:
        require(
            native_pages(room["script_text_bodies"][name]["commands"]) == COVE[key]
            and room["scripts"][name] == data["map_scripts"][name],
            "compiled picnic pages",
        )
    for n in (
        "MN_ML_PARK_ROOST",
        "MN_ML_PARK_RANGER",
        "MN_ML_PARK_BOATER",
        "MN_ML_PARK_ANGLER",
        "MN_ML_PARK_BIRDER",
    ):
        require(
            room["scripts"][n] == former["scripts"][n], f"unrelated park visitor {n}"
        )
    print(
        f"Picnic flowers and Lure Ball: {foot[ACCESS]} steps from Fly, {len(water)} connected water tiles, 15 new. "
        "The offshore PP Up, dry approaches to all five residents, and both Route 18 lanes pass."
    )


if __name__ == "__main__":
    opt = argparse.ArgumentParser(description=__doc__)
    opt.add_argument("parent", type=Path)
    opt.add_argument("candidate", nargs="?", type=Path)
    a = opt.parse_args()
    old = load_pack(a.parent)
    park(old, load_pack(a.candidate) if a.candidate else None)
