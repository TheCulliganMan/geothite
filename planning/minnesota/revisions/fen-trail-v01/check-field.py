#!/usr/bin/env python3
"""Check the Field Margin investigation, its physical return and its four final approaches."""
from collections import deque
from pathlib import Path
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack


def check(pack):
    data = pack["data"]
    field, road = (data["maps"][name] for name in ("MnFieldMargin", "MnR16C01R00"))
    a = field["attributes"]
    col = data["tilesets"][a["tileset_name"]]["collision"]
    start = (16, 31)
    actor = (13, 5)
    solid = {(o["x"], o["y"]) for o in field["objects"]}
    walk = set()
    for y in range(a["height"] * 2):
        for x in range(a["width"] * 2):
            block = field["blocks"][(y // 2) * a["width"] + x // 2]
            if col[f"{block:02x}"][2*(y%2)+x%2] in ("FLOOR", "TALL_GRASS") and (x, y) not in solid:
                walk.add((x, y))

    def neighbors(at):
        x, y = at
        return {(x-1, y), (x+1, y), (x, y-1), (x, y+1)}

    reached = {start}
    todo = deque(reached)
    while todo:
        for tile in (neighbors(todo.popleft()) & walk) - reached:
            reached.add(tile)
            todo.append(tile)
    require(start in walk and reached == walk, "detached field path")
    require({(e["x"], e["y"]) for e in field["events"]["coord_events"]
             if e["script_name"] == "MnKensigardReveal" and e["scene_id"] == "-1"} == neighbors(actor)
            <= reached, "unreachable reveal")
    expected = {"Edge": {(8, 6), (9, 6), (8, 7), (9, 7)},
                "Boundary": {(8, 10), (9, 10), (8, 11), (9, 11)},
                "Soil": {(12, 8), (12, 9)}, "Grooves": {(14, 22), (14, 23)}}
    for name, faces in expected.items():
        require({(e["x"], e["y"]) for e in field["events"]["bg_events"]
                 if e["script"] == "MnKensigard" + name and e["event_type"] == "BGEVENT_READ"} == faces,
                f"{name} has the wrong search squares")
        for face in faces:
            require(face not in walk and neighbors(face) & reached, f"unreadable {name} at {face}")

    flag = "EVENT_MN_KENSIGARD_"
    texts = "MnKensigardText_"
    flags = set()

    def interact(room, label, page=None):
        nonlocal flags
        _, pages, flags = read_branch(room, "MnKensigard" + label, flags)
        require(pages == ([texts + page] if page else []), f"{label} yielded {pages}")

    interact(field, "Reveal")
    interact(road, "Caretaker", "caretaker_intro")
    interact(field, "Edge", "buried_edge")
    interact(field, "Soil", "soil")
    interact(field, "Grooves", "grooves_first")
    interact(field, "Grooves", "grooves_first")
    require(flag + "GROOVES_REVISIT_READY" not in flags, "re-reading counted as a return")
    interact(road, "Caretaker", "caretaker_partial")
    hint = native_pages(road["scripts"][texts + "caretaker_partial"])
    require("Step out here, then go back." in " ".join(w for page in hint for w in page), "physical return hint")
    require(instructions(field, "MnKensigardNewVisit")[:3] == [
        ("checkevent", flag + "GROOVES_FIRST_READ"), ("iffalse", "MnKensigardRestore"),
        ("setevent", flag + "GROOVES_REVISIT_READY")], "changed physical return callback")
    flags.add(flag + "GROOVES_REVISIT_READY")
    interact(field, "Grooves", "grooves_changed")
    interact(road, "Caretaker", "caretaker_comparison")
    interact(field, "Boundary", "boundary_marker")
    interact(field, "Reveal")
    interact(road, "Caretaker", "claim_corrected")
    interact(field, "Reveal", "reveal")
    interact(field, "Reveal")
    require(flag + "REVEALED" in flags and flag + "CAUGHT" not in flags, "unexpected encounter result")
    print(f"Field Margin: {len(reached)} connected walking squares, 12 readable faces, four reveal approaches, one physical revisit.")


if __name__ == "__main__":
    check(load_pack(Path(sys.argv[1])))
