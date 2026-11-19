#!/usr/bin/env python3
"""Check the written Fen, its actual exam squares and both waters around the trail."""

import argparse
import base64
from collections import deque
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import SourceArt, load_pack

OLD = runpy.run_path(str(HERE.parent / "fen-trail-v01/check-fen.py"))
FEN, WOODS = OLD["FEN"], OLD["WOODS"]
Trail = OLD["Trail"]
ART = "data/tilesets/johto_metatiles.bin"
SHEET = "gfx/tilesets/johto.2bpp"
ANIM = "data/tilesets/johto_animations.json"
CORNERS = {0x74: 0x5C, 0x75: 0x5D, 0x76: 0x4D, 0x77: 0x4B}
DIRECTIONS = ((0, -1), (-1, 0), (1, 0), (0, 1))


def require(ok, message):
    if not ok:
        raise ValueError(message)


def connected(positions, start):
    found, queue = {start}, deque([start])
    while queue:
        x, y = queue.popleft()
        for dx, dy in DIRECTIONS:
            point = x + dx, y + dy
            if point in positions and point not in found:
                found.add(point)
                queue.append(point)
    return found


def terrain(data, room):
    grid = Trail(data, [FEN]).grid[FEN]
    water = {
        (x, y)
        for y, row in enumerate(grid)
        for x, role in enumerate(row)
        if role == "WATER"
    }
    mask = (HERE / "fen-water.txt").read_text().splitlines()
    require(
        water
        == {(x, y) for y, r in enumerate(mask) for x, c in enumerate(r) if c == "~"},
        "actual water permissions differ from authored outlines",
    )
    pools = [connected(water, p) for p in ((10, 4), (12, 18))]
    require(
        not pools[0] & pools[1] and pools[0] | pools[1] == water,
        "expected two distinct pools",
    )
    require(
        not any(x in (0, 39) or y in (0, 25) for x, y in water),
        "water opens off the map",
    )
    require(
        all((x, y) in water for x in range(23, 27) for y in range(18, 21)),
        "obstructed lower strait",
    )
    blocks = room["blocks"]
    require(
        (blocks[6 * 20 + 6], blocks[9 * 20 + 6], blocks[6 * 20 + 15])
        == (0x61, 0x54, 0x3C),
        "wrong original root, reflective cove or eastern fir",
    )
    return grid, pools


def art(parent, child, stock, room, grid):
    old, new = parent["runtime_files"], child["runtime_files"]
    atlas = bytes(new[ART])
    require(atlas.startswith(bytes(old[ART])), "replaced an established Johto block")
    require(new[ANIM] == old[ANIM], "changed Johto animation timing or operands")
    sheet, before = bytes(new[SHEET]), bytes(old[SHEET])
    require(len(sheet) == len(before) == 4096, "unexpected native Johto page size")
    old_col = parent["data"]["tilesets"]["johto"]["collision"]
    new_col = child["data"]["tilesets"]["johto"]["collision"]
    require(
        all(new_col[k] == v for k, v in old_col.items())
        and json.loads(bytes(new["data/tilesets/johto.json"])) == new_col,
        "changed preexisting permissions or mismatched native bank mirror",
    )
    palette = child["data"]["tilesets"]["johto"]["palette_map"]
    original = parent["data"]["tilesets"]["johto"]["palette_map"]
    require(
        json.loads(bytes(new["data/tilesets/johto_palette_map.json"])) == palette,
        "native palette mirror",
    )
    require(len(palette) == len(original), "Johto palette map length")
    for i in range(256):
        glyph = sheet[i * 16 : (i + 1) * 16]
        if i in CORNERS:
            j = CORNERS[i]
            want = [before[k : k + 2] for k in range(j * 16, (j + 1) * 16, 2)]
            require(
                glyph == b"".join(reversed(want)) and palette[i] == original[j],
                "incorrect curvature, source palette or native row order",
            )
        else:
            require(
                glyph == before[i * 16 : (i + 1) * 16]
                and (i >= len(palette) or palette[i] == original[i]),
                f"unrelated Johto native pixels changed at {i:#x}",
            )
    blocks = room["blocks"]
    ground = [
        [int(t, 16) for t in line.split()]
        for line in (HERE / "fen-ground.txt").read_text().splitlines()
    ]
    for y, row in enumerate(grid):
        for x, role in enumerate(row):
            if role == "WATER":
                continue
            q = y % 2 * 2 + x % 2
            dry = ground[y // 2][x // 2]
            block = blocks[y // 2 * 20 + x // 2]
            off = (q // 2) * 8 + (q % 2) * 2
            indices = (off, off + 1, off + 4, off + 5)
            require(
                role == old_col[f"{dry:02x}"][q]
                and all(
                    atlas[16 * block + d] == old[ART][16 * dry + d] for d in indices
                ),
                f"wrong native dry quadrant at {x},{y}",
            )

    def glyphs(x, y):
        block = blocks[(y // 2) * 20 + x // 2]
        off = block * 16 + y % 2 * 8 + x % 2 * 2
        return [atlas[off + d] for d in (0, 1, 4, 5)]

    # At the narrowest portion, genuine low gravel lies precisely at the two drawable edges.
    # Open water between banks must be animated, including the former projecting point.
    require(
        glyphs(24, 19) == [0x14] * 4
        and glyphs(25, 19) == [0x14] * 4
        and glyphs(24, 20)[2:] == [0x2C, 0x2C]
        and glyphs(25, 20)[2:] == [0x2C, 0x2C],
        "stray geometry or still water at the narrows",
    )
    a, b = SourceArt(stock), SourceArt(child)
    for time in ("morn", "day", "nite"):
        for site in (0x61, 0x54):
            require(
                a.blocks("johto", time)[site].tobytes()
                == b.blocks("johto", time)[site].tobytes(),
                f"original sapling or rounded cove pixels altered at {time}",
            )


def paths(data, room, grid, pools):
    start = (FEN, 39, 16)
    trails = Trail(data, [FEN, WOODS])
    for y in (16, 17):
        require(
            trails.step((FEN, 39, y), "RIGHT") == (WOODS, 0, y)
            and trails.step((WOODS, 0, y), "LEFT") == (FEN, 39, y),
            "blocked Northwoods seam",
        )
    require(trails.both(start, (FEN, 36, 16), dry=True) == (3, 3), "observer approach")
    required = set()
    for _, _, (x, y) in OLD["SITES"]:
        directions = trails.both(start, (FEN, x, y), dry=True)
        require(
            directions[0] == directions[1] and directions[0] > 0,
            "unreachable observation stance",
        )
        required.add((x, y))
    dry = {
        (x, y)
        for y, row in enumerate(grid)
        for x, role in enumerate(row)
        if role in ("FLOOR", "TALL_GRASS")
    }
    dry -= trails.bodies[FEN]
    public = connected(dry, (39, 16))
    require(required <= public, "disconnected observation")
    # Both independent sides of the southern lake connect the entrance to the western cove.
    # Excluding the north bank forces a southern walk, and excluding the southern bank forces the northern walk.
    for edge in (
        lambda x, y: y <= 17 and 15 <= x <= 19,
        lambda x, y: y >= 22 and 15 <= x <= 19,
    ):
        require(
            (11, 20) in connected({p for p in dry if not edge(*p)}, (39, 16)),
            "broken lake circuit",
        )
    contacts = {(e["x"], e["y"]) for e in room["events"]["bg_events"]}
    shores = []
    for pool in pools:
        boarding = [
            (x, y)
            for x, y in pool
            if (x, y) not in contacts
            for dx, dy in DIRECTIONS
            if (x + dx, y + dy) in public
        ]
        require(
            len(boarding) > 0
            and len(connected(public | pool, (39, 16)) - public) == len(pool),
            "inaccessible fishing/Surf shore",
        )
        shores.append(len(boarding))
    adjacent = {(31, 5), (33, 5), (32, 4), (32, 6)}
    require(
        not (adjacent & connected(dry | set.union(*pools), (39, 16))),
        "premature access on foot or over either pool",
    )
    (change,) = room["script_block_changes"]
    require(
        (change["x"], change["y"], change["block_id"]) == (26, 8, 1)
        and room["blocks"][4 * 20 + 13] == 5,
        "invalid public entrance to hollow",
    )
    opened_blocks = room["blocks"][:]
    opened_blocks[4 * 20 + 13] = 1
    opened = Trail(
        dict(data, maps={**data["maps"], FEN: dict(room, blocks=opened_blocks)}),
        [FEN, WOODS],
    )
    out = opened.both(start, (FEN, 31, 5), dry=True)
    require(out[0] == out[1], "one-way hidden clearing")
    print(
        f"Open pools of {len(pools[0])}/{len(pools[1])} walking squares with {shores[0]}/{shores[1]} two-way shore contacts. "
        f"Six examination approaches and both paths around the lower pool pass. Hollow remains sealed until reveal, "
        f"then {out[0]} dry steps each way from the east seam."
    )


def check(parent, child, original):
    old, new = (p["data"]["maps"][FEN] for p in (parent, child))
    require(
        all(
            old[k] == new[k]
            for k in ("events", "objects", "attributes", "script_block_changes")
        ),
        "changed events, actor identities, world connections or hidden gate",
    )
    require(
        parent["data"]["wild_encounters"].get(FEN)
        == child["data"]["wild_encounters"].get(FEN),
        "changed encounter tables",
    )
    require(
        base64.b64decode(child["data"]["map_blocks"][new["attributes"]["blocks_label"]])
        == bytes(new["blocks"]),
        "split room layout disagrees with native map",
    )
    grid, pools = terrain(child["data"], new)
    art(parent, child, original, new, grid)
    paths(child["data"], new, grid, pools)
    print(
        "Verified original sapling and rounded pond at three clocks, all original native Johto blocks and animation, "
        "all authored dry quadrants, curve orientations and the open-water neck."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("parent", "candidate", "original"):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    check(*(load_pack(p) for p in (args.parent, args.candidate, args.original)))
