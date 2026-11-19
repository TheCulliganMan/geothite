"""Verify a Gym Walk parent and Cave Returns child. Optional --images DIR saves matched native cameras."""

import argparse
import base64
import json
import runpy
import sys
from collections import Counter
from itertools import permutations, product
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
MN = HERE.parents[1]
sys.path.insert(0, str(MN / "tools"))
from audit_story_flow import instructions, native_pages, require
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_north_shore import oriented_sprite

NAV = runpy.run_path(str(HERE.parent / "lake-observations-v01/check-hidden.py"))
ROOM = "MnR03Cave1F"
WALKER = "MN_R03_CAVE_WALKER"
READY = "EVENT_MN_CASTOROCK_TRACKS_READ"
CAUGHT = "EVENT_MN_CASTOROCK_CAUGHT"
MARKS = (
    ("pool", (2, 5), (3, 5), "west", 0x04, 0x50),
    ("pillar", (20, 4), (19, 4), "east", 0x34, 0x54),
    ("bed", (23, 13), (23, 12), "south", 0x21, 0x58),
)
FLAGS = ["EVENT_MN_CASTOROCK_" + name.upper() for name, *_ in MARKS]


def survey(data):
    rooms = data["maps"]
    constants = {r["attributes"]["map_constant"]: n for n, r in rooms.items()}
    doors = set()
    for n, r in rooms.items():
        if not n.startswith("Mn") or r["attributes"]["environment"] not in {
            "TOWN",
            "ROUTE",
        }:
            continue
        for w in r["events"]["warps"]:
            dst = constants.get(w["target_map_constant"])
            if dst and rooms[dst]["attributes"]["environment"] in {"CAVE", "DUNGEON"}:
                doors.add((n, w["x"], w["y"], dst))
    print(
        f"{len(doors)} exterior warp records to {len({w[-1] for w in doors})} Minnesota cave/dungeon rooms (includes return-only records)."
    )
    room, road = rooms[ROOM], rooms["MnR03C00R00"]
    require(
        [
            (w["x"], w["y"], w["target_warp_id"])
            for w in road["events"]["warps"]
            if w["target_map_constant"] == room["attributes"]["map_constant"]
        ]
        == [(8, 37, 1), (8, 38, 1)],
        "Route 3 threshold and landing",
    )
    require(
        [
            (w["x"], w["y"], w["target_warp_id"])
            for w in room["events"]["warps"]
            if w["target_map_constant"] == road["attributes"]["map_constant"]
        ]
        == [(7, 9, 2)],
        "River Pocket exit",
    )
    return room


def movement(data, room):
    def access(pushed, mode):
        staged_room = dict(
            room,
            objects=[
                dict(o, x=15) if pushed and o["script"] == "MN_R03_JAW_BOULDER" else o
                for o in room["objects"]
            ],
        )
        staged = dict(data, maps={**data["maps"], ROOM: staged_room})
        return NAV["approaches"](staged, ROOM, {(7, 8)}, mode)

    (rock,) = [o for o in room["objects"] if o["script"] == "MN_R03_JAW_BOULDER"]
    require(
        (rock["x"], rock["y"], rock["spritemovedata"])
        == (13, 6, "SPRITEMOVEDATA_STRENGTH_BOULDER")
        and instructions(room, rock["script"])
        == [("jumpstd", "StrengthBoulderScript")],
        "real boulder",
    )
    require(
        all(NAV["quadrant"](data, ROOM, x, 6) == "FLOOR" for x in range(12, 16)),
        "two consecutive legal east pushes",
    )
    for mode in ("Walk", "Surf"):
        closed = access(False, mode)
        require(
            (3, 5) in closed
            and (12, 6) in closed
            and (21, 10) not in closed
            and all(not (NAV["adjacent"](q) & closed) for _, q, *_ in MARKS[1:]),
            "unopened Strength throat: " + mode,
        )
    opened = access(True, "Walk")
    require(
        all(spot in opened for _, _, spot, *_ in MARKS) and (21, 10) in opened,
        "three dry searches and creature with an ordinary return",
    )


def walk_program(room, label, state, hm=False):
    state = set(state)
    outcome = False
    stack = []
    pages = []
    calls = []
    index = 0
    for _ in range(180):
        op, *args = instructions(room, label)[index]
        index += 1
        if op == "checkevent":
            outcome = args[0] in state
        elif op == "checkitem":
            require(args == ["HM_STRENGTH"], "unexpected item test")
            outcome = hm
        elif op == "setevent":
            state.add(args[0])
        elif op == "writetext":
            pages.append(args[0])
        elif op == "appear":
            calls.append(args[0])
        elif op == "scall":
            stack.append((label, index))
            label, index = args[0], 0
        elif (
            op == "sjump"
            or (op == "iftrue" and outcome)
            or (op == "iffalse" and not outcome)
        ):
            label, index = args[0], 0
        elif op == "end":
            if not stack:
                return state, pages, calls
            label, index = stack.pop()
        else:
            require(
                op
                in (
                    "opentext",
                    "closetext",
                    "faceplayer",
                    "waitbutton",
                    "iftrue",
                    "iffalse",
                    "cry",
                ),
                f"unexpected {label} op {op}",
            )
    raise AssertionError("unexpected guide/observation loop")


def story(before, after):
    orig, room = (q["maps"][ROOM] for q in (before, after))
    mutable = {
        "blocks",
        "scripts",
        "script_item_checks",
        "script_flag_commands",
        "script_object_commands",
        "script_control_commands",
        "script_text_commands",
        "script_text_bodies",
    }
    require(
        {k: v for k, v in room.items() if k not in mutable}
        == {k: v for k, v in orig.items() if k not in mutable},
        "cave events, warps, encounter, callbacks, pickups or other unrelated metadata",
    )
    require(
        all(room["scripts"][k] == v for k, v in orig["scripts"].items() if k != WALKER),
        "original observation or wild encounter logic changed",
    )
    require(
        instructions(orig, WALKER) == [("jumptextfaceplayer", WALKER + "Text")],
        "unexpected guide preimage",
    )
    faces = [
        row for row in room["script_object_commands"] if row["source_script"] == WALKER
    ]
    require(
        [(row["command"], row["command_index"]) for row in faces] == [("faceplayer", 0)]
        and [
            row
            for row in room["script_object_commands"]
            if row["source_script"] != WALKER
        ]
        == orig["script_object_commands"],
        "guide must face the visitor through its real object command",
    )
    canonical = json.loads((MN / "story.json").read_text())["dialogue_overhaul"][
        "texts"
    ][ROOM]
    added = set(room["scripts"]) - set(orig["scripts"])
    require(
        added
        == {
            "MnJawGuide" + k
            for k in (
                "Pool",
                "Pillar",
                "Found",
                "Caught",
                "Travel",
                "Finish",
                "Close",
                "PoolText",
                "PillarText",
                "BedText",
                "FoundText",
                "CaughtText",
                "StrengthText",
            )
        },
        "unexpected new scripts",
    )
    for label in added | {WALKER}:
        require(
            room["scripts"][label] == after["map_scripts"][label],
            f"{label}: native split",
        )
        if label.endswith("Text"):
            pages = native_pages(room["scripts"][label])
            require(
                pages
                == canonical[label]
                == native_pages(room["script_text_bodies"][label]["commands"])
                and all(
                    len(pg) <= 2 and all(0 < len(s) <= 18 for s in pg) for pg in pages
                ),
                label,
            )
    programs = {WALKER} | {k for k in added if not k.endswith("Text")}
    families = [
        ("script_object_commands", {"faceplayer"}, "object_id"),
        ("script_flag_commands", {"checkevent"}, "flag_id"),
        (
            "script_control_commands",
            {"iffalse", "iftrue", "sjump", "end"},
            "target_label",
        ),
        (
            "script_text_commands",
            {"opentext", "writetext", "waitbutton", "closetext"},
            "text_label",
        ),
        ("script_item_checks", {"checkitem"}, "item_id"),
    ]
    projected = set()
    used = {row[0] for label in programs for row in instructions(room, label)}
    for table, codes, argument in families:
        projected.update(codes)
        expected = sorted(
            (label, index, op, args[0] if args else None)
            for label in programs
            for index, (op, *args) in enumerate(instructions(room, label))
            if op in codes
        )
        actual = sorted(
            (r["source_script"], r["command_index"], r["command"], r[argument])
            for r in room[table]
            if r["source_script"] in programs
        )
        require(
            expected == actual,
            f"{table}: guide needs exact native offsets, opcodes and operands",
        )
        if table == "script_control_commands":
            require(
                all(
                    r["resolved_target_script"] == r["target_label"]
                    for r in room[table]
                    if r["source_script"] in programs
                ),
                "local destinations must resolve to guide scripts",
            )
    require(used == projected, "new guide operation needs a native projection check")
    donor = after["maps"]["MnMinneapolisMillForemanHome"]
    (grant,) = [g for g in donor["script_item_grants"] if g["item_id"] == "HM_STRENGTH"]
    require(
        grant["source_script"] == "MN_MP_MILLFOREMANHOME_HOSTFieldHM",
        "Strength provenance",
    )
    foreman = instructions(donor, grant["source_script"])
    require(
        foreman.index(("checkflag", "ENGINE_PLAINBADGE"))
        < foreman.index(("verbosegiveitem", "HM_STRENGTH", "1")),
        "foreman's Current Badge requirement",
    )
    hm_checks = [
        (r["command_index"], r["item_id"])
        for r in room["script_item_checks"]
        if r["source_script"] == "MnJawGuideTravel"
    ]
    require(hm_checks == [(1, "HM_STRENGTH")], "typed HM check")
    for state in product([False, True], repeat=5):
        bits = {f for f, v in zip([*FLAGS, READY, CAUGHT], state) if v}
        expected = next(
            label
            for needed, label in (
                (state[4], "Caught"),
                (state[3], "Found"),
                (not state[0], "Pool"),
                (not state[1], "Pillar"),
                (True, "Bed"),
            )
            if needed
        )
        for hm in (False, True):
            keep, pages, actors = walk_program(room, WALKER, bits, hm)
            advice = ["MnJawGuide" + expected + "Text"]
            if not hm and not (state[3] or state[4]):
                advice.append("MnJawGuideStrengthText")
            require(
                keep == bits and pages == advice and not actors,
                f"guide from {state}/{hm}",
            )
    for order in permutations(range(3)):
        bits = set()
        for i, idx in enumerate(order):
            label = "MnJaw" + MARKS[idx][0].title()
            bits, pages, actors = walk_program(room, label, bits)
            require(
                FLAGS[idx] in bits
                and ((READY in bits) == (i == 2))
                and actors == (["MN_R03_JAW_CASTOROCK"] if i == 2 else []),
                "observation order/reveal",
            )
            same, _, again = walk_program(room, label, bits)
            require(same == bits and not again, "repeat search replayed reveal")


def stones(old, new):
    data, child = old["data"], new["data"]
    a, b = data["maps"][ROOM], child["maps"][ROOM]
    src, dst = old["runtime_files"], new["runtime_files"]
    k = "data/tilesets/cave_metatiles.bin"
    l, m = map(bytes, (src[k], dst[k]))
    n = len(l) // 16
    require(len(m) == len(l) + 48 and m.startswith(l), "original metatile art")
    blockkey = b["attributes"]["blocks_label"]
    require(
        list(base64.b64decode(child["map_blocks"][blockkey])) == b["blocks"],
        "native map block mirror",
    )
    require(
        all(
            data["maps"][name]["blocks"] == room["blocks"]
            for name, room in child["maps"].items()
            if name != ROOM and room["attributes"]["tileset_name"] == "cave"
        ),
        "other cave maps must retain original art",
    )
    bank, other = data["tilesets"]["cave"], child["tilesets"]["cave"]
    require(
        set(other["collision"])
        == set(bank["collision"]) | {f"{i:02x}" for i in range(n, n + 3)}
        and all(other["collision"][key] == v for key, v in bank["collision"].items()),
        "original movement recipe",
    )
    require(
        other["palette_map"]
        == json.loads(bytes(dst["data/tilesets/cave_palette_map.json"]))
        and other["collision"] == json.loads(bytes(dst["data/tilesets/cave.json"])),
        "native bank mirrors",
    )
    sk = "gfx/tilesets/cave.2bpp"
    ss, tt = map(bytes, (src[sk], dst[sk]))
    require(
        len(ss) == len(tt) == 1536
        and ss[:0x500] == tt[:0x500]
        and ss[0x5C0:] == tt[0x5C0:]
        and bytes(0xC0) == ss[0x500:0x5C0],
        "existing cave glyphs changed",
    )
    require(
        not (set(l) & set(range(0x50, 0x5C)))
        and all(
            other["palette_map"][i] == (5 if 0x50 <= i < 0x5C else v)
            for i, v in enumerate(bank["palette_map"])
        ),
        "shared cave palette",
    )
    art = SourceArt(old)
    result = SourceArt(new)
    for phase in ("morn", "day", "nite", "dark", "indoor"):
        former = art.blocks("cave", phase)
        now = result.blocks("cave", phase)
        require(
            all(x.tobytes() == y.tobytes() for x, y in zip(former, now[:n])),
            f"original cave art at {phase}",
        )
    marks = json.loads((HERE / "caves-marks.json").read_text())
    w = a["attributes"]["width"]
    changed = set()
    for i, (name, (x, y), _, _, prior, first) in enumerate(MARKS):
        at = (y // 2) * w + x // 2
        changed.add(at)
        require(
            (a["blocks"][at], b["blocks"][at]) == (prior, n + i),
            f"{name} map placement",
        )
        q = 8 * (y % 2) + 2 * (x % 2)
        patch = list(l[prior * 16 : prior * 16 + 16])
        for j, v in enumerate([q, q + 1, q + 4, q + 5]):
            patch[v] = first + j
        require(
            bytes(patch) == m[16 * (n + i) : 16 * (n + i + 1)]
            and other["collision"][f"{n + i:02x}"] == bank["collision"][f"{prior:02x}"],
            f"{name} footprint or adjoining land",
        )
        for dy, row in enumerate(marks[name]):
            for dx, c in enumerate(row):
                sub = first + (dx // 8) + 2 * (dy // 8)
                ti = 16 * sub + 2 * (dy % 8)
                bit = 7 - dx % 8
                v = (tt[ti] >> bit & 1) | ((tt[ti + 1] >> bit & 1) << 1)
                require(v == int(c), f"{name} native pixel")
    require(
        {i for i, (x, y) in enumerate(zip(a["blocks"], b["blocks"])) if x != y}
        == changed,
        "unmarked room cells",
    )

    def image(art, room):
        return art.render("cave", w, a["attributes"]["height"], room["blocks"], "nite")

    left, right = image(art, a), image(result, b)
    prior_counts = Counter(
        left.crop((x, y, x + 16, y + 16)).tobytes()
        for x in range(0, left.width, 16)
        for y in range(0, left.height, 16)
    )
    new_counts = Counter(
        right.crop((x, y, x + 16, y + 16)).tobytes()
        for x in range(0, right.width, 16)
        for y in range(0, right.height, 16)
    )
    for i, (_, (x, y), *_) in enumerate(MARKS):
        box = (16 * x, 16 * y, 16 * x + 16, 16 * y + 16)
        require(
            prior_counts[left.crop(box).tobytes()] == (8, 21, 21)[i]
            and new_counts[right.crop(box).tobytes()] == 1,
            "distinct native read site",
        )
    return left, right, n


def cameras(old, new, images, out):
    out.mkdir(parents=True, exist_ok=True)
    sheet = Image.new("RGB", (500, 338), "#232b30")
    d = ImageDraw.Draw(sheet)
    for row, (pack, im) in enumerate(zip((old, new), images)):
        sprites = npc_palette_pack(pack, "nite")
        r = pack["data"]["maps"][ROOM]
        for i, (tag, _, (px, py), facing, *_) in enumerate(MARKS):
            scene = im.convert("RGBA")
            for o in r["objects"]:
                if o["script"] == "MN_R03_JAW_CASTOROCK":
                    continue
                x = 15 if i and o["script"] == "MN_R03_JAW_BOULDER" else o["x"]
                scene.alpha_composite(
                    oriented_sprite(sprites, o["sprite"], "south"),
                    (x * 16, o["y"] * 16),
                )
            scene.alpha_composite(
                oriented_sprite(sprites, "SPRITE_CHRIS", facing), (px * 16, py * 16)
            )
            x = max(0, min(im.width - 160, px * 16 - 64))
            y = max(0, min(im.height - 144, py * 16 - 64))
            camera = scene.crop((x, y, x + 160, y + 144)).convert("RGB")
            c = ("Before", "After")[row]
            camera.save(out / f"{c.lower()}-{tag}.png")
            sheet.paste(camera, (3 + 166 * i, 20 + 167 * row))
            d.text((4 + 166 * i, 2 + 167 * row), f"{c} / {tag}", fill="white")
    sheet.save(out / "castorock-marks.png")


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("child", type=Path)
    cli.add_argument("--images", type=Path)
    args = cli.parse_args()
    old, new = map(load_pack, (args.parent, args.child))
    old_data, new_data = old["data"], new["data"]
    room = survey(new_data)
    movement(new_data, room)
    story(old_data, new_data)
    left, right, count = stones(old, new)
    if args.images:
        cameras(old, new, (left, right), args.images)
    print(
        f"Castorock: 64 guide states, six acknowledged clue orders and repeats, two eastward pushes; three unique 16×16 faces. All {count} previous cave blocks unchanged in five native palette groups."
    )


if __name__ == "__main__":
    main()
